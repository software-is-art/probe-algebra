//! select::internal — the greedy set-cover selector. PRIVATE; reached only through
//! `select::boundary`. Like the interpreter's interior it carries **zero example tests**:
//! the oracle-free property probes below — cover validity, the coverable/uncoverable
//! partition, the no-useless-pick greedy invariant, the discrimination preference — are its
//! entire verification, and `cargo mutants` (now scoped over this file) measures whether
//! that suffices. This is the method turned on its own kernel.
//!
//! Result of that sweep: EVERY mutant of this file is detected — zero survivors. The three
//! equivalents earlier runs found were not excluded but ENGINEERED AWAY: the discrimination
//! key uses `Reverse(total_kills)` (no `usize::MAX - x` arithmetic to mutate), selection uses
//! `max_by_key` (no hand-written `>` whose tie-break a `>=` mutant could flip), and the noise
//! check drops its redundant `mutants > 0` guard (a row only reaches it with `new > 0`, which
//! already implies a mutant). What remains as TIMEOUT — relaxing the loop's termination guard
//! (`new > 0`) or its coverage test (`row[m] && !covered[m]`) so the greedy never stops — is a
//! genuine detection (the mutant hangs; the original terminates), not a gap.

use crate::gdp::{at_in_bounds, positions, prove_position, with_region};
use crate::select::boundary::{Index, KillMatrix};

/// Whether relation `r` kills every mutant (noisy: kills all, attributes nothing). The row
/// is read through a GDP `InBounds` proof tying `r` to THIS matrix (`prove_position` ⇒
/// `at_in_bounds`), so an out-of-range relation is a defined `false` (it kills nothing)
/// rather than the panic a raw `rows[r]` would risk — the bounds obligation discharged by a
/// proof, not by trust.
pub(super) fn is_noisy(matrix: &KillMatrix, relation: Index) -> bool {
    let mutants = matrix.mutants();
    with_region(|region| {
        let rows = region.brand(matrix.rows().to_vec());
        match prove_position(&rows, relation.get()) {
            Some(w) => {
                let row = at_in_bounds(&rows, w.named(), w.proof());
                mutants > 0 && row.iter().filter(|&&k| k).count() == mutants
            }
            None => false,
        }
    })
}

/// Greedy set cover with the discrimination preference. Returns the selected relation
/// indices in pick order; mutants no relation kills are skipped (see `uncoverable`).
///
/// The matrix is branded into a region and every row read through its `InBounds` proof
/// (`positions` ⇒ `at_in_bounds`): the kernel indexes its own matrix entirely by proof, and
/// a position witnessed for this matrix cannot index another.
pub(super) fn select(matrix: &KillMatrix) -> Vec<Index> {
    let mutants = matrix.mutants();
    let total_kills = |row: &[bool]| row.iter().filter(|&&k| k).count();

    with_region(|region| {
        let rows = region.brand(matrix.rows().to_vec());
        let row_positions = positions(&rows);

        let mut covered = vec![false; mutants];
        let mut selected: Vec<Index> = Vec::new();

        loop {
            // Rank every row that still covers something and take the best. The key is
            // compared by `max_by_key` (no hand-written comparison to mutate), larger wins:
            //   1. non-noisy preferred (defer kill-everything relations),
            //   2. more newly-covered mutants,
            //   3. fewer total kills = more discriminating (`Reverse`, no arithmetic to
            //      mutate),
            //   4. (implicit) ties resolved by `max_by_key`, an equally valid cover.
            // `noisy` needs no `mutants > 0` guard: a row only reaches here with `new > 0`,
            // which already implies at least one mutant.
            let best = row_positions
                .iter()
                .filter_map(|w| {
                    let row = at_in_bounds(&rows, w.named(), w.proof());
                    let new = (0..mutants).filter(|&m| row[m] && !covered[m]).count();
                    (new > 0).then(|| {
                        let noisy = total_kills(row) == mutants;
                        (
                            *w.named().value(),
                            (!noisy, new, core::cmp::Reverse(total_kills(row))),
                        )
                    })
                })
                .max_by_key(|&(_, key)| key);

            match best {
                Some((r, _)) => {
                    // re-read the chosen row by a fresh proof (it came from a proven position).
                    let chosen = prove_position(&rows, r).expect("r is a proven row position");
                    let row = at_in_bounds(&rows, chosen.named(), chosen.proof());
                    for (slot, &killed) in covered.iter_mut().zip(row.iter()) {
                        if killed {
                            *slot = true;
                        }
                    }
                    selected.push(Index::new(r).expect("a row index is non-negative"));
                }
                None => break,
            }
        }

        selected
    })
}

/// Mutants no relation kills — surviving mutants, each evidence that a relation or a degree
/// of freedom is MISSING. Rows are read by proof, exactly as in `select`.
pub(super) fn uncoverable(matrix: &KillMatrix) -> Vec<Index> {
    let mutants = matrix.mutants();
    with_region(|region| {
        let rows = region.brand(matrix.rows().to_vec());
        let row_positions = positions(&rows);
        (0..mutants)
            .filter(|&m| {
                !row_positions
                    .iter()
                    .any(|w| at_in_bounds(&rows, w.named(), w.proof())[m])
            })
            .map(|m| Index::new(m).expect("a column index is non-negative"))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use crate::select::boundary::{Index, KillMatrix};
    use proptest::prelude::*;

    /// Random RECTANGULAR grids — the oracle-free probes run over these, never over
    /// hand-picked example tables.
    fn grid() -> impl Strategy<Value = Vec<Vec<bool>>> {
        (1usize..=4, 0usize..=4).prop_flat_map(|(rows, cols)| {
            prop::collection::vec(
                prop::collection::vec(any::<bool>(), cols..=cols),
                rows..=rows,
            )
        })
    }

    /// Whether mutant column `m` is killed by ANY relation (coverable at all).
    fn coverable(grid: &[Vec<bool>], m: usize) -> bool {
        grid.iter().any(|row| row[m])
    }

    proptest! {
        /// CORRECTNESS: the cover kills every coverable mutant. The core invariant — a
        /// selector that drops a killable mutant is wrong.
        #[test]
        fn cover_kills_every_coverable_mutant(grid in grid()) {
            let matrix = KillMatrix::new(grid.clone()).expect("rectangular");
            let cover = matrix.select();
            for m in 0..matrix.mutants() {
                if coverable(&grid, m) {
                    let killed = cover.positions().iter().any(|r| grid[r.get()][m]);
                    prop_assert!(killed, "coverable mutant {} left uncovered", m);
                }
            }
        }

        /// PARTITION: every mutant is coverable XOR uncoverable, and `uncoverable` reports
        /// exactly the all-false columns.
        #[test]
        fn coverable_and_uncoverable_partition_the_mutants(grid in grid()) {
            let matrix = KillMatrix::new(grid.clone()).expect("rectangular");
            // pin the width accessor against the grid (else `mutants() -> 0` makes every
            // column loop vacuous).
            prop_assert_eq!(matrix.mutants(), grid.first().map(Vec::len).unwrap_or(0));
            let survivors = matrix.uncoverable();
            for m in 0..matrix.mutants() {
                let idx = Index::new(m).unwrap();
                prop_assert_eq!(survivors.contains(idx), !coverable(&grid, m));
            }
        }

        /// GREEDY INVARIANT: no useless pick — every selected relation kills at least one
        /// mutant, and the selection is duplicate-free.
        #[test]
        fn no_selected_relation_is_useless(grid in grid()) {
            let matrix = KillMatrix::new(grid.clone()).expect("rectangular");
            let cover = matrix.select();
            let picks = cover.positions();
            for (i, r) in picks.iter().enumerate() {
                prop_assert!(r.get() < matrix.relations(), "index out of bounds");
                prop_assert!(grid[r.get()].iter().any(|&k| k), "a relation killing nothing was picked");
                prop_assert!(!picks[..i].contains(r), "a relation was picked twice");
            }
        }

        /// NOISE, pinned INDEPENDENTLY: `is_noisy(r)` holds iff there is at least one mutant
        /// and relation `r` kills all of them — checked against the grid directly, NOT via
        /// `is_noisy` itself (the discrimination probe below uses `is_noisy` in its own
        /// guard, so a broken `is_noisy` would make it vacuous; this anchors it).
        #[test]
        fn is_noisy_matches_killing_every_mutant(grid in grid()) {
            let matrix = KillMatrix::new(grid.clone()).expect("rectangular");
            for (r, row) in grid.iter().enumerate() {
                let kills_all = matrix.mutants() > 0 && row.iter().all(|&k| k);
                prop_assert_eq!(matrix.is_noisy(Index::new(r).unwrap()), kills_all);
            }
        }

        /// DISCRIMINATION: if the NON-noisy relations alone cover everything coverable, then
        /// no noisy (kill-everything) relation is selected — attribution is preferred over a
        /// smaller cover.
        #[test]
        fn discrimination_defers_noisy_relations(grid in grid()) {
            let matrix = KillMatrix::new(grid.clone()).expect("rectangular");
            let specifics_suffice = (0..matrix.mutants()).all(|m| {
                !coverable(&grid, m)
                    || (0..matrix.relations())
                        .any(|r| grid[r][m] && !matrix.is_noisy(Index::new(r).unwrap()))
            });
            if specifics_suffice {
                for r in matrix.select().positions() {
                    prop_assert!(!matrix.is_noisy(*r), "a noisy relation was selected needlessly");
                }
            }
        }

        /// REDUNDANCY INVARIANCE (metamorphic): appending a duplicate of an existing
        /// relation cannot change which mutants end up covered.
        #[test]
        fn a_duplicate_relation_does_not_change_coverage(grid in grid()) {
            prop_assume!(!grid.is_empty());
            let base = KillMatrix::new(grid.clone()).expect("rectangular");
            let mut grown = grid.clone();
            grown.push(grid[0].clone());
            let grown = KillMatrix::new(grown).expect("rectangular");

            let covered = |cover: &crate::select::boundary::Cover, src: &[Vec<bool>]| -> Vec<bool> {
                (0..src[0].len())
                    .map(|m| cover.positions().iter().any(|r| src[r.get()][m]))
                    .collect::<Vec<bool>>()
            };
            let a = covered(&base.select(), &grid);
            let b = covered(&grown.select(), grown.rows());
            prop_assert_eq!(a, b, "a redundant relation changed the covered set");
        }
    }

    /// VALIDITY RULE: a ragged grid is rejected (parse, don't validate); a rectangular one
    /// is admitted. Pins the one thing the round-trip cannot — that the check rejects.
    #[test]
    fn construction_rejects_ragged_grids() {
        assert!(KillMatrix::new(vec![vec![true, false], vec![true]]).is_none());
        assert!(KillMatrix::new(vec![vec![true, false], vec![false, true]]).is_some());
        assert!(KillMatrix::new(vec![]).is_some());
    }
}
