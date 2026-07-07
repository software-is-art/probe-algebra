//! review_agenda — the reading list derived from which locks moved.
//!
//!     git diff --name-only main...HEAD | xargs cargo run --example review_agenda --
//!
//! Routes changed paths into the ratifications they require (see `discover::agenda`):
//! one question per moved lock class, prose listed for sense, interior code named
//! machinery-verified. A spec-directory artifact of unknown class refuses — teach the
//! router before it can misfile a review. Teaching is DATA: a committed
//! `spec/agenda.register`, one `suffix: question` line per consumer lock class
//! (`spec_lock::Register` grammar) — this repo needs none, so the file is absent here.

use boundary_spec::discover::agenda::{Agenda, GuardVoices};

/// The consumer-taught lock classes, from `spec/agenda.register` (missing = none).
/// A refused register (bare key, duplicate, no question) is a parse error surfaced to
/// the caller — a malformed class table must never silently drop a ratification.
fn taught_classes() -> Result<Vec<(String, String)>, String> {
    spec_lock::Register {
        name: "agenda".to_string(),
        path: std::path::PathBuf::from("spec/agenda.register"),
    }
    .entries()
}

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: review_agenda <changed paths...> | --guard <path>");
        std::process::exit(2);
    }
    // the HOOK mode: one edited path, one line if a downstream refusal is being walked
    // into, silence otherwise. Fail-open — a missing file is an empty source, and a
    // refused class register is the routing mode's problem, not the advisory guard's.
    if paths[0] == "--guard" {
        if let Some(path) = paths.get(1) {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            // the voices are DERIVED from the tree (the hook runs at the repo root):
            // kernel-exemption from spec/kernel.register, the structural voice from
            // evidence the enforcement shim actually runs here — so each voice only
            // pre-fires a refusal that exists.
            let voices = GuardVoices::for_edit(std::path::Path::new("."), path);
            let classes = taught_classes().unwrap_or_default();
            if let Some(guard) = Agenda::edit_guard(path, &source, &voices, &classes) {
                println!("{guard}");
            }
        }
        return;
    }
    let routed = taught_classes().and_then(|classes| Agenda::of_with(&paths, &classes));
    match routed {
        Ok(agenda) => print!("{}", agenda.render()),
        Err(refusal) => {
            eprintln!("refused: {refusal}");
            std::process::exit(1);
        }
    }
}
