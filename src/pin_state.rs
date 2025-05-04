#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireState {
    Low,
    High,
    Z,
    Error,
    WeakLow,
    WeakHigh,
}

impl WireState {
    pub fn from_bool(b: bool) -> WireState {
        match b {
            true => WireState::High,
            false => WireState::Low,
        }
    }
    pub fn combine(&self, other: &WireState) -> WireState {
        match (*self, *other) {
            (WireState::Z, x) | (x, WireState::Z) => x,
            (WireState::Error, _) | (_, WireState::Error) => WireState::Error,
            (WireState::High, WireState::Low) | (WireState::Low, WireState::High) => {
                WireState::Error
            }
            (WireState::WeakHigh, WireState::WeakLow)
            | (WireState::WeakLow, WireState::WeakHigh) => WireState::Error,

            (WireState::High, _) | (_, WireState::High) => WireState::High,
            (WireState::Low, _) | (_, WireState::Low) => WireState::Low,
            (WireState::WeakHigh, WireState::WeakHigh) => WireState::WeakHigh,
            (WireState::WeakLow, WireState::WeakLow) => WireState::WeakLow,
        }
    }

    pub fn to_bool(&self) -> bool {
        InputPinState::read_wire_state(*self) == InputPinState::High
    }

    pub fn from_u8(x: u8) -> [WireState; 8] {
        let mut r = [WireState::Low; 8];
        for i in 0..8 {
            r[i] = if ((x >> i) & 1) > 0 {
                WireState::High
            } else {
                WireState::Low
            };
        }
        r.reverse();
        r
    }

    pub fn from_u16(x: u16) -> [WireState; 16] {
        let mut r = [WireState::Low; 16];
        for i in 0..16 {
            r[i] = if ((x >> i) & 1) > 0 {
                WireState::High
            } else {
                WireState::Low
            };
        }
        r.reverse();
        r
    }

    pub fn from_u32(x: u32) -> [WireState; 32] {
        let mut r = [WireState::Low; 32];
        for i in 0..32 {
            r[i] = if ((x >> i) & 1) > 0 {
                WireState::High
            } else {
                WireState::Low
            };
        }
        r.reverse();
        r
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputPinState {
    Low,
    High,
}

impl InputPinState {
    pub fn to_wire_state(&self) -> WireState {
        match self {
            InputPinState::Low => WireState::Low,
            InputPinState::High => WireState::High,
        }
    }

    pub fn read_wire_state(wire_state: WireState) -> InputPinState {
        match wire_state {
            WireState::Low => InputPinState::Low,
            WireState::High => InputPinState::High,
            WireState::WeakLow => InputPinState::Low,
            WireState::WeakHigh => InputPinState::High,
            _ => InputPinState::High,
        }
    }
}
