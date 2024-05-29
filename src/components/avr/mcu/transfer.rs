use bitfield::Bit;

use crate::components::avr::bit_helpers::{
    bit_field_combined, get_d_field, get_io6, get_k8, get_rd_fields,
};

use super::Mcu;

const X_REG: u16 = 26;
const Y_REG: u16 = 28;
const Z_REG: u16 = 30;

impl Mcu {
    pub fn instr_mov(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);

        self.write_register(d, rr);
        self.pc += 1;

        1
    }

    pub fn instr_movw(&mut self, opcode: u16) -> u8 {
        let r = bit_field_combined(opcode, &[3..=0]) << 1;
        let d = bit_field_combined(opcode, &[7..=4]) << 1;
        let rr = self.read_register_pair(r);

        self.write_register_pair(d, rr);
        self.pc += 1;

        1
    }

    pub fn instr_ldi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);

        self.write_register(d, k);
        self.pc += 1;

        1
    }

    pub fn instr_ld(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);

        match bit_field_combined(opcode, &[3..=0]) {
            0b0001 => {
                let addr = self.read_register_pair(Z_REG);
                let val = self.read(addr);
                self.write_register(d, val);
                self.write_register_pair(Z_REG, addr + 1);
            }
            0b0010 => {
                let addr = self.read_register_pair(Z_REG);
                let val = self.read(addr - 1);
                self.write_register(d, val);
                self.write_register_pair(Z_REG, addr - 1);
            }

            0b1001 => {
                let addr = self.read_register_pair(Y_REG);
                let val = self.read(addr);
                self.write_register(d, val);
                self.write_register_pair(Y_REG, addr + 1);
            }
            0b1010 => {
                let addr = self.read_register_pair(Y_REG);
                let val = self.read(addr - 1);
                self.write_register(d, val);
                self.write_register_pair(Y_REG, addr - 1);
            }

            0b1100 => {
                let addr = self.read_register_pair(X_REG);
                let val = self.read(addr);
                self.write_register(d, val);
            }
            0b1101 => {
                let addr = self.read_register_pair(X_REG);
                let val = self.read(addr);
                self.write_register(d, val);
                self.write_register_pair(X_REG, addr + 1);
            }
            0b1110 => {
                let addr = self.read_register_pair(X_REG);
                let val = self.read(addr - 1);
                self.write_register(d, val);
                self.write_register_pair(X_REG, addr - 1);
            }
            _ => panic!("Invalid LD instruction"),
        }

        self.pc += 1;

        2
    }

    pub fn instr_ldd(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let q = bit_field_combined(opcode, &[13..=13, 11..=10, 2..=0]);

        let addr_base = if opcode.bit(3) {
            self.read_register_pair(Y_REG)
        } else {
            self.read_register_pair(Z_REG)
        };

        let addr = addr_base.wrapping_add(q);
        let val = self.read(addr);
        self.write_register(d, val);
        self.pc += 1;

        2
    }

    pub fn instr_lds(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);

        let addr = self.read_at_pc_offset(1);

        let val = self.read(addr);
        self.write_register(d, val);
        self.pc += 2;

        2
    }

    pub fn instr_st(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let val = self.read_register(d);

        match bit_field_combined(opcode, &[3..=0]) {
            0b0001 => {
                let addr = self.read_register_pair(Z_REG);
                self.write(addr, val);
                self.write_register_pair(Z_REG, addr + 1);
            }
            0b0010 => {
                let addr = self.read_register_pair(Z_REG);
                self.write(addr - 1, val);
                self.write_register_pair(Z_REG, addr - 1);
            }

            0b1001 => {
                let addr = self.read_register_pair(Y_REG);
                self.write(addr, val);
                self.write_register_pair(Y_REG, addr + 1);
            }
            0b1010 => {
                let addr = self.read_register_pair(Y_REG);
                self.write(addr - 1, val);
                self.write_register_pair(Y_REG, addr - 1);
            }

            0b1100 => {
                let addr = self.read_register_pair(X_REG);
                self.write(addr, val);
            }
            0b1101 => {
                let addr = self.read_register_pair(X_REG);
                self.write(addr, val);
                self.write_register_pair(X_REG, addr + 1);
            }
            0b1110 => {
                let addr = self.read_register_pair(X_REG);
                self.write(addr - 1, val);
                self.write_register_pair(X_REG, addr - 1);
            }
            _ => panic!("Invalid LD instruction"),
        }

        self.pc += 1;

        2
    }

    pub fn instr_std(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let q = bit_field_combined(opcode, &[13..=13, 11..=10, 2..=0]);

        let addr_base = if opcode.bit(3) {
            self.read_register_pair(Y_REG)
        } else {
            self.read_register_pair(Z_REG)
        };

        let addr = addr_base.wrapping_add(q);
        let val = self.read_register(d);
        self.write(addr, val);
        self.pc += 1;

        2
    }

    pub fn instr_sts(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);

        let addr = self.read_at_pc_offset(1);

        let val = self.read_register(d);
        self.write(addr, val);
        self.pc += 2;

        2
    }

    pub fn instr_lpm(&mut self, opcode: u16) -> u8 {
        let d = if opcode == 0x95C8 {
            0
        } else {
            get_d_field(opcode, 5)
        };

        let addr = self.read_register_pair(Z_REG);

        let val = self.read_flash(addr as u32 >> 1);

        let val = if addr.bit(0) {
            (val >> 8) as u8
        } else {
            val as u8
        };
        self.write_register(d, val);
        if opcode & 0x000F == 0x5 {
            self.write_register_pair(Z_REG, addr + 1);
        }
        self.pc += 1;

        3
    }

    pub fn instr_elpm(&mut self, opcode: u16) -> u8 {
        let d = if opcode == 0x95D8 {
            0
        } else {
            get_d_field(opcode, 5)
        };

        let z = self.read_register_pair(Z_REG);
        let addr = self.rampz_address(z);

        let val = self.read_flash(addr as u32 >> 1);

        let val = if addr.bit(0) {
            (val >> 8) as u8
        } else {
            val as u8
        };
        self.write_register(d, val);
        if opcode & 0x000F == 0x7 {
            self.write_register_pair(Z_REG, (addr + 1) as u16);
            self.rampz = ((addr + 1) >> 16) as u8;
        }
        self.pc += 1;

        3
    }

    pub fn instr_spm(&mut self, _opcode: u16) -> u8 {
        todo!()
    }

    pub fn instr_in(&mut self, opcode: u16) -> u8 {
        let io = get_io6(opcode);
        let d = get_d_field(opcode, 5);
        let val = self.read_io(io);
        self.write_register(d, val);
        self.pc += 1;
        1
    }

    pub fn instr_out(&mut self, opcode: u16) -> u8 {
        let io = get_io6(opcode);
        let d = get_d_field(opcode, 5);
        let val = self.read_register(d);
        self.write_io(io, val);
        self.pc += 1;
        1
    }

    pub fn instr_push(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let val = self.read_register(d);
        self.write_at_sp_offset(0, val);
        self.sp -= 1;
        self.pc += 1;
        2
    }

    pub fn instr_pop(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        self.sp += 1;
        let val = self.read_at_sp_offset(0);
        self.write_register(d, val);
        self.pc += 1;
        2
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn mov() {
        let mut mcu = Mcu::default();
        mcu.write_register(21, 0x98);
        mcu.write_register(25, 0x12);
        mcu.execute_and_assert_sreg(
            0b0010_11_1_11001_0101, // mov r25, r21
            "--------",
        );
        assert_eq!(mcu.read_register(25), 0x98);
    }

    #[test]
    fn movw() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(26, 0xBA98);
        mcu.write_register_pair(20, 0x1234);
        mcu.execute_and_assert_sreg(
            0b0000_0001_1101_1010, // movw r26, r20
            "--------",
        );
        assert_eq!(mcu.read_register_pair(26), 0x1234);
    }

    #[test]
    fn ldi() {
        let mut mcu = Mcu::default();
        mcu.execute_and_assert_sreg(
            0b1110_1010_1001_1011, // ldi r25, 0xAB
            "--------",
        );
        assert_eq!(mcu.read_register(25), 0xAB);
    }

    #[test]
    fn ld() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(X_REG, 0x1000);
        mcu.write_register_pair(Y_REG, 0x1200);
        mcu.write_register_pair(Z_REG, 0x1230);

        mcu.write(0x1000, 0x12);
        mcu.write(0x1200, 0x34);
        mcu.write(0x1230, 0x56);

        mcu.execute_and_assert_sreg(
            0b1001_000_10001_1100, // ld r17, X
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x12);
        assert_eq!(mcu.read_register_pair(X_REG), 0x1000);

        mcu.execute_and_assert_sreg(
            0b1001_000_10010_1101, // ld r18, X+
            "--------",
        );
        assert_eq!(mcu.read_register(18), 0x12);
        assert_eq!(mcu.read_register_pair(X_REG), 0x1001);

        mcu.execute_and_assert_sreg(
            0b1001_000_10011_1110, // ld r19, -X
            "--------",
        );
        assert_eq!(mcu.read_register(19), 0x12);
        assert_eq!(mcu.read_register_pair(X_REG), 0x1000);

        mcu.execute_and_assert_sreg(
            0b1001_000_10010_1001, // ld r18, Y+
            "--------",
        );
        assert_eq!(mcu.read_register(18), 0x34);
        assert_eq!(mcu.read_register_pair(Y_REG), 0x1201);

        mcu.execute_and_assert_sreg(
            0b1001_000_10011_1010, // ld r19, -Y
            "--------",
        );
        assert_eq!(mcu.read_register(19), 0x34);
        assert_eq!(mcu.read_register_pair(Y_REG), 0x1200);

        mcu.execute_and_assert_sreg(
            0b1001_000_10010_0001, // ld r18, Z+
            "--------",
        );
        assert_eq!(mcu.read_register(18), 0x56);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x1231);

        mcu.execute_and_assert_sreg(
            0b1001_000_10011_0010, // ld r19, -Z
            "--------",
        );
        assert_eq!(mcu.read_register(19), 0x56);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x1230);
    }

    #[test]
    fn ldd() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(Y_REG, 0x1000);
        mcu.write_register_pair(Z_REG, 0x1200);

        mcu.write(0x1000, 0x12);
        mcu.write(0x1010, 0x23);
        mcu.write(0x1015, 0x34);
        mcu.write(0x1200, 0x45);
        mcu.write(0x1210, 0x56);
        mcu.write(0x1215, 0x67);

        mcu.execute_and_assert_sreg(
            0b10_0_0_00_0_10001_1_000, // ldd r17, Y
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x12);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_0_10001_1_000, // ldd r17, Y+0x10
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x23);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_0_10001_1_101, // ldd r17, Y+0x15
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x34);

        mcu.execute_and_assert_sreg(
            0b10_0_0_00_0_10001_0_000, // ldd r17, Z
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x45);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_0_10001_0_000, // ldd r17, Z+0x10
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x56);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_0_10001_0_101, // ldd r17, Z+0x15
            "--------",
        );
        assert_eq!(mcu.read_register(17), 0x67);
    }

    #[test]
    fn lds() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_flash(0x1235, 0x2023);
        mcu.write(0x2023, 0xAB);

        mcu.execute_and_assert_sreg(
            0b1001_000_11001_0000, // lds r25, 0x2023
            "--------",
        );
        assert_eq!(mcu.read_register(25), 0xAB);
    }

    #[test]
    fn st() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(X_REG, 0x1000);
        mcu.write_register_pair(Y_REG, 0x1200);
        mcu.write_register_pair(Z_REG, 0x1230);

        mcu.write_register(17, 0x12);
        mcu.write_register(18, 0x34);
        mcu.write_register(19, 0x56);

        mcu.execute_and_assert_sreg(
            0b1001_001_10001_1100, // st X, r17
            "--------",
        );
        assert_eq!(mcu.read(0x1000), 0x12);
        assert_eq!(mcu.read_register_pair(X_REG), 0x1000);

        mcu.execute_and_assert_sreg(
            0b1001_001_10010_1101, // st X+, r18
            "--------",
        );
        assert_eq!(mcu.read(0x1000), 0x34);
        assert_eq!(mcu.read_register_pair(X_REG), 0x1001);

        mcu.execute_and_assert_sreg(
            0b1001_001_10011_1110, // st -X, r19
            "--------",
        );
        assert_eq!(mcu.read(0x1000), 0x56);
        assert_eq!(mcu.read_register_pair(X_REG), 0x1000);

        mcu.execute_and_assert_sreg(
            0b1001_001_10010_1001, // st Y+, r18
            "--------",
        );
        assert_eq!(mcu.read(0x1200), 0x34);
        assert_eq!(mcu.read_register_pair(Y_REG), 0x1201);

        mcu.execute_and_assert_sreg(
            0b1001_001_10011_1010, // st -Y, r19
            "--------",
        );
        assert_eq!(mcu.read(0x1200), 0x56);
        assert_eq!(mcu.read_register_pair(Y_REG), 0x1200);

        mcu.execute_and_assert_sreg(
            0b1001_001_10010_0001, // st Z+, r18
            "--------",
        );
        assert_eq!(mcu.read(0x1230), 0x34);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x1231);

        mcu.execute_and_assert_sreg(
            0b1001_001_10011_0010, // st -Z, r19
            "--------",
        );
        assert_eq!(mcu.read(0x1230), 0x56);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x1230);
    }

    #[test]
    fn std() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(Y_REG, 0x1000);
        mcu.write_register_pair(Z_REG, 0x1200);

        mcu.write_register(17, 0x12);
        mcu.write_register(18, 0x23);
        mcu.write_register(19, 0x34);

        mcu.execute_and_assert_sreg(
            0b10_0_0_00_1_10001_1_000, // ldd r17, Y
            "--------",
        );
        assert_eq!(mcu.read(0x1000), 0x12);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_1_10010_1_000, // ldd r17, Y+0x10
            "--------",
        );
        assert_eq!(mcu.read(0x1010), 0x23);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_1_10011_1_101, // ldd r17, Y+0x15
            "--------",
        );
        assert_eq!(mcu.read(0x1015), 0x34);

        mcu.execute_and_assert_sreg(
            0b10_0_0_00_1_10001_0_000, // ldd r17, Z
            "--------",
        );
        assert_eq!(mcu.read(0x1200), 0x12);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_1_10010_0_000, // ldd r17, Z+0x10
            "--------",
        );
        assert_eq!(mcu.read(0x1210), 0x23);

        mcu.execute_and_assert_sreg(
            0b10_0_0_10_1_10011_0_101, // ldd r17, Z+0x15
            "--------",
        );
        assert_eq!(mcu.read(0x1215), 0x34);
    }

    #[test]
    fn sts() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_flash(0x1235, 0x2023);
        mcu.write_register(25, 0xAB);

        mcu.execute_and_assert_sreg(
            0b1001_001_11001_0000, // sts r25, 0x2023
            "--------",
        );
        assert_eq!(mcu.read(0x2023), 0xAB);
    }

    #[test]
    fn lpm() {
        let mut mcu = Mcu::default();
        mcu.write_flash(0x1234, 0x2023);
        mcu.write_register_pair(Z_REG, 0x2468);

        mcu.execute_and_assert_sreg(
            0x95C8, // lpm
            "--------",
        );
        assert_eq!(mcu.read_register(0), 0x23);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x2468);

        mcu.execute_and_assert_sreg(
            0b1001_000_00001_0101, // lpm r1, Z+
            "--------",
        );
        assert_eq!(mcu.read_register(1), 0x23);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x2469);

        mcu.execute_and_assert_sreg(
            0b1001_000_00010_0100, // lpm r2, Z
            "--------",
        );
        assert_eq!(mcu.read_register(2), 0x20);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x2469);
    }

    #[test]
    fn elpm() {
        let mut mcu = Mcu::default();
        mcu.rampz = 0x2;
        mcu.write_flash(0x11234, 0x2023);
        mcu.write_register_pair(Z_REG, 0x2468);

        mcu.execute_and_assert_sreg(
            0x95D8, // elpm
            "--------",
        );
        assert_eq!(mcu.read_register(0), 0x23);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x2468);

        mcu.execute_and_assert_sreg(
            0b1001_000_00001_0111, // elpm r1, Z+
            "--------",
        );
        assert_eq!(mcu.read_register(1), 0x23);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x2469);

        mcu.execute_and_assert_sreg(
            0b1001_000_00010_0110, // elpm r2, Z
            "--------",
        );
        assert_eq!(mcu.read_register(2), 0x20);
        assert_eq!(mcu.read_register_pair(Z_REG), 0x2469);
    }

    // #[test]
    // fn r#in() {
    //     let mut io = MockIoControllerTrait::new();

    //     io.expect_read_internal_u8()
    //       .with(eq(15))
    //       .times(1)
    //       .return_const(0xAB);
    //     io.expect_read_internal_u8()
    //       .with(eq(17))
    //       .times(1)
    //       .return_const(0xCD);

    //     let mut mcu: Mcu = Mcu::new(io);

    //     mcu.execute_and_assert_sreg(
    //         0b1011_0_00_01100_1111, // in r12, 0x0F
    //         "--------");
    //     assert_eq!(mcu.read_register(12), 0xAB);

    //     mcu.execute_and_assert_sreg(
    //         0b1011_0_01_01100_0001, // in r12, 0x11
    //         "--------");
    //         assert_eq!(mcu.read_register(12), 0xCD);
    // }

    // #[test]
    // fn out() {
    //     let mut io = MockIoControllerTrait::new();

    //     io.expect_write_internal_u8()
    //       .with(eq(15), eq(0xAB))
    //       .times(1)
    //       .return_const(());
    //     io.expect_write_internal_u8()
    //       .with(eq(17), eq(0xCD))
    //       .times(1)
    //       .return_const(());

    //     let mut mcu: Mcu = Mcu::new(io);

    //     mcu.write_register(12, 0xAB);
    //     mcu.execute_and_assert_sreg(
    //         0b1011_1_00_01100_1111, // out r12, 0x0F
    //         "--------");

    //     mcu.write_register(12, 0xCD);
    //     mcu.execute_and_assert_sreg(
    //         0b1011_1_01_01100_0001, // out r12, 0x11
    //         "--------");
    // }

    #[test]
    fn push() {
        let mut mcu = Mcu::default();
        mcu.sp = 0x21FF;
        mcu.write_register(12, 0xAB);
        mcu.write_register(13, 0xCD);

        mcu.execute_and_assert_sreg(
            0b1001_001_01100_1111, // push r12
            "--------",
        );
        assert_eq!(mcu.read(0x21FF), 0xAB);
        assert_eq!(mcu.sp, 0x21FE);

        mcu.execute_and_assert_sreg(
            0b1001_001_01101_1111, // push r13
            "--------",
        );
        assert_eq!(mcu.read(0x21FE), 0xCD);
        assert_eq!(mcu.sp, 0x21FD);
    }

    #[test]
    fn pop() {
        let mut mcu = Mcu::default();
        mcu.sp = 0x21FD;
        mcu.write(0x21FF, 0xAB);
        mcu.write(0x21FE, 0xCD);

        mcu.execute_and_assert_sreg(
            0b1001_000_01100_1111, // pop r12
            "--------",
        );
        assert_eq!(mcu.read_register(12), 0xCD);
        assert_eq!(mcu.sp, 0x21FE);

        mcu.execute_and_assert_sreg(
            0b1001_000_01101_1111, // pop r13
            "--------",
        );
        assert_eq!(mcu.read_register(13), 0xAB);
        assert_eq!(mcu.sp, 0x21FF);
    }
}
