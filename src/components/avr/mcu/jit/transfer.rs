use iced_x86::code_asm::*;

use crate::components::avr::mcu::jit::instructions::AvrInstructionType;

use super::{assembly::*, instructions::Inst};

impl JitAssembler {
    pub fn avr_mov(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Mov);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_movw(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Movw);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(r))?;
        self.a.mov(ah, avr_reg(r + 1))?;
        self.a.mov(avr_reg(d), al)?;
        self.a.mov(avr_reg(d + 1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_ldi(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ldi);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(avr_reg(d), k as u32)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_ld(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ld);
        let d = i.fields[0];
        let t = i.fields[1];

        match t {
            0b0001 => {
                self.a.mov(al, avr_reg(Z_REG))?;
                self.a.mov(ah, avr_reg(Z_REG + 1))?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
                self.a.add(avr_reg(Z_REG), 1)?;
                self.a.adc(avr_reg(Z_REG + 1), 0)?;
            }
            0b0010 => {
                self.a.mov(al, avr_reg(Z_REG))?;
                self.a.mov(ah, avr_reg(Z_REG + 1))?;
                self.a.dec(ax)?;
                self.a.mov(avr_reg(Z_REG), al)?;
                self.a.mov(avr_reg(Z_REG + 1), ah)?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
            }

            0b1001 => {
                self.a.mov(al, avr_reg(Y_REG))?;
                self.a.mov(ah, avr_reg(Y_REG + 1))?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
                self.a.add(avr_reg(Y_REG), 1)?;
                self.a.adc(avr_reg(Y_REG + 1), 0)?;
            }
            0b1010 => {
                self.a.mov(al, avr_reg(Y_REG))?;
                self.a.mov(ah, avr_reg(Y_REG + 1))?;
                self.a.dec(ax)?;
                self.a.mov(avr_reg(Y_REG), al)?;
                self.a.mov(avr_reg(Y_REG + 1), ah)?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
            }

            0b1100 => {
                self.a.mov(al, avr_reg(X_REG))?;
                self.a.mov(ah, avr_reg(X_REG + 1))?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
            }
            0b1101 => {
                self.a.mov(al, avr_reg(X_REG))?;
                self.a.mov(ah, avr_reg(X_REG + 1))?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
                self.a.add(avr_reg(X_REG), 1)?;
                self.a.adc(avr_reg(X_REG + 1), 0)?;
            }
            0b1110 => {
                self.a.mov(al, avr_reg(X_REG))?;
                self.a.mov(ah, avr_reg(X_REG + 1))?;
                self.a.dec(ax)?;
                self.a.mov(avr_reg(X_REG), al)?;
                self.a.mov(avr_reg(X_REG + 1), ah)?;
                let val = self.read_memory(ax)?;
                self.a.mov(avr_reg(d), val)?;
            }
            _ => panic!("Invalid LD instruction"),
        }

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_ldd(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ldd);
        let d = i.fields[0];
        let q = i.fields[1];
        let t = i.fields[2];

        if t != 0 {
            self.a.mov(al, avr_reg(Y_REG))?;
            self.a.mov(ah, avr_reg(Y_REG + 1))?;
        } else {
            self.a.mov(al, avr_reg(Z_REG))?;
            self.a.mov(ah, avr_reg(Z_REG + 1))?;
        }
        self.a.add(ax, q as u32)?;
        let val = self.read_memory(ax)?;
        self.a.mov(avr_reg(d), val)?;

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_lds(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ld);
        let d = i.fields[0];
        let addr = i.fields[1];

        let val = self.read_memory(addr as u32)?;
        self.a.mov(avr_reg(d), val)?;

        self.a.inc(avr_pc)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_st(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::St);
        let d = i.fields[0];
        let t = i.fields[1];

        match t {
            0b0001 => {
                self.a.mov(al, avr_reg(Z_REG))?;
                self.a.mov(ah, avr_reg(Z_REG + 1))?;
                self.write_memory(ax, avr_reg(d))?;
                self.a.add(avr_reg(Z_REG), 1)?;
                self.a.adc(avr_reg(Z_REG + 1), 0)?;
            }
            0b0010 => {
                self.a.mov(al, avr_reg(Z_REG))?;
                self.a.mov(ah, avr_reg(Z_REG + 1))?;
                self.a.dec(ax)?;
                self.a.mov(avr_reg(Z_REG), al)?;
                self.a.mov(avr_reg(Z_REG + 1), ah)?;
                self.write_memory(ax, avr_reg(d))?;
            }

            0b1001 => {
                self.a.mov(al, avr_reg(Y_REG))?;
                self.a.mov(ah, avr_reg(Y_REG + 1))?;
                self.write_memory(ax, avr_reg(d))?;
                self.a.add(avr_reg(Y_REG), 1)?;
                self.a.adc(avr_reg(Y_REG + 1), 0)?;
            }
            0b1010 => {
                self.a.mov(al, avr_reg(Y_REG))?;
                self.a.mov(ah, avr_reg(Y_REG + 1))?;
                self.a.dec(ax)?;
                self.a.mov(avr_reg(Y_REG), al)?;
                self.a.mov(avr_reg(Y_REG + 1), ah)?;
                self.write_memory(ax, avr_reg(d))?;
            }

            0b1100 => {
                self.a.mov(al, avr_reg(X_REG))?;
                self.a.mov(ah, avr_reg(X_REG + 1))?;
                self.write_memory(ax, avr_reg(d))?;
            }
            0b1101 => {
                self.a.mov(al, avr_reg(X_REG))?;
                self.a.mov(ah, avr_reg(X_REG + 1))?;
                self.write_memory(ax, avr_reg(d))?;
                self.a.add(avr_reg(X_REG), 1)?;
                self.a.adc(avr_reg(X_REG + 1), 0)?;
            }
            0b1110 => {
                self.a.mov(al, avr_reg(X_REG))?;
                self.a.mov(ah, avr_reg(X_REG + 1))?;
                self.a.dec(ax)?;
                self.a.mov(avr_reg(X_REG), al)?;
                self.a.mov(avr_reg(X_REG + 1), ah)?;
                self.write_memory(ax, avr_reg(d))?;
            }
            _ => panic!("Invalid LD instruction"),
        }

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_std(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Std);
        let d = i.fields[0];
        let q = i.fields[1];
        let t = i.fields[2];

        if t != 0 {
            self.a.mov(al, avr_reg(Y_REG))?;
            self.a.mov(ah, avr_reg(Y_REG + 1))?;
        } else {
            self.a.mov(al, avr_reg(Z_REG))?;
            self.a.mov(ah, avr_reg(Z_REG + 1))?;
        }
        self.a.add(ax, q as u32)?;
        self.write_memory(ax, avr_reg(d))?;

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_sts(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sts);
        let d = i.fields[0];
        let addr = i.fields[1];

        self.write_memory(addr as u32, avr_reg(d))?;

        self.a.inc(avr_pc)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_lpm(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Lpm);
        let d = i.fields[0];
        let t = i.fields[0];

        self.a.movzx(eax, avr_reg(Z_REG))?;
        self.a.mov(ah, avr_reg(Z_REG + 1))?;
        self.a.add(rax, flash_reg)?;
        self.a.mov(bl, byte_ptr(rax))?;

        self.a.mov(avr_reg(d), bl)?;
        if t == 0x5 {
            self.a.sub(rax, flash_reg)?;
            self.a.inc(rax)?;
            self.a.mov(avr_reg(Z_REG), al)?;
            self.a.mov(avr_reg(Z_REG + 1), ah)?;
        }
        self.a.inc(avr_pc)?;
        self.a.add(current_tick, 3)
    }

    pub fn avr_elpm(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Elpm);
        let d = i.fields[0];
        let t = i.fields[1];

        self.a.movzx(eax, avr_rampz())?;
        self.a.shl(eax, 16)?;
        self.a.or(al, avr_reg(Z_REG))?;
        self.a.or(ah, avr_reg(Z_REG + 1))?;
        self.a.add(rax, flash_reg)?;
        self.a.mov(bl, byte_ptr(rax))?;

        self.a.mov(avr_reg(d), bl)?;
        if t == 0x7 {
            self.a.sub(rax, flash_reg)?;
            self.a.inc(rax)?;
            self.a.mov(avr_reg(Z_REG), al)?;
            self.a.mov(avr_reg(Z_REG + 1), ah)?;
            self.a.shl(eax, 16)?;
            self.a.mov(avr_rampz(), al)?;
        }
        self.a.inc(avr_pc)?;
        self.a.add(current_tick, 3)
    }

    pub fn avr_in(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::In);
        let io = i.fields[0];
        let d = i.fields[1];

        let val = self.read_io(io as u8)?;
        self.a.mov(avr_reg(d), val)?;

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_out(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Out);
        let io = i.fields[0];
        let d = i.fields[1];

        self.write_io(io as u32, avr_reg(d))?;

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_push(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Push);
        let d: u16 = i.fields[0];

        self.write_memory(avr_sp(), avr_reg(d))?;

        self.a.dec(avr_sp())?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }
    pub fn avr_pop(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Pop);
        let d: u16 = i.fields[0];

        self.a.inc(avr_sp())?;
        let val = self.read_memory(avr_sp())?;
        self.a.mov(avr_reg(d), val)?;

        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }
}
