use bitfield::{Bit, BitMut};

use crate::{
    events::{EventQueue, InternalEvent},
    module::{DataModule, Module, PinId, PortId, WireableModule},
    module_id::ModuleAddress,
    pin_state::{InputPinState, WireState},
};

#[derive(Debug, Clone)]
pub struct GpioBank {
    module_id: ModuleAddress,
    port_register: u8,
    ddr_register: u8,

    output_states: [WireState; 8],
    input_states: [InputPinState; 8],
    readable_states: [InputPinState; 8],
}
impl GpioBank {
    pub fn new(module_id: ModuleAddress) -> GpioBank {
        GpioBank {
            module_id,
            port_register: 0,
            ddr_register: 0,
            output_states: [WireState::Z; 8],
            input_states: [InputPinState::Low; 8],
            readable_states: [InputPinState::Low; 8],
        }
    }

    fn read_pin(&self) -> u8 {
        let mut x = 0;
        for i in 0..8 {
            if self.readable_states[i] == InputPinState::High {
                x.set_bit(i, true);
            }
        }
        x
    }

    fn write_pin(&mut self, val: u8, queue: &mut EventQueue) {
        for i in 0..8 {
            if val.bit(i) {
                self.port_register ^= 1 << i;
            }
        }
        self.update_outputs(queue);
    }

    #[inline]
    fn set_output_state(&mut self, i: usize, state: WireState, queue: &mut EventQueue) {
        if self.output_states[i] != state {
            queue.set_wire(self.module_id.with_pin(i as u8), state);
        }
        self.output_states[i] = state;
        self.input_states[i] = InputPinState::read_wire_state(state);
    }

    fn update_outputs(&mut self, queue: &mut EventQueue) {
        for i in 0..8 {
            let port = self.port_register.bit(i);
            let dd = self.ddr_register.bit(i);
            match (dd, port) {
                (false, false) => self.set_output_state(i, WireState::Z, queue),
                (false, true) => self.set_output_state(i, WireState::WeakHigh, queue),
                (true, false) => self.set_output_state(i, WireState::Low, queue),
                (true, true) => self.set_output_state(i, WireState::High, queue),
            }
        }
    }
}

impl Module for GpioBank {
    #[inline]
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    fn handle_event(&mut self, event: InternalEvent, _queue: &mut EventQueue) {
        assert_eq!(event.receiver_id.event_port_id, 0);
        self.readable_states = self.input_states;
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

impl WireableModule for GpioBank {
    fn get_pin(&self, _queue: &EventQueue, id: PinId) -> WireState {
        self.output_states[id]
    }

    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState) {
        self.input_states[id] = InputPinState::read_wire_state(data);
        queue.fire_event_next_tick(InternalEvent {
            receiver_id: self.module_id.with_event_port(0),
        });
    }
}

impl DataModule for GpioBank {
    type PortType = u8;
    fn read_port(&self, _queue: &EventQueue, id: PortId) -> u8 {
        match id {
            0 => self.read_pin(),
            1 => self.ddr_register,
            2 => self.port_register,
            _ => 0,
        }
    }

    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: Self::PortType) {
        match id {
            0 => self.write_pin(data, queue),
            1 => {
                self.ddr_register = data;
                self.update_outputs(queue);
            }
            2 => {
                self.port_register = data;
                self.update_outputs(queue);
            }
            _ => {}
        }
    }
}
