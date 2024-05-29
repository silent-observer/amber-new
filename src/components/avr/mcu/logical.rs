use bitfield::Bit;

use crate::components::avr::bit_helpers::{get_d_field, get_k8, get_rd_fields};

use super::Mcu;

impl Mcu {
    fn status_logic(&mut self, r: u8) {
        self.sreg.set_z(r == 0);
        self.sreg.set_n(r.bit(7));
        self.sreg.set_v(false);
        self.sreg.set_s(r.bit(7));
    }

    pub fn instr_and(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd & rr;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_andi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd & k;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_or(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd | rr;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_ori(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd | k;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_eor(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd ^ rr;

        self.write_register(d, result);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_com(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = !rd;

        self.write_register(d, result);

        self.sreg.set_c(true);
        self.status_logic(result);
        self.pc += 1;

        1
    }

    pub fn instr_neg(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = 0x00u8.wrapping_sub(rd);

        self.write_register(d, result);

        self.sreg.set_c(result != 0);
        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(result == 0x80);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(!rd.bit(3) & result.bit(3));
        self.pc += 1;

        1
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn and() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0010_00_1_00011_0001, // and r3, r17
            "---0000-",
        );
        assert_eq!(mcu.read_register(3), 0x10);

        mcu.execute_and_assert_sreg(
            0b0010_00_0_00011_1111, // and r3, r15
            "---0001-",
        );
        assert_eq!(mcu.read_register(3), 0x00);

        mcu.execute_and_assert_sreg(
            0b0010_00_0_01111_1111, // and r15, r15
            "---1010-",
        );
        assert_eq!(mcu.read_register(15), 0xAB);
    }

    #[test]
    fn andi() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0xAB);
        mcu.execute_and_assert_sreg(
            0b0111_0110_0001_1001, // andi r17, 0x69
            "---0000-",
        );
        assert_eq!(mcu.read_register(17), 0x29);

        mcu.execute_and_assert_sreg(
            0b0111_1100_0001_0010, // andi r17, 0xC2
            "---0001-",
        );
        assert_eq!(mcu.read_register(17), 0x00);
    }

    #[test]
    fn or() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0010_10_1_00011_0001, // or r3, r17
            "---0000-",
        );
        assert_eq!(mcu.read_register(3), 0x36);

        mcu.execute_and_assert_sreg(
            0b0010_10_0_00011_1111, // or r3, r15
            "---1010-",
        );
        assert_eq!(mcu.read_register(3), 0xBF);

        mcu.execute_and_assert_sreg(
            0b0010_10_0_01111_1111, // or r15, r15
            "---1010-",
        );
        assert_eq!(mcu.read_register(15), 0xAB);
    }

    #[test]
    fn ori() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0xAB);
        mcu.execute_and_assert_sreg(
            0b0110_0110_0001_1001, // ori r17, 0x69
            "---1010-",
        );
        assert_eq!(mcu.read_register(17), 0xEB);

        mcu.execute_and_assert_sreg(
            0b0110_0001_0001_0000, // ori r17, 0x10
            "---1010-",
        );
        assert_eq!(mcu.read_register(17), 0xFB);
    }

    #[test]
    fn eor() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0010_01_1_00011_0001, // eor r3, r17
            "---0000-",
        );
        assert_eq!(mcu.read_register(3), 0x26);

        mcu.execute_and_assert_sreg(
            0b0010_01_0_00011_1111, // eor r3, r15
            "---1010-",
        );
        assert_eq!(mcu.read_register(3), 0x8D);

        mcu.execute_and_assert_sreg(
            0b0010_01_0_01111_1111, // eor r15, r15
            "---0001-",
        );
        assert_eq!(mcu.read_register(15), 0x00);
    }

    #[test]
    fn com() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(18, 0xFF);
        mcu.execute_and_assert_sreg(
            0b1001_010_10001_0000, // com r17
            "---10101",
        );
        assert_eq!(mcu.read_register(17), 0xED);

        mcu.execute_and_assert_sreg(
            0b1001_010_10001_0000, // com r17
            "---00001",
        );
        assert_eq!(mcu.read_register(17), 0x12);

        mcu.execute_and_assert_sreg(
            0b1001_010_10010_0000, // com r18
            "---00011",
        );
        assert_eq!(mcu.read_register(18), 0x00);

        mcu.execute_and_assert_sreg(
            0b1001_010_10010_0000, // com r18
            "---10101",
        );
        assert_eq!(mcu.read_register(18), 0xFF);
    }

    #[test]
    fn neg() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(18, 0xFF);
        mcu.execute_and_assert_sreg(
            0b1001_010_10001_0001, // neg r17
            "--110101",
        );
        assert_eq!(mcu.read_register(17), 0xEE);

        mcu.execute_and_assert_sreg(
            0b1001_010_10001_0001, // neg r17
            "--000001",
        );
        assert_eq!(mcu.read_register(17), 0x12);

        mcu.execute_and_assert_sreg(
            0b1001_010_10010_0001, // neg r18
            "--000001",
        );
        assert_eq!(mcu.read_register(18), 0x01);

        mcu.execute_and_assert_sreg(
            0b1001_010_10010_0001, // neg r18
            "--110101",
        );
        assert_eq!(mcu.read_register(18), 0xFF);
    }
}
