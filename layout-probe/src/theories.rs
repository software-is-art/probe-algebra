//! theories — the two engines as theories over ONE operator vocabulary, differing only
//! in the observation (which engine renders the diagram) and the `render` operator's
//! body. The load-bearing choice is `Obs(Diagram) = the engine's geometry`: with that
//! observation, "declaration reorder must not move the layout" is literally the `inert`
//! catalog shape, and "rename-then-render = render-then-relabel" is the `equivariant
//! map` square — no bespoke checks, just discovery.
//!
//! Both theories DECLARE the same three laws (`expects`-equivalent, via `Expected`):
//! reorder inert, theme inert, render equivariant. Neither engine earns all three —
//! stable loses equivariance (the name is load-bearing for position, so renames move
//! nodes), eager loses reorder-inertness (the declaration is load-bearing, so reorders
//! move nodes) — and the two distance reports, pinned in the tests, are the ENGINE
//! SCORECARD the scoping promised: a real tradeoff named by a missing law, not a bug
//! in either engine.

use boundary_spec::discover::engine::{Fixity, Operator, Theory};
use boundary_spec::discover::expect::{Expectation, Expected};

use crate::diagram::{Diagram, Swap};
use crate::layout::{layout, Geometry, Policy};

/// The three sorts: diagram sources, rendered geometries, rename parameters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LSort {
    Dg,
    Ge,
    Rn,
}

/// The value sum over the three sorts.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LVal {
    D(Diagram),
    G(Geometry),
    R(Swap),
}

/// The observation sum: a DIAGRAM is observed by its rendered geometry (the theory's
/// engine decides which), a geometry by its placements, a swap by its display.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LObs {
    Placed(Vec<(String, (i64, i64))>),
    Swap(String),
}

fn as_d(v: &LVal) -> &Diagram {
    match v {
        LVal::D(d) => d,
        _ => unreachable!("sort-checked by the engine"),
    }
}
fn as_g(v: &LVal) -> &Geometry {
    match v {
        LVal::G(g) => g,
        _ => unreachable!("sort-checked by the engine"),
    }
}
fn as_r(v: &LVal) -> &Swap {
    match v {
        LVal::R(r) => r,
        _ => unreachable!("sort-checked by the engine"),
    }
}

fn op_reorder(v: &[LVal]) -> Option<LVal> {
    Some(LVal::D(as_d(&v[0]).reorder()))
}
fn op_theme(v: &[LVal]) -> Option<LVal> {
    Some(LVal::D(as_d(&v[0]).theme()))
}
fn op_rename(v: &[LVal]) -> Option<LVal> {
    Some(LVal::D(as_d(&v[0]).rename(as_r(&v[1]))))
}
fn op_relabel(v: &[LVal]) -> Option<LVal> {
    Some(LVal::G(as_g(&v[0]).relabel(as_r(&v[1]))))
}
fn op_render_stable(v: &[LVal]) -> Option<LVal> {
    Some(LVal::G(layout(as_d(&v[0]), Policy::Stable)))
}
fn op_render_eager(v: &[LVal]) -> Option<LVal> {
    Some(LVal::G(layout(as_d(&v[0]), Policy::Eager)))
}

/// The DELIBERATE source grid — every regime the laws pivot on: the empty diagram, a
/// single node (renames must be visible), a chain declared OUT of name order and a
/// bare pair declared out of name order (the fixtures where the two policies actually
/// disagree), the diamond (a rank with two members — within-rank order is real), and
/// a two-component diagram (disconnected ranks).
pub fn source_grid() -> Vec<Diagram> {
    vec![
        Diagram::of(&[], &[]),
        Diagram::of(&["a"], &[]),
        Diagram::of(&["b", "a"], &[]),
        Diagram::of(&["c", "a", "b"], &[("a", "b"), ("b", "c")]),
        Diagram::of(
            &["a", "b", "c", "d"],
            &[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")],
        ),
        Diagram::of(&["b", "a", "c"], &[("b", "c")]),
    ]
}

/// The rename parameters — swaps over names the grid actually uses, so every rename's
/// image stays on the grid.
pub fn swap_grid() -> Vec<Swap> {
    vec![Swap::of("a", "b"), Swap::of("b", "c")]
}

macro_rules! layout_theory {
    ($thy:ident, $name:literal, $policy:expr, $render:ident) => {
        /// One engine as a theory — see the module doc; the twin differs only here.
        pub struct $thy;

        impl Theory for $thy {
            type Sort = LSort;
            type Value = LVal;
            type Obs = LObs;

            fn name() -> &'static str {
                $name
            }
            fn operators() -> Vec<Operator<Self>> {
                use LSort::{Dg, Ge, Rn};
                vec![
                    Operator {
                        name: "render",
                        symbol: "render",
                        fixity: Fixity::Prefix,
                        inputs: vec![Dg],
                        output: Ge,
                        eval: $render,
                    },
                    Operator {
                        name: "reorder",
                        symbol: "reorder",
                        fixity: Fixity::Prefix,
                        inputs: vec![Dg],
                        output: Dg,
                        eval: op_reorder,
                    },
                    Operator {
                        name: "theme",
                        symbol: "theme",
                        fixity: Fixity::Prefix,
                        inputs: vec![Dg],
                        output: Dg,
                        eval: op_theme,
                    },
                    Operator {
                        name: "rename",
                        symbol: "rename",
                        fixity: Fixity::Prefix,
                        inputs: vec![Dg, Rn],
                        output: Dg,
                        eval: op_rename,
                    },
                    Operator {
                        name: "relabel",
                        symbol: "relabel",
                        fixity: Fixity::Prefix,
                        inputs: vec![Ge, Rn],
                        output: Ge,
                        eval: op_relabel,
                    },
                ]
            }
            fn inhabitants(sort: LSort) -> Vec<LVal> {
                match sort {
                    LSort::Dg => source_grid().into_iter().map(LVal::D).collect(),
                    LSort::Ge => source_grid()
                        .iter()
                        .map(|d| LVal::G(layout(d, $policy)))
                        .collect(),
                    LSort::Rn => swap_grid().into_iter().map(LVal::R).collect(),
                }
            }
            fn sort_of(v: &LVal) -> LSort {
                match v {
                    LVal::D(_) => LSort::Dg,
                    LVal::G(_) => LSort::Ge,
                    LVal::R(_) => LSort::Rn,
                }
            }
            fn observe(v: &LVal) -> LObs {
                match v {
                    // THE choice: a source is observed by what the engine makes of it.
                    LVal::D(d) => LObs::Placed(layout(d, $policy).placements().to_vec()),
                    LVal::G(g) => LObs::Placed(g.placements().to_vec()),
                    LVal::R(s) => LObs::Swap(s.render()),
                }
            }
            fn sort_vars(sort: LSort) -> &'static [&'static str] {
                match sort {
                    LSort::Dg => &["x", "y", "z"],
                    LSort::Ge => &["g", "h", "i"],
                    LSort::Rn => &["p", "q"],
                }
            }
            fn grid_size() -> usize {
                512 // covers every assignment space the laws use, exhaustively.
            }
        }

        // The declared laws — IDENTICAL for both engines, so the two distance reports
        // are directly comparable: that comparability IS the scorecard.
        impl Expected for $thy {
            fn expectations() -> Vec<Expectation> {
                vec![
                    Expectation::of("inert", vec!["reorder"]),
                    Expectation::of("inert", vec!["theme"]),
                    Expectation::of("equivariant", vec!["render", "rename", "relabel"]),
                ]
            }
        }
    };
}

layout_theory!(
    StableLayout,
    "stable layout",
    Policy::Stable,
    op_render_stable
);
layout_theory!(EagerLayout, "eager layout", Policy::Eager, op_render_eager);

#[cfg(test)]
mod probes {
    use super::*;
    use boundary_spec::discover::expect::Distance;

    /// THE ENGINE SCORECARD, pinned lib-side (the member's mutation sweep judges lib
    /// mutants against lib tests): both engines declare the SAME three laws, neither
    /// earns all three, and each is red on exactly its own axis — name-ordered layout
    /// loses rename-equivariance, declaration-ordered layout loses reorder-inertness.
    /// A real tradeoff named by a missing law; the surprises are ratified in the specs.
    #[test]
    fn the_scorecard_names_the_tradeoff_no_engine_escapes() {
        assert_eq!(
            Distance::of::<StableLayout>().render(),
            "stable layout: 2 of 3 declared laws hold; MISSING: \
             equivariant(render, rename, relabel); SURPRISES (discovered, never \
             declared — ratify or refute): involution(reorder), projection(reorder), \
             round_trip(reorder, theme), equivariant(reorder, rename), \
             involution(theme), projection(theme), round_trip(theme, reorder), \
             equivariant(theme, rename), nontrivial(rename), nontrivial(relabel)"
        );
        assert_eq!(
            Distance::of::<EagerLayout>().render(),
            "eager layout: 2 of 3 declared laws hold; MISSING: inert(reorder); \
             SURPRISES (discovered, never declared — ratify or refute): \
             equivariant(reorder, rename), involution(theme), projection(theme), \
             equivariant(theme, rename), nontrivial(rename), nontrivial(relabel)"
        );
    }
}
