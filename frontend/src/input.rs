use gameboy::input::Keycode;
use winit::event::VirtualKeyCode;
use winit_input_helper::WinitInputHelper;

const GB_KEYS: [VirtualKeyCode; 8] = [
    VirtualKeyCode::W,
    VirtualKeyCode::A,
    VirtualKeyCode::S,
    VirtualKeyCode::D,
    VirtualKeyCode::O,
    VirtualKeyCode::K,
    VirtualKeyCode::M,
    VirtualKeyCode::N,
];

pub fn try_into_gameboy_input(key: VirtualKeyCode) -> Option<Keycode> {
    match key {
        VirtualKeyCode::W => Some(gameboy::input::Keycode::Up),
        VirtualKeyCode::A => Some(gameboy::input::Keycode::Left),
        VirtualKeyCode::S => Some(gameboy::input::Keycode::Down),
        VirtualKeyCode::D => Some(gameboy::input::Keycode::Right),

        VirtualKeyCode::O => Some(gameboy::input::Keycode::A),
        VirtualKeyCode::K => Some(gameboy::input::Keycode::B),

        VirtualKeyCode::M => Some(gameboy::input::Keycode::Start),
        VirtualKeyCode::N => Some(gameboy::input::Keycode::Select),

        _ => None,
    }
}

pub fn get_key_downs(input: &mut WinitInputHelper) -> Vec<Keycode> {
    GB_KEYS
        .iter()
        .copied()
        .filter(|key| input.key_pressed(*key))
        .filter_map(try_into_gameboy_input)
        .collect()
}

pub fn get_key_ups(input: &mut WinitInputHelper) -> Vec<Keycode> {
    GB_KEYS
        .iter()
        .copied()
        .filter(|key| input.key_released(*key))
        .filter_map(try_into_gameboy_input)
        .collect()
}
