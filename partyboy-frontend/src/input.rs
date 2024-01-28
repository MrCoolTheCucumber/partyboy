use partyboy_core::input::Keycode;
use winit::keyboard::Key;

pub fn try_into_gameboy_input(key: Key<&str>) -> Option<Keycode> {
    match key {
        Key::Character("w") => Some(Keycode::Up),
        Key::Character("a") => Some(Keycode::Left),
        Key::Character("s") => Some(Keycode::Down),
        Key::Character("d") => Some(Keycode::Right),

        Key::Character("o") => Some(Keycode::A),
        Key::Character("k") => Some(Keycode::B),

        Key::Character("m") => Some(Keycode::Start),
        Key::Character("n") => Some(Keycode::Select),

        _ => None,
    }
}
