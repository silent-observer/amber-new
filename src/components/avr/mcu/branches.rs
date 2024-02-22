use bitfield::Bit;

use crate::components::avr::bit_helpers::{
    bit_field_combined, get_d_field, get_io5, get_rd_fields, is_two_word,
};

use super::Mcu;

const Z_REG: u16 = 30;

impl Mcu {
    pub fn instr_rjmp(&mut self, opcode: u16) -> u8 {
        if opcode == 0xCFFF {
            self.halted = true;
            return 2;
        }

        let k = opcode & 0x0FFF;

        if k.bit(11) {
            let k = (k ^ 0x0FFF) + 1;
            self.set_pc(self.pc - (k as u32) + 1);
        } else {
            self.set_pc(self.pc + (k as u32) + 1);
        }
        2
    }

    pub fn instr_ijmp(&mut self, _opcode: u16) -> u8 {
        self.set_pc(self.read_register_pair(Z_REG) as u32);
        2
    }

    pub fn instr_eijmp(&mut self, _opcode: u16) -> u8 {
        let z = self.read_register_pair(Z_REG);
        self.set_pc(self.eind_address(z));
        2
    }

    pub fn instr_jmp(&mut self, opcode: u16) -> u8 {
        let addr = (bit_field_combined(opcode, &[8..=4, 0..=0]) as u32) << 16
            | self.read_at_pc_offset(1) as u32;
        self.set_pc(addr);
        3
    }

    fn push_pc(&mut self) {
        self.pc += 1;
        self.write_at_sp_offset(0, (self.pc) as u8);
        self.write_at_sp_offset(-1, (self.pc >> 8) as u8);
        self.write_at_sp_offset(-2, (self.pc >> 16) as u8);
        self.pc -= 1;
        self.sp -= 3;
    }

    pub fn instr_rcall(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_rjmp(opcode) + 2
    }

    pub fn instr_icall(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_ijmp(opcode) + 2
    }

    pub fn instr_eicall(&mut self, opcode: u16) -> u8 {
        self.push_pc();
        self.instr_eijmp(opcode) + 2
    }

    pub fn instr_call(&mut self, opcode: u16) -> u8 {
        self.pc += 1;
        self.push_pc();
        self.pc -= 1;
        self.instr_jmp(opcode) + 2
    }

    pub fn instr_ret(&mut self, _opcode: u16) -> u8 {
        let v1 = self.read_at_sp_offset(1) as u32;
        let v2 = self.read_at_sp_offset(2) as u32;
        let v3 = self.read_at_sp_offset(3) as u32;
        self.sp += 3;

        self.set_pc(v1 << 16 | v2 << 8 | v3);

        5
    }

    pub fn instr_reti(&mut self, opcode: u16) -> u8 {
        self.sreg.set_i(true);
        self.instr_ret(opcode)
    }

    fn skip_if(&mut self, cond: bool) -> u8 {
        if cond {
            if is_two_word(self.read_at_pc_offset(1)) {
                self.pc += 3;
                3
            } else {
                self.pc += 2;
                2
            }
        } else {
            self.pc += 1;
            1
        }
    }

    fn jump_if(&mut self, cond: bool, k: u16) -> u8 {
        self.pc += 1;
        if cond {
            if k.bit(6) {
                let k = (k ^ 0x007F) + 1; // Negation
                self.set_pc(self.pc - (k as u32));
            } else {
                self.set_pc(self.pc + (k as u32));
            }
            2
        } else {
            1
        }
    }

    pub fn instr_cpse(&mut self, opcode: u16) -> u8 {
        let (r, d) = get_rd_fields(opcode, 5);
        let rr = self.read_register(r);
        let rd = self.read_register(d);

        self.skip_if(rr == rd)
    }

    pub fn instr_sbrc(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let b = opcode & 0x0007;
        let rd = self.read_register(d);

        self.skip_if(!rd.bit(b as usize))
    }

    pub fn instr_sbrs(&mut self, opcode: u16) -> u8 {
        let d = get_d_field(opcode, 5);
        let b = opcode & 0x0007;
        let rd = self.read_register(d);

        self.skip_if(rd.bit(b as usize))
    }

    pub fn instr_sbic(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let val = self.read_io(io);

        self.skip_if(!val.bit(b as usize))
    }

    pub fn instr_sbis(&mut self, opcode: u16) -> u8 {
        let io = get_io5(opcode);
        let b = opcode & 0x0007;
        let val = self.read_io(io);

        self.skip_if(val.bit(b as usize))
    }

    pub fn instr_brbc(&mut self, opcode: u16) -> u8 {
        let k = bit_field_combined(opcode, &[9..=3]);
        let s = bit_field_combined(opcode, &[2..=0]) as usize;

        self.jump_if(!self.sreg.bit(s), k)
    }

    pub fn instr_brbs(&mut self, opcode: u16) -> u8 {
        let k = bit_field_combined(opcode, &[9..=3]);
        let s = bit_field_combined(opcode, &[2..=0]) as usize;

        self.jump_if(self.sreg.bit(s), k)
    }

    pub fn execute_interrupt(&mut self, addr: u16) -> u8 {
        self.halted = false;
        self.pc -= 1;
        self.push_pc();
        self.set_pc(addr as u32);
        5
    }
}

#[cfg(test)]
mod tests {
    use crate::components::avr::sreg::StatusRegister;

    use super::*;

    #[test]
    fn rjmp() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.execute_and_assert_sreg(
            0xC123, // rjmp +0x123
            "--------",
        );
        assert_eq!(mcu.pc, 0x1358);

        mcu.execute_and_assert_sreg(
            0xC3AB, // rjmp +0x3AB
            "--------",
        );
        assert_eq!(mcu.pc, 0x1704);

        mcu.execute_and_assert_sreg(
            0xCA99, // rjmp -0x567
            "--------",
        );
        assert_eq!(mcu.pc, 0x119E);
    }

    #[test]
    fn ijmp() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_register_pair(Z_REG, 0x5678);
        mcu.execute_and_assert_sreg(
            0x9409, // ijmp
            "--------",
        );
        assert_eq!(mcu.pc, 0x5678);

        mcu.write_register_pair(Z_REG, 0xABCD);
        mcu.execute_and_assert_sreg(
            0x9409, // ijmp
            "--------",
        );
        assert_eq!(mcu.pc, 0xABCD);
    }

    #[test]
    fn eijmp() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.eind = 0x01;
        mcu.write_register_pair(Z_REG, 0x5678);
        mcu.execute_and_assert_sreg(
            0x9419, // eijmp
            "--------",
        );
        assert_eq!(mcu.pc, 0x15678);

        mcu.write_register_pair(Z_REG, 0xABCD);
        mcu.eind = 0x02;
        mcu.execute_and_assert_sreg(
            0x9419, // eijmp
            "--------",
        );
        assert_eq!(mcu.pc, 0x0ABCD);
    }

    #[test]
    fn jmp() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_flash(0x1235, 0x5678);
        mcu.execute_and_assert_sreg(
            0b1001_010_00001_110_1, // jmp 0x35678
            "--------",
        );
        assert_eq!(mcu.pc, 0x15678);

        mcu.write_flash(0x15679, 0x0003);
        mcu.execute_and_assert_sreg(
            0b1001_010_01101_110_0, // jmp 0x1A0003
            "--------",
        );
        assert_eq!(mcu.pc, 0x0003);
    }

    #[test]
    fn rcall() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sp = 0x21FF;
        mcu.execute_and_assert_sreg(
            0xD123, // rcall +0x123
            "--------",
        );
        assert_eq!(mcu.pc, 0x1358);
        assert_eq!(mcu.read(0x21FF), 0x35);
        assert_eq!(mcu.read(0x21FE), 0x12);
        assert_eq!(mcu.read(0x21FD), 0x00);

        mcu.execute_and_assert_sreg(
            0xD3AB, // rcall +0x3AB
            "--------",
        );
        assert_eq!(mcu.pc, 0x1704);
        assert_eq!(mcu.read(0x21FC), 0x59);
        assert_eq!(mcu.read(0x21FB), 0x13);
        assert_eq!(mcu.read(0x21FA), 0x00);

        mcu.execute_and_assert_sreg(
            0xDA99, // rcall -0x567
            "--------",
        );
        assert_eq!(mcu.pc, 0x119E);
        assert_eq!(mcu.read(0x21F9), 0x05);
        assert_eq!(mcu.read(0x21F8), 0x17);
        assert_eq!(mcu.read(0x21F7), 0x00);
    }

    #[test]
    fn icall() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sp = 0x21FF;
        mcu.write_register_pair(Z_REG, 0x5678);
        mcu.execute_and_assert_sreg(
            0x9509, // icall
            "--------",
        );
        assert_eq!(mcu.pc, 0x5678);
        assert_eq!(mcu.read(0x21FF), 0x35);
        assert_eq!(mcu.read(0x21FE), 0x12);
        assert_eq!(mcu.read(0x21FD), 0x00);

        mcu.write_register_pair(Z_REG, 0xABCD);
        mcu.execute_and_assert_sreg(
            0x9509, // icall
            "--------",
        );
        assert_eq!(mcu.pc, 0xABCD);
        assert_eq!(mcu.read(0x21FC), 0x79);
        assert_eq!(mcu.read(0x21FB), 0x56);
        assert_eq!(mcu.read(0x21FA), 0x00);
    }

    #[test]
    fn eicall() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sp = 0x21FF;
        mcu.eind = 0x01;
        mcu.write_register_pair(Z_REG, 0x5678);
        mcu.execute_and_assert_sreg(
            0x9519, // eicall
            "--------",
        );
        assert_eq!(mcu.pc, 0x15678);
        assert_eq!(mcu.read(0x21FF), 0x35);
        assert_eq!(mcu.read(0x21FE), 0x12);
        assert_eq!(mcu.read(0x21FD), 0x00);

        mcu.write_register_pair(Z_REG, 0xABCD);
        mcu.eind = 0x00;
        mcu.execute_and_assert_sreg(
            0x9519, // eicall
            "--------",
        );
        assert_eq!(mcu.pc, 0xABCD);
        assert_eq!(mcu.read(0x21FC), 0x79);
        assert_eq!(mcu.read(0x21FB), 0x56);
        assert_eq!(mcu.read(0x21FA), 0x01);
    }

    #[test]
    fn call() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sp = 0x21FF;
        mcu.write_flash(0x1235, 0x5678);
        mcu.execute_and_assert_sreg(
            0b1001_010_00001_111_1, // call 0x35678
            "--------",
        );
        assert_eq!(mcu.pc, 0x15678);
        assert_eq!(mcu.read(0x21FF), 0x36);
        assert_eq!(mcu.read(0x21FE), 0x12);
        assert_eq!(mcu.read(0x21FD), 0x00);

        mcu.write_flash(0x15679, 0x0003);
        mcu.execute_and_assert_sreg(
            0b1001_010_01101_111_0, // call 0x1A0003
            "--------",
        );
        assert_eq!(mcu.pc, 0x0003);
        assert_eq!(mcu.read(0x21FC), 0x7A);
        assert_eq!(mcu.read(0x21FB), 0x56);
        assert_eq!(mcu.read(0x21FA), 0x01);
    }

    #[test]
    fn ret() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sp = 0x21FF;
        mcu.write_flash(0x1235, 0x5678);
        mcu.execute_and_assert_sreg(
            0b1001_010_00001_111_1, // call 0x35678
            "--------",
        );
        assert_eq!(mcu.pc, 0x15678);
        assert_eq!(mcu.read(0x21FF), 0x36);
        assert_eq!(mcu.read(0x21FE), 0x12);
        assert_eq!(mcu.read(0x21FD), 0x00);

        mcu.write_flash(0x15679, 0x0003);
        mcu.execute_and_assert_sreg(
            0b1001_010_01101_111_0, // call 0x1A0003
            "--------",
        );
        assert_eq!(mcu.pc, 0x0003);
        assert_eq!(mcu.read(0x21FC), 0x7A);
        assert_eq!(mcu.read(0x21FB), 0x56);
        assert_eq!(mcu.read(0x21FA), 0x01);

        mcu.execute_and_assert_sreg(
            0x9508, // ret
            "--------",
        );
        assert_eq!(mcu.pc, 0x1567A);

        mcu.execute_and_assert_sreg(
            0x9508, // ret
            "--------",
        );
        assert_eq!(mcu.pc, 0x1236);
    }

    #[test]
    fn reti() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sp = 0x21FF;
        mcu.write_flash(0x1235, 0x5678);
        mcu.execute_and_assert_sreg(
            0b1001_010_00001_111_1, // call 0x35678
            "--------",
        );
        assert_eq!(mcu.pc, 0x15678);
        assert_eq!(mcu.read(0x21FF), 0x36);
        assert_eq!(mcu.read(0x21FE), 0x12);
        assert_eq!(mcu.read(0x21FD), 0x00);

        mcu.write_flash(0x15679, 0x0003);
        mcu.execute_and_assert_sreg(
            0b1001_010_01101_111_0, // call 0x1A0003
            "--------",
        );
        assert_eq!(mcu.pc, 0x0003);
        assert_eq!(mcu.read(0x21FC), 0x7A);
        assert_eq!(mcu.read(0x21FB), 0x56);
        assert_eq!(mcu.read(0x21FA), 0x01);

        mcu.execute_and_assert_sreg(
            0x9518, // reti
            "1-------",
        );
        assert_eq!(mcu.pc, 0x1567A);

        mcu.execute_and_assert_sreg(
            0x9518, // reti
            "1-------",
        );
        assert_eq!(mcu.pc, 0x1236);
    }

    #[test]
    fn cpse() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_register(1, 0x12);
        mcu.write_register(2, 0x34);
        mcu.write_register(3, 0x34);
        mcu.write_flash(0x1235, 0x9508); // ret
        mcu.execute_and_assert_sreg(
            0b0001_00_0_00001_0010, // cpse r1, r2
            "--------",
        );
        assert_eq!(mcu.pc, 0x1235);

        mcu.write_flash(0x1236, 0x9508); // ret
        mcu.execute_and_assert_sreg(
            0b0001_00_0_00010_0011, // cpse r2, r3
            "--------",
        );
        assert_eq!(mcu.pc, 0x1237);

        mcu.write_flash(0x1238, 0b1001_010_00001_111_1); // call
        mcu.execute_and_assert_sreg(
            0b0001_00_0_00010_0011, // cpse r2, r3
            "--------",
        );
        assert_eq!(mcu.pc, 0x123A);
    }

    #[test]
    fn sbrc() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_register(1, 0x12);
        mcu.write_flash(0x1235, 0x9508); // ret
        mcu.execute_and_assert_sreg(
            0b1111_110_00001_0001, // sbrc r1, 1
            "--------",
        );
        assert_eq!(mcu.pc, 0x1235);

        mcu.write_flash(0x1236, 0x9508); // ret
        mcu.execute_and_assert_sreg(
            0b1111_110_00001_0000, // sbrc r1, 0
            "--------",
        );
        assert_eq!(mcu.pc, 0x1237);

        mcu.write_flash(0x1238, 0b1001_010_00001_111_1); // call
        mcu.execute_and_assert_sreg(
            0b1111_110_00001_0000, // sbrc r1, 0
            "--------",
        );
        assert_eq!(mcu.pc, 0x123A);
    }

    #[test]
    fn sbrs() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.write_register(1, 0x12);
        mcu.write_flash(0x1235, 0x9508); // ret
        mcu.execute_and_assert_sreg(
            0b1111_111_00001_0000, // sbrs r1, 0
            "--------",
        );
        assert_eq!(mcu.pc, 0x1235);

        mcu.write_flash(0x1236, 0x9508); // ret
        mcu.execute_and_assert_sreg(
            0b1111_111_00001_0001, // sbrs r1, 1
            "--------",
        );
        assert_eq!(mcu.pc, 0x1237);

        mcu.write_flash(0x1238, 0b1001_010_00001_111_1); // call
        mcu.execute_and_assert_sreg(
            0b1111_111_00001_0001, // sbrs r1, 1
            "--------",
        );
        assert_eq!(mcu.pc, 0x123A);
    }

    // #[test]
    // fn sbic() {
    //     let mut io = MockIoControllerTrait::new();

    //     io.expect_read_internal_u8()
    //       .with(eq(9))
    //       .times(3)
    //       .return_const(0x12);

    //     let mut mcu: Mcu = Mcu::new(io);
    //     mcu.pc = 0x1234;
    //     mcu.write_flash(0x1235, 0x9508); // ret
    //     mcu.execute_and_assert_sreg(
    //         0b1001_1001_0100_1001, // sbic r1, 1
    //         "--------");
    //     assert_eq!(mcu.pc, 0x1235);

    //     mcu.write_flash(0x1236, 0x9508); // ret
    //     mcu.execute_and_assert_sreg(
    //         0b1001_1001_0100_1000, // sbic r1, 0
    //         "--------");
    //     assert_eq!(mcu.pc, 0x1237);

    //     mcu.write_flash(0x1238, 0b1001_010_00001_111_1); // call
    //     mcu.execute_and_assert_sreg(
    //         0b1001_1001_0100_1000, // sbic r1, 0
    //         "--------");
    //     assert_eq!(mcu.pc, 0x123A);
    // }

    // #[test]
    // fn sbis() {
    //     let mut io = MockIoControllerTrait::new();

    //     io.expect_read_internal_u8()
    //       .with(eq(9))
    //       .times(3)
    //       .return_const(0x12);

    //     let mut mcu: Mcu = Mcu::new(io);

    //     mcu.pc = 0x1234;
    //     mcu.write_register(1, 0x12);
    //     mcu.write_flash(0x1235, 0x9508); // ret
    //     mcu.execute_and_assert_sreg(
    //         0b1001_1011_0100_1000, // sbis r1, 0
    //         "--------");
    //     assert_eq!(mcu.pc, 0x1235);

    //     mcu.write_flash(0x1236, 0x9508); // ret
    //     mcu.execute_and_assert_sreg(
    //         0b1001_1011_0100_1001, // sbis r1, 1
    //         "--------");
    //     assert_eq!(mcu.pc, 0x1237);

    //     mcu.write_flash(0x1238, 0b1001_010_00001_111_1); // call
    //     mcu.execute_and_assert_sreg(
    //         0b1001_1011_0100_1001, // sbis r1, 1
    //         "--------");
    //     assert_eq!(mcu.pc, 0x123A);
    // }

    #[test]
    fn brbc() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sreg = StatusRegister(0x56);
        mcu.execute_and_assert_sreg(
            0b1111_01_0010010_001, // brbc 1, +0x12
            "--------",
        );
        assert_eq!(mcu.pc, 0x1235);

        mcu.execute_and_assert_sreg(
            0b1111_01_0010010_000, // brbc 0, +0x12
            "--------",
        );
        assert_eq!(mcu.pc, 0x1248);

        mcu.execute_and_assert_sreg(
            0b1111_01_1000110_000, // brbc 0, -0x3A
            "--------",
        );
        assert_eq!(mcu.pc, 0x120F);
    }

    #[test]
    fn brbs() {
        let mut mcu = Mcu::default();
        mcu.pc = 0x1234;
        mcu.sreg = StatusRegister(0x56);
        mcu.execute_and_assert_sreg(
            0b1111_00_0010010_000, // brbs 0, +0x12
            "--------",
        );
        assert_eq!(mcu.pc, 0x1235);

        mcu.execute_and_assert_sreg(
            0b1111_00_0010010_001, // brbs 1, +0x12
            "--------",
        );
        assert_eq!(mcu.pc, 0x1248);

        mcu.execute_and_assert_sreg(
            0b1111_00_1000110_001, // brbs 1, -0x3A
            "--------",
        );
        assert_eq!(mcu.pc, 0x120F);
    }
}
