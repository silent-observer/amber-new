use std::mem;

use iced_x86::{Formatter, NasmFormatter};
use itertools::Itertools;
use memmap::{Mmap, MmapMut, MmapOptions};

use self::assembly::JitAssembler;

use super::Mcu;

pub mod add_sub;
pub mod assembly;
pub mod bitops;
pub mod branches;
pub mod first_pass;
pub mod instructions;
pub mod logical;
pub mod mul;
pub mod transfer;

pub struct JitBlock {
    buffer: Mmap,
}

impl JitBlock {
    pub fn compile(flash: &[u16], start_address: usize) -> Self {
        let result = first_pass::first_pass(flash, start_address);
        for (i, instr) in result.block.instructions.iter().enumerate() {
            println!("{}: {}", i, instr);
        }

        let mut j = JitAssembler::new();
        j.prologue().unwrap();
        for i in result.block.instructions.iter() {
            j.create_addr_label(i);
        }
        for i in result.block.instructions.into_iter() {
            println!("{}", i);
            j.instr(i).unwrap();
        }
        j.epilogue().unwrap();

        // let mut formatter = NasmFormatter::new();
        // let mut output = String::new();
        // for (i, instr) in j.a.instructions().iter().enumerate() {
        //     output.clear();
        //     formatter.format(instr, &mut output);
        //     println!("{}: {}", i, &output);
        // }

        let bytes = j.a.assemble(0x7FFF_0000_0000).unwrap();
        //let bytes_len = j.a.assemble(0x7FFF_0000_0000).unwrap().len();
        let mut buffer = MmapMut::map_anon(bytes.len()).expect("Couldn't mmap buffer");
        // let addr = &buffer[0] as *const u8 as usize;
        // assert_eq!(bytes.len(), bytes_len);
        // println!("{:02X?}", bytes);
        buffer.clone_from_slice(&bytes);

        Self {
            buffer: buffer.make_exec().unwrap(),
        }
    }

    pub fn exec(&self, mcu: &mut Mcu) -> u8 {
        let ptr = self.buffer.as_ptr();
        let f: extern "C" fn(*mut Mcu, *mut u8, *mut u16) -> u8 = unsafe { mem::transmute(ptr) };
        f(mcu, mcu.sram.as_mut_ptr(), mcu.flash.as_mut_ptr())
    }
}

#[cfg(test)]
mod tests {
    use iced_x86::code_asm::{eax, ebx, ecx, CodeAssembler};

    use super::*;
    use crate::{
        clock::TimeDiff,
        components::{
            avr::mcu::hex::{self, read_flash_hex},
            led::Led,
        },
        events::EventQueue,
        module::ActiveModule,
        module_id::PinAddress,
        wiring::{InboxTable, WiringTable},
    };

    //#[test]
    // fn stupid() {
    //     let flash = [
    //         0xEFCF, 0xE2D1, 0xBFDE, 0xBFCD, 0x940E, 0x00C5, 0x940C, 0x00F8,
    //     ];
    //     //compile(&flash, 0x0072);
    //     //JitBlock::compile(&flash, 0x008D);

    //     let mut it = InboxTable::new();
    //     let mut wt = WiringTable::new();

    //     let event_queue = EventQueue::new(TimeDiff(1), 0, it.add_listener(0));
    //     let mut mcu = Mcu::new(event_queue).with_flash_hex("./hex/sort.hex");
    //     let led = PinAddress::from(mcu.module_store().add_module(|id| Led::new(id)), 0);

    //     wt.add_wire(PinAddress::from(&mcu, 15), vec![led]);

    //     it.save();
    //     wt.save();

    //     let block = JitBlock::compile(&flash, 0x0000);
    //     println!("Done: {}", block.exec(&mut mcu));
    //     println!("{:04X}", mcu.pc);
    // }
    #[test]
    fn it_works() {
        let flash = read_flash_hex("hex/sort.hex");
        //compile(&flash, 0x0072);
        //JitBlock::compile(&flash, 0x008D);

        let mut it = InboxTable::new();
        let mut wt = WiringTable::new();

        let event_queue = EventQueue::new(TimeDiff(1), 0, it.add_listener(0));
        let mut mcu = Mcu::new(event_queue).with_flash_hex("./hex/sort.hex");
        let led = PinAddress::from(mcu.module_store().add_module(|id| Led::new(id)), 0);

        wt.add_wire(PinAddress::from(&mcu, 15), vec![led]);

        it.save();
        wt.save();

        let block1 = JitBlock::compile(&flash, 0x0000);
        println!("Done 1: {}", block1.exec(&mut mcu));
        println!("{:04X}", mcu.pc);

        let block2 = JitBlock::compile(&flash, mcu.pc as usize);
        println!("Done 2: {}", block2.exec(&mut mcu));
        println!("{:04X}", mcu.pc);

        let block3 = JitBlock::compile(&flash, mcu.pc as usize);
        println!("Done 3: {}", block3.exec(&mut mcu));
        println!("{:04X}", mcu.pc);

        let block4 = JitBlock::compile(&flash, mcu.pc as usize);
        println!("Done 4: {}", block4.exec(&mut mcu));
        println!("{:04X}", mcu.pc);

        let block5 = JitBlock::compile(&flash, mcu.pc as usize);
        println!("Done 5: {}", block5.exec(&mut mcu));
        println!("{:04X}", mcu.pc);
    }
}
