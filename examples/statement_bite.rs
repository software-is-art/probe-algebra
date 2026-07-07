//! statement_bite — the SUBSTRATE half of the statement-bite gate: judge every
//! definition mutant of `lean/ProbeBool.lean` with the real Lean kernel, then hold the
//! survivors against `lean/bites.register`.
//!
//!     cargo run --example statement_bite
//!
//! Needs a `lean` binary: `$LEAN`, then `~/.elan/bin/lean`, then `lean` on PATH (the
//! corpus is core-only — no lake, no mathlib, one file). CI's weekly gate
//! (`.github/statement-bite.sh`) installs elan on the fly; the expected survivor set
//! is ALSO pinned toolchain-free by `discover::bite`'s mirror probe in `cargo test`,
//! so this run is the kernel's countersignature, not the only witness.

use std::path::PathBuf;
use std::process::Command;

use boundary_spec::discover::bite::Corpus;

fn lean_binary() -> PathBuf {
    if let Ok(explicit) = std::env::var("LEAN") {
        return PathBuf::from(explicit);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let elan = PathBuf::from(home).join(".elan/bin/lean");
        if elan.exists() {
            return elan;
        }
    }
    PathBuf::from("lean")
}

fn main() {
    let lean = lean_binary();
    let version = Command::new(&lean)
        .arg("--version")
        .output()
        .unwrap_or_else(|e| {
            eprintln!(
                "no usable `lean` at {} ({e}). Set $LEAN, install elan, or rely on the \
                 CI gate — the mirror probe in `cargo test` pins the expected survivors \
                 without a toolchain.",
                lean.display()
            );
            std::process::exit(1);
        });
    print!("{}", String::from_utf8_lossy(&version.stdout));

    let corpus = Corpus::read(&Corpus::committed_path()).expect("the committed corpus parses");
    let scratch = std::env::temp_dir().join("statement-bite");
    std::fs::create_dir_all(&scratch).expect("a scratch dir for mutants");
    let mutant_path = scratch.join("ProbeBool.lean");

    let verdicts = corpus
        .judge_with(|text| {
            std::fs::write(&mutant_path, text).map_err(|e| format!("write mutant: {e}"))?;
            let out = Command::new(&lean)
                .arg(&mutant_path)
                .output()
                .map_err(|e| format!("run lean: {e}"))?;
            Ok(out.status.success())
        })
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });

    for key in &verdicts.killed {
        println!("KILLED   {key}");
    }
    for key in &verdicts.survived {
        println!("SURVIVED {key}");
    }
    match verdicts.gate(&Corpus::survivor_register()) {
        Ok(summary) => println!("{summary}"),
        Err(drift) => {
            eprintln!("{drift}");
            std::process::exit(1);
        }
    }
}
