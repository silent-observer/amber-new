use std::ops::Deref;

use crate::{
    clock::Timestamp,
    events::{EventQueue, InternalEvent},
    module::{Module, WireableModule},
    module_id::ModuleAddress,
};

#[derive(Debug)]
pub struct PassiveModuleStore {
    module_id: ModuleAddress,
    modules: Vec<Box<dyn WireableModule>>,
}

impl PassiveModuleStore {
    pub fn new(module_id: ModuleAddress) -> Self {
        Self {
            module_id,
            modules: Vec::new(),
        }
    }

    pub fn add_module<M, F>(&mut self, f: F) -> &dyn WireableModule
    where
        M: WireableModule + 'static,
        F: FnOnce(ModuleAddress) -> M,
    {
        let module_id = self.module_id.child_id(self.modules.len() as u8);
        let module = Box::new(f(module_id));
        self.modules.push(module);
        self.modules.last().unwrap().deref()
    }
}

impl Module for PassiveModuleStore {
    #[inline]
    fn address(&self) -> ModuleAddress {
        self.module_id
    }

    fn handle_event(&mut self, _event: InternalEvent, _queue: &mut EventQueue, _t: Timestamp) {
        panic!("Cannot send event to passive module store");
    }

    fn find(&self, mut address: ModuleAddress) -> Option<&dyn Module> {
        if address.is_empty() {
            return Some(self);
        }

        let i = address.current() as usize;
        assert!(i < self.modules.len());
        address.advance();
        self.modules[i].find(address)
    }

    fn find_mut(&mut self, mut address: ModuleAddress) -> Option<&mut dyn Module> {
        assert!(!address.is_empty());
        let i = address.current() as usize;
        assert!(i < self.modules.len());
        address.advance();
        self.modules[i].find_mut(address)
    }

    fn to_wireable(&self) -> Option<&dyn WireableModule> {
        None
    }
    fn to_wireable_mut(&mut self) -> Option<&mut dyn WireableModule> {
        None
    }
}
