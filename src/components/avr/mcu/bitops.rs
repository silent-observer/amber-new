use bitfield::{Bit, BitMut};

use crate::components::avr::bit_helpers::{bit_field_combined, get_d_field, get_io5};

use super::Mcu;

impl Mcu {
    fn status_shr(&mut self, rd: u8, result: u8) {
        self.sreg.set_c(rd.bit(0));
        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(self.sreg.n() ^ self.sreg.c());
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
    }

    pub fn instr_lsr(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd >> 1;

        self.write_register(d, result);
        self.status_shr(rd, result);
        self.pc += 1;

        1
    }

    pub fn instr_ror(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd >> 1 | if self.sreg.c() { 0x80 } else { 0x00 };

        self.write_register(d, result);
        self.status_shr(rd, result);
        self.pc += 1;

        1
    }

    pub fn instr_asr(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd >> 1 | if rd.bit(7) { 0x80 } else { 0x00 };

        self.write_register(d, result);
        self.status_shr(rd, result);
        self.pc += 1;

        1
    }

    pub fn instr_swap(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = (rd >> 4) | (rd << 4);

        self.write_register(d, result);
        self.pc += 1;

        1
    }

    pub fn instr_bset(&mut self, opcode: u16) -> u8 {
        let b = bit_field_combined(opcode, &[6..=4]);
        self.sreg.set_bit(b as usize, true);
        self.pc += 1;
        1
    }

    pub fn instr_bclr(&mut self, opcode: u16) -> u8 {
        let b = bit_field_combined(opcode, &[6..=4]);
        self.sreg.set_bit(b as usize, false);
        self.pc += 1;
        1
    }

    pub fn instr_bst(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let b = opcode & 0x0007;
        self.sreg.set_t(rd.bit(b as usize));
        self.pc += 1;
        1
    }

    pub fn instr_bld(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let mut rd = self.read_register(d);
        let b = opcode & 0x0007;
        rd.set_bit(b as usize, self.sreg.t());
        self.write_register(d, rd);
        self.pc += 1;
        1
    }
    pub fn instr_sbi(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let mut val = self.read_io(io);
        val.set_bit(b as usize, true);
        self.write_io(io, val);

        self.pc += 1;
        2
    }

    pub fn instr_cbi(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let mut val = self.read_io(io);
        val.set_bit(b as usize, false);
        self.write_io(io, val);

        self.pc += 1;
        2
    }
}

#[cfg(test)]
mod tests {

    use crate::components::avr::sreg::StatusRegister;

    use super::*;

    // #[test]
    // fn sbi() {
    //     let mut io = MockIoControllerTrait::new();

    //     io.expect_read_internal_u8()
    //       .with(eq(15))
    //       .return_const(0b10000010)
    //       .times(1);
    //     io.expect_write_internal_u8()
    //       .with(eq(15), eq(0b10100010))
    //       .return_const(())
    //       .times(1);

    //     io.expect_read_internal_u8()
    //       .with(eq(18))
    //       .return_const(0b10000110)
    //       .times(1);
    //     io.expect_write_internal_u8()
    //       .with(eq(18), eq(0b10000110))
    //       .return_const(())
    //       .times(1);

    //     let mut mcu: Mcu = Mcu::new(io);

    //     mcu.execute_and_assert_sreg(
    //         0b1001_1010_01111_101, // sbi 0x0F, 5
    //         "--------");

    //     mcu.execute_and_assert_sreg(
    //         0b1001_1010_10010_010, // sbi 0x12, 2
    //         "--------");
    // }

    // #[test]
    // fn cbi() {
    //     let mut io = MockIoControllerTrait::new();

    //     io.expect_read_internal_u8()
    //       .with(eq(15))
    //       .return_const(0b10000010)
    //       .times(1);
    //     io.expect_write_internal_u8()
    //       .with(eq(15), eq(0b10000010))
    //       .return_const(())
    //       .times(1);

    //     io.expect_read_internal_u8()
    //       .with(eq(18))
    //       .return_const(0b10000110)
    //       .times(1);
    //     io.expect_write_internal_u8()
    //       .with(eq(18), eq(0b10000010))
    //       .return_const(())
    //       .times(1);

    //     let mut mcu: Mcu = Mcu::new(io);

    //     mcu.execute_and_assert_sreg(
    //         0b1001_1000_01111_101, // cbi 0x0F, 5
    //         "--------");

    //     mcu.execute_and_assert_sreg(
    //         0b1001_1000_10010_010, // cbi 0x12, 2
    //         "--------");
    // }

    #[test]
    fn lsr() {
        let mut mcu = Mcu::default();
        mcu.write_register(21, 0x98);
        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x4C);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x26);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x13);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---11001",
        );
        assert_eq!(mcu.read_register(21), 0x09);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---11001",
        );
        assert_eq!(mcu.read_register(21), 0x04);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x02);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x01);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0110, // lsr r21
            "---11011",
        );
        assert_eq!(mcu.read_register(21), 0x00);
    }

    #[test]
    fn ror() {
        let mut mcu = Mcu::default();
        mcu.write_register(21, 0x98);
        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x4C);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x26);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x13);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---11001",
        );
        assert_eq!(mcu.read_register(21), 0x09);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---10101",
        );
        assert_eq!(mcu.read_register(21), 0x84);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---01100",
        );
        assert_eq!(mcu.read_register(21), 0xC2);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---00000",
        );
        assert_eq!(mcu.read_register(21), 0x61);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0111, // ror r21
            "---11001",
        );
        assert_eq!(mcu.read_register(21), 0x30);
    }

    #[test]
    fn asr() {
        let mut mcu = Mcu::default();
        mcu.write_register(21, 0x98);
        mcu.write_register(8, 0x37);
        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---01100",
        );
        assert_eq!(mcu.read_register(21), 0xCC);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---01100",
        );
        assert_eq!(mcu.read_register(21), 0xE6);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---01100",
        );
        assert_eq!(mcu.read_register(21), 0xF3);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---10101",
        );
        assert_eq!(mcu.read_register(21), 0xF9);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---10101",
        );
        assert_eq!(mcu.read_register(21), 0xFC);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---01100",
        );
        assert_eq!(mcu.read_register(21), 0xFE);

        mcu.execute_and_assert_sreg(
            0b1001_010_10101_0101, // asr r21
            "---01100",
        );
        assert_eq!(mcu.read_register(21), 0xFF);

        mcu.execute_and_assert_sreg(
            0b1001_010_01000_0101, // asr r8
            "---11001",
        );
        assert_eq!(mcu.read_register(8), 0x1B);
    }

    #[test]
    fn swap() {
        let mut mcu = Mcu::default();
        mcu.write_register(22, 0xAB);

        mcu.execute_and_assert_sreg(
            0b1001_010_10110_0010, // asr r22
            "--------",
        );
        assert_eq!(mcu.read_register(22), 0xBA);

        mcu.execute_and_assert_sreg(
            0b1001_010_10110_0010, // asr r22
            "--------",
        );
        assert_eq!(mcu.read_register(22), 0xAB);
    }

    #[test]
    fn bset() {
        let mut mcu = Mcu::default();

        mcu.execute_and_assert_sreg(
            0b1001_0100_0_001_1000, // bset 1
            "------1-",
        );

        mcu.execute_and_assert_sreg(
            0b1001_0100_0_101_1000, // bset 5
            "--1-----",
        );

        mcu.execute_and_assert_sreg(
            0b1001_0100_0_010_1000, // bset 2
            "-----1--",
        );

        mcu.execute_and_assert_sreg(
            0b1001_0100_0_111_1000, // bset 7
            "1-------",
        );
    }

    #[test]
    fn bclr() {
        let mut mcu = Mcu::default();
        mcu.sreg = StatusRegister(0xFF);

        mcu.execute_and_assert_sreg(
            0b1001_0100_1_001_1000, // bclr 1
            "------0-",
        );

        mcu.execute_and_assert_sreg(
            0b1001_0100_1_101_1000, // bclr 5
            "--0-----",
        );

        mcu.execute_and_assert_sreg(
            0b1001_0100_1_010_1000, // bclr 2
            "-----0--",
        );

        mcu.execute_and_assert_sreg(
            0b1001_0100_1_111_1000, // bclr 7
            "0-------",
        );
    }

    #[test]
    fn bst() {
        let mut mcu = Mcu::default();
        mcu.write_register(21, 0x98);

        mcu.execute_and_assert_sreg(
            0b1111_101_10101_0_011, // bst r21, 3
            "-1------",
        );

        mcu.execute_and_assert_sreg(
            0b1111_101_10101_0_100, // bst r21, 4
            "-1------",
        );

        mcu.execute_and_assert_sreg(
            0b1111_101_10101_0_101, // bst r21, 5
            "-0------",
        );

        mcu.execute_and_assert_sreg(
            0b1111_101_10101_0_110, // bst r21, 6
            "-0------",
        );

        mcu.execute_and_assert_sreg(
            0b1111_101_10101_0_111, // bst r21, 7
            "-1------",
        );
    }

    #[test]
    fn bld() {
        let mut mcu = Mcu::default();
        mcu.write_register(21, 0x98);

        mcu.execute_and_assert_sreg(
            0b1111_100_10101_0_011, // bld r21, 3
            "--------",
        );
        assert_eq!(mcu.read_register(21), 0x90);

        mcu.execute_and_assert_sreg(
            0b1111_100_10101_0_100, // bst r21, 4
            "--------",
        );
        assert_eq!(mcu.read_register(21), 0x80);

        mcu.sreg.set_t(true);
        mcu.execute_and_assert_sreg(
            0b1111_100_10101_0_101, // bst r21, 5
            "--------",
        );
        assert_eq!(mcu.read_register(21), 0xA0);

        mcu.execute_and_assert_sreg(
            0b1111_100_10101_0_110, // bst r21, 6
            "--------",
        );
        assert_eq!(mcu.read_register(21), 0xE0);

        mcu.execute_and_assert_sreg(
            0b1111_100_10101_0_111, // bst r21, 7
            "--------",
        );
        assert_eq!(mcu.read_register(21), 0xE0);
    }
}
