//!
//! freeze — the discovered spec is a committed, diffable artifact (a behaviour lock).
//!
//! Discovery is a pure function of the boundary, so the expensive enumeration is run ONCE and its
//! output FROZEN into a committed file per theory under `spec/`. The staleness gate (`Spec::check_all_fresh`)
//! re-derives the live spec and compares it to the committed text: a match means the code still
//! means what was ratified; a mismatch is a build error whose fix is to regenerate
//! (`cargo run --example freeze_spec`) and ratify the diff in review. So the committed spec file,
//! read in a pull request's diff, IS the ratification — the one human act discovery cannot perform.
//!
//! The freeze/drift mechanics (compare live text to a committed file; write the regeneration) are
//! the domain-agnostic `spec-lock` crate; this module is the thin adapter that knows what THIS
//! repo's artifacts are — where a theory's lock lives (`Spec::lock_in`) and how a `Spec` renders
//! (`render`). See `docs/ci-discipline.md` for the whole discipline.
//!
//! One principle governs what gets rendered INTO a lock: **a lock contains only facts about the
//! DOMAIN, never facts about the engine.** The header, the named laws, and the coverage line
//! (which operators no law speaks for) are all statements about the boundary's behaviour; they
//! belong in the lock. The consequence-equality count is NOT — it measures how many implied
//! equalities OUR enumeration happened to sample, so it churns whenever the engine's sampling
//! improves, with no behaviour change underneath. Keeping it out means an engine upgrade can only
//! drift a consumer's lock by changing the LAWS — which is exactly the drift a consumer must
//! ratify. (Consequence counts stay visible where engine facts belong: this repo's golden tests
//! and the `discovered_spec` example.)

use std::path::{Path, PathBuf};

use spec_lock::Lock;

use super::{all_specs, Spec};

/// The lock file for a theory inside a given spec directory (the theory name slugified).
#[crate::mutate]
fn lock_path(spec_dir: &Path, theory: &str) -> PathBuf {
    let slug: String = theory
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .collect();
    spec_dir.join(format!("{slug}.spec"))
}

/// The canonical text of a discovered spec — deterministic, human-readable, diffable. Renders
/// only domain facts (header, named laws, coverage) per the lock principle in the module docs.
#[crate::mutate]
fn render(spec: &Spec) -> String {
    // the header names no crate-specific regen command — this render serves DOWNSTREAM locks
    // too, and a consumer's freeze path is its own (see docs/ci-discipline.md).
    let mut out = String::new();
    out.push_str(&format!(
        "# discovered spec: {} — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.\n",
        spec.theory
    ));
    // the REGISTERED tolerance is part of the ratified artifact: review approves ε along
    // with the laws, never an ambient constant.
    if let Some(tolerance) = spec.tolerance {
        out.push_str(&format!(
            "# tolerance (registered with the theory): {tolerance}\n"
        ));
    }
    out.push('\n');
    for law in &spec.laws {
        out.push_str(&format!("- {}\n      {}\n", law.prose(), law.equation()));
    }
    out.push('\n');
    // the DISCLOSED band: candidates neither held nor refuted at the declared tolerance.
    // Absent entirely for exact theories, so no existing lock moves.
    if !spec.undecided.is_empty() {
        out.push_str(
            "# undecided at the declared tolerance (disclosed — neither held nor refuted):\n",
        );
        for law in &spec.undecided {
            out.push_str(&format!("- {}\n      {}\n", law.prose(), law.equation()));
        }
        out.push('\n');
    }
    let coverage = if spec.uncovered_ops.is_empty() {
        "none — every operator participates in a law".to_string()
    } else {
        spec.uncovered_ops.join(", ")
    };
    out.push_str(&format!(
        "# operators in no law (where the spec is silent): {coverage}\n"
    ));
    out
}

#[crate::mutate]
impl Spec {
    /// This spec as a `spec_lock::Lock` rooted in a caller-supplied spec directory — the
    /// CONSUMER-facing form. The lock file lives at `spec_dir/<slugified-theory>.spec` and
    /// carries the canonical rendering (`render`). This is the whole adapter: everything
    /// domain-specific about the freeze is in here (`lock_path` + `render`); the compare
    /// (`spec_lock::check`) and the regeneration write (`spec_lock::bless`) are generic.
    /// Attached to the `Spec` value object per the no-rats-nest rule: every public callable
    /// hangs off a typestate.
    ///
    /// A downstream crate points `spec_dir` at a directory in ITS OWN repository (e.g.
    /// `Path::new("spec")` resolved against its manifest dir) — the lock must live where the
    /// consumer's CI can diff and ratify it, never inside the library checkout.
    pub fn lock_in(&self, spec_dir: &Path) -> Lock {
        Lock {
            name: self.theory.to_string(),
            path: lock_path(spec_dir, self.theory),
            live: render(self),
        }
    }

    /// This spec as a `spec_lock::Lock` in THIS repo's `spec/` directory — a convenience for
    /// this repository's own freeze (`examples/freeze_spec.rs`) and staleness gate. The path
    /// is fixed at this crate's compile time (`CARGO_MANIFEST_DIR`), so for a downstream
    /// crate it would point into the cargo checkout, not the consumer's repo: consumers use
    /// [`Spec::lock_in`] with their own spec directory instead.
    pub fn lock(&self) -> Lock {
        self.lock_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec"))
    }

    /// Parse a frozen lock render back into its HELD laws — `(prose, equation)` pairs,
    /// in lock order: the inverse of `render`'s law section, for `discover::depend`'s
    /// per-consumer judgment. The undecided band is excluded deliberately (disclosed is
    /// not held — a law that moved there is no longer relied-upon-able), as are the
    /// header and coverage lines.
    pub fn parse_lock(text: &str) -> Vec<(String, String)> {
        let mut laws = Vec::new();
        let mut prose: Option<String> = None;
        for line in text.lines() {
            if line.starts_with("# undecided") {
                break;
            }
            if let Some(p) = line.strip_prefix("- ") {
                prose = Some(p.to_string());
            } else if let (true, Some(p)) = (line.starts_with("      "), prose.take()) {
                laws.push((p, line.trim().to_string()));
            }
        }
        laws
    }

    /// Check every theory's LIVE spec against its committed lock. On success returns the
    /// theory names verified fresh; on drift returns the names that no longer match (the
    /// fix is to regenerate and ratify the diff).
    ///
    /// Capability: Effectful — reads the committed spec locks from disk (a world-read,
    /// performed by `spec_lock::check`).
    pub fn check_all_fresh() -> Result<Vec<&'static str>, Vec<String>> {
        let specs = all_specs();
        let locks: Vec<Lock> = specs.iter().map(Spec::lock).collect();
        match spec_lock::check(&locks) {
            // spec-lock reports every lock fresh, in order — so the fresh names are exactly
            // the theories', still `&'static` from `Spec` rather than borrowed locally.
            Ok(_) => Ok(specs.iter().map(|s| s.theory).collect()),
            Err(stale) => Err(stale.into_iter().map(String::from).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    /// The committed spec locks are FRESH: every theory's live discovered spec matches what was
    /// frozen and ratified. A drift (an operator changed behaviour, the engine changed, a domain was
    /// edited) fails here — regenerate with `cargo run --example freeze_spec` and ratify the diff.
    #[test]
    fn the_committed_specs_are_fresh() {
        match super::Spec::check_all_fresh() {
            // all four theories' committed locks match their live discovered spec.
            Ok(fresh) => assert_eq!(
                fresh,
                vec![
                    "interpreter arithmetic",
                    "router",
                    "date calculus",
                    "ttl store",
                    "store protocol",
                    "doc flow",
                    "fabric",
                    "zset kernel"
                ]
            ),
            Err(stale) => panic!(
                "discovered spec drifted from the committed lock for: {}. \
                 Run `cargo run --example freeze_spec` and ratify the diff.",
                stale.join(", ")
            ),
        }
    }
}
