#![allow(non_snake_case)]

mod common;

use common::APPROX_CYCLES_PER_SCREEN_DRAW;
use partyboy_core::{builder::SerialWriteHandler, GameBoy};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

const PASSING_FIB: [u8; 6] = [3, 5, 8, 13, 21, 34];

fn assert_output(buffer: &[u8]) {
    assert_eq!(buffer.len(), 6);

    for i in 0..6 {
        assert_eq!(buffer[i], PASSING_FIB[i]);
    }
}

macro_rules! define_mooneye_tests {
    ($($name:ident: $file:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let mut path = get_root_path();
                path.push("");
                path.push($file);

                let path = path.to_str().unwrap();
                let rom = std::fs::read(path).unwrap();

                let buffer = Rc::new(RefCell::new(Vec::new()));

                let buffer_closure = buffer.clone();
                let serial_write_handler: SerialWriteHandler = Box::new(move |val| {
                    buffer_closure.borrow_mut().push(val);
                });

                let mut gb = GameBoy::builder()
                    .rom(rom)
                    .serial_write_handler(serial_write_handler)
                    .build()
                    .unwrap();

                for _ in 0..APPROX_CYCLES_PER_SCREEN_DRAW * 60 * 10 {
                    gb.tick();
                }

                let buffer = (*buffer).borrow();
                assert_output(&buffer);
            }
        )*
    };
}

mod acceptance {
    fn get_root_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("test/test_roms/mooneye/acceptance/");
        path
    }

    use super::*;

    mod bits {
        use super::*;

        define_mooneye_tests! {
            mem_oam: "bits/mem_oam.gb",
            reg_f: "bits/reg_f.gb",
            // unused_hwio: "bits/unused_hwio-GS.gb"
        }
    }

    mod instr {
        use super::*;

        define_mooneye_tests! {
            daa: "instr/daa.gb",
        }
    }

    mod interrupts {
        use super::*;

        define_mooneye_tests! {
            ie_push: "interrupts/ie_push.gb",
        }
    }

    mod oam_dma {
        use super::*;

        define_mooneye_tests! {
            basic: "oam_dma/basic.gb",
            reg_read: "oam_dma/reg_read.gb",
            // sources: "oam_dma/sources-GS.gb",
            oam_dma_restart: "oam_dma_restart.gb",
            oam_dma_start: "oam_dma_start.gb",
            oam_dma_timing: "oam_dma_timing.gb",
        }
    }

    mod ppu {
        use super::*;

        define_mooneye_tests! {
            // hblank_ly_scx_timing: "ppu/hblank_ly_scx_timing-GS.gb",
            // intr_1_2_timing: "ppu/intr_1_2_timing-GS.gb",
            intr_2_0_timing: "ppu/intr_2_0_timing.gb",
            intr_2_mode0_timing: "ppu/intr_2_mode0_timing.gb",
            intr_2_mode0_timing_sprites: "ppu/intr_2_mode0_timing_sprites.gb",
            intr_2_mode3_timing: "ppu/intr_2_mode3_timing.gb",
            intr_2_oam_ok_timing: "ppu/intr_2_oam_ok_timing.gb",
            // lcdon_timing: "ppu/lcdon_timing-GS.gb",
            // lcdon_write_timing: "ppu/lcdon_write_timing-GS.gb",
            stat_irq_blocking: "ppu/stat_irq_blocking.gb",
            stat_lyc_onoff: "ppu/stat_lyc_onoff.gb",
            // vblank_stat_intr: "ppu/vblank_stat_intr-GS.gb",
        }
    }

    // -- serial --, dont really care for now

    mod timer {
        use super::*;

        define_mooneye_tests! {
            div_write: "timer/div_write.gb",
            rapid_toggle: "timer/rapid_toggle.gb",
            tim00: "timer/tim00.gb",
            tim00_div_trigger: "timer/tim00_div_trigger.gb",
            tim01: "timer/tim01.gb",
            tim01_div_trigger: "timer/tim01_div_trigger.gb",
            tim10: "timer/tim10.gb",
            tim10_div_trigger: "timer/tim10_div_trigger.gb",
            tim11: "timer/tim11.gb",
            tim11_div_trigger: "timer/tim11_div_trigger.gb",
            tima_reload: "timer/tima_reload.gb",
            tima_write_reloading: "timer/tima_write_reloading.gb",
            tma_write_reloading: "timer/tma_write_reloading.gb",
        }
    }

    define_mooneye_tests! {
        add_sp_e_timing: "add_sp_e_timing.gb",

        // boot_div: "boot_div-dmgABCmgb.gb",
        //boot_regs: "boot_regs-dmgABC.gb",
        call_cc_timing: "call_cc_timing.gb",
        call_cc_timing2: "call_cc_timing2.gb",
        call_timing: "call_timing.gb",
        call_timing2: "call_timing2.gb",
        div_timing: "div_timing.gb",
        // di_timing: "di_timing-GS.gb",
        ei_sequence: "ei_sequence.gb",
        ei_timing: "ei_timing.gb",
        halt_ime0_ei: "halt_ime0_ei.gb",
        halt_ime0_nointr_timing: "halt_ime0_nointr_timing.gb",
        halt_ime1_timing: "halt_ime1_timing.gb",
        //halt_ime1_timing2: "halt_ime1_timing2-GS.gb",
        if_ie_registers: "if_ie_registers.gb",


        intr_timing: "intr_timing.gb",
        jp_cc_timing: "jp_cc_timing.gb",
        jp_timing: "jp_timing.gb",
        ld_hl_sp_e_timing: "ld_hl_sp_e_timing.gb",

        pop_timing: "pop_timing.gb",

        push_timing: "push_timing.gb",
        rapid_di_ei: "rapid_di_ei.gb",
        reti_intr_timing: "reti_intr_timing.gb",
        reti_timing: "reti_timing.gb",
        ret_cc_timing: "ret_cc_timing.gb",
        ret_timing: "ret_timing.gb",
        rst_timing: "rst_timing.gb",
        // boot_sclk_align: "serial/boot_sclk_align-dmgABCmgb.gb",

    }
}

mod emulator_only {
    use super::*;

    mod mbc1 {
        use super::*;

        fn get_root_path() -> PathBuf {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.pop();
            path.push("test/test_roms/mooneye/emulator-only/mbc1/");
            path
        }

        define_mooneye_tests! {
            bits_bank1: "bits_bank1.gb",
            bits_bank2: "bits_bank2.gb",
            bits_mode: "bits_mode.gb",
            bits_ramg: "bits_ramg.gb",
            multicart_rom_8Mb: "multicart_rom_8Mb.gb",
            ram_256kb: "ram_256kb.gb",
            ram_64kb: "ram_64kb.gb",
            rom_16Mb: "rom_16Mb.gb",
            rom_1Mb: "rom_1Mb.gb",
            rom_2Mb: "rom_2Mb.gb",
            rom_4Mb: "rom_4Mb.gb",
            rom_512kb: "rom_512kb.gb",
            rom_8Mb: "rom_8Mb.gb",
        }
    }

    mod mbc2 {
        use super::*;

        fn get_root_path() -> PathBuf {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.pop();
            path.push("test/test_roms/mooneye/emulator-only/mbc2/");
            path
        }

        define_mooneye_tests! {
            bits_ramg: "bits_ramg.gb",
            bits_romb: "bits_romb.gb",
            bits_unused: "bits_unused.gb",
            ram: "ram.gb",
            rom_1Mb: "rom_1Mb.gb",
            rom_2Mb: "rom_2Mb.gb",
            rom_512kb: "rom_512kb.gb",
        }
    }

    mod mbc5 {
        use super::*;

        fn get_root_path() -> PathBuf {
            let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.pop();
            path.push("test/test_roms/mooneye/emulator-only/mbc5/");
            path
        }

        define_mooneye_tests! {
            rom_16Mb: "rom_16Mb.gb",
            rom_1Mb: "rom_1Mb.gb",
            rom_2Mb: "rom_2Mb.gb",
            rom_32Mb: "rom_32Mb.gb",
            rom_4Mb: "rom_4Mb.gb",
            rom_512kb: "rom_512kb.gb",
            rom_64Mb: "rom_64Mb.gb",
            rom_8Mb: "rom_8Mb.gb",
        }
    }
}
