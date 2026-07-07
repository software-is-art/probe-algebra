//! place_watch — edit code, watch the layout move.
//!
//!     cargo run --example place_watch -- src/discover/workbench.rs
//!     cargo run --example place_watch -- src/discover/arithmetic.rs --once
//!
//! The live half of the placer: parses the `ops { ... }` stanzas out of the file's TEXT
//! (no compilation) on every save, re-derives the placement, and narrates the change in
//! the monotone vocabulary — seeded / joined / BRIDGED. Point it at a workbench and
//! write; layout stops being something to think about. `--once` prints the current
//! shape and exits (CI-safe).

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use boundary_spec::discover::watch::Ticker;

/// The HOOK mode (`--event <file> <state-dir>`): one invocation per agent edit, state
/// persisted between invocations, output only when the shape moved (see
/// `Ticker::hook_line` for the noise policy). Fail-open by design: any problem is
/// silence and exit 0 — a broken hook must degrade to no feedback, never to a broken
/// edit loop.
fn event_mode(file: &str, state_dir: &str) {
    let run = || -> Option<String> {
        let source = std::fs::read_to_string(file).ok()?;
        let mut key = String::new();
        for c in file.chars() {
            key.push(if c.is_ascii_alphanumeric() { c } else { '-' });
        }
        let state = PathBuf::from(state_dir).join(format!("{key}.sigs"));
        let name: &'static str = Box::leak(file.to_string().into_boxed_str());
        let line = match std::fs::read_to_string(&state) {
            Ok(stored) => Ticker::resume(name, &stored).hook_line(name, &source),
            // first sight: capture the baseline; a multi-component file announces
            // itself once (the extraction is already available).
            Err(_) => Ticker::new().hook_line(name, &source),
        };
        std::fs::create_dir_all(state_dir).ok()?;
        std::fs::write(&state, Ticker::store(&source).ok()?).ok()?;
        line
    };
    if let Some(line) = run() {
        println!("{line}");
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(file) = args.next() else {
        eprintln!("usage: place_watch <file.rs> [--once] | --event <file.rs> <state-dir>");
        std::process::exit(2);
    };
    if file == "--event" {
        let (Some(file), Some(state_dir)) = (args.next(), args.next()) else {
            std::process::exit(0); // fail open: a misinvoked hook is silence.
        };
        event_mode(&file, &state_dir);
        return;
    }
    let once = args.next().as_deref() == Some("--once");
    let path = PathBuf::from(&file);
    let mut ticker = Ticker::new();
    let mut last_mtime: Option<SystemTime> = None;
    loop {
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if mtime != last_mtime {
            last_mtime = mtime;
            match std::fs::read_to_string(&path) {
                Err(e) => eprintln!("{file}: {e}"),
                Ok(source) => match ticker.step("watched", &source) {
                    Err(refusal) => eprintln!("refused: {refusal}"),
                    Ok((placement, event)) => {
                        if let Some(event) = event {
                            println!("{}", event.render());
                        }
                        print!("{}", placement.render());
                    }
                },
            }
        }
        if once {
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
