//! census — the VISUAL CENSUS: the constraints EMERGE from the corpus instead of being
//! typed by anyone.
//!
//! The census derives, from the ratified source corpus rendered by the stable engine,
//! the regularities the layouts actually exhibit — the horizontal and vertical floors
//! (the smallest gaps that occur anywhere), rank alignment, and the extremes a new
//! diagram could move. Every row is FREEZE-STABLE by construction: a floor or a
//! maximum is a fact a new corpus member can only move VISIBLY (the diff names the
//! movement), never smear the way a mean would. Nobody decides the pitch is 4; the
//! corpus exhibits it, the freeze ratifies it, and from then on a cramped layout is a
//! red gate reading "the horizontal floor moved 4 → 1 — ratify or fix", never silent
//! erosion. (The qualify-census move, pointed at pixels; the epistemics are the
//! method's: a derived floor is DESCRIPTIVE, the freeze is the only normative act.)
//!
//! Also here: the LOCALITY WITNESS — insertion locality measured, not asserted. For
//! each corpus member we add one early-sorting node (`_new`, attached to the first
//! declared node) and one late-sorting node (`zz`), re-render, and record the maximum
//! displacement of the PRE-EXISTING nodes. The numbers are computed by the real
//! engines and disclose another face of the same tradeoff: the stable engine shifts a
//! whole rank when an early name arrives (the name is load-bearing for position); the
//! eager engine appends and displaces nothing.

use std::path::Path;

use crate::diagram::Diagram;
use crate::layout::{layout, Policy};
use crate::theories::source_grid;

/// Max |Δ| displacement of pre-existing nodes when `name` (attached to the first
/// declared node, or standalone on an empty diagram) joins `d`.
fn displacement_when_adding(d: &Diagram, name: &str, policy: Policy) -> i64 {
    let before = layout(d, policy);
    let mut nodes: Vec<&str> = d.nodes().iter().map(String::as_str).collect();
    nodes.push(name);
    let mut edges: Vec<(&str, &str)> = d
        .edges()
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    if let Some(first) = d.nodes().first() {
        edges.push((first.as_str(), name));
    }
    let after = layout(&Diagram::of(&nodes, &edges), policy);
    before
        .placements()
        .iter()
        .filter_map(|(n, (x0, y0))| {
            after
                .placements()
                .iter()
                .find(|(m, _)| m == n)
                .map(|(_, (x1, y1))| (x1 - x0).abs() + (y1 - y0).abs())
        })
        .max()
        .unwrap_or(0)
}

/// The census text — deterministic, human-readable, freeze-stable rows only.
pub fn render() -> String {
    render_over(&source_grid())
}

/// [`render`] over an explicit corpus — the fire drill's handle: growing the corpus
/// with a wider or denser diagram must move a row VISIBLY, and the drill demands it.
pub fn render_over(corpus: &[Diagram]) -> String {
    let corpus = corpus.to_vec();
    let mut h_floor: Option<i64> = None;
    let mut v_floor: Option<i64> = None;
    let mut max_per_rank = 0usize;
    let mut aligned = true;
    for d in &corpus {
        let g = layout(d, Policy::Stable);
        let mut rows: std::collections::BTreeMap<i64, Vec<i64>> = Default::default();
        for (_, (x, y)) in g.placements() {
            rows.entry(*y).or_default().push(*x);
        }
        let ys: Vec<i64> = rows.keys().copied().collect();
        for w in ys.windows(2) {
            let gap = w[1] - w[0];
            v_floor = Some(v_floor.map_or(gap, |f: i64| f.min(gap)));
        }
        for xs in rows.values_mut() {
            xs.sort_unstable();
            max_per_rank = max_per_rank.max(xs.len());
            for w in xs.windows(2) {
                let gap = w[1] - w[0];
                if gap == 0 {
                    aligned = false; // two nodes share a cell: an overlap, not a rank.
                }
                h_floor = Some(h_floor.map_or(gap, |f: i64| f.min(gap)));
            }
        }
    }
    let show = |f: Option<i64>| f.map_or("none (no pair occurs)".to_string(), |v| v.to_string());
    let disp = |name: &str, policy: Policy| -> i64 {
        corpus
            .iter()
            .map(|d| displacement_when_adding(d, name, policy))
            .max()
            .unwrap_or(0)
    };
    format!(
        "# visual census: the constraints the ratified corpus EXHIBITS — derived, never typed;\n\
         # regenerate via `cargo run -p layout-probe --example freeze` and ratify the diff.\n\
         #\n\
         # Every row is freeze-stable: a floor or a maximum a new corpus member moves VISIBLY.\n\
         # A derived floor is DESCRIPTIVE of the corpus; committing this file is the normative\n\
         # act — from then on erosion is a red gate naming the movement, and raising a\n\
         # standard by example is a reviewable diff too.\n\n\
         corpus: {} diagrams (the source grid, rendered by the stable engine)\n\
         horizontal floor (smallest within-rank gap): {}\n\
         vertical floor (smallest rank gap): {}\n\
         widest rank: {} nodes\n\
         rank alignment: {}\n\n\
         # locality witness — insertion displacement, MEASURED (computed by the real\n\
         # engines, never asserted): max |Δ| of pre-existing nodes when one node joins.\n\
         # The early-sorting name exposes the stable engine's name/position coupling;\n\
         # the late-sorting name is the benign case. Another face of the one tradeoff.\n\
         insert `_new` (sorts early): stable moves {}, eager moves {}\n\
         insert `zz` (sorts late):   stable moves {}, eager moves {}\n",
        corpus.len(),
        show(h_floor),
        show(v_floor),
        max_per_rank,
        if aligned {
            "every rank shares one row (no overlaps)"
        } else {
            "OVERLAP — two nodes share a cell"
        },
        disp("_new", Policy::Stable),
        disp("_new", Policy::Eager),
        disp("zz", Policy::Stable),
        disp("zz", Policy::Eager),
    )
}

/// The census as a lock in this crate's `spec/` directory.
pub fn lock_in(spec_dir: &Path) -> spec_lock::Lock {
    spec_lock::Lock {
        name: "visual census".into(),
        path: spec_dir.join("visual.census"),
        live: render(),
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The locality witness's OPPOSITION, pinned point-blank (the census lock carries
    /// the same numbers; this pin keeps the measurement function itself honest): an
    /// early-sorting insertion shifts a whole rank under the stable engine and nothing
    /// under the eager one; a late-sorting insertion is benign under both.
    #[test]
    fn the_locality_opposition_is_measured_not_asserted() {
        // the diamond: `_new` attaches under `a` and joins rank 1 = {b, c}; its
        // early-sorting name shifts that whole rank under the stable engine.
        let d = Diagram::of(
            &["a", "b", "c", "d"],
            &[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")],
        );
        assert_eq!(
            displacement_when_adding(&d, "_new", Policy::Stable),
            crate::layout::H_GAP
        );
        assert_eq!(displacement_when_adding(&d, "_new", Policy::Eager), 0);
        assert_eq!(displacement_when_adding(&d, "zz", Policy::Stable), 0);
        assert_eq!(displacement_when_adding(&d, "zz", Policy::Eager), 0);
        // and a node landing ALONE in a new rank displaces nothing under either
        // engine — locality trouble needs a shared rank, pinned so the witness's
        // mechanism stays legible.
        let pair = Diagram::of(&["b", "a"], &[]);
        assert_eq!(displacement_when_adding(&pair, "_new", Policy::Stable), 0);
    }

    /// Corpus growth moves the census VISIBLY: a diagram with a three-node rank widens
    /// the widest-rank row — the freeze-stability property every census row promises.
    #[test]
    fn a_wider_corpus_member_moves_the_census_visibly() {
        let mut grown = source_grid();
        grown.push(Diagram::of(&["p", "q", "r"], &[]));
        let text = render_over(&grown);
        assert!(text.contains("widest rank: 3 nodes"), "{text}");
        assert_ne!(text, render(), "growth must be visible, never absorbed");
    }
}
