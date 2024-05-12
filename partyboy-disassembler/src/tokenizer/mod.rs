use winnow::prelude::*;
use winnow::stream::Stream as _;
use winnow::{combinator::repeat, token::take};

use crate::opcode::Instruction;

use self::tokenize_x_0::tokenize_x_0;

mod tokenize_x_0;

trait OpcodeParts {
    fn x(&self) -> u8;
    fn y(&self) -> u8;
    fn z(&self) -> u8;
    fn p(&self) -> u8;
    fn q(&self) -> u8;
}

impl OpcodeParts for u8 {
    fn x(&self) -> u8 {
        self >> 6
    }

    fn y(&self) -> u8 {
        (self & 0b00111000) >> 3
    }

    fn z(&self) -> u8 {
        self & 0b00000111
    }

    fn p(&self) -> u8 {
        (self & 0b00110000) >> 4
    }

    fn q(&self) -> u8 {
        (self & 0b00001000) >> 3
    }
}

pub type Stream<'i> = &'i [(usize, u8)];

pub fn parse(data: &[u8]) -> PResult<Vec<Instruction>> {
    let enumerated = data.iter_offsets().collect::<Vec<_>>();
    let mut buf = enumerated.as_slice();
    repeat(0.., parse_opcode).parse_next(&mut buf)
}

fn parse_opcode(input: &mut Stream) -> PResult<Instruction> {
    let (offset, byte) = take(1usize)
        .map(|slice: &[(usize, u8)]| slice[0])
        .parse_next(input)?;

    match byte.x() {
        0 => tokenize_x_0(byte, offset, input), // TODO: call parse_next?
        1 => todo!(),
        2 => todo!(),
        3 => todo!(),
        _ => unreachable!(),
    }
}
