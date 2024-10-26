use winnow::PResult;

use crate::opcode::{BitArg, Instruction, Opcode, OpcodeParts, OpcodeVal, Register8};

/// https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
///
/// Entrypoint for the "CB-PREFIXED OPCODES" section of the decoding docs
pub(super) fn tokenize_cb_prefix(byte: u8, offset: usize) -> PResult<Instruction> {
    match *byte.x() {
        0 => {
            let opcode = match *byte.y() {
                0 => Opcode::RLC((*byte.z()).into()),
                1 => Opcode::RRC((*byte.z()).into()),
                2 => Opcode::RL((*byte.z()).into()),
                3 => Opcode::RR((*byte.z()).into()),
                4 => Opcode::SLA((*byte.z()).into()),
                5 => Opcode::SRA((*byte.z()).into()),
                6 => Opcode::SWAP((*byte.z()).into()),
                7 => Opcode::SRL((*byte.z()).into()),
                _ => unreachable!(),
            };

            Ok(Instruction {
                val: OpcodeVal::Prefixed(byte),
                opcode,
                span: offset.into(),
            })
        }

        x @ 1..=3 => {
            let big_arg = BitArg {
                bit: *byte.y(),
                register: Register8::from(*byte.z()),
            };

            let opcode = match x {
                1 => Opcode::BIT(big_arg),
                2 => Opcode::RES(big_arg),
                3 => Opcode::SET(big_arg),
                _ => unreachable!(),
            };

            Ok(Instruction {
                val: OpcodeVal::Prefixed(byte),
                opcode,
                span: offset.into(),
            })
        }

        _ => unreachable!(),
    }
}
