//! zset — the Z-set carrier: rows weighted by integers, the Abelian group every license
//! reads against.
//!
//! A `ZSet` maps rows to non-zero integer weights — a multiset that can also RETRACT
//! (negative weight = deletion in flight), which is the whole reason incremental view
//! maintenance is an algebra problem: deltas need inverses. The smart-constructor
//! invariant is canonical form (no zero weights stored), so structural equality is
//! semantic equality and the group identity is literally the empty map.
//!
//! The theory below is judged EXHAUSTIVELY (the full 8³ assignment space) over a grid
//! chosen the deliberate way: the empty set, single rows at +1/−1/+3, a cancellation
//! pair (+2/−2 on one row, so sums cancel to empty), a two-row set, and a mixed-sign
//! set. Discovery freezes the full Abelian-group spec — commutativity, associativity,
//! identity, INVERSE — plus neg's own laws (involution, the additive homomorphism, the
//! fixed point at zero) with zero hand-written law tests; `spec/zset.spec` is the lock.
//!
//! v1 excludes NULL semantics entirely — rows are total opaque values. (NULLs re-enter
//! only via a world battery when a SQL frontend is attached; see the design's non-goals.)

use std::collections::BTreeMap;

/// An opaque row: the values a Z-set weighs. Total, comparable, hashable — nothing else
/// is assumed anywhere (the operators in `ops` that need more take functions over rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Row(u8);

impl Row {
    /// Mint a row from its identity. Total: every `u8` names a row.
    pub fn new(id: u8) -> Row {
        Row(id)
    }

    /// The identity — the sanctioned exit hatch.
    pub fn get(&self) -> u8 {
        self.0
    }
}

/// A Z-set: rows weighted by non-zero integers, in canonical form (no zero weights
/// stored — the invariant every constructor and operator maintains, so `==` is semantic
/// equality and `ZSet::empty()` is the group identity).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ZSet(BTreeMap<Row, i64>);

impl ZSet {
    /// The group identity: the empty Z-set.
    pub fn empty() -> ZSet {
        ZSet(BTreeMap::new())
    }

    /// Build from row/weight pairs: duplicate rows sum, zero totals are dropped — the one
    /// mint, so a non-canonical `ZSet` is unconstructible.
    pub fn of(pairs: &[(Row, i64)]) -> ZSet {
        let mut map: BTreeMap<Row, i64> = BTreeMap::new();
        for (row, w) in pairs {
            *map.entry(*row).or_insert(0) += w;
        }
        map.retain(|_, w| *w != 0);
        ZSet(map)
    }

    /// Pointwise weight sum, dropping cancellations — the group operation.
    pub fn plus(&self, other: &ZSet) -> ZSet {
        let mut out = self.0.clone();
        for (row, w) in &other.0 {
            *out.entry(*row).or_insert(0) += w;
        }
        out.retain(|_, w| *w != 0);
        ZSet(out)
    }

    /// Pointwise negation — the group inverse (a retraction of everything).
    pub fn neg(&self) -> ZSet {
        ZSet(self.0.iter().map(|(row, w)| (*row, -w)).collect())
    }

    /// A row's weight (zero when absent — absence IS weight zero).
    pub fn weight(&self, row: &Row) -> i64 {
        self.0.get(row).copied().unwrap_or(0)
    }

    /// The canonical entries, sorted by row — the observation the discovery engine
    /// fingerprints, and the sanctioned exit hatch.
    pub fn entries(&self) -> Vec<(Row, i64)> {
        self.0.iter().map(|(r, w)| (*r, *w)).collect()
    }
}

// ===== the theory: the Z-set group, judged exhaustively ====================

/// The Z-set algebra's marker — `spec/zset.spec` is its frozen law set.
pub struct ZSetAlgebra;

/// The one sort.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Zs;

fn op_zero(_: &[ZSet]) -> Option<ZSet> {
    Some(ZSet::empty())
}
fn op_plus(v: &[ZSet]) -> Option<ZSet> {
    Some(v[0].plus(&v[1]))
}
fn op_neg(v: &[ZSet]) -> Option<ZSet> {
    Some(v[0].neg())
}

/// The deliberate grid: every weight regime the group laws pivot on. `a@+2` and `a@−2`
/// are both present so `plus` actually cancels ON the grid — the inverse law is judged
/// against real annihilation, not just symbol shuffling.
fn grid() -> Vec<ZSet> {
    let a = Row::new(0);
    let b = Row::new(1);
    vec![
        ZSet::empty(),
        ZSet::of(&[(a, 1)]),
        ZSet::of(&[(a, -1)]),
        ZSet::of(&[(a, 3)]),
        ZSet::of(&[(a, 2)]),
        ZSet::of(&[(a, -2)]),
        ZSet::of(&[(a, 1), (b, 1)]),
        ZSet::of(&[(a, 1), (b, -2)]),
    ]
}

// Hand-written `Theory` impl (the `theory!` full form, expanded) for ONE deviation the
// macro cannot spell: `grid_size` covers the whole 8³ assignment space, so the Abelian
// group is judged exhaustively — the same move the engine's own maximal theories make.
impl boundary_spec::discover::engine::Theory for ZSetAlgebra {
    type Sort = Zs;
    type Value = ZSet;
    type Obs = Vec<(Row, i64)>;

    fn name() -> &'static str {
        "zset"
    }
    fn operators() -> Vec<boundary_spec::discover::engine::Operator<Self>> {
        use boundary_spec::discover::engine::{Fixity, Operator};
        vec![
            Operator {
                name: "zero",
                symbol: "zero",
                fixity: Fixity::Nullary,
                inputs: vec![],
                output: Zs,
                eval: op_zero,
            },
            Operator {
                name: "plus",
                symbol: "plus",
                fixity: Fixity::Infix,
                inputs: vec![Zs, Zs],
                output: Zs,
                eval: op_plus,
            },
            Operator {
                name: "neg",
                symbol: "neg",
                fixity: Fixity::Prefix,
                inputs: vec![Zs],
                output: Zs,
                eval: op_neg,
            },
        ]
    }
    fn inhabitants(_: Zs) -> Vec<ZSet> {
        grid()
    }
    fn sort_of(_: &ZSet) -> Zs {
        Zs
    }
    fn observe(v: &ZSet) -> Vec<(Row, i64)> {
        v.entries()
    }
    fn sort_vars(_: Zs) -> &'static [&'static str] {
        &["x", "y", "z"]
    }
    fn grid_size() -> usize {
        512 // = 8³, the whole space: the group laws are judged exhaustively.
    }
}

// The declared algebra — the top-down half, verified against discovery by the distance
// gate in `tests/freeze_gate.rs`. Declaring is free; it makes the distance measurable.
impl boundary_spec::discover::expect::Expected for ZSetAlgebra {
    fn expectations() -> Vec<boundary_spec::discover::expect::Expectation> {
        use boundary_spec::discover::expect::Expectation;
        vec![
            Expectation::of("commutative", vec!["plus"]),
            Expectation::of("associative", vec!["plus"]),
            Expectation::of("identity", vec!["plus", "zero"]),
            Expectation::of("inverse", vec!["plus", "neg", "zero"]),
            Expectation::of("involution", vec!["neg"]),
            Expectation::of("homomorphism", vec!["neg", "plus", "plus"]),
            Expectation::of("fixed_point", vec!["neg", "zero"]),
        ]
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The canonical-form invariant, from every door: the mint sums duplicates and drops
    /// zero totals; `plus` drops cancellations; `neg` maps weights pointwise. Oracle-free
    /// where possible (absence IS weight zero), pinned where the invariant itself is the
    /// content.
    #[test]
    fn canonical_form_has_no_zero_weights() {
        let a = Row::new(0);
        let b = Row::new(1);
        // the mint: duplicates sum, zeros drop.
        let s = ZSet::of(&[(a, 2), (a, -2), (b, 5), (b, 0)]);
        assert_eq!(s.entries(), vec![(b, 5)]);
        assert_eq!(s.weight(&a), 0, "a cancelled away — absence is weight zero");
        // plus: a cancellation pair sums to the identity, structurally.
        let up = ZSet::of(&[(a, 2)]);
        let down = ZSet::of(&[(a, -2)]);
        assert_eq!(up.plus(&down), ZSet::empty());
        // neg: pointwise, and the exit hatch reads it back.
        assert_eq!(
            ZSet::of(&[(a, 3), (b, -1)]).neg().entries(),
            vec![(a, -3), (b, 1)]
        );
    }

    /// The exit hatches read back what was minted — two distinct points per accessor, so
    /// no constant can satisfy the pin.
    #[test]
    fn the_exit_hatches_read_back_what_was_minted() {
        assert_eq!(Row::new(7).get(), 7);
        assert_eq!(Row::new(3).get(), 3);
        let a = Row::new(0);
        assert_eq!(ZSet::of(&[(a, 4)]).weight(&a), 4);
        assert_eq!(ZSet::of(&[(a, -4)]).weight(&a), -4);
    }
}
