use std::fmt::Display;

use crate::module::Module;

const MODULE_ID_MAX_LENGTH: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleAddress {
    pub depth: u8,
    pub address: [u8; MODULE_ID_MAX_LENGTH],
}

impl Default for ModuleAddress {
    fn default() -> Self {
        ModuleAddress::root()
    }
}

impl ModuleAddress {
    pub const fn root() -> Self {
        Self {
            depth: 0,
            address: [0; MODULE_ID_MAX_LENGTH],
        }
    }
    pub const fn is_empty(&self) -> bool {
        self.depth == 0
    }
    pub const fn current(&self) -> u8 {
        self.address[self.depth as usize - 1]
    }
    pub fn advance(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }
    pub const fn child_id(self: Self, id: u8) -> ModuleAddress {
        // Unelegant, but the only way I managed to make it work at compile-time
        let address = [
            id,
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
        ];
        ModuleAddress {
            depth: self.depth + 1,
            address,
        }
    }
    pub const fn with_event_port(self: Self, event_port_id: u8) -> EventPortAddress {
        EventPortAddress {
            module_address: self,
            event_port_id,
        }
    }
    pub const fn with_pin(self: Self, pin_id: u8) -> PinAddress {
        PinAddress {
            module_address: self,
            pin_id,
        }
    }
}

impl Display for ModuleAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.depth > 0 {
            for i in (1..self.depth).rev() {
                write!(f, "{:02x}.", self.address[i as usize])?;
            }
            write!(f, "{:02x}", self.address[0])
        } else {
            write!(f, "_")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventPortAddress {
    pub module_address: ModuleAddress,
    pub event_port_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PinAddress {
    pub module_address: ModuleAddress,
    pub pin_id: u8,
}

impl PinAddress {
    #[inline]
    pub fn from<M>(m: &M, pin_id: u8) -> Self
    where
        M: Module + ?Sized,
    {
        Self {
            module_address: m.address(),
            pin_id,
        }
    }
}

impl Display for EventPortAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.module_address, self.event_port_id)
    }
}

impl Display for PinAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}!{}", self.module_address, self.pin_id)
    }
}
