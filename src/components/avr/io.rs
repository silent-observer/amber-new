use crate::{
    events::{Event, EventQueue, EventReceiver},
    module::{DataModule, Module, PinId, PortId, WireableModule},
    module_holder::PassiveModuleStore,
    module_id::ModuleId,
    pin_state::WireState,
};

use self::gpio::GpioBank;

mod gpio;

#[derive(Debug)]
pub struct IoController {
    module_id: ModuleId,
    pub module_store: PassiveModuleStore,
    gpio: [GpioBank; 11],
    interrupt: bool,
}

impl IoController {
    pub fn new() -> Self {
        Self {
            gpio: [
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
                GpioBank::new(),
            ],
            interrupt: false,
            module_id: ModuleId::default(),
            module_store: PassiveModuleStore::new(),
        }
    }
}

impl Module for IoController {
    #[inline]
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    fn set_module_id(&mut self, module_id: ModuleId) {
        self.module_id = module_id;
        self.module_store.set_module_id(module_id.child_id(0));
        for (i, bank) in self.gpio.iter_mut().enumerate() {
            bank.set_module_id(module_id.child_id(i as u8 + 1));
        }
    }
}

impl EventReceiver for IoController {
    fn receive_event(&mut self, mut event: Event, queue: &mut EventQueue) {
        let id = event.receiver_id.current();
        event.receiver_id.advance();
        match id {
            0 => self.module_store.receive_event(event, queue),
            1..=11 => self.gpio[id as usize - 1].receive_event(event, queue),
            _ => panic!("Invalid event receiver: {}", event.receiver_id),
        }
    }
}

impl WireableModule for IoController {
    fn get_pin(&self, id: PinId) -> WireState {
        match id {
            0..=7 => self.gpio[0].get_pin(id),         // Port A
            8..=15 => self.gpio[1].get_pin(id - 8),    // Port B
            16..=23 => self.gpio[2].get_pin(id - 16),  // Port C
            24..=31 => self.gpio[3].get_pin(id - 24),  // Port D
            32..=39 => self.gpio[4].get_pin(id - 32),  // Port E
            40..=47 => self.gpio[5].get_pin(id - 40),  // Port F
            48..=53 => self.gpio[6].get_pin(id - 48),  // Port G, ONLY 6 PINS
            54..=61 => self.gpio[7].get_pin(id - 54),  // Port H
            62..=69 => self.gpio[8].get_pin(id - 62),  // Port J
            70..=77 => self.gpio[9].get_pin(id - 70),  // Port K
            78..=85 => self.gpio[10].get_pin(id - 78), // Port L
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

    fn get_pin_module(&self, id: PinId) -> Option<ModuleId> {
        match id {
            0..=7 => self.gpio[0].get_pin_module(id),         // Port A
            8..=15 => self.gpio[1].get_pin_module(id - 8),    // Port B
            16..=23 => self.gpio[2].get_pin_module(id - 16),  // Port C
            24..=31 => self.gpio[3].get_pin_module(id - 24),  // Port D
            32..=39 => self.gpio[4].get_pin_module(id - 32),  // Port E
            40..=47 => self.gpio[5].get_pin_module(id - 40),  // Port F
            48..=53 => self.gpio[6].get_pin_module(id - 48),  // Port G, ONLY 6 PINS
            54..=61 => self.gpio[7].get_pin_module(id - 54),  // Port H
            62..=69 => self.gpio[8].get_pin_module(id - 62),  // Port J
            70..=77 => self.gpio[9].get_pin_module(id - 70),  // Port K
            78..=85 => self.gpio[10].get_pin_module(id - 78), // Port L
            _ => None,
        }
    }
}

impl DataModule for IoController {
    type PortType = u8;
    fn read_port(&self, id: PortId) -> u8 {
        match id {
            0x00..=0x1F => panic!("Invalid address {:#02X}", id),

            0x20..=0x22 => self.gpio[0].read_port(id - 0x20), // Port A
            0x23..=0x25 => self.gpio[1].read_port(id - 0x23), // Port B
            0x26..=0x28 => self.gpio[2].read_port(id - 0x26), // Port C
            0x29..=0x2B => self.gpio[3].read_port(id - 0x29), // Port D
            0x2C..=0x2E => self.gpio[4].read_port(id - 0x2C), // Port E
            0x2F..=0x31 => self.gpio[5].read_port(id - 0x2F), // Port F
            0x32..=0x34 => self.gpio[6].read_port(id - 0x32), // Port G

            0x100..=0x102 => self.gpio[7].read_port(id - 0x100), // Port H
            0x103..=0x105 => self.gpio[8].read_port(id - 0x103), // Port J
            0x106..=0x108 => self.gpio[9].read_port(id - 0x106), // Port K
            0x109..=0x10B => self.gpio[10].read_port(id - 0x109), // Port L

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

            0x100..=0x102 => self.gpio[7].write_port(queue, id - 0x100, data), // Port H
            0x103..=0x105 => self.gpio[8].write_port(queue, id - 0x103, data), // Port J
            0x106..=0x108 => self.gpio[9].write_port(queue, id - 0x106, data), // Port K
            0x109..=0x10B => self.gpio[10].write_port(queue, id - 0x109, data), // Port L

            _ => panic!("Invalid address: {}", id),
        }
    }
}

impl IoController {
    #[inline]
    pub fn read_port_internal(&self, id: PortId) -> u8 {
        assert!(id < 0x40);
        self.read_port(id + 0x20)
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
