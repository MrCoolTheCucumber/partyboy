use crate::gameboy::bus::Bus;

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

macro_rules! ret {
    () => {
        instruction! {
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

macro_rules! jp_u16 {
    () => {
        instruction! {
            fetch16,
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.temp16;
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
                        fetch8,
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
            cpu.clear_flag(Flag::H);
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
                cpu.pc = ($addr * 8) as u16;
                InstructionState::Finished
            })
        }
    };
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
            cpu.set_flag_if_cond_else_clear((cpu.temp8 & 0x0F) == 0x0F, Flag::H);
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
            0x22 => ld_mem_a!(hlplus),
            0x23 => inc_r16!(hl),
            0x24 => inc_r8!(hl, hi),
            0x25 => dec_r8!(hl, hi),
            0x26 => ld_r8_u8!(hl, hi),
            0x27 => instruction!(InstructionStep::Instant(|_, _| unimplemented!("DAA"))),
            0x28 => jr_i8!(Z),
            0x29 => add_hl_r16!(hl),
            0x2A => ld_a_mem!(hlplus),
            0x2B => dec_r16!(hl),
            0x2C => inc_r8!(hl, lo),
            0x2D => dec_r8!(hl, lo),
            0x2E => ld_r8_u8!(hl, lo),
            0x2F => instruction!(InstructionStep::Instant(|_, _| unimplemented!("CPL"))),

            0x30 => jr_i8!(NC),
            0x31 => ld_r16_u16!(sp),
            0x32 => ld_mem_a!(hlminus),
            0x33 => inc_r16!(sp),
            0x34 => inc_hl(),
            0x35 => dec_hl(),
            0x36 => ld_hlmem_u8(),
            0x37 => instruction!(InstructionStep::Instant(|_, _| unimplemented!("SCF"))),
            0x38 => jr_i8!(C),
            0x39 => add_hl_r16!(sp),
            0x3A => ld_a_mem!(hlminus),
            0x3B => dec_r16!(sp),
            0x3C => inc_r8!(af, hi),
            0x3D => dec_r8!(af, hi),
            0x3E => ld_r8_u8!(af, hi),
            0x3F => instruction!(InstructionStep::Instant(|_, _| unimplemented!("CCF"))),

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
            0x76 => instruction!(InstructionStep::Instant(|_, _| unimplemented!("HALT"))),
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

            _ => instruction!(InstructionStep::Instant(|cpu, bus| {
                cpu.pc -= 1;
                unimplemented!("{:#04X}", cpu.fetch(bus))
            })),
        };

        // mini playground
        let _ = |cpu: &mut Cpu| {};

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
