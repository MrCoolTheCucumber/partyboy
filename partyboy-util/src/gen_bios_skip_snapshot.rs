use clap::Parser;
use partyboy_core::builder::GameBoyBuilder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    bios: String,
    #[arg(short, long)]
    rom: String,
    #[arg(short, long)]
    output: String,
}

pub fn execute(args: Args) {
    let bios = std::fs::read(&args.bios).expect("Unable to read bios file");
    let rom = std::fs::read(&args.rom).expect("Unable to read rom file");

    let mut gb = GameBoyBuilder::new()
        .bios(bios)
        .rom(rom)
        .build()
        .expect("Unable to build gameboy");

    // tick for 60 * 20 frames
    for _ in 0..(70_224 * 60 * 10) {
        gb.tick();
    }

    let snapshot = rmp_serde::to_vec(&gb).expect("Unable to generate snapshot");
    std::fs::write(args.output, snapshot).expect("Unable to write snapshot to output path");
}
