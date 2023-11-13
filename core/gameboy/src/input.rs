use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq)]
pub struct Input {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
    pub start: u8,
    pub select: u8,
    pub a: u8,
    pub b: u8,

    column_line: u8,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            up: 1,
            down: 1,
            left: 1,
            right: 1,
            start: 1,
            select: 1,
            a: 1,
            b: 1,

            column_line: 0x30,
        }
    }

    pub fn set_column_line(&mut self, val: u8) {
        self.column_line = val & 0b0011_0000;
    }

    pub fn read_joyp(&self) -> u8 {
        let joyp = match self.column_line {
            // 4th bit
            0x10 => {
                let mut result = self.a;
                result |= self.b << 1;
                result |= self.select << 2;
                result |= self.start << 3;
                result
            }

            // 5th bit
            0x20 => {
                let mut result = self.right;
                result |= self.left << 1;
                result |= self.up << 2;
                result |= self.down << 3;
                result
            }

            // 4th & 5th
            0x30 => {
                self.a
                    | (self.b << 1)
                    | (self.select << 2)
                    | (self.start << 3)
                    | (self.left << 1)
                    | (self.up << 2)
                    | (self.down << 3)
            }

            _ => 0,
        };

        joyp | 0b1100_0000
    }

    pub fn key_down(&mut self, code: Keycode) -> bool {
        match code {
            Keycode::Up => self.up = 0,
            Keycode::Left => self.left = 0,
            Keycode::Down => self.down = 0,
            Keycode::Right => self.right = 0,
            Keycode::A => self.a = 0,
            Keycode::B => self.b = 0,
            Keycode::Select => self.select = 0,
            Keycode::Start => self.start = 0,
        };

        true
    }

    pub fn key_up(&mut self, code: Keycode) {
        match code {
            Keycode::Up => self.up = 1,
            Keycode::Left => self.left = 1,
            Keycode::Down => self.down = 1,
            Keycode::Right => self.right = 1,
            Keycode::A => self.a = 1,
            Keycode::B => self.b = 1,
            Keycode::Select => self.select = 1,
            Keycode::Start => self.start = 1,
        }
    }

    pub fn release_all_keys(&mut self) {
        self.up = 1;
        self.left = 1;
        self.down = 1;
        self.right = 1;
        self.a = 1;
        self.b = 1;
        self.select = 1;
        self.start = 1;
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "web", wasm_bindgen)]
pub enum Keycode {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

impl Display for Keycode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keycode::Up => write!(f, "Up"),
            Keycode::Down => write!(f, "Down"),
            Keycode::Left => write!(f, "Left"),
            Keycode::Right => write!(f, "Right"),
            Keycode::A => write!(f, "A"),
            Keycode::B => write!(f, "B"),
            Keycode::Start => write!(f, "Start"),
            Keycode::Select => write!(f, "Select"),
        }
    }
}
