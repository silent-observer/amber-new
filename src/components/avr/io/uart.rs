use std::{any::Any, mem::transmute};

use kanal::Sender;

use crate::{
    clock::{TickTimestamp, Timestamp},
    events::{EventQueue, InternalEvent},
    module::{DataModule, Module, PinId, PortId, WireableModule},
    module_id::{EventPortAddress, ModuleAddress},
    pin_state::{InputPinState, WireState},
    vcd::{VcdEvent, VcdSender, VcdSignal},
};

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParityMode {
    Disabled = 0,
    Reserved = 1,
    Even = 2,
    Odd = 3,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterSizeMode {
    Char5 = 0,
    Char6 = 1,
    Char7 = 2,
    Char8 = 3,
    Reserved1 = 4,
    Reserved2 = 5,
    Reserved3 = 6,
    Char9 = 7,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UartMode {
    Async = 0,
    Sync = 1,
    Reserved = 2,
    MasterSpi = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameState {
    Idle,
    Start,
    Data(u8),
    Parity,
    End(u8),
}

#[derive(Debug, Clone)]
pub struct Uart {
    last_write_t: TickTimestamp,
    last_write_counter: u16,

    module_id: ModuleAddress,
    interrupt_reciever: EventPortAddress,

    mode: UartMode,
    u2x: bool,
    parity: ParityMode,
    double_stop_bit: bool,
    char_size: CharacterSizeMode,
    ucpol: bool,
    baud_rate: u16,

    tx_prescaler: u8,

    txen: bool,
    rxen: bool,

    tx_buf: u16,
    rx_buf: u16,
    tx_state: FrameState,
    rx_state: FrameState,
    tx_data: u16,
    tx_data_present: bool,
    rx_data: [u16; 2],
    rx_data_len: u8,

    pub ddr_xck: bool,
    xck_val: InputPinState,
    tx_val: WireState,
    rx_val: InputPinState,

    parity_error: bool,
    frame_error: bool,
    data_overrun_error: bool,
    pub tx_interrupt: bool,

    pub rx_interrupt_enable: bool,
    pub tx_interrupt_enable: bool,
    pub udr_interrupt_enable: bool,

    vcd_sender: Option<Sender<VcdEvent>>,
}

impl Uart {
    pub const RX_PIN: u8 = 0;
    pub const TX_PIN: u8 = 1;
    pub const XCK_PIN: u8 = 2;

    pub fn new(module_id: ModuleAddress, interrupt_reciever: EventPortAddress) -> Uart {
        Uart {
            last_write_t: 0,
            last_write_counter: 0,

            module_id,
            interrupt_reciever,

            mode: UartMode::Async,
            u2x: false,
            parity: ParityMode::Disabled,
            double_stop_bit: false,
            char_size: CharacterSizeMode::Char5,
            ucpol: false,
            baud_rate: 0,

            tx_prescaler: 0,

            txen: false,
            rxen: false,
            tx_buf: 0,
            rx_buf: 0,
            tx_state: FrameState::Idle,
            rx_state: FrameState::Idle,
            tx_data: 0,
            tx_data_present: false,
            rx_data: [0, 0],
            rx_data_len: 0,

            ddr_xck: false,
            xck_val: InputPinState::Low,
            tx_val: WireState::Z,
            rx_val: InputPinState::High,

            parity_error: false,
            frame_error: false,
            data_overrun_error: false,
            tx_interrupt: false,

            rx_interrupt_enable: false,
            tx_interrupt_enable: false,
            udr_interrupt_enable: false,

            vcd_sender: None,
        }
    }

    fn set_tx(&mut self, data: WireState, queue: &mut EventQueue) {
        self.tx_val = data;
        queue.set_wire(
            self.module_id.with_pin(Self::TX_PIN),
            data.combine(&WireState::WeakHigh),
        );
    }

    fn char_size_bits(&self) -> u8 {
        match self.char_size {
            CharacterSizeMode::Char5 => 5,
            CharacterSizeMode::Char6 => 6,
            CharacterSizeMode::Char7 => 7,
            CharacterSizeMode::Char8 => 8,
            CharacterSizeMode::Char9 => 9,
            _ => 9,
        }
    }

    fn advance_state(&self, s: FrameState) -> FrameState {
        match s {
            FrameState::Idle => FrameState::Idle,
            FrameState::Start => FrameState::Data(0),
            FrameState::Data(i) => {
                if i + 1 < self.char_size_bits() {
                    FrameState::Data(i + 1)
                } else {
                    if self.parity != ParityMode::Disabled {
                        FrameState::Parity
                    } else {
                        FrameState::End(0)
                    }
                }
            }
            FrameState::Parity => FrameState::End(0),
            FrameState::End(0) => {
                if self.double_stop_bit {
                    FrameState::End(1)
                } else {
                    FrameState::Idle
                }
            }
            FrameState::End(_) => FrameState::Idle,
        }
    }

    fn trigger_receiver(&mut self, bit: InputPinState, queue: &mut EventQueue) {
        self.rx_state = self.advance_state(self.rx_state);

        if self.rx_state == FrameState::Idle {
            if bit == InputPinState::Low {
                self.rx_state = FrameState::Start;
            }
            return;
        }

        match self.rx_state {
            FrameState::Idle => unreachable!(),
            FrameState::Start => unreachable!(),
            FrameState::Data(i) => {
                let b = (bit == InputPinState::High) as u16;
                let mask = 1 << i;
                self.rx_buf = (self.rx_buf & !mask) | (b << i);
            }
            FrameState::Parity => {
                let mask = (1u16 << self.char_size_bits()) - 1;
                let data = self.rx_buf & mask;
                let parity_bit = match self.parity {
                    ParityMode::Even => data.count_ones() % 2 == 1,
                    ParityMode::Odd => data.count_ones() % 2 == 0,
                    _ => unreachable!(),
                };
                let actual_bit = bit == InputPinState::High;
                if parity_bit != actual_bit {
                    self.parity_error = true;
                }
            }
            FrameState::End(0) => {
                if bit == InputPinState::Low {
                    self.frame_error = true;
                }

                if self.rx_data_len < 2 {
                    self.rx_data[self.rx_data_len as usize] = self.rx_buf;
                    self.rx_data_len += 1;
                } else {
                    self.data_overrun_error = true;
                }

                if self.rx_interrupt_enable {
                    queue.fire_event_now(InternalEvent {
                        receiver_id: self.interrupt_reciever,
                    })
                }
            }
            FrameState::End(_) => {}
        }
    }

    fn trigger_transmitter(&mut self, queue: &mut EventQueue) {
        self.tx_state = self.advance_state(self.tx_state);

        if self.tx_state == FrameState::Idle && self.tx_data_present {
            self.tx_state = FrameState::Start;
            self.tx_buf = self.tx_data;
            self.tx_data_present = false;

            if self.udr_interrupt_enable {
                queue.fire_event_now(InternalEvent {
                    receiver_id: self.interrupt_reciever,
                })
            }
        }

        match self.tx_state {
            FrameState::Idle => {
                self.set_tx(WireState::Z, queue);
            }
            FrameState::Start => {
                self.set_tx(WireState::Low, queue);
            }
            FrameState::Data(i) => {
                let bit = ((self.tx_buf >> i) & 1) != 0;
                self.set_tx(WireState::from_bool(bit), queue);
            }
            FrameState::Parity => {
                let mask = (1u16 << self.char_size_bits()) - 1;
                let data = self.tx_buf & mask;
                let parity_bit = match self.parity {
                    ParityMode::Even => data.count_ones() % 2 == 1,
                    ParityMode::Odd => data.count_ones() % 2 == 0,
                    _ => unreachable!(),
                };
                self.set_tx(WireState::from_bool(parity_bit), queue);
            }
            FrameState::End(_) => {
                self.set_tx(WireState::High, queue);
                self.tx_interrupt = true;

                if self.tx_interrupt_enable {
                    queue.fire_event_now(InternalEvent {
                        receiver_id: self.interrupt_reciever,
                    })
                }
            }
        }
    }

    fn trigger_receiver_clock(&mut self, queue: &mut EventQueue) {
        if self.mode != UartMode::Sync {
            panic!("Only Sync UART receiver is implemented right now!");
        }

        let sample = self.rx_val;
        self.trigger_receiver(sample, queue);
    }
    fn trigger_transmitter_clock(&mut self, queue: &mut EventQueue) {
        let prescaler = match (self.mode, self.u2x) {
            (UartMode::Async, false) => 8,
            (UartMode::Async, true) => 4,
            (UartMode::Sync, _) => 1,
            _ => 2,
        };

        self.tx_prescaler += 1;
        if self.tx_prescaler == prescaler {
            self.tx_prescaler = 0;
            self.trigger_transmitter(queue);
        }
    }

    fn trigger_clock(&mut self, queue: &mut EventQueue) {
        queue.set_wire(
            self.module_id.with_pin(Self::XCK_PIN),
            self.xck_val.to_wire_state(),
        );
        if self.ucpol {
            // UCPOL = 1 => sample on rising, change on falling
            match self.xck_val {
                InputPinState::High => self.trigger_receiver_clock(queue),
                InputPinState::Low => self.trigger_transmitter_clock(queue),
            }
        } else {
            // UCPOL = 0 => change on rising, sample on falling
            match self.xck_val {
                InputPinState::High => self.trigger_transmitter_clock(queue),
                InputPinState::Low => self.trigger_receiver_clock(queue),
            }
        }
    }

    fn simulate(&mut self, timestamp: TickTimestamp, queue: &mut EventQueue) {
        if !self.txen && !self.rxen {
            self.last_write_t = timestamp;
            return;
        }

        if self.mode == UartMode::Sync && !self.ddr_xck {
            self.last_write_t = timestamp;
            return;
        }

        let ticks = timestamp - self.last_write_t;
        let new_counter = self.last_write_counter as i64 - ticks;

        if new_counter == -1 {
            self.last_write_counter = self.baud_rate;
            self.xck_val = self.xck_val.flip();
            self.trigger_clock(queue);
        } else if new_counter < 0 {
            panic!("Skipped event!");
        } else {
            self.last_write_counter = new_counter as u16;
        }
        self.last_write_t = timestamp;
    }

    fn schedule_event(&mut self, queue: &mut EventQueue, timestamp: TickTimestamp) {
        if !self.txen && !self.rxen {
            return;
        }

        if self.mode == UartMode::Async && self.ddr_xck {
            return;
        }

        let clockgen_ticks = self.last_write_counter as i64 + 1;
        let next_event = timestamp + clockgen_ticks;
        queue.fire_event_at_ticks(
            InternalEvent {
                receiver_id: self.module_id.with_event_port(0),
            },
            next_event,
        )
    }

    pub fn rx_interrupt(&self) -> bool {
        self.rx_data_len > 0
    }
    pub fn udr_interrupt(&self) -> bool {
        !self.tx_data_present
    }
}

impl VcdSender for Uart {
    fn register_vcd(&mut self, sender: Sender<VcdEvent>, _start_id: i32) -> (Vec<VcdSignal>, i32) {
        self.vcd_sender = Some(sender);
        (vec![], 0)
    }

    fn vcd_sender(&self) -> Option<&Sender<VcdEvent>> {
        self.vcd_sender.as_ref()
    }
}

impl Module for Uart {
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_event(&mut self, event: InternalEvent, queue: &mut EventQueue, t: Timestamp) {
        assert_eq!(event.receiver_id.event_port_id, 0);
        self.simulate(queue.clock.time_to_ticks(t), queue);
        self.schedule_event(queue, queue.clock.time_to_ticks(t));
    }

    fn find(&self, address: ModuleAddress) -> Option<&dyn Module> {
        if address.is_empty() {
            Some(self)
        } else {
            None
        }
    }

    fn find_mut(&mut self, address: ModuleAddress) -> Option<&mut dyn Module> {
        if address.is_empty() {
            Some(self)
        } else {
            None
        }
    }

    fn to_wireable_mut(&mut self) -> Option<&mut dyn WireableModule> {
        Some(self)
    }
    fn to_wireable(&self) -> Option<&dyn WireableModule> {
        Some(self)
    }
}

impl DataModule for Uart {
    type PortType = u8;

    fn read_port(&mut self, _queue: &mut EventQueue, id: PortId) -> Self::PortType {
        match id {
            0 => {
                // UCSRnA
                let rxc = self.rx_interrupt() as u8;
                let txc = self.tx_interrupt as u8;
                let udre = self.udr_interrupt() as u8;
                let fe = self.frame_error as u8;
                let dor = self.data_overrun_error as u8;
                let upe = self.parity_error as u8;
                let u2x = self.u2x as u8;
                let mpcm = 0u8;
                rxc << 7 | txc << 6 | udre << 5 | fe << 4 | dor << 3 | upe << 2 | u2x << 1 | mpcm
            }
            1 => {
                // UCSRnB
                let rxcie = self.rx_interrupt_enable as u8;
                let txcie = self.tx_interrupt_enable as u8;
                let udrie = self.udr_interrupt_enable as u8;
                let rxen = self.rxen as u8;
                let txen = self.txen as u8;
                let ucsz2 = (self.char_size as u8) >> 2;
                let rxb8 = ((self.rx_data[0] >> 8) & 1) as u8;
                let txb8 = ((self.tx_data >> 8) & 1) as u8;
                rxcie << 7
                    | txcie << 6
                    | udrie << 5
                    | rxen << 4
                    | txen << 3
                    | ucsz2 << 2
                    | rxb8 << 1
                    | txb8
            }
            2 => {
                // UCSRnC
                let umsel = self.mode as u8;
                let upm = self.parity as u8;
                let usbs = self.double_stop_bit as u8;
                let ucsz = (self.char_size as u8) & 0x3;
                let ucpol = self.ucpol as u8;
                umsel << 6 | upm << 4 | usbs << 3 | ucsz << 1 | ucpol
            }
            3 => 0,                                    // Reserved
            4 => (self.baud_rate & 0xFF) as u8,        // UBRRnL
            5 => ((self.baud_rate >> 8) & 0xFF) as u8, // UBRRnH
            6 => match self.rx_data_len {
                // UDRn
                0 => 0,
                1 => {
                    let data = self.rx_data[0];
                    self.rx_data_len = 0;
                    data as u8
                }
                2 => {
                    let data = self.rx_data[0];
                    self.rx_data[0] = self.rx_data[1];
                    self.rx_data_len = 1;
                    data as u8
                }
                _ => unreachable!(),
            },
            _ => panic!("Invalid port {}", id),
        }
    }

    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: Self::PortType) {
        self.simulate(queue.clock.current_tick(), queue);
        match id {
            0 => {
                // UCSRnA
                self.tx_interrupt &= ((data >> 6) & 1) == 0; // Clear if 1
                self.u2x = (data >> 1) & 1 != 0;
            }
            1 => {
                // UCSRnB
                self.rx_interrupt_enable = ((data >> 7) & 1) != 0;
                self.tx_interrupt_enable = ((data >> 6) & 1) != 0;
                self.udr_interrupt_enable = ((data >> 5) & 1) != 0;
                self.rxen = ((data >> 4) & 1) != 0;
                self.txen = ((data >> 3) & 1) != 0;
                unsafe {
                    let ucsz2 = (data >> 2) & 1;
                    self.char_size = transmute((self.char_size as u8) & 0x3 | ucsz2 << 2);
                }
                self.tx_data = self.tx_data & 0xFF | ((data & 1) as u16) << 8;

                queue.set_multiplexer_flag(self.module_id.with_pin(Self::RX_PIN), self.rxen);
                queue.set_multiplexer_flag(self.module_id.with_pin(Self::TX_PIN), self.txen);
                if self.rxen {
                    queue.set_wire(self.module_id.with_pin(Self::RX_PIN), WireState::WeakHigh);
                }
                if self.txen {
                    queue.set_wire(self.module_id.with_pin(Self::TX_PIN), WireState::WeakHigh);
                }
            }
            2 => {
                // UCSRnC
                let umsel = (data >> 6) & 0x3;
                let upm = (data >> 4) & 0x3;
                let usbs = (data >> 3) & 1;
                let ucsz = (data >> 1) & 0x3;
                let ucpol = data & 1;

                unsafe {
                    self.mode = transmute(umsel);
                    self.parity = transmute(upm);
                    self.char_size = transmute((self.char_size as u8) & 0x4 | ucsz)
                }
                self.double_stop_bit = usbs != 0;
                self.ucpol = ucpol != 0;

                queue.set_multiplexer_flag(
                    self.module_id.with_pin(Self::XCK_PIN),
                    self.mode == UartMode::Sync,
                );
                println!("{:02X} -> {:?}", data, self.mode);
            }
            3 => {} // Reserved
            4 => {
                // UBRRnL
                assert!(self.last_write_t == queue.clock.current_tick());
                self.baud_rate = self.baud_rate & 0xFF00 | data as u16;
                self.last_write_counter = self.baud_rate;
            }
            5 => {
                // UBRRnH
                self.baud_rate = self.baud_rate & 0xFF | (data as u16) << 8;
            }
            6 => {
                // UDRn
                if !self.tx_data_present {
                    self.tx_data = self.tx_data & 0x100 | data as u16;
                    self.tx_data_present = true;
                }
            }
            _ => panic!("Invalid port {}", id),
        }
        self.schedule_event(queue, queue.clock.current_tick());
    }
}

impl WireableModule for Uart {
    fn get_pin(&self, _queue: &EventQueue, id: PinId) -> WireState {
        match id as u8 {
            Self::RX_PIN => self.rx_val.to_wire_state().combine(&WireState::WeakHigh),
            Self::TX_PIN => self.tx_val.combine(&WireState::WeakHigh),
            Self::XCK_PIN => self.xck_val.to_wire_state(),
            _ => panic!("Invalid pin {}", id),
        }
    }

    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState) {
        match id as u8 {
            Self::RX_PIN => {
                self.rx_val = InputPinState::read_wire_state(data.combine(&WireState::WeakHigh));
                // println!("RX -> {:?}", self.rx_val);
            }
            Self::TX_PIN => {}
            Self::XCK_PIN => {
                if self.mode == UartMode::Sync && !self.ddr_xck {
                    self.xck_val = InputPinState::read_wire_state(data);
                    self.trigger_clock(queue);
                    self.last_write_t = queue.clock.current_tick();
                    self.schedule_event(queue, self.last_write_t);
                }
            }
            _ => panic!("Invalid pin {}", id),
        }
    }
}
