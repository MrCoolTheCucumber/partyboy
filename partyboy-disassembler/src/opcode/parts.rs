use std::ops::Deref;

/// See https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
/// for more context.
///
/// A trait for extracting the `x`, `y`, `z`, `p` and `q` values from a byte.
pub trait OpcodeParts {
    fn x(&self) -> X;
    fn y(&self) -> Y;
    fn z(&self) -> Z;
    fn p(&self) -> P;
    fn q(&self) -> Q;
}

impl OpcodeParts for u8 {
    fn x(&self) -> X {
        X(self >> 6)
    }

    fn y(&self) -> Y {
        Y((self & 0b00111000) >> 3)
    }

    fn z(&self) -> Z {
        Z(self & 0b00000111)
    }

    fn p(&self) -> P {
        P((self & 0b00110000) >> 4)
    }

    fn q(&self) -> Q {
        Q((self & 0b00001000) >> 3)
    }
}

macro_rules! define_opcode_part_microtype {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name(u8);

        impl Deref for $name {
            type Target = u8;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

define_opcode_part_microtype!(X);
define_opcode_part_microtype!(Y);
define_opcode_part_microtype!(Z);
define_opcode_part_microtype!(P);
define_opcode_part_microtype!(Q);
