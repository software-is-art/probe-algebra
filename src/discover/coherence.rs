//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! coherence — does a value's ALGEBRA survive a seam between modules?
//!
//! The type system already answers "do these two modules CONNECT?" — type, witness, and grading
//! compatibility are total at compile time. It does NOT answer "do they AGREE?": two modules can be
//! perfectly connectable and silently incoherent, meaning different things about a value type they
//! share. Module A merges with `max` (commutative); module B merges the same type first-match
//! (NOT commutative). Wiring A's output into B type-checks, compiles, stays within budget — and is
//! subtly wrong, because the modules disagree about an algebra the types never named.
//!
//! Coherence is the behavioural analog of `gdp`'s proof-carrying seam: gdp carries a value's PROOF
//! across a seam (`WellTyped`, `InBounds`); this checks a value's LAWS survive it. Two same-signature
//! theories are COHERENT iff every law one discovers also holds under the other's operators — checked
//! by re-running one theory's discovered laws through the other's `Engine` (`check`). A surviving
//! disagreement is a coherence bug the types can't see. (Grid-bounded, like all discovery: it finds
//! DISAGREEMENT, it does not prove coherence.)

use super::engine::{Engine, Theory};

/// One sort: an integer-valued "key" several modules merge.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Key,
}

fn val(v: &i64) -> i64 {
    *v
}
fn empty(_: &[i64]) -> Option<i64> {
    Some(0)
}
fn max_merge(v: &[i64]) -> Option<i64> {
    Some(val(&v[0]).max(val(&v[1])))
}
fn gcd_merge(v: &[i64]) -> Option<i64> {
    let (mut a, mut b) = (val(&v[0]), val(&v[1]));
    while b != 0 {
        (a, b) = (b, a % b);
    }
    Some(a)
}
fn first_merge(v: &[i64]) -> Option<i64> {
    // first-match: the left key wins unless it is "empty" (0). NOT commutative.
    Some(if val(&v[0]) != 0 {
        val(&v[0])
    } else {
        val(&v[1])
    })
}

/// Three theories with the SAME signature (`merge : Key × Key -> Key`, identity `empty = 0`) but
/// different merge semantics — so a law's term indices are valid across all three.
pub struct MaxMerge;
pub struct GcdMerge;
pub struct FirstMerge;

macro_rules! merge_theory {
    ($thy:ty, $name:literal, $merge:expr) => {
        crate::theory! {
            $thy : $name, Value = i64, Obs = i64, Sort = Sort,
            sort_of = |_: &i64| Sort::Key,
            observe = |v: &i64| *v,
            vars { Sort::Key => &["a", "b", "c"], }
            inhabit { Sort::Key => vec![0, 1, 2, 3, 4, 6, 12], }
            ops {
                Nullary "Empty" "empty" () -> Sort::Key = empty;
                Infix   "Merge" "merge" (Sort::Key, Sort::Key) -> Sort::Key = $merge;
            }
        }
    };
}
merge_theory!(MaxMerge, "max-merge", max_merge);
merge_theory!(GcdMerge, "gcd-merge", gcd_merge);
merge_theory!(FirstMerge, "first-merge", first_merge);

/// The coherence violations between two same-signature theories: the laws one module discovers that
/// do NOT hold under the other's operators (checked both directions). Empty ⇒ the modules agree
/// about the shared algebra; non-empty ⇒ they are connectable but INCOHERENT.
pub fn coherence_violations<A, B>() -> Vec<String>
where
    A: Theory,
    B: Theory<Sort = A::Sort, Value = A::Value, Obs = A::Obs>,
{
    let (ea, eb) = (Engine::<A>::new(), Engine::<B>::new());
    let mut out = Vec::new();
    for law in ea.discover().laws {
        if eb.check(std::slice::from_ref(&law)).is_err() {
            out.push(format!(
                "\"{}\" holds in {} but not {}",
                law.prose,
                A::name(),
                B::name()
            ));
        }
    }
    for law in eb.discover().laws {
        if ea.check(std::slice::from_ref(&law)).is_err() {
            out.push(format!(
                "\"{}\" holds in {} but not {}",
                law.prose,
                B::name(),
                A::name()
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// COHERENT despite different operators: `max` and `gcd` are both commutative, associative,
    /// idempotent monoids with identity 0, so every law one discovers holds under the other — no
    /// violations. Coherence is law-agreement, NOT operator-equality (`max ≠ gcd`).
    #[test]
    fn max_and_gcd_merge_are_coherent() {
        assert!(
            coherence_violations::<MaxMerge, GcdMerge>().is_empty(),
            "max and gcd merge share the same laws — they must be coherent"
        );
    }

    /// INCOHERENT though type-compatible: `max` commutes, first-match does NOT. Both expose
    /// `merge : Key × Key -> Key` (wireable, type-checks), but they disagree about commutativity —
    /// the bug class the type system cannot see.
    #[test]
    fn max_and_first_merge_are_incoherent() {
        let v = coherence_violations::<MaxMerge, FirstMerge>();
        assert!(
            v.iter()
                .any(|s| s.contains("either order") && s.contains("max-merge")),
            "the commutativity disagreement must be reported, got: {v:?}"
        );
    }

    /// The shared identity (`empty = 0`) is real: every merge discovers `merge with empty leaves a
    /// value unchanged`. Pins `empty` against an always-undefined mutant (which would silently drop
    /// the identity law from all theories at once, leaving coherence falsely intact).
    #[test]
    fn merge_has_a_discovered_identity() {
        let laws = Engine::<MaxMerge>::new().discover().laws;
        assert!(
            laws.iter()
                .any(|l| l.prose == "Merge with empty leaves a value unchanged."),
            "the identity law must be discovered"
        );
    }

    /// The check is symmetric in what it reports: first-match also discovers laws (associativity,
    /// idempotence, identity) that DO hold for max, so those are not flagged — only the genuine
    /// disagreement is. Pins that coherence is not vacuously "everything differs".
    #[test]
    fn only_the_genuine_disagreement_is_reported() {
        let v = coherence_violations::<MaxMerge, FirstMerge>();
        // exactly the commutativity law disagrees (in one direction); everything else is shared.
        assert_eq!(v.len(), 1, "only commutativity should disagree, got: {v:?}");
    }
}
