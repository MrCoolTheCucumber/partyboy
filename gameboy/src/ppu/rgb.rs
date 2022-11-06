use std::{cmp, fmt};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "web", wasm_bindgen)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl fmt::Debug for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Rgb {
    pub const fn const_mono(val: u8) -> Self {
        Self {
            r: val,
            g: val,
            b: val,
        }
    }

    pub(super) fn from_bgr555(bgr555: u16) -> Self {
        Self {
            r: (bgr555 & 0x1F) as u8,
            g: ((bgr555 >> 5) & 0x1F) as u8,
            b: ((bgr555 >> 10) & 0x1F) as u8,
        }
        .convert_555_to_888()
    }

    // see also: https://stackoverflow.com/questions/4409763/how-to-convert-from-rgb555-to-rgb888-in-c
    #[allow(dead_code)]
    fn convert_555_to_888(mut self) -> Self {
        self.r = (self.r << 3) | (self.r >> 2);
        self.g = (self.g << 3) | (self.g >> 2);
        self.b = (self.b << 3) | (self.b >> 2);

        self
    }

    #[allow(dead_code)]
    pub(super) fn byuu_correction(mut self) -> Self {
        self.r = cmp::min(240, ((26 * self.r + 4 * self.g + 2 * self.b) >> 2) as u8);
        self.g = cmp::min(240, ((24 * self.g + 8 * self.b) >> 2) as u8);
        self.b = cmp::min(240, ((6 * self.r + 4 * self.g + 22 * self.b) >> 2) as u8);

        self
    }
}
