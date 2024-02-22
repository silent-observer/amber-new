pub mod first_pass;
pub mod instructions;

pub fn compile(flash: &[u16], start_address: usize) {
    let result = first_pass::first_pass(flash, start_address);
    for (i, instr) in result.block.instructions.iter().enumerate() {
        println!("{}: {}", i, instr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::avr::mcu::hex::{self, read_flash_hex};

    #[test]
    fn it_works() {
        let flash = read_flash_hex("hex/sort.hex");
        //compile(&flash, 0x0072);
        compile(&flash, 0x008D);
    }
}
