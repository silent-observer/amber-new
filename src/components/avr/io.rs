use std::{any::Any, mem::transmute};

use kanal::Sender;
use uart::Uart;

use crate::{
    clock::Timestamp,
    events::{EventQueue, InternalEvent},
    module::{DataModule, Module, PinId, PortId, WireableModule},
    module_holder::PassiveModuleStore,
    module_id::ModuleAddress,
    pin_state::WireState,
    vcd::{VcdEvent, VcdSender, VcdSignal},
};

use self::{gpio::GpioBank, timer16::Timer16};

mod gpio;
pub mod timer16;
pub mod uart;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepMode {
    Idle = 0,
    ADCNoiseReduction = 1,
    PowerDown = 2,
    PowerSave = 3,
    Reserved1 = 4,
    Reserved2 = 5,
    Standby = 6,
    ExtendedStandby = 7,
}

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

    uart0: Uart,
    uart1: Uart,
    uart2: Uart,
    uart3: Uart,

    pub sleep_mode: SleepMode,
    pub sleep_enabled: bool,
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

const UART_0: u8 = 16;
const UART_1: u8 = 17;
const UART_2: u8 = 18;
const UART_3: u8 = 19;

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

        for i in 0..3 {
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_E, i)),
                &[
                    module_id.child_id(UART_0).with_pin(i),
                    module_id.child_id(BANK_E).with_pin(i),
                ],
            );
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_H, i)),
                &[
                    module_id.child_id(UART_2).with_pin(i),
                    module_id.child_id(BANK_H).with_pin(i),
                ],
            );
            queue.register_multiplexer(
                module_id.with_pin(pin_id(BANK_J, i)),
                &[
                    module_id.child_id(UART_3).with_pin(i),
                    module_id.child_id(BANK_J).with_pin(i),
                ],
            );
        }

        queue.register_multiplexer(
            module_id.with_pin(pin_id(BANK_D, 2)),
            &[
                module_id.child_id(UART_1).with_pin(Uart::RX_PIN),
                module_id.child_id(BANK_D).with_pin(2),
            ],
        );
        queue.register_multiplexer(
            module_id.with_pin(pin_id(BANK_D, 3)),
            &[
                module_id.child_id(UART_1).with_pin(Uart::TX_PIN),
                module_id.child_id(BANK_D).with_pin(3),
            ],
        );
        queue.register_multiplexer(
            module_id.with_pin(pin_id(BANK_D, 5)),
            &[
                module_id.child_id(UART_1).with_pin(Uart::XCK_PIN),
                module_id.child_id(BANK_D).with_pin(5),
            ],
        );

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

            uart0: Uart::new(module_id.child_id(UART_0), module_id.with_event_port(0)),
            uart1: Uart::new(module_id.child_id(UART_1), module_id.with_event_port(0)),
            uart2: Uart::new(module_id.child_id(UART_2), module_id.with_event_port(0)),
            uart3: Uart::new(module_id.child_id(UART_3), module_id.with_event_port(0)),

            sleep_mode: SleepMode::Idle,
            sleep_enabled: false,
        }
    }
}

impl VcdSender for IoController {
    fn register_vcd(&mut self, sender: Sender<VcdEvent>, start_id: i32) -> (Vec<VcdSignal>, i32) {
        let mut signals = Vec::new();
        let mut count = 0;
        for (i, m) in &mut self.gpio.iter_mut().enumerate() {
            const GPIO_NAMES: &[u8] = "abcdefghjkl".as_bytes();
            let (new_signals, new_count) = m.register_vcd(sender.clone(), start_id + count);
            signals.push(VcdSignal::Scope {
                name: format!("gpio_{}", GPIO_NAMES[i] as char),
                children: new_signals,
            });
            count += new_count;
        }
        (signals, count)
    }

    fn vcd_sender(&self) -> Option<&Sender<VcdEvent>> {
        None
    }
}

impl Module for IoController {
    #[inline]
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_event(&mut self, event: InternalEvent, _queue: &mut EventQueue, _t: Timestamp) {
        assert_eq!(event.receiver_id.event_port_id, 0);
        self.interrupt = true;
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
            16 => self.uart0.find(address),
            17 => self.uart1.find(address),
            18 => self.uart2.find(address),
            19 => self.uart3.find(address),
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
            16 => self.uart0.find_mut(address),
            17 => self.uart1.find_mut(address),
            18 => self.uart2.find_mut(address),
            19 => self.uart3.find_mut(address),
            _ => None,
        }
    }

    fn to_wireable_mut(&mut self) -> Option<&mut dyn WireableModule> {
        Some(self)
    }
    fn to_wireable(&self) -> Option<&dyn WireableModule> {
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
    fn read_port(&mut self, queue: &mut EventQueue, id: PortId) -> u8 {
        match id {
            0x00..=0x1F => panic!("Invalid address {:#02X}", id),

            0x20..=0x22 => self.gpio[0].read_port(queue, id - 0x20), // Port A
            0x23..=0x25 => self.gpio[1].read_port(queue, id - 0x23), // Port B
            0x26..=0x28 => self.gpio[2].read_port(queue, id - 0x26), // Port C
            0x29..=0x2B => self.gpio[3].read_port(queue, id - 0x29), // Port D
            0x2C..=0x2E => self.gpio[4].read_port(queue, id - 0x2C), // Port E
            0x2F..=0x31 => self.gpio[5].read_port(queue, id - 0x2F), // Port F
            0x32..=0x34 => self.gpio[6].read_port(queue, id - 0x32), // Port G

            0x35 => todo!(),
            0x36 => self.timer1.read_port(queue, Timer16::TIFR_PORT),
            0x37 => todo!(),
            0x38 => self.timer3.read_port(queue, Timer16::TIFR_PORT),
            0x39 => self.timer4.read_port(queue, Timer16::TIFR_PORT),
            0x3A => self.timer5.read_port(queue, Timer16::TIFR_PORT),

            0x53 => {
                // SMCR
                let sm = self.sleep_mode as u8;
                let se = self.sleep_enabled as u8;
                sm << 1 | se
            }

            0x6E => todo!(),
            0x6F => self.timer1.read_port(queue, Timer16::TIMSK_PORT),
            0x70 => todo!(),
            0x71 => self.timer3.read_port(queue, Timer16::TIMSK_PORT),
            0x72 => self.timer4.read_port(queue, Timer16::TIMSK_PORT),
            0x73 => self.timer5.read_port(queue, Timer16::TIMSK_PORT),

            0x80..=0x8F => self.timer1.read_port(queue, id - 0x80), // Timer 1
            0x90..=0x9F => self.timer3.read_port(queue, id - 0x90), // Timer 3
            0xA0..=0xAF => self.timer4.read_port(queue, id - 0xA0), // Timer 4

            0xC0..=0xC7 => self.uart0.read_port(queue, id - 0xC0), // UART 0
            0xC8..=0xCF => self.uart1.read_port(queue, id - 0xC8), // UART 1
            0xD0..=0xD7 => self.uart2.read_port(queue, id - 0xD0), // UART 2

            0x100..=0x102 => self.gpio[7].read_port(queue, id - 0x100), // Port H
            0x103..=0x105 => self.gpio[8].read_port(queue, id - 0x103), // Port J
            0x106..=0x108 => self.gpio[9].read_port(queue, id - 0x106), // Port K
            0x109..=0x10B => self.gpio[10].read_port(queue, id - 0x109), // Port L

            0x120..=0x12F => self.timer5.read_port(queue, id - 0x120), // Timer 5
            0x130..=0x137 => self.uart3.read_port(queue, id - 0x130),  // UART 3

            _ => panic!("Invalid address: {}", id),
        }
    }

    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: u8) {
        match id {
            0x00..=0x1F => panic!("Invalid address {:#02X}", id),

            0x20..=0x22 => self.gpio[0].write_port(queue, id - 0x20, data), // Port A
            0x23..=0x25 => self.gpio[1].write_port(queue, id - 0x23, data), // Port B
            0x26..=0x28 => self.gpio[2].write_port(queue, id - 0x26, data), // Port C
            0x29..=0x2B => {
                // Port D
                self.gpio[3].write_port(queue, id - 0x29, data);
                // Uart DDR
                if id == 0x2A {
                    self.uart1.ddr_xck = ((data >> 5) & 1) != 0;
                }
            }
            0x2C..=0x2E => {
                // Port E
                self.gpio[4].write_port(queue, id - 0x2C, data);
                // Uart DDR
                if id == 0x2D {
                    self.uart0.ddr_xck = ((data >> 2) & 1) != 0;
                }
            }
            0x2F..=0x31 => self.gpio[5].write_port(queue, id - 0x2F, data), // Port F
            0x32..=0x34 => self.gpio[6].write_port(queue, id - 0x32, data), // Port G

            0x35 => todo!(),
            0x36 => self.timer1.write_port(queue, Timer16::TIFR_PORT, data),
            0x37 => todo!(),
            0x38 => self.timer3.write_port(queue, Timer16::TIFR_PORT, data),
            0x39 => self.timer4.write_port(queue, Timer16::TIFR_PORT, data),
            0x3A => self.timer5.write_port(queue, Timer16::TIFR_PORT, data),

            0x53 => {
                // SMCR
                self.sleep_enabled = (data & 1) != 0;
                unsafe {
                    self.sleep_mode = transmute((data >> 1) & 0x7);
                }
            }

            0x6E => todo!(),
            0x6F => self.timer1.write_port(queue, Timer16::TIMSK_PORT, data),
            0x70 => todo!(),
            0x71 => self.timer3.write_port(queue, Timer16::TIMSK_PORT, data),
            0x72 => self.timer4.write_port(queue, Timer16::TIMSK_PORT, data),
            0x73 => self.timer5.write_port(queue, Timer16::TIMSK_PORT, data),

            0x80..=0x8F => self.timer1.write_port(queue, id - 0x80, data), // Timer 1
            0x90..=0x9F => self.timer3.write_port(queue, id - 0x90, data), // Timer 3
            0xA0..=0xAF => self.timer4.write_port(queue, id - 0xA0, data), // Timer 4

            0xC0..=0xC7 => self.uart0.write_port(queue, id - 0xC0, data), // UART 0
            0xC8..=0xCF => self.uart1.write_port(queue, id - 0xC8, data), // UART 1
            0xD0..=0xD7 => self.uart2.write_port(queue, id - 0xD0, data), // UART 2

            0x100..=0x102 => {
                // Port H
                self.gpio[7].write_port(queue, id - 0x100, data);
                // Uart DDR
                if id == 0x101 {
                    self.uart2.ddr_xck = ((data >> 2) & 1) != 0;
                }
            }
            0x103..=0x105 => {
                // Port J
                self.gpio[8].write_port(queue, id - 0x103, data);
                // Uart DDR
                if id == 0x104 {
                    self.uart3.ddr_xck = ((data >> 2) & 1) != 0;
                }
            }
            0x106..=0x108 => self.gpio[9].write_port(queue, id - 0x106, data), // Port K
            0x109..=0x10B => self.gpio[10].write_port(queue, id - 0x109, data), // Port L

            0x120..=0x12F => self.timer5.write_port(queue, id - 0x120, data), // Timer 5
            0x130..=0x137 => self.uart2.write_port(queue, id - 0x130, data),  // UART 2

            _ => panic!("Invalid address: {}", id),
        }
    }
}

impl IoController {
    #[inline]
    pub fn read_port_internal(&mut self, queue: &mut EventQueue, id: PortId) -> u8 {
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
        fn update(
            addr: u16,
            result: &mut Option<u16>,
            have_others: &mut bool,
            flag: &mut bool,
            mask: bool,
        ) {
            if mask && *flag {
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

        fn update_readonly(
            addr: u16,
            result: &mut Option<u16>,
            have_others: &mut bool,
            flag: bool,
            mask: bool,
        ) {
            if mask && flag {
                *result = match result {
                    None => Some(addr),
                    Some(x) => {
                        *have_others = true;
                        Some(*x)
                    }
                }
            }
        }

        update(
            0x0020,
            &mut result,
            &mut have_others,
            &mut self.timer1.interrupt_flags.input_capture,
            self.timer1.interrupt_masks.input_capture,
        );
        update(
            0x0022,
            &mut result,
            &mut have_others,
            &mut self.timer1.interrupt_flags.oc[0],
            self.timer1.interrupt_masks.oc[0],
        );
        update(
            0x0024,
            &mut result,
            &mut have_others,
            &mut self.timer1.interrupt_flags.oc[1],
            self.timer1.interrupt_masks.oc[1],
        );
        update(
            0x0026,
            &mut result,
            &mut have_others,
            &mut self.timer1.interrupt_flags.oc[2],
            self.timer1.interrupt_masks.oc[2],
        );
        update(
            0x0028,
            &mut result,
            &mut have_others,
            &mut self.timer1.interrupt_flags.overflow,
            self.timer1.interrupt_masks.overflow,
        );

        update_readonly(
            0x0032,
            &mut result,
            &mut have_others,
            self.uart0.rx_interrupt(),
            self.uart0.rx_interrupt_enable,
        );
        update_readonly(
            0x0034,
            &mut result,
            &mut have_others,
            self.uart0.udr_interrupt(),
            self.uart0.udr_interrupt_enable,
        );
        update(
            0x0036,
            &mut result,
            &mut have_others,
            &mut self.uart0.tx_interrupt,
            self.uart0.tx_interrupt_enable,
        );

        update(
            0x003E,
            &mut result,
            &mut have_others,
            &mut self.timer3.interrupt_flags.input_capture,
            self.timer3.interrupt_masks.input_capture,
        );
        update(
            0x0040,
            &mut result,
            &mut have_others,
            &mut self.timer3.interrupt_flags.oc[0],
            self.timer3.interrupt_masks.oc[0],
        );
        update(
            0x0042,
            &mut result,
            &mut have_others,
            &mut self.timer3.interrupt_flags.oc[1],
            self.timer3.interrupt_masks.oc[1],
        );
        update(
            0x0044,
            &mut result,
            &mut have_others,
            &mut self.timer3.interrupt_flags.oc[2],
            self.timer3.interrupt_masks.oc[2],
        );
        update(
            0x0046,
            &mut result,
            &mut have_others,
            &mut self.timer3.interrupt_flags.overflow,
            self.timer3.interrupt_masks.overflow,
        );

        update_readonly(
            0x0048,
            &mut result,
            &mut have_others,
            self.uart1.rx_interrupt(),
            self.uart1.rx_interrupt_enable,
        );
        update_readonly(
            0x004A,
            &mut result,
            &mut have_others,
            self.uart1.udr_interrupt(),
            self.uart1.udr_interrupt_enable,
        );
        update(
            0x004C,
            &mut result,
            &mut have_others,
            &mut self.uart1.tx_interrupt,
            self.uart1.tx_interrupt_enable,
        );

        update(
            0x0052,
            &mut result,
            &mut have_others,
            &mut self.timer4.interrupt_flags.input_capture,
            self.timer4.interrupt_masks.input_capture,
        );
        update(
            0x0054,
            &mut result,
            &mut have_others,
            &mut self.timer4.interrupt_flags.oc[0],
            self.timer4.interrupt_masks.oc[0],
        );
        update(
            0x0056,
            &mut result,
            &mut have_others,
            &mut self.timer4.interrupt_flags.oc[1],
            self.timer4.interrupt_masks.oc[1],
        );
        update(
            0x0058,
            &mut result,
            &mut have_others,
            &mut self.timer4.interrupt_flags.oc[2],
            self.timer4.interrupt_masks.oc[2],
        );
        update(
            0x005A,
            &mut result,
            &mut have_others,
            &mut self.timer4.interrupt_flags.overflow,
            self.timer4.interrupt_masks.overflow,
        );

        update(
            0x005C,
            &mut result,
            &mut have_others,
            &mut self.timer5.interrupt_flags.input_capture,
            self.timer5.interrupt_masks.input_capture,
        );
        update(
            0x005E,
            &mut result,
            &mut have_others,
            &mut self.timer5.interrupt_flags.oc[0],
            self.timer5.interrupt_masks.oc[0],
        );
        update(
            0x0060,
            &mut result,
            &mut have_others,
            &mut self.timer5.interrupt_flags.oc[1],
            self.timer5.interrupt_masks.oc[1],
        );
        update(
            0x0062,
            &mut result,
            &mut have_others,
            &mut self.timer5.interrupt_flags.oc[2],
            self.timer5.interrupt_masks.oc[2],
        );
        update(
            0x0064,
            &mut result,
            &mut have_others,
            &mut self.timer5.interrupt_flags.overflow,
            self.timer5.interrupt_masks.overflow,
        );

        update_readonly(
            0x0066,
            &mut result,
            &mut have_others,
            self.uart2.rx_interrupt(),
            self.uart2.rx_interrupt_enable,
        );
        update_readonly(
            0x0068,
            &mut result,
            &mut have_others,
            self.uart2.udr_interrupt(),
            self.uart2.udr_interrupt_enable,
        );
        update(
            0x006A,
            &mut result,
            &mut have_others,
            &mut self.uart2.tx_interrupt,
            self.uart2.tx_interrupt_enable,
        );
        update_readonly(
            0x006C,
            &mut result,
            &mut have_others,
            self.uart3.rx_interrupt(),
            self.uart3.rx_interrupt_enable,
        );
        update_readonly(
            0x006E,
            &mut result,
            &mut have_others,
            self.uart3.udr_interrupt(),
            self.uart3.udr_interrupt_enable,
        );
        update(
            0x0070,
            &mut result,
            &mut have_others,
            &mut self.uart3.tx_interrupt,
            self.uart3.tx_interrupt_enable,
        );

        if !have_others {
            self.interrupt = false;
        }
        result
    }
}
