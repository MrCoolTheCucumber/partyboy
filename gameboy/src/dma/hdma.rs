use std::fmt::Display;

use crate::{
    bus::{Bus, CgbCompatibility},
    cartridge::Cartridge,
};

#[derive(Copy, Clone)]
pub enum DmaType {
    Hdma,
    Gdma,
}

impl Display for DmaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DmaType::Hdma => write!(f, "HDMA"),
            DmaType::Gdma => write!(f, "GDMA"),
        }
    }
}

pub(crate) struct Hdma {
    pub src_hi: u8,
    pub src_lo: u8,
    pub dest_hi: u8,
    pub dest_lo: u8,
    pub hdma5: u8, // length/start/mode?

    pub current_dma: Option<DmaType>,
    pub hdma_currently_copying: bool,
    hdma_stop_requested: bool,

    bytes_to_transfer: u16,
    bytes_transfered: u16,

    src_addr: u16,
    dest_addr: u16,

    console_compatibility_mode: CgbCompatibility,
}

impl Default for Hdma {
    fn default() -> Self {
        Self {
            src_hi: 0x0,
            src_lo: 0x0,
            dest_hi: 0x0,
            dest_lo: 0x0,
            hdma5: 0xFF,

            current_dma: None,
            hdma_currently_copying: false,
            hdma_stop_requested: false,

            bytes_to_transfer: 0,
            bytes_transfered: 0,

            src_addr: 0,
            dest_addr: 0,

            console_compatibility_mode: CgbCompatibility::CgbOnly,
        }
    }
}

impl Hdma {
    pub fn set_console_compatibility(&mut self, mode: CgbCompatibility) {
        self.console_compatibility_mode = mode;
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        if !self.console_compatibility_mode.is_cgb_mode() {
            return 0xFF;
        }

        match addr {
            0xFF51 => self.src_hi,
            0xFF52 => self.src_lo,
            0xFF53 => self.dest_hi,
            0xFF54 => self.dest_lo,
            0xFF55 => self.hdma5,

            _ => panic!("HDMA doesnt handle reading from address: {:#06X}", addr),
        }
    }

    fn update_src_addr(&mut self) {
        self.src_addr = ((self.src_hi as u16) << 8) | ((self.src_lo) as u16);
    }

    fn update_dest_addr(&mut self) {
        self.dest_addr = 0x8000 + ((self.dest_hi as u16) << 8) | (self.dest_lo as u16);
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        if !self.console_compatibility_mode.is_cgb_mode() {
            return;
        }

        match addr {
            0xFF51 => {
                self.src_hi = val;
                self.update_src_addr();
            }
            0xFF52 => {
                self.src_lo = val & 0b1111_0000;
                self.update_src_addr();
            }
            0xFF53 => {
                self.dest_hi = val & 0x1F;
                self.update_dest_addr();
            }
            0xFF54 => {
                self.dest_lo = val & 0b1111_0000;
                self.update_dest_addr();
            }

            0xFF55 => {
                self.bytes_to_transfer = ((val & 0b0111_1111) + 1) as u16 * 0x10;

                let dma_type = if (val & 0b1000_0000) != 0 {
                    DmaType::Hdma
                } else {
                    DmaType::Gdma
                };

                if self.is_hdma_active() {
                    log::debug!("Stopping HDMA early.");
                    self.hdma_stop_requested = true;
                    return;
                }

                self.current_dma = Some(dma_type);
                self.bytes_transfered = 0;
                self.hdma5 = val & 0b0111_1111;

                #[cfg(debug_assertions)]
                if self.current_dma.is_some() {
                    log::debug!(
                        "Starting {}: Src: {:#06X}, Dest: {:#06X}, blocks: {}, raw: {:#04X}",
                        dma_type,
                        self.src_addr,
                        self.dest_addr,
                        (val & 0b00011111) + 1,
                        val
                    );
                }
            }

            _ => panic!("HDMA doesnt handle writing to address: {:#06X}", addr),
        }
    }

    pub fn is_hdma_active(&self) -> bool {
        if let Some(dma) = self.current_dma {
            return matches!(dma, DmaType::Hdma);
        }

        false
    }

    pub fn tick_gdma(bus: &mut Bus) {
        // transfer a block
        let bytes_to_transfer = 2;

        for i in 0..bytes_to_transfer {
            let transfer_val = bus.read_u8(bus.ppu.hdma.src_addr + i);
            bus.write_u8(bus.ppu.hdma.dest_addr + i, transfer_val);
        }

        bus.ppu.hdma.bytes_to_transfer -= bytes_to_transfer;
        if bus.ppu.hdma.bytes_to_transfer == 0 {
            bus.ppu.hdma.current_dma = None;
            bus.ppu.hdma.hdma5 = 0xFF;
        } else {
            bus.ppu.hdma.src_addr += bytes_to_transfer;
            bus.ppu.hdma.dest_addr += bytes_to_transfer;
        }
    }

    fn get_src_read_func(&self) -> read::ReaderFn {
        match self.src_addr {
            0x0000..=0x7FFF => read::read_rom,
            0xA000..=0xBFFF => read::read_ram,
            0xC000..=0xCFFF => read::read_wram,
            0xD000..=0xDFFF => read::read_wram_bank,

            _ => panic!("Unable to handle src addr when getting read func"),
        }
    }

    pub fn tick_hdma(
        &mut self,
        cartridge: &Box<dyn Cartridge>,
        working_ram: &[[u8; 4096]; 8],
        working_ram_bank: usize,
        gpu_vram: &mut [[u8; 8192]; 2],
        vram_bank: usize,
    ) {
        let dest_addr_index = (self.dest_addr - 0x8000) as usize;
        let bytes_to_transfer = 2;

        let read: read::ReaderFn = self.get_src_read_func();

        for i in 0..bytes_to_transfer {
            let transfer_val = read(self.src_addr + i, cartridge, working_ram, working_ram_bank);
            gpu_vram[vram_bank][dest_addr_index + (i as usize)] = transfer_val;
        }

        self.bytes_to_transfer -= bytes_to_transfer;
        self.bytes_transfered += 2;

        if self.bytes_transfered == 16 {
            self.hdma5 -= 1;
            self.bytes_transfered = 0;
        }

        if self.hdma_stop_requested {
            self.hdma_stop_requested = false;
            self.current_dma = None;
            self.hdma_currently_copying = false;
            self.hdma5 |= 0x80;
        } else if self.bytes_to_transfer == 0 {
            self.current_dma = None;
            self.hdma5 = 0xFF; // technically its already 0xFF
            self.hdma_currently_copying = false;
        } else {
            self.src_addr += bytes_to_transfer;
            self.dest_addr += bytes_to_transfer; // TODO: if this overflows, then HDMA stops (according to pandocs?)
        }
    }
}

mod read {
    pub(super) type ReaderFn = fn(u16, &Box<dyn Cartridge>, &[[u8; 4096]; 8], usize) -> u8;

    use crate::cartridge::Cartridge;

    pub(super) fn read_rom(
        addr: u16,
        cartridge: &Box<dyn Cartridge>,
        _: &[[u8; 4096]; 8],
        _: usize,
    ) -> u8 {
        cartridge.read_rom(addr)
    }

    pub(super) fn read_ram(
        addr: u16,
        cartridge: &Box<dyn Cartridge>,
        _: &[[u8; 4096]; 8],
        _: usize,
    ) -> u8 {
        cartridge.read_ram(addr - 0xA000)
    }

    pub(super) fn read_wram(
        addr: u16,
        _: &Box<dyn Cartridge>,
        working_ram: &[[u8; 4096]; 8],
        _: usize,
    ) -> u8 {
        working_ram[0][(addr - 0xC000) as usize]
    }

    pub(super) fn read_wram_bank(
        addr: u16,
        _: &Box<dyn Cartridge>,
        working_ram: &[[u8; 4096]; 8],
        working_ram_bank: usize,
    ) -> u8 {
        working_ram[working_ram_bank][(addr - 0xD000) as usize]
    }
}
