//! suite_audit — close the loop: feed the project's OWN mutation results into `select`.
//!
//! `select` (the self-hosted kill-matrix selector) is usually exercised on synthetic grids.
//! Here it consumes the REAL data cargo-mutants leaves in `mutants.out/`:
//!
//!   1. `outcomes.json` lists every mutant and whether the suite caught it;
//!   2. each caught mutant's per-mutant log names the test(s) that FAILED on it.
//!
//! Together those are a real kill matrix — rows = tests (the probes), columns = mutants,
//! cell = "this test caught this mutant". Running `select` over it answers two questions
//! with the method's own machinery:
//!
//!   - **the minimal attributing suite** — the fewest tests that still kill every killable
//!     mutant, preferring tests that kill FEW (so a failure points at a specific cause); and
//!   - **the survivors** — mutants no test kills, i.e. `KillMatrix::uncoverable`: each is a
//!     MISSING relation or degree of freedom, the exact signal the method is built to surface.
//!
//! Run `cargo mutants` first, then `cargo run --example suite_audit`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use boundary_spec::select::boundary::KillMatrix;
use serde_json::Value;

/// One mutant from the sweep: its cargo-mutants name, whether it was caught, and the set of
/// tests that caught it (empty for a survivor).
struct Mutant {
    name: String,
    killers: Vec<String>,
}

fn main() {
    let out = Path::new("mutants.out");
    if !out.join("outcomes.json").exists() {
        eprintln!("no mutants.out/outcomes.json — run `cargo mutants` first.");
        std::process::exit(1);
    }

    let json: Value = serde_json::from_str(
        &fs::read_to_string(out.join("outcomes.json")).expect("read outcomes.json"),
    )
    .expect("parse outcomes.json");

    let mut mutants: Vec<Mutant> = Vec::new();
    let (mut timeouts, mut unviable, mut unattributed) = (0usize, 0usize, 0usize);

    for o in json["outcomes"].as_array().expect("outcomes array") {
        let scenario = &o["scenario"]["Mutant"];
        if !scenario.is_object() {
            continue; // the baseline run, not a mutant
        }
        let name = scenario["name"].as_str().unwrap_or("<unknown>").to_string();
        match o["summary"].as_str().unwrap_or("") {
            "CaughtMutant" => {
                let log = o["log_path"].as_str().unwrap_or("");
                let killers = failing_tests(&fs::read_to_string(out.join(log)).unwrap_or_default());
                if killers.is_empty() {
                    // caught, but by a process ABORT (e.g. an overflow panic that kills the
                    // test binary mid-run) — no single test prints `... FAILED`, so it has no
                    // attribution. Detected, but excluded from the matrix (it is not a hole).
                    unattributed += 1;
                } else {
                    mutants.push(Mutant { name, killers });
                }
            }
            // a survivor: a real spec hole. No test kills it → an all-false column.
            "MissedMutant" => mutants.push(Mutant {
                name,
                killers: Vec::new(),
            }),
            // detected by hanging, but no NAMED test owns it — excluded from the matrix.
            "Timeout" => timeouts += 1,
            // not a real bug (the mutant did not compile) — excluded.
            _ => unviable += 1,
        }
    }

    // Rows = every test that caught at least one mutant, in stable order.
    let tests: Vec<String> = mutants
        .iter()
        .flat_map(|m| m.killers.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    // The real kill matrix: rows = tests, columns = mutants.
    let grid: Vec<Vec<bool>> = tests
        .iter()
        .map(|t| mutants.iter().map(|m| m.killers.contains(t)).collect())
        .collect();
    let matrix = KillMatrix::new(grid).expect("the kill matrix is rectangular by construction");

    let cover = matrix.select();
    let survivors = matrix.uncoverable();

    // ---- report -----------------------------------------------------------
    println!("== suite audit (select over real mutation data) ==\n");
    println!(
        "matrix: {} tests x {} attributable mutants",
        tests.len(),
        mutants.len(),
    );
    println!(
        "excluded: {unattributed} caught-by-abort (no single test owns them), \
         {timeouts} timeouts, {unviable} unviable"
    );

    println!(
        "\nminimal attributing suite: {} of {} tests retain full kill power",
        cover.positions().len(),
        tests.len()
    );
    for idx in cover.positions() {
        let test = &tests[idx.get()];
        let kills = mutants.iter().filter(|m| m.killers.contains(test)).count();
        println!("  - {test}  (kills {kills})");
    }

    let dropped: Vec<&String> = tests
        .iter()
        .enumerate()
        .filter(|(i, _)| !cover.positions().iter().any(|p| p.get() == *i))
        .map(|(_, t)| t)
        .collect();
    if !dropped.is_empty() {
        println!(
            "\nredundant for kill power ({}): every mutant they catch is already covered",
            dropped.len()
        );
        for t in &dropped {
            println!("  - {t}");
        }
    }

    println!(
        "\nsurvivors — missing relation / degree of freedom: {}",
        survivors.positions().len()
    );
    for idx in survivors.positions() {
        println!("  ! {}", mutants[idx.get()].name);
    }
    if survivors.positions().is_empty() {
        println!("  (none — every attributable mutant is killed)");
    }
}

/// Pull the failing test names out of a cargo-mutants per-mutant log. Lines look like:
/// `test capability::tests::the_resolve_family_audits_as_specified ... FAILED`.
fn failing_tests(log: &str) -> Vec<String> {
    log.lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("test ")?;
            let name = rest.strip_suffix(" ... FAILED")?;
            Some(name.to_string())
        })
        .collect()
}
