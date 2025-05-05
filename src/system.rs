use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};

use kanal::Sender;

use crate::{
    clock::{TimeDiff, Timestamp},
    module::{ActiveModule, Module, PinId, WireableModule},
    module_id::{ModuleAddress, PinAddress},
    pin_state::WireState,
    system_tables::SystemTables,
    vcd::{VcdEvent, VcdReceiver},
};

pub struct System {
    pub system_tables: SystemTables,
    pub modules: Vec<Box<dyn ActiveModule>>,
    pub id_map: HashMap<String, ModuleAddress>,
    pub vcd: Option<VcdReceiver>,
    pub vcd_sender: Sender<VcdEvent>,
    pub t: Timestamp,
}

impl System {
    pub fn run_for(&mut self, delta: i64) {
        self.modules[0].run_until_time(self.t + delta);
        self.t += delta;
    }

    pub fn run_realtime(&mut self, freq: i64) -> ! {
        let fps = 60;
        let timesteps = freq / fps;
        let delta = Duration::from_secs(1) / fps as u32;
        loop {
            let start = Instant::now();
            self.modules[0].run_until_time(self.t + timesteps);
            self.t += timesteps;
            let elapsed = start.elapsed();
            if elapsed < delta {
                sleep(delta - elapsed);
            }
        }
    }

    pub fn pin_address(&self, id: &str) -> PinAddress {
        find_pin_addr(id, &self.id_map, &self.modules)
    }

    pub fn find_module<'a>(&'a self, id: &str) -> &'a dyn Module {
        let mut addr = *self.id_map.get(id).unwrap();
        let root = self.modules[addr.current() as usize].as_ref();
        addr.advance();

        root.find(addr).unwrap()
    }

    pub fn find_module_mut<'a>(&'a mut self, id: &str) -> &'a mut dyn Module {
        let mut addr = *self.id_map.get(id).unwrap();
        let root = self.modules[addr.current() as usize].as_mut();
        addr.advance();

        root.find_mut(addr).unwrap()
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
