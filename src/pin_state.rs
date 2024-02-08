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
