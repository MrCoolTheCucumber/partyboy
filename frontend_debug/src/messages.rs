use gameboy::input::Keycode;
use gameboy::ppu::rgb::Rgb;

pub enum MessageToGB {
    /// init with given rom path
    New(String),
    Start,
    Stop,
    KeyDown(Vec<Keycode>),
    KeyUp(Vec<Keycode>),
}

pub enum MessageFromGb {
    /// GB wants to draw frame with given frame buffer
    Draw(Vec<Rgb>),
}
