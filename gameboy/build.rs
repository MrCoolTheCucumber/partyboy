use std::{
    env,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("boot_rom.rs");

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("bin/_cgb_boot.bin");

    if !path.exists() {
        path.pop();
        path.push("cgb_boot.bin")
    }

    let mut file = File::open(path).unwrap();
    let mut buffer = [0u8; 0x900];
    file.read_exact(&mut buffer).ok();

    let boot_rom_code = format!("const BOOT_ROM: [u8; 0x900] = {:?};", buffer);

    fs::write(&dest_path, boot_rom_code).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
