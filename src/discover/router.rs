//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! A tiny HTTP-style router, as a `Theory` — to show the engine discovers a NON-arithmetic, and
//! crucially NON-COMMUTATIVE, algebra by running the operators.
//!
//! A router is modelled by its observable behaviour: a routing TABLE mapping each path to the
//! handler it resolves to (`None` = unrouted). `or` is first-match union (`a` wins where both
//! match), `empty` routes nothing. Two routers are equal iff they route every path the same way —
//! OBSERVATIONAL equality, exactly what the engine groups by, so routers need no structural `Eq`
//! of their own. The discovered algebra is a monoid: identity (`empty`), associativity, and
//! idempotence — but NOT commutativity, which the engine correctly refuses to report because
//! overlapping routers route differently in each order.

/// The number of distinct paths the router is observed over.
const PATHS: usize = 4;

/// A routing table: the handler each path resolves to (`None` = unrouted). This is the router value
/// object — defined by what it routes, not how it was built.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Routes([Option<u8>; PATHS]);

/// The router theory.
pub struct Router;

/// One sort: routers.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Router,
}

fn empty(_: &[Routes]) -> Option<Routes> {
    Some(Routes([None; PATHS]))
}

/// First-match union: where both route a path, the LEFT router wins (so `or` is not commutative).
fn or(vs: &[Routes]) -> Option<Routes> {
    let (a, b) = (&vs[0].0, &vs[1].0);
    let mut out = [None; PATHS];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = a[i].or(b[i]);
    }
    Some(Routes(out))
}

/// A spread that includes OVERLAPPING routers (several route path 0), so the grid can tell
/// `a or b` from `b or a` — and the engine correctly omits commutativity.
fn routers() -> Vec<Routes> {
    vec![
        Routes([None, None, None, None]),
        Routes([Some(1), None, None, None]),
        Routes([None, Some(2), None, None]),
        Routes([Some(3), Some(4), None, None]),
        Routes([Some(5), None, Some(6), None]),
        Routes([None, None, None, Some(7)]),
    ]
}

// The whole `Theory` impl is generated from this block — the operator functions (`empty`, `or`) and
// the value object (`Routes`) above are the only authored content.
crate::theory! {
    Router : "router", Value = Routes, Obs = Vec<Option<u8>>, Sort = Sort,
    sort_of = |_: &Routes| Sort::Router,
    observe = |v: &Routes| v.0.to_vec(),
    vars { Sort::Router => &["a", "b", "c"], }
    inhabit { Sort::Router => routers(), }
    ops {
        Nullary "Empty" "empty" () -> Sort::Router = empty;
        Infix   "Or"    "or"    (Sort::Router, Sort::Router) -> Sort::Router = or;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::{Engine, Theory};

    /// The engine discovers the router's monoid by running it: identity, associativity, idempotence —
    /// and crucially NOT commutativity (overlapping routers route differently in each order, so the
    /// behavioural signatures differ and the engine refuses to report it). A mutation to `or` or
    /// `empty` that broke a law would change this set; `check` (fed these as frozen laws) catches it.
    #[test]
    fn the_router_monoid_is_discovered() {
        assert_eq!(Router::name(), "router");
        let e = Engine::<Router>::new();
        let d = e.discover();
        let got: Vec<(String, String)> = d
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "With Or, the grouping of three values doesn't matter.",
                "((a or b) or c) = (a or (b or c))",
            ),
            (
                "Or of a value with itself gives that value.",
                "(a or a) = a",
            ),
            (
                "Or with empty leaves a value unchanged.",
                "(empty or a) = a",
            ),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, q)| (p.to_string(), q.to_string()))
            .collect();
        assert_eq!(got, expected, "the discovered router algebra changed");
        // the whole point: commutativity is NOT discovered (it does not hold).
        assert!(
            !d.laws.iter().any(|l| l.prose.contains("either order")),
            "router `or` is not commutative — it must not be reported as such"
        );
        assert_eq!(d.consequences, 14);
        assert!(d.uncovered_ops.is_empty());
        // the laws hold on replay, and `check` would reject a false commutativity claim.
        assert_eq!(e.check(&d.laws), Ok(()));
    }
}
