use std::fmt::Debug;

use crate::bus::Bus;

use super::{register::Flag, Cpu};
use paste::paste;

type InstructionFn = fn(&mut Cpu, &mut Bus) -> InstructionState;

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

const FETCH_OP8_EXECNEXTINSTANT: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    cpu.operand8 = cpu.fetch(bus);
    InstructionState::ExecNextInstantly
});

const BLANK_PROGRESS: InstructionStep =
    InstructionStep::Standard(|_, _| InstructionState::InProgress);

const BLANK_PROGRESS_EXEC_NEXT_INSTANT: InstructionStep =
    InstructionStep::Standard(|_, _| InstructionState::ExecNextInstantly);

const BRANCH_ZERO: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(cpu.is_flag_set(Flag::Z)));

const BRANCH_NOT_ZERO: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(!cpu.is_flag_set(Flag::Z)));

const BRANCH_CARRY: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(cpu.is_flag_set(Flag::C)));

const BRANCH_NOT_CARRY: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(!cpu.is_flag_set(Flag::C)));

macro_rules! instruction {
    (fetch8, $($step:expr),*) => {
        {
            let steps: Vec<InstructionStep> = vec![
                __FETCH_OPERAND8,
                $(
                    $step,
                )*
            ];

            Instruction {
                index: 0,
                steps
            }
        }
    };

    (fetch16, $($step:expr),*) => {
        {
            let steps: Vec<InstructionStep> = vec![
                __FETCH_OPERAND8,
                __FETCH_OPERAND16,
                $(
                    $step,
                )*
            ];

            Instruction {
                index: 0,
                steps
            }
        }
    };

    ($($step:expr),*) => {
        {
            let steps: Vec<InstructionStep> = vec![
                $(
                    $step,
                )*
            ];

            Instruction {
                index: 0,
                steps
            }
        }
    };
}

macro_rules! ld_r16_u16 {
    (sp) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.operand8 = cpu.fetch(bus);
                // TODO: below line looks wrong, why are we shifting cpu.sp 8 bits left?
                cpu.sp = ((cpu.sp << 8) & 0xFF00) | cpu.operand8 as u16;
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let higher_bits = cpu.fetch(bus);
                cpu.sp = (higher_bits as u16) << 8 | cpu.operand8 as u16;
                InstructionState::Finished
            })
        }
    };

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
    (hlplus) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(cpu.hl.into(), cpu.af.hi);
                cpu.hl += 1;
                InstructionState::Finished
            })
        }
    };

    (hlminus) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(cpu.hl.into(), cpu.af.hi);
                cpu.hl -= 1;
                InstructionState::Finished
            })
        }
    };

    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(cpu.$reg.into(), cpu.af.hi);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! inc_r16 {
    (sp) => {
        instruction! {
            InstructionStep::Standard(|cpu, _| {
                cpu.sp = cpu.sp.wrapping_add(1);
                InstructionState::Finished
            })
        }
    };

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
    (sp) => {
        instruction! {
            InstructionStep::Standard(|cpu, _| {
                cpu.sp = cpu.sp.wrapping_sub(1);
                InstructionState::Finished
            })
        }
    };

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
                cpu.set_flag_if_cond_else_clear((cpu.$reg.$bit & 0x0F) == 0x00, Flag::H);
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
            FETCH_OP8_EXECNEXTINSTANT,
            InstructionStep::Instant(|cpu, _| {
                cpu.$reg.$bit = cpu.operand8;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! ld_a_mem {
    (hlplus) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.af.hi = bus.read_u8(cpu.hl.into());
                cpu.hl += 1;
                InstructionState::Finished
            })
        }
    };

    (hlminus) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.af.hi = bus.read_u8(cpu.hl.into());
                cpu.hl -= 1;
                InstructionState::Finished
            })
        }
    };

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

macro_rules! __define_branching_op_macro {
    ($op:ident, $suffix:ident) => {
        paste! {
            macro_rules! [<$op $suffix>] {
                () => {
                    instruction! {
                        fetch8,
                        [<__ $op>]!()
                    }
                };

                ($cond:tt) => {
                    instruction! {
                        FETCH_OP8_EXECNEXTINSTANT,
                        branch_condition!($cond),
                        [<__ $op>]!()
                    }
                };
            }
        }
    };
}

macro_rules! __jr {
    () => {
        InstructionStep::Standard(|cpu, _| {
            let jmp_amount = cpu.operand8 as i8;
            if jmp_amount < 0 {
                cpu.pc = cpu.pc.wrapping_sub(jmp_amount.abs() as u16);
            } else {
                cpu.pc = cpu.pc.wrapping_add(jmp_amount as u16);
            }

            InstructionState::Finished
        })
    };
}

__define_branching_op_macro!(jr, _i8);

macro_rules! ret_cc {
    ($cc:tt) => {
        instruction! {
            BLANK_PROGRESS_EXEC_NEXT_INSTANT,
            branch_condition!($cc),
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp16 = cpu.pop_u16_from_stack(bus);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.temp16;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! enable_master {
    (i $bus:ident) => {
        $bus.interrupts.enable_master();
    };
}

macro_rules! ret {
    ($($i:tt)?) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp16 = cpu.pop_u16_from_stack(bus);
                InstructionState::InProgress
            }),
            #[allow(unused)]
            InstructionStep::Standard(|cpu, bus| {
                cpu.pc = cpu.temp16;
                $(
                    enable_master!($i bus);
                )?
                InstructionState::Finished
            })
        }
    };
}

macro_rules! jp_u16 {
    () => {
        instruction! {
            fetch16,
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! jp_cc_u16 {
    ($cc:tt) => {
        instruction! {
            __FETCH_OPERAND8,
            InstructionStep::Standard(|cpu, bus| {
                let hi = cpu.fetch(bus);
                cpu.operand16 = (hi as u16) << 8 | cpu.operand8 as u16;
                InstructionState::ExecNextInstantly
            }),
            branch_condition!($cc),
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! call_cc_u16 {
    ($cc:tt) => {
        instruction! {
            __FETCH_OPERAND8,
            InstructionStep::Standard(|cpu, bus| {
                let hi = cpu.fetch(bus);
                cpu.operand16 = (hi as u16) << 8 | cpu.operand8 as u16;
                InstructionState::ExecNextInstantly
            }),
            branch_condition!($cc),
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, cpu.pc as u8);
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! call_u16 {
    () => {
        instruction! {
            fetch16,
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, cpu.pc as u8);
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! ld_r8_r8 {
    ($dest_reg:ident,$dest_bit:ident <= HL) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.$dest_reg.$dest_bit = bus.read_u8(cpu.hl.into());
                InstructionState::Finished
            })
        }
    };

    ($dest_reg:ident,$dest_bit:ident <= $src_reg:ident,$src_bit:ident) => {
        instruction! {
            #[allow(clippy::self_assignment)]
            InstructionStep::Instant(|cpu, _| {
                cpu.$dest_reg.$dest_bit = cpu.$src_reg.$src_bit;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! ld_memhl_r8 {
    (HL <= $reg:ident,$bit:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(cpu.hl.into(), cpu.$reg.$bit);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! r8 {
    ($cpu:ident, a) => {
        $cpu.af.hi
    };
    ($cpu:ident, b) => {
        $cpu.bc.hi
    };
    ($cpu:ident, c) => {
        $cpu.bc.lo
    };
    ($cpu:ident, d) => {
        $cpu.de.hi
    };
    ($cpu:ident, e) => {
        $cpu.de.lo
    };
    ($cpu:ident, h) => {
        $cpu.hl.hi
    };
    ($cpu:ident, l) => {
        $cpu.hl.lo
    };

    ($cpu:ident, u8) => {
        $cpu.operand8
    };

    // where we temporarily store the 8bit result
    // of fetching address HL in mem
    ($cpu:ident, hl) => {
        $cpu.temp8
    };
}

macro_rules! __read_hl {
    () => {
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, hl) = bus.read_u8(cpu.hl.into());
            InstructionState::ExecNextInstantly
        })
    };
}

macro_rules! __define_op_macro {
    ($op:ident) => {
        paste! {
            macro_rules! [<$op _a_r8>] {
                (hl) => {
                    instruction! {
                        __read_hl!(),
                        [<__ $op>]!(hl)
                    }
                };

                (u8) => {
                    instruction! {
                        FETCH_OP8_EXECNEXTINSTANT,
                        [<__ $op>]!(u8)
                    }
                };

                ($reg:tt) => {
                    instruction! {
                        [<__ $op>]!($reg)
                    }
                };
            }
        }
    };
}

macro_rules! __add {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            let (result, overflown) = cpu.af.hi.overflowing_add(r8!(cpu, $location));

            cpu.clear_flag(Flag::N);
            // TODO: bug in frosty? we did .is_none() in add(..) code so wat?
            cpu.set_flag_if_cond_else_clear(overflown, Flag::C);
            cpu.handle_z_flag(result);

            let is_half_carry = ((cpu.af.hi & 0x0F) + (r8!(cpu, $location) & 0x0F)) > 0x0F;
            cpu.set_flag_if_cond_else_clear(is_half_carry, Flag::H);

            cpu.af.hi = result;
            InstructionState::Finished
        })
    };
}

macro_rules! __adc {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            let carry: u8 = if cpu.is_flag_set(Flag::C) { 1 } else { 0 };

            let is_half_carry =
                ((r8!(cpu, a) & 0x0F) + (r8!(cpu, $location) & 0x0F) + carry) & 0x10 == 0x10;

            // TODO: use overflowing add?
            let is_carry = ((r8!(cpu, a) as u16) + (r8!(cpu, $location) as u16) + (carry as u16))
                & 0x100
                == 0x100;

            let result = r8!(cpu, a)
                .wrapping_add(r8!(cpu, $location))
                .wrapping_add(carry);

            cpu.handle_z_flag(result);
            cpu.set_flag_if_cond_else_clear(is_half_carry, Flag::H);
            cpu.set_flag_if_cond_else_clear(is_carry, Flag::C);

            cpu.clear_flag(Flag::N);
            r8!(cpu, a) = result;
            InstructionState::Finished
        })
    };
}

macro_rules! __sub {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            cpu.set_flag_if_cond_else_clear(r8!(cpu, $location) > r8!(cpu, a), Flag::C);
            cpu.set_flag_if_cond_else_clear(
                (r8!(cpu, $location) & 0x0F) > (r8!(cpu, a) & 0x0F),
                Flag::H,
            );

            r8!(cpu, a) = r8!(cpu, a).wrapping_sub(r8!(cpu, $location));

            cpu.handle_z_flag(r8!(cpu, a));
            cpu.set_flag(Flag::N);
            InstructionState::Finished
        })
    };
}

macro_rules! __sbc {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            let carry: u8 = if cpu.is_flag_set(Flag::C) { 1 } else { 0 };

            let is_half_carry = ((r8!(cpu, a) & 0x0F) as i16)
                - ((r8!(cpu, $location) & 0x0F) as i16)
                - (carry as i16)
                < 0;

            let is_full_carry =
                (r8!(cpu, a) as i16) - (r8!(cpu, $location) as i16) - (carry as i16) < 0;

            let result = r8!(cpu, a)
                .wrapping_sub(r8!(cpu, $location))
                .wrapping_sub(carry);

            cpu.handle_z_flag(result);
            cpu.set_flag_if_cond_else_clear(is_half_carry, Flag::H);
            cpu.set_flag_if_cond_else_clear(is_full_carry, Flag::C);

            cpu.set_flag(Flag::N);
            r8!(cpu, a) = result;
            InstructionState::Finished
        })
    };
}

macro_rules! __and {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            r8!(cpu, a) = r8!(cpu, a) & r8!(cpu, $location);

            cpu.handle_z_flag(r8!(cpu, a));
            cpu.clear_flag(Flag::C);
            cpu.clear_flag(Flag::N);
            cpu.set_flag(Flag::H);
            InstructionState::Finished
        })
    };
}

macro_rules! __xor {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            r8!(cpu, a) = r8!(cpu, a) ^ r8!(cpu, $location);

            cpu.handle_z_flag(r8!(cpu, a));
            cpu.clear_flag(Flag::C);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    };
}

macro_rules! __or {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            r8!(cpu, a) = r8!(cpu, a) | r8!(cpu, $location);

            cpu.handle_z_flag(r8!(cpu, a));
            cpu.clear_flag(Flag::C);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    };
}

macro_rules! __cp {
    ($location:ident) => {
        InstructionStep::Instant(|cpu, _| {
            cpu.set_flag_if_cond_else_clear(r8!(cpu, a) == r8!(cpu, $location), Flag::Z);
            cpu.set_flag_if_cond_else_clear(r8!(cpu, $location) > r8!(cpu, a), Flag::C);
            cpu.set_flag_if_cond_else_clear(
                (r8!(cpu, $location) & 0xF) > (r8!(cpu, a) & 0x0F),
                Flag::H,
            );
            cpu.set_flag(Flag::N);
            InstructionState::Finished
        })
    };
}

__define_op_macro!(add);
__define_op_macro!(adc);
__define_op_macro!(sub);
__define_op_macro!(sbc);
__define_op_macro!(and);
__define_op_macro!(xor);
__define_op_macro!(or);
__define_op_macro!(cp);

macro_rules! __pop_r16_af_edgecase {
    (af $cpu:ident) => {
        $cpu.af.lo &= 0xF0;
    };

    ($_:ident $cpu:ident) => {};
}

macro_rules! pop_r16 {
    ($reg:ident) => {
        instruction! {
            InstructionStep::Standard(|cpu, bus| {
                cpu.$reg.lo = cpu.pop_u8_from_stack(bus);
                __pop_r16_af_edgecase!($reg cpu);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.$reg.hi = cpu.pop_u8_from_stack(bus);
                __pop_r16_af_edgecase!($reg cpu);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! push_r16 {
    ($reg:ident) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                let val = cpu.$reg.hi;
                cpu.push_u8_to_stack(bus, val);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let val = cpu.$reg.lo;
                cpu.push_u8_to_stack(bus, val);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! rst_yy {
    ($addr:tt) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, cpu.pc as u8);
                // TODO: verify if this is correct
                cpu.pc = $addr as u16;
                InstructionState::Finished
            })
        }
    };
}

macro_rules! unused_opcode {
    ($opcode:tt) => {
        instruction!(InstructionStep::Instant(|_, _| unimplemented!(
            "Unused Opcode: {}",
            $opcode
        )))
    };
}

fn daa() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            // https://forums.nesdev.com/viewtopic.php?t=15944
            if cpu.is_flag_set(Flag::N) {
                if cpu.is_flag_set(Flag::C) {
                    r8!(cpu, a) = r8!(cpu, a).wrapping_sub(0x60);
                }

                if cpu.is_flag_set(Flag::H) {
                    r8!(cpu, a) = r8!(cpu, a).wrapping_sub(0x6);
                }
            } else {
                if cpu.is_flag_set(Flag::C) || r8!(cpu, a) > 0x99 {
                    r8!(cpu, a) = r8!(cpu, a).wrapping_add(0x60);
                    cpu.set_flag(Flag::C);
                }

                if cpu.is_flag_set(Flag::H) || (r8!(cpu, a) & 0xF) > 0x09 {
                    r8!(cpu, a) = r8!(cpu, a).wrapping_add(0x6);
                }
            }

            cpu.handle_z_flag(r8!(cpu, a));
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn cpl() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            r8!(cpu, a) = !r8!(cpu, a);
            cpu.set_flag(Flag::N);
            cpu.set_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn scf() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            cpu.set_flag(Flag::C);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn ccf() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            if cpu.is_flag_set(Flag::C) {
                cpu.clear_flag(Flag::C);
            } else {
                cpu.set_flag(Flag::C);
            }

            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn rlca() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            let carry = (r8!(cpu, a) & 0x80) >> 7;
            cpu.set_flag_if_cond_else_clear(carry != 0, Flag::C);

            r8!(cpu, a) = (r8!(cpu, a) << 1).wrapping_add(carry);

            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn rrca() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            let carry = r8!(cpu, a) & 0b00000001 > 0;
            r8!(cpu, a) = r8!(cpu, a) >> 1;
            if carry { r8!(cpu, a) = r8!(cpu, a) | 0b10000000; }

            cpu.set_flag_if_cond_else_clear(carry, Flag::C);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn rla() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            let is_carry_set = cpu.is_flag_set(Flag::C);
            cpu.set_flag_if_cond_else_clear(r8!(cpu, a) & 0x80 > 0, Flag::C);

            r8!(cpu, a) = r8!(cpu, a) << 1;
            if is_carry_set { r8!(cpu, a) += 1 };

            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn rra() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            let carry = if cpu.is_flag_set(Flag::C) {1 << 7} else {0};
            cpu.set_flag_if_cond_else_clear(r8!(cpu, a) & 0x01 != 0, Flag::C);

            r8!(cpu, a) = (r8!(cpu, a) >> 1).wrapping_add(carry);

            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

fn inc_hl() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, bus| {
            cpu.temp8 = bus.read_u8(cpu.hl.into());
            InstructionState::InProgress
        }),
        InstructionStep::Standard(|cpu, bus| {
            cpu.set_flag_if_cond_else_clear((cpu.temp8 & 0x0F) == 0x0F, Flag::H);
            cpu.temp8 = cpu.temp8.wrapping_add(1);
            cpu.handle_z_flag(cpu.temp8);
            cpu.clear_flag(Flag::N);

            bus.write_u8(cpu.hl.into(), cpu.temp8);
            InstructionState::Finished
        })
    }
}

fn dec_hl() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, bus| {
            cpu.temp8 = bus.read_u8(cpu.hl.into());
            InstructionState::InProgress
        }),
        InstructionStep::Standard(|cpu, bus| {
            cpu.set_flag_if_cond_else_clear((cpu.temp8 & 0x0F) == 0, Flag::H);
            cpu.temp8 = cpu.temp8.wrapping_sub(1);
            cpu.handle_z_flag(cpu.temp8);
            cpu.set_flag(Flag::N);

            bus.write_u8(cpu.hl.into(), cpu.temp8);
            InstructionState::Finished
        })
    }
}

fn ld_hlmem_u8() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.hl.into(), cpu.operand8);
            InstructionState::Finished
        })
    }
}

fn ld_ff00_u8_a() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(0xFF00 + (cpu.operand8 as u16), r8!(cpu, a));
            InstructionState::Finished
        })
    }
}

fn ld_a_ff00_u8() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, a) = bus.read_u8(0xFF00 + (cpu.operand8 as u16));
            InstructionState::Finished
        })
    }
}

fn ld_ff00_c_a() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(0xFF00 + (r8!(cpu, c) as u16), r8!(cpu, a));
            InstructionState::Finished
        })
    }
}

fn ld_a_ff00_c() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, a) = bus.read_u8(0xFF00 + (r8!(cpu, c) as u16));
            InstructionState::Finished
        })
    }
}

fn add_sp_i8() -> Instruction {
    instruction! {
        fetch8,
        BLANK_PROGRESS,
        InstructionStep::Standard(|cpu, _| {
            let arg = cpu.operand8 as i8 as i16 as u16;

            let half_carry = (cpu.sp & 0x000F) + (arg & 0x000F) > 0x000F;
            let carry = (cpu.sp & 0x00FF) + (arg & 0x00FF) > 0x00FF;

            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::N);
            cpu.set_flag_if_cond_else_clear(carry, Flag::C);
            cpu.set_flag_if_cond_else_clear(half_carry, Flag::H);

            cpu.sp = cpu.sp.wrapping_add(arg);

            InstructionState::Finished
        })
    }
}

fn ld_hl_sp_i8() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, _| {
            let arg = cpu.operand8 as i8 as i16 as u16;

            let half_carry = (cpu.sp & 0x000F) + (arg & 0x000F) > 0x000F;
            let carry = (cpu.sp & 0x00FF) + (arg & 0x00FF) > 0x00FF;

            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::N);
            cpu.set_flag_if_cond_else_clear(carry, Flag::C);
            cpu.set_flag_if_cond_else_clear(half_carry, Flag::H);

            let result = cpu.sp.wrapping_add(arg);
            cpu.hl = result.into();
            InstructionState::Finished
        })
    }
}

fn jp_hl() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            cpu.pc = cpu.hl.into();
            InstructionState::Finished
        })
    }
}

fn stop() -> Instruction {
    instruction! {
        InstructionStep::Instant(|_, _| {
            // TODO: for now lets just do nothing!
            // cpu.stopped = true;
            InstructionState::Finished
        })
    }
}

fn ld_sp_hl() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, _| {
            cpu.sp = cpu.hl.into();
            InstructionState::Finished
        })
    }
}

fn ld_mem_u16_a() -> Instruction {
    instruction! {
        fetch16,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.operand16, r8!(cpu, a));
            InstructionState::Finished
        })
    }
}

fn ld_a_mem_u16() -> Instruction {
    instruction! {
        fetch16,
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, a) = bus.read_u8(cpu.operand16);
            InstructionState::Finished
        })
    }
}

fn di() -> Instruction {
    instruction! {
        InstructionStep::Instant(|_, bus| {
            bus.interrupts.disable_master();
            InstructionState::Finished
        })
    }
}

fn ei() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, bus| {
            if !bus.interrupts.is_master_enabled() && !cpu.ei_delay {
                cpu.ei_delay = true;
                cpu.ei_delay_cycles = 4;
            }

            InstructionState::Finished
        })
    }
}

fn halt() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, bus| {
            if bus.interrupts.is_master_enabled() {
                // IME set
                cpu.halted = true;
            }
            else if bus.interrupts.enable & bus.interrupts.flags & 0x1F != 0 {
                // IME not set, interupt pending
                // continue execution, but the next byte is read twice
                // or in other words, after the next byte is read the pc gets
                // decremented back to what it was
                cpu.halt_bug_triggered = true;
            }
            else {
                // IME not set, no interupt pending
                cpu.halted = true;
                cpu.halted_waiting_for_interrupt_pending = true;
                bus.interrupts.waiting_for_halt_if = true;
            }

            InstructionState::Finished
        })
    }
}

fn interrupt_service_routine() -> Instruction {
    instruction! {
        BLANK_PROGRESS,
        BLANK_PROGRESS,
        InstructionStep::Standard(|cpu, bus| {
            cpu.temp8 = bus.interrupts.enable;
            cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
            InstructionState::InProgress
        }),
        InstructionStep::Standard(|cpu, bus| {
            let ir_flags = bus.interrupts.flags;
            cpu.push_u8_to_stack(bus, (cpu.pc & 0x00FF) as u8);
            cpu.temp16 = ir_flags as u16;
            InstructionState::InProgress
        }),
        InstructionStep::Standard(|cpu, bus| {
            let interrupt_state = bus.interrupts.get_interupt_state_latched(cpu.temp8, cpu.temp16 as u8);
            let vector = match interrupt_state {
                Some(flag) => {
                    bus.interrupts.clear_interupt(flag);
                    flag.vector()
                },
                None => 0
            };

            bus.interrupts.disable_master();
            cpu.pc = vector;

            InstructionState::Finished
        })
    }
}

macro_rules! bit {
    ($bit:expr, hl) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                let arg = bus.read_u8(cpu.hl.into());

                cpu.set_flag_if_cond_else_clear(arg & (1 << $bit) == 0, Flag::Z);
                cpu.clear_flag(Flag::N);
                cpu.set_flag(Flag::H);

                InstructionState::Finished
            })
        }
    };

    ($bit:expr, $reg:tt) => {
        instruction! {
            BLANK_PROGRESS_EXEC_NEXT_INSTANT,
            InstructionStep::Instant(|cpu, _| {
                cpu.set_flag_if_cond_else_clear(r8!(cpu, $reg) & (1 << $bit) == 0, Flag::Z);
                cpu.clear_flag(Flag::N);
                cpu.set_flag(Flag::H);

                InstructionState::Finished
            })
        }
    };
}

macro_rules! res {
    ($bit:expr, hl) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp8 = bus.read_u8(cpu.hl.into());
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let result = cpu.temp8 & !(1 << $bit);
                bus.write_u8(cpu.hl.into(), result);
                InstructionState::Finished
            })
        }
    };

    ($bit:expr, $reg:tt) => {
        instruction! {
            BLANK_PROGRESS_EXEC_NEXT_INSTANT,
            InstructionStep::Instant(|cpu, _| {
                r8!(cpu, $reg) = r8!(cpu, $reg) & !(1 << $bit);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! set {
    ($bit:expr, hl) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp8 = bus.read_u8(cpu.hl.into());
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let result = cpu.temp8 | (1 << $bit);
                bus.write_u8(cpu.hl.into(), result);
                InstructionState::Finished
            })
        }
    };

    ($bit:expr, $reg:tt) => {
        instruction! {
            BLANK_PROGRESS_EXEC_NEXT_INSTANT,
            InstructionStep::Instant(|cpu, _| {
                r8!(cpu, $reg) = r8!(cpu, $reg) | (1 << $bit);
                InstructionState::Finished
            })
        }
    };
}

macro_rules! cb_op_instr {
    ($func:ident, $reg:tt) => {
        instruction! {
            BLANK_PROGRESS_EXEC_NEXT_INSTANT,
            InstructionStep::Instant(|cpu, _| {
                r8!(cpu, $reg) = cpu.$func(r8!(cpu, $reg));
                InstructionState::Finished
            })
        }
    };
}

macro_rules! cb_op_hl_instr {
    ($func:ident, hl) => {
        instruction! {
            BLANK_PROGRESS,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp8 = bus.read_u8(cpu.hl.into());
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let result = cpu.$func(cpu.temp8);
                bus.write_u8(cpu.hl.into(), result);
                InstructionState::Finished
            })
        }
    };
}

impl Cpu {
    fn rlc(&mut self, val: u8) -> u8 {
        self.set_flag_if_cond_else_clear((val & 0x80) != 0, Flag::C);

        let carry = (val & 0x80) >> 7;
        let result = (val << 1).wrapping_add(carry);

        self.handle_z_flag(val);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn rrc(&mut self, val: u8) -> u8 {
        let carry = val & 0x01;
        let mut result = val >> 1;

        if carry != 0 {
            self.set_flag(Flag::C);
            result |= 0x80;
        } else {
            self.clear_flag(Flag::C);
        }

        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn rl(&mut self, val: u8) -> u8 {
        let carry = if self.is_flag_set(Flag::C) { 1 } else { 0 };
        let result = (val << 1).wrapping_add(carry);

        self.set_flag_if_cond_else_clear(val & 0x80 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn rr(&mut self, val: u8) -> u8 {
        let mut result = val >> 1;
        if self.is_flag_set(Flag::C) {
            result |= 0x80;
        }

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn sla(&mut self, val: u8) -> u8 {
        let result = val << 1;

        self.set_flag_if_cond_else_clear(val & 0x80 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn sra(&mut self, val: u8) -> u8 {
        let result = (val & 0x80) | (val >> 1);

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn swap(&mut self, val: u8) -> u8 {
        let result = ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);

        self.handle_z_flag(result);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    fn srl(&mut self, val: u8) -> u8 {
        let result = val >> 1;

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

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

    fn pop_u16_from_stack(&mut self, bus: &Bus) -> u16 {
        let val = bus.read_u16(self.sp);
        self.sp = self.sp.wrapping_add(2);
        val
    }

    fn pop_u8_from_stack(&mut self, bus: &Bus) -> u8 {
        let val = bus.read_u8(self.sp);
        self.sp = self.sp.wrapping_add(1);
        val
    }

    fn push_u8_to_stack(&mut self, bus: &mut Bus, val: u8) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write_u8(self.sp, val);
    }
}

pub struct InstructionCache {
    interrupt_service_routine: Instruction,
    instructions: [Instruction; 256],
    cb_instructions: [Instruction; 256],
}

impl InstructionCache {
    pub fn new() -> Self {
        Self {
            interrupt_service_routine: interrupt_service_routine(),
            instructions: Self::gen_instructions(),
            cb_instructions: Self::gen_cb_instructions(),
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
            0x07 => rlca(),
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
            0x0F => rrca(),

            0x10 => stop(),
            0x11 => ld_r16_u16!(de),
            0x12 => ld_mem_a!(de),
            0x13 => inc_r16!(de),
            0x14 => inc_r8!(de, hi),
            0x15 => dec_r8!(de, hi),
            0x16 => ld_r8_u8!(de, hi),
            0x17 => rla(),
            0x18 => jr_i8!(),
            0x19 => add_hl_r16!(de),
            0x1A => ld_a_mem!(de),
            0x1B => dec_r16!(de),
            0x1C => inc_r8!(de, lo),
            0x1D => dec_r8!(de, lo),
            0x1E => ld_r8_u8!(de, lo),
            0x1F => rra(),

            0x20 => jr_i8!(NZ),
            0x21 => ld_r16_u16!(hl),
            0x22 => ld_mem_a!(hlplus),
            0x23 => inc_r16!(hl),
            0x24 => inc_r8!(hl, hi),
            0x25 => dec_r8!(hl, hi),
            0x26 => ld_r8_u8!(hl, hi),
            0x27 => daa(),
            0x28 => jr_i8!(Z),
            0x29 => add_hl_r16!(hl),
            0x2A => ld_a_mem!(hlplus),
            0x2B => dec_r16!(hl),
            0x2C => inc_r8!(hl, lo),
            0x2D => dec_r8!(hl, lo),
            0x2E => ld_r8_u8!(hl, lo),
            0x2F => cpl(),

            0x30 => jr_i8!(NC),
            0x31 => ld_r16_u16!(sp),
            0x32 => ld_mem_a!(hlminus),
            0x33 => inc_r16!(sp),
            0x34 => inc_hl(),
            0x35 => dec_hl(),
            0x36 => ld_hlmem_u8(),
            0x37 => scf(),
            0x38 => jr_i8!(C),
            0x39 => add_hl_r16!(sp),
            0x3A => ld_a_mem!(hlminus),
            0x3B => dec_r16!(sp),
            0x3C => inc_r8!(af, hi),
            0x3D => dec_r8!(af, hi),
            0x3E => ld_r8_u8!(af, hi),
            0x3F => ccf(),

            // ld b, r8
            0x40 => ld_r8_r8!(bc, hi <= bc, hi),
            0x41 => ld_r8_r8!(bc, hi <= bc, lo),
            0x42 => ld_r8_r8!(bc, hi <= de, hi),
            0x43 => ld_r8_r8!(bc, hi <= de, lo),
            0x44 => ld_r8_r8!(bc, hi <= hl, hi),
            0x45 => ld_r8_r8!(bc, hi <= hl, lo),
            0x46 => ld_r8_r8!(bc, hi <= HL),
            0x47 => ld_r8_r8!(bc, hi <= af, hi),

            // ld c, r8
            0x48 => ld_r8_r8!(bc, lo <= bc, hi),
            0x49 => ld_r8_r8!(bc, lo <= bc, lo),
            0x4A => ld_r8_r8!(bc, lo <= de, hi),
            0x4B => ld_r8_r8!(bc, lo <= de, lo),
            0x4C => ld_r8_r8!(bc, lo <= hl, hi),
            0x4D => ld_r8_r8!(bc, lo <= hl, lo),
            0x4E => ld_r8_r8!(bc, lo <= HL),
            0x4F => ld_r8_r8!(bc, lo <= af, hi),

            // ld d, r8
            0x50 => ld_r8_r8!(de, hi <= bc, hi),
            0x51 => ld_r8_r8!(de, hi <= bc, lo),
            0x52 => ld_r8_r8!(de, hi <= de, hi),
            0x53 => ld_r8_r8!(de, hi <= de, lo),
            0x54 => ld_r8_r8!(de, hi <= hl, hi),
            0x55 => ld_r8_r8!(de, hi <= hl, lo),
            0x56 => ld_r8_r8!(de, hi <= HL),
            0x57 => ld_r8_r8!(de, hi <= af, hi),

            // ld e, r8
            0x58 => ld_r8_r8!(de, lo <= bc, hi),
            0x59 => ld_r8_r8!(de, lo <= bc, lo),
            0x5A => ld_r8_r8!(de, lo <= de, hi),
            0x5B => ld_r8_r8!(de, lo <= de, lo),
            0x5C => ld_r8_r8!(de, lo <= hl, hi),
            0x5D => ld_r8_r8!(de, lo <= hl, lo),
            0x5E => ld_r8_r8!(de, lo <= HL),
            0x5F => ld_r8_r8!(de, lo <= af, hi),

            // ld h, r8
            0x60 => ld_r8_r8!(hl, hi <= bc, hi),
            0x61 => ld_r8_r8!(hl, hi <= bc, lo),
            0x62 => ld_r8_r8!(hl, hi <= de, hi),
            0x63 => ld_r8_r8!(hl, hi <= de, lo),
            0x64 => ld_r8_r8!(hl, hi <= hl, hi),
            0x65 => ld_r8_r8!(hl, hi <= hl, lo),
            0x66 => ld_r8_r8!(hl, hi <= HL),
            0x67 => ld_r8_r8!(hl, hi <= af, hi),

            // ld l, r8
            0x68 => ld_r8_r8!(hl, lo <= bc, hi),
            0x69 => ld_r8_r8!(hl, lo <= bc, lo),
            0x6A => ld_r8_r8!(hl, lo <= de, hi),
            0x6B => ld_r8_r8!(hl, lo <= de, lo),
            0x6C => ld_r8_r8!(hl, lo <= hl, hi),
            0x6D => ld_r8_r8!(hl, lo <= hl, lo),
            0x6E => ld_r8_r8!(hl, lo <= HL),
            0x6F => ld_r8_r8!(hl, lo <= af, hi),

            // ld (HL), r8
            0x70 => ld_memhl_r8!(HL <= bc, hi),
            0x71 => ld_memhl_r8!(HL <= bc, lo),
            0x72 => ld_memhl_r8!(HL <= de, hi),
            0x73 => ld_memhl_r8!(HL <= de, lo),
            0x74 => ld_memhl_r8!(HL <= hl, hi),
            0x75 => ld_memhl_r8!(HL <= hl, lo),
            0x76 => halt(),
            0x77 => ld_memhl_r8!(HL <= af, hi),

            // ld a, r8
            0x78 => ld_r8_r8!(af, hi <= bc, hi),
            0x79 => ld_r8_r8!(af, hi <= bc, lo),
            0x7A => ld_r8_r8!(af, hi <= de, hi),
            0x7B => ld_r8_r8!(af, hi <= de, lo),
            0x7C => ld_r8_r8!(af, hi <= hl, hi),
            0x7D => ld_r8_r8!(af, hi <= hl, lo),
            0x7E => ld_r8_r8!(af, hi <= HL),
            0x7F => ld_r8_r8!(af, hi <= af, hi),

            // add a, r8
            0x80 => add_a_r8!(b),
            0x81 => add_a_r8!(c),
            0x82 => add_a_r8!(d),
            0x83 => add_a_r8!(e),
            0x84 => add_a_r8!(h),
            0x85 => add_a_r8!(l),
            0x86 => add_a_r8!(hl),
            0x87 => add_a_r8!(a),

            // adc a, r8
            0x88 => adc_a_r8!(b),
            0x89 => adc_a_r8!(c),
            0x8A => adc_a_r8!(d),
            0x8B => adc_a_r8!(e),
            0x8C => adc_a_r8!(h),
            0x8D => adc_a_r8!(l),
            0x8E => adc_a_r8!(hl),
            0x8F => adc_a_r8!(a),

            // sub a, r8
            0x90 => sub_a_r8!(b),
            0x91 => sub_a_r8!(c),
            0x92 => sub_a_r8!(d),
            0x93 => sub_a_r8!(e),
            0x94 => sub_a_r8!(h),
            0x95 => sub_a_r8!(l),
            0x96 => sub_a_r8!(hl),
            0x97 => sub_a_r8!(a),

            // sbc a, r8
            0x98 => sbc_a_r8!(b),
            0x99 => sbc_a_r8!(c),
            0x9A => sbc_a_r8!(d),
            0x9B => sbc_a_r8!(e),
            0x9C => sbc_a_r8!(h),
            0x9D => sbc_a_r8!(l),
            0x9E => sbc_a_r8!(hl),
            0x9F => sbc_a_r8!(a),

            // and a, r8
            0xA0 => and_a_r8!(b),
            0xA1 => and_a_r8!(c),
            0xA2 => and_a_r8!(d),
            0xA3 => and_a_r8!(e),
            0xA4 => and_a_r8!(h),
            0xA5 => and_a_r8!(l),
            0xA6 => and_a_r8!(hl),
            0xA7 => and_a_r8!(a),

            // xor a, r8
            0xA8 => xor_a_r8!(b),
            0xA9 => xor_a_r8!(c),
            0xAA => xor_a_r8!(d),
            0xAB => xor_a_r8!(e),
            0xAC => xor_a_r8!(h),
            0xAD => xor_a_r8!(l),
            0xAE => xor_a_r8!(hl),
            0xAF => xor_a_r8!(a),

            // or a, r8
            0xB0 => or_a_r8!(b),
            0xB1 => or_a_r8!(c),
            0xB2 => or_a_r8!(d),
            0xB3 => or_a_r8!(e),
            0xB4 => or_a_r8!(h),
            0xB5 => or_a_r8!(l),
            0xB6 => or_a_r8!(hl),
            0xB7 => or_a_r8!(a),

            // cp a, r8
            0xB8 => cp_a_r8!(b),
            0xB9 => cp_a_r8!(c),
            0xBA => cp_a_r8!(d),
            0xBB => cp_a_r8!(e),
            0xBC => cp_a_r8!(h),
            0xBD => cp_a_r8!(l),
            0xBE => cp_a_r8!(hl),
            0xBF => cp_a_r8!(a),

            0xC0 => ret_cc!(NZ),
            0xC1 => pop_r16!(bc),
            0xC2 => jp_cc_u16!(NZ),
            0xC3 => jp_u16!(),
            0xC4 => call_cc_u16!(NZ),
            0xC5 => push_r16!(bc),
            0xC6 => add_a_r8!(u8),
            0xC7 => rst_yy!(0),
            0xC8 => ret_cc!(Z),
            0xC9 => ret!(),
            0xCA => jp_cc_u16!(Z),
            0xCB => instruction!(InstructionStep::Instant(|_, _| unimplemented!("CB PREFIX"))),
            0xCC => call_cc_u16!(Z),
            0xCD => call_u16!(),
            0xCE => adc_a_r8!(u8),
            0xCF => rst_yy!(0x08),

            0xD0 => ret_cc!(NC),
            0xD1 => pop_r16!(de),
            0xD2 => jp_cc_u16!(NC),
            0xD3 => unused_opcode!("0xD3"),
            0xD4 => call_cc_u16!(NC),
            0xD5 => push_r16!(de),
            0xD6 => sub_a_r8!(u8),
            0xD7 => rst_yy!(0x10),
            0xD8 => ret_cc!(C),
            0xD9 => ret!(i),
            0xDA => jp_cc_u16!(C),
            0xDB => unused_opcode!("0xDB"),
            0xDC => call_cc_u16!(C),
            0xDD => unused_opcode!("0xDD"),
            0xDE => sbc_a_r8!(u8),
            0xDF => rst_yy!(0x18),

            0xE0 => ld_ff00_u8_a(),
            0xE1 => pop_r16!(hl),
            0xE2 => ld_ff00_c_a(),
            0xE3 => unused_opcode!("0xE3"),
            0xE4 => unused_opcode!("0xE4"),
            0xE5 => push_r16!(hl),
            0xE6 => and_a_r8!(u8),
            0xE7 => rst_yy!(0x20),
            0xE8 => add_sp_i8(),
            0xE9 => jp_hl(),
            0xEA => ld_mem_u16_a(),
            0xEB => unused_opcode!("0xEB"),
            0xEC => unused_opcode!("0xEC"),
            0xED => unused_opcode!("0xED"),
            0xEE => xor_a_r8!(u8),
            0xEF => rst_yy!(0x28),

            0xF0 => ld_a_ff00_u8(),
            0xF1 => pop_r16!(af),
            0xF2 => ld_a_ff00_c(),
            0xF3 => di(),
            0xF4 => unused_opcode!("0xF4"),
            0xF5 => push_r16!(af),
            0xF6 => or_a_r8!(u8),
            0xF7 => rst_yy!(0x30),
            0xF8 => ld_hl_sp_i8(),
            0xF9 => ld_sp_hl(),
            0xFA => ld_a_mem_u16(),
            0xFB => ei(),
            0xFC => unused_opcode!("0xFC"),
            0xFD => unused_opcode!("0xFD"),
            0xFE => cp_a_r8!(u8),
            0xFF => rst_yy!(0x38),
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
        let helper = |opcode: u8| match opcode {
            0x00 => cb_op_instr!(rlc, b),
            0x01 => cb_op_instr!(rlc, c),
            0x02 => cb_op_instr!(rlc, d),
            0x03 => cb_op_instr!(rlc, e),
            0x04 => cb_op_instr!(rlc, h),
            0x05 => cb_op_instr!(rlc, l),
            0x06 => cb_op_hl_instr!(rlc, hl),
            0x07 => cb_op_instr!(rlc, a),

            0x08 => cb_op_instr!(rrc, b),
            0x09 => cb_op_instr!(rrc, c),
            0x0A => cb_op_instr!(rrc, d),
            0x0B => cb_op_instr!(rrc, e),
            0x0C => cb_op_instr!(rrc, h),
            0x0D => cb_op_instr!(rrc, l),
            0x0E => cb_op_hl_instr!(rrc, hl),
            0x0F => cb_op_instr!(rrc, a),

            0x10 => cb_op_instr!(rl, b),
            0x11 => cb_op_instr!(rl, c),
            0x12 => cb_op_instr!(rl, d),
            0x13 => cb_op_instr!(rl, e),
            0x14 => cb_op_instr!(rl, h),
            0x15 => cb_op_instr!(rl, l),
            0x16 => cb_op_hl_instr!(rl, hl),
            0x17 => cb_op_instr!(rl, a),

            0x18 => cb_op_instr!(rr, b),
            0x19 => cb_op_instr!(rr, c),
            0x1A => cb_op_instr!(rr, d),
            0x1B => cb_op_instr!(rr, e),
            0x1C => cb_op_instr!(rr, h),
            0x1D => cb_op_instr!(rr, l),
            0x1E => cb_op_hl_instr!(rr, hl),
            0x1F => cb_op_instr!(rr, a),

            0x20 => cb_op_instr!(sla, b),
            0x21 => cb_op_instr!(sla, c),
            0x22 => cb_op_instr!(sla, d),
            0x23 => cb_op_instr!(sla, e),
            0x24 => cb_op_instr!(sla, h),
            0x25 => cb_op_instr!(sla, l),
            0x26 => cb_op_hl_instr!(sla, hl),
            0x27 => cb_op_instr!(sla, a),

            0x28 => cb_op_instr!(sra, b),
            0x29 => cb_op_instr!(sra, c),
            0x2A => cb_op_instr!(sra, d),
            0x2B => cb_op_instr!(sra, e),
            0x2C => cb_op_instr!(sra, h),
            0x2D => cb_op_instr!(sra, l),
            0x2E => cb_op_hl_instr!(sra, hl),
            0x2F => cb_op_instr!(sra, a),

            0x30 => cb_op_instr!(swap, b),
            0x31 => cb_op_instr!(swap, c),
            0x32 => cb_op_instr!(swap, d),
            0x33 => cb_op_instr!(swap, e),
            0x34 => cb_op_instr!(swap, h),
            0x35 => cb_op_instr!(swap, l),
            0x36 => cb_op_hl_instr!(swap, hl),
            0x37 => cb_op_instr!(swap, a),

            0x38 => cb_op_instr!(srl, b),
            0x39 => cb_op_instr!(srl, c),
            0x3A => cb_op_instr!(srl, d),
            0x3B => cb_op_instr!(srl, e),
            0x3C => cb_op_instr!(srl, h),
            0x3D => cb_op_instr!(srl, l),
            0x3E => cb_op_hl_instr!(srl, hl),
            0x3F => cb_op_instr!(srl, a),

            0x40 => bit!(0, b),
            0x41 => bit!(0, c),
            0x42 => bit!(0, d),
            0x43 => bit!(0, e),
            0x44 => bit!(0, h),
            0x45 => bit!(0, l),
            0x46 => bit!(0, hl),
            0x47 => bit!(0, a),

            0x48 => bit!(1, b),
            0x49 => bit!(1, c),
            0x4A => bit!(1, d),
            0x4B => bit!(1, e),
            0x4C => bit!(1, h),
            0x4D => bit!(1, l),
            0x4E => bit!(1, hl),
            0x4F => bit!(1, a),

            0x50 => bit!(2, b),
            0x51 => bit!(2, c),
            0x52 => bit!(2, d),
            0x53 => bit!(2, e),
            0x54 => bit!(2, h),
            0x55 => bit!(2, l),
            0x56 => bit!(2, hl),
            0x57 => bit!(2, a),

            0x58 => bit!(3, b),
            0x59 => bit!(3, c),
            0x5A => bit!(3, d),
            0x5B => bit!(3, e),
            0x5C => bit!(3, h),
            0x5D => bit!(3, l),
            0x5E => bit!(3, hl),
            0x5F => bit!(3, a),

            0x60 => bit!(4, b),
            0x61 => bit!(4, c),
            0x62 => bit!(4, d),
            0x63 => bit!(4, e),
            0x64 => bit!(4, h),
            0x65 => bit!(4, l),
            0x66 => bit!(4, hl),
            0x67 => bit!(4, a),

            0x68 => bit!(5, b),
            0x69 => bit!(5, c),
            0x6A => bit!(5, d),
            0x6B => bit!(5, e),
            0x6C => bit!(5, h),
            0x6D => bit!(5, l),
            0x6E => bit!(5, hl),
            0x6F => bit!(5, a),

            0x70 => bit!(6, b),
            0x71 => bit!(6, c),
            0x72 => bit!(6, d),
            0x73 => bit!(6, e),
            0x74 => bit!(6, h),
            0x75 => bit!(6, l),
            0x76 => bit!(6, hl),
            0x77 => bit!(6, a),

            0x78 => bit!(7, b),
            0x79 => bit!(7, c),
            0x7A => bit!(7, d),
            0x7B => bit!(7, e),
            0x7C => bit!(7, h),
            0x7D => bit!(7, l),
            0x7E => bit!(7, hl),
            0x7F => bit!(7, a),

            0x80 => res!(0, b),
            0x81 => res!(0, c),
            0x82 => res!(0, d),
            0x83 => res!(0, e),
            0x84 => res!(0, h),
            0x85 => res!(0, l),
            0x86 => res!(0, hl),
            0x87 => res!(0, a),

            0x88 => res!(1, b),
            0x89 => res!(1, c),
            0x8A => res!(1, d),
            0x8B => res!(1, e),
            0x8C => res!(1, h),
            0x8D => res!(1, l),
            0x8E => res!(1, hl),
            0x8F => res!(1, a),

            0x90 => res!(2, b),
            0x91 => res!(2, c),
            0x92 => res!(2, d),
            0x93 => res!(2, e),
            0x94 => res!(2, h),
            0x95 => res!(2, l),
            0x96 => res!(2, hl),
            0x97 => res!(2, a),

            0x98 => res!(3, b),
            0x99 => res!(3, c),
            0x9A => res!(3, d),
            0x9B => res!(3, e),
            0x9C => res!(3, h),
            0x9D => res!(3, l),
            0x9E => res!(3, hl),
            0x9F => res!(3, a),

            0xA0 => res!(4, b),
            0xA1 => res!(4, c),
            0xA2 => res!(4, d),
            0xA3 => res!(4, e),
            0xA4 => res!(4, h),
            0xA5 => res!(4, l),
            0xA6 => res!(4, hl),
            0xA7 => res!(4, a),

            0xA8 => res!(5, b),
            0xA9 => res!(5, c),
            0xAA => res!(5, d),
            0xAB => res!(5, e),
            0xAC => res!(5, h),
            0xAD => res!(5, l),
            0xAE => res!(5, hl),
            0xAF => res!(5, a),

            0xB0 => res!(6, b),
            0xB1 => res!(6, c),
            0xB2 => res!(6, d),
            0xB3 => res!(6, e),
            0xB4 => res!(6, h),
            0xB5 => res!(6, l),
            0xB6 => res!(6, hl),
            0xB7 => res!(6, a),

            0xB8 => res!(7, b),
            0xB9 => res!(7, c),
            0xBA => res!(7, d),
            0xBB => res!(7, e),
            0xBC => res!(7, h),
            0xBD => res!(7, l),
            0xBE => res!(7, hl),
            0xBF => res!(7, a),

            0xC0 => set!(0, b),
            0xC1 => set!(0, c),
            0xC2 => set!(0, d),
            0xC3 => set!(0, e),
            0xC4 => set!(0, h),
            0xC5 => set!(0, l),
            0xC6 => set!(0, hl),
            0xC7 => set!(0, a),

            0xC8 => set!(1, b),
            0xC9 => set!(1, c),
            0xCA => set!(1, d),
            0xCB => set!(1, e),
            0xCC => set!(1, h),
            0xCD => set!(1, l),
            0xCE => set!(1, hl),
            0xCF => set!(1, a),

            0xD0 => set!(2, b),
            0xD1 => set!(2, c),
            0xD2 => set!(2, d),
            0xD3 => set!(2, e),
            0xD4 => set!(2, h),
            0xD5 => set!(2, l),
            0xD6 => set!(2, hl),
            0xD7 => set!(2, a),

            0xD8 => set!(3, b),
            0xD9 => set!(3, c),
            0xDA => set!(3, d),
            0xDB => set!(3, e),
            0xDC => set!(3, h),
            0xDD => set!(3, l),
            0xDE => set!(3, hl),
            0xDF => set!(3, a),

            0xE0 => set!(4, b),
            0xE1 => set!(4, c),
            0xE2 => set!(4, d),
            0xE3 => set!(4, e),
            0xE4 => set!(4, h),
            0xE5 => set!(4, l),
            0xE6 => set!(4, hl),
            0xE7 => set!(4, a),

            0xE8 => set!(5, b),
            0xE9 => set!(5, c),
            0xEA => set!(5, d),
            0xEB => set!(5, e),
            0xEC => set!(5, h),
            0xED => set!(5, l),
            0xEE => set!(5, hl),
            0xEF => set!(5, a),

            0xF0 => set!(6, b),
            0xF1 => set!(6, c),
            0xF2 => set!(6, d),
            0xF3 => set!(6, e),
            0xF4 => set!(6, h),
            0xF5 => set!(6, l),
            0xF6 => set!(6, hl),
            0xF7 => set!(6, a),

            0xF8 => set!(7, b),
            0xF9 => set!(7, c),
            0xFA => set!(7, d),
            0xFB => set!(7, e),
            0xFC => set!(7, h),
            0xFD => set!(7, l),
            0xFE => set!(7, hl),
            0xFF => set!(7, a),
        };

        let mut instructions = Vec::new();
        for opcode in 0..=255 {
            instructions.push(helper(opcode));
        }

        instructions
            .try_into()
            .unwrap_or_else(|_| panic!("Unable to convert instruction vec into array."))
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
            InstructionOpcode::Prefixed(opcode) => {
                self.cb_instructions[opcode as usize].exec(cpu, bus)
            }
            InstructionOpcode::InterruptServiceRoutine => {
                self.interrupt_service_routine.exec(cpu, bus)
            }
        }
    }

    pub fn get(&mut self, opcode: InstructionOpcode) -> &InstructionStep {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => self.instructions[opcode as usize].get(),
            InstructionOpcode::Prefixed(opcode) => self.cb_instructions[opcode as usize].get(),
            InstructionOpcode::InterruptServiceRoutine => self.interrupt_service_routine.get(),
        }
    }

    pub fn reset(&mut self, opcode: InstructionOpcode) {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => self.instructions[opcode as usize].reset(),
            InstructionOpcode::Prefixed(opcode) => self.cb_instructions[opcode as usize].reset(),
            InstructionOpcode::InterruptServiceRoutine => self.interrupt_service_routine.reset(),
        }
    }
}
