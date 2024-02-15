use std::collections::HashMap;

use kanal::{Receiver, Sender};
use once_cell::sync::OnceCell;

use crate::{clock::Timestamp, events::WireChangeEvent, module_id::PinAddress};

pub struct InboxTable(HashMap<u8, Sender<(WireChangeEvent, Timestamp)>>);

static INBOX_TABLE: OnceCell<InboxTable> = OnceCell::new();

impl InboxTable {
    pub fn new() -> Self {
        InboxTable(HashMap::new())
    }
    pub fn add_listener(&mut self, id: u8) -> Receiver<(WireChangeEvent, Timestamp)> {
        let (s, r) = kanal::bounded(4);
        self.0.insert(id, s);
        r
    }
    pub fn save(self: Self) {
        INBOX_TABLE.get_or_init(|| self);
    }
    pub fn send(e: WireChangeEvent, t: Timestamp) {
        let it = INBOX_TABLE.get().expect("Uninitialized InboxTable!");
        if let Some(s) = it.0.get(&e.receiver_id.module_address.current()) {
            s.send((e, t)).expect("Couldn't send event");
        } else {
            panic!("Unknown receiver id: {}", e.receiver_id);
        }
    }
}

pub type WireId = u32;

pub struct WiringTable(HashMap<PinAddress, Vec<PinAddress>>);

static WIRING_TABLE: OnceCell<WiringTable> = OnceCell::new();

impl WiringTable {
    pub fn new() -> Self {
        WiringTable(HashMap::new())
    }
    pub fn add_wire(&mut self, from: PinAddress, to: Vec<PinAddress>) {
        self.0.insert(from, to);
    }
    pub fn save(self: Self) {
        WIRING_TABLE.get_or_init(|| self);
    }
    pub fn get_connected(id: PinAddress) -> Option<&'static Vec<PinAddress>> {
        let wt = WIRING_TABLE.get().expect("Uninitialized WiringTable!");
        wt.0.get(&id)
    }
}
