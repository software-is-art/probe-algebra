//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! The TTL store's algebra, as a `Theory` — the first STATEFUL domain handed to the
//! discovery engine, to test whether "the spec falls out of running the operators"
//! survives contact with state and decay.
//!
//! Two sorts: `Store` (the state itself) and `Duration` (spans of logical ticks). The
//! operators are the boundary's own — `empty`, the right-biased `merge`, and `tick`
//! ROUTED THROUGH THE REAL `Tick` EDGE (via `Store::tick`), so the discovered laws
//! genuinely probe the eager sweep in `internal`, not a parallel model. The observation
//! is `Store::view` — the LIVE entries at the store's own clock, sorted, with
//! clock-relative remaining life — so expiry is visible to the engine and consequences
//! like "merging with a later store silently expires entries" are within its sight.
//!
//! `Put` is deliberately NOT an operator here: it is 4-ary (`Store × Key × Val × Ttl`),
//! and the engine's universal shapes name laws over unary and binary operators only, so
//! a `put` in the signature would be honest dead weight in `uncovered_ops`. It is
//! CURRIED OUT of the theory and certified at the edge layer instead (`probes`), where
//! its residual round-trip and put-then-read coherence do the work. The store
//! inhabitants are still BUILT with `put`, so its effects are inside the grid.
//!
//! A limit this domain FOUND, and what it forced: the classic monoid shapes certify
//! that `merge` is a monoid but cannot name the DIRECTION of its bias — a
//! first-write-wins union satisfies the very same laws, and a planted left-biased
//! mutant survived the then-golden spec. That finding became a new universal shape:
//! the engine now tries the regular-band "sandwich" laws on every non-commutative
//! binary, and the discovered spec STATES the bias — "the later operand wins where
//! the two disagree" — so the first-write-wins mutant is killed by the spec itself.
//! The dedicated two-route probe in `probes` (`merge_is_right_biased_at_the_later_clock`)
//! stays as defense in depth: it pins the winning VALUES and the clock max, which is
//! finer than the band law alone.

use super::store::{Key, Snapshot, Store, Ttl, Val};

/// A value in the theory: a store or a duration.
#[derive(Clone)]
pub enum KvVal {
    Store(Store),
    Dur(Ttl),
}

/// The TTL-store theory.
pub struct TtlStore;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Store,
    Duration,
}

fn store(v: &KvVal) -> Store {
    match v {
        KvVal::Store(s) => s.clone(),
        KvVal::Dur(_) => Store::new(), // unreachable: the sorts keep arguments apart.
    }
}
fn dur(v: &KvVal) -> Ttl {
    match v {
        KvVal::Dur(d) => *d,
        KvVal::Store(_) => Ttl::zero(), // unreachable, as above.
    }
}

fn empty(_: &[KvVal]) -> Option<KvVal> {
    Some(KvVal::Store(Store::new()))
}
fn merge(v: &[KvVal]) -> Option<KvVal> {
    Some(KvVal::Store(store(&v[0]).merge(&store(&v[1]))))
}
fn tick(v: &[KvVal]) -> Option<KvVal> {
    Some(KvVal::Store(store(&v[0]).tick(dur(&v[1]))))
}
fn zero(_: &[KvVal]) -> Option<KvVal> {
    Some(KvVal::Dur(Ttl::zero()))
}
fn plus(v: &[KvVal]) -> Option<KvVal> {
    Some(KvVal::Dur(dur(&v[0]).plus(dur(&v[1]))))
}

fn key(s: &str) -> Key {
    Key::new(s).expect("a valid key")
}
fn val(n: i64) -> Val {
    Val::new(n).expect("a valid value")
}
fn ttl(n: i64) -> Ttl {
    Ttl::new(n).expect("a valid duration")
}

/// A spread of stores that includes CONFLICTS (same key, different value and TTL) and
/// CLOCK SKEW (a store that has already ticked), so the grid can refute the false laws:
/// conflicting stores make `merge` visibly non-commutative, and the skewed store makes
/// merged-in entries expire. Built through the boundary (`put`/`tick` route through the
/// real edges), never by hand-assembling state.
fn stores() -> Vec<KvVal> {
    let e = Store::new();
    vec![
        e.clone(),
        e.put(key("a"), val(1), ttl(2)),
        e.put(key("a"), val(2), ttl(5)), // conflicts with the previous on "a"
        e.put(key("b"), val(3), ttl(3)),
        e.put(key("a"), val(4), ttl(4))
            .put(key("b"), val(5), ttl(1)),
        e.put(key("a"), val(6), ttl(1)).tick(ttl(2)), // clock 2: skew, entry swept
    ]
    .into_iter()
    .map(KvVal::Store)
    .collect()
}
fn durs() -> Vec<KvVal> {
    [0i64, 1, 2, 5]
        .into_iter()
        .map(|n| KvVal::Dur(ttl(n)))
        .collect()
}

// The whole multi-sorted `Theory` impl is generated from this block — only the value
// object (`KvVal`) and the operator functions above are authored.
crate::theory! {
    TtlStore : "ttl store", Value = KvVal, Obs = (u8, Snapshot, Ttl), Sort = Sort,
    sort_of = |v: &KvVal| match v {
        KvVal::Store(_) => Sort::Store,
        KvVal::Dur(_) => Sort::Duration,
    },
    observe = |v: &KvVal| match v {
        KvVal::Store(s) => (0u8, s.view(), Ttl::zero()),
        KvVal::Dur(d) => (1u8, Snapshot::empty(), *d),
    },
    vars {
        Sort::Store => &["s", "t", "u"],
        Sort::Duration => &["p", "q", "r"],
    }
    inhabit {
        Sort::Store => stores(),
        Sort::Duration => durs(),
    }
    ops {
        Nullary "Empty" "empty" () -> Sort::Store = empty;
        Infix   "Merge" "<+"    (Sort::Store, Sort::Store) -> Sort::Store = merge;
        Prefix  "Tick"  "tick"  (Sort::Store, Sort::Duration) -> Sort::Store = tick;
        Nullary "Zero"  "zero"  () -> Sort::Duration = zero;
        Infix   "Plus"  "+"     (Sort::Duration, Sort::Duration) -> Sort::Duration = plus;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::{Engine, Theory};

    /// The engine discovers, on STATE, exactly this spec — and each law certifies a fact
    /// about state that a pure domain cannot express:
    ///
    ///   - merge associativity + idempotence + the `empty` identity: accumulated state
    ///     composes as a monoid, and replaying a store onto itself changes nothing (so
    ///     "apply the same batch twice" is safe);
    ///   - NOT commutativity — the engine correctly refuses it, because last-write-wins
    ///     means merge ORDER is semantics (the conflicting inhabitants make the orders
    ///     observably different) — and in its place the BIAS law: the sandwich shape
    ///     names WHICH side wins ("the later operand wins where the two disagree"), so
    ///     a first-write-wins mutant now breaks the discovered spec itself;
    ///   - `tick` with `zero` as identity and repeated `tick` combining by `Plus`: time
    ///     is a MONOID ACTION of durations on stores — the same action template the date
    ///     calculus exercises, now moving real state (each `tick` runs the eager sweep,
    ///     so these two laws also pin sweep-then-sweep against sweep-once);
    ///   - the `Plus`/`zero` duration monoid the action is keyed on.
    ///
    /// Every operator participates in a law (put was curried out precisely so coverage
    /// stays total — see the module header).
    #[test]
    fn the_ttl_store_algebra_is_discovered() {
        assert_eq!(TtlStore::name(), "ttl store");
        let e = Engine::<TtlStore>::new();
        let d = e.discover();
        let got: Vec<(String, String)> = d
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "With Merge, the grouping of three values doesn't matter.",
                "((s <+ t) <+ u) = (s <+ (t <+ u))",
            ),
            (
                "Merge of a value with itself gives that value.",
                "(s <+ s) = s",
            ),
            (
                "With Merge, the later operand wins where the two disagree — re-applying \
                 an earlier one cannot overwrite it.",
                "((s <+ t) <+ s) = (t <+ s)",
            ),
            (
                "Merge with empty leaves a value unchanged.",
                "(empty <+ s) = s",
            ),
            (
                "Tick with zero leaves a value unchanged.",
                "tick(s, zero) = s",
            ),
            (
                "Repeated Tick combines its parameters with Plus.",
                "tick(tick(s, p), q) = tick(s, (p + q))",
            ),
            (
                "Tick applications commute — the parameter order doesn't matter.",
                "tick(tick(s, p), q) = tick(tick(s, q), p)",
            ),
            (
                "Tick leaves empty fixed — no parameter moves it.",
                "tick(empty, p) = empty",
            ),
            (
                "Plus gives the same result in either order.",
                "(p + q) = (q + p)",
            ),
            (
                "With Plus, the grouping of three values doesn't matter.",
                "((p + q) + r) = (p + (q + r))",
            ),
            ("Plus with zero leaves a value unchanged.", "(zero + p) = p"),
            // the WITNESS law — the inequation that closes the trivial-action survivor:
            // a clock that never advances now contradicts the spec instead of
            // satisfying every action equation vacuously.
            (
                "Tick actually acts — some parameter moves some value.",
                "tick(s, p) ≠ s",
            ),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, q)| (p.to_string(), q.to_string()))
            .collect();
        assert_eq!(got, expected, "the discovered ttl-store algebra changed");
        assert_eq!(d.consequences, 147, "consequence count changed");
        // put was curried out of the signature, so coverage is total by construction.
        assert!(
            d.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            d.uncovered_ops
        );
        assert_eq!(e.check(&d.laws), Ok(()));
    }
}
