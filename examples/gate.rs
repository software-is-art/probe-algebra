//! gate — run the pipeline's EVERY-CHANGE gates locally, from the same declaration CI
//! executes.
//!
//!     cargo run --example gate
//!
//! CI stops being where verification is defined (`discover::gates` is) and becomes one
//! more machine that executes it: a green run here is the same claim as the workflow's
//! `check` job, because both are rendered from one registry. The per-diff and scheduled
//! gates are listed but not run (they are the pipeline's economics — mutation is paid for
//! at PR boundaries and on the weekly clock).

use std::process::Command;

use boundary_spec::discover::gates::{Cadence, GateRegistry};

fn main() {
    let mut failed = false;
    for gate in GateRegistry::declared() {
        if gate.cadence != Cadence::EveryChange {
            println!(
                "skip  {} ({}) — see spec/gates.spec",
                gate.name,
                gate.command_line()
            );
            continue;
        }
        println!("gate  {} — {}", gate.name, gate.command_line());
        let status = Command::new(gate.command[0])
            .args(&gate.command[1..])
            .status()
            .unwrap_or_else(|e| panic!("could not launch `{}`: {e}", gate.command_line()));
        if status.success() {
            println!("pass  {}", gate.name);
        } else {
            println!("FAIL  {}", gate.name);
            failed = true;
            break;
        }
    }
    if failed {
        std::process::exit(1);
    }
    println!("every-change gates green — the same claim the workflow's check job makes.");
}
