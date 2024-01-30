use partyboy_core::{input::Keycode, ppu::rgb::Rgb};

pub enum MsgFromGb {
    Frame(Vec<Rgb>),
    Fps(f64),
}

pub enum MsgToGb {
    #[allow(dead_code)]
    Load,
    KeyDown(Keycode),
    KeyUp(Keycode),
    Turbo(bool),
    Rewind(bool),

    SaveSnapshot,
    LoadSnapshot,

    Shutdown,
}
