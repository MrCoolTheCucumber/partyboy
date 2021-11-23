use gameboy::GameBoy;
use sdl2::keyboard::Keycode;

fn try_into_gameboy_input(code: Keycode) -> Option<gameboy::input::Keycode> {
    match code {
        Keycode::W => Some(gameboy::input::Keycode::Up),
        Keycode::A => Some(gameboy::input::Keycode::Left),
        Keycode::S => Some(gameboy::input::Keycode::Down),
        Keycode::D => Some(gameboy::input::Keycode::Right),

        Keycode::O => Some(gameboy::input::Keycode::A),
        Keycode::K => Some(gameboy::input::Keycode::B),

        Keycode::M => Some(gameboy::input::Keycode::Start),
        Keycode::N => Some(gameboy::input::Keycode::Select),

        _ => None,
    }
}

pub fn handle_key_down(gb: &mut GameBoy, code: Keycode) {
    match try_into_gameboy_input(code) {
        Some(key) => gb.key_down(key),
        None => {}
    }
}

pub fn handle_key_up(gb: &mut GameBoy, code: Keycode) {
    match try_into_gameboy_input(code) {
        Some(key) => gb.key_up(key),
        None => {}
    }
}
