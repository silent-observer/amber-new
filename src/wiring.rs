use std::collections::HashMap;

use kanal::{Receiver, Sender};
use once_cell::sync::OnceCell;

use crate::{events::Event, module_id::ModuleId};

pub struct InboxTable(HashMap<u8, Sender<Event>>);

static INBOX_TABLE: OnceCell<InboxTable> = OnceCell::new();

impl InboxTable {
    pub fn new() -> Self {
        InboxTable(HashMap::new())
    }
    pub fn add_listener(&mut self, id: u8) -> Receiver<Event> {
        let (s, r) = kanal::bounded(4);
        self.0.insert(id, s);
        r
    }
    pub fn save(self: Self) {
        INBOX_TABLE.get_or_init(|| self);
    }
    pub fn send(e: Event) {
        let it = INBOX_TABLE.get().expect("Uninitialized InboxTable!");
        if let Some(s) = it.0.get(&e.receiver_id.current()) {
            s.send(e).expect("Couldn't send event");
        } else {
            panic!("Unknown receiver id: {}", e.receiver_id);
        }
    }
}

pub type WireId = u32;

pub struct WiringTable(HashMap<ModuleId, Vec<ModuleId>>);

static WIRING_TABLE: OnceCell<WiringTable> = OnceCell::new();

impl WiringTable {
    pub fn new() -> Self {
        WiringTable(HashMap::new())
    }
    pub fn add_wire(&mut self, from: ModuleId, to: Vec<ModuleId>) {
        self.0.insert(from, to);
    }
    pub fn save(self: Self) {
        WIRING_TABLE.get_or_init(|| self);
    }
    pub fn get_connected(id: ModuleId) -> Option<&'static Vec<ModuleId>> {
        let wt = WIRING_TABLE.get().expect("Uninitialized WiringTable!");
        wt.0.get(&id)
    }
}
