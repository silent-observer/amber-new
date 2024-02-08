use bitfield::Bit;

use crate::components::avr::bit_helpers::get_rd_fields;

use super::Mcu;

impl Mcu {
    pub fn instr_mul(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = (rd as u16) * (rr as u16);

        self.write_register_pair(0, result);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result == 0);

        self.pc += 1;
        2
    }

    pub fn instr_muls(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 4);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let rd_signed = (rd as i8) as i16;
        let rr_signed = (rr as i8) as i16;
        let result = (rd_signed * rr_signed) as u16;

        self.write_register_pair(0, result);
        self.sreg.set_c((result).bit(15));
        self.sreg.set_z(result == 0);

        self.pc += 1;
        2
    }

    pub fn instr_mulsu(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let rd_signed = (rd as i8) as i16;
        let rr_unsigned = (rr as u16) as i16;
        let result = (rd_signed * rr_unsigned) as u16;

        self.write_register_pair(0, result);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result == 0);

        self.pc += 1;
        2
    }

    pub fn instr_fmul(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let result = (rd as u16) * (rr as u16);

        self.write_register_pair(0, result << 1);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result << 1 == 0);

        self.pc += 1;
        2
    }

    pub fn instr_fmuls(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let rd_signed = (rd as i8) as i16;
        let rr_signed = (rr as i8) as i16;
        let result = (rd_signed * rr_signed) as u16;

        self.write_register_pair(0, result << 1);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result << 1 == 0);

        self.pc += 1;
        2
    }

    pub fn instr_fmulsu(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 3);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        let rd_signed = (rd as i8) as i16;
        let rr_unsigned = (rr as u16) as i16;
        let result = (rd_signed * rr_unsigned) as u16;

        self.write_register_pair(0, result << 1);
        self.sreg.set_c(result.bit(15));
        self.sreg.set_z(result << 1 == 0);

        self.pc += 1;
        2
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn mul() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0x34);
        mcu.execute_and_assert_sreg(
            0b1001_11_0_10001_1111, // mul r17, r15
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xA8);
        assert_eq!(mcu.read_register(1), 0x03);

        mcu.write_register(17, 0x85);
        mcu.write_register(15, 0x34);
        mcu.execute_and_assert_sreg(
            0b1001_11_0_10001_1111, // mul r17, r15
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x04);
        assert_eq!(mcu.read_register(1), 0x1B);

        mcu.write_register(17, 0x85);
        mcu.write_register(15, 0x96);
        mcu.execute_and_assert_sreg(
            0b1001_11_0_10001_1111, // mul r17, r15
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xEE);
        assert_eq!(mcu.read_register(1), 0x4D);
    }

    #[test]
    fn muls() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0010_0001_0111, // muls r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xA8);
        assert_eq!(mcu.read_register(1), 0x03);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0010_0001_0111, // muls r17, r23
            "------01",
        );
        assert_eq!(mcu.read_register(0), 0x04);
        assert_eq!(mcu.read_register(1), 0xE7);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x96);
        mcu.execute_and_assert_sreg(
            0b0000_0010_0001_0111, // muls r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xEE);
        assert_eq!(mcu.read_register(1), 0x32);
    }

    #[test]
    fn mulsu() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_0_111, // mulsu r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xA8);
        assert_eq!(mcu.read_register(1), 0x03);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_0_111, // mulsu r17, r23
            "------01",
        );
        assert_eq!(mcu.read_register(0), 0x04);
        assert_eq!(mcu.read_register(1), 0xE7);

        mcu.write_register(17, 0x34);
        mcu.write_register(23, 0x85);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_0_111, // mulsu r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x04);
        assert_eq!(mcu.read_register(1), 0x1B);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x96);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_0_111, // mulsu r17, r23
            "------01",
        );
        assert_eq!(mcu.read_register(0), 0xEE);
        assert_eq!(mcu.read_register(1), 0xB7);
    }

    #[test]
    fn fmul() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_1_111, // fmul r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x50);
        assert_eq!(mcu.read_register(1), 0x07);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_1_111, // fmul r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x08);
        assert_eq!(mcu.read_register(1), 0x36);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x96);
        mcu.execute_and_assert_sreg(
            0b0000_0011_0_001_1_111, // fmul r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xDC);
        assert_eq!(mcu.read_register(1), 0x9B);
    }

    #[test]
    fn fmuls() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_0_111, // fmuls r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x50);
        assert_eq!(mcu.read_register(1), 0x07);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_0_111, // fmuls r17, r23
            "------01",
        );
        assert_eq!(mcu.read_register(0), 0x08);
        assert_eq!(mcu.read_register(1), 0xCE);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x96);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_0_111, // fmuls r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0xDC);
        assert_eq!(mcu.read_register(1), 0x65);
    }

    #[test]
    fn fmulsu() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_1_111, // fmulsu r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x50);
        assert_eq!(mcu.read_register(1), 0x07);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_1_111, // fmulsu r17, r23
            "------01",
        );
        assert_eq!(mcu.read_register(0), 0x08);
        assert_eq!(mcu.read_register(1), 0xCE);

        mcu.write_register(17, 0x34);
        mcu.write_register(23, 0x85);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_1_111, // fmulsu r17, r23
            "------00",
        );
        assert_eq!(mcu.read_register(0), 0x08);
        assert_eq!(mcu.read_register(1), 0x36);

        mcu.write_register(17, 0x85);
        mcu.write_register(23, 0x96);
        mcu.execute_and_assert_sreg(
            0b0000_0011_1_001_1_111, // fmulsu r17, r23
            "------01",
        );
        assert_eq!(mcu.read_register(0), 0xDC);
        assert_eq!(mcu.read_register(1), 0x6F);
    }
}
