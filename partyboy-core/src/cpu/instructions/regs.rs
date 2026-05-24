use super::super::Cpu;
use super::super::register::Flag;

pub trait Reg8 {
    fn read(cpu: &Cpu) -> u8;
    fn write(cpu: &mut Cpu, v: u8);
}

pub struct A;
pub struct B;
pub struct C;
pub struct D;
pub struct E;
pub struct H;
pub struct L;

impl Reg8 for A {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.af.hi
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.af.hi = v
    }
}

impl Reg8 for B {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.bc.hi
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.bc.hi = v
    }
}

impl Reg8 for C {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.bc.lo
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.bc.lo = v
    }
}

impl Reg8 for D {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.de.hi
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.de.hi = v
    }
}

impl Reg8 for E {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.de.lo
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.de.lo = v
    }
}

impl Reg8 for H {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.hl.hi
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.hl.hi = v
    }
}

impl Reg8 for L {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u8 {
        cpu.hl.lo
    }
    #[inline(always)]
    fn write(cpu: &mut Cpu, v: u8) {
        cpu.hl.lo = v
    }
}

pub trait Reg16 {
    fn read(cpu: &Cpu) -> u16;
    fn read_hi(cpu: &Cpu) -> u8;
    fn read_lo(cpu: &Cpu) -> u8;
    fn write_lo(cpu: &mut Cpu, v: u8);
    fn write_hi(cpu: &mut Cpu, v: u8);
    fn inc(cpu: &mut Cpu);
    fn dec(cpu: &mut Cpu);
}

pub struct BC;
pub struct DE;
pub struct HL;
pub struct AF;
pub struct SP;

impl Reg16 for BC {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u16 {
        cpu.bc.into()
    }
    #[inline(always)]
    fn read_hi(cpu: &Cpu) -> u8 {
        cpu.bc.hi
    }
    #[inline(always)]
    fn read_lo(cpu: &Cpu) -> u8 {
        cpu.bc.lo
    }
    #[inline(always)]
    fn write_lo(cpu: &mut Cpu, v: u8) {
        cpu.bc.lo = v
    }
    #[inline(always)]
    fn write_hi(cpu: &mut Cpu, v: u8) {
        cpu.bc.hi = v
    }
    #[inline(always)]
    fn inc(cpu: &mut Cpu) {
        cpu.bc += 1
    }
    #[inline(always)]
    fn dec(cpu: &mut Cpu) {
        cpu.bc -= 1
    }
}

impl Reg16 for DE {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u16 {
        cpu.de.into()
    }
    #[inline(always)]
    fn read_hi(cpu: &Cpu) -> u8 {
        cpu.de.hi
    }
    #[inline(always)]
    fn read_lo(cpu: &Cpu) -> u8 {
        cpu.de.lo
    }
    #[inline(always)]
    fn write_lo(cpu: &mut Cpu, v: u8) {
        cpu.de.lo = v
    }
    #[inline(always)]
    fn write_hi(cpu: &mut Cpu, v: u8) {
        cpu.de.hi = v
    }
    #[inline(always)]
    fn inc(cpu: &mut Cpu) {
        cpu.de += 1
    }
    #[inline(always)]
    fn dec(cpu: &mut Cpu) {
        cpu.de -= 1
    }
}

impl Reg16 for HL {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u16 {
        cpu.hl.into()
    }
    #[inline(always)]
    fn read_hi(cpu: &Cpu) -> u8 {
        cpu.hl.hi
    }
    #[inline(always)]
    fn read_lo(cpu: &Cpu) -> u8 {
        cpu.hl.lo
    }
    #[inline(always)]
    fn write_lo(cpu: &mut Cpu, v: u8) {
        cpu.hl.lo = v
    }
    #[inline(always)]
    fn write_hi(cpu: &mut Cpu, v: u8) {
        cpu.hl.hi = v
    }
    #[inline(always)]
    fn inc(cpu: &mut Cpu) {
        cpu.hl += 1
    }
    #[inline(always)]
    fn dec(cpu: &mut Cpu) {
        cpu.hl -= 1
    }
}

impl Reg16 for AF {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u16 {
        cpu.af.into()
    }
    #[inline(always)]
    fn read_hi(cpu: &Cpu) -> u8 {
        cpu.af.hi
    }
    #[inline(always)]
    fn read_lo(cpu: &Cpu) -> u8 {
        cpu.af.lo
    }
    // AF's low byte is the flag register; only the upper nibble holds meaningful
    // flag bits. Popping into AF must mask the low nibble.
    #[inline(always)]
    fn write_lo(cpu: &mut Cpu, v: u8) {
        cpu.af.lo = v & 0xF0
    }
    #[inline(always)]
    fn write_hi(cpu: &mut Cpu, v: u8) {
        cpu.af.hi = v
    }
    #[inline(always)]
    fn inc(cpu: &mut Cpu) {
        cpu.af += 1
    }
    #[inline(always)]
    fn dec(cpu: &mut Cpu) {
        cpu.af -= 1
    }
}

impl Reg16 for SP {
    #[inline(always)]
    fn read(cpu: &Cpu) -> u16 {
        cpu.sp
    }
    #[inline(always)]
    fn read_hi(cpu: &Cpu) -> u8 {
        (cpu.sp >> 8) as u8
    }
    #[inline(always)]
    fn read_lo(cpu: &Cpu) -> u8 {
        cpu.sp as u8
    }
    #[inline(always)]
    fn write_lo(cpu: &mut Cpu, v: u8) {
        cpu.sp = (cpu.sp & 0xFF00) | v as u16
    }
    #[inline(always)]
    fn write_hi(cpu: &mut Cpu, v: u8) {
        cpu.sp = (cpu.sp & 0x00FF) | ((v as u16) << 8)
    }
    #[inline(always)]
    fn inc(cpu: &mut Cpu) {
        cpu.sp = cpu.sp.wrapping_add(1)
    }
    #[inline(always)]
    fn dec(cpu: &mut Cpu) {
        cpu.sp = cpu.sp.wrapping_sub(1)
    }
}

pub mod cond {
    use super::*;

    pub trait Cond {
        fn matches(cpu: &Cpu) -> bool;
    }

    pub struct Z;
    pub struct NZ;
    pub struct Carry;
    pub struct NotCarry;

    impl Cond for Z {
        #[inline(always)]
        fn matches(cpu: &Cpu) -> bool {
            cpu.is_flag_set(Flag::Z)
        }
    }
    impl Cond for NZ {
        #[inline(always)]
        fn matches(cpu: &Cpu) -> bool {
            !cpu.is_flag_set(Flag::Z)
        }
    }
    impl Cond for Carry {
        #[inline(always)]
        fn matches(cpu: &Cpu) -> bool {
            cpu.is_flag_set(Flag::C)
        }
    }
    impl Cond for NotCarry {
        #[inline(always)]
        fn matches(cpu: &Cpu) -> bool {
            !cpu.is_flag_set(Flag::C)
        }
    }
}
