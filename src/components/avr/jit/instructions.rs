use std::ops::BitAnd;

use bitfield::Bit;
use bitvec::prelude::*;

use crate::jit::instructions::{Instruction, InstructionKind};

use super::super::bit_helpers::{
    bit_field_combined, get_d_field, get_io5, get_io6, get_k6, get_k8, get_r_field, get_rd_fields,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AvrInstructionType {
    Nop,
    Movw,
    Muls,

    Mulsu,
    Fmul,
    Fmuls,
    Fmulsu,

    Cpc,
    Sbc,
    Add,
    Cpse,
    Cp,
    Sub,
    Adc,

    And,
    Eor,
    Or,
    Mov,

    Cpi,
    Sbci,
    Subi,
    Ori,
    Andi,

    Std,
    Ldd,

    Lds,
    Ld,
    Pop,

    Sts,
    St,
    Push,

    Com,
    Neg,
    Swap,
    Inc,
    Asr,
    Lsr,
    Ror,

    Bclr,
    Bset,
    Ret,
    Reti,
    Lpm,
    Elpm,
    Spm,

    Ijmp,
    Eijmp,
    Icall,
    Eicall,

    Dec,

    Jmp,
    Call,

    Adiw,
    Sbiw,
    Cbi,
    Sbic,
    Sbi,
    Sbis,
    Mul,

    In,
    Out,

    Rjmp,
    Rcall,
    Ldi,

    Brbs,
    Brbc,
    Bld,
    Bst,
    Sbrc,
    Sbrs,
}

impl From<AvrInstructionType> for InstructionKind {
    fn from(value: AvrInstructionType) -> Self {
        InstructionKind(value as u8)
    }
}

type Inst = Instruction<AvrInstructionType>;

pub fn decode(flash: &[u16], addr: usize) -> (Inst, i32) {
    let opcode = flash[addr];

    let head = (opcode >> 8) as u8;
    let t = match head {
        0x00 => {
            if opcode == 0x0000 {
                AvrInstructionType::Nop
            } else {
                panic!("Reserved")
            }
        }
        0x01 => AvrInstructionType::Movw,
        0x02 => AvrInstructionType::Muls,
        0x03 => {
            let bits = bit_field_combined(opcode, &[7..=7, 3..=3]);
            match bits {
                0b00 => AvrInstructionType::Mulsu,
                0b01 => AvrInstructionType::Fmul,
                0b10 => AvrInstructionType::Fmuls,
                0b11 => AvrInstructionType::Fmulsu,
                _ => panic!("2-bit field impossible value"),
            }
        }
        0x04..=0x07 => AvrInstructionType::Cpc,
        0x08..=0x0B => AvrInstructionType::Sbc,
        0x0C..=0x0F => AvrInstructionType::Add,
        0x10..=0x13 => AvrInstructionType::Cpse,
        0x14..=0x17 => AvrInstructionType::Cp,
        0x18..=0x1B => AvrInstructionType::Sub,
        0x1C..=0x1F => AvrInstructionType::Adc,

        0x20..=0x23 => AvrInstructionType::And,
        0x24..=0x27 => AvrInstructionType::Eor,
        0x28..=0x2B => AvrInstructionType::Or,
        0x2C..=0x2F => AvrInstructionType::Mov,

        0x30..=0x3F => AvrInstructionType::Cpi,
        0x40..=0x4F => AvrInstructionType::Sbci,
        0x50..=0x5F => AvrInstructionType::Subi,
        0x60..=0x6F => AvrInstructionType::Ori,
        0x70..=0x7F => AvrInstructionType::Andi,

        0x80..=0x8F | 0xA0..=0xAF => {
            if head.bit(1) {
                AvrInstructionType::Std
            } else {
                AvrInstructionType::Ldd
            }
        }

        0x90 | 0x91 => {
            let tail = opcode & 0x000F;
            match tail {
                0x0 => AvrInstructionType::Lds,
                0x1 | 0x2 | 0x9 | 0xA | 0xC..=0xE => AvrInstructionType::Ld,
                0x4 | 0x5 => AvrInstructionType::Lpm,
                0x6 | 0x7 => AvrInstructionType::Elpm,
                0xF => AvrInstructionType::Pop,
                0x3 | 0x8 | 0xB => panic!("Reserved"),
                _ => panic!("Impossible for 4-bit value"),
            }
        }

        0x92 | 0x93 => {
            let tail = opcode & 0x000F;
            match tail {
                0x0 => AvrInstructionType::Sts,
                0x1 | 0x2 | 0x9 | 0xA | 0xC..=0xE => AvrInstructionType::St,
                0xF => AvrInstructionType::Push,
                0x3..=0x8 | 0xB => panic!("Reserved"),
                _ => panic!("Impossible for 4-bit value"),
            }
        }

        0x94 | 0x95 => {
            let tail = opcode & 0x000F;
            match tail {
                0x0 => AvrInstructionType::Com,
                0x1 => AvrInstructionType::Neg,
                0x2 => AvrInstructionType::Swap,
                0x3 => AvrInstructionType::Inc,
                0x4 => panic!("Reserved"),
                0x5 => AvrInstructionType::Asr,
                0x6 => AvrInstructionType::Lsr,
                0x7 => AvrInstructionType::Ror,

                0x8 => {
                    if head == 0x94 {
                        if opcode.bit(7) {
                            AvrInstructionType::Bclr
                        } else {
                            AvrInstructionType::Bset
                        }
                    } else {
                        match opcode {
                            0x9508 => AvrInstructionType::Ret,
                            0x9518 => AvrInstructionType::Reti,
                            0x9588 => todo!(),
                            0x9598 => todo!(),
                            0x95A8 => todo!(),
                            0x95C8 => AvrInstructionType::Lpm,
                            0x95D8 => AvrInstructionType::Elpm,
                            0x95E8 => AvrInstructionType::Spm,
                            _ => panic!("Reserved"),
                        }
                    }
                }
                0x9 => match opcode {
                    0x9409 => AvrInstructionType::Ijmp,
                    0x9419 => AvrInstructionType::Eijmp,
                    0x9509 => AvrInstructionType::Icall,
                    0x9519 => AvrInstructionType::Eicall,
                    _ => panic!("Reserved"),
                },
                0xA => AvrInstructionType::Dec,
                0xB => panic!("Reserved"),
                0xC | 0xD => AvrInstructionType::Jmp,
                0xE | 0xF => AvrInstructionType::Call,
                _ => panic!("Impossible for 4-bit value"),
            }
        }

        0x96 => AvrInstructionType::Adiw,
        0x97 => AvrInstructionType::Sbiw,
        0x98 => AvrInstructionType::Cbi,
        0x99 => AvrInstructionType::Sbic,
        0x9A => AvrInstructionType::Sbi,
        0x9B => AvrInstructionType::Sbis,
        0x9C..=0x9F => AvrInstructionType::Mul,

        0xB0..=0xB7 => AvrInstructionType::In,
        0xB8..=0xBF => AvrInstructionType::Out,

        0xC0..=0xCF => AvrInstructionType::Rjmp,
        0xD0..=0xDF => AvrInstructionType::Rcall,
        0xE0..=0xEF => AvrInstructionType::Ldi,

        0xF0..=0xF3 => AvrInstructionType::Brbs,
        0xF4..=0xF7 => AvrInstructionType::Brbc,
        0xF8..=0xF9 => AvrInstructionType::Bld,
        0xFA..=0xFB => AvrInstructionType::Bst,
        0xFC..=0xFD => AvrInstructionType::Sbrc,
        0xFE..=0xFF => AvrInstructionType::Sbrs,
    };

    let inst = Instruction::new(t, addr);
    let inst = match t {
        AvrInstructionType::Nop
        | AvrInstructionType::Ijmp
        | AvrInstructionType::Eijmp
        | AvrInstructionType::Icall
        | AvrInstructionType::Eicall
        | AvrInstructionType::Ret
        | AvrInstructionType::Reti => inst,
        AvrInstructionType::Add
        | AvrInstructionType::Adc
        | AvrInstructionType::Sub
        | AvrInstructionType::Sbc
        | AvrInstructionType::Cp
        | AvrInstructionType::Cpc
        | AvrInstructionType::Cpse
        | AvrInstructionType::And
        | AvrInstructionType::Or
        | AvrInstructionType::Eor
        | AvrInstructionType::Mul
        | AvrInstructionType::Mov => {
            let (r, d) = get_rd_fields(opcode, 5);
            inst.with_field(r).with_field(d)
        }
        AvrInstructionType::Muls => {
            let (r, d) = get_rd_fields(opcode, 4);
            inst.with_field(r).with_field(d)
        }
        AvrInstructionType::Mulsu
        | AvrInstructionType::Fmul
        | AvrInstructionType::Fmuls
        | AvrInstructionType::Fmulsu => {
            let (r, d) = get_rd_fields(opcode, 3);
            inst.with_field(r).with_field(d)
        }

        AvrInstructionType::Adiw | AvrInstructionType::Sbiw => {
            let k = get_k6(opcode) as u16;
            let d = get_d_field(opcode, 2);
            inst.with_field(k).with_field(d)
        }
        AvrInstructionType::Subi
        | AvrInstructionType::Cpi
        | AvrInstructionType::Sbci
        | AvrInstructionType::Andi
        | AvrInstructionType::Ori
        | AvrInstructionType::Ldi => {
            let k = get_k8(opcode) as u16;
            let d = get_d_field(opcode, 4);
            inst.with_field(k).with_field(d)
        }
        AvrInstructionType::Inc
        | AvrInstructionType::Dec
        | AvrInstructionType::Lsr
        | AvrInstructionType::Ror
        | AvrInstructionType::Asr
        | AvrInstructionType::Swap
        | AvrInstructionType::Com
        | AvrInstructionType::Neg
        | AvrInstructionType::Push
        | AvrInstructionType::Pop => {
            let d = get_d_field(opcode, 5);
            inst.with_field(d)
        }

        AvrInstructionType::Bset | AvrInstructionType::Bclr => {
            let b = bit_field_combined(opcode, &[6..=4]);
            inst.with_field(b)
        }
        AvrInstructionType::Bst
        | AvrInstructionType::Bld
        | AvrInstructionType::Sbrc
        | AvrInstructionType::Sbrs => {
            let d = get_d_field(opcode, 5);
            let b = opcode & 0x0007;
            inst.with_field(d).with_field(b)
        }

        AvrInstructionType::Sbi
        | AvrInstructionType::Cbi
        | AvrInstructionType::Sbic
        | AvrInstructionType::Sbis => {
            let io = get_io5(opcode) as u16;
            let b = opcode & 0x0007;
            inst.with_field(io).with_field(b)
        }

        AvrInstructionType::Rjmp | AvrInstructionType::Rcall => {
            let k = opcode & 0x0FFF;
            inst.with_field(k)
        }

        AvrInstructionType::Jmp | AvrInstructionType::Call => {
            let addr_high = bit_field_combined(opcode, &[8..=4, 0..=0]);
            let addr_low = flash[addr + 1];
            inst.with_field(addr_high).with_field(addr_low)
        }

        AvrInstructionType::Brbc | AvrInstructionType::Brbs => {
            let k = bit_field_combined(opcode, &[9..=3]);
            let s = bit_field_combined(opcode, &[2..=0]);
            inst.with_field(k).with_field(s)
        }

        AvrInstructionType::Movw => {
            let r = bit_field_combined(opcode, &[3..=0]) << 1;
            let d = bit_field_combined(opcode, &[7..=4]) << 1;
            inst.with_field(r).with_field(d)
        }

        AvrInstructionType::Ld | AvrInstructionType::St => {
            let d = get_d_field(opcode, 5);
            let t = bit_field_combined(opcode, &[3..=0]);
            inst.with_field(d).with_field(t)
        }

        AvrInstructionType::Ldd | AvrInstructionType::Std => {
            let d = get_d_field(opcode, 5);
            let q = bit_field_combined(opcode, &[13..=13, 11..=10, 2..=0]);
            let addr_base = if opcode.bit(3) { 28 } else { 30 };
            inst.with_field(d).with_field(q).with_field(addr_base)
        }

        AvrInstructionType::Lds | AvrInstructionType::Sts => {
            let d = get_d_field(opcode, 5);
            let addr = flash[addr + 1];
            inst.with_field(d).with_field(addr)
        }

        AvrInstructionType::Lpm | AvrInstructionType::Elpm => {
            let d = if opcode & 0xFF0F == 0x9508 {
                0
            } else {
                get_d_field(opcode, 5)
            };
            let t = opcode & 0x000F;
            inst.with_field(d).with_field(t)
        }

        AvrInstructionType::In | AvrInstructionType::Out => {
            let io = get_io6(opcode) as u16;
            let d = get_d_field(opcode, 5);
            inst.with_field(io).with_field(d)
        }

        AvrInstructionType::Spm => todo!(),
    };
    let size = match t {
        AvrInstructionType::Lds
        | AvrInstructionType::Sts
        | AvrInstructionType::Jmp
        | AvrInstructionType::Call => 2,
        _ => 1,
    };
    (inst, size)
}

impl Inst {
    pub fn is_jump(&self) -> bool {
        matches!(
            self.kind,
            AvrInstructionType::Jmp
                | AvrInstructionType::Call
                | AvrInstructionType::Rjmp
                | AvrInstructionType::Rcall
                | AvrInstructionType::Ijmp
                | AvrInstructionType::Eijmp
                | AvrInstructionType::Icall
                | AvrInstructionType::Eicall
                | AvrInstructionType::Ret
                | AvrInstructionType::Reti
                | AvrInstructionType::Cpse
                | AvrInstructionType::Sbrc
                | AvrInstructionType::Sbrs
                | AvrInstructionType::Sbic
                | AvrInstructionType::Sbis
                | AvrInstructionType::Brbc
                | AvrInstructionType::Brbs
        )
    }

    pub fn is_unconditional_jump(&self) -> bool {
        matches!(
            self.kind,
            AvrInstructionType::Jmp
                | AvrInstructionType::Call
                | AvrInstructionType::Rjmp
                | AvrInstructionType::Rcall
                | AvrInstructionType::Ijmp
                | AvrInstructionType::Eijmp
                | AvrInstructionType::Icall
                | AvrInstructionType::Eicall
                | AvrInstructionType::Ret
                | AvrInstructionType::Reti
        )
    }

    pub fn is_end(&self) -> bool {
        matches!(
            self.kind,
            AvrInstructionType::Jmp
                | AvrInstructionType::Rjmp
                | AvrInstructionType::Ijmp
                | AvrInstructionType::Eijmp
                | AvrInstructionType::Ret
                | AvrInstructionType::Reti
        )
    }

    pub fn branch_target(&self, current_addr: usize) -> Option<usize> {
        match self.kind {
            AvrInstructionType::Brbc | AvrInstructionType::Brbs => {
                let k = self.fields[0];
                if k.bit(6) {
                    let k = (k ^ 0x007F) + 1; // Negation
                    Some(current_addr - (k as usize) + 1)
                } else {
                    Some(current_addr + (k as usize) + 1)
                }
            }
            AvrInstructionType::Rjmp => {
                let k = self.fields[0];
                if k.bit(11) {
                    let k = (k ^ 0x0FFF) + 1;
                    Some(current_addr - (k as usize) + 1)
                } else {
                    Some(current_addr + (k as usize) + 1)
                }
            }
            _ => None,
        }
    }

    pub fn affected_flags(&self) -> BitArray {
        match self.kind {
            AvrInstructionType::Add
            | AvrInstructionType::Adc
            | AvrInstructionType::Sub
            | AvrInstructionType::Sbc
            | AvrInstructionType::Subi
            | AvrInstructionType::Sbci
            | AvrInstructionType::Cp
            | AvrInstructionType::Cpi
            | AvrInstructionType::Cpc
            | AvrInstructionType::Neg => {
                bitarr!(1, 1, 1, 1, 1, 1, 0, 0)
            }

            AvrInstructionType::Adiw
            | AvrInstructionType::Sbiw
            | AvrInstructionType::Lsr
            | AvrInstructionType::Ror
            | AvrInstructionType::Asr => {
                bitarr!(1, 1, 1, 1, 1, 0, 0, 0)
            }

            AvrInstructionType::Inc
            | AvrInstructionType::Dec
            | AvrInstructionType::And
            | AvrInstructionType::Andi
            | AvrInstructionType::Or
            | AvrInstructionType::Ori
            | AvrInstructionType::Eor
            | AvrInstructionType::Com => {
                bitarr!(0, 1, 1, 1, 1, 0, 0, 0)
            }

            AvrInstructionType::Bset | AvrInstructionType::Bclr => {
                let b = self.fields[0];
                let mut arr = BitArray::ZERO;
                arr.set(b as usize, true);
                arr
            }
            AvrInstructionType::Bst => {
                bitarr!(0, 0, 0, 0, 0, 0, 1, 0)
            }

            _ => BitArray::ZERO,
        }
    }

    pub fn reads_flags(&self) -> BitArray {
        match self.kind {
            AvrInstructionType::Adc | AvrInstructionType::Ror => {
                bitarr!(1, 0, 0, 0, 0, 0, 0, 0)
            }

            AvrInstructionType::Sbc | AvrInstructionType::Sbci | AvrInstructionType::Cpc => {
                bitarr!(1, 1, 0, 0, 0, 0, 0, 0)
            }

            AvrInstructionType::Bld => {
                bitarr!(0, 0, 0, 0, 0, 0, 1, 0)
            }

            AvrInstructionType::Jmp
            | AvrInstructionType::Call
            | AvrInstructionType::Rjmp
            | AvrInstructionType::Rcall
            | AvrInstructionType::Ijmp
            | AvrInstructionType::Eijmp
            | AvrInstructionType::Icall
            | AvrInstructionType::Eicall
            | AvrInstructionType::Ret
            | AvrInstructionType::Reti
            | AvrInstructionType::Cpse
            | AvrInstructionType::Sbrc
            | AvrInstructionType::Sbrs
            | AvrInstructionType::Sbic
            | AvrInstructionType::Sbis
            | AvrInstructionType::Brbc
            | AvrInstructionType::Brbs => {
                bitarr!(1, 1, 1, 1, 1, 1, 1, 1)
            }

            _ => BitArray::ZERO,
        }
    }
}
