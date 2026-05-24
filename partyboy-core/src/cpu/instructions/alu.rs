use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::register::Flag;

pub(super) trait CbOp {
    fn apply(cpu: &mut Cpu, v: u8) -> u8;
}

pub(super) struct Rlc;
pub(super) struct Rrc;
pub(super) struct Rl;
pub(super) struct Rr;
pub(super) struct Sla;
pub(super) struct Sra;
pub(super) struct Swap;
pub(super) struct Srl;

impl CbOp for Rlc {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.rlc(v)
    }
}

impl CbOp for Rrc {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.rrc(v)
    }
}

impl CbOp for Rl {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.rl(v)
    }
}

impl CbOp for Rr {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.rr(v)
    }
}

impl CbOp for Sla {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.sla(v)
    }
}

impl CbOp for Sra {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.sra(v)
    }
}

impl CbOp for Swap {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.swap(v)
    }
}

impl CbOp for Srl {
    #[inline(always)]
    fn apply(cpu: &mut Cpu, v: u8) -> u8 {
        cpu.srl(v)
    }
}

impl Cpu {
    pub(super) fn alu_add(&mut self, src: u8) {
        let (result, overflown) = self.af.hi.overflowing_add(src);
        self.clear_flag(Flag::N);
        self.set_flag_if_cond_else_clear(overflown, Flag::C);
        self.handle_z_flag(result);
        let half_carry = ((self.af.hi & 0x0F) + (src & 0x0F)) > 0x0F;
        self.set_flag_if_cond_else_clear(half_carry, Flag::H);
        self.af.hi = result;
    }

    pub(super) fn alu_adc(&mut self, src: u8) {
        let carry = u8::from(self.is_flag_set(Flag::C));
        let a = self.af.hi;
        let half_carry = ((a & 0x0F) + (src & 0x0F) + carry) > 0x0F;
        let full_carry = (a as u16) + (src as u16) + (carry as u16) > 0xFF;
        let result = a.wrapping_add(src).wrapping_add(carry);

        self.handle_z_flag(result);
        self.set_flag_if_cond_else_clear(half_carry, Flag::H);
        self.set_flag_if_cond_else_clear(full_carry, Flag::C);
        self.clear_flag(Flag::N);
        self.af.hi = result;
    }

    pub(super) fn alu_sub(&mut self, src: u8) {
        let a = self.af.hi;
        self.set_flag_if_cond_else_clear(src > a, Flag::C);
        self.set_flag_if_cond_else_clear((src & 0x0F) > (a & 0x0F), Flag::H);
        let result = a.wrapping_sub(src);
        self.af.hi = result;
        self.handle_z_flag(result);
        self.set_flag(Flag::N);
    }

    pub(super) fn alu_sbc(&mut self, src: u8) {
        let carry = u8::from(self.is_flag_set(Flag::C));
        let a = self.af.hi;
        let half_carry = ((a & 0x0F) as i16) - ((src & 0x0F) as i16) - (carry as i16) < 0;
        let full_carry = (a as i16) - (src as i16) - (carry as i16) < 0;
        let result = a.wrapping_sub(src).wrapping_sub(carry);

        self.handle_z_flag(result);
        self.set_flag_if_cond_else_clear(half_carry, Flag::H);
        self.set_flag_if_cond_else_clear(full_carry, Flag::C);
        self.set_flag(Flag::N);
        self.af.hi = result;
    }

    pub(super) fn alu_and(&mut self, src: u8) {
        self.af.hi &= src;
        self.handle_z_flag(self.af.hi);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.set_flag(Flag::H);
    }

    pub(super) fn alu_xor(&mut self, src: u8) {
        self.af.hi ^= src;
        self.handle_z_flag(self.af.hi);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);
    }

    pub(super) fn alu_or(&mut self, src: u8) {
        self.af.hi |= src;
        self.handle_z_flag(self.af.hi);
        self.clear_flag(Flag::C);
        self.clear_flag(Flag::N);
        self.clear_flag(Flag::H);
    }

    pub(super) fn alu_cp(&mut self, src: u8) {
        let a = self.af.hi;
        self.set_flag_if_cond_else_clear(a == src, Flag::Z);
        self.set_flag_if_cond_else_clear(src > a, Flag::C);
        self.set_flag_if_cond_else_clear((src & 0x0F) > (a & 0x0F), Flag::H);
        self.set_flag(Flag::N);
    }

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
