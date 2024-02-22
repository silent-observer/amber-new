use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use bitvec::array::BitArray;
use smallvec::SmallVec;

use super::target_model::TargetModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionKind(pub u8);

type InstructionField = u16;
const INSTRUCTION_FIELD_COUNT: usize = 4;
const MAX_FLAGS: usize = 8;

#[derive(Debug, Clone)]
pub struct Instruction<T> {
    pub kind: T,
    pub fields: SmallVec<[InstructionField; INSTRUCTION_FIELD_COUNT]>,
    pub used_flags: BitArray,
    pub address: usize,
}

impl<T> Instruction<T> {
    pub fn new(kind: T, address: usize) -> Self {
        Self {
            kind,
            fields: SmallVec::new(),
            used_flags: BitArray::ZERO,
            address,
        }
    }

    pub fn with_field(mut self, field: InstructionField) -> Self {
        self.fields.push(field);
        self
    }
}

impl<T: Debug> Display for Instruction<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {:?} F:[", self.kind, self.fields)?;
        for i in 0..MAX_FLAGS {
            write!(f, "{}", *self.used_flags.get(i).unwrap() as u8)?;
        }
        write!(f, "] @ {:04X}", self.address)
    }
}

pub struct InstructionBlock<IT> {
    pub instructions: Vec<Instruction<IT>>,
    pub addresses: HashMap<usize, usize>,
    pub range: (usize, usize),
}

pub type ConstantType = u32;

pub enum RegisterValue {
    Unchanged,
    Constant(ConstantType),
    Changed,
}
