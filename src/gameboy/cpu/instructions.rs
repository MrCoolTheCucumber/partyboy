use crate::gameboy::bus::Bus;

use super::{register::Flag, Cpu};

type InstructionFn = fn(&mut Cpu, &mut Bus) -> InstructionState;

#[derive(Clone, Copy)]
pub enum InstructionState {
    /// There are more steps to execute, wait 4T
    InProgress,

    /// There are more steps to execute, execute the next step instantly
    ExecNextInstantly,

    /// We've finished fully executing the opcode
    Finished,

    /// TODO
    Branch(bool),
}

pub enum InstructionStep {
    Standard(InstructionFn),
    Instant(InstructionFn),
}

#[derive(Clone, Copy)]
pub enum InstructionOpcode {
    Unprefixed(u8),
    Prefixed(u8),
}

pub struct Instruction {
    index: usize,
    steps: Vec<InstructionStep>,
}

impl Instruction {
    pub fn exec(&mut self, cpu: &mut Cpu, bus: &mut Bus) -> InstructionState {
        let state = match self.steps[self.index] {
            InstructionStep::Standard(step) => step(cpu, bus),
            InstructionStep::Instant(step) => step(cpu, bus),
        };
        self.index += 1;
        state
    }

    pub fn get(&self) -> &InstructionStep {
        &self.steps[self.index]
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }
}

const __FETCH_OPERAND8: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    cpu.operand8 = cpu.fetch(bus);
    InstructionState::InProgress
});

const __FETCH_OPERAND16: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    let hi = cpu.fetch(bus);
    cpu.operand16 = (hi as u16) << 8 | cpu.operand8 as u16;
    InstructionState::InProgress
});

const BLANK_PROGRESS: InstructionStep =
    InstructionStep::Standard(|_, _| InstructionState::InProgress);

const BRANCH_ZERO: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(cpu.is_flag_set(Flag::Z)));

const BRANCH_NOT_ZERO: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(!cpu.is_flag_set(Flag::Z)));

const BRANCH_CARRY: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(cpu.is_flag_set(Flag::C)));

const BRANCH_NOT_CARRY: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(!cpu.is_flag_set(Flag::C)));

// TODO: There's probably a more DRY way of doing this..
macro_rules! instruction {
    (fetch8, $($step:expr),*) => {
        {
            let mut steps: Vec<InstructionStep> = Vec::new();
            steps.push(__FETCH_OPERAND8);

            $(
                steps.push($step);
            )*

            Instruction {
                index: 0,
                steps
            }
        }
    };

    (fetch16, $($step:expr),*) => {
        {
            let mut steps: Vec<InstructionStep> = Vec::new();
            steps.push(__FETCH_OPERAND8);
            steps.push(__FETCH_OPERAND16);

            $(
                steps.push($step);
            )*

            Instruction {
                index: 0,
                steps
            }
        }
    };

    ($($step:expr),*) => {
        {
            let mut steps: Vec<InstructionStep> = Vec::new();
            $(
                steps.push($step);
            )*

            Instruction {
                index: 0,
                steps
            }
        }
    };
}

macro_rules! ld_r16_u16 {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.$reg.lo = cpu.fetch(bus);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.$reg.hi = cpu.fetch(bus);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! add_hl_r16 {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, _| {
                let (result, overflown) = u16::from(cpu.hl).overflowing_add(u16::from(cpu.$reg));

                cpu.clear_flag(Flag::N);
                cpu.set_flag_if_cond_else_clear(overflown, Flag::C);

                let half_carry_occured = (u16::from(cpu.hl) & 0xFFF) + ((u16::from(cpu.$reg) & 0xFFF)) > 0xFFF;
                cpu.set_flag_if_cond_else_clear(half_carry_occured, Flag::H);

                cpu.hl = result.into();
                InstructionState::Finished
            })
        }
    };
}

macro_rules! ld_mem_a {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.af.hi = bus.read_u8(cpu.$reg.into());
                InstructionState::Finished
            })
        }
    };
}

macro_rules! inc_r16 {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, _| {
                cpu.$reg += 1;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! dec_r16 {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, _| {
                cpu.$reg -= 1;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! inc_r8 {
    ($reg:ident,$bit:ident) => {
        instruction! {
            InstructionStep::Instant(|cpu, _| {
                cpu.set_flag_if_cond_else_clear((cpu.$reg.$bit & 0x0F) == 0x0F, Flag::H);
                cpu.$reg.$bit = cpu.$reg.$bit.wrapping_add(1);
                cpu.handle_z_flag(cpu.$reg.$bit);
                cpu.clear_flag(Flag::N);

                InstructionState::Finished
            })
        }
    };
}

macro_rules! dec_r8 {
    ($reg:ident,$bit:ident) => {
        instruction! {
            InstructionStep::Instant(|cpu, _| {
                cpu.set_flag_if_cond_else_clear((cpu.$reg.$bit & 0x0F) == 0x0F, Flag::H);
                cpu.$reg.$bit = cpu.$reg.$bit.wrapping_sub(1);
                cpu.handle_z_flag(cpu.$reg.$bit);
                cpu.set_flag(Flag::N);

                InstructionState::Finished
            })
        }
    };
}

macro_rules! ld_r8_u8 {
    ($reg:ident,$bit:ident) => {
        instruction! {
            fetch8,
            InstructionStep::Instant(|cpu, _| {
                cpu.$reg.$bit = cpu.operand8;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! ld_a_mem {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.af.hi = bus.read_u8(cpu.$reg.into());
                InstructionState::Finished
            })
        }
    };
}

macro_rules! branch_condition {
    (Z) => {
        BRANCH_ZERO
    };
    (NZ) => {
        BRANCH_NOT_ZERO
    };
    (C) => {
        BRANCH_CARRY
    };
    (NC) => {
        BRANCH_NOT_CARRY
    };
}

macro_rules! jr_i8 {
    ($($cond:tt)?) => {
        instruction! {
            fetch8,
            $(
                branch_condition!($cond),
            )?
            InstructionStep::Standard(|cpu, _| {
                let jmp_amount = cpu.operand8 as i8;
                if jmp_amount < 0 {
                    cpu.pc = cpu.pc.wrapping_sub(jmp_amount.abs() as u16);
                } else {
                    cpu.pc = cpu.pc.wrapping_add(jmp_amount as u16);
                }

                InstructionState::Finished
            })
        }
    };
}

impl Cpu {
    #[inline(always)]
    fn set_flag(&mut self, flag: Flag) {
        self.af.lo |= flag as u8;
    }

    #[inline(always)]
    fn clear_flag(&mut self, flag: Flag) {
        self.af.lo &= !(flag as u8);
    }

    #[inline(always)]
    fn is_flag_set(&self, flag: Flag) -> bool {
        self.af.lo & (flag as u8) > 0
    }

    #[inline(always)]
    fn set_flag_if_cond_else_clear(&mut self, cond: bool, flag: Flag) {
        match cond {
            true => self.af.lo |= flag as u8,
            false => self.af.lo &= !(flag as u8),
        }
    }

    #[inline(always)]
    fn handle_z_flag(&mut self, val: u8) {
        if val == 0 {
            self.af.lo |= Flag::Z as u8;
        } else {
            self.af.lo &= !(Flag::Z as u8);
        }
    }
}

pub struct InstructionCache {
    instructions: [Instruction; 256],
    // cb_instructions: [Instruction; 256],
}

impl InstructionCache {
    pub fn new() -> Self {
        Self {
            instructions: Self::gen_instructions(),
            // cb_instructions: Self::gen_cb_instructions(),
        }
    }

    fn gen_instructions() -> [Instruction; 256] {
        let helper = |opcode: u8| match opcode {
            0x00 => instruction!(InstructionStep::Instant(|_, _| InstructionState::Finished)),
            0x01 => ld_r16_u16!(bc),
            0x02 => ld_mem_a!(bc),
            0x03 => inc_r16!(bc),
            0x04 => inc_r8!(bc, hi),
            0x05 => dec_r8!(bc, hi),
            0x06 => ld_r8_u8!(bc, hi),
            0x07 => instruction!(InstructionStep::Instant(|_, _| unimplemented!("RLCA"))),
            0x08 => instruction! { // LD (u16), SP
                fetch16,
                BLANK_PROGRESS,
                InstructionStep::Standard(|cpu, bus| {
                    bus.write_u16(cpu.operand16, cpu.sp);
                    InstructionState::Finished
                })
            },
            0x09 => add_hl_r16!(bc),
            0x0A => ld_a_mem!(bc),
            0x0B => dec_r16!(bc),
            0x0C => inc_r8!(bc, lo),
            0x0D => dec_r8!(bc, lo),
            0x0E => ld_r8_u8!(bc, lo),
            0x0F => instruction!(InstructionStep::Instant(|_, _| unimplemented!("RRCA"))),

            0x10 => instruction!(InstructionStep::Instant(|_, _| unimplemented!("STOP"))),
            0x11 => ld_r16_u16!(de),
            0x12 => ld_mem_a!(de),
            0x13 => inc_r16!(de),
            0x14 => inc_r8!(de, hi),
            0x15 => dec_r8!(de, hi),
            0x16 => ld_r8_u8!(de, hi),
            0x17 => instruction!(InstructionStep::Instant(|_, _| unimplemented!("RLA"))),
            0x18 => jr_i8!(),
            0x19 => add_hl_r16!(de),
            0x1A => ld_a_mem!(de),
            0x1B => dec_r16!(de),
            0x1C => inc_r8!(de, lo),
            0x1D => dec_r8!(de, lo),
            0x1E => ld_r8_u8!(de, lo),
            0x1F => instruction!(InstructionStep::Instant(|_, _| unimplemented!("RRA"))),

            0x20 => jr_i8!(NZ),
            0x21 => ld_r16_u16!(hl),

            _ => instruction!(InstructionStep::Instant(|_, _| unimplemented!(""))),
        };

        // mini playground
        let _ = |cpu: &mut Cpu| {
            let (result, overflown) = u16::from(cpu.hl).overflowing_add(15);
        };

        let mut instructions = Vec::new();
        for opcode in 0..=255 {
            instructions.push(helper(opcode));
        }

        instructions
            .try_into()
            .unwrap_or_else(|_| panic!("Unable to convert instruction vec into array."))
    }

    fn gen_cb_instructions() -> [Instruction; 256] {
        todo!()
    }

    pub fn exec(
        &mut self,
        opcode: InstructionOpcode,
        cpu: &mut Cpu,
        bus: &mut Bus,
    ) -> InstructionState {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => {
                self.instructions[opcode as usize].exec(cpu, bus)
            }
            InstructionOpcode::Prefixed(opcode) => todo!(),
        }
    }

    pub fn get(&mut self, opcode: InstructionOpcode) -> &InstructionStep {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => self.instructions[opcode as usize].get(),
            InstructionOpcode::Prefixed(_) => todo!(),
        }
    }

    pub fn reset(&mut self, opcode: InstructionOpcode) {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => self.instructions[opcode as usize].reset(),
            InstructionOpcode::Prefixed(_) => todo!(),
        }
    }
}
