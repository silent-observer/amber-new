use std::{
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

use super::Mcu;

struct HexLine {
    size: u8,
    addr: u16,
    record_type: u8,
    data: Vec<u8>,
}

impl FromStr for HexLine {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let size = u8::from_str_radix(&s[1..3], 16)?;
        let addr = u16::from_str_radix(&s[3..7], 16)?;
        let record_type = u8::from_str_radix(&s[7..9], 16)?;
        if size == 0 && addr == 0 && record_type == 1 {
            Ok(HexLine {
                size,
                addr,
                record_type,
                data: Vec::new(),
            })
        } else {
            assert_eq!(record_type, 0);
            let mut data = vec![0; size as usize];
            let mut index = 9;
            let mut checksum = size as u16 + (addr & 0xFF) + (addr >> 8) + record_type as u16;
            for i in 0..size {
                let x = u8::from_str_radix(&s[index..index + 2], 16)?;
                index += 2;
                data[i as usize] = x;
                checksum += x as u16;
            }
            checksum += u16::from_str_radix(&s[index..index + 2], 16)?;
            assert_eq!(checksum & 0xFF, 0);
            Ok(HexLine {
                size,
                addr,
                record_type,
                data,
            })
        }
    }
}

/// Reads flash from Intel .hex file
pub fn read_flash_hex(filename: &str) -> Vec<u16> {
    let mut flash = vec![0; 1024];
    let file = File::open(filename).unwrap();
    let lines = BufReader::new(file).lines();
    for line in lines {
        let l = line.unwrap();
        if l.starts_with(':') {
            let data: HexLine = l.parse().unwrap();
            match data.record_type {
                0 => {
                    let mut i = 0;
                    while i < data.size as usize {
                        let x = data.data[i] as u16 | (data.data[i + 1] as u16) << 8;
                        let addr = (data.addr as usize + i) >> 1;
                        if addr >= flash.len() {
                            flash.resize(addr + 1, 0);
                        }
                        flash[addr] = x;
                        i += 2;
                    }
                }
                1 => break,
                _ => panic!("Invalid record type"),
            }
        }
    }
    flash
}

impl Mcu {
    /// Reads flash from Intel .hex file
    pub fn load_flash_hex(&mut self, filename: &str) {
        let flash = read_flash_hex(filename);
        for (i, x) in flash.iter().enumerate() {
            self.write_flash(i as u32, *x);
        }
    }

    pub fn with_flash_hex(mut self: Self, filename: &str) -> Self {
        self.load_flash_hex(filename);
        self
    }
}
