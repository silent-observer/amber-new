use bitfield::bitfield;

bitfield! {
    #[derive(Clone, Copy)]
    pub struct StatusRegister(u8);
    impl Debug;
    pub c, set_c: 0;
    pub z, set_z: 1;
    pub n, set_n: 2;
    pub v, set_v: 3;
    pub s, set_s: 4;
    pub h, set_h: 5;
    pub t, set_t: 6;
    pub i, set_i: 7;
}

#[cfg(test)]
const BIT_NAMES: [&str; 8] = ["I", "T", "H", "S", "V", "N", "Z", "C"];

#[cfg(test)]
pub mod test_helper {
    use bitfield::Bit;

    use super::*;

    pub fn assert_sreg(sreg: &StatusRegister, sreg_initial: &StatusRegister, mask: &'static str) {
        assert!(mask.len() == 8);
        for (i, c) in mask.chars().enumerate() {
            if c == '0' {
                assert_eq!(
                    sreg.bit(7 - i),
                    false,
                    "Expected 0 in flag {}, got {}",
                    BIT_NAMES[i],
                    sreg.bit(7 - i) as u8
                )
            } else if c == '1' {
                assert_eq!(
                    sreg.bit(7 - i),
                    true,
                    "Expected 1 in flag {}, got {}",
                    BIT_NAMES[i],
                    sreg.bit(7 - i) as u8
                )
            } else if c == '-' {
                assert_eq!(
                    sreg.bit(7 - i),
                    sreg_initial.bit(7 - i),
                    "Expected no change in flag {}, got {} instead of {}",
                    BIT_NAMES[i],
                    sreg.bit(7 - i) as u8,
                    sreg_initial.bit(7 - i) as u8
                )
            }
        }
    }
}
