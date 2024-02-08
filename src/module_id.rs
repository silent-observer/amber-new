use std::fmt::Display;

const SELF_ADDRESS: u8 = 0xFF;
const MODULE_ID_MAX_LENGTH: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId {
    pub depth: u8,
    pub event_port: u8,
    pub address: [u8; MODULE_ID_MAX_LENGTH],
}

impl Default for ModuleId {
    fn default() -> Self {
        ModuleId::root()
    }
}

impl ModuleId {
    pub const fn root() -> Self {
        Self {
            depth: 0,
            event_port: SELF_ADDRESS,
            address: [0; MODULE_ID_MAX_LENGTH],
        }
    }
    pub const fn is_empty(&self) -> bool {
        self.depth == 0
    }
    pub const fn current(&self) -> u8 {
        self.address[self.depth as usize]
    }
    pub fn advance(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }
    pub const fn with_event(mut self: Self, port: u8) -> ModuleId {
        self.event_port = port;
        self
    }
    pub const fn child_id(self: Self, id: u8) -> ModuleId {
        // Unelegant, but the only way I managed to make it work at compile-time
        let address = [
            id,
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5],
        ];
        ModuleId {
            depth: self.depth + 1,
            event_port: SELF_ADDRESS,
            address,
        }
    }
}

impl Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.depth > 0 {
            for i in (1..self.depth).rev() {
                write!(f, "{:02x}.", self.address[i as usize])?;
            }
            write!(f, "{:02x}", self.address[0])?;
        }
        write!(f, ":{:02x}", self.event_port)
    }
}
