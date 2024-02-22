use std::collections::HashMap;

use crate::jit::instructions::{Instruction, InstructionBlock};

use super::instructions::{decode, AvrInstructionType};

pub struct FirstPassResults {
    pub block: InstructionBlock<AvrInstructionType>,
}

pub fn first_pass(flash: &[u16], start_address: usize) -> FirstPassResults {
    let mut instructions: Vec<Instruction<AvrInstructionType>> = Vec::new();
    let mut addresses = HashMap::new();
    let mut current_address = start_address;

    let mut last_branch_target = current_address;
    let mut last_flag_changes: [Option<usize>; 8] = [None; 8];

    loop {
        addresses.insert(current_address, instructions.len());
        let (instr, size) = decode(flash, current_address);
        let is_end = instr.is_end();
        if let Some(target) = instr.branch_target(current_address) {
            if last_branch_target < target && target != current_address {
                last_branch_target = target;
            }
        }
        current_address += size as usize;

        for i in instr.reads_flags().iter_ones() {
            if let Some(addr) = last_flag_changes[i as usize] {
                instructions[addr].used_flags.set(i as usize, true);
                last_flag_changes[i as usize] = None;
            }
        }
        for i in instr.affected_flags().iter_ones() {
            last_flag_changes[i as usize] = Some(instructions.len());
        }
        instructions.push(instr);

        if is_end && current_address > last_branch_target {
            break;
        }
    }

    FirstPassResults {
        block: InstructionBlock {
            instructions,
            addresses,
            range: (start_address, current_address),
        },
    }
}
