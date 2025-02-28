use std::collections::HashMap;

use crate::{
    clock::Timestamp,
    module::{ActiveModule, PinId},
    module_id::{ModuleAddress, PinAddress},
    pin_state::WireState,
    system_tables::SystemTables,
};

pub struct System {
    pub system_tables: SystemTables,
    pub modules: Vec<Box<dyn ActiveModule>>,
    pub id_map: HashMap<String, ModuleAddress>,
    pub t: Timestamp,
}

impl System {
    pub fn run_for(&mut self, delta: i64) {
        self.modules[0].run_until_time(self.t + delta);
        self.t += delta;
    }

    pub fn pin_address(&self, id: &str) -> PinAddress {
        find_pin_addr(id, &self.id_map, &self.modules)
    }

    pub fn get_pin(&self, pin_addr: PinAddress) -> WireState {
        let root = self.modules[pin_addr.module_address.current() as usize].as_ref();
        let translated_addr = root.event_queue().lookup_pin(pin_addr);
        let mut addr = translated_addr.module_address;

        addr.advance();
        let m = root.find(addr).unwrap().to_wireable().unwrap();

        m.get_pin(root.event_queue(), translated_addr.pin_id as PinId)
    }
}

pub fn find_pin_addr(
    name: &str,
    id_map: &HashMap<String, ModuleAddress>,
    components: &[Box<dyn ActiveModule>],
) -> PinAddress {
    let (name, pin) = name.split_once(':').unwrap();
    let mut addr = *id_map.get(name).unwrap();
    let root = components[addr.current() as usize].as_ref();
    addr.advance();

    let m = root.find(addr).unwrap();
    PinAddress::from(m, pin.parse::<u8>().unwrap())
}
