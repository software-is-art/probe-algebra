//! fire-drill — prove your gates still FIRE.
//!
//! Every pipeline accumulates gates: validators, reconciliation checks, drift gates, review
//! stamps, coverage manifests. Every gate can rot into a rubber stamp — and a vacuous gate is
//! worse than no gate, because it keeps emitting the confidence while no longer doing the
//! work. The first production adoption of this repo's discipline hit three such failures in
//! ONE system: a coverage manifest asserted by the session that produced it, a "pass" stamp
//! byte-identical whether the work happened or not, and a reconciliation that would have
//! passed on zero items checked. None of them were caught by the gates' positive tests,
//! because a rubber stamp passes every positive test.
//!
//! The fix is the mutation-testing move applied to processes: keep a standing battery of
//! KNOWN-BAD fixtures, one or more per gate, and on every run demand that each gate REJECTS
//! its planted bad input. A drill that passes is a gate that has gone vacuous, named. This is
//! the `∃`-polarity witness idea one level up — "the gate actually acts" is a statement no
//! amount of green can make; only a red that arrives on cue can.
//!
//! Substrate-free on purpose: a "gate" here is anything whose verdict you can observe — a
//! Rust function, a CLI you shell out to, a prose checklist a reviewer walks. You run the
//! gate over your known-bad fixture however you run it, and hand this crate the outcome:
//!
//! ```
//! use fire_drill::{Battery, Outcome};
//!
//! let battery = Battery::named("nightly gates")
//!     .requires(["reconciliation", "coverage manifest"])
//!     .drill("reconciliation", "an empty tree (zero items checked)", Outcome::Fired)
//!     .drill("coverage manifest", "a manifest asserting a file that does not exist", Outcome::Fired);
//!
//! battery.verdict().expect("every gate still fires");
//! ```
//!
//! `verdict()` fails, by name, on either failure mode — and the second one is the sleeper:
//!
//! - **VACUOUS**: a drill whose gate PASSED its planted bad fixture — the drills catch rot;
//! - **UNPROVEN**: a required gate with no drill at all. This census half changes the
//!   DEFAULT for new gates: a gate cannot join the pipeline without a fixture proving it
//!   can fail. In systems where gates accrete fast, that default is worth more than the
//!   drills themselves — the census prevents gates being born rotten. Lead with
//!   `requires([...])`; the drills follow.
//!
//! `render()` is deterministic text; freeze it with `spec-lock` and the battery's shape is
//! itself drift-gated, so removing a drill is a reviewed diff, not a quiet deletion.
//!
//! ## Mapping your gate's result to an `Outcome` — where consumers quietly cheat
//!
//! "Run your gate over your bad fixture however you run it" leaves one hole a consumer can
//! reintroduce vacuousness through: the mapping to [`Outcome`]. `Fired` iff exit ≠ 0 is
//! wrong twice — a USAGE error (missing input, bad flag) counts as fired though the gate
//! never judged the planted defect, and a gate can fail for an UNRELATED reason (a second
//! latent defect, an environment problem) while the drill silently stops testing what it
//! claims. The strict mapping, from a production consumer's battery: `Fired` ONLY when the
//! gate failed AND its verdict NAMES the planted defect; a clean pass is `Passed`; anything
//! else is a harness bug — panic, don't count it.
//!
//! ```
//! use fire_drill::Outcome;
//!
//! /// Fired ONLY when the gate exited fail AND the verdict names the planted defect.
//! /// Exit 0 is a vacuous pass. Anything else is a harness bug — panic, don't count it.
//! fn observe(code: i32, verdict: &str, planted: &str) -> Outcome {
//!     match code {
//!         0 => Outcome::Passed,
//!         1 => {
//!             assert!(verdict.contains(planted),
//!                 "gate failed but not for the planted defect ({planted:?}): {verdict}");
//!             Outcome::Fired
//!         }
//!         other => panic!("drill harness error: exit {other}, verdict {verdict}"),
//!     }
//! }
//! # assert_eq!(observe(0, "", "x"), Outcome::Passed);
//! # assert_eq!(observe(1, "found x", "x"), Outcome::Fired);
//! ```
//!
//! The same discipline applies to PLANTING: if your mutation helper cannot find the text it
//! plans to corrupt, panic — a mutation that silently missed its target makes the drill
//! vacuous the wrong way round (the gate "fires" on input that was never actually bad, or
//! passes on pristine input). Neither belongs in this crate as code — substrate-freedom is
//! the point — but both belong in every consumer's harness.
//!
//! Honest frame, inherited: a drill refutes vacuousness for ITS fixture only — a gate can
//! fire on the planted bad input and still miss others. The battery proves the alarm rings
//! when the button is pressed; it never proves the alarm hears everything. Grow the battery
//! the way this repo grows its shape catalog: every vacuous-pass incident becomes a drill.

use std::fmt::Write as _;

/// What the gate said about its planted known-bad fixture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// The gate REJECTED the known-bad fixture — the alarm rang. This is the good outcome.
    Fired,
    /// The gate ACCEPTED the known-bad fixture — the gate is vacuous for this input, and
    /// every green it has ever emitted is now suspect.
    Passed,
}

/// One exercise: a gate, the known-bad fixture planted on it, and what the gate said.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drill {
    /// The gate under exercise, by the name the pipeline knows it by.
    pub gate: String,
    /// The planted fixture, described so the report reads as a claim ("an empty tree —
    /// zero items checked").
    pub fixture: String,
    /// The observed verdict.
    pub outcome: Outcome,
}

/// The standing battery: every drill, plus the census of gates that MUST carry one.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Battery {
    /// The battery's display name ("nightly gates").
    pub name: String,
    /// Gates that must each carry at least one drill — the census half. A gate listed here
    /// with no drill fails the verdict as UNPROVEN, so a new gate cannot join the pipeline
    /// without a known-bad fixture.
    pub required: Vec<String>,
    /// The exercises, in declaration order.
    pub drills: Vec<Drill>,
}

impl Battery {
    /// An empty battery with a display name.
    pub fn named(name: impl Into<String>) -> Battery {
        Battery {
            name: name.into(),
            ..Battery::default()
        }
    }

    /// Declare the gates this battery must cover (appends; order preserved, duplicates
    /// collapsed).
    pub fn requires<S: Into<String>>(mut self, gates: impl IntoIterator<Item = S>) -> Battery {
        for gate in gates {
            let gate = gate.into();
            if !self.required.contains(&gate) {
                self.required.push(gate);
            }
        }
        self
    }

    /// Record one drill: the gate, the planted fixture, the observed outcome.
    pub fn drill(
        mut self,
        gate: impl Into<String>,
        fixture: impl Into<String>,
        outcome: Outcome,
    ) -> Battery {
        self.drills.push(Drill {
            gate: gate.into(),
            fixture: fixture.into(),
            outcome,
        });
        self
    }

    /// Record one drill, RUNNING its gate now: the closure executes at declaration, so
    /// the declaration order in source is the execution order — and, once the register
    /// is spec-locked, the frozen order. One expression per drill, no separated
    /// run-then-record.
    pub fn drill_with(
        self,
        gate: impl Into<String>,
        fixture: impl Into<String>,
        run: impl FnOnce() -> Outcome,
    ) -> Battery {
        let outcome = run();
        self.drill(gate, fixture, outcome)
    }

    /// The drills whose gate accepted its known-bad fixture — the vacuous gates.
    pub fn vacuous(&self) -> Vec<&Drill> {
        self.drills
            .iter()
            .filter(|d| d.outcome == Outcome::Passed)
            .collect()
    }

    /// The required gates with no drill at all — unproven, which the census refuses to
    /// let read as fine.
    pub fn unproven(&self) -> Vec<&str> {
        self.required
            .iter()
            .filter(|gate| !self.drills.iter().any(|d| &d.gate == *gate))
            .map(String::as_str)
            .collect()
    }

    /// The battery's verdict: `Ok` iff every drill fired and every required gate carries at
    /// least one drill. `Err` names each failure in the failure's own vocabulary — a
    /// VACUOUS gate passed a named bad fixture; an UNPROVEN gate has no fixture at all.
    pub fn verdict(&self) -> Result<(), String> {
        let mut failures: Vec<String> = Vec::new();
        for drill in self.vacuous() {
            failures.push(format!(
                "VACUOUS: `{}` passed a known-bad fixture ({}) — every green it emits is \
                 now suspect",
                drill.gate, drill.fixture
            ));
        }
        for gate in self.unproven() {
            failures.push(format!(
                "UNPROVEN: `{gate}` carries no known-bad fixture — nothing shows it can \
                 still fire"
            ));
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("\n"))
        }
    }

    /// The battery as deterministic text — one line per drill, census included — for
    /// freezing with `spec-lock`, so removing a drill (or a required gate) is a reviewed
    /// diff, never a quiet deletion.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# fire drill: {} — {} drill(s) over {} required gate(s); every fixture below \
             is KNOWN BAD and its gate must fire.\n",
            self.name,
            self.drills.len(),
            self.required.len()
        );
        for gate in &self.required {
            let _ = writeln!(out, "\ngate `{gate}`");
            let mut any = false;
            for drill in self.drills.iter().filter(|d| &d.gate == gate) {
                any = true;
                let verdict = match drill.outcome {
                    Outcome::Fired => "fired ",
                    Outcome::Passed => "VACUOUS",
                };
                let _ = writeln!(out, "  - {verdict}  {}", drill.fixture);
            }
            if !any {
                let _ = writeln!(out, "  - UNPROVEN  (no known-bad fixture)");
            }
        }
        for drill in self
            .drills
            .iter()
            .filter(|d| !self.required.contains(&d.gate))
        {
            let verdict = match drill.outcome {
                Outcome::Fired => "fired ",
                Outcome::Passed => "VACUOUS",
            };
            let _ = writeln!(
                out,
                "\ngate `{}` (undeclared)\n  - {verdict}  {}",
                drill.gate, drill.fixture
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn battery() -> Battery {
        Battery::named("nightly")
            .requires(["reconciliation", "coverage"])
            .drill("reconciliation", "an empty tree", Outcome::Fired)
            .drill(
                "coverage",
                "a manifest naming a missing file",
                Outcome::Fired,
            )
    }

    /// The good path: every gate fired on its planted fixture, every required gate covered.
    #[test]
    fn a_covered_firing_battery_is_green() {
        assert_eq!(battery().verdict(), Ok(()));
    }

    /// A gate that accepts its known-bad fixture is named VACUOUS — the failure this crate
    /// exists to surface, phrased so the report says why it matters.
    #[test]
    fn a_gate_passing_its_bad_fixture_is_named_vacuous() {
        let b = battery().drill(
            "reconciliation",
            "a tree with zero items checked",
            Outcome::Passed,
        );
        let err = b.verdict().unwrap_err();
        assert!(err.contains("VACUOUS: `reconciliation`"));
        assert!(err.contains("a tree with zero items checked"));
    }

    /// A required gate with no drill fails the CENSUS — a gate cannot join the pipeline
    /// without a known-bad fixture proving it can fire.
    #[test]
    fn a_required_gate_without_a_drill_is_named_unproven() {
        let b = battery().requires(["review stamp"]);
        let err = b.verdict().unwrap_err();
        assert!(err.contains("UNPROVEN: `review stamp`"));
    }

    /// Both failure modes report together, each in its own vocabulary.
    #[test]
    fn vacuous_and_unproven_report_together() {
        let b =
            Battery::named("rotten")
                .requires(["a", "b"])
                .drill("a", "bad input", Outcome::Passed);
        let err = b.verdict().unwrap_err();
        assert!(err.contains("VACUOUS: `a`"));
        assert!(err.contains("UNPROVEN: `b`"));
    }

    /// `drill_with` runs the gate AT declaration — so source order, execution order, and
    /// (once frozen) the register's order are one order, observable through a shared
    /// counter.
    #[test]
    fn drill_with_executes_in_declaration_order() {
        let log = std::cell::RefCell::new(Vec::new());
        let observe = |tag: &'static str| {
            log.borrow_mut().push(tag);
            Outcome::Fired
        };
        let b = Battery::named("ordered")
            .requires(["first", "second"])
            .drill_with("first", "a planted defect", || observe("first"))
            .drill_with("second", "another planted defect", || observe("second"));
        assert_eq!(b.verdict(), Ok(()));
        assert_eq!(*log.borrow(), vec!["first", "second"]);
        assert_eq!(
            b.drills.iter().map(|d| d.gate.as_str()).collect::<Vec<_>>(),
            vec!["first", "second"],
            "the register order is the execution order"
        );
    }

    /// The render is deterministic and lockable: census order, one line per drill, the
    /// undeclared-gate stanza kept visible rather than dropped.
    #[test]
    fn the_render_is_the_lockable_register() {
        let b = battery().drill("adhoc", "a stray fixture", Outcome::Fired);
        assert_eq!(
            b.render(),
            "# fire drill: nightly — 3 drill(s) over 2 required gate(s); every fixture below \
             is KNOWN BAD and its gate must fire.\n\
             \ngate `reconciliation`\n  - fired   an empty tree\n\
             \ngate `coverage`\n  - fired   a manifest naming a missing file\n\
             \ngate `adhoc` (undeclared)\n  - fired   a stray fixture\n"
        );
    }
}
