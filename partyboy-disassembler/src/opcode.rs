//! https://rgbds.gbdev.io/docs/v0.5.1/gbz80.7

#![allow(clippy::upper_case_acronyms, non_camel_case_types)]

#[derive(Debug)]
pub enum Register8 {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    /// The byte at the mem address HL, **not** HL itself
    HL,
}

#[derive(Debug, Clone, Copy)]
pub enum Register16 {
    BC,
    DE,
    HL,
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

#[derive(Debug)]
pub enum ConditionCode {
    Z = 1,
    NZ = 0,
    C = 3,
    NC = 2,
}

impl From<u8> for ConditionCode {
    fn from(value: u8) -> Self {
        assert!(value < 4);
        match value {
            0 => ConditionCode::NZ,
            1 => ConditionCode::Z,
            2 => ConditionCode::NC,
            3 => ConditionCode::C,
            _ => unreachable!(),
        }
    }
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

#[derive(Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl From<usize> for Span {
    fn from(value: usize) -> Self {
        Self {
            start: value,
            end: value + 1,
        }
    }
}

impl From<(usize, usize)> for Span {
    fn from((a, b): (usize, usize)) -> Self {
        Self {
            start: a,
            end: b + 1,
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub val: OpcodeVal,
    pub opcode: Opcode,
    pub span: Span,
}
