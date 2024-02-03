use std::{
    any::Any,
    fs, iter, panic,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use image::{ImageBuffer, RgbImage};
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use partyboy_core::{builder::GameBoyBuilder, input::Keycode, ppu::rgb::Rgb};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    roms_dir: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(short, long)]
    bios: Option<PathBuf>,
}

#[allow(unused)]
enum RunResult {
    Success {
        rom_name: String,
        fourty_seconds_frame_buffer: Vec<Rgb>,
        onetwenty_seconds_frame_buffer: Vec<Rgb>,
        rom_path: PathBuf,
    },
    Fail {
        rom_name: String,
        error: Box<dyn Any + Send>,
    },
}

struct Rom {
    bytes: Vec<u8>,
    path: PathBuf,
}

fn get_all_roms(path: &Path) -> Result<Vec<Rom>> {
    fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                return None;
            }
            Some(entry.path())
        })
        .map(|path| {
            let bytes = fs::read(&path)?;
            Ok(Rom { bytes, path })
        })
        .collect::<Result<Vec<_>>>()
}

fn into_img(fb: Vec<Rgb>) -> RgbImage {
    let mut img: RgbImage = ImageBuffer::new(160, 144);
    fb.into_iter()
        .flat_map(|rgb| [rgb.r, rgb.g, rgb.b])
        .zip(img.iter_mut())
        .for_each(|(px, img_px)| *img_px = px);
    img
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.roms_dir.is_dir() {
        bail!("roms_dir is not a directory!");
    }

    if args.output.is_file() {
        bail!("output is not a directory!");
    }

    let bios = if let Some(bios_path) = args.bios {
        Some(fs::read(bios_path)?)
    } else {
        None
    };

    let roms = get_all_roms(&args.roms_dir)?;

    println!("Found {} roms.", roms.len());

    let pb = ProgressBar::new(roms.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );

    let run_results = roms
        .into_par_iter()
        .progress_with(pb)
        .map(|rom| {
            let rom_name = rom
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or(anyhow!("Unable to get filename"))
                .unwrap()
                .to_owned();

            panic::set_hook(Box::new(|_| {}));

            let result = panic::catch_unwind({
                let rom_name = rom_name.clone();
                let rom_path = rom.path.clone();
                || {
                    let mut builder = GameBoyBuilder::new();

                    if let Some(bios) = &bios {
                        builder = builder.bios(bios.clone());
                    }

                    let mut gb = builder.rom(rom.bytes).build().unwrap();
                    for _ in 0..(partyboy_core::SPEED * 40) {
                        let _ = gb.tick();
                    }

                    let fourty_seconds_frame_buffer = gb.get_frame_buffer().to_vec();

                    let mut pressed = false;
                    for tick in 0..(partyboy_core::SPEED * 80) {
                        let _ = gb.tick();

                        if tick % (partyboy_core::SPEED / 2) == 0 {
                            match pressed {
                                true => {
                                    gb.key_down(Keycode::A);
                                    gb.key_up(Keycode::Start);
                                }
                                false => {
                                    gb.key_up(Keycode::A);
                                    gb.key_down(Keycode::Start);
                                }
                            }
                            pressed = !pressed;
                        }
                    }

                    let onetwenty_seconds_frame_buffer = gb.get_frame_buffer().to_vec();

                    RunResult::Success {
                        rom_name,
                        fourty_seconds_frame_buffer,
                        onetwenty_seconds_frame_buffer,
                        rom_path,
                    }
                }
            });

            match result {
                Ok(run_result) => run_result,
                Err(payload) => RunResult::Fail {
                    rom_name,
                    error: payload,
                },
            }
        })
        .collect::<Vec<RunResult>>();

    if !args.output.exists() {
        fs::create_dir(&args.output)?;
    }

    println!();
    println!();

    let mut html = Vec::new();

    for result in run_results {
        match result {
            RunResult::Success {
                rom_name,
                fourty_seconds_frame_buffer,
                onetwenty_seconds_frame_buffer,
                mut rom_path,
            } => {
                let fourty = into_img(fourty_seconds_frame_buffer);
                let onetwenty = into_img(onetwenty_seconds_frame_buffer);

                rom_path.set_extension("");
                let name = rom_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or(anyhow!("Unable to get filename"))?;

                fourty.save(args.output.join(format!("{name}_40.png")))?;
                onetwenty.save(args.output.join(format!("{name}_120.png")))?;

                let html_fragment = format!(
                    r#"
                    <div class="runner-result runner-success">
                        <div>{rom_name}</div>
                        <img src="{name}_40.png">
                        <img src="{name}_120.png">
                    </div>
                "#
                );

                html.push(html_fragment);
            }
            RunResult::Fail { rom_name, error } => {
                let msg = panic_message::panic_message(&error);

                let html_fragment = format!(
                    r#"
                    <div class="runner-result runner-fail">
                        <div>{rom_name}</div>
                        <div>{msg}</div>
                    </div>
                "#
                );

                html.push(html_fragment);

                eprintln!("A run failed:");
                eprintln!("Rom: {rom_name}");
                eprintln!("Error: {msg}");
                eprintln!();
            }
        }
    }

    let html = ["
        <!DOCTYPE html>
        <html>
        <head>
        <title>Page Title</title>
        </head>
        <body>
    "
    .to_owned()]
    .into_iter()
    .chain(html)
    .chain(iter::once("</body></html>".to_owned()))
    .collect::<String>();

    fs::write(args.output.join("report.html"), html)
        .map_err(|_| anyhow!("Unable to write html report file"))?;

    Ok(())
}
