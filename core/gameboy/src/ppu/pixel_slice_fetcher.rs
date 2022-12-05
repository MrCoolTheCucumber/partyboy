#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::debug::CgbCompatibility;

use super::{BGMapFlags, LcdControlFlag, Ppu};

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
    pub fn reset_fetcher(&mut self) {
        self.fifo_state.fetcher = PixelSliceFetcherState::default();
    }

    pub fn tick_fetcher(&mut self) {
        match self.fifo_state.fetcher.fetch_mode {
            FetchMode::Background(mode) => self.tick_bg(mode),
            FetchMode::Sprite => self.tick_sprite(),
        }
    }

    fn tick_bg(&mut self, bg_fetch_mode: BackgroundFetchMode) {
        self.fifo_state.fetcher.cycle = self.fifo_state.fetcher.cycle.saturating_add(1);

        match self.fifo_state.fetcher.cycle {
            // fetch tile id
            2 => {
                let (bit_10, bit_9_to_5, bit_4_to_0) = match bg_fetch_mode {
                    BackgroundFetchMode::Background => {
                        let bit_10 =
                            ((self.lcdc & LcdControlFlag::BGTileMapAddress as u8) >> 3) as u16;
                        let bit_9_to_5 = (self.ly.wrapping_add(self.scy) / 8) as u16;
                        let bit_4_to_0 = (self.fifo_state.lx.wrapping_add(self.scy) / 8) as u16;
                        (bit_10, bit_9_to_5, bit_4_to_0)
                    }
                    BackgroundFetchMode::Window => {
                        let bit_10 =
                            ((self.lcdc & LcdControlFlag::WindowTileMapAddress as u8) >> 6) as u16;
                        let bit_9_to_5 = (self.wy / 8) as u16;
                        let bit_4_to_0 = (self.fifo_state.lx / 8) as u16;
                        (bit_10, bit_9_to_5, bit_4_to_0)
                    }
                };

                let map_addr: u16 =
                    0b1001_1000_0000_0000 | (bit_10 << 10) | (bit_9_to_5 << 5) | bit_4_to_0;
                let is_signed_tile_addressing = self.is_signed_tile_addressing();

                self.fifo_state.fetcher.bg_state.tile_id =
                    self.get_adjusted_tile_index(map_addr, is_signed_tile_addressing);

                let flags = self.gpu_vram[1][map_addr as usize];
                self.fifo_state.fetcher.bg_state.tile_attr = BGMapFlags::from(flags);
            }

            // fetch tile data lo byte
            4 => {
                let bit_12 = match self.lcdc & LcdControlFlag::BGAndWindowTileData as u8 > 0 {
                    true => 0,
                    false => (self.fifo_state.fetcher.bg_state.tile_id & 0b1000) >> 3,
                };

                let mut bit_3_to_1 = match bg_fetch_mode {
                    BackgroundFetchMode::Background => (self.ly.wrapping_add(self.scy) % 8) as u16,
                    BackgroundFetchMode::Window => (self.wy % 8) as u16,
                };

                let tile_attr = self.fifo_state.fetcher.bg_state.tile_attr;
                let tile_id = self.fifo_state.fetcher.bg_state.tile_id;
                let bank_index = match self.console_compatibility_mode {
                    CgbCompatibility::CgbOnly => tile_attr.tile_bank,
                    _ => 0,
                };

                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.vertical_flip {
                    bit_3_to_1 = !bit_3_to_1;
                }

                let tile_data_addr =
                    0b1000_0000_0000_0000 | (bit_12 << 12) | (tile_id << 4) | (bit_3_to_1 << 1);
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
                let tile_data_addr = self.fifo_state.fetcher.bg_state.tile_data_addr + 1;
                let bank_index = match self.console_compatibility_mode {
                    CgbCompatibility::CgbOnly => tile_attr.tile_bank,
                    _ => 0,
                };

                let mut tile_byte_hi = self.gpu_vram[bank_index][tile_data_addr as usize];
                if self.console_compatibility_mode.is_cgb_mode() && tile_attr.horizontal_flip {
                    tile_byte_hi = tile_byte_hi.reverse_bits()
                }

                self.fifo_state.fetcher.bg_state.data_hi = tile_byte_hi;
            }

            8..=u8::MAX => {
                if !self.fifo_state.bg_fifo.is_empty() {
                    return;
                }
            }

            _ => {}
        }
    }

    fn tick_sprite(&mut self) {}
}
