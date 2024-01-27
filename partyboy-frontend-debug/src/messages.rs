use gameboy::debug::GBDebugInfo;
use gameboy::ppu::rgb::Rgb;

use crate::app::InputType;

pub enum MessageToGB {
    /// init with given rom path
    New(String),
    Start,
    Stop,
    KeyDown(Vec<InputType>),
    KeyUp(Vec<InputType>),
}

pub enum MessageFromGb {
    /// GB wants to draw frame with given frame buffer
    Draw(Vec<Rgb>),
    DebugInfo(Box<GBDebugInfo>),
}
