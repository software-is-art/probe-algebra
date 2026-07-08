//!
//! mutation — MUTATION TESTING AT THE ALGEBRA LEVEL: the spec's kill power, measured
//! in-process.
//!
//! Source-level mutation (`cargo mutants`, the pipeline's dogfood gates) is expensive
//! because every mutant is a BUILD: patch a file, recompile, run the suite — ~15 seconds a
//! mutant, hours for a tree. But for anything that is a THEORY, a mutant does not have to
//! be a build. The implementation surface discovery judges is the operator table — plain
//! `fn` evaluators — so a planted bug can be a VALUE: a perturbed copy of that table.
//! Because a function on a bounded grid IS its value table, "how different from the real
//! thing" becomes a DISTANCE, and the batteries here sit at three points on that dial:
//!
//!   - **deafness** (the noise floor, farthest) — the operator stops listening to its
//!     inputs and returns one constant everywhere, one mutant per distinct output value.
//!     Answers: is this operator's output constrained to depend on its input AT ALL?
//!     (Derandomised noise: on enumerable codomains, every constant beats a coin flip.)
//!   - **the table battery** (mid-distance) — **confusion** (one operator evaluates as
//!     another of compatible signature: `min` as `max`), **projection** (`a ⊕ b` = `a`),
//!     **partiality** (undefined everywhere). Answers: do the laws tell the operators
//!     apart?
//!   - **the dent sweep** (nearest — the minimal meaning change on the sampled domain) —
//!     exactly one input tuple returns a wrong value, everything else untouched. A
//!     surviving dent is not a source line, it is a COORDINATE: the exact input whose
//!     output no ratified law constrains — the unpinned region of the domain, mapped.
//!
//! The table battery is judged by RE-RUNNING DISCOVERY: a mutant is KILLED iff the
//! discovered NAMED-LAW set changes — exactly the freshness gate's lock-drift semantics,
//! applied to a hypothetical implementation (an appearing law kills too). The surgical
//! layers (deafness, dents) are judged by RE-CHECKING the discovered laws — at hundreds
//! of mutants per theory, the appearing-law direction is traded for a judgment that stays
//! inside every `cargo test`; the check is exact over the laws NAMING the mutated
//! operator, since a term evaluates only the operators it names. Milliseconds per mutant
//! either way — fully shifted left, no CI economics at all. (The fourth distance — the
//! plausible-typo neighbours of free-form interior Rust, where domains are not
//! enumerable — stays with the source-level sweeps.)
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

use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use spec_lock::Lock;

use super::engine::{DiscoveredLaw, Engine, EvalFn, Theory};

/// How many grid points each operator is dented at (the first N sampled input tuples,
/// a resource bound like every grid here — never a curated list).
const DENT_POINTS_PER_OP: usize = 16;
/// How many wrong outputs are tried per dented point (the first N inhabitants whose
/// observation differs from the true output).
const DENT_WRONGS_PER_POINT: usize = 2;

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

thread_local! {
    /// The active SURGICAL mutant on this thread, read by [`surgical_eval`]. A
    /// `thread_local` cannot be generic, so the state is type-erased and the trampoline
    /// downcasts it back — the engine keeps its bare-`fn` evaluator table (the design
    /// the whole term evaluator runs on), and a mutant WITH CAPTURED STATE stays a
    /// value anyway. Tests on other threads never see it.
    static SURGERY: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
}

/// A perturbation applied OVER the real evaluator — the captured state deafness and
/// dents need, which a bare `fn` cannot carry.
struct Surgery<T: Theory> {
    real: EvalFn<T>,
    kind: SurgeryKind<T>,
}

enum SurgeryKind<T: Theory> {
    /// DEAFNESS — the noise floor, derandomised: the operator stops listening to its
    /// inputs and returns one constant everywhere. Random noise would make detection a
    /// coin flip on small codomains; enumerating every distinct constant is the same
    /// destruction of input-dependence with a deterministic verdict.
    Deaf { value: T::Value },
    /// A DENT — the minimal meaning change on the sampled domain: exactly one input
    /// tuple (matched observationally) returns a wrong value; every other point keeps
    /// the real behaviour. A surviving dent is not a source line, it is a COORDINATE:
    /// the input whose output no ratified law constrains.
    Dent { point: Vec<T::Obs>, wrong: T::Value },
}

/// The trampoline: a plain [`EvalFn`] (what the operator table stores) that applies the
/// thread-local surgery over the real evaluator. Installed at exactly one operator
/// index per mutant; the surgery is always set for the duration of a judgment.
fn surgical_eval<T>(args: &[T::Value]) -> Option<T::Value>
where
    T: Theory + 'static,
    T::Value: 'static,
    T::Obs: 'static,
{
    SURGERY.with(|s| {
        let borrow = s.borrow();
        let surgery = borrow.as_ref()?.downcast_ref::<Surgery<T>>()?;
        match &surgery.kind {
            SurgeryKind::Deaf { value } => Some(value.clone()),
            SurgeryKind::Dent { point, wrong } => {
                let here: Vec<T::Obs> = args.iter().map(T::observe).collect();
                if here == *point {
                    Some(wrong.clone())
                } else {
                    (surgery.real)(args)
                }
            }
        }
    })
}

/// Judge one surgery at operator index `i`: install it, re-CHECK the ratified laws
/// against the trampolined table, uninstall. Killed iff a law refutes it — the frozen
/// laws are the probe (see `Engine::check`), which is the honest judgment for these
/// layers: "does the COMMITTED spec pin this?" (Re-discovery, the table battery's
/// judgment, would also notice new laws appearing — but at ~three orders of magnitude
/// more mutants, checking is the price that lets these layers ride every `cargo test`.)
///
/// `laws` must already be filtered to those NAMING operator `i` — an exact economy,
/// not an approximation: a term evaluates only the operators it names, so a law that
/// never names `i` cannot reach the surgery and its verdict cannot change.
fn judge_surgery<T>(
    engine: &Engine<T>,
    laws: &[DiscoveredLaw],
    evals: &[EvalFn<T>],
    i: usize,
    surgery: Surgery<T>,
) -> bool
where
    T: Theory + 'static,
    T::Value: 'static,
    T::Obs: 'static,
{
    let mut m = evals.to_vec();
    m[i] = surgical_eval::<T>;
    let mutant = engine.with_evals(&m);
    SURGERY.with(|s| *s.borrow_mut() = Some(Box::new(surgery)));
    let killed = mutant.check(laws).is_err();
    SURGERY.with(|s| *s.borrow_mut() = None);
    killed
}

/// The laws that NAME each operator, by operator index — the exact set a surgery at
/// that index can disturb.
fn laws_naming<T: Theory>(engine: &Engine<T>, laws: &[DiscoveredLaw]) -> Vec<Vec<DiscoveredLaw>> {
    let symbols: Vec<&'static str> = engine.signatures().iter().map(|s| s.0).collect();
    symbols
        .iter()
        .map(|sym| {
            laws.iter()
                .filter(|l| l.ops(&symbols).contains(sym))
                .cloned()
                .collect()
        })
        .collect()
}

/// The in-process mutation verdict for one theory: every operator-table mutant planted, and
/// whether re-discovery killed it. Survivors are the spec's named degrees of freedom.
pub struct MutationReport {
    /// The theory's display name.
    pub theory: &'static str,
    /// `(what was planted, killed?)`, in deterministic generation order.
    pub verdicts: Vec<(String, bool)>,
    /// The DEAFNESS floor: `(what was planted, killed?)` for every constant-return
    /// mutant — is each operator's output constrained to depend on its input at all?
    pub deaf: Vec<(String, bool)>,
    /// The DENT sweep: `(what was planted, killed?)` for every one-point perturbation —
    /// a survivor names the exact grid coordinate the ratified laws do not pin.
    pub dents: Vec<(String, bool)>,
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

/// The deafness battery for one operator: one mutant per distinct-observation
/// inhabitant of the output sort, skipping constants the operator already behaves as
/// on the sampled tuples (equivalent by construction — noise, not a spec gap).
fn deaf_battery<T>(
    engine: &Engine<T>,
    laws: &[DiscoveredLaw],
    evals: &[EvalFn<T>],
) -> Vec<(String, bool)>
where
    T: Theory + 'static,
    T::Value: 'static,
    T::Obs: Debug + 'static,
{
    let sigs = engine.signatures();
    let per_op = laws_naming(engine, laws);
    let mut out = Vec::new();
    for i in 0..sigs.len() {
        if sigs[i].1.is_empty() {
            continue; // a constant cannot go deaf: it never listened.
        }
        let tuples = input_tuples::<T>(&sigs[i].1, 64);
        let mut seen: Vec<T::Obs> = Vec::new();
        for value in T::inhabitants(sigs[i].2) {
            let obs = T::observe(&value);
            if seen.contains(&obs) {
                continue;
            }
            seen.push(obs.clone());
            let already = tuples
                .iter()
                .all(|t| evals[i](t).map(|v| T::observe(&v)) == Some(obs.clone()));
            if already {
                continue;
            }
            let killed = judge_surgery(
                engine,
                &per_op[i],
                evals,
                i,
                Surgery::<T> {
                    real: evals[i],
                    kind: SurgeryKind::Deaf {
                        value: value.clone(),
                    },
                },
            );
            out.push((format!("`{}` goes deaf: always {obs:?}", sigs[i].0), killed));
        }
    }
    out
}

/// The dent battery for one operator: perturb one sampled input tuple at a time,
/// trying the first wrong outputs whose observation differs from the true one. Points
/// where the operator is undefined are skipped (a definedness change is the partiality
/// mutant's job, at whole-operator granularity).
fn dent_battery<T>(
    engine: &Engine<T>,
    laws: &[DiscoveredLaw],
    evals: &[EvalFn<T>],
) -> Vec<(String, bool)>
where
    T: Theory + 'static,
    T::Value: 'static,
    T::Obs: Debug + 'static,
{
    let sigs = engine.signatures();
    let per_op = laws_naming(engine, laws);
    let mut out = Vec::new();
    for i in 0..sigs.len() {
        if sigs[i].1.is_empty() {
            continue; // a constant's one point deaf-mutates above; a dent adds nothing.
        }
        let tuples = input_tuples::<T>(&sigs[i].1, DENT_POINTS_PER_OP);
        for tuple in &tuples {
            let Some(truth) = evals[i](tuple).map(|v| T::observe(&v)) else {
                continue;
            };
            let point: Vec<T::Obs> = tuple.iter().map(T::observe).collect();
            let mut tried = 0usize;
            let mut seen: Vec<T::Obs> = Vec::new();
            for wrong in T::inhabitants(sigs[i].2) {
                let obs = T::observe(&wrong);
                if obs == truth || seen.contains(&obs) {
                    continue;
                }
                seen.push(obs.clone());
                let killed = judge_surgery(
                    engine,
                    &per_op[i],
                    evals,
                    i,
                    Surgery::<T> {
                        real: evals[i],
                        kind: SurgeryKind::Dent {
                            point: point.clone(),
                            wrong: wrong.clone(),
                        },
                    },
                );
                out.push((
                    format!("`{}` dented at {point:?}: {truth:?} -> {obs:?}", sigs[i].0),
                    killed,
                ));
                tried += 1;
                if tried >= DENT_WRONGS_PER_POINT {
                    break;
                }
            }
        }
    }
    out
}

#[crate::mutate]
impl MutationReport {
    /// Plant the batteries against `T`'s operator table. The TABLE battery (confusion,
    /// projection, partiality) is judged by re-discovery — the law SET must move. The
    /// deafness floor and the dent sweep are judged by re-CHECKING the discovered laws
    /// — the committed spec must refute them. Build once (the engine); every mutant is
    /// a value and every verdict a re-run over the same grid.
    pub fn of<T>() -> MutationReport
    where
        T: Theory + 'static,
        T::Value: 'static,
        T::Obs: Debug + 'static,
    {
        let engine = Engine::<T>::new();
        let laws = engine.discover().laws;
        let baseline: Vec<String> = laws
            .iter()
            .map(|l| format!("{} {}", l.prose, l.equation))
            .collect();
        let verdicts = mutants(&engine)
            .into_iter()
            .map(|m| {
                let killed = law_set(&engine.with_evals(&m.evals)) != baseline;
                (m.description, killed)
            })
            .collect();
        let evals = engine.evals();
        MutationReport {
            theory: T::name(),
            verdicts,
            deaf: deaf_battery(&engine, &laws, &evals),
            dents: dent_battery(&engine, &laws, &evals),
        }
    }

    /// The table mutants the named laws could NOT tell from the real implementation —
    /// each one a degree of freedom the spec leaves open (the ratified text names them).
    pub fn survivors(&self) -> Vec<&str> {
        self.verdicts
            .iter()
            .filter(|(_, killed)| !killed)
            .map(|(d, _)| d.as_str())
            .collect()
    }

    /// Operators whose output the committed laws do not constrain to depend on the
    /// input at all — the noise floor's findings.
    pub fn deaf_survivors(&self) -> Vec<&str> {
        self.deaf
            .iter()
            .filter(|(_, killed)| !killed)
            .map(|(d, _)| d.as_str())
            .collect()
    }

    /// The UNPINNED REGION of the sampled domain: each survivor is a coordinate — an
    /// input tuple whose output the committed laws leave free. The map of where a
    /// missing probe would go.
    pub fn dent_survivors(&self) -> Vec<&str> {
        self.dents
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
            "# algebra mutation: {} — {} operator-table mutants, {} — regenerate via this repo's freeze path; ratify the diff.\n\
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

        // The surgical layers render as CENSUS + SURVIVORS: the kill lists would bloat
        // the lock without informing review (a killed dent is the expected case), but
        // every survivor is a finding with coordinates, and the counts pin the battery
        // size so a silently shrunken sweep is lock drift.
        let deaf_survivors = self.deaf_survivors();
        out.push_str(&format!(
            "\n# deafness floor: {} constant-return mutants (every operator × every distinct\n\
             # output), judged by re-checking the discovered laws — {}.\n",
            self.deaf.len(),
            if deaf_survivors.is_empty() {
                "all killed: every operator's output provably depends on its input".to_string()
            } else {
                format!("{} SURVIVED", deaf_survivors.len())
            }
        ));
        for survivor in &deaf_survivors {
            out.push_str(&format!("- SURVIVED  {survivor}\n"));
        }
        let dent_survivors = self.dent_survivors();
        out.push_str(&format!(
            "\n# dent sweep: {} one-point mutants (first {DENT_POINTS_PER_OP} grid points per \
             operator,\n\
             # {DENT_WRONGS_PER_POINT} wrong outputs per point — a resource bound, not a curated \
             list), judged by\n\
             # re-checking the discovered laws — {}.\n",
            self.dents.len(),
            if dent_survivors.is_empty() {
                "all killed: every sampled point is pinned".to_string()
            } else {
                format!(
                    "{} SURVIVED — each an UNPINNED COORDINATE, \
                     the exact input a missing probe would constrain",
                    dent_survivors.len()
                )
            }
        ));
        for survivor in &dent_survivors {
            out.push_str(&format!("- SURVIVED  {survivor}\n"));
        }
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
    use crate::discover::fabric::Fabric;
    use crate::discover::router::Router;
    use crate::discover::world::StoreProtocol;
    use crate::kvstore::theory::TtlStore;

    // A deliberately WEAK theory: two distinguishable constants, one three-cycle unary
    // (`spin`: 1→2→3→1 — not an involution, not a projection, so no unary shape can
    // name it), and not one law. Its survivors are therefore REAL across all three
    // batteries, which the registry theories (all table survivors closed) can no
    // longer supply: the fixture that keeps `survivors()`, the surgical survivor
    // accessors, and the SURVIVED renders honest (a report that lies "no survivors"
    // was invisible to every green theory — a mutant caught by the changed-lines
    // dogfood sweep).
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
    fn spin_op(v: &[MV]) -> Option<MV> {
        Some(MV(v[0].0 % 3 + 1))
    }

    crate::theory! {
        Mute : "mute",
        Value = MV,
        Obs = u8,
        Sort = M,
        sort_of = |_: &MV| M,
        observe = |v: &MV| v.0,
        vars { M => &["x", "y", "z"], }
        inhabit { M => vec![MV(1), MV(2), MV(3)], }
        ops {
            Nullary "one"  "one"  () -> M = one_op;
            Nullary "two"  "two"  () -> M = two_op;
            Prefix  "spin" "spin" (M) -> M = spin_op;
        }
    }

    /// A spec that says nothing kills nothing — and the report SAYS so, across all
    /// three batteries. The mute theory's constants appear in no law, so confusing
    /// them (and starving them) survives; `spin` is named by no law, so every
    /// deafness constant and every one-point dent survives WITH ITS COORDINATE —
    /// the unpinned-region map in miniature. (The confusion/projection mutants of
    /// `spin` die by law APPEARANCE: a constant or identity unary suddenly satisfies
    /// projection/involution, and the table battery's re-discovery judgment notices a
    /// law set that grew — which the check-judged surgical layers deliberately do not.)
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
                "`spin` becomes undefined everywhere",
            ]
        );
        assert_eq!(
            report.deaf_survivors(),
            vec![
                "`spin` goes deaf: always 1",
                "`spin` goes deaf: always 2",
                "`spin` goes deaf: always 3",
            ]
        );
        assert_eq!(
            report.dent_survivors(),
            vec![
                "`spin` dented at [1]: 2 -> 1",
                "`spin` dented at [1]: 2 -> 3",
                "`spin` dented at [2]: 3 -> 1",
                "`spin` dented at [2]: 3 -> 2",
                "`spin` dented at [3]: 1 -> 2",
                "`spin` dented at [3]: 1 -> 3",
            ]
        );
        let render = report.render();
        assert!(render.contains("5 SURVIVED"));
        assert!(render.contains("\n- SURVIVED  `one` evaluates as `two`"));
        assert!(render.contains("3 SURVIVED"), "the deafness census shouts");
        assert!(
            render.contains("6 SURVIVED — each an UNPINNED COORDINATE"),
            "{render}"
        );
        assert!(render.contains("- SURVIVED  `spin` dented at [2]: 3 -> 1\n"));
    }

    /// The committed mutation locks are fresh — the drift gate. A spec change that
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
            MutationReport::of::<Fabric>().lock(),
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
            ("fabric", MutationReport::of::<Fabric>().survivors().len()),
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
            ("fabric", MutationReport::of::<Fabric>()),
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
            // the surgical layers BITE on every registry theory: at least one
            // deafness constant and at least one dent are refuted by the committed
            // laws (all-survive would mean the check-judged path went silent).
            assert!(
                report.deaf.iter().any(|(_, killed)| *killed),
                "`{theory}` kills no deafness mutant — no law constrains any \
                 operator's output to depend on its input?"
            );
            assert!(
                report.dents.iter().any(|(_, killed)| *killed),
                "`{theory}` kills no dent — not one sampled point is pinned?"
            );
        }
    }
}
