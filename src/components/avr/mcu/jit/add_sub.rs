use iced_x86::code_asm::*;

use crate::components::avr::mcu::jit::instructions::AvrInstructionType;

use super::{assembly::*, instructions::Inst};

impl JitAssembler {
    fn set_addsub_flags(&mut self, i: Inst, is_sbc: bool) -> Result<(), IcedError> {
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
            if is_sbc {
                self.a.or(bl, 0xFD)?;
                self.a.and(avr_sreg, bl)?;
            } else {
                self.a.and(avr_sreg, 0xFD)?;
                self.a.or(avr_sreg, bl)?;
            }
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
        if *i.used_flags.get(5).unwrap() {
            // TODO: H
        }
        Ok(())
    }

    pub fn avr_add(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Add);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.add(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_adc(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Adc);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.bt(avr_sreg16, 0)?;
        self.a.adc(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_adiw(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Adiw);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.mov(ah, avr_reg(d + 1))?;
        self.a.add(ax, k as u32)?;
        self.a.mov(avr_reg(d), al)?;
        self.a.mov(avr_reg(d + 1), ah)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_sub(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sub);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.sub(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_sbc(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbc);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.bt(avr_sreg16, 0)?;
        self.a.sbb(al, avr_reg(r))?;
        self.a.mov(avr_reg(d), al)?;
        self.set_addsub_flags(i, true)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_subi(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Subi);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.sub(al, k as u32)?;
        self.a.mov(avr_reg(d), al)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_sbci(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbci);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.bt(avr_sreg16, 0)?;
        self.a.sbb(al, k as u32)?;
        self.a.mov(avr_reg(d), al)?;
        self.set_addsub_flags(i, true)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_sbiw(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Sbiw);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.mov(ah, avr_reg(d + 1))?;
        self.a.sub(ax, k as u32)?;
        self.a.mov(avr_reg(d), al)?;
        self.a.mov(avr_reg(d + 1), ah)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_inc(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Inc);
        let d = i.fields[0];
        self.a.inc(avr_reg(d))?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_dec(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Dec);
        let d = i.fields[0];
        self.a.dec(avr_reg(d))?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_cp(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Cp);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.sub(al, avr_reg(r))?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_cpc(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Cpc);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.bt(avr_sreg16, 0)?;
        self.a.sbb(al, avr_reg(r))?;
        self.set_addsub_flags(i, true)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }

    pub fn avr_cpi(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Cpi);
        let k = i.fields[0];
        let d = i.fields[1];
        self.a.mov(al, avr_reg(d))?;
        self.a.sub(al, k as u32)?;
        self.set_addsub_flags(i, false)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)
    }
}
