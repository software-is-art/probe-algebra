//! layout — two deterministic layered engines over one algorithm, split by ONE policy
//! bit: how nodes are ordered within a rank.
//!
//! The algorithm is Sugiyama-lite on an integer grid: rank = longest path from a
//! source (relaxed to a fixed bound, so cycles terminate), x = position within the
//! rank × [`H_GAP`], y = rank × [`V_GAP`]. Everything is total and deterministic; the
//! policies differ only in the within-rank order key:
//!
//! - [`Policy::Stable`] orders by NODE NAME — immune to declaration reorder, but a
//!   rename moves nodes (the name is load-bearing for position);
//! - [`Policy::Eager`] orders by DECLARATION INDEX — immune to rename, but a reorder
//!   moves nodes: dagre's "agent-edited diagrams jump between iterations" pathology,
//!   reproduced honestly.
//!
//! Neither policy wins both sides; that tradeoff is exactly what the discovered specs
//! name (`inert(reorder)` vs `equivariant(render, rename, relabel)`), which is the
//! point of the member: the scorecard is a fact about layout engines, not a bug in
//! either one.

use crate::diagram::{Diagram, Swap};

/// Horizontal grid pitch between within-rank neighbours.
pub const H_GAP: i64 = 4;
/// Vertical grid pitch between ranks.
pub const V_GAP: i64 = 2;

/// A rendered layout: node name → integer grid position, CANONICAL (sorted by name),
/// so structural equality is semantic equality.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Geometry(Vec<(String, (i64, i64))>);

impl Geometry {
    /// Build in canonical form.
    pub fn of(mut placed: Vec<(String, (i64, i64))>) -> Geometry {
        placed.sort();
        Geometry(placed)
    }

    /// The placements, canonical — the observation every theory fingerprints, and the
    /// sanctioned exit hatch.
    pub fn placements(&self) -> &[(String, (i64, i64))] {
        &self.0
    }

    /// Apply a rename to the KEYS only — positions stay put. This is the geometry-side
    /// action the equivariance square runs against: a rename-equivariant engine
    /// satisfies `render(rename(d, p)) = relabel(render(d), p)`.
    pub fn relabel(&self, swap: &Swap) -> Geometry {
        Geometry::of(
            self.0
                .iter()
                .map(|(n, pos)| (swap.apply(n), *pos))
                .collect(),
        )
    }
}

/// The one policy bit the two engines differ by.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Policy {
    /// Within-rank order by node name.
    Stable,
    /// Within-rank order by declaration index.
    Eager,
}

/// The layered layout, total and deterministic. Ranks by bounded longest-path
/// relaxation (a cycle simply stops relaxing at the bound); within-rank order by the
/// policy; positions on the integer grid at [`H_GAP`]/[`V_GAP`] pitch.
pub fn layout(d: &Diagram, policy: Policy) -> Geometry {
    let nodes = d.nodes();
    let index = |name: &str| nodes.iter().position(|n| n == name);
    let mut rank = vec![0usize; nodes.len()];
    // bounded relaxation: |nodes| passes reach any longest path an acyclic diagram
    // has; a cycle stops moving when the bound runs out — total, never looping.
    for _ in 0..nodes.len() {
        let mut moved = false;
        for (a, b) in d.edges() {
            let (ia, ib) = match (index(a), index(b)) {
                (Some(ia), Some(ib)) => (ia, ib),
                _ => continue,
            };
            if ia != ib && rank[ib] < rank[ia] + 1 && rank[ia] < nodes.len() {
                rank[ib] = rank[ia] + 1;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    // within-rank order: the policy's ONE decision.
    let mut placed: Vec<(String, (i64, i64))> = Vec::new();
    let max_rank = rank.iter().copied().max().unwrap_or(0);
    for r in 0..=max_rank {
        let mut members: Vec<usize> = (0..nodes.len()).filter(|i| rank[*i] == r).collect();
        match policy {
            Policy::Stable => members.sort_by(|a, b| nodes[*a].cmp(&nodes[*b])),
            Policy::Eager => {} // declaration order IS the order.
        }
        for (slot, i) in members.iter().enumerate() {
            placed.push((nodes[*i].clone(), (slot as i64 * H_GAP, r as i64 * V_GAP)));
        }
    }
    Geometry::of(placed)
}

#[cfg(test)]
mod probes {
    use super::*;

    fn diamond() -> Diagram {
        Diagram::of(
            &["a", "b", "c", "d"],
            &[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")],
        )
    }

    /// The layered skeleton, pinned on the diamond: ranks by longest path, one row per
    /// rank, positions on the declared pitch — the values the visual census derives
    /// its floors from.
    #[test]
    fn the_diamond_lays_out_in_three_ranks_on_the_declared_pitch() {
        let g = layout(&diamond(), Policy::Stable);
        assert_eq!(
            g.placements(),
            [
                ("a".to_string(), (0, 0)),
                ("b".to_string(), (0, V_GAP)),
                ("c".to_string(), (H_GAP, V_GAP)),
                ("d".to_string(), (0, 2 * V_GAP)),
            ]
        );
    }

    /// THE POLICY SPLIT, point-blank: on a declaration whose order disagrees with the
    /// name order, `Stable` ignores the declaration (reorder-inert) and `Eager` obeys
    /// it (reorder-sensitive) — the two engines' whole difference in one fixture.
    #[test]
    fn the_policies_disagree_exactly_on_within_rank_order() {
        let d = Diagram::of(&["b", "a"], &[]);
        let stable = layout(&d, Policy::Stable);
        let eager = layout(&d, Policy::Eager);
        assert_eq!(
            stable.placements(),
            [("a".to_string(), (0, 0)), ("b".to_string(), (H_GAP, 0))]
        );
        assert_eq!(
            eager.placements(),
            [("a".to_string(), (H_GAP, 0)), ("b".to_string(), (0, 0))]
        );
        // reorder flips eager, not stable.
        assert_eq!(layout(&d.reorder(), Policy::Stable), stable);
        assert_ne!(layout(&d.reorder(), Policy::Eager), eager);
    }

    /// A cyclic diagram terminates (the relaxation bound is the totality guarantee)
    /// and still places every node.
    #[test]
    fn a_cycle_terminates_and_places_every_node() {
        let d = Diagram::of(&["a", "b"], &[("a", "b"), ("b", "a")]);
        let g = layout(&d, Policy::Stable);
        assert_eq!(g.placements().len(), 2);
    }

    /// relabel moves KEYS, never positions — the geometry-side half of the
    /// equivariance square.
    #[test]
    fn relabel_moves_names_and_not_positions() {
        let g = layout(&diamond(), Policy::Stable);
        let r = g.relabel(&Swap::of("a", "d"));
        assert_eq!(
            r.placements().iter().find(|(n, _)| n == "d").unwrap().1,
            (0, 0),
            "d inherited a's position; nothing re-laid-out"
        );
        assert_eq!(
            g.relabel(&Swap::of("a", "d")).relabel(&Swap::of("a", "d")),
            g
        );
    }
}
