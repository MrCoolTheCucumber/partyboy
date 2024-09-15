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
