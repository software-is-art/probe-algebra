//! ops — the lifted relational operators, one theory per operator, classified by DISCOVERY.
//!
//! Each operator is a pure function over [`ZSet`]s, declared alongside the group operators
//! (`zero`/`plus`/`neg`) so the license-bearing laws can fire: an operator is LINEAR when
//! its spec contains the additive homomorphism (`{op} turns plus into plus.`) and the zero
//! fixed point; BILINEAR when additive in each argument slot. Nothing here declares a
//! classification — `spec/licenses.spec` is derived by reading the frozen specs
//! (see [`crate::license`]).
//!
//! The inventory includes deliberate negatives: `distinct` and `min` are the canonical
//! non-incremental operators — `distinct` because re-weighting to 1 forgets how many
//! supports a row has (additivity dies exactly there), `min` because a retraction can
//! delete the current minimum and no delta short of recomputation says what the next one
//! is. Their specs LACK the license laws; the registry maps them to the generic fallback.
//!
//! Every theory is judged exhaustively (`grid_size` = the whole assignment space) over the
//! same deliberate grid as the carrier.

use crate::zset::{Row, ZSet, Zs};

// ===== the lifted operators, as pure functions =============================

/// `filter`: keep the even rows, weights untouched — the easy linear yes.
pub fn filter_even(x: &ZSet) -> ZSet {
    ZSet::of(
        &x.entries()
            .into_iter()
            .filter(|(r, _)| r.get() % 2 == 0)
            .collect::<Vec<_>>(),
    )
}

/// `map` (a projection): relabel every row to `id / 2`, so distinct rows MERGE and their
/// weights add — the subtle linear yes worth freezing (a map that dropped or clamped
/// colliding weights would lose the homomorphism law).
pub fn project_halved(x: &ZSet) -> ZSet {
    ZSet::of(
        &x.entries()
            .into_iter()
            .map(|(r, w)| (Row::new(r.get() / 2), w))
            .collect::<Vec<_>>(),
    )
}

/// `sum`: the canonical incremental aggregate — the weighted total of row values
/// (value of row `r` is `r + 1`, so every row counts), carried as a weight on one
/// output row. A linear functional of the weights, so deltas of the input ARE deltas
/// of the aggregate.
pub fn total(x: &ZSet) -> ZSet {
    let t: i64 = x
        .entries()
        .into_iter()
        .map(|(r, w)| w * (i64::from(r.get()) + 1))
        .sum();
    ZSet::of(&[(Row::new(0), t)])
}

/// `join` (an intersection join on the shared row space): pointwise weight PRODUCT —
/// the three-term delta rule's client. Bilinear: products distribute over sums in each
/// slot, and this join is commutative, so one distributivity law plus commutativity
/// licenses both slots.
pub fn join(a: &ZSet, b: &ZSet) -> ZSet {
    ZSet::of(
        &a.entries()
            .into_iter()
            .map(|(r, w)| (r, w * b.weight(&r)))
            .collect::<Vec<_>>(),
    )
}

/// `distinct`: the rows with positive weight, re-weighted to 1 — the canonical
/// NON-linear operator. Additivity dies exactly at the re-weighting: two +1 supports
/// sum to +2 upstream but `distinct` forgets the multiplicity, so
/// `distinct(x + y) ≠ distinct(x) + distinct(y)` already at `x = y = {a:+1}`.
pub fn distinct(x: &ZSet) -> ZSet {
    ZSet::of(
        &x.entries()
            .into_iter()
            .filter(|(_, w)| *w > 0)
            .map(|(r, _)| (r, 1))
            .collect::<Vec<_>>(),
    )
}

/// `min`: the least positively-supported row, at weight 1 (empty when nothing is
/// positive) — non-linear for the deeper reason: a RETRACTION can delete the current
/// minimum, and no delta short of recomputation says what the next minimum is. The
/// spec's silence on every additivity law is the honest rendering of that fact.
pub fn least(x: &ZSet) -> ZSet {
    x.entries()
        .into_iter()
        .filter(|(_, w)| *w > 0)
        .map(|(r, _)| r)
        .min()
        .map(|r| ZSet::of(&[(r, 1)]))
        .unwrap_or_else(ZSet::empty)
}

// ===== the theories: group + one lifted operator, judged exhaustively ======

fn op_zero(_: &[ZSet]) -> Option<ZSet> {
    Some(ZSet::empty())
}
fn op_plus(v: &[ZSet]) -> Option<ZSet> {
    Some(v[0].plus(&v[1]))
}
fn op_neg(v: &[ZSet]) -> Option<ZSet> {
    Some(v[0].neg())
}

/// One lifted-operator theory: the Z-set group plus the operator under classification,
/// over the carrier's deliberate grid, judged exhaustively. A local macro because the
/// six theories differ ONLY in marker, name, and the lifted operator's stanza — and
/// `grid_size` (the exhaustiveness deviation) is not spellable in `theory!`.
macro_rules! lifted_theory {
    ($thy:ident, $name:literal, $( $fix:ident $opname:literal ($($insort:expr),*) = $eval:expr );+ $(;)?) => {
        pub struct $thy;

        impl boundary_spec::discover::engine::Theory for $thy {
            type Sort = Zs;
            type Value = ZSet;
            type Obs = Vec<(Row, i64)>;

            fn name() -> &'static str {
                $name
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
                    $( Operator {
                        name: $opname,
                        symbol: $opname,
                        fixity: Fixity::$fix,
                        inputs: vec![$($insort),*],
                        output: Zs,
                        eval: $eval,
                    } ),+
                ]
            }
            fn inhabitants(_: Zs) -> Vec<ZSet> {
                crate::zset::grid()
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
                512 // = 8³, the whole space: the license laws are judged exhaustively.
            }
        }
    };
}

fn op_filter(v: &[ZSet]) -> Option<ZSet> {
    Some(filter_even(&v[0]))
}
fn op_map(v: &[ZSet]) -> Option<ZSet> {
    Some(project_halved(&v[0]))
}
fn op_sum(v: &[ZSet]) -> Option<ZSet> {
    Some(total(&v[0]))
}
fn op_join(v: &[ZSet]) -> Option<ZSet> {
    Some(join(&v[0], &v[1]))
}
fn op_distinct(v: &[ZSet]) -> Option<ZSet> {
    Some(distinct(&v[0]))
}
fn op_min(v: &[ZSet]) -> Option<ZSet> {
    Some(least(&v[0]))
}

lifted_theory!(FilterOp, "filter", Prefix "filter"(Zs) = op_filter);
lifted_theory!(MapOp, "map", Prefix "map"(Zs) = op_map);
lifted_theory!(SumOp, "sum", Prefix "sum"(Zs) = op_sum);
lifted_theory!(JoinOp, "join", Infix "join"(Zs, Zs) = op_join);
lifted_theory!(DistinctOp, "distinct", Prefix "distinct"(Zs) = op_distinct);
lifted_theory!(MinOp, "min", Prefix "min"(Zs) = op_min);

#[cfg(test)]
mod probes {
    use super::*;

    fn z(pairs: &[(u8, i64)]) -> ZSet {
        ZSet::of(
            &pairs
                .iter()
                .map(|(r, w)| (Row::new(*r), *w))
                .collect::<Vec<_>>(),
        )
    }

    /// The two deliberate negatives REFUSE additivity on a named instance each — the
    /// exact break the specs' silence renders: `distinct` forgets multiplicity,
    /// `min` cannot survive a retraction of the current minimum.
    #[test]
    fn the_negatives_break_additivity_where_the_docs_say() {
        // distinct: two +1 supports of one row.
        let one = z(&[(0, 1)]);
        assert_ne!(
            distinct(&one.plus(&one)),
            distinct(&one).plus(&distinct(&one)),
            "distinct must not be additive"
        );
        // min: retracting the current minimum uncovers the next — no delta says which.
        let both = z(&[(0, 1), (1, 1)]);
        let retract_min = z(&[(0, -1)]);
        assert_eq!(least(&both.plus(&retract_min)), z(&[(1, 1)]));
        assert_ne!(
            least(&both.plus(&retract_min)),
            least(&both).plus(&least(&retract_min)),
            "min must not be additive under retraction"
        );
    }

    /// The lifted operators compute what their docs claim, two points each — the
    /// per-operator floor under the law-level classification.
    #[test]
    fn the_lifted_operators_read_back_their_own_stories() {
        let x = z(&[(0, 2), (1, 3)]);
        assert_eq!(filter_even(&x), z(&[(0, 2)]));
        assert_eq!(filter_even(&z(&[(1, 5)])), ZSet::empty());
        // map merges rows 0 and 1 into row 0 — the weights ADD.
        assert_eq!(project_halved(&x), z(&[(0, 5)]));
        assert_eq!(project_halved(&z(&[(0, 1), (1, -1)])), ZSet::empty());
        // sum: 2·(0+1) + 3·(1+1) = 8, as a weight.
        assert_eq!(total(&x), z(&[(0, 8)]));
        assert_eq!(total(&z(&[(0, 1), (1, -1)])), z(&[(0, -1)]));
        // join: pointwise products.
        assert_eq!(join(&x, &z(&[(0, 5), (1, -1)])), z(&[(0, 10), (1, -3)]));
        assert_eq!(join(&x, &ZSet::empty()), ZSet::empty());
        // distinct: positive support at weight 1.
        assert_eq!(distinct(&z(&[(0, 3), (1, -2)])), z(&[(0, 1)]));
        // min: least positive row.
        assert_eq!(least(&z(&[(0, -1), (1, 2)])), z(&[(1, 1)]));
        assert_eq!(least(&ZSet::empty()), ZSet::empty());
    }
}
