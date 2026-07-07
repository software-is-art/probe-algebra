//! The binary is three lines of dispatch; everything judged lives in the library
//! (`probe_hook::respond` / `probe_hook::install`), where the drills can reach it.
//! The process NEVER exits nonzero on the hook path — fail-open is the contract, and
//! a hook that breaks the edit loop is worse than no hook.

use std::io::Read;
use std::path::PathBuf;

fn main() {
    let project = std::env::var("CLAUDE_PROJECT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    match std::env::args().nth(1).as_deref() {
        Some("install") => match probe_hook::install(&project) {
            Ok(done) => println!("{done}"),
            Err(refusal) => {
                eprintln!("refused: {refusal}");
                std::process::exit(1);
            }
        },
        Some("--version") | Some("-V") => {
            println!("probe-hook {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            let mut input = String::new();
            let _ = std::io::stdin().read_to_string(&mut input);
            if let Some(voice) = probe_hook::respond(&input, &project) {
                println!("{voice}");
            }
        }
    }
}
