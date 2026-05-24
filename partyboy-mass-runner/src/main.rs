mod args;
mod emulator;
mod rom;
mod types;
mod writer;

use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{fs, panic, process, thread};

use anyhow::{bail, Result};
use clap::Parser;
use crossbeam::channel;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use panic::AssertUnwindSafe;

use crate::args::Args;
use crate::emulator::run_one_rom;
use crate::rom::{filter_completed_for_resume, get_all_roms, rom_display_name, Rom};
use crate::types::{
    current_shutdown, new_shutdown, request_shutdown, RunResult, ShutdownState, WorkerStatus,
};

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    if !args.roms_dir.is_dir() {
        bail!("roms_dir is not a directory!");
    }
    if args.output.is_file() {
        bail!("output is not a directory!");
    }

    let bios: Option<Arc<[u8]>> = args
        .bios
        .as_deref()
        .map(fs::read)
        .transpose()?
        .map(Arc::from);

    let mut roms = get_all_roms(&args.roms_dir)?;

    if args.resume {
        if !args.output.exists() {
            fs::create_dir_all(&args.output)?;
        }
        let before = roms.len();
        roms = filter_completed_for_resume(roms, &args.output);
        tracing::info!(
            "Resume mode: {} already done, {} remaining.",
            before - roms.len(),
            roms.len()
        );
    }

    let total_to_process = roms.len();
    tracing::info!("Found {} ROMs to process.", total_to_process);

    let jobs = args.jobs.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(16)
    });

    let (work_tx, work_rx) = channel::unbounded::<Rom>();
    let (result_tx, result_rx) = channel::unbounded::<RunResult>();
    let completed_count = Arc::new(AtomicUsize::new(0));

    for rom in roms {
        let _ = work_tx.send(rom);
    }
    drop(work_tx);

    let mp = MultiProgress::new();

    // Route panic output through mp.println so it doesn't interleave with the bar
    // redraws (stderr writes by the default hook leave half-rendered bars stuck in
    // the scrollback). The full message is still persisted via the writer thread's
    // .failed marker.
    {
        let mp = mp.clone();
        panic::set_hook(Box::new(move |info| {
            let name = thread::current().name().unwrap_or("<unnamed>").to_string();
            let _ = mp.println(format!("thread '{name}' {info}"));
        }));
    }

    let global_bar = mp.add(ProgressBar::new(total_to_process as u64));
    global_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>5}/{len:5} ROMs ({eta})",
        )
        .unwrap()
        .progress_chars("##-"),
    );
    global_bar.set_message("Overall");

    let worker_bars: Vec<ProgressBar> = (0..jobs)
        .map(|i| {
            let pb = mp.add(ProgressBar::new(120));
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "  [W{i:02}] {{spinner:.yellow}} {{msg:<20.20}} [{{bar:30.green/blue}}] {{pos:>3}}/{{len}}s"
                ))
                .unwrap()
                .progress_chars("##-"),
            );
            pb.set_message("idle");
            pb
        })
        .collect();

    let writer_handle = writer::spawn_writer_thread(
        result_rx,
        args.output.clone(),
        completed_count.clone(),
        Some(mp.clone()),
    );

    let worker_statuses: Vec<Arc<WorkerStatus>> =
        (0..jobs).map(|_| Arc::new(WorkerStatus::new())).collect();

    let shutdown = new_shutdown();

    // A single UI driver thread polls worker statuses and updates bars, so workers
    // never touch ProgressBar objects directly. Serializing draws here avoids races
    // with mp.println from the writer/panic hook that would otherwise duplicate or
    // freeze the bar block.
    {
        let statuses = worker_statuses.clone();
        let bars = worker_bars.clone();
        let shutdown = shutdown.clone();
        thread::spawn(move || {
            while current_shutdown(&shutdown) == ShutdownState::Running {
                for (status, bar) in statuses.iter().zip(&bars) {
                    let (rom, emul) = status.snapshot();
                    bar.set_message(rom.unwrap_or_else(|| "idle".into()));
                    bar.set_position(emul);
                }
                thread::sleep(Duration::from_millis(80));
            }
        });
    }

    // Ctrl+C: first press = graceful drain, second = force exit.
    {
        let shutdown = shutdown.clone();
        let count = Arc::new(AtomicU8::new(0));
        ctrlc::set_handler(move || {
            if count.fetch_add(1, Ordering::Relaxed) == 0 {
                tracing::warn!(
                    "Ctrl+C received — finishing current ROMs then exiting (press again to force)"
                );
                request_shutdown(&shutdown, ShutdownState::Draining);
            } else {
                tracing::error!("Second Ctrl+C — forcing exit");
                request_shutdown(&shutdown, ShutdownState::ForceKill);
                process::exit(1);
            }
        })
        .expect("Error setting Ctrl+C handler");
    }

    let mut handles = Vec::with_capacity(jobs);
    for status in &worker_statuses {
        let status = status.clone();
        let work_rx = work_rx.clone();
        let result_tx = result_tx.clone();
        let bios = bios.clone();
        let speed = args.speed_factor;
        let shutdown = shutdown.clone();
        let global_bar = global_bar.clone();

        handles.push(thread::spawn(move || {
            // Top-level safety net: run_one_rom catches its own panics and converts
            // them to RunResult::Fail, so this only fires for the truly unexpected.
            let _ = panic::catch_unwind(AssertUnwindSafe(|| {
                while current_shutdown(&shutdown) == ShutdownState::Running {
                    let Ok(rom) = work_rx.recv() else { break };
                    status.set_rom(Some(rom_display_name(&rom.path)));
                    let res = run_one_rom(rom, bios.as_deref(), speed, Some(&status));
                    let _ = result_tx.send(res);
                    status.set_rom(None);
                    global_bar.inc(1);
                }
            }));
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    request_shutdown(&shutdown, ShutdownState::Draining);
    drop(result_tx);
    let _ = writer_handle.join();

    global_bar.finish_with_message("Done");
    for b in &worker_bars {
        b.finish_with_message("idle");
    }

    tracing::info!("Mass runner finished.");
    Ok(())
}
