use super::super::{register::Flag, Cpu};
use super::instruction::{Instruction, InstructionState, InstructionStep};
use crate::bus::Bus;

use paste::paste;

pub(super) const __FETCH_OPERAND8: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    cpu.operand8 = cpu.fetch(bus);
    InstructionState::InProgress
});

pub(super) const __FETCH_OPERAND16: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    let hi = cpu.fetch(bus);
    cpu.operand16 = (hi as u16) << 8 | cpu.operand8 as u16;
    InstructionState::InProgress
});

pub(super) const FETCH_OP8_EXECNEXTINSTANT: InstructionStep =
    InstructionStep::Standard(|cpu, bus| {
        cpu.operand8 = cpu.fetch(bus);
        InstructionState::ExecNextInstantly
    });

pub(super) const BLANK_PROGRESS: InstructionStep =
    InstructionStep::Standard(|_, _| InstructionState::InProgress);

pub(super) const BLANK_PROGRESS_EXEC_NEXT_INSTANT: InstructionStep =
    InstructionStep::Standard(|_, _| InstructionState::ExecNextInstantly);

pub(super) const BRANCH_ZERO: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(cpu.is_flag_set(Flag::Z)));

pub(super) const BRANCH_NOT_ZERO: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(!cpu.is_flag_set(Flag::Z)));

pub(super) const BRANCH_CARRY: InstructionStep =
    InstructionStep::Instant(|cpu, _| InstructionState::Branch(cpu.is_flag_set(Flag::C)));

pub(super) const BRANCH_NOT_CARRY: InstructionStep =
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
                cpu.pc = cpu.pc.wrapping_sub(jmp_amount.unsigned_abs() as u16);
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
            r8!(cpu, a) &= r8!(cpu, $location);

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
            r8!(cpu, a) ^= r8!(cpu, $location);

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
            r8!(cpu, a) |= r8!(cpu, $location);

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

pub(super) fn daa() -> Instruction {
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

pub(super) fn cpl() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            r8!(cpu, a) = !r8!(cpu, a);
            cpu.set_flag(Flag::N);
            cpu.set_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

pub(super) fn scf() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            cpu.set_flag(Flag::C);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

pub(super) fn ccf() -> Instruction {
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

pub(super) fn rlca() -> Instruction {
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

pub(super) fn rrca() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            let carry = r8!(cpu, a) & 0b00000001 > 0;
            r8!(cpu, a) >>= 1;
            if carry { r8!(cpu, a) |= 0b10000000; }

            cpu.set_flag_if_cond_else_clear(carry, Flag::C);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

pub(super) fn rla() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            let is_carry_set = cpu.is_flag_set(Flag::C);
            cpu.set_flag_if_cond_else_clear(r8!(cpu, a) & 0x80 > 0, Flag::C);

            r8!(cpu, a) <<= 1;
            if is_carry_set { r8!(cpu, a) += 1 };

            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })
    }
}

pub(super) fn rra() -> Instruction {
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

pub(super) fn inc_hl() -> Instruction {
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

pub(super) fn dec_hl() -> Instruction {
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

pub(super) fn ld_hlmem_u8() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.hl.into(), cpu.operand8);
            InstructionState::Finished
        })
    }
}

pub(super) fn ld_ff00_u8_a() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(0xFF00 + (cpu.operand8 as u16), r8!(cpu, a));
            InstructionState::Finished
        })
    }
}

pub(super) fn ld_a_ff00_u8() -> Instruction {
    instruction! {
        fetch8,
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, a) = bus.read_u8(0xFF00 + (cpu.operand8 as u16));
            InstructionState::Finished
        })
    }
}

pub(super) fn ld_ff00_c_a() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(0xFF00 + (r8!(cpu, c) as u16), r8!(cpu, a));
            InstructionState::Finished
        })
    }
}

pub(super) fn ld_a_ff00_c() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, a) = bus.read_u8(0xFF00 + (r8!(cpu, c) as u16));
            InstructionState::Finished
        })
    }
}

pub(super) fn add_sp_i8() -> Instruction {
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

pub(super) fn ld_hl_sp_i8() -> Instruction {
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

pub(super) fn jp_hl() -> Instruction {
    instruction! {
        InstructionStep::Instant(|cpu, _| {
            cpu.pc = cpu.hl.into();
            InstructionState::Finished
        })
    }
}

pub(super) fn stop() -> Instruction {
    let mut steps = Vec::new();
    steps.push(InstructionStep::Instant(|cpu, bus| {
        cpu.switching_speed = true;
        if bus.cpu_speed_controller.is_speed_switch_prepared() {
            bus.cpu_speed_controller.switch_speed();
            return InstructionState::InProgress;
        }

        InstructionState::Finished
    }));

    for _ in 0..2049 {
        steps.push(BLANK_PROGRESS)
    }

    steps.push(InstructionStep::Standard(|cpu, _| {
        cpu.switching_speed = false;
        InstructionState::Finished
    }));

    Instruction { steps }
}

pub(super) fn ld_sp_hl() -> Instruction {
    instruction! {
        InstructionStep::Standard(|cpu, _| {
            cpu.sp = cpu.hl.into();
            InstructionState::Finished
        })
    }
}

pub(super) fn ld_mem_u16_a() -> Instruction {
    instruction! {
        fetch16,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.operand16, r8!(cpu, a));
            InstructionState::Finished
        })
    }
}

pub(super) fn ld_a_mem_u16() -> Instruction {
    instruction! {
        fetch16,
        InstructionStep::Standard(|cpu, bus| {
            r8!(cpu, a) = bus.read_u8(cpu.operand16);
            InstructionState::Finished
        })
    }
}

pub(super) fn di() -> Instruction {
    instruction! {
        InstructionStep::Instant(|_, bus| {
            bus.interrupts.disable_master();
            InstructionState::Finished
        })
    }
}

pub(super) fn ei() -> Instruction {
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

pub(super) fn halt() -> Instruction {
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

pub(super) fn interrupt_service_routine() -> Instruction {
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
                r8!(cpu, $reg) &= !(1 << $bit);
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
                r8!(cpu, $reg) |= (1 << $bit);
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
    pub(super) fn rlc(&mut self, val: u8) -> u8 {
        self.set_flag_if_cond_else_clear((val & 0x80) != 0, Flag::C);

        let carry = (val & 0x80) >> 7;
        let result = (val << 1).wrapping_add(carry);

        self.handle_z_flag(val);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    pub(super) fn rrc(&mut self, val: u8) -> u8 {
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

    pub(super) fn rl(&mut self, val: u8) -> u8 {
        let carry = u8::from(self.is_flag_set(Flag::C));
        let result = (val << 1).wrapping_add(carry);

        self.set_flag_if_cond_else_clear(val & 0x80 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    pub(super) fn rr(&mut self, val: u8) -> u8 {
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

    pub(super) fn sla(&mut self, val: u8) -> u8 {
        let result = val << 1;

        self.set_flag_if_cond_else_clear(val & 0x80 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    pub(super) fn sra(&mut self, val: u8) -> u8 {
        let result = (val & 0x80) | (val >> 1);

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    pub(super) fn swap(&mut self, val: u8) -> u8 {
        let result = ((val & 0x0F) << 4) | ((val & 0xF0) >> 4);

        self.handle_z_flag(result);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    pub(super) fn srl(&mut self, val: u8) -> u8 {
        let result = val >> 1;

        self.set_flag_if_cond_else_clear(val & 0x01 != 0, Flag::C);
        self.handle_z_flag(result);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);

        result
    }

    #[inline(always)]
    pub(super) fn set_flag(&mut self, flag: Flag) {
        self.af.lo |= flag as u8;
    }

    #[inline(always)]
    pub(super) fn clear_flag(&mut self, flag: Flag) {
        self.af.lo &= !(flag as u8);
    }

    #[inline(always)]
    pub(super) fn is_flag_set(&self, flag: Flag) -> bool {
        self.af.lo & (flag as u8) > 0
    }

    #[inline(always)]
    pub(super) fn set_flag_if_cond_else_clear(&mut self, cond: bool, flag: Flag) {
        match cond {
            true => self.af.lo |= flag as u8,
            false => self.af.lo &= !(flag as u8),
        }
    }

    #[inline(always)]
    pub(super) fn handle_z_flag(&mut self, val: u8) {
        if val == 0 {
            self.af.lo |= Flag::Z as u8;
        } else {
            self.af.lo &= !(Flag::Z as u8);
        }
    }

    pub(super) fn pop_u16_from_stack(&mut self, bus: &Bus) -> u16 {
        let val = bus.read_u16(self.sp);
        self.sp = self.sp.wrapping_add(2);
        val
    }

    pub(super) fn pop_u8_from_stack(&mut self, bus: &Bus) -> u8 {
        let val = bus.read_u8(self.sp);
        self.sp = self.sp.wrapping_add(1);
        val
    }

    pub(super) fn push_u8_to_stack(&mut self, bus: &mut Bus, val: u8) {
        self.sp = self.sp.wrapping_sub(1);
        bus.write_u8(self.sp, val);
    }
}

pub(super) fn nop() -> Instruction {
    instruction!(InstructionStep::Instant(|_, _| InstructionState::Finished))
}

pub(super) fn ld_u16_sp() -> Instruction {
    instruction! {
        fetch16,
        BLANK_PROGRESS,
        InstructionStep::Standard(|cpu, bus| {
            bus.write_u16(cpu.operand16, cpu.sp);
            InstructionState::Finished
        })
    }
}

pub(super) use __adc;
pub(super) use __add;
pub(super) use __and;
pub(super) use __cp;
pub(super) use __define_branching_op_macro;
pub(super) use __define_op_macro;
pub(super) use __jr;
pub(super) use __or;
pub(super) use __pop_r16_af_edgecase;
pub(super) use __read_hl;
pub(super) use __sbc;
pub(super) use __sub;
pub(super) use __xor;
pub(super) use add_hl_r16;
pub(super) use bit;
pub(super) use branch_condition;
pub(super) use call_cc_u16;
pub(super) use call_u16;
pub(super) use cb_op_hl_instr;
pub(super) use cb_op_instr;
pub(super) use dec_r16;
pub(super) use dec_r8;
pub(super) use enable_master;
pub(super) use inc_r16;
pub(super) use inc_r8;
pub(super) use instruction;
pub(super) use jp_cc_u16;
pub(super) use jp_u16;
pub(super) use ld_a_mem;
pub(super) use ld_mem_a;
pub(super) use ld_memhl_r8;
pub(super) use ld_r16_u16;
pub(super) use ld_r8_r8;
pub(super) use ld_r8_u8;
pub(super) use pop_r16;
pub(super) use push_r16;
pub(super) use r8;
pub(super) use res;
pub(super) use ret;
pub(super) use ret_cc;
pub(super) use rst_yy;
pub(super) use set;
pub(super) use unused_opcode;

// Export macros generated by other macros
//
// Looks like there's some sort of clippy bug where it thinks the "import" is redundant, if
// they are removed then the instruction_cache module will fail to compile

#[allow(clippy::single_component_path_imports)]
pub(super) use adc_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use add_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use and_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use cp_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use jr_i8;
#[allow(clippy::single_component_path_imports)]
pub(super) use or_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use sbc_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use sub_a_r8;
#[allow(clippy::single_component_path_imports)]
pub(super) use xor_a_r8;
