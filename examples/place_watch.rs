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

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(file) = args.next() else {
        eprintln!("usage: place_watch <file.rs> [--once]");
        std::process::exit(2);
    };
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
