use gameboy::gameboy::GameBoy;
use sdl2::keyboard::Keycode;

fn try_into_gameboy_input(code: Keycode) -> Option<gameboy::gameboy::input::Keycode> {
    match code {
        Keycode::W => Some(gameboy::gameboy::input::Keycode::Up),
        Keycode::A => Some(gameboy::gameboy::input::Keycode::Left),
        Keycode::S => Some(gameboy::gameboy::input::Keycode::Down),
        Keycode::D => Some(gameboy::gameboy::input::Keycode::Right),

        Keycode::O => Some(gameboy::gameboy::input::Keycode::A),
        Keycode::K => Some(gameboy::gameboy::input::Keycode::B),

        Keycode::M => Some(gameboy::gameboy::input::Keycode::Start),
        Keycode::N => Some(gameboy::gameboy::input::Keycode::Select),

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
