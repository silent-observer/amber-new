use std::sync::{Arc, RwLock};

use crate::wiring::{InboxTable, WiringTable};

#[derive(Debug, Clone)]
pub struct SystemTables {
    pub inbox: Arc<RwLock<InboxTable>>,
    pub wiring: Arc<RwLock<WiringTable>>,
    pub messages: Arc<RwLock<Vec<String>>>,
}

impl SystemTables {
    pub fn new() -> Self {
        SystemTables {
            inbox: Arc::new(RwLock::new(InboxTable::new())),
            wiring: Arc::new(RwLock::new(WiringTable::new())),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}
