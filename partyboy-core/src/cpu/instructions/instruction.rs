use crate::{bus::Bus, cpu::Cpu};
use std::fmt::Debug;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type InstructionFn = fn(&mut Cpu, &mut Bus) -> InstructionState;

#[derive(Clone, Copy)]
pub enum InstructionState {
    /// There are more steps to execute, wait 4T
    InProgress,

    /// There are more steps to execute, execute the next step instantly
    ExecNextInstantly,

    /// We've finished fully executing the opcode
    Finished,

    /// Are we finishing the instruction early?
    Branch(bool),
}

pub enum InstructionStep {
    Standard(InstructionFn),
    Instant(InstructionFn),
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InstructionOpcode {
    InterruptServiceRoutine,
    Unprefixed(u8),
    Prefixed(u8),
}

impl Debug for InstructionOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InterruptServiceRoutine => f.write_str("ISR"),
            Self::Unprefixed(arg0) => f.write_str(format!("{:#06X}", arg0).as_str()),
            Self::Prefixed(arg0) => f.write_str(format!("{:#06X}", 0xCB00 + *arg0 as u16).as_str()),
        }
    }
}

pub struct Instruction {
    pub steps: Vec<InstructionStep>,
}
