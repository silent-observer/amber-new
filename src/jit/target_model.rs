use smallvec::SmallVec;

pub trait TargetModel {
    type DataType;
    type AddressType;
    const MAIN_REGISTER_COUNT: u8;
    const MAIN_MEMORY_START: Self::AddressType;
}

struct AvrTarget;

impl TargetModel for AvrTarget {
    type DataType = u8;
    type AddressType = u16;
    const MAIN_REGISTER_COUNT: u8 = 32;
    const MAIN_MEMORY_START: Self::AddressType = 0x200;
}
