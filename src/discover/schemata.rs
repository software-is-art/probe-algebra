//!
//! schemata — MUTANT SCHEMATA: the whole expression-mutant population compiled into
//! ONE binary behind a runtime selector, so a mutant costs a test run, not a rebuild.
//!
//! Source-level mutation pays a build per mutant (~tens of seconds each; the sweeps'
//! whole economics problem). The in-process layers (`discover::mutation`,
//! `discover::judgment`) dissolved that cost wherever behaviour is DATA — operator
//! tables, live states — but free-form interior Rust remained the rebuild-priced
//! remainder. Schemata is the classic answer, built on the one interface we control:
//! the macro layer. `#[mutate]` (boundary-spec-macros) rewrites each `==`, `!=`,
//! `<`, `<=`, `>`, `>=`, `&&`, `||` (and each `!`, as a deletion) in an instrumented
//! function as
//!
//! ```text
//! if Schemata::active("<site>") { <flipped> } else { <original> }
//! ```
//!
//! plus one DEAFNESS mutant per constructible form of the return type (`Ok(default)`,
//! `Err(default)`, `None`, both booleans — the whole-body replacement class, read from
//! the type syntax), so every mutant ships in the same build, inert until selected.
//! The `mutation (schemata)` gate then builds ONCE and runs the lib suite once per site
//! with `PROBE_MUTANT=<site>` — cheap enough (~a minute warm) to ride EVERY change,
//! not a weekly clock: a run that goes green with a flip active is a SURVIVOR
//! — a probe hole (or a ratified equivalence, one line in `spec/schemata.register`
//! with its justification). These are exactly the operator classes every recent
//! source-sweep survivor lived in, now judged at test-run price.
//!
//! The census is DERIVED, never listed by hand: each instrumented function's sites
//! register into [`MUTANT_SITES`] at link time (a distributed slice), and the sorted
//! inventory freezes as `spec/schemata.spec` — instrumenting a function, or editing
//! one enough to move its sites, is a ratified diff. Which functions carry
//! `#[mutate]` is itself reviewable data: the ones whose survivor species kept
//! recurring — the judges, the router's classifier, the reliance register's judge.
//!
//! Honest frame: schemata covers what a runtime branch can express — with the
//! deafness forms that now includes the whole-body replacement class, leaving the
//! source sweeps type-level mutations, statement deletion, and the uninstrumented
//! remainder as their earned territory. Sites inside `matches!`/`assert!`
//! macro bodies are opaque tokens and stay uninstrumented (disclosed by the macro's
//! docs). And the selector is read ONCE per process: one mutant per run, by design —
//! interactions between flips are out of scope, as they are for every mutation layer.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use spec_lock::{Lock, Register};

/// Every flip site the build carries, contributed by each `#[mutate]` expansion at
/// link time. Link order is arbitrary — always read through [`Schemata::census`],
/// which sorts and refuses collisions.
#[linkme::distributed_slice]
pub static MUTANT_SITES: [&'static str] = [..];

thread_local! {
    /// A drill's forced site — checked before the process-wide selector so probes can
    /// activate one mutant without touching the environment (see [`Schemata::force`]).
    static FORCED: RefCell<Option<&'static str>> = const { RefCell::new(None) };
}

/// The schemata harness: the selector the instrumented code consults, the census the
/// lock freezes, and the survivor register the weekly sweep judges against.
pub struct Schemata;

impl Schemata {
    /// Is this site's flip active? A drill's forced site wins; otherwise the
    /// `PROBE_MUTANT` environment selector, read once per process. With neither set
    /// this is a cached `None` comparison — the instrumented functions run their
    /// original expressions.
    pub fn active(site: &'static str) -> bool {
        Self::record(site);
        if let Some(forced) = FORCED.with(|f| *f.borrow()) {
            return forced == site;
        }
        static SELECTED: OnceLock<Option<String>> = OnceLock::new();
        SELECTED
            .get_or_init(|| std::env::var("PROBE_MUTANT").ok())
            .as_deref()
            == Some(site)
    }

    /// COVERAGE recording: with `SCHEMATA_RECORD=<dir>` set, every site whose guard
    /// executes is appended (once) to a per-process file headed by this process's
    /// test filter (nextest runs one test per process, so the file IS the site→test
    /// edge list). The map makes the sweep EXACT and selective: a mutant can only
    /// change behaviour where its guard executes, so running just the covering tests
    /// loses nothing — and a site no test reaches is a survivor before any run.
    /// Without the variable this is one cached `None` check.
    fn record(site: &'static str) {
        struct Recorder {
            file: Mutex<(std::fs::File, BTreeSet<&'static str>)>,
        }
        static RECORDER: OnceLock<Option<Recorder>> = OnceLock::new();
        let recorder = RECORDER.get_or_init(|| {
            let dir = std::env::var("SCHEMATA_RECORD").ok()?;
            let path = PathBuf::from(dir).join(format!("{}.touch", std::process::id()));
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()?;
            // the test names in this process's argv (nextest passes the exact
            // filter; test paths are the `::`-carrying arguments).
            let tests: Vec<String> = std::env::args().filter(|a| a.contains("::")).collect();
            writeln!(file, "T {}", tests.join(" ")).ok()?;
            Some(Recorder {
                file: Mutex::new((file, BTreeSet::new())),
            })
        });
        if let Some(recorder) = recorder {
            if let Ok(mut guard) = recorder.file.lock() {
                if guard.1.insert(site) {
                    let (file, _) = &mut *guard;
                    let _ = writeln!(file, "S {site}");
                }
            }
        }
    }

    /// Run `probe` with one site's flip forced on THIS thread — the drill hook, so a
    /// probe can watch a mutant change behaviour without environment plumbing. The
    /// force is dropped even if the probe panics.
    pub fn force<R>(site: &'static str, probe: impl FnOnce() -> R) -> R {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                FORCED.with(|f| *f.borrow_mut() = None);
            }
        }
        FORCED.with(|f| *f.borrow_mut() = Some(site));
        let _reset = Reset;
        probe()
    }

    /// The site census: sorted (link order is arbitrary), a duplicate id REFUSED by
    /// name — two functions sharing a `#[mutate]` label would make the selector
    /// ambiguous, and an ambiguous mutant is no mutant.
    pub fn census() -> Result<Vec<&'static str>, String> {
        let mut sites: Vec<&'static str> = MUTANT_SITES.iter().copied().collect();
        sites.sort_unstable();
        for pair in sites.windows(2) {
            if pair[0] == pair[1] {
                return Err(format!(
                    "schemata site `{}` is registered twice — two `#[mutate]` \
                     functions share a label; pass distinct labels",
                    pair[0]
                ));
            }
        }
        Ok(sites)
    }

    /// The human-readable census — what `spec/schemata.spec` freezes.
    pub fn render() -> Result<String, String> {
        let sites = Self::census()?;
        let mut out = format!(
            "# the schemata census, DERIVED — every expression-flip mutant this build\n\
             # carries (`#[mutate]` sites, registered at link time), one per line. The\n\
             # every-change `mutation (schemata)` gate builds once and runs the lib suite\n\
             # once per site with the flip active; survivors are ratified by key in\n\
             # spec/schemata.register or killed with a probe. Instrumenting a function\n\
             # (or moving its sites) is a ratified diff to this file. Regenerate with\n\
             # `cargo run --example freeze_gates`.\n\
             #\n\
             # {} sites.\n\n",
            sites.len()
        );
        for site in sites {
            out.push_str(&format!("- {site}\n"));
        }
        Ok(out)
    }

    /// `spec/schemata.spec` — the census, frozen.
    pub fn lock() -> Result<Lock, String> {
        Ok(Lock {
            name: "schemata".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("schemata.spec"),
            live: Self::render()?,
        })
    }

    /// The ratified-survivor register: `spec/schemata.register`, one
    /// `<site>: <justification>` line per mutant the suite provably cannot kill and a
    /// human has accepted as equivalent (or as a freedom). Judged with the standard
    /// set-difference semantics on every sweep.
    pub fn register() -> Register {
        Register {
            name: "schemata survivors".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("schemata.register"),
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::substrate::TagLaw;

    /// The census is real: sites registered from the instrumented functions, sorted,
    /// collision-free, every id carrying the `<label>:<index>: <op> -> <flip>` shape.
    #[test]
    fn the_census_is_sorted_and_collision_free() {
        let sites = Schemata::census().expect("no label collisions");
        assert!(
            sites.len() >= 20,
            "suspiciously few schemata sites: {}",
            sites.len()
        );
        let mut sorted = sites.clone();
        sorted.sort_unstable();
        assert_eq!(sites, sorted);
        for site in &sites {
            assert!(
                site.contains(" -> ") && site.contains(':'),
                "malformed site id: {site}"
            );
        }
    }

    /// The plumbing flips REAL behaviour: with no selector, an instrumented function
    /// is its original self; with its site forced, the flip is live — and the force
    /// is thread-local and dropped afterwards.
    #[test]
    fn a_forced_site_flips_the_instrumented_expression() {
        let law = TagLaw {
            pattern: "mutants-green",
            means: "",
            required: true,
        };
        assert!(law.matches("mutants-green"), "unmutated: exact match holds");
        // the exact-match arm's `==` site, flipped to `!=`:
        let site = "tag_law::matches:0: == -> !=";
        assert!(
            Schemata::census().unwrap().contains(&site),
            "the drill's site must exist in the census (renumbered? re-pin it)"
        );
        Schemata::force(site, || {
            assert!(
                !law.matches("mutants-green"),
                "with the flip active, the exact match must invert"
            );
            assert!(law.matches("something-else"));
        });
        assert!(law.matches("mutants-green"), "the force is dropped");
        // the DEAFNESS form, same plumbing: the function returns the constant.
        Schemata::force("tag_law::matches:deaf -> true", || {
            assert!(law.matches("anything-at-all"), "deaf-true hears nothing");
        });
        Schemata::force("tag_law::matches:deaf -> false", || {
            assert!(
                !law.matches("mutants-green"),
                "deaf-false denies everything"
            );
        });
    }

    /// The committed schemata lock is FRESH — instrumentation and census move
    /// together or the build refuses.
    #[test]
    fn the_committed_schemata_lock_is_fresh() {
        let lock = Schemata::lock().expect("census renders");
        if let Err(stale) = spec_lock::check(&[lock]) {
            panic!(
                "the schemata census drifted: {}. Regenerate with \
                 `cargo run --example freeze_gates` and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
