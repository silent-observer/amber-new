use std::fmt::Debug;

use crate::{
    clock::Timestamp,
    events::{EventQueue, EventReceiver},
    module_holder::PassiveModuleStore,
    module_id::ModuleId,
    pin_state::WireState,
};

pub type PortId = usize;
pub type PinId = usize;

pub trait Module: Debug {
    fn module_id(&self) -> ModuleId;
    fn set_module_id(&mut self, module_id: ModuleId);
}

pub trait DataModule: Module {
    type PortType;
    fn read_port(&self, queue: &EventQueue, id: PortId) -> Self::PortType;
    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: Self::PortType);
}

pub trait WireableModule: Module {
    fn get_pin(&self, queue: &EventQueue, id: PinId) -> WireState;
    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState);
    fn get_pin_module(&self, id: PinId) -> Option<ModuleId>;
}

pub trait ActiveModule: EventReceiver + Module {
    fn run_until_time(&mut self, t: Timestamp);
    fn module_store(&mut self) -> &mut PassiveModuleStore;
}
