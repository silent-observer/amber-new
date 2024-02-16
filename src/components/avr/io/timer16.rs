use std::mem::transmute;

use crate::{
    clock::TickTimestamp,
    events::{EventQueue, InternalEvent},
    module::{DataModule, Module, PinId, PortId, WireableModule},
    module_id::{EventPortAddress, ModuleAddress},
    pin_state::WireState,
};

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompareOutputMode {
    Disabled = 0,
    Toggle = 1,
    Clear = 2,
    Set = 3,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClockMode {
    Disabled = 0,
    Clk1 = 1,
    Clk8 = 2,
    Clk64 = 3,
    Clk256 = 4,
    Clk1024 = 5,
    ExternalFalling = 6,
    ExternalRising = 7,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum WaveformGenerationMode {
    Normal = 0,
    Pwm8Bit = 1,
    Pwm9Bit = 2,
    Pwm10Bit = 3,

    Ctc = 4,
    FastPwm8Bit = 5,
    FastPwm9Bit = 6,
    FastPwm10Bit = 7,

    PwmPhaseFreqIcr = 8,
    PwmPhaseFreqOcrA = 9,
    PwmPhaseIcr = 10,
    PwmPhaseOcrA = 11,

    CtcIcr = 12,
    Reserved = 13,
    FastPwmIcr = 14,
    FastPwmOcrA = 15,
}

#[derive(Debug, Clone, Copy)]
pub struct Timer16Interrupts {
    pub overflow: bool,
    pub oc: [bool; 3],
    pub input_capture: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Timer16 {
    last_write_t: TickTimestamp,
    last_write_counter: u16,

    module_id: ModuleAddress,
    interrupt_reciever: EventPortAddress,

    upcounting: bool,
    clock_mode: ClockMode,
    waveform_mode: WaveformGenerationMode,

    ocr: [u16; 3],
    compare_output_mode: [CompareOutputMode; 3],
    pins: [bool; 3],

    interrupt_masks: Timer16Interrupts,
    pub interrupt_flags: Timer16Interrupts,

    icnc: bool, // TODO
    ices: bool, // TODO
}

impl Timer16 {
    pub const TIMSK_PORT: PortId = 16;
    pub const TIFR_PORT: PortId = 17;

    pub fn new(module_id: ModuleAddress, interrupt_reciever: EventPortAddress) -> Timer16 {
        Timer16 {
            last_write_t: 0,
            last_write_counter: 0,

            module_id,
            interrupt_reciever,

            clock_mode: ClockMode::Disabled,
            upcounting: true,
            waveform_mode: WaveformGenerationMode::Normal,

            ocr: [0; 3],
            compare_output_mode: [CompareOutputMode::Disabled; 3],
            pins: [false; 3],

            interrupt_masks: Timer16Interrupts {
                overflow: false,
                oc: [false; 3],
                input_capture: false,
            },
            interrupt_flags: Timer16Interrupts {
                overflow: false,
                oc: [false; 3],
                input_capture: false,
            },

            icnc: false,
            ices: false,
        }
    }

    fn timer_top_value(&self) -> u16 {
        match self.waveform_mode {
            WaveformGenerationMode::Normal => 0xFFFF,
            WaveformGenerationMode::Pwm8Bit => 0x00FF,
            WaveformGenerationMode::Pwm9Bit => 0x01FF,
            WaveformGenerationMode::Pwm10Bit => 0x03FF,
            WaveformGenerationMode::Ctc => self.ocr[0],
            WaveformGenerationMode::FastPwm8Bit => 0x00FF,
            WaveformGenerationMode::FastPwm9Bit => 0x01FF,
            WaveformGenerationMode::FastPwm10Bit => 0x03FF,
            WaveformGenerationMode::PwmPhaseFreqIcr => todo!(),
            WaveformGenerationMode::PwmPhaseFreqOcrA => self.ocr[0],
            WaveformGenerationMode::PwmPhaseIcr => todo!(),
            WaveformGenerationMode::PwmPhaseOcrA => self.ocr[0],
            WaveformGenerationMode::CtcIcr => todo!(),
            WaveformGenerationMode::Reserved => todo!(),
            WaveformGenerationMode::FastPwmIcr => todo!(),
            WaveformGenerationMode::FastPwmOcrA => self.ocr[0],
        }
    }

    fn overflow_value(&self) -> u16 {
        match self.waveform_mode {
            WaveformGenerationMode::Normal => 0xFFFF,
            WaveformGenerationMode::Pwm8Bit
            | WaveformGenerationMode::Pwm9Bit
            | WaveformGenerationMode::Pwm10Bit => 0,
            WaveformGenerationMode::Ctc => 0xFFFF,
            WaveformGenerationMode::FastPwm8Bit => 0x00FF,
            WaveformGenerationMode::FastPwm9Bit => 0x01FF,
            WaveformGenerationMode::FastPwm10Bit => 0x03FF,
            WaveformGenerationMode::PwmPhaseFreqIcr
            | WaveformGenerationMode::PwmPhaseFreqOcrA
            | WaveformGenerationMode::PwmPhaseIcr
            | WaveformGenerationMode::PwmPhaseOcrA => 0,
            WaveformGenerationMode::CtcIcr => 0xFFFF,
            WaveformGenerationMode::Reserved => todo!(),
            WaveformGenerationMode::FastPwmIcr => todo!(),
            WaveformGenerationMode::FastPwmOcrA => self.ocr[0],
        }
    }

    fn is_oc_active(&self, i: usize) -> bool {
        self.compare_output_mode[i] != CompareOutputMode::Disabled || self.interrupt_masks.oc[i]
    }

    fn timer_ticks_until_next_event(&self) -> u16 {
        if self.upcounting {
            let top = self.timer_top_value();
            let ticks_to_top = top - self.last_write_counter + 1;
            let mut min_ticks = ticks_to_top;
            for i in 0..3 {
                if self.is_oc_active(i) && (self.last_write_counter < self.ocr[i]) {
                    let ticks_to_ocr = self.ocr[i] - self.last_write_counter;
                    if ticks_to_ocr < min_ticks {
                        min_ticks = ticks_to_ocr;
                    }
                }
            }
            if self.interrupt_masks.overflow {
                let overflow_value = self.overflow_value();
                if self.last_write_counter < overflow_value {
                    let ticks_to_overflow = overflow_value - self.last_write_counter;
                    if ticks_to_overflow < min_ticks {
                        min_ticks = ticks_to_overflow;
                    }
                }
            }
            min_ticks
        } else {
            let ticks_to_bottom = self.last_write_counter + 1;
            let mut min_ticks = ticks_to_bottom;
            for i in 0..3 {
                if self.is_oc_active(i) & (self.last_write_counter > self.ocr[i]) {
                    let ticks_to_ocr = self.last_write_counter - self.ocr[i];
                    if ticks_to_ocr < min_ticks {
                        min_ticks = ticks_to_ocr;
                    }
                }
            }
            if self.interrupt_masks.overflow {
                let overflow_value = self.overflow_value();
                if self.last_write_counter > overflow_value {
                    let ticks_to_overflow = self.last_write_counter - overflow_value;
                    if ticks_to_overflow < min_ticks {
                        min_ticks = ticks_to_overflow;
                    }
                }
            }
            min_ticks
        }
    }

    fn prescaler_shift(&self) -> i64 {
        match self.clock_mode {
            ClockMode::Disabled => panic!("Cannot take shift of disabled timer"),
            ClockMode::Clk1 => 0,
            ClockMode::Clk8 => 3,
            ClockMode::Clk64 => 6,
            ClockMode::Clk256 => 8,
            ClockMode::Clk1024 => 10,
            ClockMode::ExternalFalling => todo!(),
            ClockMode::ExternalRising => todo!(),
        }
    }

    fn ticks_up_to(&self, timestamp: TickTimestamp) -> i64 {
        let shift = self.prescaler_shift();
        (timestamp >> shift) - (self.last_write_t >> shift)
    }

    fn add_ticks(&self, timestamp: TickTimestamp, timer_ticks: i64) -> TickTimestamp {
        if self.clock_mode == ClockMode::Disabled {
            return timestamp;
        }

        let shift = self.prescaler_shift();
        ((timestamp >> shift) + timer_ticks) << shift
    }

    fn trigger_oc(&mut self, i: usize, queue: &mut EventQueue) {
        if self.interrupt_masks.oc[i] {
            self.interrupt_flags.oc[i] = true;
            queue.fire_event_now(InternalEvent {
                receiver_id: self.interrupt_reciever,
            })
        }
        match self.compare_output_mode[i] {
            CompareOutputMode::Disabled => {}
            CompareOutputMode::Toggle => {
                queue.set_wire(
                    self.module_id.with_pin(i as u8),
                    WireState::from_bool(!self.pins[i]),
                );
                self.pins[i] = !self.pins[i];
            }
            CompareOutputMode::Clear => {
                if self.pins[i] {
                    queue.set_wire(self.module_id.with_pin(i as u8), WireState::Low);
                }
                self.pins[i] = false;
            }
            CompareOutputMode::Set => {
                if !self.pins[i] {
                    queue.set_wire(self.module_id.with_pin(i as u8), WireState::High);
                }
                self.pins[i] = true;
            }
        }
    }

    fn simulate(&mut self, timestamp: TickTimestamp, queue: &mut EventQueue) {
        if self.clock_mode == ClockMode::Disabled {
            self.last_write_t = timestamp;
            return;
        }

        let ticks = self.ticks_up_to(timestamp);
        if self.upcounting {
            let top = self.timer_top_value();
            let new_counter = self.last_write_counter as i64 + ticks;
            if new_counter == top as i64 + 1 {
                match self.waveform_mode {
                    WaveformGenerationMode::Normal
                    | WaveformGenerationMode::Ctc
                    | WaveformGenerationMode::CtcIcr
                    | WaveformGenerationMode::FastPwm10Bit
                    | WaveformGenerationMode::FastPwm9Bit
                    | WaveformGenerationMode::FastPwm8Bit
                    | WaveformGenerationMode::FastPwmIcr
                    | WaveformGenerationMode::FastPwmOcrA => {
                        self.last_write_counter = 0;
                    }
                    WaveformGenerationMode::Pwm8Bit
                    | WaveformGenerationMode::Pwm9Bit
                    | WaveformGenerationMode::Pwm10Bit
                    | WaveformGenerationMode::PwmPhaseFreqIcr
                    | WaveformGenerationMode::PwmPhaseFreqOcrA
                    | WaveformGenerationMode::PwmPhaseIcr
                    | WaveformGenerationMode::PwmPhaseOcrA => {
                        self.upcounting = false;
                        self.last_write_counter = top - 1;
                    }
                    WaveformGenerationMode::Reserved => todo!(),
                }
            } else if new_counter > top as i64 + 1 {
                panic!("Skipped event!");
            } else {
                self.last_write_counter = new_counter as u16;
            }
            self.last_write_t = timestamp;
        } else {
            let new_counter = self.last_write_counter - ticks as u16;
            if ticks - 1 == self.last_write_counter as i64 {
                match self.waveform_mode {
                    WaveformGenerationMode::Normal
                    | WaveformGenerationMode::Ctc
                    | WaveformGenerationMode::CtcIcr
                    | WaveformGenerationMode::FastPwm10Bit
                    | WaveformGenerationMode::FastPwm9Bit
                    | WaveformGenerationMode::FastPwm8Bit
                    | WaveformGenerationMode::FastPwmIcr
                    | WaveformGenerationMode::FastPwmOcrA => {
                        panic!("Downcounting impossible")
                    }
                    WaveformGenerationMode::Pwm8Bit
                    | WaveformGenerationMode::Pwm9Bit
                    | WaveformGenerationMode::Pwm10Bit
                    | WaveformGenerationMode::PwmPhaseFreqIcr
                    | WaveformGenerationMode::PwmPhaseFreqOcrA
                    | WaveformGenerationMode::PwmPhaseIcr
                    | WaveformGenerationMode::PwmPhaseOcrA => {
                        self.upcounting = true;
                        self.last_write_counter = 1;
                    }
                    WaveformGenerationMode::Reserved => todo!(),
                }
            } else if ticks - 1 > self.last_write_counter as i64 {
                panic!("Skipped event!");
            } else {
                self.last_write_counter = new_counter as u16;
            }
            self.last_write_t = timestamp;
        }

        for i in 0..3 {
            if self.ocr[i] == self.last_write_counter {
                self.trigger_oc(i, queue);
            }
        }
        if self.interrupt_masks.overflow && self.overflow_value() == self.last_write_counter {
            self.interrupt_flags.overflow = true;
            queue.fire_event_now(InternalEvent {
                receiver_id: self.interrupt_reciever,
            })
        }
    }

    fn calculate_counter(&self, timestamp: TickTimestamp) -> u16 {
        let ticks = self.ticks_up_to(timestamp);
        if ticks > self.timer_ticks_until_next_event() as i64 {
            panic!("Skipped event!");
        }
        if self.upcounting {
            self.last_write_counter + ticks as u16
        } else {
            self.last_write_counter - ticks as u16
        }
    }

    fn schedule_event(&mut self, queue: &mut EventQueue) {
        if self.clock_mode == ClockMode::Disabled {
            return;
        }

        let timer_ticks = self.timer_ticks_until_next_event();
        let next_event = self.add_ticks(queue.clock.current_tick(), timer_ticks as i64);
        queue.fire_event_at_ticks(
            InternalEvent {
                receiver_id: self.module_id.with_event_port(0),
            },
            next_event,
        )
    }
}

impl Module for Timer16 {
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    fn handle_event(&mut self, event: InternalEvent, queue: &mut EventQueue) {
        assert_eq!(event.receiver_id.event_port_id, 0);
        self.simulate(queue.clock.current_tick(), queue);
        self.schedule_event(queue);
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

    fn to_wireable(&mut self) -> Option<&mut dyn WireableModule> {
        Some(self)
    }
}

impl DataModule for Timer16 {
    type PortType = u8;

    fn read_port(&self, queue: &EventQueue, id: PortId) -> Self::PortType {
        match id {
            0 => {
                // TCCRnA
                let wgm = (self.waveform_mode as u8) & 0x3;
                let coma = self.compare_output_mode[0] as u8;
                let comb = self.compare_output_mode[1] as u8;
                let comc = self.compare_output_mode[2] as u8;
                coma << 6 | comb << 4 | comc << 2 | wgm
            }
            1 => {
                // TCCRnB
                let cs = self.clock_mode as u8;
                let wgm = (self.waveform_mode as u8 >> 2) & 0x3;
                let icnc = self.icnc as u8;
                let ices = self.ices as u8;
                icnc << 7 | ices << 6 | wgm << 3 | cs
            }
            2 => 0, // TCCRnC
            3 => 0, // Reserved
            4 => {
                // TCNTnL
                let cnt = self.calculate_counter(queue.clock.current_tick());
                (cnt & 0xFF) as u8
            }
            5 => {
                // TCNTnH
                let cnt = self.calculate_counter(queue.clock.current_tick());
                (cnt >> 8) as u8
            }
            6 | 7 => todo!(),                 // ICRnL/ICRnH
            8 => (self.ocr[0] & 0xFF) as u8,  // OCRnAL
            9 => (self.ocr[0] >> 8) as u8,    // OCRnAH
            10 => (self.ocr[1] & 0xFF) as u8, // OCRnBL
            11 => (self.ocr[1] >> 8) as u8,   // OCRnBH
            12 => (self.ocr[2] & 0xFF) as u8, // OCRnCL
            13 => (self.ocr[2] >> 8) as u8,   // OCRnCH

            14 | 15 => 0, // Reserved

            Self::TIMSK_PORT => {
                let icie = self.interrupt_masks.input_capture as u8;
                let ociea = self.interrupt_masks.oc[0] as u8;
                let ocieb = self.interrupt_masks.oc[1] as u8;
                let ociec = self.interrupt_masks.oc[2] as u8;
                let toie = self.interrupt_masks.overflow as u8;

                icie << 5 | ociea << 1 | ocieb << 2 | ociec << 3 | toie
            }

            Self::TIFR_PORT => {
                let icf = self.interrupt_flags.input_capture as u8;
                let ocfa = self.interrupt_flags.oc[0] as u8;
                let ocfb = self.interrupt_flags.oc[1] as u8;
                let ocfc = self.interrupt_flags.oc[2] as u8;
                let tof = self.interrupt_flags.overflow as u8;

                icf << 5 | ocfa << 1 | ocfb << 2 | ocfc << 3 | tof
            }
            _ => panic!("Invalid port {}", id),
        }
    }

    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: u8) {
        self.simulate(queue.clock.current_tick(), queue);
        match id {
            0 => {
                // TCCRnA
                let wgm = data & 0x3;
                let coma = (data >> 6) & 0x3;
                let comb = (data >> 4) & 0x3;
                let comc = (data >> 2) & 0x3;

                unsafe {
                    self.waveform_mode = transmute((self.waveform_mode as u8 & 0x0C) | wgm);
                    self.compare_output_mode[0] = transmute(coma);
                    self.compare_output_mode[1] = transmute(comb);
                    self.compare_output_mode[2] = transmute(comc);
                }
                self.upcounting = true;
                for i in 0..3 {
                    queue.set_multiplexer_flag(
                        self.module_id.with_pin(i),
                        self.compare_output_mode[i as usize] != CompareOutputMode::Disabled,
                    );
                }
            }
            1 => {
                // TCCRnB
                let cs = data & 0x7;
                let wgm = (data >> 3) & 0x3;
                let icnc = (data >> 7) & 0x1;
                let ices = (data >> 6) & 0x1;

                unsafe {
                    self.clock_mode = transmute(cs);
                    self.waveform_mode = transmute((self.waveform_mode as u8 & 0x03) | wgm << 2);
                }
                self.upcounting = true;
                self.icnc = icnc != 0;
                self.ices = ices != 0;
            }
            2 => todo!(), // TCCRnC
            3 => {}       // Reserved
            4 => {
                // TCNTnL
                assert!(self.last_write_t == queue.clock.current_tick());
                self.last_write_counter = (self.last_write_counter & 0xFF00) | data as u16;
            }
            5 => {
                // TCNTnH
                assert!(self.last_write_t == queue.clock.current_tick());
                self.last_write_counter = (self.last_write_counter & 0x00FF) | (data as u16) << 8;
            }
            6 | 7 => todo!(),                                        // ICRnL/ICRnH
            8 => self.ocr[0] = (self.ocr[0] & 0xFF00) | data as u16, // OCRnAL
            9 => self.ocr[0] = (self.ocr[0] & 0x00FF) | (data as u16) << 8, // OCRnAH
            10 => self.ocr[1] = (self.ocr[1] & 0xFF00) | data as u16, // OCRnBL
            11 => self.ocr[1] = (self.ocr[1] & 0x00FF) | (data as u16) << 8, // OCRnBH
            12 => self.ocr[2] = (self.ocr[2] & 0xFF00) | data as u16, // OCRnCL
            13 => self.ocr[2] = (self.ocr[2] & 0x00FF) | (data as u16) << 8, // OCRnCH

            14 | 15 => {} // Reserved

            Self::TIMSK_PORT => {
                self.interrupt_masks.input_capture = (data & 0x20) != 0;
                self.interrupt_masks.oc[0] = (data & 0x02) != 0;
                self.interrupt_masks.oc[1] = (data & 0x04) != 0;
                self.interrupt_masks.oc[2] = (data & 0x08) != 0;
                self.interrupt_masks.overflow = (data & 0x01) != 0;
            }

            Self::TIFR_PORT => {
                self.interrupt_flags.input_capture &= (data & 0x20) == 0; // Clear if 1
                self.interrupt_flags.oc[0] &= (data & 0x02) == 0;
                self.interrupt_flags.oc[1] &= (data & 0x04) == 0;
                self.interrupt_flags.oc[2] &= (data & 0x08) == 0;
                self.interrupt_flags.overflow &= (data & 0x01) == 0;
            }
            _ => panic!("Invalid port {}", id),
        }
        self.schedule_event(queue);
    }
}

impl WireableModule for Timer16 {
    fn get_pin(&self, _queue: &EventQueue, id: PinId) -> WireState {
        match id {
            0..=2 => WireState::from_bool(self.pins[id as usize]),
            _ => panic!("Invalid pin {}", id),
        }
    }

    fn set_pin(&mut self, _queue: &mut EventQueue, _id: PinId, _data: WireState) {}
}
