use std::ops::RangeInclusive;

#[inline]
pub fn bit_field(x: u16, r: &'static RangeInclusive<usize>) -> u16 {
    (x as u16) << (16 - r.start() - 1) >> (16 - r.start() - 1 + r.end())
}

#[inline]
pub fn bit_field_combined(x: u16, data: &'static [RangeInclusive<usize>]) -> u16 {
    let mut result = 0;
    let mut offset = 0;
    for r in data.iter().rev() {
        result |= bit_field(x, r) << offset;
        offset += r.start() - r.end() + 1;
    }
    result
}

#[inline]
pub fn get_r_field(opcode: u16, size: usize) -> u16 {
    match size {
        3 => 0x10 | bit_field_combined(opcode, &[2..=0]),
        4 => 0x10 | bit_field_combined(opcode, &[3..=0]),
        5 => bit_field_combined(opcode, &[9..=9, 3..=0]),
        _ => panic!("Invalid R field size"),
    }
}

#[inline]
pub fn get_d_field(opcode: u16, size: usize) -> u16 {
    match size {
        2 => 0x18 | bit_field_combined(opcode, &[5..=4]) << 1,
        3 => 0x10 | bit_field_combined(opcode, &[6..=4]),
        4 => 0x10 | bit_field_combined(opcode, &[7..=4]),
        5 => bit_field_combined(opcode, &[8..=4]),
        _ => panic!("Invalid R field size"),
    }
}

#[inline]
pub fn get_rd_fields(opcode: u16, size: usize) -> (u16, u16) {
    (get_r_field(opcode, size), get_d_field(opcode, size))
}

#[inline]
pub fn get_k6(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[7..=6, 3..=0]) as u8
}

#[inline]
pub fn get_k8(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[11..=8, 3..=0]) as u8
}

#[inline]
pub fn get_io6(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[10..=9, 3..=0]) as u8
}

#[inline]
pub fn get_io5(opcode: u16) -> u8 {
    bit_field_combined(opcode, &[7..=3]) as u8
}

#[inline]
pub fn is_two_word(opcode: u16) -> bool {
    (opcode & 0xFC0F) == 0x9000 || (opcode & 0xFE0C) == 0x940C
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_field() {
        let x = 0x1234;
        assert_eq!(bit_field(x, &(3..=0)), 0x4);
        assert_eq!(bit_field(x, &(7..=4)), 0x3);
        assert_eq!(bit_field(x, &(11..=8)), 0x2);
        assert_eq!(bit_field(x, &(15..=12)), 0x1);

        assert_eq!(bit_field(x, &(7..=0)), 0x34);
        assert_eq!(bit_field(x, &(11..=4)), 0x23);
        assert_eq!(bit_field(x, &(15..=8)), 0x12);

        assert_eq!(bit_field(x, &(11..=0)), 0x234);
        assert_eq!(bit_field(x, &(15..=4)), 0x123);

        assert_eq!(bit_field(0b0001_0010_0011_0100, &(5..=2)), 0b1101);
    }

    #[test]
    fn test_bit_field_combined() {
        let x = 0x1234;
        assert_eq!(bit_field_combined(x, &[11..=8, 3..=0]), 0x24);
        assert_eq!(bit_field_combined(x, &[3..=0, 11..=8]), 0x42);
        assert_eq!(bit_field_combined(x, &[3..=0, 15..=12]), 0x41);

        assert_eq!(
            bit_field_combined(0b0001_0010_0011_0100, &[5..=2, 7..=7, 10..=9]),
            0b1101_0_01
        );
    }
}
