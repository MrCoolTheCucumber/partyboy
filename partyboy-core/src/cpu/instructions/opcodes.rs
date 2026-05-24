use super::alu::CbOp;
use super::instruction::{Instruction, InstructionState, InstructionStep};
use super::regs::cond::Cond;
use super::regs::{Reg8, Reg16};

const FETCH_OP8: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    cpu.operand8 = cpu.fetch(bus);
    InstructionState::InProgress
});

const FETCH_OP8_EXEC_NEXT: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    cpu.operand8 = cpu.fetch(bus);
    InstructionState::ExecNextInstantly
});

const FETCH_OP16_HI: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    let hi = cpu.fetch(bus);
    cpu.operand16 = ((hi as u16) << 8) | cpu.operand8 as u16;
    InstructionState::InProgress
});

const FETCH_OP16_HI_EXEC_NEXT: InstructionStep = InstructionStep::Standard(|cpu, bus| {
    let hi = cpu.fetch(bus);
    cpu.operand16 = ((hi as u16) << 8) | cpu.operand8 as u16;
    InstructionState::ExecNextInstantly
});

const BLANK: InstructionStep = InstructionStep::Standard(|_, _| InstructionState::InProgress);

const BLANK_EXEC_NEXT: InstructionStep =
    InstructionStep::Standard(|_, _| InstructionState::ExecNextInstantly);

pub fn nop() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|_, _| InstructionState::Finished)],
    }
}

pub fn cb_prefix_stub() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|_, _| unimplemented!("CB prefix"))],
    }
}

pub fn unused_opcode<const OP: u16>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|_, _| {
            panic!("Executed unused opcode {OP:#06X}")
        })],
    }
}

pub fn di() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|_, bus| {
            bus.interrupts.disable_master();
            InstructionState::Finished
        })],
    }
}

pub fn ei() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, bus| {
            if !bus.interrupts.is_master_enabled() && !cpu.ei_delay {
                cpu.ei_delay = true;
                cpu.ei_delay_cycles = 4;
            }
            InstructionState::Finished
        })],
    }
}

pub fn halt() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, bus| {
            if bus.interrupts.is_master_enabled() {
                cpu.halted = true;
            } else if bus.interrupts.enable & bus.interrupts.flags & 0x1F != 0 {
                cpu.halt_bug_triggered = true;
            } else {
                cpu.halted = true;
                cpu.halted_waiting_for_interrupt_pending = true;
                bus.interrupts.waiting_for_halt_if = true;
            }
            InstructionState::Finished
        })],
    }
}

pub fn stop() -> Instruction {
    static STEPS: [InstructionStep; 2051] = {
        let mut arr = [BLANK; 2051];
        arr[0] = InstructionStep::Instant(|cpu, bus| {
            cpu.switching_speed = true;
            if bus.cpu_speed_controller.is_speed_switch_prepared() {
                bus.cpu_speed_controller.switch_speed();
                return InstructionState::InProgress;
            }
            InstructionState::Finished
        });
        arr[2050] = InstructionStep::Standard(|cpu, _| {
            cpu.switching_speed = false;
            InstructionState::Finished
        });
        arr
    };
    Instruction { steps: &STEPS }
}

pub fn daa() -> Instruction {
    use crate::cpu::register::Flag;
    // https://forums.nesdev.com/viewtopic.php?t=15944
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            if cpu.is_flag_set(Flag::N) {
                if cpu.is_flag_set(Flag::C) {
                    cpu.af.hi = cpu.af.hi.wrapping_sub(0x60);
                }
                if cpu.is_flag_set(Flag::H) {
                    cpu.af.hi = cpu.af.hi.wrapping_sub(0x6);
                }
            } else {
                if cpu.is_flag_set(Flag::C) || cpu.af.hi > 0x99 {
                    cpu.af.hi = cpu.af.hi.wrapping_add(0x60);
                    cpu.set_flag(Flag::C);
                }
                if cpu.is_flag_set(Flag::H) || (cpu.af.hi & 0xF) > 0x09 {
                    cpu.af.hi = cpu.af.hi.wrapping_add(0x6);
                }
            }
            cpu.handle_z_flag(cpu.af.hi);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn cpl() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            cpu.af.hi = !cpu.af.hi;
            cpu.set_flag(Flag::N);
            cpu.set_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn scf() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            cpu.set_flag(Flag::C);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn ccf() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            if cpu.is_flag_set(Flag::C) {
                cpu.clear_flag(Flag::C);
            } else {
                cpu.set_flag(Flag::C);
            }
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn rlca() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            let carry = (cpu.af.hi & 0x80) >> 7;
            cpu.set_flag_if_cond_else_clear(carry != 0, Flag::C);
            cpu.af.hi = (cpu.af.hi << 1).wrapping_add(carry);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn rrca() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            let carry = cpu.af.hi & 0x01 != 0;
            cpu.af.hi >>= 1;
            if carry {
                cpu.af.hi |= 0x80;
            }
            cpu.set_flag_if_cond_else_clear(carry, Flag::C);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn rla() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            let carry_in = cpu.is_flag_set(Flag::C);
            cpu.set_flag_if_cond_else_clear(cpu.af.hi & 0x80 != 0, Flag::C);
            cpu.af.hi <<= 1;
            if carry_in {
                cpu.af.hi += 1;
            }
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn rra() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            let carry_in = if cpu.is_flag_set(Flag::C) { 1 << 7 } else { 0 };
            cpu.set_flag_if_cond_else_clear(cpu.af.hi & 0x01 != 0, Flag::C);
            cpu.af.hi = (cpu.af.hi >> 1).wrapping_add(carry_in);
            cpu.clear_flag(Flag::N);
            cpu.clear_flag(Flag::Z);
            cpu.clear_flag(Flag::H);
            InstructionState::Finished
        })],
    }
}

pub fn ld_r16_u16<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[
            InstructionStep::Standard(|cpu, bus| {
                let lo = cpu.fetch(bus);
                R::write_lo(cpu, lo);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let hi = cpu.fetch(bus);
                R::write_hi(cpu, hi);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_u16_sp() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI,
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u16(cpu.operand16, cpu.sp);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_sp_hl() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, _| {
            cpu.sp = cpu.hl.into();
            InstructionState::Finished
        })],
    }
}

pub fn ld_hl_sp_i8() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[
            FETCH_OP8,
            InstructionStep::Standard(|cpu, _| {
                let arg = cpu.operand8 as i8 as i16 as u16;
                let half_carry = (cpu.sp & 0x000F) + (arg & 0x000F) > 0x000F;
                let carry = (cpu.sp & 0x00FF) + (arg & 0x00FF) > 0x00FF;

                cpu.clear_flag(Flag::Z);
                cpu.clear_flag(Flag::N);
                cpu.set_flag_if_cond_else_clear(carry, Flag::C);
                cpu.set_flag_if_cond_else_clear(half_carry, Flag::H);

                cpu.hl = cpu.sp.wrapping_add(arg).into();
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_mem_u16_a() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI,
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(cpu.operand16, cpu.af.hi);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_a_mem_u16() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI,
            InstructionStep::Standard(|cpu, bus| {
                cpu.af.hi = bus.read_u8(cpu.operand16);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_ff00_u8_a() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(0xFF00 + cpu.operand8 as u16, cpu.af.hi);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_a_ff00_u8() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            InstructionStep::Standard(|cpu, bus| {
                cpu.af.hi = bus.read_u8(0xFF00 + cpu.operand8 as u16);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_ff00_c_a() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(0xFF00 + cpu.bc.lo as u16, cpu.af.hi);
            InstructionState::Finished
        })],
    }
}

pub fn ld_a_ff00_c() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            cpu.af.hi = bus.read_u8(0xFF00 + cpu.bc.lo as u16);
            InstructionState::Finished
        })],
    }
}

pub fn ld_r8_u8<R: Reg8>() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| {
                R::write(cpu, cpu.operand8);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_r8_r8<Dst: Reg8, Src: Reg8>() -> Instruction {
    Instruction {
        steps: &[
            #[allow(clippy::self_assignment)]
            InstructionStep::Instant(|cpu, _| {
                let v = Src::read(cpu);
                Dst::write(cpu, v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_r8_hlmem<Dst: Reg8>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            let v = bus.read_u8(cpu.hl.into());
            Dst::write(cpu, v);
            InstructionState::Finished
        })],
    }
}

pub fn ld_hlmem_r8<Src: Reg8>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.hl.into(), Src::read(cpu));
            InstructionState::Finished
        })],
    }
}

pub fn ld_hlmem_u8() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            InstructionStep::Standard(|cpu, bus| {
                bus.write_u8(cpu.hl.into(), cpu.operand8);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ld_mem_a<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(R::read(cpu), cpu.af.hi);
            InstructionState::Finished
        })],
    }
}

pub fn ld_mem_a_hl_inc() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.hl.into(), cpu.af.hi);
            cpu.hl += 1;
            InstructionState::Finished
        })],
    }
}

pub fn ld_mem_a_hl_dec() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            bus.write_u8(cpu.hl.into(), cpu.af.hi);
            cpu.hl -= 1;
            InstructionState::Finished
        })],
    }
}

pub fn ld_a_mem<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            cpu.af.hi = bus.read_u8(R::read(cpu));
            InstructionState::Finished
        })],
    }
}

pub fn ld_a_mem_hl_inc() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            cpu.af.hi = bus.read_u8(cpu.hl.into());
            cpu.hl += 1;
            InstructionState::Finished
        })],
    }
}

pub fn ld_a_mem_hl_dec() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, bus| {
            cpu.af.hi = bus.read_u8(cpu.hl.into());
            cpu.hl -= 1;
            InstructionState::Finished
        })],
    }
}

pub fn inc_r16<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, _| {
            R::inc(cpu);
            InstructionState::Finished
        })],
    }
}

pub fn dec_r16<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, _| {
            R::dec(cpu);
            InstructionState::Finished
        })],
    }
}

pub fn inc_r8<R: Reg8>() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            let old = R::read(cpu);
            cpu.set_flag_if_cond_else_clear((old & 0x0F) == 0x0F, Flag::H);
            let new = old.wrapping_add(1);
            R::write(cpu, new);
            cpu.handle_z_flag(new);
            cpu.clear_flag(Flag::N);
            InstructionState::Finished
        })],
    }
}

pub fn dec_r8<R: Reg8>() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            let old = R::read(cpu);
            cpu.set_flag_if_cond_else_clear((old & 0x0F) == 0x00, Flag::H);
            let new = old.wrapping_sub(1);
            R::write(cpu, new);
            cpu.handle_z_flag(new);
            cpu.set_flag(Flag::N);
            InstructionState::Finished
        })],
    }
}

pub fn inc_hlmem() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[
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
            }),
        ],
    }
}

pub fn dec_hlmem() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[
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
            }),
        ],
    }
}

pub fn add_hl_r16<R: Reg16>() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[InstructionStep::Standard(|cpu, _| {
            let lhs: u16 = cpu.hl.into();
            let rhs: u16 = R::read(cpu);
            let (result, overflown) = lhs.overflowing_add(rhs);

            cpu.clear_flag(Flag::N);
            cpu.set_flag_if_cond_else_clear(overflown, Flag::C);
            let half_carry = (lhs & 0xFFF) + (rhs & 0xFFF) > 0xFFF;
            cpu.set_flag_if_cond_else_clear(half_carry, Flag::H);

            cpu.hl = result.into();
            InstructionState::Finished
        })],
    }
}

pub fn add_sp_i8() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[
            FETCH_OP8,
            BLANK,
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
            }),
        ],
    }
}

macro_rules! alu_builders {
    ($name:ident, $method:ident) => {
        paste::paste! {
            pub fn [<$name _a_r8>]<R: Reg8>() -> Instruction {
                Instruction { steps: &[InstructionStep::Instant(|cpu, _| {
                    let src = R::read(cpu);
                    cpu.$method(src);
                    InstructionState::Finished
                })] }
            }

            pub fn [<$name _a_imm8>]() -> Instruction {
                Instruction { steps: &[
                    FETCH_OP8_EXEC_NEXT,
                    InstructionStep::Instant(|cpu, _| {
                        cpu.$method(cpu.operand8);
                        InstructionState::Finished
                    }),
                ]}
            }

            pub fn [<$name _a_hlmem>]() -> Instruction {
                Instruction { steps: &[
                    InstructionStep::Standard(|cpu, bus| {
                        cpu.temp8 = bus.read_u8(cpu.hl.into());
                        InstructionState::ExecNextInstantly
                    }),
                    InstructionStep::Instant(|cpu, _| {
                        cpu.$method(cpu.temp8);
                        InstructionState::Finished
                    }),
                ]}
            }
        }
    };
}

alu_builders!(add, alu_add);
alu_builders!(adc, alu_adc);
alu_builders!(sub, alu_sub);
alu_builders!(sbc, alu_sbc);
alu_builders!(and, alu_and);
alu_builders!(xor, alu_xor);
alu_builders!(or, alu_or);
alu_builders!(cp, alu_cp);

pub fn push_r16<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                let v = R::read_hi(cpu);
                cpu.push_u8_to_stack(bus, v);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let v = R::read_lo(cpu);
                cpu.push_u8_to_stack(bus, v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn pop_r16<R: Reg16>() -> Instruction {
    Instruction {
        steps: &[
            InstructionStep::Standard(|cpu, bus| {
                let v = cpu.pop_u8_from_stack(bus);
                R::write_lo(cpu, v);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let v = cpu.pop_u8_from_stack(bus);
                R::write_hi(cpu, v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn jp_u16() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI,
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn jp_cc_u16<Cnd: Cond>() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| InstructionState::Branch(Cnd::matches(cpu))),
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn jp_hl() -> Instruction {
    Instruction {
        steps: &[InstructionStep::Instant(|cpu, _| {
            cpu.pc = cpu.hl.into();
            InstructionState::Finished
        })],
    }
}

pub fn jr_i8() -> Instruction {
    Instruction {
        steps: &[FETCH_OP8, JR_STEP],
    }
}

pub fn jr_cc_i8<Cnd: Cond>() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| InstructionState::Branch(Cnd::matches(cpu))),
            JR_STEP,
        ],
    }
}

const JR_STEP: InstructionStep = InstructionStep::Standard(|cpu, _| {
    let jmp = cpu.operand8 as i8;
    if jmp < 0 {
        cpu.pc = cpu.pc.wrapping_sub(jmp.unsigned_abs() as u16);
    } else {
        cpu.pc = cpu.pc.wrapping_add(jmp as u16);
    }
    InstructionState::Finished
});

pub fn call_u16() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI,
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, cpu.pc as u8);
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn call_cc_u16<Cnd: Cond>() -> Instruction {
    Instruction {
        steps: &[
            FETCH_OP8,
            FETCH_OP16_HI_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| InstructionState::Branch(Cnd::matches(cpu))),
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, cpu.pc as u8);
                cpu.pc = cpu.operand16;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ret() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp16 = cpu.pop_u16_from_stack(bus);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.temp16;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn reti() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp16 = cpu.pop_u16_from_stack(bus);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.pc = cpu.temp16;
                bus.interrupts.enable_master();
                InstructionState::Finished
            }),
        ],
    }
}

pub fn ret_cc<Cnd: Cond>() -> Instruction {
    Instruction {
        steps: &[
            BLANK_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| InstructionState::Branch(Cnd::matches(cpu))),
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp16 = cpu.pop_u16_from_stack(bus);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, _| {
                cpu.pc = cpu.temp16;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn rst<const ADDR: u16>() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, (cpu.pc >> 8) as u8);
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                cpu.push_u8_to_stack(bus, cpu.pc as u8);
                cpu.pc = ADDR;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn interrupt_service_routine() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            BLANK,
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
                let state = bus
                    .interrupts
                    .get_interupt_state_latched(cpu.temp8, cpu.temp16 as u8);
                let vector = match state {
                    Some(flag) => {
                        bus.interrupts.clear_interupt(flag);
                        flag.vector()
                    }
                    None => 0,
                };
                bus.interrupts.disable_master();
                cpu.pc = vector;
                InstructionState::Finished
            }),
        ],
    }
}

pub fn cb_op_r8<Op: CbOp, R: Reg8>() -> Instruction {
    Instruction {
        steps: &[
            BLANK_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| {
                let v = R::read(cpu);
                let result = Op::apply(cpu, v);
                R::write(cpu, result);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn cb_op_hlmem<Op: CbOp>() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp8 = bus.read_u8(cpu.hl.into());
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let v = Op::apply(cpu, cpu.temp8);
                bus.write_u8(cpu.hl.into(), v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn bit<const N: u8, R: Reg8>() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[
            BLANK_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| {
                cpu.set_flag_if_cond_else_clear(R::read(cpu) & (1 << N) == 0, Flag::Z);
                cpu.clear_flag(Flag::N);
                cpu.set_flag(Flag::H);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn bit_hlmem<const N: u8>() -> Instruction {
    use crate::cpu::register::Flag;
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                let v = bus.read_u8(cpu.hl.into());
                cpu.set_flag_if_cond_else_clear(v & (1 << N) == 0, Flag::Z);
                cpu.clear_flag(Flag::N);
                cpu.set_flag(Flag::H);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn res<const N: u8, R: Reg8>() -> Instruction {
    Instruction {
        steps: &[
            BLANK_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| {
                let v = R::read(cpu) & !(1 << N);
                R::write(cpu, v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn res_hlmem<const N: u8>() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp8 = bus.read_u8(cpu.hl.into());
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let v = cpu.temp8 & !(1 << N);
                bus.write_u8(cpu.hl.into(), v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn set<const N: u8, R: Reg8>() -> Instruction {
    Instruction {
        steps: &[
            BLANK_EXEC_NEXT,
            InstructionStep::Instant(|cpu, _| {
                let v = R::read(cpu) | (1 << N);
                R::write(cpu, v);
                InstructionState::Finished
            }),
        ],
    }
}

pub fn set_hlmem<const N: u8>() -> Instruction {
    Instruction {
        steps: &[
            BLANK,
            InstructionStep::Standard(|cpu, bus| {
                cpu.temp8 = bus.read_u8(cpu.hl.into());
                InstructionState::InProgress
            }),
            InstructionStep::Standard(|cpu, bus| {
                let v = cpu.temp8 | (1 << N);
                bus.write_u8(cpu.hl.into(), v);
                InstructionState::Finished
            }),
        ],
    }
}
