mod common;

use common::APPROX_CYCLES_PER_SCREEN_DRAW;
use gameboy::{builder::SerialWriteHandler, GameBoy};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

macro_rules! define_blargg_cpu_test {
    ($($name:ident, $file:expr, $cycle_mult:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let mut path = get_root_path();
                path.push($file);

                let path = path.to_str().unwrap();

                let buffer = Rc::new(RefCell::new(Vec::new()));

                let buffer_closure = buffer.clone();
                let serial_write_handler: SerialWriteHandler = Box::new(move |val| {
                    buffer_closure.borrow_mut().push(val as char);
                });

                let mut gb = GameBoy::builder()
                    .rom_path(path)
                    .serial_write_handler(serial_write_handler)
                    .build()
                    .unwrap();

                for _ in 0..APPROX_CYCLES_PER_SCREEN_DRAW * $cycle_mult {
                    gb.tick();
                }

                let buffer = (*buffer).borrow_mut();
                let buffer_iter = buffer.iter();
                let output = String::from_iter(buffer_iter);
                assert!(output.contains("Passed"), "Output: \n{}", output);
            }
        )*
    };
}

mod cpu_instrs {
    fn get_root_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("test/test_roms/blargg/");
        path.push("cpu_instrs/individual/");
        path
    }

    use super::*;

    define_blargg_cpu_test! {
        cpu_01, "01-special.gb", 60 * 7 ,
        cpu_02, "02-interrupts.gb", 60 * 5,
        cpu_03, "03-op sp,hl.gb", 60 * 7,
        cpu_04, "04-op r,imm.gb", 60 * 7,
        cpu_05, "05-op rp.gb", 60 * 8,
        cpu_06, "06-ld r,r.gb", 60 * 5,
        cpu_07, "07-jr,jp,call,ret,rst.gb", 60 * 5,
        cpu_08, "08-misc instrs.gb", 60 * 5,
        cpu_09, "09-op r,r.gb", 60 * 13,
        cpu_10, "10-bit ops.gb", 60 * 17,
        cpu_11, "11-op a,(hl).gb", 60 * 25,
    }
}

mod instr_timing {
    use super::*;

    fn get_root_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("test/test_roms/blargg/");
        path.push("instr_timing/");
        path
    }

    define_blargg_cpu_test! {
        instr_timing, "instr_timing.gb", 60 * 10,
    }
}

mod mem_timing {
    use super::*;

    fn get_root_path() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop();
        path.push("test/test_roms/blargg/");
        path.push("mem_timing/");
        path
    }

    define_blargg_cpu_test! {
        mem_timing, "mem_timing.gb", 60 * 10,
    }
}
