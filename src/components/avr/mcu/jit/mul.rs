use bitfield::Bit;
use iced_x86::code_asm::*;
use memoffset::offset_of;

use crate::components::avr::mcu::{jit::instructions::AvrInstructionType, Mcu};

use super::{assembly::*, instructions::Inst};

impl JitAssembler {
    pub fn avr_mul(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Mul);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(ah, avr_reg(d))?;
        self.a.mov(al, avr_reg(r))?;
        self.a.mul(ah)?;

        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.setc(bl)?;
            if *i.used_flags.get(1).unwrap() {
                // C+Z
                self.a.setz(bh)?;
                self.a.and(avr_sreg, 0xFC)?;
                self.a.shl(bh, 1)?;
                self.a.or(avr_sreg, bh)?;
            } else {
                // C+!Z
                self.a.and(avr_sreg, 0xFE)?;
            }
            self.a.or(avr_sreg, bl)?;
        } else if *i.used_flags.get(1).unwrap() {
            // !C+Z
            self.a.setz(bh)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.shl(bh, 1)?;
            self.a.or(avr_sreg, bh)?;
        }
        self.a.mov(avr_reg(0), al)?;
        self.a.mov(avr_reg(1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_muls(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Muls);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(ah, avr_reg(d))?;
        self.a.mov(al, avr_reg(r))?;
        self.a.imul(ah)?;
        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.setc(bl)?;
            self.a.and(avr_sreg, 0xFE)?;
            self.a.or(avr_sreg, bl)?;
        }
        if *i.used_flags.get(1).unwrap() {
            // Z
            self.a.setz(bl)?;
            self.a.rorx(ebx, ebx, 32 - 1)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.mov(avr_reg(0), al)?;
        self.a.mov(avr_reg(1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_mulsu(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Muls);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.movsx(ax, avr_reg(d))?;
        self.a.movzx(bx, avr_reg(r))?;
        self.a.imul_2(ax, bx)?;
        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.setc(bl)?;
            if *i.used_flags.get(1).unwrap() {
                // C+Z
                self.a.setz(bh)?;
                self.a.and(avr_sreg, 0xFC)?;
                self.a.shl(bh, 1)?;
                self.a.or(avr_sreg, bh)?;
            } else {
                // C+!Z
                self.a.and(avr_sreg, 0xFE)?;
            }
            self.a.or(avr_sreg, bl)?;
        } else if *i.used_flags.get(1).unwrap() {
            // !C+Z
            self.a.setz(bh)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.shl(bh, 1)?;
            self.a.or(avr_sreg, bh)?;
        }
        self.a.mov(avr_reg(0), al)?;
        self.a.mov(avr_reg(1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_fmul(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Fmul);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(ah, avr_reg(d))?;
        self.a.mov(al, avr_reg(r))?;
        self.a.mul(ah)?;
        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.setc(bl)?;
            self.a.and(avr_sreg, 0xFE)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.shl(ax, 1)?;
        if *i.used_flags.get(1).unwrap() {
            // Z
            self.a.setz(bl)?;
            self.a.rorx(ebx, ebx, 32 - 1)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.mov(avr_reg(0), al)?;
        self.a.mov(avr_reg(1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_fmuls(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Fmuls);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.mov(ah, avr_reg(d))?;
        self.a.mov(al, avr_reg(r))?;
        self.a.imul(ah)?;
        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.setc(bl)?;
            self.a.and(avr_sreg, 0xFE)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.shl(ax, 1)?;
        if *i.used_flags.get(1).unwrap() {
            // Z
            self.a.setz(bl)?;
            self.a.rorx(ebx, ebx, 32 - 1)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.mov(avr_reg(0), al)?;
        self.a.mov(avr_reg(1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }

    pub fn avr_fmulsu(&mut self, i: Inst) -> Result<(), IcedError> {
        assert_eq!(i.kind, AvrInstructionType::Fmulsu);
        let r = i.fields[0];
        let d = i.fields[1];
        self.a.movsx(ax, avr_reg(d))?;
        self.a.movzx(bx, avr_reg(r))?;
        self.a.imul_2(ax, bx)?;
        if *i.used_flags.get(0).unwrap() {
            // C
            self.a.setc(bl)?;
            self.a.and(avr_sreg, 0xFE)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.shl(ax, 1)?;
        if *i.used_flags.get(1).unwrap() {
            // Z
            self.a.setz(bl)?;
            self.a.rorx(ebx, ebx, 32 - 1)?;
            self.a.and(avr_sreg, 0xFD)?;
            self.a.or(avr_sreg, bl)?;
        }
        self.a.mov(avr_reg(0), al)?;
        self.a.mov(avr_reg(1), ah)?;
        self.a.inc(avr_pc)?;
        self.a.inc(current_tick)?;
        self.a.inc(current_tick)
    }
}
