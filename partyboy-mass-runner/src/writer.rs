use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use anyhow::Result;
use crossbeam::channel::Receiver;
use indicatif::MultiProgress;

use crate::emulator::into_img;
use crate::types::RunResult;

const ROM_EXTS: &[&str] = &[".gbc", ".gb", ".sgb"];

/// Strip a trailing ROM extension (`.gbc`, `.gb`, `.sgb`) so display names, dedup keys,
/// and marker filenames stay consistent regardless of where the input came from.
fn clean_stem(name: &str) -> &str {
    ROM_EXTS
        .iter()
        .fold(name, |acc, ext| acc.trim_end_matches(ext))
}

/// Background thread that saves screenshots, writes `.failed` markers, and rewrites the
/// HTML report after each result. On startup it reconstructs state from disk so
/// `--resume` produces a complete report even when no new ROMs need processing.
pub fn spawn_writer_thread(
    rx: Receiver<RunResult>,
    output_dir: PathBuf,
    completed: Arc<AtomicUsize>,
    mp: Option<MultiProgress>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let log = |msg: String| match &mp {
            Some(m) => {
                let _ = m.println(msg);
            }
            None => tracing::info!("{}", msg),
        };

        let (mut successes, mut failures) = bootstrap_from_disk(&output_dir);
        if !successes.is_empty() || !failures.is_empty() {
            match write_html_report(&output_dir, &successes, &failures) {
                Ok(()) => log(format!(
                    "Regenerated report from disk: {} successes, {} failures",
                    successes.len(),
                    failures.len()
                )),
                Err(e) => log(format!("warning: failed to write initial report: {e}")),
            }
            completed.fetch_add(successes.len() + failures.len(), Ordering::Relaxed);
        }

        for result in rx {
            match result {
                RunResult::Success {
                    fourty_seconds_frame_buffer,
                    onetwenty_seconds_frame_buffer,
                    mut rom_path,
                } => {
                    rom_path.set_extension("");
                    let stem = rom_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let _ = into_img(fourty_seconds_frame_buffer)
                        .save(output_dir.join(format!("{stem}_40.png")));
                    let _ = into_img(onetwenty_seconds_frame_buffer)
                        .save(output_dir.join(format!("{stem}_120.png")));

                    if !successes.contains(&stem) {
                        successes.push(stem);
                    }
                }
                RunResult::Fail { rom_name, error } => {
                    let msg = panic_message::panic_message(&error).to_string();
                    log(format!("ERROR: ROM {rom_name} failed: {msg}"));

                    let stem = clean_stem(&rom_name).to_string();
                    let _ = fs::write(output_dir.join(format!("{stem}.failed")), &msg);
                    if !failures.iter().any(|(name, _)| name == &stem) {
                        failures.push((stem, msg));
                    }
                }
            }

            completed.fetch_add(1, Ordering::Relaxed);
            if let Err(e) = write_html_report(&output_dir, &successes, &failures) {
                log(format!("Failed to write report: {e}"));
            }
        }

        if let Err(e) = write_html_report(&output_dir, &successes, &failures) {
            log(format!("Final report write failed: {e}"));
        }
    })
}

/// Reconstruct `(successes, failures)` from `.failed` markers and matched `_40.png`/
/// `_120.png` pairs in `output_dir`. Returns empty vecs if the dir doesn't exist.
fn bootstrap_from_disk(output_dir: &Path) -> (Vec<String>, Vec<(String, String)>) {
    let mut successes: Vec<String> = Vec::new();
    let mut failures: Vec<(String, String)> = Vec::new();

    let Ok(entries) = fs::read_dir(output_dir) else {
        return (successes, failures);
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if name.ends_with(".failed") {
            let stem = clean_stem(name.trim_end_matches(".failed")).to_string();
            if let Ok(msg) = fs::read_to_string(&path) {
                if !failures.iter().any(|(n, _)| n == &stem) {
                    failures.push((stem, msg));
                }
            }
        } else if let Some(stem) = name.strip_suffix("_40.png") {
            if output_dir.join(format!("{stem}_120.png")).exists()
                && !successes.iter().any(|n| n == stem)
            {
                successes.push(stem.to_string());
            }
        }
    }

    (successes, failures)
}

fn write_html_report(
    output_dir: &Path,
    successes: &[String],
    failures: &[(String, String)],
) -> Result<()> {
    let mut html = String::from(HTML_HEAD);

    for stem in successes {
        html.push_str(&format!(
            r#"<div class="card"><div class="name">{stem}</div><img src="{stem}_40.png"><img src="{stem}_120.png"></div>"#
        ));
    }

    html.push_str(HTML_MIDDLE);

    for (name, msg) in failures {
        html.push_str(&format!(
            r#"<div class="fail"><b>{name}</b><br>{msg}</div>"#
        ));
    }

    html.push_str(HTML_FOOT);

    let html = html
        .replace("##success_count##", &successes.len().to_string())
        .replace("##fail_count##", &failures.len().to_string());

    fs::write(output_dir.join("report.html"), html)?;
    Ok(())
}

const HTML_HEAD: &str = r##"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Mass Runner Report</title>
<style>
:root { --bg:#0d0d0d; --card:#1a1a1a; --accent:#00aaff; }
body { font-family: system-ui, sans-serif; background:var(--bg); color:#ddd; padding: 20px; }
h1 { color: var(--accent); }
.tabs { display:flex; gap: 10px; margin-bottom: 20px; }
.tab-button { padding: 8px 16px; background:#222; border: none; color:#ddd; cursor:pointer; border-radius:4px; }
.tab-button.active { background: var(--accent); color:black; }
.tab-content { display:none; }
.tab-content.active { display:block; }
.grid { display: flex; flex-wrap: wrap; gap: 12px; }
.card { background: var(--card); padding: 8px; border-radius: 6px; width: 200px; box-shadow: 0 2px 4px rgba(0,0,0,0.4); }
.card .name { font-size: 0.85em; margin-bottom: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
img { width: 100%; image-rendering: pixelated; background:#000; }
.fail-list .fail { background:#2a1212; padding:10px; margin-bottom:8px; border-radius:4px; font-family: monospace; font-size:0.9em; white-space: pre-wrap; }
</style></head><body>
<h1>Mass Runner Report</h1>
<div class="tabs">
  <button class="tab-button active" onclick="showTab('success')">Successes (##success_count##)</button>
  <button class="tab-button" onclick="showTab('failures')">Failures (##fail_count##)</button>
</div>

<div id="success" class="tab-content active">
  <div class="grid">
"##;

const HTML_MIDDLE: &str = r#"</div></div>
<div id="failures" class="tab-content">
  <div class="fail-list">
"#;

const HTML_FOOT: &str = r#"</div></div>
<script>
function showTab(tab) {
  document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
  document.querySelectorAll('.tab-button').forEach(el => el.classList.remove('active'));
  document.getElementById(tab).classList.add('active');
  event.target.classList.add('active');
}
</script>
</body></html>"#;
