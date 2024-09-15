use std::ops::Deref;

use super::parts::P;

#[derive(Debug)]
pub enum Register8 {
    A = 7,
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    /// (HL), the byte at the mem address HL, **not** HL itself
    HL = 6,
}

impl From<u8> for Register8 {
    fn from(val: u8) -> Self {
        match val {
            0 => Register8::B,
            1 => Register8::C,
            2 => Register8::D,
            3 => Register8::E,
            4 => Register8::H,
            5 => Register8::L,
            6 => Register8::HL,
            7 => Register8::A,
            val => panic!("Tried to turn {val} into an R8"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

/// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
///
/// Allows converting [`P`] values into [`Register16`] values via 2 lookup tables: `rp` and `rp2`.
pub trait PIntoR16 {
    fn r16_via_rp(&self) -> Register16;
    fn r16_via_rp2(&self) -> Register16;
}

impl PIntoR16 for P {
    fn r16_via_rp(&self) -> Register16 {
        match self.deref() {
            0 => Register16::BC,
            1 => Register16::DE,
            2 => Register16::HL,
            3 => Register16::SP,
            _ => unreachable!(),
        }
    }

    fn r16_via_rp2(&self) -> Register16 {
        match self.deref() {
            0 => Register16::BC,
            1 => Register16::DE,
            2 => Register16::HL,
            3 => Register16::AF,
            _ => unreachable!(),
        }
    }
}
