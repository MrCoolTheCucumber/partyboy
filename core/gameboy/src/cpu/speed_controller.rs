use crate::bus::CgbCompatibility;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum CpuSpeedMode {
    Single = 0,
    Double = 1,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct CpuSpeedController {
    cpu_speed_mode: CpuSpeedMode,
    prepare_speed_switch: bool,
    cgb_compatibility: CgbCompatibility,
}

impl CpuSpeedController {
    pub fn new(cgb_compatibility: CgbCompatibility) -> Self {
        Self {
            cpu_speed_mode: CpuSpeedMode::Single,
            prepare_speed_switch: false,
            cgb_compatibility,
        }
    }

    pub fn is_double_speed(&self) -> bool {
        matches!(self.cpu_speed_mode, CpuSpeedMode::Double)
    }

    pub fn set_console_compatibility(&mut self, cgb_compatibility: CgbCompatibility) {
        self.cgb_compatibility = cgb_compatibility;
    }

    pub fn set_prepare_speed_switch(&mut self, set: bool) {
        if set {
            log::debug!("Cpu speed switch prepared");
        }
        self.prepare_speed_switch = set;
    }

    pub fn is_speed_switch_prepared(&self) -> bool {
        self.prepare_speed_switch
    }

    pub fn switch_speed(&mut self) {
        debug_assert!(self.prepare_speed_switch);
        self.prepare_speed_switch = false;
        self.cpu_speed_mode = match self.cpu_speed_mode {
            CpuSpeedMode::Single => CpuSpeedMode::Double,
            CpuSpeedMode::Double => CpuSpeedMode::Single,
        };
    }

    pub fn read_key1(&self) -> u8 {
        ((self.cpu_speed_mode as u8) << 7) | (self.prepare_speed_switch as u8) | 0b0111_1110
    }
}
