use std::{
    any::Any,
    collections::VecDeque,
    io::{stdin, stdout, Read, Write},
    thread::JoinHandle,
};

use getch::Getch;
use kanal::{Receiver, Sender};

use crate::{
    clock::Timestamp,
    events::{EventQueue, InternalEvent},
    module::{Module, PinId, WireableModule},
    module_id::ModuleAddress,
    pin_state::{InputPinState, WireState},
    vcd::{VcdEvent, VcdSender, VcdSignal},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityMode {
    Disabled,
    Even,
    Odd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameState {
    Idle,
    Start,
    Data(u8),
    Parity,
    End(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct UartConfig {
    pub parity: ParityMode,
    pub double_stop_bit: bool,
    pub char_size: u8,
    pub polarity: bool,
}

pub struct UartModule {
    module_id: ModuleAddress,

    config: UartConfig,

    tx_buf: u16,
    rx_buf: u16,

    tx_state: FrameState,
    rx_state: FrameState,
    tx_data: VecDeque<u16>,
    rx_data: VecDeque<u16>,

    xck_val: InputPinState,
    tx_val: WireState,
    rx_val: InputPinState,

    parity_error: bool,
    frame_error: bool,

    tx_receiver: Option<Receiver<u16>>,
    rx_sender: Option<Sender<u16>>,

    vcd_sender: Option<Sender<VcdEvent>>,
}

impl UartModule {
    pub const RX_PIN: u8 = 0;
    pub const TX_PIN: u8 = 1;
    pub const XCK_PIN: u8 = 2;

    pub fn new(module_id: ModuleAddress, config: UartConfig) -> UartModule {
        UartModule {
            module_id,
            config,

            tx_buf: 0,
            rx_buf: 0,
            tx_state: FrameState::Idle,
            rx_state: FrameState::Idle,
            tx_data: VecDeque::new(),
            rx_data: VecDeque::new(),

            xck_val: InputPinState::Low,
            tx_val: WireState::Z,
            rx_val: InputPinState::High,

            parity_error: false,
            frame_error: false,

            vcd_sender: None,

            tx_receiver: None,
            rx_sender: None,
        }
    }

    fn set_tx(&mut self, data: WireState, queue: &mut EventQueue) {
        self.tx_val = data;
        queue.set_wire(
            self.module_id.with_pin(Self::TX_PIN),
            data.combine(&WireState::WeakHigh),
        );
    }

    fn advance_state(&self, s: FrameState) -> FrameState {
        match s {
            FrameState::Idle => FrameState::Idle,
            FrameState::Start => FrameState::Data(0),
            FrameState::Data(i) => {
                if i + 1 < self.config.char_size {
                    FrameState::Data(i + 1)
                } else {
                    if self.config.parity != ParityMode::Disabled {
                        FrameState::Parity
                    } else {
                        FrameState::End(0)
                    }
                }
            }
            FrameState::Parity => FrameState::End(0),
            FrameState::End(0) => {
                if self.config.double_stop_bit {
                    FrameState::End(1)
                } else {
                    FrameState::Idle
                }
            }
            FrameState::End(_) => FrameState::Idle,
        }
    }

    fn trigger_receiver(&mut self, _queue: &mut EventQueue) {
        self.rx_state = self.advance_state(self.rx_state);

        if self.rx_state == FrameState::Idle {
            if self.rx_val == InputPinState::Low {
                self.rx_state = FrameState::Start;
            }
            return;
        }

        match self.rx_state {
            FrameState::Idle => unreachable!(),
            FrameState::Start => unreachable!(),
            FrameState::Data(i) => {
                let b = (self.rx_val == InputPinState::High) as u16;
                let mask = 1 << i;
                self.rx_buf = (self.rx_buf & !mask) | (b << i);
            }
            FrameState::Parity => {
                let mask = (1u16 << self.config.char_size) - 1;
                let data = self.rx_buf & mask;
                let parity_bit = match self.config.parity {
                    ParityMode::Even => data.count_ones() % 2 == 1,
                    ParityMode::Odd => data.count_ones() % 2 == 0,
                    _ => unreachable!(),
                };
                let actual_bit = self.rx_val == InputPinState::High;
                if parity_bit != actual_bit {
                    self.parity_error = true;
                }
            }
            FrameState::End(0) => {
                if self.rx_val == InputPinState::Low {
                    self.frame_error = true;
                }

                if let Some(sender) = &self.rx_sender {
                    if let Err(_) = sender.send(self.rx_buf) {
                        self.rx_sender = None;
                    }
                } else {
                    self.rx_data.push_back(self.rx_buf);
                }
            }
            FrameState::End(_) => {}
        }
    }

    fn trigger_transmitter(&mut self, queue: &mut EventQueue) {
        if let Some(tx) = self.tx_receiver.as_mut() {
            while let Ok(Some(x)) = tx.try_recv() {
                self.tx_data.push_back(x);
            }
        }

        self.tx_state = self.advance_state(self.tx_state);

        if self.tx_state == FrameState::Idle {
            if let Some(data) = self.tx_data.pop_front() {
                self.tx_state = FrameState::Start;
                self.tx_buf = data;
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
                let mask = (1u16 << self.config.char_size) - 1;
                let data = self.tx_buf & mask;
                let parity_bit = match self.config.parity {
                    ParityMode::Even => data.count_ones() % 2 == 1,
                    ParityMode::Odd => data.count_ones() % 2 == 0,
                    _ => unreachable!(),
                };
                self.set_tx(WireState::from_bool(parity_bit), queue);
            }
            FrameState::End(_) => {
                self.set_tx(WireState::High, queue);
            }
        }
    }

    fn trigger_clock(&mut self, queue: &mut EventQueue) {
        if self.config.polarity {
            // Polarity = 1 => sample on rising, change on falling
            match self.xck_val {
                InputPinState::High => self.trigger_receiver(queue),
                InputPinState::Low => self.trigger_transmitter(queue),
            }
        } else {
            // Polarity = 0 => change on rising, sample on falling
            match self.xck_val {
                InputPinState::High => self.trigger_transmitter(queue),
                InputPinState::Low => self.trigger_receiver(queue),
            }
        }
    }

    pub fn write_u16(&mut self, data: u16) {
        self.tx_data.push_back(data);
    }
    pub fn write_char(&mut self, data: char) {
        assert_eq!(self.config.char_size, 8);
        self.tx_data.push_back(data as u16);
    }
    pub fn write_str(&mut self, data: &str) {
        assert_eq!(self.config.char_size, 8);
        for c in data.chars() {
            self.tx_data.push_back(c as u16);
        }
    }

    pub fn read_u16(&mut self) -> Option<u16> {
        self.rx_data.pop_front()
    }
    pub fn read_char(&mut self) -> Option<char> {
        assert_eq!(self.config.char_size, 8);
        self.rx_data.pop_front().map(|x| x as u8 as char)
    }

    pub fn connect(&mut self) -> (JoinHandle<()>, JoinHandle<()>) {
        let (tx_sender, tx_receiver) = kanal::bounded(1024);
        let (rx_sender, rx_receiver) = kanal::bounded(1024);
        self.tx_receiver = Some(tx_receiver);
        self.rx_sender = Some(rx_sender);

        let t1 = std::thread::spawn(move || {
            let mut f = stdout();
            for x in rx_receiver {
                f.write_all(&[x as u8]).unwrap();
                f.flush().unwrap();
            }
        });
        let t2 = std::thread::spawn(move || {
            let g = Getch::new();
            while let Ok(x) = g.getch() {
                tx_sender.send(x as u16).unwrap();
            }
        });
        (t1, t2)
    }
}

impl std::fmt::Debug for UartModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UartModule")
            .field("module_id", &self.module_id)
            .field("config", &self.config)
            .field("tx_buf", &self.tx_buf)
            .field("rx_buf", &self.rx_buf)
            .field("tx_state", &self.tx_state)
            .field("rx_state", &self.rx_state)
            .field("tx_data", &self.tx_data)
            .field("rx_data", &self.rx_data)
            .field("xck_val", &self.xck_val)
            .field("tx_val", &self.tx_val)
            .field("rx_val", &self.rx_val)
            .field("parity_error", &self.parity_error)
            .field("frame_error", &self.frame_error)
            .field("vcd_sender", &self.vcd_sender)
            .finish()
    }
}

impl VcdSender for UartModule {
    fn register_vcd(&mut self, sender: Sender<VcdEvent>, _start_id: i32) -> (Vec<VcdSignal>, i32) {
        self.vcd_sender = Some(sender);
        (vec![], 0)
    }

    fn vcd_sender(&self) -> Option<&Sender<VcdEvent>> {
        self.vcd_sender.as_ref()
    }
}

impl Module for UartModule {
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_event(&mut self, _event: InternalEvent, _queue: &mut EventQueue, _t: Timestamp) {
        panic!("UART console can't handle events");
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

impl WireableModule for UartModule {
    fn get_pin(&self, _queue: &EventQueue, id: PinId) -> WireState {
        match id as u8 {
            Self::RX_PIN => self.rx_val.to_wire_state().combine(&WireState::WeakHigh),
            Self::TX_PIN => self.tx_val.combine(&WireState::WeakHigh),
            Self::XCK_PIN => WireState::Z,
            _ => panic!("Invalid pin {}", id),
        }
    }

    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState) {
        match id as u8 {
            Self::RX_PIN => {
                self.rx_val = InputPinState::read_wire_state(data.combine(&WireState::WeakHigh));
            }
            Self::TX_PIN => {}
            Self::XCK_PIN => {
                self.xck_val = InputPinState::read_wire_state(data);
                self.trigger_clock(queue);
            }
            _ => panic!("Invalid pin {}", id),
        }
    }
}
