use crate::{bus::Bus, GameBoy};
use thiserror::Error;

pub type SerialWriteHandler = Box<dyn FnMut(u8)>;

#[derive(Error, Debug)]
pub enum GameBoyBuilderError {
    #[error("A rom path must be specified")]
    NoRomPath,
}

pub struct GameBoyBuilder {
    rom_path: Option<String>,
    serial_write_handler: Option<SerialWriteHandler>,
}

impl Default for GameBoyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GameBoyBuilder {
    pub fn new() -> Self {
        GameBoyBuilder {
            rom_path: None,
            serial_write_handler: None,
        }
    }

    pub fn rom_path(self, rom_path: &str) -> Self {
        let mut builder = self;
        builder.rom_path = Some(rom_path.to_owned());
        builder
    }

    pub fn serial_write_handler(self, on_serial_write: SerialWriteHandler) -> Self {
        let mut builder = self;
        builder.serial_write_handler = Some(on_serial_write);
        builder
    }

    pub fn build(self) -> Result<GameBoy, GameBoyBuilderError> {
        if self.rom_path.is_none() {
            return Err(GameBoyBuilderError::NoRomPath);
        }

        let serial_write_handler = self
            .serial_write_handler
            .unwrap_or_else(|| Box::new(Bus::get_handle_blargg_output()));

        Ok(GameBoy::new(
            self.rom_path.unwrap().as_str(),
            serial_write_handler,
        ))
    }
}
