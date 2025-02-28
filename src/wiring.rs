use std::collections::HashMap;

use kanal::{Receiver, Sender};

use crate::{clock::Timestamp, events::WireChangeEvent, module_id::PinAddress};

#[derive(Debug)]
pub struct InboxTable(HashMap<u8, Sender<(WireChangeEvent, Timestamp)>>);

impl InboxTable {
    pub fn new() -> Self {
        InboxTable(HashMap::new())
    }
    pub fn add_listener(&mut self, id: u8) -> Receiver<(WireChangeEvent, Timestamp)> {
        let (s, r) = kanal::bounded(4);
        self.0.insert(id, s);
        r
    }
    pub fn send(&self, e: WireChangeEvent, t: Timestamp) {
        if let Some(s) = self.0.get(&e.receiver_id.module_address.current()) {
            s.send((e, t)).expect("Couldn't send event");
        } else {
            panic!("Unknown receiver id: {}", e.receiver_id);
        }
    }
}

pub type WireId = u32;

#[derive(Debug)]
pub struct WiringTable(HashMap<PinAddress, Vec<PinAddress>>);

impl WiringTable {
    pub fn new() -> Self {
        WiringTable(HashMap::new())
    }
    pub fn add_wire(&mut self, from: PinAddress, to: Vec<PinAddress>) {
        self.0.insert(from, to);
    }
    pub fn get_connected(&self, id: PinAddress) -> Option<&Vec<PinAddress>> {
        self.0.get(&id)
    }
}
