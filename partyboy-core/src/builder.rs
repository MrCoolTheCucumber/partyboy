use crate::{
    bus::{Bus, CgbCompatibility},
    cartridge::Cartridge,
    ppu::{cgb_palette, rgb::Rgb, ObjectPriorityMode},
    GameBoy,
};
use thiserror::Error;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::wasm_bindgen;

pub type SerialWriteHandler = Box<dyn FnMut(u8)>;

#[derive(Error, Debug)]
#[cfg_attr(feature = "web", wasm_bindgen)]
pub enum GameBoyBuilderError {
    #[error("A rom path must be specified")]
    NoRomPath,
    #[error("Unable to parse bios file. Is it a CGB bios?")]
    UnableToParseBios,
    #[error("Internal error: unable to parse bios skip snapshot")]
    UnableToLoadBiosSkipSnapshot,
}

#[cfg_attr(feature = "web", wasm_bindgen)]
pub struct GameBoyBuilder {
    rom: Option<Vec<u8>>,
    ram: Option<Vec<u8>>,
    bios: Option<Vec<u8>>,
    serial_write_handler: Option<SerialWriteHandler>,
}

impl Default for GameBoyBuilder {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg_attr(feature = "web", wasm_bindgen)]
impl GameBoyBuilder {
    pub fn new() -> Self {
        GameBoyBuilder {
            rom: None,
            ram: None,
            serial_write_handler: None,
            bios: None,
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

    #[cfg(not(feature = "web"))]
    pub fn serial_write_handler(self, on_serial_write: SerialWriteHandler) -> Self {
        let mut builder = self;
        builder.serial_write_handler = Some(on_serial_write);
        builder
    }

    /// Must be a gameboy color bios?
    //  TODO: what happens if DMG bios is used
    pub fn bios(self, bios: Vec<u8>) -> Self {
        let mut builder = self;
        builder.bios = Some(bios);
        builder
    }

    fn create_gameboy_from_snapshot(self) -> Result<GameBoy, GameBoyBuilderError> {
        log::info!("SKIPPING BIOS VIA SNAPSHOT");
        let bios_skip_snapshot = include_bytes!("../../bin/bios_skip_snapshot.bin");
        let mut gb: GameBoy = rmp_serde::from_slice(bios_skip_snapshot)
            .map_err(|_| GameBoyBuilderError::UnableToLoadBiosSkipSnapshot)?;
        let cartridge = self.rom.map(|rom| Cartridge::new(rom, self.ram));

        gb.bus.ppu.gpu_vram[0].iter_mut().for_each(|x| *x = 0);
        gb.bus.ppu.gpu_vram[1].iter_mut().for_each(|x| *x = 0);
        gb.bus.ppu.sprite_table.iter_mut().for_each(|x| *x = 0);
        gb.bus
            .ppu
            .frame_buffer
            .iter_mut()
            .for_each(|x| *x = Rgb::default());
        gb.cpu.handle_bios_skip();
        gb.bus.bios_enabled = false;

        // Set compatibility mode
        let cart_header_cgb_flag = match &cartridge {
            Some(cartridge) => cartridge.read_rom(0x143),
            None => 0,
        };
        gb.bus.write_u8(0xFF4C, cart_header_cgb_flag);

        let compatibility = CgbCompatibility::from(cart_header_cgb_flag);
        if matches!(
            compatibility,
            CgbCompatibility::None | CgbCompatibility::CgbAndDmg
        ) {
            if let Some(cartridge) = &cartridge {
                // unwrap: get_color_palettes(..) returns an array of 12 too
                let palettes: [Rgb; 12] = cgb_palette::get_color_palettes(cartridge)
                    .into_iter()
                    .map(Rgb::from_rgb32)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();

                gb.bus.ppu.override_color_palettes(&palettes);
            }
        }

        // TODO: is this set also for CgbAndDmg mode?
        let obj_prio_mode = match compatibility {
            CgbCompatibility::CgbOnly => ObjectPriorityMode::OamOrder,
            _ => ObjectPriorityMode::CoordinateOrder,
        };
        gb.bus.ppu.override_obj_prio_mode(obj_prio_mode);

        if let Some(serial_write_handler) = self.serial_write_handler {
            gb.bus.set_serial_write_handler(serial_write_handler);
        }

        gb.bus.cartridge = cartridge;

        Ok(gb)
    }

    #[cfg(not(feature = "web"))]
    pub fn build(self) -> Result<GameBoy, GameBoyBuilderError> {
        match self.bios {
            Some(bios) => {
                let serial_write_handler = self
                    .serial_write_handler
                    .unwrap_or_else(|| Box::new(Bus::get_handle_blargg_output()));
                let bios: [u8; 2304] = bios
                    .try_into()
                    .map_err(|_| GameBoyBuilderError::UnableToParseBios)?;

                Ok(GameBoy::new(self.rom, self.ram, bios, serial_write_handler))
            }
            None => self.create_gameboy_from_snapshot(),
        }
    }

    #[cfg(feature = "web")]
    pub fn build(self) -> GameBoy {
        match self.bios {
            Some(bios) => {
                let serial_write_handler = self
                    .serial_write_handler
                    .unwrap_or_else(|| Box::new(Bus::get_handle_blargg_output()));
                let bios: [u8; 2304] = bios
                    .try_into()
                    .map_err(|_| GameBoyBuilderError::UnableToParseBios)
                    .unwrap();

                GameBoy::new(self.rom, self.ram, bios, serial_write_handler)
            }
            None => self.create_gameboy_from_snapshot().unwrap(),
        }
    }
}
