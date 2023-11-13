use std::{fmt::Debug, ops};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum Flag {
    Z = 0b10000000,
    N = 0b01000000,
    H = 0b00100000,
    C = 0b00010000,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Register {
    pub hi: u8,
    pub lo: u8,
}

impl Register {
    #[inline(always)]
    pub fn new(hi: u8, lo: u8) -> Self {
        Self { hi, lo }
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val: u16 = (*self).into();
        f.write_str(format!("{}", val).as_str())
    }
}

impl From<Register> for u16 {
    #[inline(always)]
    fn from(reg: Register) -> Self {
        ((reg.hi as u16) << 8) | (reg.lo as u16)
    }
}

impl From<u16> for Register {
    #[inline(always)]
    fn from(val: u16) -> Self {
        Register {
            hi: ((val & 0xFF00) >> 8) as u8,
            lo: (val & 0x00FF) as u8,
        }
    }
}

impl ops::AddAssign<u16> for Register {
    #[inline(always)]
    fn add_assign(&mut self, rhs: u16) {
        let val: u16 = u16::from(*self).wrapping_add(rhs);
        self.hi = ((val & 0xFF00) >> 8) as u8;
        self.lo = (val & 0x00FF) as u8;
    }
}

impl ops::SubAssign<u16> for Register {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: u16) {
        let val: u16 = u16::from(*self).wrapping_sub(rhs);
        self.hi = ((val & 0xFF00) >> 8) as u8;
        self.lo = (val & 0x00FF) as u8;
    }
}
