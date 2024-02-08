use std::ops::Deref;

use crate::{
    events::{Event, EventData, EventQueue, EventReceiver},
    module::{Module, WireableModule},
    module_id::ModuleId,
};

#[derive(Debug)]
pub struct PassiveModuleStore {
    module_id: ModuleId,
    modules: Vec<Box<dyn WireableModule>>,
}

impl PassiveModuleStore {
    pub fn new() -> Self {
        Self {
            module_id: ModuleId::default(),
            modules: Vec::new(),
        }
    }

    pub fn add_module(&mut self, mut module: Box<dyn WireableModule>) -> &dyn WireableModule {
        let module_id = self.module_id.child_id(self.modules.len() as u8);
        module.set_module_id(module_id);
        self.modules.push(module);
        self.modules.last().unwrap().deref()
    }
}

impl Module for PassiveModuleStore {
    #[inline]
    fn module_id(&self) -> ModuleId {
        self.module_id
    }

    fn set_module_id(&mut self, module_id: ModuleId) {
        self.module_id = module_id;
        for (i, module) in self.modules.iter_mut().enumerate() {
            module.set_module_id(self.module_id.child_id(i as u8));
        }
    }
}

impl EventReceiver for PassiveModuleStore {
    fn receive_event(&mut self, event: Event, queue: &mut EventQueue) {
        let i = event.receiver_id.current() as usize;
        let port = event.receiver_id.event_port as usize;
        assert!(i < self.modules.len());
        if let EventData::WireState(state) = event.data {
            self.modules[i].set_pin(queue, port, state);
        } else {
            panic!("Cannot send non-wire event to passive module");
        }
    }
}
