use iced_x86::code_asm::*;

use crate::components::avr::mcu::jit::instructions::AvrInstructionType;

use super::{assembly::*, instructions::Inst};
impl JitAssembler {
    fn set_shr_flags(&mut self, i: Inst) -> Result<(), IcedError> {
        if i.used_flags.any() {
            self.a.lahf()?;
        }
        if *i.used_flags.get(3).unwrap() {
            // V
            self.a.seto(bl)?;
            self.a.rorx(ebx, ebx, 32 - 3)?;
            self.a.and(avr_sreg, 0xF7)?;
            self.a.or(avr_sreg, bl)?;
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
        if *i.used_flags.get(4).unwrap() {
            // S
            self.a.bt(avr_sreg16, 2)?;
            self.a.setc(bl)?;
            self.a.bt(avr_sreg16, 3)?;
            self.a.setc(bh)?;
            self.a.xor(bl, bh)?;
            self.a.shl(bl, 4)?;
            self.a.and(avr_sreg, 0xEF)?;
            self.a.or(avr_sreg, bl)?;
        }
        Ok(())
    }

    pub fn avr_lsr(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Lsr);
        let d = i.fields[0];
        self.a.shr(avr_reg(d), 1)?;
        self.set_shr_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_ror(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Ror);
        let d = i.fields[0];
        self.a.bt(avr_sreg16, 0)?;
        self.a.rcr(avr_reg(d), 1)?;
        self.set_shr_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_asr(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Asr);
        let d = i.fields[0];
        self.a.sar(avr_reg(d), 1)?;
        self.set_shr_flags(i)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_swap(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Swap);
        let d = i.fields[0];
        self.a.rol(avr_reg(d), 4)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_bset(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Bset);
        let b = i.fields[0];
        self.a.bts(avr_sreg16, b as u32)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_bclr(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Bclr);
        let b = i.fields[0];
        self.a.btr(avr_sreg16, b as u32)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_bst(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Bst);
        let d = i.fields[0];
        let b = i.fields[1];
        self.a.bt(avr_reg(d), b as u32)?;
        self.a.setc(al)?;
        self.a.shl(al, 6)?;
        self.a.and(avr_sreg, 0xBF)?;
        self.a.or(avr_sreg, al)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_bld(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Bld);
        let d = i.fields[0];
        let b = i.fields[1];
        let mask = 0xFF as u32 ^ (1 << b);
        self.a.bt(avr_sreg16, 6)?;
        self.a.setc(ah)?;
        self.a.shl(ah, b as u32)?;
        self.a.mov(al, avr_reg(d))?;
        self.a.and(avr_sreg, mask)?;
        self.a.or(avr_sreg, ah)?;
        self.a.mov(avr_reg(d), al)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_sbi(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbi);
        let io = i.fields[0];
        let b = i.fields[1];

        self.change_io(io as u8, |j| j.a.bts(ax, b as u32))?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_cbi(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbi);
        let io = i.fields[0];
        let b = i.fields[1];

        self.change_io(io as u8, |j| j.a.btr(ax, b as u32))?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }
}
