use crate::{bus::Bus, GameBoy};
use thiserror::Error;

pub type SerialWriteHandler = Box<dyn FnMut(u8)>;

#[derive(Error, Debug)]
#[cfg(not(feature = "web"))]
pub enum GameBoyBuilderError {
    #[error("A rom path must be specified")]
    NoRomPath,
}

#[cfg(not(feature = "web"))]
pub struct GameBoyBuilder {
    rom: Option<Vec<u8>>,
    ram: Option<Vec<u8>>,
    serial_write_handler: Option<SerialWriteHandler>,
}

#[cfg(not(feature = "web"))]
impl Default for GameBoyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "web"))]
impl GameBoyBuilder {
    pub fn new() -> Self {
        GameBoyBuilder {
            rom: None,
            ram: None,
            serial_write_handler: None,
        }
    }

    pub fn rom(self, rom: Vec<u8>) -> Self {
        let mut builder = self;
        builder.rom = Some(rom);
        builder
    }

    pub fn ram(self, ram: Vec<u8>) -> Self {
        let mut builder = self;
        builder.ram = Some(ram);
        builder
    }

    pub fn serial_write_handler(self, on_serial_write: SerialWriteHandler) -> Self {
        let mut builder = self;
        builder.serial_write_handler = Some(on_serial_write);
        builder
    }

    pub fn build(self) -> Result<GameBoy, GameBoyBuilderError> {
        if self.rom.is_none() {
            return Err(GameBoyBuilderError::NoRomPath);
        }
        let serial_write_handler = self
            .serial_write_handler
            .unwrap_or_else(|| Box::new(Bus::get_handle_blargg_output()));

        Ok(GameBoy::new(
            self.rom.unwrap(),
            self.ram,
            serial_write_handler,
        ))
    }
}
