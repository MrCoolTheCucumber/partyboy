mod register;

use self::register::Register;

use super::bus::Bus;

struct Cpu {
    af: Register,
    bc: Register,
    de: Register,
    hl: Register,

    pc: u16,
    sp: u16,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            af: Register::new(0, 0),
            bc: Register::new(0, 0),
            de: Register::new(0, 0),
            hl: Register::new(0, 0),

            pc: 0x0,
            sp: 0x0,
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {}
}
