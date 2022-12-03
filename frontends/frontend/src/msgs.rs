use gameboy::{input::Keycode, ppu::rgb::Rgb};

pub enum MsgFromGb {
    Frame(Vec<Rgb>),
    Fps(f64),
}

pub enum MsgToGb {
    #[allow(dead_code)]
    Load,
    KeyDown(Vec<Keycode>),
    KeyUp(Vec<Keycode>),
    Turbo(bool),

    SaveSnapshot,
    LoadSnapshot,
}
