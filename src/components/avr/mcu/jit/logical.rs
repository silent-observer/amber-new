use bitfield::Bit;
use iced_x86::code_asm::*;
use memoffset::offset_of;

use crate::components::avr::mcu::{jit::instructions::AvrInstructionType, Mcu};

use super::{assembly::*, instructions::Inst};

impl JitAssembler {
    fn set_logic_flags(&mut self, i: Inst) -> Result<(), IcedError> {
        if i.used_flags.any() {
            self.a.lahf()?;
        }
        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.mov(bl, ah)?;
            self.a.and(bl, 0x01)?;
            self.a.and(avr_sreg, 0xFE)?;
            self.a.or(avr_sreg, bl)?;
        }
        if *i.used_flags.get(1).unwrap() {
            // Z
            self.a.mov(bl, ah)?;
            self.a.and(bl, 0x40)?;
            self.a.shr(bl, 5)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.or(avr_sreg, bl)?;
        }
        if *i.used_flags.get(2).unwrap() {
            // N
            self.a.mov(bl, ah)?;
            self.a.and(bl, 0x80)?;
            self.a.shr(bl, 5)?;
            self.a.and(avr_sreg, 0xFB)?;
            self.a.or(avr_sreg, bl)?;
        }
        if *i.used_flags.get(3).unwrap() {
            // V
            self.a.and(avr_sreg, 0xF7)?;
        }
        if *i.used_flags.get(4).unwrap() {
            // S
            self.a.bt(avr_sreg16, 2)?;
            self.a.setc(bl)?;
            self.a.shl(bl, 4)?;
            self.a.and(avr_sreg, 0xEF)?;
            self.a.or(avr_sreg, bl)?;
        }
        if *i.used_flags.get(5).unwrap() {
            // TODO: H
        }
        Ok(())
    }

    pub fn avr_and(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::And);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.and(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_andi(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Andi);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.and(al, k as u32)?;
        self.a.mov(avr_reg(d), al)?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_or(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Or);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.or(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_ori(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ori);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.or(al, k as u32)?;
        self.a.mov(avr_reg(d), al)?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_eor(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Eor);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.xor(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_com(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Eor);
        let d = i.fields[0];
        self.a.not(avr_reg(d))?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_neg(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Neg);
        let d = i.fields[0];
        self.a.neg(avr_reg(d))?;
        self.set_logic_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }
}
