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

pub(crate) enum InstructionStep {
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

pub(super) struct Instruction {
    pub index: usize,
    pub steps: Vec<InstructionStep>,
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
        InstructionStep::Instant(|_, bus| {
            // TODO: for now lets just do nothing!
            // cpu.stopped = true;
            if bus.cpu_speed_controller.is_speed_switch_prepared() {
                bus.cpu_speed_controller.switch_speed();
            }

            // TODO: the cpu stops for 8200t? when speed switch occurs

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
            cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
            cpu.temp8 = bus.interrupts.enable;
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
        [
            instruction!(InstructionStep::Instant(|_, _| InstructionState::Finished)),
            ld_r16_u16!(bc),
            ld_mem_a!(bc),
            inc_r16!(bc),
            inc_r8!(bc, hi),
            dec_r8!(bc, hi),
            ld_r8_u8!(bc, hi),
            rlca(),
            instruction! { // LD (u16), SP
                fetch16,
                BLANK_PROGRESS,
                InstructionStep::Standard(|cpu, bus| {
                    bus.write_u16(cpu.operand16, cpu.sp);
                    InstructionState::Finished
                })
            },
            add_hl_r16!(bc),
            ld_a_mem!(bc),
            dec_r16!(bc),
            inc_r8!(bc, lo),
            dec_r8!(bc, lo),
            ld_r8_u8!(bc, lo),
            rrca(),
            stop(),
            ld_r16_u16!(de),
            ld_mem_a!(de),
            inc_r16!(de),
            inc_r8!(de, hi),
            dec_r8!(de, hi),
            ld_r8_u8!(de, hi),
            rla(),
            jr_i8!(),
            add_hl_r16!(de),
            ld_a_mem!(de),
            dec_r16!(de),
            inc_r8!(de, lo),
            dec_r8!(de, lo),
            ld_r8_u8!(de, lo),
            rra(),
            jr_i8!(NZ),
            ld_r16_u16!(hl),
            ld_mem_a!(hlplus),
            inc_r16!(hl),
            inc_r8!(hl, hi),
            dec_r8!(hl, hi),
            ld_r8_u8!(hl, hi),
            daa(),
            jr_i8!(Z),
            add_hl_r16!(hl),
            ld_a_mem!(hlplus),
            dec_r16!(hl),
            inc_r8!(hl, lo),
            dec_r8!(hl, lo),
            ld_r8_u8!(hl, lo),
            cpl(),
            jr_i8!(NC),
            ld_r16_u16!(sp),
            ld_mem_a!(hlminus),
            inc_r16!(sp),
            inc_hl(),
            dec_hl(),
            ld_hlmem_u8(),
            scf(),
            jr_i8!(C),
            add_hl_r16!(sp),
            ld_a_mem!(hlminus),
            dec_r16!(sp),
            inc_r8!(af, hi),
            dec_r8!(af, hi),
            ld_r8_u8!(af, hi),
            ccf(),
            // ld b, r8
            ld_r8_r8!(bc, hi <= bc, hi),
            ld_r8_r8!(bc, hi <= bc, lo),
            ld_r8_r8!(bc, hi <= de, hi),
            ld_r8_r8!(bc, hi <= de, lo),
            ld_r8_r8!(bc, hi <= hl, hi),
            ld_r8_r8!(bc, hi <= hl, lo),
            ld_r8_r8!(bc, hi <= HL),
            ld_r8_r8!(bc, hi <= af, hi),
            // ld c, r8
            ld_r8_r8!(bc, lo <= bc, hi),
            ld_r8_r8!(bc, lo <= bc, lo),
            ld_r8_r8!(bc, lo <= de, hi),
            ld_r8_r8!(bc, lo <= de, lo),
            ld_r8_r8!(bc, lo <= hl, hi),
            ld_r8_r8!(bc, lo <= hl, lo),
            ld_r8_r8!(bc, lo <= HL),
            ld_r8_r8!(bc, lo <= af, hi),
            // ld d, r8
            ld_r8_r8!(de, hi <= bc, hi),
            ld_r8_r8!(de, hi <= bc, lo),
            ld_r8_r8!(de, hi <= de, hi),
            ld_r8_r8!(de, hi <= de, lo),
            ld_r8_r8!(de, hi <= hl, hi),
            ld_r8_r8!(de, hi <= hl, lo),
            ld_r8_r8!(de, hi <= HL),
            ld_r8_r8!(de, hi <= af, hi),
            // ld e, r8
            ld_r8_r8!(de, lo <= bc, hi),
            ld_r8_r8!(de, lo <= bc, lo),
            ld_r8_r8!(de, lo <= de, hi),
            ld_r8_r8!(de, lo <= de, lo),
            ld_r8_r8!(de, lo <= hl, hi),
            ld_r8_r8!(de, lo <= hl, lo),
            ld_r8_r8!(de, lo <= HL),
            ld_r8_r8!(de, lo <= af, hi),
            // ld h, r8
            ld_r8_r8!(hl, hi <= bc, hi),
            ld_r8_r8!(hl, hi <= bc, lo),
            ld_r8_r8!(hl, hi <= de, hi),
            ld_r8_r8!(hl, hi <= de, lo),
            ld_r8_r8!(hl, hi <= hl, hi),
            ld_r8_r8!(hl, hi <= hl, lo),
            ld_r8_r8!(hl, hi <= HL),
            ld_r8_r8!(hl, hi <= af, hi),
            // ld l, r8
            ld_r8_r8!(hl, lo <= bc, hi),
            ld_r8_r8!(hl, lo <= bc, lo),
            ld_r8_r8!(hl, lo <= de, hi),
            ld_r8_r8!(hl, lo <= de, lo),
            ld_r8_r8!(hl, lo <= hl, hi),
            ld_r8_r8!(hl, lo <= hl, lo),
            ld_r8_r8!(hl, lo <= HL),
            ld_r8_r8!(hl, lo <= af, hi),
            // ld (HL), r8
            ld_memhl_r8!(HL <= bc, hi),
            ld_memhl_r8!(HL <= bc, lo),
            ld_memhl_r8!(HL <= de, hi),
            ld_memhl_r8!(HL <= de, lo),
            ld_memhl_r8!(HL <= hl, hi),
            ld_memhl_r8!(HL <= hl, lo),
            halt(),
            ld_memhl_r8!(HL <= af, hi),
            // ld a, r8
            ld_r8_r8!(af, hi <= bc, hi),
            ld_r8_r8!(af, hi <= bc, lo),
            ld_r8_r8!(af, hi <= de, hi),
            ld_r8_r8!(af, hi <= de, lo),
            ld_r8_r8!(af, hi <= hl, hi),
            ld_r8_r8!(af, hi <= hl, lo),
            ld_r8_r8!(af, hi <= HL),
            ld_r8_r8!(af, hi <= af, hi),
            // add a, r8
            add_a_r8!(b),
            add_a_r8!(c),
            add_a_r8!(d),
            add_a_r8!(e),
            add_a_r8!(h),
            add_a_r8!(l),
            add_a_r8!(hl),
            add_a_r8!(a),
            // adc a, r8
            adc_a_r8!(b),
            adc_a_r8!(c),
            adc_a_r8!(d),
            adc_a_r8!(e),
            adc_a_r8!(h),
            adc_a_r8!(l),
            adc_a_r8!(hl),
            adc_a_r8!(a),
            // sub a, r8
            sub_a_r8!(b),
            sub_a_r8!(c),
            sub_a_r8!(d),
            sub_a_r8!(e),
            sub_a_r8!(h),
            sub_a_r8!(l),
            sub_a_r8!(hl),
            sub_a_r8!(a),
            // sbc a, r8
            sbc_a_r8!(b),
            sbc_a_r8!(c),
            sbc_a_r8!(d),
            sbc_a_r8!(e),
            sbc_a_r8!(h),
            sbc_a_r8!(l),
            sbc_a_r8!(hl),
            sbc_a_r8!(a),
            // and a, r8
            and_a_r8!(b),
            and_a_r8!(c),
            and_a_r8!(d),
            and_a_r8!(e),
            and_a_r8!(h),
            and_a_r8!(l),
            and_a_r8!(hl),
            and_a_r8!(a),
            // xor a, r8
            xor_a_r8!(b),
            xor_a_r8!(c),
            xor_a_r8!(d),
            xor_a_r8!(e),
            xor_a_r8!(h),
            xor_a_r8!(l),
            xor_a_r8!(hl),
            xor_a_r8!(a),
            // or a, r8
            or_a_r8!(b),
            or_a_r8!(c),
            or_a_r8!(d),
            or_a_r8!(e),
            or_a_r8!(h),
            or_a_r8!(l),
            or_a_r8!(hl),
            or_a_r8!(a),
            // cp a, r8
            cp_a_r8!(b),
            cp_a_r8!(c),
            cp_a_r8!(d),
            cp_a_r8!(e),
            cp_a_r8!(h),
            cp_a_r8!(l),
            cp_a_r8!(hl),
            cp_a_r8!(a),
            ret_cc!(NZ),
            pop_r16!(bc),
            jp_cc_u16!(NZ),
            jp_u16!(),
            call_cc_u16!(NZ),
            push_r16!(bc),
            add_a_r8!(u8),
            rst_yy!(0),
            ret_cc!(Z),
            ret!(),
            jp_cc_u16!(Z),
            instruction!(InstructionStep::Instant(|_, _| unimplemented!("CB PREFIX"))),
            call_cc_u16!(Z),
            call_u16!(),
            adc_a_r8!(u8),
            rst_yy!(0x08),
            ret_cc!(NC),
            pop_r16!(de),
            jp_cc_u16!(NC),
            unused_opcode!("0xD3"),
            call_cc_u16!(NC),
            push_r16!(de),
            sub_a_r8!(u8),
            rst_yy!(0x10),
            ret_cc!(C),
            ret!(i),
            jp_cc_u16!(C),
            unused_opcode!("0xDB"),
            call_cc_u16!(C),
            unused_opcode!("0xDD"),
            sbc_a_r8!(u8),
            rst_yy!(0x18),
            ld_ff00_u8_a(),
            pop_r16!(hl),
            ld_ff00_c_a(),
            unused_opcode!("0xE3"),
            unused_opcode!("0xE4"),
            push_r16!(hl),
            and_a_r8!(u8),
            rst_yy!(0x20),
            add_sp_i8(),
            jp_hl(),
            ld_mem_u16_a(),
            unused_opcode!("0xEB"),
            unused_opcode!("0xEC"),
            unused_opcode!("0xED"),
            xor_a_r8!(u8),
            rst_yy!(0x28),
            ld_a_ff00_u8(),
            pop_r16!(af),
            ld_a_ff00_c(),
            di(),
            unused_opcode!("0xF4"),
            push_r16!(af),
            or_a_r8!(u8),
            rst_yy!(0x30),
            ld_hl_sp_i8(),
            ld_sp_hl(),
            ld_a_mem_u16(),
            ei(),
            unused_opcode!("0xFC"),
            unused_opcode!("0xFD"),
            cp_a_r8!(u8),
            rst_yy!(0x38),
        ]
    }

    fn gen_cb_instructions() -> [Instruction; 256] {
        [
            cb_op_instr!(rlc, b),
            cb_op_instr!(rlc, c),
            cb_op_instr!(rlc, d),
            cb_op_instr!(rlc, e),
            cb_op_instr!(rlc, h),
            cb_op_instr!(rlc, l),
            cb_op_hl_instr!(rlc, hl),
            cb_op_instr!(rlc, a),
            cb_op_instr!(rrc, b),
            cb_op_instr!(rrc, c),
            cb_op_instr!(rrc, d),
            cb_op_instr!(rrc, e),
            cb_op_instr!(rrc, h),
            cb_op_instr!(rrc, l),
            cb_op_hl_instr!(rrc, hl),
            cb_op_instr!(rrc, a),
            cb_op_instr!(rl, b),
            cb_op_instr!(rl, c),
            cb_op_instr!(rl, d),
            cb_op_instr!(rl, e),
            cb_op_instr!(rl, h),
            cb_op_instr!(rl, l),
            cb_op_hl_instr!(rl, hl),
            cb_op_instr!(rl, a),
            cb_op_instr!(rr, b),
            cb_op_instr!(rr, c),
            cb_op_instr!(rr, d),
            cb_op_instr!(rr, e),
            cb_op_instr!(rr, h),
            cb_op_instr!(rr, l),
            cb_op_hl_instr!(rr, hl),
            cb_op_instr!(rr, a),
            cb_op_instr!(sla, b),
            cb_op_instr!(sla, c),
            cb_op_instr!(sla, d),
            cb_op_instr!(sla, e),
            cb_op_instr!(sla, h),
            cb_op_instr!(sla, l),
            cb_op_hl_instr!(sla, hl),
            cb_op_instr!(sla, a),
            cb_op_instr!(sra, b),
            cb_op_instr!(sra, c),
            cb_op_instr!(sra, d),
            cb_op_instr!(sra, e),
            cb_op_instr!(sra, h),
            cb_op_instr!(sra, l),
            cb_op_hl_instr!(sra, hl),
            cb_op_instr!(sra, a),
            cb_op_instr!(swap, b),
            cb_op_instr!(swap, c),
            cb_op_instr!(swap, d),
            cb_op_instr!(swap, e),
            cb_op_instr!(swap, h),
            cb_op_instr!(swap, l),
            cb_op_hl_instr!(swap, hl),
            cb_op_instr!(swap, a),
            cb_op_instr!(srl, b),
            cb_op_instr!(srl, c),
            cb_op_instr!(srl, d),
            cb_op_instr!(srl, e),
            cb_op_instr!(srl, h),
            cb_op_instr!(srl, l),
            cb_op_hl_instr!(srl, hl),
            cb_op_instr!(srl, a),
            bit!(0, b),
            bit!(0, c),
            bit!(0, d),
            bit!(0, e),
            bit!(0, h),
            bit!(0, l),
            bit!(0, hl),
            bit!(0, a),
            bit!(1, b),
            bit!(1, c),
            bit!(1, d),
            bit!(1, e),
            bit!(1, h),
            bit!(1, l),
            bit!(1, hl),
            bit!(1, a),
            bit!(2, b),
            bit!(2, c),
            bit!(2, d),
            bit!(2, e),
            bit!(2, h),
            bit!(2, l),
            bit!(2, hl),
            bit!(2, a),
            bit!(3, b),
            bit!(3, c),
            bit!(3, d),
            bit!(3, e),
            bit!(3, h),
            bit!(3, l),
            bit!(3, hl),
            bit!(3, a),
            bit!(4, b),
            bit!(4, c),
            bit!(4, d),
            bit!(4, e),
            bit!(4, h),
            bit!(4, l),
            bit!(4, hl),
            bit!(4, a),
            bit!(5, b),
            bit!(5, c),
            bit!(5, d),
            bit!(5, e),
            bit!(5, h),
            bit!(5, l),
            bit!(5, hl),
            bit!(5, a),
            bit!(6, b),
            bit!(6, c),
            bit!(6, d),
            bit!(6, e),
            bit!(6, h),
            bit!(6, l),
            bit!(6, hl),
            bit!(6, a),
            bit!(7, b),
            bit!(7, c),
            bit!(7, d),
            bit!(7, e),
            bit!(7, h),
            bit!(7, l),
            bit!(7, hl),
            bit!(7, a),
            res!(0, b),
            res!(0, c),
            res!(0, d),
            res!(0, e),
            res!(0, h),
            res!(0, l),
            res!(0, hl),
            res!(0, a),
            res!(1, b),
            res!(1, c),
            res!(1, d),
            res!(1, e),
            res!(1, h),
            res!(1, l),
            res!(1, hl),
            res!(1, a),
            res!(2, b),
            res!(2, c),
            res!(2, d),
            res!(2, e),
            res!(2, h),
            res!(2, l),
            res!(2, hl),
            res!(2, a),
            res!(3, b),
            res!(3, c),
            res!(3, d),
            res!(3, e),
            res!(3, h),
            res!(3, l),
            res!(3, hl),
            res!(3, a),
            res!(4, b),
            res!(4, c),
            res!(4, d),
            res!(4, e),
            res!(4, h),
            res!(4, l),
            res!(4, hl),
            res!(4, a),
            res!(5, b),
            res!(5, c),
            res!(5, d),
            res!(5, e),
            res!(5, h),
            res!(5, l),
            res!(5, hl),
            res!(5, a),
            res!(6, b),
            res!(6, c),
            res!(6, d),
            res!(6, e),
            res!(6, h),
            res!(6, l),
            res!(6, hl),
            res!(6, a),
            res!(7, b),
            res!(7, c),
            res!(7, d),
            res!(7, e),
            res!(7, h),
            res!(7, l),
            res!(7, hl),
            res!(7, a),
            set!(0, b),
            set!(0, c),
            set!(0, d),
            set!(0, e),
            set!(0, h),
            set!(0, l),
            set!(0, hl),
            set!(0, a),
            set!(1, b),
            set!(1, c),
            set!(1, d),
            set!(1, e),
            set!(1, h),
            set!(1, l),
            set!(1, hl),
            set!(1, a),
            set!(2, b),
            set!(2, c),
            set!(2, d),
            set!(2, e),
            set!(2, h),
            set!(2, l),
            set!(2, hl),
            set!(2, a),
            set!(3, b),
            set!(3, c),
            set!(3, d),
            set!(3, e),
            set!(3, h),
            set!(3, l),
            set!(3, hl),
            set!(3, a),
            set!(4, b),
            set!(4, c),
            set!(4, d),
            set!(4, e),
            set!(4, h),
            set!(4, l),
            set!(4, hl),
            set!(4, a),
            set!(5, b),
            set!(5, c),
            set!(5, d),
            set!(5, e),
            set!(5, h),
            set!(5, l),
            set!(5, hl),
            set!(5, a),
            set!(6, b),
            set!(6, c),
            set!(6, d),
            set!(6, e),
            set!(6, h),
            set!(6, l),
            set!(6, hl),
            set!(6, a),
            set!(7, b),
            set!(7, c),
            set!(7, d),
            set!(7, e),
            set!(7, h),
            set!(7, l),
            set!(7, hl),
            set!(7, a),
        ]
    }

    pub(super) fn get(&mut self, opcode: InstructionOpcode) -> &mut Instruction {
        match opcode {
            InstructionOpcode::Unprefixed(opcode) => &mut self.instructions[opcode as usize],
            InstructionOpcode::Prefixed(opcode) => &mut self.cb_instructions[opcode as usize],
            InstructionOpcode::InterruptServiceRoutine => &mut self.interrupt_service_routine,
        }
    }
}
