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

    let mut boot_rom_code = "const BOOT_ROM: [u8; 0x900] = [".to_owned();
    for i in 0..0x900 {
        let mut val = buffer[i].to_string();
        if i != 0x8FF {
            val.push(',');
        }

        boot_rom_code.push_str(val.as_str());
    }

    boot_rom_code.push_str("];");

    fs::write(&dest_path, boot_rom_code).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}
