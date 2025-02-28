mod add_sub;
mod bitops;
mod branches;
mod hex;
mod logical;
mod memory_controller;
mod mul;
mod transfer;

use bitfield::Bit;

use crate::{
    clock::Timestamp,
    events::{EventQueue, InternalEvent},
    module::{ActiveModule, Module, PinId, WireableModule},
    module_holder::PassiveModuleStore,
    module_id::ModuleAddress,
    pin_state::WireState,
    system_tables::SystemTables,
};

use super::{
    bit_helpers::bit_field_combined, io::IoController, regfile::RegisterFile, sreg::StatusRegister,
};

const SRAM_SIZE: usize = 8192;
const SRAM_END: u16 = 0x200 + SRAM_SIZE as u16 - 1;
const FLASH_SIZE: usize = 128 * 1024;
const RAMPZ_MASK: u8 = 0x3;
const EIND_MASK: u8 = 0x1;

#[derive(Debug)]
pub struct Mcu {
    reg_file: RegisterFile,
    pub io: IoController,
    sram: Vec<u8>,
    flash: Vec<u16>,

    pc: u32,
    sp: u16,
    sreg: StatusRegister,

    rampz: u8,
    eind: u8,

    halted: bool,

    queue: EventQueue,
}

impl Default for Mcu {
    fn default() -> Self {
        let (_, r) = kanal::bounded(0);
        let queue = EventQueue::new(SystemTables::new(), 1, 0, r);
        Mcu::new(queue)
    }
}

impl Mcu {
    pub fn new(mut queue: EventQueue) -> Self {
        Self {
            reg_file: RegisterFile::new(),
            io: IoController::new(queue.root_module_id(), &mut queue),
            sram: vec![0; SRAM_SIZE],
            flash: vec![0; FLASH_SIZE],

            pc: 0,
            sp: 0,
            rampz: 0,
            eind: 0,
            sreg: StatusRegister(0),
            halted: false,

            queue,
        }
    }
    pub fn step(&mut self) {
        self.queue.update(&mut self.io);

        if self.io.has_interrupt() && self.sreg.i() {
            if let Some(addr) = self.io.get_interrupt_address() {
                let ticks = self.execute_interrupt(addr);
                self.queue.clock.advance(ticks as i64);
            }
        }

        if self.halted {
            self.queue.skip_to_event();
        } else {
            let opcode: u16 = self.read_at_pc_offset(0);
            let ticks = self.execute(opcode);
            self.queue.clock.advance(ticks as i64);
        }
    }

    /// Executes an opcode and returns number of cycles.
    fn execute(&mut self, opcode: u16) -> u8 {
        let head = (opcode >> 8) as u8;
        match head {
            0x00 => {
                if opcode == 0x0000 {
                    self.pc += 1; // NOP
                    1
                } else {
                    panic!("Reserved")
                }
            }
            0x01 => self.instr_movw(opcode),
            0x02 => self.instr_muls(opcode),
            0x03 => {
                let bits = bit_field_combined(opcode, &[7..=7, 3..=3]);
                match bits {
                    0b00 => self.instr_mulsu(opcode),
                    0b01 => self.instr_fmul(opcode),
                    0b10 => self.instr_fmuls(opcode),
                    0b11 => self.instr_fmulsu(opcode),
                    _ => panic!("2-bit field impossible value"),
                }
            }
            0x04..=0x07 => self.instr_cpc(opcode),
            0x08..=0x0B => self.instr_sbc(opcode),
            0x0C..=0x0F => self.instr_add(opcode),
            0x10..=0x13 => self.instr_cpse(opcode),
            0x14..=0x17 => self.instr_cp(opcode),
            0x18..=0x1B => self.instr_sub(opcode),
            0x1C..=0x1F => self.instr_adc(opcode),

            0x20..=0x23 => self.instr_and(opcode),
            0x24..=0x27 => self.instr_eor(opcode),
            0x28..=0x2B => self.instr_or(opcode),
            0x2C..=0x2F => self.instr_mov(opcode),

            0x30..=0x3F => self.instr_cpi(opcode),
            0x40..=0x4F => self.instr_sbci(opcode),
            0x50..=0x5F => self.instr_subi(opcode),
            0x60..=0x6F => self.instr_ori(opcode),
            0x70..=0x7F => self.instr_andi(opcode),

            0x80..=0x8F | 0xA0..=0xAF => {
                if head.bit(1) {
                    self.instr_std(opcode)
                } else {
                    self.instr_ldd(opcode)
                }
            }

            0x90 | 0x91 => {
                let tail = opcode & 0x000F;
                match tail {
                    0x0 => self.instr_lds(opcode),
                    0x1 | 0x2 | 0x9 | 0xA | 0xC..=0xE => self.instr_ld(opcode),
                    0x4 | 0x5 => self.instr_lpm(opcode),
                    0x6 | 0x7 => self.instr_elpm(opcode),
                    0xF => self.instr_pop(opcode),
                    0x3 | 0x8 | 0xB => panic!("Reserved"),
                    _ => panic!("Impossible for 4-bit value"),
                }
            }

            0x92 | 0x93 => {
                let tail = opcode & 0x000F;
                match tail {
                    0x0 => self.instr_sts(opcode),
                    0x1 | 0x2 | 0x9 | 0xA | 0xC..=0xE => self.instr_st(opcode),
                    0xF => self.instr_push(opcode),
                    0x3..=0x8 | 0xB => panic!("Reserved"),
                    _ => panic!("Impossible for 4-bit value"),
                }
            }

            0x94 | 0x95 => {
                let tail = opcode & 0x000F;
                match tail {
                    0x0 => self.instr_com(opcode),
                    0x1 => self.instr_neg(opcode),
                    0x2 => self.instr_swap(opcode),
                    0x3 => self.instr_inc(opcode),
                    0x4 => panic!("Reserved"),
                    0x5 => self.instr_asr(opcode),
                    0x6 => self.instr_lsr(opcode),
                    0x7 => self.instr_ror(opcode),

                    0x8 => {
                        if head == 0x94 {
                            if opcode.bit(7) {
                                self.instr_bclr(opcode)
                            } else {
                                self.instr_bset(opcode)
                            }
                        } else {
                            match opcode {
                                0x9508 => self.instr_ret(opcode),
                                0x9518 => self.instr_reti(opcode),
                                0x9588 => todo!(),
                                0x9598 => todo!(),
                                0x95A8 => todo!(),
                                0x95C8 => self.instr_lpm(opcode),
                                0x95D8 => self.instr_elpm(opcode),
                                0x95E8 => self.instr_spm(opcode),
                                _ => panic!("Reserved"),
                            }
                        }
                    }
                    0x9 => match opcode {
                        0x9409 => self.instr_ijmp(opcode),
                        0x9419 => self.instr_eijmp(opcode),
                        0x9509 => self.instr_icall(opcode),
                        0x9519 => self.instr_eicall(opcode),
                        _ => panic!("Reserved"),
                    },
                    0xA => self.instr_dec(opcode),
                    0xB => panic!("Reserved"),
                    0xC | 0xD => self.instr_jmp(opcode),
                    0xE | 0xF => self.instr_call(opcode),
                    _ => panic!("Impossible for 4-bit value"),
                }
            }

            0x96 => self.instr_adiw(opcode),
            0x97 => self.instr_sbiw(opcode),
            0x98 => self.instr_cbi(opcode),
            0x99 => self.instr_sbic(opcode),
            0x9A => self.instr_sbi(opcode),
            0x9B => self.instr_sbis(opcode),
            0x9C..=0x9F => self.instr_mul(opcode),

            0xB0..=0xB7 => self.instr_in(opcode),
            0xB8..=0xBF => self.instr_out(opcode),

            0xC0..=0xCF => self.instr_rjmp(opcode),
            0xD0..=0xDF => self.instr_rcall(opcode),
            0xE0..=0xEF => self.instr_ldi(opcode),

            0xF0..=0xF3 => self.instr_brbs(opcode),
            0xF4..=0xF7 => self.instr_brbc(opcode),
            0xF8..=0xF9 => self.instr_bld(opcode),
            0xFA..=0xFB => self.instr_bst(opcode),
            0xFC..=0xFD => self.instr_sbrc(opcode),
            0xFE..=0xFF => self.instr_sbrs(opcode),
        }
    }

    /// Loads flash memory from a slice
    pub fn load_flash(&mut self, data: &[u16]) {
        for (addr, &val) in data.iter().enumerate() {
            self.write_flash(addr as u32, val);
        }
    }
}

impl Module for Mcu {
    #[inline]
    fn address(&self) -> ModuleAddress {
        self.io.address()
    }

    fn handle_event(&mut self, event: InternalEvent, queue: &mut EventQueue, t: Timestamp) {
        self.io.handle_event(event, queue, t)
    }

    fn find(&self, address: ModuleAddress) -> Option<&dyn Module> {
        if address.is_empty() {
            Some(self)
        } else {
            self.io.find(address)
        }
    }

    fn find_mut(&mut self, address: ModuleAddress) -> Option<&mut dyn Module> {
        self.io.find_mut(address)
    }

    fn to_wireable_mut(&mut self) -> Option<&mut dyn WireableModule> {
        Some(self)
    }
    fn to_wireable(&self) -> Option<&dyn WireableModule> {
        Some(self)
    }
}

impl ActiveModule for Mcu {
    fn run_until_time(&mut self, t: Timestamp) -> Timestamp {
        while self.queue.clock.current_time() < t {
            self.step();
            if self.halted && self.queue.is_empty() {
                break;
            }
        }
        self.queue.clock.current_time()
    }

    fn module_store(&mut self) -> &mut PassiveModuleStore {
        &mut self.io.module_store
    }

    fn event_queue(&self) -> &EventQueue {
        &self.queue
    }
}

impl WireableModule for Mcu {
    #[inline]
    fn get_pin(&self, queue: &EventQueue, id: PinId) -> WireState {
        self.io.get_pin(queue, id)
    }

    #[inline]
    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState) {
        self.io.set_pin(queue, id, data)
    }
}

#[cfg(test)]
mod test_helper {
    use crate::components::avr::sreg::test_helper::assert_sreg;

    use super::Mcu;

    impl Mcu {
        /// Helper test function, for executing an instruction and checking the correct [StatusRegister] change.
        pub fn execute_and_assert_sreg(&mut self, opcode: u16, sreg_mask: &'static str) {
            let sreg_initial = self.sreg;
            self.execute(opcode);
            assert_sreg(&mut self.sreg, &sreg_initial, sreg_mask);
        }
    }
}
