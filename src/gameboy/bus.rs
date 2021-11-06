use super::cartridge::Cartridge;

pub struct Bus {
    cartridge: Box<dyn Cartridge>,
}
