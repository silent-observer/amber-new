use kanal::Sender;

use crate::{
    clock::Timestamp,
    events::{EventQueue, InternalEvent},
    module::{Module, PinId, WireableModule},
    module_id::ModuleAddress,
    pin_state::{InputPinState, WireState},
    vcd::{VcdEvent, VcdSender, VcdSignal},
};

#[derive(Debug, Clone)]
pub struct Led {
    module_id: ModuleAddress,
    state: bool,

    vcd_sender: Option<Sender<VcdEvent>>,
    vcd_start_id: i32,
}

impl Led {
    pub fn new(module_id: ModuleAddress) -> Led {
        Led {
            module_id,
            state: false,
            vcd_sender: None,
            vcd_start_id: 0,
        }
    }
}

impl VcdSender for Led {
    fn register_vcd(&mut self, sender: Sender<VcdEvent>, start_id: i32) -> (Vec<VcdSignal>, i32) {
        let signal = VcdSignal::Signal {
            name: "led".to_string(),
            id: start_id,
            size: 1,
        };
        self.vcd_sender = Some(sender);
        self.vcd_start_id = start_id;
        (vec![signal], 1)
    }

    fn vcd_sender(&self) -> Option<&Sender<VcdEvent>> {
        self.vcd_sender.as_ref()
    }
}

impl Module for Led {
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    fn handle_event(&mut self, _event: InternalEvent, _queue: &mut EventQueue, _t: Timestamp) {
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

    fn to_wireable(&self) -> Option<&dyn WireableModule> {
        Some(self)
    }
    fn to_wireable_mut(&mut self) -> Option<&mut dyn WireableModule> {
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
                    queue.add_message(format!(
                        "{}: LED ON: {}",
                        queue.clock.current_time(),
                        self.module_id
                    ));
                }
                self.state = true;
            }
            InputPinState::Low => {
                if self.state {
                    queue.add_message(format!(
                        "{}: LED OFF: {}",
                        queue.clock.current_time(),
                        self.module_id
                    ));
                }
                self.state = false;
            }
        }
        self.send_vcd(queue.clock.current_time(), self.vcd_start_id, &[data]);
    }
}
