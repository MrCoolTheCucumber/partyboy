use std::fmt::Display;

use crate::{
    bus::{Bus, CgbCompatibility},
    cartridge::Cartridge,
    cpu::Cpu,
    ppu::{Ppu, PpuMode},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct Hdma {
    pub src_hi: u8,
    pub src_lo: u8,
    pub dest_hi: u8,
    pub dest_lo: u8,
    pub hdma5: u8, // length/start/mode?

    pub current_dma: Option<DmaType>,
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
        if matches!(self.console_compatibility_mode, CgbCompatibility::None) {
            return 0xFF;
        }

        match addr {
            0xFF51 => 0xFF,
            0xFF52 => 0xFF,
            0xFF53 => 0xFF,
            0xFF54 => 0xFF,
            0xFF55 => self.hdma5,

            _ => panic!("HDMA doesnt handle reading from address: {:#06X}", addr),
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        if matches!(self.console_compatibility_mode, CgbCompatibility::None) {
            return;
        }

        fn update_src_addr(hdma: &mut Hdma) {
            hdma.src_addr = ((hdma.src_hi as u16) << 8) | ((hdma.src_lo) as u16);
        }

        fn update_dest_addr(hdma: &mut Hdma) {
            hdma.dest_addr = 0x8000 + (((hdma.dest_hi as u16) << 8) | (hdma.dest_lo as u16));
        }

        match addr {
            0xFF51 => {
                self.src_hi = val;
                update_src_addr(self);
            }
            0xFF52 => {
                self.src_lo = val & 0b1111_0000;
                update_src_addr(self);
            }
            0xFF53 => {
                self.dest_hi = val & 0b0001_1111;
                update_dest_addr(self);
            }
            0xFF54 => {
                self.dest_lo = val & 0b1111_0000;
                update_dest_addr(self);
            }

            0xFF55 => {
                let dma_type = if (val & 0b1000_0000) != 0 {
                    DmaType::Hdma
                } else {
                    DmaType::Gdma
                };

                if self.is_hdma_active() && (val & 0b1000_0000) == 0 {
                    // stop copy
                    self.hdma_stop_requested = true;
                    self.hdma5 = 0x80 | val;
                    return;
                }

                self.bytes_to_transfer = ((val & 0x7F) + 1) as u16 * 0x10;

                self.current_dma = Some(dma_type);
                self.bytes_transfered = 0;
                self.hdma5 = val & 0b0111_1111;

                // #[cfg(debug_assertions)]
                // if self.current_dma.is_some() {
                //     log::debug!(
                //         "Starting {}: Src: {:#06X}, Dest: {:#06X}, blocks: {}, bytes: {}, raw: {:#04X}",
                //         dma_type,
                //         self.src_addr,
                //         self.dest_addr,
                //         (val & 0x7F) + 1,
                //         self.bytes_to_transfer,
                //         val
                //     );
                // }
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
        let dest_addr_index = (bus.ppu.hdma.dest_addr - 0x8000) as usize;
        let bytes_to_transfer: u16 = 2;
        let mut dest_overflow = false;

        for i in 0..bytes_to_transfer {
            if dest_addr_index + (i as usize) >= 0x2000 {
                dest_overflow = false;
                break;
            }

            let transfer_val = bus.read_u8(bus.ppu.hdma.src_addr + i);
            bus.ppu.gpu_vram[(bus.ppu.gpu_vram_bank as usize) & 1]
                [dest_addr_index + (i as usize)] = transfer_val;
        }

        bus.ppu.hdma.bytes_to_transfer -= bytes_to_transfer;

        bus.ppu.hdma.src_addr += bytes_to_transfer;
        bus.ppu.hdma.dest_addr += bytes_to_transfer;

        if bus.ppu.hdma.bytes_to_transfer == 0 || dest_overflow {
            bus.ppu.hdma.current_dma = None;
            bus.ppu.hdma.hdma5 = 0xFF;
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
        cartridge: &Cartridge,
        working_ram: &[[u8; 4096]; 8],
        working_ram_bank: usize,
        gpu_vram: &mut [[u8; 8192]; 2],
        vram_bank: usize,
    ) -> bool {
        let dest_addr_index = (self.dest_addr - 0x8000) as usize;
        let bytes_to_transfer = 2;

        let read: read::ReaderFn = self.get_src_read_func();

        for i in 0..bytes_to_transfer {
            let transfer_val = read(self.src_addr + i, cartridge, working_ram, working_ram_bank);
            gpu_vram[vram_bank][dest_addr_index + (i as usize)] = transfer_val;
        }

        self.bytes_to_transfer -= bytes_to_transfer;
        self.bytes_transfered += 2;

        let mut finished_block_copy = false;
        if self.bytes_transfered == 16 {
            self.hdma5 -= 1;
            self.bytes_transfered = 0;
            finished_block_copy = true;
        }

        self.src_addr += bytes_to_transfer;
        self.dest_addr += bytes_to_transfer;

        if self.hdma_stop_requested && finished_block_copy {
            self.hdma_stop_requested = false;
            self.current_dma = None;
            self.hdma5 |= 0x80;
        } else if self.bytes_to_transfer == 0 {
            self.current_dma = None;
            self.hdma5 = 0xFF; // technically its already 0xFF
        }

        finished_block_copy
    }

    fn handle_stop_request(&mut self) {
        self.hdma_stop_requested = false;
        self.current_dma = None;
        self.hdma5 |= 0x80;
    }
}

const HDMA_T_PER_WORD_COPY: u32 = 4;
const HDMA_WIND_UP_T: u32 = 4;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct HdmaController {
    state: HdmaControllerState,
    clock: u32,
    hblank_rising_edge: HBlankRisingEdge,
}

impl Default for HdmaController {
    fn default() -> Self {
        Self {
            state: HdmaControllerState::Waiting,
            clock: HDMA_T_PER_WORD_COPY,
            // technically the ppu starts in mode 0 but initial state probably doesn't matter anyway
            hblank_rising_edge: HBlankRisingEdge::Hi(false),
        }
    }
}

impl HdmaController {
    /// returns true if we consume a rising edge
    fn try_consume_rising_edge(&mut self) -> bool {
        if matches!(self.hblank_rising_edge, HBlankRisingEdge::Hi(false)) {
            self.hblank_rising_edge = HBlankRisingEdge::Hi(true);
            true
        } else {
            false
        }
    }

    fn handle_hblank_rising_edge(&mut self, ppu: &Ppu) {
        let is_hblank = matches!(ppu.get_mode_stat(), PpuMode::HBlank);

        match self.hblank_rising_edge {
            HBlankRisingEdge::Lo => {
                if is_hblank {
                    self.hblank_rising_edge = HBlankRisingEdge::Hi(false);
                }
            }
            HBlankRisingEdge::Hi(_) => {
                if !is_hblank {
                    self.hblank_rising_edge = HBlankRisingEdge::Lo;
                }
            }
        }
    }

    fn handle_hdma_state(&mut self, state: HdmaState, bus: &mut Bus) {
        match state {
            HdmaState::CopyBlock => {
                self.clock -= 1;
                if self.clock == 0 {
                    self.clock = HDMA_T_PER_WORD_COPY;
                    let full_block_copied = bus.hdma_copy_word();

                    if full_block_copied {
                        // check if hdma is canceled/finished
                        self.state = match bus.ppu.hdma.current_dma {
                            Some(DmaType::Hdma) => {
                                HdmaControllerState::Started(HdmaState::WaitingForHBlank)
                            }
                            _ => HdmaControllerState::Waiting,
                        }
                    }
                }
            }
            HdmaState::WaitingForHBlank => {
                if bus.ppu.hdma.hdma_stop_requested {
                    bus.ppu.hdma.handle_stop_request();
                    self.state = HdmaControllerState::Waiting;
                    return;
                }

                if self.try_consume_rising_edge() {
                    self.state = HdmaControllerState::Started(HdmaState::CopyBlock);
                    self.clock -= 1;
                }
            }
        }
    }

    pub fn handle_hdma(&mut self, bus: &mut Bus, cpu: &mut Cpu) {
        self.handle_hblank_rising_edge(&bus.ppu);
        match self.state {
            HdmaControllerState::Waiting => {
                // we only check if we can start hdma inbetween cpu
                // we should probably only check "once"
                // I think actual time is 3t into cpu fetch? not too sure
                // comment out || is_fetching so it only checks the t cycle after the instruction finishes

                // if !(!cpu.is_processing_instruction()/*|| cpu.is_fetching()*/) {
                //     return;
                // }

                // TODO: above code might be better, but this seems fine for now
                if cpu.is_processing_instruction() {
                    return;
                }

                // if hdma isn't requested then return
                if !matches!(bus.ppu.hdma.current_dma, Some(DmaType::Hdma)) {
                    return;
                }

                // NEW: check if there is a rising edge we can consume
                if self.try_consume_rising_edge() {
                    self.clock = HDMA_T_PER_WORD_COPY + HDMA_WIND_UP_T;
                    self.clock -= 1;
                    self.state = HdmaControllerState::Started(HdmaState::CopyBlock);
                }
            }
            HdmaControllerState::Started(hdma_state) => self.handle_hdma_state(hdma_state, bus),
        }
    }

    pub fn currently_copying(&self, bus: &Bus) -> bool {
        matches!(
            self.state,
            HdmaControllerState::Started(HdmaState::CopyBlock)
        ) || matches!(bus.ppu.hdma.current_dma, Some(DmaType::Gdma))
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum HdmaControllerState {
    Waiting,
    Started(HdmaState),
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) enum HdmaState {
    CopyBlock,
    WaitingForHBlank,
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
enum HBlankRisingEdge {
    Lo,
    /// bool is have we "consumed" the rising edge, true => yes, false => no
    Hi(bool),
}

mod read {
    pub(super) type ReaderFn = fn(u16, &Cartridge, &[[u8; 4096]; 8], usize) -> u8;

    use crate::cartridge::Cartridge;

    pub(super) fn read_rom(addr: u16, cartridge: &Cartridge, _: &[[u8; 4096]; 8], _: usize) -> u8 {
        cartridge.read_rom(addr)
    }

    pub(super) fn read_ram(addr: u16, cartridge: &Cartridge, _: &[[u8; 4096]; 8], _: usize) -> u8 {
        cartridge.read_ram(addr - 0xA000)
    }

    pub(super) fn read_wram(
        addr: u16,
        _: &Cartridge,
        working_ram: &[[u8; 4096]; 8],
        _: usize,
    ) -> u8 {
        working_ram[0][(addr - 0xC000) as usize]
    }

    pub(super) fn read_wram_bank(
        addr: u16,
        _: &Cartridge,
        working_ram: &[[u8; 4096]; 8],
        working_ram_bank: usize,
    ) -> u8 {
        working_ram[working_ram_bank][(addr - 0xD000) as usize]
    }
}
