use winnow::prelude::*;
use winnow::stream::Stream as _;
use winnow::{combinator::repeat, token::take};

use crate::opcode::{Instruction, OpcodeParts as _};

use self::tokenize_x_0::tokenize_x_0;
use self::tokenize_x_1::tokenize_x_1;
use self::tokenize_x_2::tokenize_x_2;

mod tokenize_x_0;
mod tokenize_x_1;
mod tokenize_x_2;

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

    match *byte.x() {
        0 => tokenize_x_0(byte, offset, input),
        1 => tokenize_x_1(byte, offset),
        2 => tokenize_x_2(byte, offset),
        3 => todo!(),
        _ => unreachable!(),
    }
}
