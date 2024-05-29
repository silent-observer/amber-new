use bitfield::Bit;
use iced_x86::code_asm::*;
use memoffset::offset_of;

use crate::components::avr::mcu::{jit::instructions::AvrInstructionType, Mcu};

use super::{assembly::*, instructions::Inst};

impl JitAssembler {
    pub fn avr_rjmp(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Rjmp);
        let k = i.fields[0];
        let addr = if k.bit(11) {
            let k = (k ^ 0x0FFF) + 1;
            i.address - (k as usize) + 1
        } else {
            i.address + (k as usize) + 1
        };
        if addr == i.address {
            const HALTED_OFFSET: usize = offset_of!(Mcu, halted);
            self.a.mov(byte_ptr(mcu_reg + HALTED_OFFSET), 1)?;
            self.a.inc(current_tick)?;
            self.a.inc(current_tick)?;
            return self.jump_to_epilogue(0);
        }

        self.a.mov(avr_pc, addr as u32)?;
        if let Some(label) = self.addr_labels.get(&addr) {
            self.a.inc(current_tick)?;
            self.a.inc(current_tick)?;
            self.a.jmp(*label)
        } else {
            self.a.inc(current_tick)?;
            self.a.inc(current_tick)?;
            self.jump_to_epilogue(0)
        }
    }

    pub fn avr_ijmp(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ijmp);
        self.a.mov(avr_pc_low, avr_reg(Z_REG))?;
        self.a.mov(avr_pc_high, avr_reg(Z_REG + 1))?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.jump_to_epilogue(0)
    }

    pub fn avr_eijmp(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Eijmp);
        self.a.mov(avr_pc_low, avr_eind())?;
        self.a.shl(avr_pc, 16)?;

        self.a.mov(avr_pc_low, avr_reg(Z_REG))?;
        self.a.mov(avr_pc_high, avr_reg(Z_REG + 1))?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.jump_to_epilogue(0)
    }

    pub fn avr_jmp(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Jmp);
        let addr_high = i.fields[0];
        let addr_low = i.fields[1];
        let addr = ((addr_high as u32) << 16) | (addr_low as u32);

        self.a.mov(avr_pc, addr)?;
        self.a.add(current_tick, 3)?;
        self.jump_to_epilogue(0)
    }

    fn push_pc(&mut self) -> Result<(), IcedError> {
        self.a.mov(r11w, avr_sp())?;
        self.a.movzx(rax, r11w)?;
        self.a.mov(ebx, avr_pc)?;
        self.a.inc(ebx)?;

        self.checked_write_memory()?;

        self.a.dec(r11w)?;
        self.a.movzx(rax, r11w)?;
        self.a.shr(ebx, 8)?;
        self.checked_write_memory()?;

        self.a.dec(r11w)?;
        self.a.movzx(rax, r11w)?;
        self.a.shr(ebx, 8)?;
        self.checked_write_memory()?;

        self.a.dec(r11w)?;
        self.a.mov(avr_sp(), r11w)
    }

    pub fn avr_rcall(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Rcall);
        self.push_pc()?;

        let k = i.fields[0];
        let addr = if k.bit(11) {
            let k = (k ^ 0x0FFF) + 1;
            i.address - (k as usize) + 1
        } else {
            i.address + (k as usize) + 1
        };
        if addr == i.address {
            const HALTED_OFFSET: usize = offset_of!(Mcu, halted);
            self.a.mov(byte_ptr(HALTED_OFFSET), 1)?;
            self.a.add(current_tick, 4)?;
            return self.jump_to_epilogue(0);
        }

        if let Some(label) = self.addr_labels.get(&addr) {
            self.a.add(current_tick, 4)?;
            self.a.jmp(*label)
        } else {
            self.a.mov(avr_pc, addr as u32)?;
            self.a.add(current_tick, 4)?;
            self.jump_to_epilogue(0)
        }
    }

    pub fn avr_icall(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Icall);
        self.push_pc()?;

        self.a.mov(avr_pc_low, avr_reg(Z_REG))?;
        self.a.mov(avr_pc_high, avr_reg(Z_REG + 1))?;
        self.a.add(current_tick, 4)?;
        self.jump_to_epilogue(0)
    }

    pub fn avr_eicall(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Eicall);
        self.push_pc()?;

        self.a.mov(avr_pc_low, avr_eind())?;
        self.a.shl(avr_pc, 16)?;

        self.a.mov(avr_pc_low, avr_reg(Z_REG))?;
        self.a.mov(avr_pc_high, avr_reg(Z_REG + 1))?;
        self.a.add(current_tick, 4)?;
        self.jump_to_epilogue(0)
    }

    pub fn avr_call(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Call);
        self.a.inc(avr_pc)?;
        self.push_pc()?;

        let addr_high = i.fields[0];
        let addr_low = i.fields[1];
        let addr = ((addr_high as u32) << 16) | (addr_low as u32);

        self.a.mov(avr_pc, addr)?;
        self.a.add(current_tick, 5)?;
        self.jump_to_epilogue(0)
    }

    fn avr_ret_helper(&mut self, i: Inst) -> Result<(), IcedError> {
        self.a.mov(r11w, avr_sp())?;
        self.a.inc(r11w)?;
        self.a.movzx(rax, r11w)?;
        let v1 = self.checked_read_memory()?;
        self.a.movzx(avr_pc, v1)?;

        self.a.inc(r11w)?;
        self.a.movzx(rax, r11w)?;
        let v2 = self.checked_read_memory()?;
        self.a.shl(avr_pc, 8)?;
        self.a.or(avr_pc_low, v2)?;

        self.a.inc(r11w)?;
        self.a.movzx(rax, r11w)?;
        let v3 = self.checked_read_memory()?;
        self.a.shl(avr_pc, 8)?;
        self.a.or(avr_pc_low, v3)?;

        self.a.inc(r11w)?;
        self.a.mov(avr_sp(), r11w)?;
        self.a.add(current_tick, 5)?;
        self.jump_to_epilogue(0)
    }

    pub fn avr_ret(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ret);
        self.avr_ret_helper(i)
    }

    pub fn avr_reti(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Reti);
        self.a.bts(avr_sreg16, 7)?;
        self.avr_ret_helper(i)
    }

    pub fn avr_cpse(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Cpse);
        let r = i.fields[0];
        let d = i.fields[1];

        self.a.mov(al, avr_reg(d))?;
        self.a.sub(al, avr_reg(r))?;

        let skip_addr = i.address + i.fields[2] as usize;
        let skip_label = self.addr_labels.get(&skip_addr).expect("missing label");

        self.a.inc(avr_pc)?;
        self.a.mov(eax, skip_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovz(avr_pc, eax)?;
        self.a.jz(*skip_label)?;
        self.a.dec(current_tick)
    }

    pub fn avr_sbrc(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbrc);
        let d = i.fields[0];
        let b = i.fields[1];

        self.a.movzx(ax, avr_reg(d))?;
        self.a.bt(ax, b as u32)?;

        let skip_addr = i.address + i.fields[2] as usize;
        let skip_label = self.addr_labels.get(&skip_addr).expect("missing label");

        self.a.inc(avr_pc)?;
        self.a.mov(eax, skip_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovnc(avr_pc, eax)?;
        self.a.jnc(*skip_label)?;
        self.a.dec(current_tick)
    }

    pub fn avr_sbrs(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbrs);
        let d = i.fields[0];
        let b = i.fields[1];

        self.a.movzx(ax, avr_reg(d))?;
        self.a.bt(ax, b as u32)?;

        let skip_addr = i.address + i.fields[2] as usize;
        let skip_label = self.addr_labels.get(&skip_addr).expect("missing label");

        self.a.inc(avr_pc)?;
        self.a.mov(eax, skip_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovc(avr_pc, eax)?;
        self.a.jc(*skip_label)?;
        self.a.dec(current_tick)
    }

    pub fn avr_sbic(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbic);
        let io = i.fields[0];
        let b = i.fields[1];

        let val = self.read_io(io as u8)?;
        assert_eq!(val, al);
        self.a.bt(ax, b as u32)?;

        let skip_addr = i.address + i.fields[2] as usize;
        let skip_label = self.addr_labels.get(&skip_addr).expect("missing label");

        self.a.inc(avr_pc)?;
        self.a.mov(eax, skip_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovnc(avr_pc, eax)?;
        self.a.jnc(*skip_label)?;
        self.a.dec(current_tick)
    }

    pub fn avr_sbis(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbis);
        let io = i.fields[0];
        let b = i.fields[1];

        let val = self.read_io(io as u8)?;
        assert_eq!(val, al);
        self.a.bt(ax, b as u32)?;

        let skip_addr = i.address + i.fields[2] as usize;
        let skip_label = self.addr_labels.get(&skip_addr).expect("missing label");

        self.a.inc(avr_pc)?;
        self.a.mov(eax, skip_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovc(avr_pc, eax)?;
        self.a.jnc(*skip_label)?;
        self.a.dec(current_tick)
    }

    pub fn avr_brbc(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Brbc);
        let k = i.fields[0];
        let s = i.fields[1];

        let jump_addr = if k.bit(6) {
            let k = (k ^ 0x007F) + 1; // Negation
            i.address - (k as usize) + 1
        } else {
            i.address + (k as usize) + 1
        };
        let jump_label = self.addr_labels.get(&jump_addr).expect("missing label");

        self.a.bt(avr_sreg16, s as u32)?;

        self.a.inc(avr_pc)?;
        self.a.mov(eax, jump_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovnc(avr_pc, eax)?;
        self.a.jnc(*jump_label)?;
        self.a.dec(current_tick)
    }

    pub fn avr_brbs(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Brbs);
        let k = i.fields[0];
        let s = i.fields[1];

        let jump_addr = if k.bit(6) {
            let k = (k ^ 0x007F) + 1; // Negation
            i.address - (k as usize) + 1
        } else {
            i.address + (k as usize) + 1
        };
        let jump_label = self.addr_labels.get(&jump_addr).expect("missing label");

        self.a.bt(avr_sreg16, s as u32)?;

        self.a.inc(avr_pc)?;
        self.a.mov(eax, jump_addr as u32)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)?;
        self.a.cmovc(avr_pc, eax)?;
        self.a.jc(*jump_label)?;
        self.a.dec(current_tick)
    }
}
