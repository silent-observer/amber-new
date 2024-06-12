use bitfield::Bit;

use crate::components::avr::bit_helpers::{get_d_field, get_k6, get_k8, get_rd_fields};

use super::Mcu;

impl Mcu {
    fn status_add(&mut self, rd: u8, rr: u8, r: u8) {
        let rd7 = rd.bit(7);
        let rr7 = rr.bit(7);
        let r7 = r.bit(7);
        let rd3 = rd.bit(3);
        let rr3 = rr.bit(3);
        let r3 = r.bit(3);

        self.sreg.set_c(rd7 && rr7 || rd7 && !r7 || !r7 && rr7);
        self.sreg.set_z(r == 0x00);
        self.sreg.set_n(r7);
        self.sreg.set_v(rd7 && rr7 && !r7 || !rd7 && !rr7 && r7);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(rd3 && rr3 || rd3 && !r3 || !r3 && rr3);
    }

    pub fn instr_add(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rr.wrapping_add(rd);

        self.write_register(d, result);
        self.status_add(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_adc(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rr.wrapping_add(rd).wrapping_add(self.sreg.c() as u8);

        self.write_register(d, result);
        self.status_add(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_adiw(&mut self, opcode: u16) -> u8 {
        let k = get_k6(opcode) as u16;
        let d = get_d_field(opcode, 2);
        let rd = self.read_register_pair(d);
        let result = rd.wrapping_add(k);

        self.write_register_pair(d, result);

        let rd15 = rd.bit(15);
        let r15 = result.bit(15);

        self.sreg.set_c(!r15 && rd15);
        self.sreg.set_z(result == 0);
        self.sreg.set_n(r15);
        self.sreg.set_v(!rd15 && r15);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());

        self.pc += 1;
        2
    }

    fn status_sub(&mut self, rd: u8, rr: u8, r: u8) {
        let rd7 = rd.bit(7);
        let rr7 = rr.bit(7);
        let r7 = r.bit(7);
        let rd3 = rd.bit(3);
        let rr3 = rr.bit(3);
        let r3 = r.bit(3);

        self.sreg.set_c(!rd7 && rr7 || rr7 && r7 || r7 && !rd7);
        self.sreg.set_z(r == 0x00);
        self.sreg.set_n(r7);
        self.sreg.set_v(rd7 && !rr7 && !r7 || !rd7 && rr7 && r7);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(!rd3 && rr3 || rr3 && r3 || r3 && !rd3);
    }

    fn status_sbc(&mut self, rd: u8, rr: u8, r: u8) {
        let rd7 = rd.bit(7);
        let rr7 = rr.bit(7);
        let r7 = r.bit(7);
        let rd3 = rd.bit(3);
        let rr3 = rr.bit(3);
        let r3 = r.bit(3);

        self.sreg.set_c(!rd7 && rr7 || rr7 && r7 || r7 && !rd7);
        self.sreg.set_z(r == 0x00 && self.sreg.z());
        self.sreg.set_n(r7);
        self.sreg.set_v(rd7 && !rr7 && !r7 || !rd7 && rr7 && r7);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.sreg.set_h(!rd3 && rr3 || rr3 && r3 || r3 && !rd3);
    }

    pub fn instr_sub(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr);

        self.write_register(d, result);
        self.status_sub(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_subi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(k);

        self.write_register(d, result);
        self.status_sub(rd, k, result);

        self.pc += 1;
        1
    }

    pub fn instr_sbc(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr).wrapping_sub(self.sreg.c() as u8);

        self.write_register(d, result);
        self.status_sbc(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_sbci(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(k).wrapping_sub(self.sreg.c() as u8);

        self.write_register(d, result);
        self.status_sbc(rd, k, result);

        self.pc += 1;
        1
    }

    pub fn instr_sbiw(&mut self, opcode: u16) -> u8 {
        let k = get_k6(opcode) as u16;
        let d = get_d_field(opcode, 2);
        let rd = self.read_register_pair(d);
        let result = rd.wrapping_sub(k);

        self.write_register_pair(d, result);

        let rd15 = rd.bit(15);
        let r15 = result.bit(15);

        self.sreg.set_c(r15 && !rd15);
        self.sreg.set_z(result == 0);
        self.sreg.set_n(r15);
        self.sreg.set_v(r15 && !rd15);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());

        self.pc += 1;
        2
    }

    pub fn instr_inc(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd.wrapping_add(1);

        self.write_register(d, result);

        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(result == 0x80);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.pc += 1;

        1
    }

    pub fn instr_dec(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(1);

        self.write_register(d, result);

        self.sreg.set_z(result == 0);
        self.sreg.set_n(result.bit(7));
        self.sreg.set_v(result == 0x7F);
        self.sreg.set_s(self.sreg.n() ^ self.sreg.v());
        self.pc += 1;

        1
    }

    pub fn instr_cp(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr);

        self.status_sub(rd, rr, result);

        self.pc += 1;
        1
    }

    pub fn instr_cpi(&mut self, opcode: u16) -> u8 {
        let k = get_k8(opcode);
        let d = get_d_field(opcode, 4);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(k);

        self.status_sub(rd, k, result);

        self.pc += 1;
        1
    }

    pub fn instr_cpc(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);
        let result = rd.wrapping_sub(rr).wrapping_sub(self.sreg.c() as u8);

        self.status_sbc(rd, rr, result);

        self.pc += 1;
        1
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_11_1_00011_0001, // add r3, r17
            "--000000",
        );
        assert_eq!(mcu.read_register(3), 0x46);

        mcu.execute_and_assert_sreg(
            0b0000_11_0_00011_1111, // add r3, r15
            "--110100",
        );
        assert_eq!(mcu.read_register(3), 0xF1);

        mcu.execute_and_assert_sreg(
            0b0000_11_0_00011_1111, // add r3, r15
            "--010101",
        );
        assert_eq!(mcu.read_register(3), 0x9C);

        mcu.execute_and_assert_sreg(
            0b0000_11_0_00011_1111, // add r3, r15
            "--111001",
        );
        assert_eq!(mcu.read_register(3), 0x47);

        mcu.write_register(10, 0x12);
        mcu.write_register(11, 0xEE);
        mcu.execute_and_assert_sreg(
            0b0000_11_0_01010_1011, // add r10, r11
            "--100011",
        );
        assert_eq!(mcu.read_register(10), 0x00);
    }

    #[test]
    fn lsl() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x26);
        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--000000",
        );
        assert_eq!(mcu.read_register(17), 0x4C);

        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--101100",
        );
        assert_eq!(mcu.read_register(17), 0x98);

        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--111001",
        );
        assert_eq!(mcu.read_register(17), 0x30);

        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--000000",
        );
        assert_eq!(mcu.read_register(17), 0x60);

        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--001100",
        );
        assert_eq!(mcu.read_register(17), 0xC0);

        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--010101",
        );
        assert_eq!(mcu.read_register(17), 0x80);

        mcu.execute_and_assert_sreg(
            0b0000_11_1_10001_0001, // lsl r17
            "--011011",
        );
        assert_eq!(mcu.read_register(17), 0x00);
    }

    #[test]
    fn adc() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0001_11_1_00011_0001, // adc r3, r17
            "--000000",
        );
        assert_eq!(mcu.read_register(3), 0x46);

        mcu.execute_and_assert_sreg(
            0b0001_11_0_00011_1111, // adc r3, r15
            "--110100",
        );
        assert_eq!(mcu.read_register(3), 0xF1);

        mcu.execute_and_assert_sreg(
            0b0001_11_0_00011_1111, // adc r3, r15
            "--010101",
        );
        assert_eq!(mcu.read_register(3), 0x9C);

        mcu.execute_and_assert_sreg(
            0b0001_11_0_00011_1111, // adc r3, r15
            "--111001",
        );
        assert_eq!(mcu.read_register(3), 0x48);

        mcu.write_register(10, 0x11);
        mcu.write_register(11, 0xEE);
        mcu.execute_and_assert_sreg(
            0b0001_11_0_01010_1011, // adc r10, r11
            "--100011",
        );
        assert_eq!(mcu.read_register(10), 0x00);
    }

    #[test]
    fn rol() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x26);
        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--000000",
        );
        assert_eq!(mcu.read_register(17), 0x4C);

        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--101100",
        );
        assert_eq!(mcu.read_register(17), 0x98);

        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--111001",
        );
        assert_eq!(mcu.read_register(17), 0x30);

        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--000000",
        );
        assert_eq!(mcu.read_register(17), 0x61);

        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--001100",
        );
        assert_eq!(mcu.read_register(17), 0xC2);

        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--010101",
        );
        assert_eq!(mcu.read_register(17), 0x84);

        mcu.execute_and_assert_sreg(
            0b0001_11_1_10001_0001, // rol r17
            "--011001",
        );
        assert_eq!(mcu.read_register(17), 0x09);
    }

    #[test]
    fn adiw() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(26, 0x1234);
        mcu.write_register_pair(28, 0xABCD);
        mcu.write_register_pair(30, 0xFFE9);
        mcu.execute_and_assert_sreg(
            0b1001_0110_10_01_0101, // adiw r27:r26, 0x25
            "---00000",
        );
        assert_eq!(mcu.read_register_pair(26), 0x1259);

        mcu.execute_and_assert_sreg(
            0b1001_0110_11_10_0101, // adiw r29:r28, 0x35
            "---10100",
        );
        assert_eq!(mcu.read_register_pair(28), 0xAC02);

        mcu.execute_and_assert_sreg(
            0b1001_0110_01_11_0111, // adiw r31:r30, 0x17
            "---00011",
        );
        assert_eq!(mcu.read_register_pair(30), 0x0000);
    }

    #[test]
    fn sub() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0001_10_1_00011_0001, // sub r3, r17
            "--000000",
        );
        assert_eq!(mcu.read_register(3), 0x22);

        mcu.execute_and_assert_sreg(
            0b0001_10_0_00011_1111, // sub r3, r15
            "--100001",
        );
        assert_eq!(mcu.read_register(3), 0x77);

        mcu.execute_and_assert_sreg(
            0b0001_10_0_00011_1111, // sub r3, r15
            "--101101",
        );
        assert_eq!(mcu.read_register(3), 0xCC);

        mcu.write_register(10, 0x12);
        mcu.write_register(11, 0x12);
        mcu.execute_and_assert_sreg(
            0b0001_10_0_01010_1011, // sub r10, r11
            "--000010",
        );
        assert_eq!(mcu.read_register(10), 0x00);
        assert_eq!(mcu.sreg.c(), false);
        assert_eq!(mcu.sreg.z(), true);
        assert_eq!(mcu.sreg.n(), false);
        assert_eq!(mcu.sreg.v(), false);
        assert_eq!(mcu.sreg.s(), false);
        assert_eq!(mcu.sreg.h(), false);
    }

    #[test]
    fn sbc() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x12);
        mcu.write_register(15, 0xAB);
        mcu.write_register(3, 0x34);
        mcu.execute_and_assert_sreg(
            0b0000_10_1_00011_0001, // sbc r3, r17
            "--000000",
        );
        assert_eq!(mcu.read_register(3), 0x22);

        mcu.execute_and_assert_sreg(
            0b0000_10_0_00011_1111, // sbc r3, r15
            "--100001",
        );
        assert_eq!(mcu.read_register(3), 0x77);

        mcu.execute_and_assert_sreg(
            0b0000_10_0_00011_1111, // sbc r3, r15
            "--101101",
        );
        assert_eq!(mcu.read_register(3), 0xCB);

        mcu.write_register(10, 0x12);
        mcu.write_register(11, 0x11);
        mcu.execute_and_assert_sreg(
            0b0000_10_0_01010_1011, // sbc r10, r11
            "--000010",
        );
        assert_eq!(mcu.read_register(10), 0x00);
    }

    #[test]
    fn sbiw() {
        let mut mcu = Mcu::default();
        mcu.write_register_pair(26, 0x1234);
        mcu.write_register_pair(28, 0xAB03);
        mcu.write_register_pair(30, 0x0017);
        mcu.execute_and_assert_sreg(
            0b1001_0111_10_01_0101, // sbiw r27:r26, 0x25
            "---00000",
        );
        assert_eq!(mcu.read_register_pair(26), 0x120F);

        mcu.execute_and_assert_sreg(
            0b1001_0111_11_10_0101, // sbiw r29:r28, 0x35
            "---10100",
        );
        assert_eq!(mcu.read_register_pair(28), 0xAACE);

        mcu.execute_and_assert_sreg(
            0b1001_0111_01_11_0111, // sbiw r31:r30, 0x17
            "---00010",
        );
        assert_eq!(mcu.read_register_pair(30), 0x0000);
    }

    #[test]
    fn subi() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x57);
        mcu.write_register(18, 0x28);
        mcu.execute_and_assert_sreg(
            0b0101_0001_0001_0010, // subi r17, 0x12
            "--000000",
        );
        assert_eq!(mcu.read_register(17), 0x45);

        mcu.execute_and_assert_sreg(
            0b0101_0100_0001_0111, // subi r17, 0x47
            "--110101",
        );
        assert_eq!(mcu.read_register(17), 0xFE);

        mcu.execute_and_assert_sreg(
            0b0101_1010_0010_1011, // subi r18, 0xAB
            "--100001",
        );
        assert_eq!(mcu.read_register(18), 0x7D);

        mcu.execute_and_assert_sreg(
            0b0101_0111_0010_1101, // subi r18, 0x7D
            "--000010",
        );
        assert_eq!(mcu.read_register(18), 0x0000);
    }

    #[test]
    fn sbci() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x57);
        mcu.write_register(18, 0x28);
        mcu.execute_and_assert_sreg(
            0b0100_0001_0001_0010, // sbci r17, 0x12
            "--000000",
        );
        assert_eq!(mcu.read_register(17), 0x45);

        mcu.execute_and_assert_sreg(
            0b0100_0100_0001_0111, // sbci r17, 0x47
            "--110101",
        );
        assert_eq!(mcu.read_register(17), 0xFE);

        mcu.execute_and_assert_sreg(
            0b0100_1010_0010_1011, // sbci r18, 0xAB
            "--100001",
        );
        assert_eq!(mcu.read_register(18), 0x7C);

        mcu.execute_and_assert_sreg(
            0b0100_0111_0010_1011, // sbci r18, 0x7B
            "--000010",
        );
        assert_eq!(mcu.read_register(18), 0x0000);
    }

    #[test]
    fn inc() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0x57);
        mcu.write_register(5, 0xFE);
        mcu.write_register(6, 0x7F);
        mcu.execute_and_assert_sreg(
            0b1001_010_10001_0011, // inc r17
            "---0000-",
        );
        assert_eq!(mcu.read_register(17), 0x58);

        mcu.execute_and_assert_sreg(
            0b1001_010_00101_0011, // inc r5
            "---1010-",
        );
        assert_eq!(mcu.read_register(5), 0xFF);

        mcu.execute_and_assert_sreg(
            0b1001_010_00101_0011, // inc r5
            "---0001-",
        );
        assert_eq!(mcu.read_register(5), 0x00);

        mcu.execute_and_assert_sreg(
            0b1001_010_00110_0011, // inc r6
            "---0110-",
        );
        assert_eq!(mcu.read_register(6), 0x80);
    }

    #[test]
    fn dec() {
        let mut mcu = Mcu::default();
        mcu.write_register(17, 0xAB);
        mcu.write_register(5, 0x02);
        mcu.write_register(6, 0x80);
        mcu.execute_and_assert_sreg(
            0b1001_010_10001_1010, // dec r17
            "---1010-",
        );
        assert_eq!(mcu.read_register(17), 0xAA);

        mcu.execute_and_assert_sreg(
            0b1001_010_00101_1010, // dec r5
            "---0000-",
        );
        assert_eq!(mcu.read_register(5), 0x01);

        mcu.execute_and_assert_sreg(
            0b1001_010_00101_1010, // dec r5
            "---0001-",
        );
        assert_eq!(mcu.read_register(5), 0x00);

        mcu.execute_and_assert_sreg(
            0b1001_010_00110_1010, // dec r6
            "---1100-",
        );
        assert_eq!(mcu.read_register(6), 0x7F);
    }
}
