//! schemata — the compiled-mutant sweep's envelope (`discover::schemata`).
//!
//!     .github/schemata.sh                       # CI: build once, one test run per site
//!     cargo run --example schemata -- list      # the site census, sorted
//!     cargo run --example schemata -- judge <survivors.txt>
//!
//! `list` feeds the sweep loop; `judge` holds the live survivor set to the ratified
//! register (`spec/schemata.register`, standard set-difference semantics): a NEW
//! survivor wants a killing probe or a justification line, a ratified line whose
//! mutant now dies wants deleting. Exit 1 on drift, every mutant named.

use boundary_spec::discover::schemata::Schemata;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("list") => match Schemata::census() {
            Ok(sites) => {
                for site in sites {
                    println!("{site}");
                }
            }
            Err(collision) => {
                eprintln!("{collision}");
                std::process::exit(1);
            }
        },
        Some("judge") if args.len() == 2 => {
            let live = std::fs::read_to_string(&args[1]).unwrap_or_default();
            let survivors: Vec<&str> = live
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            match Schemata::register().check(survivors.iter().copied()) {
                Ok(()) => println!(
                    "schemata sweep clean: {} survivor(s), all ratified.",
                    survivors.len()
                ),
                Err(drift) => {
                    eprintln!("{drift}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("usage: schemata list | schemata judge <survivors.txt>");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The runner's two data paths hold: the census lists the committed sites, and
    /// the register judgment is the standard set-difference (an unratified survivor
    /// drifts as a new finding).
    #[test]
    fn the_census_and_register_paths_hold() {
        let sites = Schemata::census().expect("collision-free");
        if cfg!(feature = "schemata") {
            assert!(
                sites.contains(&"boundary_spec::discover::substrate::TagLaw::matches:0: == -> !=")
            );
        } else {
            assert!(
                sites.is_empty(),
                "instrumentation must not leak into normal builds"
            );
        }
        // the committed register and the ratified survivors are one set: exactly the
        // ratified keys hold, an empty sweep flags them stale, and an unratified
        // survivor drifts, named.
        let ratified: Vec<String> = Schemata::register()
            .entries()
            .expect("register parses")
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert!(Schemata::register()
            .check(ratified.iter().map(String::as_str))
            .is_ok());
        assert!(Schemata::register().check([]).is_err(), "stale lines flag");
        let err = Schemata::register()
            .check(
                ratified
                    .iter()
                    .map(String::as_str)
                    .chain(["classify:1: == -> !="]),
            )
            .unwrap_err();
        assert!(err.contains("classify:1"), "{err}");
    }
}
