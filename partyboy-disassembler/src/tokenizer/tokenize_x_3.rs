use winnow::{
    error::{ContextError, ErrMode},
    token::take,
    PResult, Parser as _,
};

use crate::opcode::{
    ArithmeticArg, Instruction, LoadArg, LoadHArg, Opcode, OpcodeParts as _, OpcodeVal, PIntoR16,
    Register8,
};

use super::{tokenize_cb_prefix::tokenize_cb_prefix, Stream};

/// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
///
/// Entrypoint for the "For `x = 3`" section of the decoding docs
pub fn tokenize_x_3(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.z() {
        0 => tokenize_z_0(byte, offset, input),
        1 => tokenize_z_1(byte, offset),
        2 => tokenize_z_2(byte, offset, input),
        3 => tokenize_z_3(byte, offset, input),
        4 => tokenize_z_4(byte, offset, input),
        5 => tokenize_z_5(byte, offset, input),
        6 => tokenize_z_6(byte, offset, input),
        7 => tokenize_z_7(byte, offset),
        _ => unreachable!(),
    }
}

fn tokenize_z_0(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.y() {
        cc @ 0..=3 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::RET(Some(cc.into())),
            span: offset.into(),
        }),

        y @ (4 | 6) => take(1usize)
            .map(|bytes: &[(usize, u8)]| {
                let mut src = LoadHArg::MEM_N16(0xFF00 + (bytes[0].1 as u16));
                let mut dest = LoadHArg::A;

                if y == 6 {
                    std::mem::swap(&mut src, &mut dest);
                }

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::LDH { src, dest },
                    span: (offset, bytes[0].0).into(),
                }
            })
            .parse_next(input),

        5 => take(1usize)
            .map(|bytes: &[(usize, u8)]| Instruction {
                val: OpcodeVal::Unprefixed(byte),
                opcode: Opcode::ADD_SP(bytes[0].1 as i8),
                span: (offset, bytes[0].0).into(),
            })
            .parse_next(input),

        7 => take(1usize)
            .map(|bytes: &[(usize, u8)]| Instruction {
                val: OpcodeVal::Unprefixed(byte),
                opcode: Opcode::LD_HL_SP_E8_OFFSET(bytes[0].1 as i8),
                span: (offset, bytes[0].0).into(),
            })
            .parse_next(input),

        _ => unreachable!(),
    }
}

fn tokenize_z_1(byte: u8, offset: usize) -> PResult<Instruction> {
    if *byte.q() == 0 {
        return Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::POP(byte.p().r16_via_rp2()),
            span: offset.into(),
        });
    }

    match *byte.p() {
        0 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::RET(None),
            span: offset.into(),
        }),

        1 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::RETI,
            span: offset.into(),
        }),

        2 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::JP_HL,
            span: offset.into(),
        }),

        3 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::LD_SP_HL,
            span: offset.into(),
        }),

        _ => unreachable!(),
    }
}

fn tokenize_z_2(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.y() {
        cc @ 0..=3 => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::JP {
                        cc: Some(cc.into()),
                        n16: (hi << 8) | lo,
                    },
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        y @ (4 | 6) => take(1usize)
            .map(|bytes: &[(usize, u8)]| {
                let mut src = LoadHArg::MEM_C;
                let mut dest = LoadHArg::A;

                if y == 6 {
                    std::mem::swap(&mut src, &mut dest);
                }

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::LDH { src, dest },
                    span: (offset, bytes[0].0).into(),
                }
            })
            .parse_next(input),

        y @ (5 | 7) => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                let mut src = LoadArg::N16((hi << 8) | lo);
                let mut dest = LoadArg::R8(Register8::A);

                if y == 7 {
                    std::mem::swap(&mut src, &mut dest);
                }

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::LD { src, dest },
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        _ => unreachable!(),
    }
}

fn tokenize_z_3(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.y() {
        0 => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::JP {
                        cc: None,
                        n16: (hi << 8) | lo,
                    },
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        1 => tokenize_cb_prefix(byte, offset),

        // 2..=5 are unimplemented opcodes. We can either fail with an unrecoverable error,
        // or we can tokenize into an "invalid opcode".
        //
        // TODO: How to add context?
        2..=5 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::Invalid(byte),
            span: offset.into(),
        }),

        6 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::DI,
            span: offset.into(),
        }),

        7 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::EI,
            span: offset.into(),
        }),

        _ => unreachable!(),
    }
}

fn tokenize_z_4(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.y() {
        y @ 0..=3 => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::CALL {
                        cc: Some(y.into()),
                        n16: (hi << 8) | lo,
                    },
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        4..=7 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::Invalid(byte),
            span: offset.into(),
        }),

        _ => unreachable!(),
    }
}

fn tokenize_z_5(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    if *byte.q() == 0 {
        return Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::PUSH(byte.p().r16_via_rp2()),
            span: offset.into(),
        });
    }

    if *byte.q() == 1 && *byte.p() == 0 {
        return take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::CALL {
                        cc: None,
                        n16: (hi << 8) | lo,
                    },
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input);
    }

    // p 1..=3 are removed opcodes
    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode: Opcode::Invalid(byte),
        span: offset.into(),
    })
}

fn tokenize_z_6(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    take(1usize)
        .map(|bytes: &[(usize, u8)]| {
            let arg = ArithmeticArg::Constant(bytes[0].1);

            let opcode = match *byte.y() {
                0 => Opcode::ADD(arg),
                1 => Opcode::ADC(arg),
                2 => Opcode::SUB(arg),
                3 => Opcode::SBC(arg),
                4 => Opcode::AND(arg),
                5 => Opcode::XOR(arg),
                6 => Opcode::OR(arg),
                7 => Opcode::CP(arg),
                _ => unreachable!(),
            };

            Instruction {
                val: OpcodeVal::Unprefixed(byte),
                opcode,
                span: (offset, bytes[0].0).into(),
            }
        })
        .parse_next(input)
}

fn tokenize_z_7(byte: u8, offset: usize) -> PResult<Instruction> {
    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode: Opcode::RST((*byte.y() as u16) * 8),
        span: offset.into(),
    })
}
