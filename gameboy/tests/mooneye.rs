mod common;

use common::APPROX_CYCLES_PER_SCREEN_DRAW;
use gameboy::{builder::SerialWriteHandler, GameBoy};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

const PASSING_FIB: [u8; 6] = [3, 5, 8, 13, 21, 34];

fn get_root_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("test_roms/mooneye/");
    path
}

fn assert_output(buffer: &Vec<u8>) {
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
                path.push("acceptance/");
                path.push($file);

                let path = path.to_str().unwrap();

                let buffer = Rc::new(RefCell::new(Vec::new()));

                let buffer_closure = buffer.clone();
                let serial_write_handler: SerialWriteHandler = Box::new(move |val| {
                    buffer_closure.borrow_mut().push(val);
                });

                let mut gb = GameBoy::builder()
                    .rom_path(path)
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

define_mooneye_tests! {
    add_sp_e_timing: "add_sp_e_timing.gb",
    mem_oam: "bits/mem_oam.gb",
    reg_f: "bits/reg_f.gb",
    // unused_hwio: "bits/unused_hwio-GS.gb"
    boot_div: "boot_div-dmgABCmgb.gb",
    boot_regs: "boot_regs-dmgABC.gb",
    call_cc_timing: "call_cc_timing.gb",
    call_cc_timing2: "call_cc_timing2.gb",
    call_timing: "call_timing.gb",
    call_timing2: "call_timing2.gb",
    div_timing: "div_timing.gb",
    di_timing: "di_timing-GS.gb",
    ei_sequence: "ei_sequence.gb",
    ei_timing: "ei_timing.gb",
    halt_ime0_ei: "halt_ime0_ei.gb",
    halt_ime0_nointr_timing: "halt_ime0_nointr_timing.gb",
    halt_ime1_timing: "halt_ime1_timing.gb",
    halt_ime1_timing2: "halt_ime1_timing2-GS.gb",
    if_ie_registers: "if_ie_registers.gb",
    daa: "instr/daa.gb",
    // ie_push: "interrupts/ie_push.gb",
    intr_timing: "intr_timing.gb",
    jp_cc_timing: "jp_cc_timing.gb",
    jp_timing: "jp_timing.gb",
    ld_hl_sp_e_timing: "ld_hl_sp_e_timing.gb",
    basic: "oam_dma/basic.gb",
    reg_read: "oam_dma/reg_read.gb",
    // sources: "oam_dma/sources-GS.gb",
    oam_dma_restart: "oam_dma_restart.gb",
    oam_dma_start: "oam_dma_start.gb",
    oam_dma_timing: "oam_dma_timing.gb",
    pop_timing: "pop_timing.gb",
    hblank_ly_scx_timing: "ppu/hblank_ly_scx_timing-GS.gb",
    intr_1_2_timing: "ppu/intr_1_2_timing-GS.gb",
    intr_2_0_timing: "ppu/intr_2_0_timing.gb",
    intr_2_mode0_timing: "ppu/intr_2_mode0_timing.gb",
    // intr_2_mode0_timing_sprites: "ppu/intr_2_mode0_timing_sprites.gb",
    intr_2_mode3_timing: "ppu/intr_2_mode3_timing.gb",
    intr_2_oam_ok_timing: "ppu/intr_2_oam_ok_timing.gb",
    lcdon_timing: "ppu/lcdon_timing-GS.gb",
    // lcdon_write_timing: "ppu/lcdon_write_timing-GS.gb",
    stat_irq_blocking: "ppu/stat_irq_blocking.gb",
    stat_lyc_onoff: "ppu/stat_lyc_onoff.gb",
    vblank_stat_intr: "ppu/vblank_stat_intr-GS.gb",
    push_timing: "push_timing.gb",
    rapid_di_ei: "rapid_di_ei.gb",
    reti_intr_timing: "reti_intr_timing.gb",
    reti_timing: "reti_timing.gb",
    ret_cc_timing: "ret_cc_timing.gb",
    ret_timing: "ret_timing.gb",
    rst_timing: "rst_timing.gb",
    // boot_sclk_align: "serial/boot_sclk_align-dmgABCmgb.gb",
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
