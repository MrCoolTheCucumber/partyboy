//! https://rgbds.gbdev.io/docs/v0.5.1/gbz80.7
#![allow(clippy::upper_case_acronyms, non_camel_case_types)]

mod condition_code;
mod parts;
mod register;
mod span;

pub use condition_code::ConditionCode;
pub use parts::OpcodeParts;
pub use register::{PIntoR16, Register16, Register8};
pub use span::Span;

#[derive(Debug)]
pub struct Instruction {
    pub val: OpcodeVal,
    pub opcode: Opcode,
    pub span: Span,
}

#[derive(Debug)]
pub enum ArithmeticArg {
    Register(Register8),
    Constant(u8),
}

#[derive(Debug)]
pub enum IncDecArg {
    R8(Register8),
    R16(Register16),
    SP,
}

#[derive(Debug)]
pub struct BitArg {
    /// A 3 bit unsigned number (0 - 7) representing the particular bit tod
    /// perform the operation on
    bit: u8,
    register: Register8,
}

#[derive(Debug)]
pub enum LoadArg {
    R8(Register8),
    R16(Register16),
    N8(u8),
    N16(u16),
    MEM_R8(Register8),
    MEM_R16(Register16),
    MEM_HLI,
    MEM_HLD,
}

#[derive(Debug)]
pub enum LoadHArg {
    MEM_C,
    A,
    MEM_N16(u16),
}

/// See: https://rgbds.gbdev.io/docs/v0.5.1/gbz80.7#INSTRUCTION_OVERVIEW
#[derive(Debug)]
pub enum Opcode {
    // 8-bit arithmetic and logic
    ADC(ArithmeticArg),
    ADD(ArithmeticArg),
    AND(ArithmeticArg),
    CP(ArithmeticArg),
    OR(ArithmeticArg),
    SBC(ArithmeticArg),
    SUB(ArithmeticArg),
    XOR(ArithmeticArg),
    INC(IncDecArg),
    DEC(IncDecArg),

    // 16-bit arithmetic
    // INC/DEC r16 included above
    ADD_HL(Register16),

    // Bit Operations
    BIT(BitArg),
    RES(BitArg),
    SET(BitArg),
    SWAP(Register8),

    // Bit Shift
    RL(Register8),
    RLA,
    RLC(Register8),
    RLCA,
    RR(Register8),
    RRA,
    RRC(Register8),
    RRCA,
    SLA(Register8),
    SRA(Register8),
    SRL(Register8),

    // Load Instructions
    LD { src: LoadArg, dest: LoadArg },
    LDH { src: LoadHArg, dest: LoadHArg },

    // Jump and Subroutines
    CALL { cc: Option<ConditionCode>, n16: u16 },
    JP { cc: Option<ConditionCode>, n16: u16 },
    JP_HL,
    JR { cc: Option<ConditionCode>, e8: i8 },
    RET(Option<ConditionCode>),
    RETI,
    RST(u16),

    // Stack Operations
    ADD_HL_SP,
    ADD_SP(i8),
    // INC/DEC SP Handled above
    LD_SP(u16),
    LD_MEM_U16_SP(u16),
    LD_HL_SP_E8_OFFSET(i8),
    LD_SP_HL,
    POP_AF,
    POP(Register16),
    PUSH_AF,
    PUSH(Register16),

    // Misc
    CCF,
    CPL,
    DAA,
    DI,
    EI,
    HALT,
    NOP,
    SCF,
    STOP,
}

#[derive(Debug)]
pub enum OpcodeVal {
    Prefixed(u8),
    Unprefixed(u8),
}
