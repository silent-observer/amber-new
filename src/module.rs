use std::fmt::Debug;

use crate::{
    clock::Timestamp,
    events::{EventQueue, InternalEvent},
    module_holder::PassiveModuleStore,
    module_id::ModuleAddress,
    pin_state::WireState,
};

pub type PortId = usize;
pub type PinId = usize;

pub trait Module: Debug {
    fn address(&self) -> ModuleAddress;
    fn handle_event(&mut self, event: InternalEvent, queue: &mut EventQueue, t: Timestamp);
    fn find(&self, address: ModuleAddress) -> Option<&dyn Module>;
    fn find_mut(&mut self, address: ModuleAddress) -> Option<&mut dyn Module>;

    fn to_wireable(&self) -> Option<&dyn WireableModule>;
    fn to_wireable_mut(&mut self) -> Option<&mut dyn WireableModule>;
}

pub trait DataModule: Module {
    type PortType;
    fn read_port(&self, queue: &EventQueue, id: PortId) -> Self::PortType;
    fn write_port(&mut self, queue: &mut EventQueue, id: PortId, data: Self::PortType);
}

pub trait WireableModule: Module {
    fn get_pin(&self, queue: &EventQueue, id: PinId) -> WireState;
    fn set_pin(&mut self, queue: &mut EventQueue, id: PinId, data: WireState);
}

pub trait ActiveModule: Module {
    fn run_until_time(&mut self, t: Timestamp) -> Timestamp;
    fn module_store(&mut self) -> &mut PassiveModuleStore;
    fn event_queue(&self) -> &EventQueue;
}
