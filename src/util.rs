macro_rules! is_flag_set {
    ($val:expr, $flag:expr) => {
        $val & $flag as u8 == 1
    };
}

pub(crate) use is_flag_set;

macro_rules! is_flag_unset {
    ($val:expr, $flag:expr) => {
        $val & $flag as u8 == 0
    };
}

pub(crate) use is_flag_unset;
