use std::collections::HashMap;

use iced_x86::{
    code_asm::{asm_traits::CodeAsmMov, *},
    MemoryOperand,
};
use memoffset::offset_of;

use crate::{
    clock::Clock,
    components::avr::{io::IoController, mcu::Mcu},
    events::EventQueue,
};

use super::instructions::{AvrInstructionType, Inst};

// pub const cpu_ptr =

// Instruction block signature:
// uint8_t block(Mcu* mcu_ptr, uint8_t* sram_ptr, uint16_t* flash_ptr)
// (mcu_ptr -> rdi)
// (sram_ptr -> rsi)
// (flash_ptr -> rdx)
//
// Prologue assignments:
// int64_t* next_interrupt -> r10
// int64_t current_tick -> r8
// uint32_t pc -> ecx
// uint8_t sreg -> r9b

const NEXT_INTERRUPT_OFFSET: usize =
    offset_of!(Mcu, queue) + offset_of!(EventQueue, next_interrupt);
const CURRENT_TICK_OFFSET: usize =
    offset_of!(Mcu, queue) + offset_of!(EventQueue, clock) + offset_of!(Clock, current_tick);
const PC_OFFSET: usize = offset_of!(Mcu, pc);
const SP_OFFSET: usize = offset_of!(Mcu, sp);
const EIND_OFFSET: usize = offset_of!(Mcu, eind);
const RAMPZ_OFFSET: usize = offset_of!(Mcu, rampz);
const SREG_OFFSET: usize = offset_of!(Mcu, sreg);
const REGISTERS_OFFSET: usize = offset_of!(Mcu, reg_file);

pub const mcu_reg: AsmRegister64 = rdi;
pub const sram_reg: AsmRegister64 = rsi;
pub const flash_reg: AsmRegister64 = rdx;
pub const avr_sreg: AsmRegister8 = r9b;
pub const avr_sreg16: AsmRegister16 = r9w;
pub const current_tick: AsmRegister64 = r8;
pub const next_interrupt: AsmRegister64 = r10;

pub const avr_pc: AsmRegister32 = ecx;
pub const avr_pc_low: AsmRegister8 = cl;
pub const avr_pc_high: AsmRegister8 = ch;

pub const X_REG: u16 = 26;
pub const Y_REG: u16 = 28;
pub const Z_REG: u16 = 30;

pub fn avr_reg(i: u16) -> AsmMemoryOperand {
    byte_ptr(mcu_reg + REGISTERS_OFFSET + i)
}
pub fn avr_sp() -> AsmMemoryOperand {
    word_ptr(mcu_reg + SP_OFFSET)
}
pub fn avr_eind() -> AsmMemoryOperand {
    byte_ptr(mcu_reg + EIND_OFFSET)
}
pub fn avr_rampz() -> AsmMemoryOperand {
    byte_ptr(mcu_reg + RAMPZ_OFFSET)
}

pub struct JitAssembler {
    pub a: CodeAssembler,
    pub addr_labels: HashMap<usize, CodeLabel>,

    epilogue_label: Option<CodeLabel>,
}

impl JitAssembler {
    pub fn new() -> Self {
        Self {
            a: CodeAssembler::new(64).unwrap(),
            addr_labels: HashMap::new(),
            epilogue_label: None,
        }
    }

    pub fn create_addr_label(&mut self, i: &Inst) {
        self.addr_labels
            .insert(i.address as usize, self.a.create_label());
    }

    pub fn restore_registers(&mut self) -> Result<(), IcedError> {
        self.a
            .lea(next_interrupt, mcu_reg + NEXT_INTERRUPT_OFFSET)?;
        self.a
            .mov(current_tick, qword_ptr(mcu_reg + CURRENT_TICK_OFFSET))?;
        self.a.mov(avr_pc, dword_ptr(mcu_reg + PC_OFFSET))?;
        self.a.mov(avr_sreg, byte_ptr(mcu_reg + SREG_OFFSET))
    }

    pub fn save_registers(&mut self) -> Result<(), IcedError> {
        self.a.mov(dword_ptr(mcu_reg + PC_OFFSET), avr_pc)?;
        self.a.mov(byte_ptr(mcu_reg + SREG_OFFSET), avr_sreg)?;
        self.a
            .mov(qword_ptr(mcu_reg + CURRENT_TICK_OFFSET), current_tick)
    }

    pub fn prologue(&mut self) -> Result<(), IcedError> {
        self.a.push(rbx)?;
        self.a.sub(rsp, 8)?;
        self.restore_registers()?;
        self.epilogue_label = Some(self.a.create_label());
        Ok(())
    }

    pub fn jump_to_epilogue(&mut self, code: u8) -> Result<(), IcedError> {
        let mut label = self.epilogue_label.expect("No epilogue label!");
        if code == 0 {
            self.a.xor(rax, rax)?;
        } else {
            self.a.mov(ax, code as u32)?;
        }
        self.a.jmp(label)
    }

    pub fn epilogue(&mut self) -> Result<(), IcedError> {
        let mut label = self.epilogue_label.expect("No epilogue label!");
        self.a.xor(rax, rax)?;
        self.a.set_label(&mut label)?;
        self.save_registers()?;
        self.a.add(rsp, 8)?;
        self.a.pop(rbx)?;
        self.a.ret()
    }

    pub fn check_interrupts(&mut self) -> Result<(), IcedError> {
        self.a.xor(rax, rax)?;
        self.a.cmp(qword_ptr(next_interrupt), current_tick)?;
        self.a.je(self.epilogue_label.unwrap())
    }

    extern "C" fn read_memory_helper(m: &mut Mcu, addr: u16) -> u8 {
        let result = m.read(addr);
        m.queue.update(&mut m.io);
        result
    }

    extern "C" fn write_memory_helper(m: &mut Mcu, addr: u16, data: u8) {
        m.write(addr, data);
        m.queue.update(&mut m.io);
    }

    pub fn read_memory<Addr>(&mut self, addr_in: Addr) -> Result<AsmRegister8, IcedError>
    where
        CodeAssembler: CodeAsmMov<AsmRegister16, Addr>,
    {
        self.save_registers()?;
        self.a.push(rdi)?;
        self.a.push(rsi)?;
        self.a.push(rdx)?;

        self.a.mov(si, addr_in)?;
        let f = Self::read_memory_helper as *const () as u64;
        self.a.call(f)?;
        self.a.pop(rdx)?;
        self.a.pop(rsi)?;
        self.a.pop(rdi)?;
        self.restore_registers()?;

        Ok(al)
    }

    pub fn write_memory<Addr, Data>(
        &mut self,
        addr_in: Addr,
        data_in: Data,
    ) -> Result<(), IcedError>
    where
        CodeAssembler: CodeAsmMov<AsmRegister16, Addr>,
        CodeAssembler: CodeAsmMov<AsmRegister8, Data>,
    {
        self.save_registers()?;
        self.a.push(rdi)?;
        self.a.push(rsi)?;
        self.a.push(rdx)?;

        self.a.mov(si, addr_in)?;
        self.a.mov(dl, data_in)?;
        let f = Self::write_memory_helper as *const () as u64;
        self.a.call(f)?;
        self.a.pop(rdx)?;
        self.a.pop(rsi)?;
        self.a.pop(rdi)?;
        self.restore_registers()
    }

    extern "C" fn read_io_helper(m: &mut Mcu, addr: u8) -> u8 {
        let result = m.read_io(addr);
        m.queue.update(&mut m.io);
        result
    }

    extern "C" fn write_io_helper(m: &mut Mcu, addr: u8, data: u8) {
        m.write_io(addr, data);
        m.queue.update(&mut m.io);
    }

    pub fn read_io(&mut self, io: u8) -> Result<AsmRegister8, IcedError> {
        self.save_registers()?;
        self.a.push(rdi)?;
        self.a.push(rsi)?;
        self.a.push(rdx)?;

        self.a.mov(sil, io as u32)?;
        let f = Self::read_io_helper as *const () as u64;
        self.a.call(f)?;
        self.a.pop(rdx)?;
        self.a.pop(rsi)?;
        self.a.pop(rdi)?;
        self.restore_registers()?;

        Ok(al)
    }

    pub fn write_io<Addr, Data>(&mut self, io: Addr, data_in: Data) -> Result<(), IcedError>
    where
        CodeAssembler: CodeAsmMov<AsmRegister8, Data>,
        CodeAssembler: CodeAsmMov<AsmRegister8, Addr>,
    {
        self.save_registers()?;
        self.a.push(rdi)?;
        self.a.push(rsi)?;
        self.a.push(rdx)?;

        self.a.mov(sil, io)?;
        self.a.mov(dl, data_in)?;
        let f = Self::write_io_helper as *const () as u64;
        self.a.call(f)?;
        self.a.pop(rdx)?;
        self.a.pop(rsi)?;
        self.a.pop(rdi)?;
        self.restore_registers()
    }

    // change_func context:
    // Mcu* mcu_reg -> rdi
    // uint8_t io_result -> al
    pub fn change_io(
        &mut self,
        io: u8,
        change_func: impl FnOnce(&mut Self) -> Result<(), IcedError>,
    ) -> Result<(), IcedError> {
        self.save_registers()?;
        self.a.push(rsi)?;
        self.a.push(rdx)?;
        self.a.push(rdi)?;

        self.a.mov(sil, io as u32)?;
        let f_read = Self::read_io_helper as *const () as u64;
        self.a.call(f_read)?;

        self.a.pop(rdi)?;
        change_func(self)?;
        self.a.push(rdi)?;

        self.a.mov(sil, io as u32)?;
        self.a.mov(dl, al)?;
        let f_write = Self::write_io_helper as *const () as u64;
        self.a.call(f_write)?;

        self.a.pop(rdi)?;
        self.a.pop(rdx)?;
        self.a.pop(rsi)?;
        self.restore_registers()
    }

    /// Address in rax, return in al
    pub fn checked_read_memory(&mut self) -> Result<AsmRegister8, IcedError> {
        self.a.sub(ax, 0x200)?;
        let mut skip_label = self.a.create_label();
        self.a.ja(skip_label)?;
        self.jump_to_epilogue(1)?;
        self.a.set_label(&mut skip_label)?;

        self.a.add(rax, sram_reg)?;
        self.a.mov(al, byte_ptr(rax))?;

        Ok(al)
    }

    /// Address in rax, data in bl
    pub fn checked_write_memory(&mut self) -> Result<(), IcedError> {
        self.a.sub(ax, 0x200)?;
        let mut skip_label = self.a.create_label();
        self.a.ja(skip_label)?;
        self.jump_to_epilogue(1)?;
        self.a.set_label(&mut skip_label)?;

        self.a.add(rax, sram_reg)?;
        self.a.mov(byte_ptr(rax), bl)
    }

    pub fn instr(&mut self, instr: Inst) -> Result<(), IcedError> {
        let l = self
            .addr_labels
            .get_mut(&instr.address)
            .expect("Couldn't find label");
        self.a.set_label(l)?;
        self.check_interrupts()?;
        match instr.kind {
            AvrInstructionType::Nop => self.a.inc(avr_pc),
            AvrInstructionType::Movw => self.avr_movw(instr),
            AvrInstructionType::Muls => self.avr_muls(instr),
            AvrInstructionType::Mulsu => self.avr_mulsu(instr),
            AvrInstructionType::Fmul => self.avr_fmul(instr),
            AvrInstructionType::Fmuls => self.avr_fmuls(instr),
            AvrInstructionType::Fmulsu => self.avr_fmulsu(instr),
            AvrInstructionType::Cpc => self.avr_cpc(instr),
            AvrInstructionType::Sbc => self.avr_sbc(instr),
            AvrInstructionType::Add => self.avr_add(instr),
            AvrInstructionType::Cpse => self.avr_cpse(instr),
            AvrInstructionType::Cp => self.avr_cp(instr),
            AvrInstructionType::Sub => self.avr_sub(instr),
            AvrInstructionType::Adc => self.avr_adc(instr),
            AvrInstructionType::And => self.avr_and(instr),
            AvrInstructionType::Eor => self.avr_eor(instr),
            AvrInstructionType::Or => self.avr_or(instr),
            AvrInstructionType::Mov => self.avr_mov(instr),
            AvrInstructionType::Cpi => self.avr_cpi(instr),
            AvrInstructionType::Sbci => self.avr_sbci(instr),
            AvrInstructionType::Subi => self.avr_subi(instr),
            AvrInstructionType::Ori => self.avr_ori(instr),
            AvrInstructionType::Andi => self.avr_andi(instr),
            AvrInstructionType::Std => self.avr_std(instr),
            AvrInstructionType::Ldd => self.avr_ldd(instr),
            AvrInstructionType::Lds => self.avr_lds(instr),
            AvrInstructionType::Ld => self.avr_ld(instr),
            AvrInstructionType::Pop => self.avr_pop(instr),
            AvrInstructionType::Sts => self.avr_sts(instr),
            AvrInstructionType::St => self.avr_st(instr),
            AvrInstructionType::Push => self.avr_push(instr),
            AvrInstructionType::Com => self.avr_com(instr),
            AvrInstructionType::Neg => self.avr_neg(instr),
            AvrInstructionType::Swap => self.avr_swap(instr),
            AvrInstructionType::Inc => self.avr_inc(instr),
            AvrInstructionType::Asr => self.avr_asr(instr),
            AvrInstructionType::Lsr => self.avr_lsr(instr),
            AvrInstructionType::Ror => self.avr_ror(instr),
            AvrInstructionType::Bclr => self.avr_bclr(instr),
            AvrInstructionType::Bset => self.avr_bset(instr),
            AvrInstructionType::Ret => self.avr_ret(instr),
            AvrInstructionType::Reti => self.avr_reti(instr),
            AvrInstructionType::Lpm => self.avr_lpm(instr),
            AvrInstructionType::Elpm => self.avr_elpm(instr),
            AvrInstructionType::Spm => todo!(),
            AvrInstructionType::Ijmp => self.avr_ijmp(instr),
            AvrInstructionType::Eijmp => self.avr_eijmp(instr),
            AvrInstructionType::Icall => self.avr_icall(instr),
            AvrInstructionType::Eicall => self.avr_eicall(instr),
            AvrInstructionType::Dec => self.avr_dec(instr),
            AvrInstructionType::Jmp => self.avr_jmp(instr),
            AvrInstructionType::Call => self.avr_call(instr),
            AvrInstructionType::Adiw => self.avr_adiw(instr),
            AvrInstructionType::Sbiw => self.avr_sbiw(instr),
            AvrInstructionType::Cbi => self.avr_cbi(instr),
            AvrInstructionType::Sbic => self.avr_sbic(instr),
            AvrInstructionType::Sbi => self.avr_sbi(instr),
            AvrInstructionType::Sbis => self.avr_sbis(instr),
            AvrInstructionType::Mul => self.avr_mul(instr),
            AvrInstructionType::In => self.avr_in(instr),
            AvrInstructionType::Out => self.avr_out(instr),
            AvrInstructionType::Rjmp => self.avr_rjmp(instr),
            AvrInstructionType::Rcall => self.avr_rcall(instr),
            AvrInstructionType::Ldi => self.avr_ldi(instr),
            AvrInstructionType::Brbs => self.avr_brbs(instr),
            AvrInstructionType::Brbc => self.avr_brbc(instr),
            AvrInstructionType::Bld => self.avr_bld(instr),
            AvrInstructionType::Bst => self.avr_bst(instr),
            AvrInstructionType::Sbrc => self.avr_sbrc(instr),
            AvrInstructionType::Sbrs => self.avr_sbrs(instr),
        }
    }
}
