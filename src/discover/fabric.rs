//!
//! fabric — the SYNTHETIC WORLD: infrastructure behaviour as a discovered algebra
//! (step one of the behaviour-as-code candidate; see `docs/roadmap.md`).
//!
//! The world is a VALUE. A `Fabric` is an overlay over a small node universe: a set of
//! granted routes and a set of standing denies (tombstones — a deny is policy, recorded
//! whether or not the route exists). There is no ambient cloud, exactly as the TTL store
//! has no ambient clock: every operator is pure, every grid is enumerable in-process, so
//! the existing engine discovers what infrastructure behaviour actually obeys — no new
//! machinery, no probes, no mutation of anything real. The interior is synthetic ON
//! PURPOSE; swapping it for read-only world probes later moves no boundary and restates
//! no law (the tier-2 rule), and sim-vs-real becomes a transport seam.
//!
//! Semantics, in three decisions:
//!   * **deny kills LINKS, not paths** — `reach` closes over the effective links
//!     (grants minus denies), so denying a direct route does not sever a multi-hop
//!     detour, exactly like a real network.
//!   * **denies are standing policy** — `join` unions both sides, so a deny survives
//!     every merge (deny-wins by construction, not by precedence rules).
//!   * **`within` is behavioural containment** — `within(f, g)` asks whether every
//!     connection `f` delivers is delivered by `g`: the overlay's promise, compared by
//!     what it DOES.
//!
//! What discovery says — and, more importantly, REFUSES to say (the refusal is the
//! security semantics made visible; the module test pins its absence):
//!   * `join` is the expected semilattice (commutative, associative, idempotent,
//!     identity at `mesh`) and `grant`/`revoke` are well-behaved actions — repeated
//!     application settles, order never matters, acting on a merge is merging the
//!     actions. Their DIRECTIONS are laws too: grant only grows delivery, revoke only
//!     shrinks it (the ordered-action stanzas — see below).
//!   * `reach` is a PROJECTION (closing twice is closing once) that leaves `mesh`
//!     fixed, and is monotone under `within` (trivially: `within` already compares
//!     deliveries, and `reach` cannot change what a fabric delivers).
//!   * **`reach` is NOT monotone in the `join`-order**: merging fabrics can REDUCE what
//!     you can reach, because a merge carries the other side's denies. Anyone whose
//!     mental model is "adding infrastructure only adds connectivity" is wrong in
//!     exactly the way this refusal states.
//!   * And one prediction the engine CORRECTED, kept here because being corrected is
//!     the method: the author expected subadditivity of `reach` over `join` to be
//!     refused too (two half-chains BRIDGE — a merge delivers connections neither side
//!     had, the same fact the placer alarms on). Discovery reports the law HOLDS,
//!     because `within` is behavioural: it compares closed delivery sets, so the
//!     merged parts' deliveries close over the bridge as well. The structural
//!     superadditivity of bridging is real but not sayable by a behavioural order —
//!     the observation quotients it away.
//!
//! This domain's first mutation sweep also GREW the law language. `grant` confused with
//! `revoke` survived in both directions — idempotent, commuting, equivariant, nontrivial
//! described both equally, so the equational vocabulary could not say which way an
//! action moves a value. The `action inflation` / `action deflation` stanzas were added
//! to the catalog in response (the witness-shapes precedent: extend the vocabulary, not
//! the battery), discovery found the two direction laws itself on the next run, and
//! both survivors died.
//!
//! One shape the catalog still cannot say: deny-wins as a single equation
//! (`revoke(grant(f, r), r) = revoke(f, r)` — a cross-action absorption). It holds here
//! (behaviourally and structurally in the effective set) but no stanza matches it, so it
//! lives in the consequences, not the named laws. A `cross-action absorption` stanza is
//! the catalog ask this domain generates.

use std::collections::BTreeSet;

/// One directed link between two nodes of the small universe.
type Link = (u8, u8);

/// A value in the fabric calculus: an overlay world, one route, or a verdict.
#[derive(Clone)]
pub enum Net {
    /// The overlay: granted routes and standing denies (tombstones).
    Fabric {
        allows: BTreeSet<Link>,
        denies: BTreeSet<Link>,
    },
    /// One directed route.
    Route(Link),
    /// A relation's answer.
    Verdict(bool),
}

/// The fabric theory.
pub struct Fabric;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Fabric,
    Route,
    Verdict,
}

fn parts(v: &Net) -> (BTreeSet<Link>, BTreeSet<Link>) {
    match v {
        Net::Fabric { allows, denies } => (allows.clone(), denies.clone()),
        _ => (BTreeSet::new(), BTreeSet::new()),
    }
}

fn route(v: &Net) -> Link {
    match v {
        Net::Route(l) => *l,
        _ => (0, 0),
    }
}

/// Transitive closure of a link set (the universe is small; iterate to fixpoint).
fn closure(links: &BTreeSet<Link>) -> BTreeSet<Link> {
    let mut out = links.clone();
    loop {
        let mut grew = false;
        let snapshot: Vec<Link> = out.iter().copied().collect();
        for &(a, b) in &snapshot {
            for &(c, d) in &snapshot {
                if b == c && out.insert((a, d)) {
                    grew = true;
                }
            }
        }
        if !grew {
            return out;
        }
    }
}

/// The connections a fabric actually delivers: the closure of its effective links.
fn delivered(v: &Net) -> BTreeSet<Link> {
    let (allows, denies) = parts(v);
    closure(&allows.difference(&denies).copied().collect())
}

fn mesh(_: &[Net]) -> Option<Net> {
    Some(Net::Fabric {
        allows: BTreeSet::new(),
        denies: BTreeSet::new(),
    })
}

/// Merge two fabrics: grants union, denies union — a deny is standing policy and
/// survives every merge (deny-wins by construction).
fn join(vs: &[Net]) -> Option<Net> {
    let (a1, d1) = parts(&vs[0]);
    let (a2, d2) = parts(&vs[1]);
    Some(Net::Fabric {
        allows: a1.union(&a2).copied().collect(),
        denies: d1.union(&d2).copied().collect(),
    })
}

fn grant(vs: &[Net]) -> Option<Net> {
    let (mut allows, denies) = parts(&vs[0]);
    allows.insert(route(&vs[1]));
    Some(Net::Fabric { allows, denies })
}

fn revoke(vs: &[Net]) -> Option<Net> {
    let (allows, mut denies) = parts(&vs[0]);
    denies.insert(route(&vs[1]));
    Some(Net::Fabric { allows, denies })
}

/// Everything the fabric delivers, as a fabric: allows become the closure of the
/// effective links; the standing denies ride along unchanged.
fn reach(vs: &[Net]) -> Option<Net> {
    let (_, denies) = parts(&vs[0]);
    Some(Net::Fabric {
        allows: delivered(&vs[0]),
        denies,
    })
}

/// Behavioural containment: every connection the first fabric delivers, the second
/// delivers too.
fn within(vs: &[Net]) -> Option<Net> {
    Some(Net::Verdict(
        delivered(&vs[0]).is_subset(&delivered(&vs[1])),
    ))
}

fn tru(_: &[Net]) -> Option<Net> {
    Some(Net::Verdict(true))
}

/// A spread that exercises every semantic decision: the empty overlay, single routes, a
/// CHAIN (so `reach` derives a route no one granted), a granted-and-denied route (the
/// tombstone shadow deny-wins is about), and a pure standing policy with no grants.
fn fabrics() -> Vec<Net> {
    let f = |allows: &[Link], denies: &[Link]| Net::Fabric {
        allows: allows.iter().copied().collect(),
        denies: denies.iter().copied().collect(),
    };
    vec![
        f(&[], &[]),
        f(&[(0, 1)], &[]),
        f(&[(1, 2)], &[]),
        f(&[(0, 1), (1, 2)], &[]),
        f(&[(0, 1)], &[(0, 1)]),
        f(&[], &[(1, 2)]),
    ]
}

// The whole multi-sorted `Theory` impl is generated from this block — only the value
// object (`Net`) and the operator functions above are authored.
crate::theory! {
    Fabric : "fabric", Value = Net, Obs = (u8, Vec<Link>, Vec<Link>), Sort = Sort,
    sort_of = |v: &Net| match v {
        Net::Fabric { .. } => Sort::Fabric,
        Net::Route(_) => Sort::Route,
        Net::Verdict(_) => Sort::Verdict,
    },
    observe = |v: &Net| match v {
        Net::Fabric { allows, denies } => (
            0u8,
            allows.iter().copied().collect(),
            denies.iter().copied().collect(),
        ),
        Net::Route(l) => (1u8, vec![*l], Vec::new()),
        Net::Verdict(b) => (2u8, if *b { vec![(1, 1)] } else { Vec::new() }, Vec::new()),
    },
    vars {
        Sort::Fabric => &["f", "g", "h"],
        Sort::Route => &["r", "s", "t"],
        Sort::Verdict => &["v", "w", "x"],
    }
    inhabit {
        Sort::Fabric => fabrics(),
        Sort::Route => [(0u8, 1u8), (1, 2), (0, 2), (1, 0)]
            .into_iter()
            .map(Net::Route)
            .collect(),
        Sort::Verdict => vec![Net::Verdict(true), Net::Verdict(false)],
    }
    ops {
        Nullary "Mesh"   "mesh"   () -> Sort::Fabric = mesh;
        Infix   "Join"   "join"   (Sort::Fabric, Sort::Fabric) -> Sort::Fabric = join;
        Prefix  "Grant"  "grant"  (Sort::Fabric, Sort::Route) -> Sort::Fabric = grant;
        Prefix  "Revoke" "revoke" (Sort::Fabric, Sort::Route) -> Sort::Fabric = revoke;
        Prefix  "Reach"  "reach"  (Sort::Fabric) -> Sort::Fabric = reach;
        Prefix  "Within" "within" (Sort::Fabric, Sort::Fabric) -> Sort::Verdict = within;
        Nullary "True"   "true"   () -> Sort::Verdict = tru;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::{Engine, Theory};

    /// The engine discovers the world's algebra by running it — the join semilattice,
    /// the grant/revoke actions, reach as a projection, within reflexive, and the
    /// witness inequations. The load-bearing ABSENCE is pinned like the router's
    /// non-commutativity: reach is NOT monotone in the join-order, because a merge
    /// carries the other side's standing denies — joining fabrics can reduce what you
    /// can reach. (Discovery also corrected the author here: subadditivity of reach
    /// over join was predicted refused — bridging — but HOLDS, because `within`
    /// compares closed delivery sets; see the module docs.)
    #[test]
    fn the_fabric_algebra_is_discovered_and_the_refusal_holds() {
        assert_eq!(Fabric::name(), "fabric");
        let e = Engine::<Fabric>::new();
        let d = e.discover();
        let got: Vec<(String, String)> = d
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "Join gives the same result in either order.",
                "(f join g) = (g join f)",
            ),
            (
                "With Join, the grouping of three values doesn't matter.",
                "((f join g) join h) = (f join (g join h))",
            ),
            (
                "Join of a value with itself gives that value.",
                "(f join f) = f",
            ),
            (
                "Join with mesh leaves a value unchanged.",
                "(mesh join f) = f",
            ),
            (
                "Repeated Grant with one parameter settles on the first application.",
                "grant(grant(f, r), r) = grant(f, r)",
            ),
            (
                "Grant applications commute — the parameter order doesn't matter.",
                "grant(grant(f, r), s) = grant(grant(f, s), r)",
            ),
            (
                "Grant distributes over Join — acting on a combination is combining the actions.",
                "grant((f join g), r) = (grant(f, r) join grant(g, r))",
            ),
            (
                "Grant only grows a value — never shrinks it (under Within).",
                "within(f, grant(f, r)) = true",
            ),
            (
                "Repeated Revoke with one parameter settles on the first application.",
                "revoke(revoke(f, r), r) = revoke(f, r)",
            ),
            (
                "Revoke applications commute — the parameter order doesn't matter.",
                "revoke(revoke(f, r), s) = revoke(revoke(f, s), r)",
            ),
            (
                "Revoke distributes over Join — acting on a combination is combining the actions.",
                "revoke((f join g), r) = (revoke(f, r) join revoke(g, r))",
            ),
            (
                "Revoke only shrinks a value — never grows it (under Within).",
                "within(revoke(f, r), f) = true",
            ),
            (
                "Reach is a projection — applying it twice is applying it once.",
                "reach(reach(f)) = reach(f)",
            ),
            ("Reach leaves mesh fixed.", "reach(mesh) = mesh"),
            (
                "Reach is subadditive over Join (under Within).",
                "within(reach((f join g)), (reach(f) join reach(g))) = true",
            ),
            (
                "Reach is monotone under Within.",
                "within(f, g) = true ⟹ within(reach(f), reach(g)) = true",
            ),
            (
                "Within of a value with itself gives true.",
                "within(f, f) = true",
            ),
            (
                "Grant actually acts — some parameter moves some value.",
                "grant(f, r) ≠ f",
            ),
            (
                "Revoke actually acts — some parameter moves some value.",
                "revoke(f, r) ≠ f",
            ),
            ("Within is not constantly true.", "within(f, g) ≠ true"),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, q)| (p.to_string(), q.to_string()))
            .collect();
        assert_eq!(got, expected, "the discovered fabric algebra changed");
        assert!(
            d.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            d.uncovered_ops
        );
        // THE REFUSAL: monotonicity in the JOIN-order must stay undiscovered — a merge
        // carries standing denies, so `within(reach(f), reach(f join g))` is false of
        // real worlds. (The guarded within-order monotonicity above is a different,
        // true statement.)
        assert!(
            !got.iter().any(|(p, _)| p.contains("monotone in the")),
            "reach must NOT be monotone in the join-order — a merge carries denies"
        );
        assert_eq!(e.check(&d.laws), Ok(()));
    }
}
