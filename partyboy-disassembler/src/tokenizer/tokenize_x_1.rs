use winnow::PResult;

use crate::opcode::{Instruction, LoadArg, Opcode, OpcodeParts, OpcodeVal, Register8};

/// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
///
/// Entrypoint for the "For `x = 1`" section of the decoding docs
pub fn tokenize_x_1(byte: u8, offset: usize) -> PResult<Instruction> {
    if *byte.z() == 6 && *byte.y() == 6 {
        return Ok(Instruction {
            val: OpcodeVal::Unprefixed(byte),
            opcode: Opcode::HALT,
            span: offset.into(),
        });
    }

    let src = Register8::from(*byte.z());
    let dest = Register8::from(*byte.y());

    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode: Opcode::LD {
            src: LoadArg::R8(src),
            dest: LoadArg::R8(dest),
        },
        span: offset.into(),
    })
}
