use crate::{
    events::EventQueue,
    module::{Module, PinId, WireableModule},
    module_id::ModuleId,
    pin_state::{InputPinState, WireState},
};

#[derive(Debug, Clone, Copy)]
pub struct Led {
    module_id: ModuleId,
    state: bool,
}

impl Led {
    pub fn new() -> Led {
        Led {
            module_id: ModuleId::default(),
            state: false,
        }
    }
}

impl Module for Led {
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    fn set_module_id(&mut self, module_id: ModuleId) {
        self.module_id = module_id;
    }
}

impl WireableModule for Led {
    fn get_pin(&self, _id: PinId) -> WireState {
        WireState::Z
    }

    fn set_pin(&mut self, queue: &mut EventQueue, _id: PinId, data: WireState) {
        match InputPinState::read_wire_state(data) {
            InputPinState::High => {
                if !self.state {
                    // println!("{}: LED ON: {}", queue.clock.current_time(), self.module_id);
                }
                self.state = true;
            }
            InputPinState::Low => {
                if self.state {
                    // println!(
                    //     "{}: LED OFF: {}",
                    //     queue.clock.current_time(),
                    //     self.module_id
                    // );
                }
                self.state = false;
            }
        }
    }

    fn get_pin_module(&self, _id: PinId) -> Option<ModuleId> {
        Some(self.module_id.with_event(0))
    }
}
