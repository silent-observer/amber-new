#[derive(Debug, Clone)]
pub struct RegisterFile {
    pub regs: [u8; 32],
}

impl RegisterFile {
    pub fn new() -> RegisterFile {
        RegisterFile { regs: [0; 32] }
    }

    pub fn read_u16(&self, i: usize) -> u16 {
        (self.regs[i + 1] as u16) << 8 | (self.regs[i] as u16)
    }

    pub fn write_u16(&mut self, i: usize, val: u16) {
        self.regs[i] = val as u8;
        self.regs[i + 1] = (val >> 8) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reg_read_u16() {
        let mut reg_file = RegisterFile::new();
        reg_file.regs[26] = 0x34;
        reg_file.regs[27] = 0x12;
        assert_eq!(reg_file.read_u16(26), 0x1234);
    }

    #[test]
    fn reg_write_u16() {
        let mut reg_file = RegisterFile::new();
        reg_file.write_u16(26, 0x1234);
        assert_eq!(reg_file.regs[26], 0x34);
        assert_eq!(reg_file.regs[27], 0x12);
    }
}
