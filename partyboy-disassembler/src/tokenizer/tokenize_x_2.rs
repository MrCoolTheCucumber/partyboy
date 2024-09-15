use winnow::PResult;

use crate::opcode::{ArithmeticArg, Instruction, Opcode, OpcodeParts, OpcodeVal, Register8};

/// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
///
/// Entrypoint for the "For `x = 2`" section of the decoding docs
pub fn tokenize_x_2(byte: u8, offset: usize) -> PResult<Instruction> {
    let reg = Register8::from(*byte.z());
    let arg = ArithmeticArg::Register(reg);

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

    Ok(Instruction {
        val: OpcodeVal::Unprefixed(byte),
        opcode,
        span: offset.into(),
    })
}
