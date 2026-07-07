//! review_agenda — the reading list derived from which locks moved.
//!
//!     git diff --name-only main...HEAD | xargs cargo run --example review_agenda --
//!
//! Routes changed paths into the ratifications they require (see `discover::agenda`):
//! one question per moved lock class, prose listed for sense, interior code named
//! machinery-verified. A spec-directory artifact of unknown class refuses — teach the
//! router before it can misfile a review.

use boundary_spec::discover::agenda::Agenda;

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: review_agenda <changed paths...> | --guard <path>");
        std::process::exit(2);
    }
    // the HOOK mode: one edited path, one line if a downstream refusal is being walked
    // into, silence otherwise. Fail-open — a missing file is an empty source.
    if paths[0] == "--guard" {
        if let Some(path) = paths.get(1) {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            // kernel-hood is the register's fact, the same one build.rs consumes — a
            // file is kernel when its manifest-relative path has a ratified entry in
            // spec/kernel.register (the hook runs at the repo root). A refused
            // register is the build gate's problem; the advisory guard fails open.
            let kernel = spec_lock::Register {
                name: "kernel".to_string(),
                path: std::path::PathBuf::from("spec/kernel.register"),
            }
            .entries()
            .unwrap_or_default()
            .iter()
            .any(|(key, _)| path == key || path.ends_with(&format!("/{key}")));
            if let Some(guard) = Agenda::edit_guard(path, &source, kernel) {
                println!("{guard}");
            }
        }
        return;
    }
    match Agenda::of(&paths) {
        Ok(agenda) => print!("{}", agenda.render()),
        Err(refusal) => {
            eprintln!("refused: {refusal}");
            std::process::exit(1);
        }
    }
}
