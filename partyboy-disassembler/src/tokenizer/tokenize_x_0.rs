use winnow::{token::take, PResult, Parser};

use crate::opcode::Opcode;

use super::{OpcodeParts, Stream};

pub fn tokenize_x_0(byte: u8, input: &mut Stream) -> PResult<Opcode> {
    match byte.z() {
        0 => tokenize_z_0(byte, input),
        1 => todo!(),
        2 => todo!(),
        3 => todo!(),
        4 => todo!(),
        5 => todo!(),
        6 => todo!(),
        7 => todo!(),
        _ => unreachable!(),
    }
}

fn tokenize_z_0(byte: u8, input: &mut Stream) -> PResult<Opcode> {
    match byte.y() {
        0 => Ok(Opcode::NOP),
        1 => take(2usize)
            .map(|bytes: &[u8]| {
                let hi = bytes[0] as u16;
                let lo = bytes[1] as u16;
                Opcode::LD_SP((hi << 8) | lo)
            })
            .parse_next(input),
        2 => Ok(Opcode::STOP),
        3 => take(1usize)
            .map(|bytes: &[u8]| Opcode::JR {
                cc: None,
                e8: bytes[0] as i8,
            })
            .parse_next(input),
        y @ 4..=7 => take(1usize)
            .map(|bytes: &[u8]| Opcode::JR {
                cc: Some((y - 4).into()),
                e8: bytes[0] as i8,
            })
            .parse_next(input),
        _ => unreachable!(),
    }
}
