use std::pin;

use crate::{
    events::{EventQueue, InternalEvent},
    module::{DataModule, Module, PinId, PortId, WireableModule},
    module_holder::PassiveModuleStore,
    module_id::{ModuleAddress, PinAddress},
    pin_state::WireState,
};

use self::{gpio::GpioBank, timer16::Timer16};

mod gpio;
pub mod timer16;

#[derive(Debug)]
pub struct IoController {
    module_id: ModuleAddress,
    pub module_store: PassiveModuleStore,
    gpio: [GpioBank; 11],
    interrupt: bool,

    timer1: Timer16,
    timer3: Timer16,
    timer4: Timer16,
    timer5: Timer16,
}

const BANK_A: u8 = 1;
const BANK_B: u8 = 2;
const BANK_C: u8 = 3;
const BANK_D: u8 = 4;
const BANK_E: u8 = 5;
const BANK_F: u8 = 6;
const BANK_G: u8 = 7;
const BANK_H: u8 = 8;
const BANK_J: u8 = 9;
const BANK_K: u8 = 10;
const BANK_L: u8 = 11;

const TIMER_1: u8 = 12;
const TIMER_3: u8 = 13;
const TIMER_4: u8 = 14;
const TIMER_5: u8 = 15;

const fn pin_id(bank: u8, pin: u8) -> u8 {
    if bank <= BANK_G {
        (bank - 1) * 8 + pin
    } else {
        (bank - 1) * 8 - 2 + pin
    }
}

impl IoController {
    pub fn new(module_id: ModuleAddress, queue: &mut EventQueue) -> Self {
        for i in 0..3 {
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_B, 5 + i)),
                &[
                    module_id.child_id(TIMER_1).with_pin(i),
                    module_id.child_id(BANK_B).with_pin(5 + i),
                ],
            );
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_E, 3 + i)),
                &[
                    module_id.child_id(TIMER_3).with_pin(i),
                    module_id.child_id(BANK_E).with_pin(3 + i),
                ],
            );
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_H, 3 + i)),
                &[
                    module_id.child_id(TIMER_4).with_pin(i),
                    module_id.child_id(BANK_H).with_pin(3 + i),
                ],
            );
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_L, 3 + i)),
                &[
                    module_id.child_id(TIMER_5).with_pin(i),
                    module_id.child_id(BANK_L).with_pin(3 + i),
                ],
            );
        }

        Self {
            module_id,
            module_store: PassiveModuleStore::new(module_id.child_id(0)),

            gpio: [
                GpioBank::new(module_id.child_id(BANK_A)),
                GpioBank::new(module_id.child_id(BANK_B)),
                GpioBank::new(module_id.child_id(BANK_C)),
                GpioBank::new(module_id.child_id(BANK_D)),
                GpioBank::new(module_id.child_id(BANK_E)),
                GpioBank::new(module_id.child_id(BANK_F)),
                GpioBank::new(module_id.child_id(BANK_G)),
                GpioBank::new(module_id.child_id(BANK_H)),
                GpioBank::new(module_id.child_id(BANK_J)),
                GpioBank::new(module_id.child_id(BANK_K)),
                GpioBank::new(module_id.child_id(BANK_L)),
            ],
            interrupt: false,

            timer1: Timer16::new(module_id.child_id(TIMER_1), module_id.with_event_port(0)),
            timer3: Timer16::new(module_id.child_id(TIMER_3), module_id.with_event_port(0)),
            timer4: Timer16::new(module_id.child_id(TIMER_4), module_id.with_event_port(0)),
            timer5: Timer16::new(module_id.child_id(TIMER_5), module_id.with_event_port(0)),
        }
    }
}

impl Module for IoController {
    #[inline]
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    fn handle_event(&mut self, event: InternalEvent, _queue: &mut EventQueue) {
        panic!("No internal event for IoController: {:?}", event);
    }

    fn find(&self, mut address: ModuleAddress) -> Option<&dyn Module> {
        let id = address.current();
        address.advance();
        match id {
            0 => self.module_store.find(address),
            1..=11 => self.gpio[id as usize - 1].find(address),
            12 => self.timer1.find(address),
            13 => self.timer3.find(address),
            14 => self.timer4.find(address),
            15 => self.timer5.find(address),
            _ => None,
        }
    }

    fn find_mut(&mut self, mut address: ModuleAddress) -> Option<&mut dyn Module> {
        if address.is_empty() {
            return Some(self);
        }

        let id = address.current();
        address.advance();
        match id {
            0 => self.module_store.find_mut(address),
            1..=11 => self.gpio[id as usize - 1].find_mut(address),
            12 => self.timer1.find_mut(address),
            13 => self.timer3.find_mut(address),
            14 => self.timer4.find_mut(address),
            15 => self.timer5.find_mut(address),
            _ => None,
        }
    }

    fn to_wireable(&mut self) -> Option<&mut dyn WireableModule> {
        Some(self)
    }
}

impl WireableModule for IoController {
    fn get_pin(&self, queue: &EventQueue, id: PinId) -> WireState {
        match id {
            0..=7 => self.gpio[0].get_pin(queue, id),         // Port A
            8..=15 => self.gpio[1].get_pin(queue, id - 8),    // Port B
            16..=23 => self.gpio[2].get_pin(queue, id - 16),  // Port C
            24..=31 => self.gpio[3].get_pin(queue, id - 24),  // Port D
            32..=39 => self.gpio[4].get_pin(queue, id - 32),  // Port E
            40..=47 => self.gpio[5].get_pin(queue, id - 40),  // Port F
            48..=53 => self.gpio[6].get_pin(queue, id - 48),  // Port G, ONLY 6 PINS
            54..=61 => self.gpio[7].get_pin(queue, id - 54),  // Port H
            62..=69 => self.gpio[8].get_pin(queue, id - 62),  // Port J
            70..=77 => self.gpio[9].get_pin(queue, id - 70),  // Port K
            78..=85 => self.gpio[10].get_pin(queue, id - 78), // Port L
            _ => panic!("Invalid port id: {}", id),
        }
    }

    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState) {
        match id {
            0..=7 => self.gpio[0].set_pin(queue, id, data), // Port A
            8..=15 => self.gpio[1].set_pin(queue, id - 8, data), // Port B
            16..=23 => self.gpio[2].set_pin(queue, id - 16, data), // Port C
            24..=31 => self.gpio[3].set_pin(queue, id - 24, data), // Port D
            32..=39 => self.gpio[4].set_pin(queue, id - 32, data), // Port E
            40..=47 => self.gpio[5].set_pin(queue, id - 40, data), // Port F
            48..=53 => self.gpio[6].set_pin(queue, id - 48, data), // Port G, ONLY 6 PINS
            54..=61 => self.gpio[7].set_pin(queue, id - 54, data), // Port H
            62..=69 => self.gpio[8].set_pin(queue, id - 62, data), // Port J
            70..=77 => self.gpio[9].set_pin(queue, id - 70, data), // Port K
            78..=85 => self.gpio[10].set_pin(queue, id - 78, data), // Port L
            _ => panic!("Invalid port id: {}", id),
        }
    }
}

impl DataModule for IoController {
    type PortType = u8;
    fn read_port(&self, queue: &EventQueue, id: PortId) -> u8 {
        match id {
            0x00..=0x1F => panic!("Invalid address {:#02X}", id),

            0x20..=0x22 => self.gpio[0].read_port(queue, id - 0x20), // Port A
            0x23..=0x25 => self.gpio[1].read_port(queue, id - 0x23), // Port B
            0x26..=0x28 => self.gpio[2].read_port(queue, id - 0x26), // Port C
            0x29..=0x2B => self.gpio[3].read_port(queue, id - 0x29), // Port D
            0x2C..=0x2E => self.gpio[4].read_port(queue, id - 0x2C), // Port E
            0x2F..=0x31 => self.gpio[5].read_port(queue, id - 0x2F), // Port F
            0x32..=0x34 => self.gpio[6].read_port(queue, id - 0x32), // Port G

            0x80..=0x8F => self.timer1.read_port(queue, id - 0x80), // Timer 1
            0x90..=0x9F => self.timer3.read_port(queue, id - 0x90), // Timer 3
            0xA0..=0xAF => self.timer4.read_port(queue, id - 0xA0), // Timer 4

            0x100..=0x102 => self.gpio[7].read_port(queue, id - 0x100), // Port H
            0x103..=0x105 => self.gpio[8].read_port(queue, id - 0x103), // Port J
            0x106..=0x108 => self.gpio[9].read_port(queue, id - 0x106), // Port K
            0x109..=0x10B => self.gpio[10].read_port(queue, id - 0x109), // Port L

            0x120..=0x12F => self.timer5.read_port(queue, id - 0x120), // Timer 5

            _ => panic!("Invalid address: {}", id),
        }
    }

    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: u8) {
        match id {
            0x00..=0x1F => panic!("Invalid address {:#02X}", id),

            0x20..=0x22 => self.gpio[0].write_port(queue, id - 0x20, data), // Port A
            0x23..=0x25 => self.gpio[1].write_port(queue, id - 0x23, data), // Port B
            0x26..=0x28 => self.gpio[2].write_port(queue, id - 0x26, data), // Port C
            0x29..=0x2B => self.gpio[3].write_port(queue, id - 0x29, data), // Port D
            0x2C..=0x2E => self.gpio[4].write_port(queue, id - 0x2C, data), // Port E
            0x2F..=0x31 => self.gpio[5].write_port(queue, id - 0x2F, data), // Port F
            0x32..=0x34 => self.gpio[6].write_port(queue, id - 0x32, data), // Port G

            0x80..=0x8F => self.timer1.write_port(queue, id - 0x80, data), // Timer 1
            0x90..=0x9F => self.timer3.write_port(queue, id - 0x90, data), // Timer 3
            0xA0..=0xAF => self.timer4.write_port(queue, id - 0xA0, data), // Timer 4

            0x100..=0x102 => self.gpio[7].write_port(queue, id - 0x100, data), // Port H
            0x103..=0x105 => self.gpio[8].write_port(queue, id - 0x103, data), // Port J
            0x106..=0x108 => self.gpio[9].write_port(queue, id - 0x106, data), // Port K
            0x109..=0x10B => self.gpio[10].write_port(queue, id - 0x109, data), // Port L

            0x120..=0x12F => self.timer5.write_port(queue, id - 0x120, data),

            _ => panic!("Invalid address: {}", id),
        }
    }
}

impl IoController {
    #[inline]
    pub fn read_port_internal(&self, queue: &EventQueue, id: PortId) -> u8 {
        assert!(id < 0x40);
        self.read_port(queue, id + 0x20)
    }

    #[inline]
    pub fn write_port_internal(&mut self, queue: &mut EventQueue, id: PortId, data: u8) {
        assert!(id < 0x40);
        self.write_port(queue, id + 0x20, data);
    }

    #[inline]
    pub fn has_interrupt(&self) -> bool {
        self.interrupt
    }

    pub fn get_interrupt_address(&mut self) -> Option<u16> {
        let mut result = None;
        let mut have_others = false;
        fn update(addr: u16, result: &mut Option<u16>, have_others: &mut bool, flag: &mut bool) {
            if *flag {
                *result = match result {
                    None => {
                        *flag = false;
                        Some(addr)
                    }
                    Some(x) => {
                        *have_others = true;
                        Some(*x)
                    }
                }
            }
        }

        // update(
        //     0x0020,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer1.interrupt_flags.input_capture,
        // );
        // update(
        //     0x0022,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer1.interrupt_flags.oc[0],
        // );
        // update(
        //     0x0024,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer1.interrupt_flags.oc[1],
        // );
        // update(
        //     0x0026,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer1.interrupt_flags.oc[2],
        // );
        // update(
        //     0x0028,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer1.interrupt_flags.overflow,
        // );

        // update(
        //     0x003E,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer3.interrupt_flags.input_capture,
        // );
        // update(
        //     0x0040,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer3.interrupt_flags.oc[0],
        // );
        // update(
        //     0x0042,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer3.interrupt_flags.oc[1],
        // );
        // update(
        //     0x0044,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer3.interrupt_flags.oc[2],
        // );
        // update(
        //     0x0046,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer3.interrupt_flags.overflow,
        // );

        // update(
        //     0x0052,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer4.interrupt_flags.input_capture,
        // );
        // update(
        //     0x0054,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer4.interrupt_flags.oc[0],
        // );
        // update(
        //     0x0056,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer4.interrupt_flags.oc[1],
        // );
        // update(
        //     0x0058,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer4.interrupt_flags.oc[2],
        // );
        // update(
        //     0x005A,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer4.interrupt_flags.overflow,
        // );

        // update(
        //     0x005C,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer5.interrupt_flags.input_capture,
        // );
        // update(
        //     0x005E,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer5.interrupt_flags.oc[0],
        // );
        // update(
        //     0x0060,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer5.interrupt_flags.oc[1],
        // );
        // update(
        //     0x0062,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer5.interrupt_flags.oc[2],
        // );
        // update(
        //     0x0064,
        //     &mut result,
        //     &mut have_others,
        //     &mut self.timer5.interrupt_flags.overflow,
        // );

        if !have_others {
            self.interrupt = false;
        }
        result
    }
}
