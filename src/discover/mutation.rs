//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! mutation — MUTATION TESTING AT THE ALGEBRA LEVEL: the spec's kill power, measured
//! in-process.
//!
//! Source-level mutation (`cargo mutants`, the pipeline's dogfood gates) is expensive
//! because every mutant is a BUILD: patch a file, recompile, run the suite — ~15 seconds a
//! mutant, hours for a tree. But for anything that is a THEORY, a mutant does not have to
//! be a build. The implementation surface discovery judges is the operator table — plain
//! `fn` evaluators — so a planted bug can be a VALUE: a perturbed copy of that table. This
//! module plants the classic mutations directly in the algebra:
//!
//!   - **confusion** — one operator evaluates as another of compatible signature
//!     (`min` evaluates as `max`; a binary evaluates as a constant of its sort);
//!   - **projection** — a binary returns an argument unchanged (`a ⊕ b` = `a`);
//!   - **partiality** — an operator becomes undefined everywhere.
//!
//! and judges each one by RE-RUNNING DISCOVERY: a mutant is KILLED iff the discovered
//! NAMED-LAW set changes — exactly the freshness gate's lock-drift semantics, applied to a
//! hypothetical implementation. Milliseconds per mutant, so the whole verdict lives in
//! `cargo test` on every change — fully shifted left, no CI economics at all.
//!
//! What a SURVIVOR means: the ratified law language cannot tell the mutant from the real
//! implementation. That is a named degree of freedom in the spec, not a test to write — the
//! founding precedent is the bias-blindness discovery: first-write-wins and last-write-wins
//! merges satisfy IDENTICAL monoid laws, an operator confusion that survived every probe
//! until the bias shapes were added to the catalog. This module industrialises exactly that
//! hunt. Survivors are therefore RATIFIED, not failed: [`MutationReport`] freezes into
//! `spec/<theory>.mutation.spec` under the standard lock discipline, so a NEW survivor (or
//! a fixed one) is a reviewed diff naming what the spec can and cannot pin.
//!
//! Judgment is over the NAMED laws only — not the consequence count or any other census
//! line a spec render carries. The counts would kill almost everything (they change with
//! nearly any behavioural difference) and thereby hide the interesting verdicts; the named
//! laws are what a human ratified and what expectations declare, so "the named laws cannot
//! tell them apart" is the honest, actionable claim. Grid-bounded like all discovery: a
//! kill is definite (the law set provably moved), a survival is "indistinguishable ON THIS
//! GRID", never a proof of equivalence.

use std::path::{Path, PathBuf};

use spec_lock::Lock;

use super::engine::{Engine, EvalFn, Theory};

/// A binary's left-projection mutant: `a ⊕ b` = `a`. (Also a unary's identity mutant.)
fn proj0<T: Theory>(v: &[T::Value]) -> Option<T::Value> {
    Some(v[0].clone())
}

/// A binary's right-projection mutant: `a ⊕ b` = `b`.
fn proj1<T: Theory>(v: &[T::Value]) -> Option<T::Value> {
    Some(v[1].clone())
}

/// The partiality mutant: defined nowhere.
fn never<T: Theory>(_: &[T::Value]) -> Option<T::Value> {
    None
}

/// One operator-table mutant: what was planted, and the full replacement evaluator table.
struct AlgebraMutant<T: Theory> {
    description: String,
    evals: Vec<EvalFn<T>>,
}

/// The in-process mutation verdict for one theory: every operator-table mutant planted, and
/// whether re-discovery killed it. Survivors are the spec's named degrees of freedom.
pub struct MutationReport {
    /// The theory's display name.
    pub theory: &'static str,
    /// `(what was planted, killed?)`, in deterministic generation order.
    pub verdicts: Vec<(String, bool)>,
}

/// The named-law identity of a spec — what judgment compares. Prose and equation together,
/// so a law is its ratified rendering, exactly as the module lock states it.
fn law_set<T: Theory>(engine: &Engine<T>) -> Vec<String> {
    engine
        .discover()
        .laws
        .iter()
        .map(|l| format!("{} {}", l.prose, l.equation))
        .collect()
}

/// Every argument tuple for an operator's input sorts, capped — the sample two evaluators
/// are compared on before a confusion mutant is planted (behaviourally identical evaluators
/// make an equivalent-by-construction mutant: noise, not a spec gap).
fn input_tuples<T: Theory>(inputs: &[T::Sort], cap: usize) -> Vec<Vec<T::Value>> {
    let pools: Vec<Vec<T::Value>> = inputs.iter().map(|s| T::inhabitants(*s)).collect();
    let mut tuples: Vec<Vec<T::Value>> = vec![Vec::new()];
    for pool in &pools {
        let mut next = Vec::new();
        for t in &tuples {
            for v in pool {
                let mut t = t.clone();
                t.push(v.clone());
                next.push(t);
                if next.len() >= cap {
                    break;
                }
            }
            if next.len() >= cap {
                break;
            }
        }
        tuples = next;
    }
    tuples
}

/// Do two evaluators behave identically on every sampled tuple? (Observationally, like all
/// judgment here.)
fn same_conduct<T: Theory>(a: EvalFn<T>, b: EvalFn<T>, tuples: &[Vec<T::Value>]) -> bool {
    tuples.iter().all(|t| {
        let oa = a(t).map(|v| T::observe(&v));
        let ob = b(t).map(|v| T::observe(&v));
        oa == ob
    })
}

/// Generate the mutant battery for a theory's operator table. Deterministic order:
/// confusions (by target then source index), projections, partialities.
fn mutants<T: Theory>(engine: &Engine<T>) -> Vec<AlgebraMutant<T>> {
    let sigs = engine.signatures();
    let evals = engine.evals();
    let mut out: Vec<AlgebraMutant<T>> = Vec::new();

    // confusion: operator `i` evaluates as operator `j`. Admissible when the output sorts
    // match and `j`'s input sorts are a PREFIX of `i`'s (a prefix-compatible evaluator can
    // only read argument positions `i` actually supplies — a constant under a binary is
    // fine, a binary under a unary would read past the arguments).
    for i in 0..sigs.len() {
        let tuples = input_tuples::<T>(&sigs[i].1, 64);
        for j in 0..sigs.len() {
            let compatible = i != j
                && sigs[j].2 == sigs[i].2
                && sigs[j].1.len() <= sigs[i].1.len()
                && sigs[i].1[..sigs[j].1.len()] == sigs[j].1[..];
            if !compatible || same_conduct::<T>(evals[i], evals[j], &tuples) {
                continue;
            }
            let mut m = evals.clone();
            m[i] = evals[j];
            out.push(AlgebraMutant {
                description: format!("`{}` evaluates as `{}`", sigs[i].0, sigs[j].0),
                evals: m,
            });
        }
    }

    // projection: a binary returns one argument unchanged (a unary endo returns its input).
    for i in 0..sigs.len() {
        let tuples = input_tuples::<T>(&sigs[i].1, 64);
        let projections: &[(usize, EvalFn<T>, &str)] =
            &[(0, proj0::<T>, "first"), (1, proj1::<T>, "second")];
        for (k, proj, which) in projections {
            let admissible = sigs[i].1.len() > *k && sigs[i].1[*k] == sigs[i].2;
            if !admissible || same_conduct::<T>(evals[i], *proj, &tuples) {
                continue;
            }
            let mut m = evals.clone();
            m[i] = *proj;
            out.push(AlgebraMutant {
                description: if sigs[i].1.len() == 1 {
                    format!("`{}` returns its input unchanged", sigs[i].0)
                } else {
                    format!("`{}` returns its {which} argument unchanged", sigs[i].0)
                },
                evals: m,
            });
        }
    }

    // partiality: an operator becomes undefined everywhere.
    for i in 0..sigs.len() {
        let tuples = input_tuples::<T>(&sigs[i].1, 64);
        if same_conduct::<T>(evals[i], never::<T>, &tuples) {
            continue;
        }
        let mut m = evals.clone();
        m[i] = never::<T>;
        out.push(AlgebraMutant {
            description: format!("`{}` becomes undefined everywhere", sigs[i].0),
            evals: m,
        });
    }

    out
}

impl MutationReport {
    /// Plant the battery against `T`'s operator table and judge every mutant by
    /// re-discovery. This is the whole harness: build once (the engine), then each mutant
    /// is a value and each verdict a re-run of discovery over the same grid.
    pub fn of<T: Theory>() -> MutationReport {
        let engine = Engine::<T>::new();
        let baseline = law_set(&engine);
        let verdicts = mutants(&engine)
            .into_iter()
            .map(|m| {
                let killed = law_set(&engine.with_evals(&m.evals)) != baseline;
                (m.description, killed)
            })
            .collect();
        MutationReport {
            theory: T::name(),
            verdicts,
        }
    }

    /// The mutants the named laws could NOT tell from the real implementation — each one a
    /// degree of freedom the spec leaves open (the ratified text below names them).
    pub fn survivors(&self) -> Vec<&str> {
        self.verdicts
            .iter()
            .filter(|(_, killed)| !killed)
            .map(|(d, _)| d.as_str())
            .collect()
    }

    /// The canonical text `spec/<theory>.mutation.spec` locks: the verdict per mutant,
    /// survivors shouting.
    pub fn render(&self) -> String {
        let survivors = self.survivors().len();
        let mut out = format!(
            "# algebra mutation: {} — {} operator-table mutants, {} — regenerate via `cargo run --example freeze_spec`; ratify the diff.\n\
             #\n\
             # Every mutant is a perturbed operator table (a VALUE, not a build), judged by\n\
             # re-running discovery: KILLED means the named-law set changed — the committed\n\
             # spec would go stale against an implementation with this bug. A SURVIVOR means\n\
             # the ratified law language cannot tell the mutant from the real thing on this\n\
             # grid — a named degree of freedom (the bias-blindness precedent), to be closed\n\
             # by a sharper shape or expectation, or ratified here as a free choice.\n",
            self.theory,
            self.verdicts.len(),
            if survivors == 0 {
                "all killed".to_string()
            } else {
                format!("{survivors} SURVIVED")
            }
        );
        for (description, killed) in &self.verdicts {
            if *killed {
                out.push_str(&format!("\n- killed    {description}"));
            } else {
                out.push_str(&format!("\n- SURVIVED  {description}"));
            }
        }
        out.push('\n');
        out
    }

    /// This report as a `spec_lock::Lock` rooted in a caller-supplied spec directory — the
    /// sibling of `Spec::lock_in` / `WorldReport::lock_in`, at
    /// `spec_dir/<slugified-theory>.mutation.spec`.
    pub fn lock_in(&self, spec_dir: &Path) -> Lock {
        let slug: String = self
            .theory
            .to_lowercase()
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        Lock {
            name: format!("{} algebra mutation", self.theory),
            path: spec_dir.join(format!("{slug}.mutation.spec")),
            live: self.render(),
        }
    }

    /// This report as a lock in THIS repo's `spec/` directory (consumers use
    /// [`MutationReport::lock_in`] with their own directory).
    pub fn lock(&self) -> Lock {
        self.lock_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::date::Calendar;
    use crate::discover::router::Router;
    use crate::discover::world::StoreProtocol;
    use crate::kvstore::theory::TtlStore;

    // A deliberately WEAK theory: two distinguishable constants and not one law — no
    // binary, no unary, no action, no relation, so the whole catalog is silent about it.
    // Its survivors are therefore REAL, which the registry theories (all survivors closed)
    // can no longer supply: the fixture that keeps `survivors()` and the SURVIVED render
    // honest (a report that lies "no survivors" was invisible to every green theory — a
    // mutant caught by the changed-lines dogfood sweep).
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct M;
    #[derive(Clone)]
    struct MV(u8);
    struct Mute;
    fn one_op(_: &[MV]) -> Option<MV> {
        Some(MV(1))
    }
    fn two_op(_: &[MV]) -> Option<MV> {
        Some(MV(2))
    }

    crate::theory! {
        Mute : "mute",
        Value = MV,
        Obs = u8,
        Sort = M,
        sort_of = |_: &MV| M,
        observe = |v: &MV| v.0,
        vars { M => &["x", "y", "z"], }
        inhabit { M => vec![MV(1), MV(2)], }
        ops {
            Nullary "one" "one" () -> M = one_op;
            Nullary "two" "two" () -> M = two_op;
        }
    }

    /// A spec that says nothing kills nothing — and the report SAYS so. The mute theory's
    /// constants appear in no law, so confusing them (and starving them) survives, the
    /// survivor list names each planted bug, and the render shouts it. This is the
    /// negative direction the all-green registry can no longer pin.
    #[test]
    fn a_lawless_spec_has_named_survivors() {
        let report = MutationReport::of::<Mute>();
        assert_eq!(
            report.survivors(),
            vec![
                "`one` evaluates as `two`",
                "`two` evaluates as `one`",
                "`one` becomes undefined everywhere",
                "`two` becomes undefined everywhere",
            ]
        );
        let render = report.render();
        assert!(render.contains("4 SURVIVED"));
        assert!(render.contains("\n- SURVIVED  `one` evaluates as `two`"));
    }

    /// The five committed mutation locks are fresh — the drift gate. A spec change that
    /// alters any theory's kill power (a new survivor, a closed one, a new mutant from a
    /// new operator) fails HERE until the regenerated report is ratified.
    #[test]
    fn the_committed_mutation_locks_are_fresh() {
        let locks = [
            MutationReport::of::<Arithmetic>().lock(),
            MutationReport::of::<Router>().lock(),
            MutationReport::of::<Calendar>().lock(),
            MutationReport::of::<TtlStore>().lock(),
            MutationReport::of::<StoreProtocol>().lock(),
        ];
        if let Err(stale) = spec_lock::check(&locks) {
            panic!(
                "the algebra-mutation verdicts drifted: {}. Regenerate with \
                 `cargo run --example freeze_spec` and ratify the diff — a new survivor \
                 is a named degree of freedom, a closed one is a spec improvement.",
                stale.join(", ")
            );
        }
    }

    /// ZERO SURVIVORS, pinned — the loop this module exists for, closed once already. Its
    /// first run found four survivors, all of one deep kind (equational laws cannot state
    /// INEQUATIONS): the trivial action (`add`/`tick` unchanged satisfies every action
    /// equation vacuously), the never-true relation (`<` pinned to constant-false
    /// satisfies irreflexivity), and the unpinned operator (`diff` as constant-zero,
    /// named by no law). The WITNESS shapes ("action nontriviality", "non-constancy")
    /// were added to the catalog in response, discovery found the closing inequations
    /// itself on the next freeze, and every survivor died — the fix was autogenerated by
    /// extending the vocabulary, not by writing per-theory tests. A regression here means
    /// a spec stopped saying one of those things.
    #[test]
    fn every_operator_table_mutant_is_killed() {
        for (theory, survivors) in [
            (
                "interpreter arithmetic",
                MutationReport::of::<Arithmetic>().survivors().len(),
            ),
            ("router", MutationReport::of::<Router>().survivors().len()),
            (
                "date calculus",
                MutationReport::of::<Calendar>().survivors().len(),
            ),
            (
                "ttl store",
                MutationReport::of::<TtlStore>().survivors().len(),
            ),
            (
                "store protocol",
                MutationReport::of::<StoreProtocol>().survivors().len(),
            ),
        ] {
            assert_eq!(
                survivors, 0,
                "`{theory}` has new algebra-mutation survivors"
            );
        }
    }

    /// The judgment is lock-drift semantics end to end: a killed mutant is exactly one
    /// whose law set differs, and the planted battery is non-trivial for every registry
    /// theory (at least a confusion or projection AND the partiality mutants exist).
    #[test]
    fn every_registry_theory_gets_a_real_battery() {
        for (theory, report) in [
            ("router", MutationReport::of::<Router>()),
            ("date calculus", MutationReport::of::<Calendar>()),
            ("ttl store", MutationReport::of::<TtlStore>()),
            ("store protocol", MutationReport::of::<StoreProtocol>()),
        ] {
            assert!(
                report.verdicts.len() >= 3,
                "`{theory}` planted a suspiciously thin battery: {:?}",
                report.verdicts
            );
            assert!(
                report.verdicts.iter().any(|(d, _)| d.contains("undefined")),
                "`{theory}` is missing its partiality mutants"
            );
            assert!(
                report
                    .verdicts
                    .iter()
                    .any(|(d, killed)| *killed && d.contains("evaluates as")),
                "`{theory}` kills no confusion mutant — the spec should at least tell \
                 an operator from a constant"
            );
        }
    }
}
