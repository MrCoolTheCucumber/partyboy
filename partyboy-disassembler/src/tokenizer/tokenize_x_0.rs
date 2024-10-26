use std::mem;

use winnow::{token::take, PResult, Parser};

use crate::opcode::{
    IncDecArg, Instruction, LoadArg, Opcode, OpcodeParts as _, OpcodeVal, PIntoR16, Register16,
    Register8,
};

use super::Stream;

/// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
///
/// Entrypoint for the "For `x = 0`" section of the decoding docs
pub fn tokenize_x_0(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.z() {
        0 => tokenize_z_0(byte, offset, input),
        1 => tokenize_z_1(byte, offset, input),
        2 => tokenize_z_2(byte, offset),
        3 => tokenize_z_3(byte, offset),
        4 => tokenize_z_4(byte, offset),
        5 => tokenize_z_5(byte, offset),
        6 => tokenize_z_6(byte, offset, input),
        7 => tokenize_z_7(byte, offset),
        _ => unreachable!(),
    }
}

fn tokenize_z_0(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    match *byte.y() {
        0 => Ok(Instruction {
            opcode: Opcode::NOP,
            val: OpcodeVal::Unprefixed(byte),
            span: offset.into(),
        }),

        1 => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::LD_SP((hi << 8) | lo),
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        2 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::STOP,
            span: offset.into(),
        }),

        3 => take(1usize)
            .map(|bytes: &[(usize, u8)]| Instruction {
                val: OpcodeVal::Unprefixed(byte),
                opcode: Opcode::JR {
                    cc: None,
                    e8: bytes[0].1 as i8,
                },
                span: (offset, bytes[0].0).into(),
            })
            .parse_next(input),

        y @ 4..=7 => take(1usize)
            .map(|bytes: &[(usize, u8)]| Instruction {
                val: OpcodeVal::Unprefixed(byte),
                opcode: Opcode::JR {
                    cc: Some((y - 4).into()),
                    e8: bytes[0].1 as i8,
                },
                span: (offset, bytes[0].0).into(),
            })
            .parse_next(input),

        _ => unreachable!(),
    }
}

fn tokenize_z_1(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    let reg = byte.p().r16_via_rp();

    match *byte.q() {
        0 if *byte.p() == 3 => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::LD_SP((hi << 8) | lo),
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        0 => take(2usize)
            .map(|bytes: &[(usize, u8)]| {
                let hi = bytes[0].1 as u16;
                let lo = bytes[1].1 as u16;

                let dest = LoadArg::R16(reg);
                let src = LoadArg::N16((hi << 8) | lo);

                Instruction {
                    val: OpcodeVal::Unprefixed(byte),
                    opcode: Opcode::LD { src, dest },
                    span: (offset, bytes[1].0).into(),
                }
            })
            .parse_next(input),

        1 if *byte.p() == 3 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::ADD_HL_SP,
            span: offset.into(),
        }),

        1 => Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::ADD_HL(reg),
            span: offset.into(),
        }),

        _ => unreachable!(),
    }
}

fn tokenize_z_2(byte: u8, offset: usize) -> PResult<Instruction> {
    let mut dest = match *byte.p() {
        0 => LoadArg::MEM_R16(Register16::BC),
        1 => LoadArg::MEM_R16(Register16::DE),
        2 => LoadArg::MEM_HLI,
        3 => LoadArg::MEM_HLD,
        _ => unreachable!(),
    };
    let mut src = LoadArg::R8(Register8::A);

    if *byte.q() == 1 {
        mem::swap(&mut src, &mut dest);
    };

    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode: Opcode::LD { src, dest },
        span: offset.into(),
    })
}

fn tokenize_z_3(byte: u8, offset: usize) -> PResult<Instruction> {
    let arg = match *byte.p() {
        0 => IncDecArg::R16(Register16::BC),
        1 => IncDecArg::R16(Register16::DE),
        2 => IncDecArg::R16(Register16::HL),
        3 => IncDecArg::SP,
        _ => unreachable!(),
    };

    let opcode = match *byte.q() {
        0 => Opcode::INC(arg),
        1 => Opcode::DEC(arg),
        _ => unreachable!(),
    };

    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode,
        span: offset.into(),
    })
}

fn tokenize_z_4(byte: u8, offset: usize) -> PResult<Instruction> {
    let register8: Register8 = (*byte.y()).into();
    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode: Opcode::INC(IncDecArg::R8(register8)),
        span: offset.into(),
    })
}

fn tokenize_z_5(byte: u8, offset: usize) -> PResult<Instruction> {
    let register8: Register8 = (*byte.y()).into();
    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode: Opcode::DEC(IncDecArg::R8(register8)),
        span: offset.into(),
    })
}

fn tokenize_z_6(byte: u8, offset: usize, input: &mut Stream) -> PResult<Instruction> {
    // TODO: return an actual winnow error
    take(1usize)
        .map(|bytes: &[(usize, u8)]| {
            let dest_reg8: Register8 = (*byte.y()).into();
            let opcode = Opcode::LD {
                src: LoadArg::N8(bytes[0].1),
                dest: LoadArg::R8(dest_reg8),
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
    let val = OpcodeVal::Unprefixed(byte);
    let span = offset.into();

    let opcode = match *byte.y() {
        0 => Opcode::RLCA,
        1 => Opcode::RRCA,
        2 => Opcode::RLA,
        3 => Opcode::RRA,
        4 => Opcode::DAA,
        5 => Opcode::CPL,
        6 => Opcode::SCF,
        7 => Opcode::CCF,
        _ => unreachable!(),
    };

    Ok(Instruction { val, opcode, span })
}
