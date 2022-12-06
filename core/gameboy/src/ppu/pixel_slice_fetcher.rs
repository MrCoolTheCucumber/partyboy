#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{BGMapFlags, FifoPixel, LcdControlFlag, Ppu};

use crate::bus::CgbCompatibility;

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) enum FetchMode {
    Background(BackgroundFetchMode),
    Sprite,
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) enum BackgroundFetchMode {
    Background,
    Window,
}

impl Default for FetchMode {
    fn default() -> Self {
        Self::Background(BackgroundFetchMode::Background)
    }
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct BgFetcherState {
    tile_data_addr: u16,
    tile_id: u16,
    tile_attr: BGMapFlags,
    tile_counter: u16,

    data_lo: u8,
    data_hi: u8,
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct SpriteFetcherState {
    tile_addr: u16,
    data_low: u8,
    data_high: u8,
}

#[derive(Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) struct PixelSliceFetcherState {
    cycle: u8,
    fetch_mode: FetchMode,
    bg_state: BgFetcherState,
    sprite_state: SpriteFetcherState,
}

impl Ppu {
    fn reset_fetcher(&mut self) {
        self.fifo_state.fetcher = PixelSliceFetcherState::default();
    }

    /// Change the fetch mode, this will reset the whole fetcher state
    pub(super) fn set_fifo_fetch_mode(&mut self, fetch_mode: FetchMode) {
        self.reset_fetcher();
        self.fifo_state.fetcher.fetch_mode = fetch_mode;
    }

    pub(super) fn tick_fetcher(&mut self) {
        match self.fifo_state.fetcher.fetch_mode {
            FetchMode::Background(mode) => self.tick_bg(mode),
            FetchMode::Sprite => self.tick_sprite(),
        }
    }

    fn get_bg_map_start_addr(&self) -> u16 {
        match self.lcdc & LcdControlFlag::BGTileMapAddress as u8 != 0 {
            true => 0x9C00,
            false => 0x9800,
        }
    }

    fn get_window_map_start_addr(&self) -> u16 {
        match (self.lcdc & LcdControlFlag::WindowTileMapAddress as u8) != 0 {
            true => 0x9C00,
            false => 0x9800,
        }
    }

    fn get_adjusted_tile_index(&self, addr: u16, signed_tile_index: bool) -> u16 {
        let addr = (addr - 0x8000) as usize;
        if signed_tile_index {
            let tile = self.gpu_vram[0][addr] as i8 as i16;
            if tile >= 0 {
                tile as u16 + 256
            } else {
                256 - (tile.unsigned_abs() as u16)
            }
        } else {
            self.gpu_vram[0][addr] as u16
        }
    }

    fn tick_bg(&mut self, bg_fetch_mode: BackgroundFetchMode) {
        self.fifo_state.fetcher.cycle += 1;

        match self.fifo_state.fetcher.cycle {
            // fetch tile id
            2 => {
                let tile_counter = self.fifo_state.fetcher.bg_state.tile_counter;
                let map_addr = match bg_fetch_mode {
                    BackgroundFetchMode::Background => {
                        let tile_y = self.ly.wrapping_add(self.scy) / 8;
                        let tile_map_id_y_offset = tile_y as u16 * 32;
                        let tile_map_id_x_offset =
                            (self.scx.wrapping_add(tile_counter as u8 * 8) / 8) as u16;
                        let bg_map_start_addr = self.get_bg_map_start_addr();

                        bg_map_start_addr + tile_map_id_y_offset + tile_map_id_x_offset
                    }
                    BackgroundFetchMode::Window => {
                        let window_tile_offset =
                            ((self.window_internal_line_counter - 1) / 8) as u16 * 32;
                        let wd_map_start_addr = self.get_window_map_start_addr();

                        wd_map_start_addr + window_tile_offset + tile_counter
                    }
                };

                self.fifo_state.fetcher.bg_state.tile_id =
                    self.get_adjusted_tile_index(map_addr, self.is_signed_tile_addressing());
                let flags = self.gpu_vram[1][map_addr as usize - 0x8000];
                self.fifo_state.fetcher.bg_state.tile_attr = BGMapFlags::from(flags);
            }

            // fetch tile data lo byte
            4 => {
                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let tile_id = self.fifo_state.fetcher.bg_state.tile_id;
                let bank_index = match self.console_compatibility_mode {
                    CgbCompatibility::CgbOnly => tile_attr.tile_bank,
                    _ => 0,
                };

                let mut offset = match bg_fetch_mode {
                    BackgroundFetchMode::Background => self.ly.wrapping_add(self.scy),
                    BackgroundFetchMode::Window => self.window_internal_line_counter - 1,
                } & 0b000_0111;

                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.vertical_flip {
                    offset = Self::flip_tile_value(offset);
                }

                offset <<= 1;

                let tile_data_addr = (tile_id * 16) + (offset as u16);
                let mut tile_byte_lo = self.gpu_vram[bank_index][tile_data_addr as usize];

                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.horizontal_flip {
                    tile_byte_lo = tile_byte_lo.reverse_bits()
                }

                self.fifo_state.fetcher.bg_state.tile_data_addr = tile_data_addr;
                self.fifo_state.fetcher.bg_state.data_lo = tile_byte_lo;
            }

            // fetch tile data hi byte
            6 => {
                // TODO: do i need to do: 'the first time we reach here we go back to step 1'?

                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let tile_data_addr = self.fifo_state.fetcher.bg_state.tile_data_addr;
                let bank_index = match self.console_compatibility_mode {
                    CgbCompatibility::CgbOnly => tile_attr.tile_bank,
                    _ => 0,
                };

                let mut tile_byte_hi = self.gpu_vram[bank_index][tile_data_addr as usize + 1];

                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.horizontal_flip {
                    tile_byte_hi = tile_byte_hi.reverse_bits()
                }

                self.fifo_state.fetcher.bg_state.data_hi = tile_byte_hi;
            }

            // push pixels into fifo
            8..=u8::MAX => {
                if !self.fifo_state.bg_fifo.is_empty() {
                    return;
                }

                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let b1 = self.fifo_state.fetcher.bg_state.data_lo;
                let b2 = self.fifo_state.fetcher.bg_state.data_hi;

                (0..8u8)
                    .rev()
                    .map(|shift| {
                        ((b1 & (1 << shift)) >> shift) | ((b2 & (1 << shift)) >> shift) << 1
                    })
                    .map(|color_index| {
                        let palette_index = match self.console_compatibility_mode {
                            CgbCompatibility::CgbOnly => tile_attr.bg_palette_number,
                            _ => 0,
                        } as u8;

                        let priority = match self.console_compatibility_mode {
                            CgbCompatibility::CgbOnly => Some(tile_attr.bg_oam_prio as u8),
                            _ => None,
                        };

                        FifoPixel {
                            color_index,
                            palette_index,
                            sprite_info: None,
                            priority,
                        }
                    })
                    .for_each(|px| self.fifo_state.bg_fifo.push_back(px));

                self.fifo_state.fetcher.cycle = 0;
                self.fifo_state.fetcher.bg_state.tile_counter += 1;
            }

            _ => {}
        }
    }

    fn tick_sprite(&mut self) {
        todo!()
    }
}
