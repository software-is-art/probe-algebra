//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! derived — a theory with NO hand-written grid: the value TYPE generates it.
//!
//! Every other domain lists its `inhabitants` by hand — the grid the laws are judged over. That grid
//! is the one part of a `theory!` that resists derivation. Here it is derived: the value object
//! `derive(Shaped)`s its structure, and the grid is grown from that structure by `shadow_grid` — a
//! SHADOW ALGEBRA of synthetic generators (variant swaps, field neighbours) the agent never writes
//! and that never enter the discovered spec. The boundary operators define the laws; the type
//! structure fattens the grid they are judged on.
//!
//! `Tri` is a three-element lattice (`Lo ≤ Mid ≤ Hi`) with `meet`/`join` and — crucially — NO
//! constant operators. So closing the grid under the BOUNDARY operators would never leave the empty
//! set: there is nothing to start from. Only the shadow algebra (the type's three inhabitants) can
//! seed it — and from it the engine discovers the full lattice spec, hands-free.
//!
//! It is also written in the MINIMAL form of `theory!` — no `Obs`, no `observe`, no `vars`, no
//! `inhabit`. For a first-order value object the observation IS the value, the variable letters
//! default, and the grid is shadow-derived, so the whole domain is the FLOOR: the value type, its
//! sort, and its operators. Everything a `theory!` once spelled out is either defaulted or derived;
//! what is left is only the irreducible meaning. (A behavioural observer or a curated grid — like the
//! router's, or arithmetic's — is a deliberate deviation from this floor, and stays written out.)

use crate::Shaped;

/// A three-element lattice value object — its grid comes entirely from its `Shaped` structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
pub enum Tri {
    Lo,
    Mid,
    Hi,
}

/// The lattice's single sort.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum LatticeSort {
    T,
}

fn meet(v: &[Tri]) -> Option<Tri> {
    Some(v[0].min(v[1]))
}
fn join(v: &[Tri]) -> Option<Tri> {
    Some(v[0].max(v[1]))
}

/// The lattice algebra — `meet` and `join`, no constants.
pub struct Lattice;

crate::theory! {
    Lattice : "tri lattice", Value = Tri, Sort = LatticeSort,
    sort_of = |_: &Tri| LatticeSort::T,
    ops {
        Infix "meet" "&" (LatticeSort::T, LatticeSort::T) -> LatticeSort::T = meet;
        Infix "join" "|" (LatticeSort::T, LatticeSort::T) -> LatticeSort::T = join;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::{shadow_grid, Engine, Theory};

    /// The grid comes from the TYPE, not the operators. `Tri`'s `Shaped` structure yields its three
    /// inhabitants, and that is the grid the macro wires in — even though the boundary has NO constant
    /// to bootstrap from. Without the shadow algebra this theory would have an EMPTY grid.
    #[test]
    fn the_grid_is_generated_from_the_type_structure() {
        assert_eq!(shadow_grid::<Tri>(24), vec![Tri::Lo, Tri::Mid, Tri::Hi]);
        assert_eq!(
            <Lattice as Theory>::inhabitants(LatticeSort::T),
            vec![Tri::Lo, Tri::Mid, Tri::Hi]
        );
        // the crux: there is no constant operator, so the grid CANNOT come from the boundary — only
        // the shadow algebra (the type structure) can produce one.
        assert!(
            <Lattice as Theory>::operators()
                .iter()
                .all(|o| !o.inputs.is_empty()),
            "the lattice has no constant operator to seed a grid from"
        );
    }

    /// Over that self-generated grid the engine discovers the WHOLE distributive-lattice spec — `meet`
    /// and `join` each commutative, associative, idempotent, with both distributivities and both
    /// absorptions — with no hand-written inhabitants and no variable letters. The shadow generators
    /// never appear: every law is over the boundary operators. The theory is just its operators.
    #[test]
    fn the_lattice_is_discovered_hands_free() {
        let proses: Vec<String> = Engine::<Lattice>::new()
            .discover()
            .laws
            .iter()
            .map(|l| l.prose.clone())
            .collect();
        assert_eq!(
            proses,
            vec![
                "meet gives the same result in either order.".to_string(),
                "With meet, the grouping of three values doesn't matter.".to_string(),
                "meet of a value with itself gives that value.".to_string(),
                "meet distributes over join.".to_string(),
                "meet absorbs join.".to_string(),
                "join gives the same result in either order.".to_string(),
                "With join, the grouping of three values doesn't matter.".to_string(),
                "join of a value with itself gives that value.".to_string(),
                "join distributes over meet.".to_string(),
                "join absorbs meet.".to_string(),
            ],
            "the lattice's discovered spec, over an entirely type-generated grid"
        );
    }
}
