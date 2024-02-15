use crate::{
    events::{EventQueue, InternalEvent},
    module::{Module, PinId, WireableModule},
    module_id::ModuleAddress,
    pin_state::{InputPinState, WireState},
};

#[derive(Debug, Clone, Copy)]
pub struct Led {
    module_id: ModuleAddress,
    state: bool,
}

impl Led {
    pub fn new(module_id: ModuleAddress) -> Led {
        Led {
            module_id,
            state: false,
        }
    }
}

impl Module for Led {
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    fn handle_event(&mut self, _event: InternalEvent, _queue: &mut EventQueue) {
        panic!("LED can't handle events");
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

impl WireableModule for Led {
    fn get_pin(&self, _queue: &EventQueue, _id: PinId) -> WireState {
        WireState::Z
    }

    fn set_pin(&mut self, queue: &mut EventQueue, _id: PinId, data: WireState) {
        match InputPinState::read_wire_state(data) {
            InputPinState::High => {
                if !self.state {
                    println!("{}: LED ON: {}", queue.clock.current_time(), self.module_id);
                }
                self.state = true;
            }
            InputPinState::Low => {
                if self.state {
                    println!(
                        "{}: LED OFF: {}",
                        queue.clock.current_time(),
                        self.module_id
                    );
                }
                self.state = false;
            }
        }
    }
}
