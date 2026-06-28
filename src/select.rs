//! select — choose a minimal, DISCRIMINATING relation set from a kill matrix.
//!
//! This is the selection half of the generation+selection loop (§2 of the
//! design). Generation enumerates candidate relations (the commutation /
//! coefficient templates instantiated against each operator) and candidate
//! mutants; running every relation against every mutant produces a KILL MATRIX.
//! This module turns that matrix into a small, powerful, *attributable* suite.
//!
//! Two ideas drive the choice:
//!   - **Minimal cover** — pick the fewest relations that still kill every
//!     killable mutant (greedy set cover).
//!   - **Discrimination preference** — among choices, prefer relations that kill
//!     FEW mutants over ones that kill many. A relation that kills everything
//!     proves *a* bug exists but cannot ATTRIBUTE it to a dimension; a relation
//!     that kills exactly one tells you precisely what broke. When minimality and
//!     attribution conflict (one kill-everything relation vs several specific
//!     ones) we choose attribution.
//!
//! A mutant NO relation kills is a survivor — the discovery signal that a
//! relation or a degree of freedom is MISSING (see `uncoverable`).
//!
//! This module is crate-level tooling (like the grammar and `build.rs`), so it
//! is exempt from the boundary discipline: its values are selection indices, not
//! domain data.

/// A kill matrix. Rows are relations, columns are mutants; `kills[r][m]` is true
/// iff relation `r` detects mutant `m`.
///
/// The honest reference implementation is deliberately NOT a column: a relation
/// that flags the correct implementation is unsound and must be filtered out
/// before it ever enters the matrix.
pub struct KillMatrix {
    kills: Vec<Vec<bool>>,
    mutants: usize,
}

impl KillMatrix {
    /// Build from a rectangular `relations x mutants` boolean grid.
    pub fn new(kills: Vec<Vec<bool>>) -> Self {
        let mutants = kills.first().map(Vec::len).unwrap_or(0);
        debug_assert!(
            kills.iter().all(|row| row.len() == mutants),
            "kill matrix rows must all have the same width"
        );
        KillMatrix { kills, mutants }
    }

    /// How many mutants relation `r` kills in total.
    fn total_kills(&self, r: usize) -> usize {
        self.kills[r].iter().filter(|&&k| k).count()
    }

    /// A relation is NOISY if it kills every mutant: maximal recall, zero
    /// attribution.
    pub fn is_noisy(&self, r: usize) -> bool {
        self.mutants > 0 && self.total_kills(r) == self.mutants
    }

    /// Greedy set cover with the discrimination preference. Returns the selected
    /// relation indices in pick order. Mutants that no relation kills are skipped
    /// (and surfaced by [`uncoverable`](Self::uncoverable)).
    pub fn select(&self) -> Vec<usize> {
        let mut covered = vec![false; self.mutants];
        let mut selected = Vec::new();

        loop {
            let mut best: Option<(usize, (bool, usize, usize))> = None;
            for r in 0..self.kills.len() {
                let new = (0..self.mutants)
                    .filter(|&m| self.kills[r][m] && !covered[m])
                    .count();
                if new == 0 {
                    continue;
                }
                // Ranking key, compared lexicographically, larger wins:
                //   1. non-noisy preferred (defer kill-everything relations),
                //   2. more newly-covered mutants,
                //   3. fewer total kills = more discriminating,
                //   4. (implicit) lower index, via strict-greater keeping the first.
                let key = (!self.is_noisy(r), new, usize::MAX - self.total_kills(r));
                if best.map(|(_, bk)| key > bk).unwrap_or(true) {
                    best = Some((r, key));
                }
            }

            match best {
                Some((r, _)) => {
                    for (slot, &killed) in covered.iter_mut().zip(self.kills[r].iter()) {
                        if killed {
                            *slot = true;
                        }
                    }
                    selected.push(r);
                }
                None => break,
            }
        }

        selected
    }

    /// Mutants no relation kills — surviving mutants. Each one is evidence that a
    /// relation or a degree of freedom is MISSING.
    pub fn uncoverable(&self) -> Vec<usize> {
        (0..self.mutants)
            .filter(|&m| !self.kills.iter().any(|row| row[m]))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::KillMatrix;

    /// Discrimination beats minimality: a single kill-everything relation (R2)
    /// would give a size-1 cover, but it cannot attribute, so the two specific
    /// relations are chosen instead.
    #[test]
    fn prefers_discriminating_over_noisy() {
        //        M0     M1
        // R0   [true , false]   kills only M0
        // R1   [false, true ]   kills only M1
        // R2   [true , true ]   noisy: kills everything
        let m = KillMatrix::new(vec![vec![true, false], vec![false, true], vec![true, true]]);
        assert!(m.is_noisy(2));
        assert_eq!(m.select(), vec![0, 1]);
    }

    /// With no noisy relation, it computes a genuine minimal cover and, on ties,
    /// keeps the more discriminating relation.
    #[test]
    fn minimal_cover_with_discrimination_tiebreak() {
        //        M0     M1     M2
        // R0   [true , true , false]
        // R1   [false, true , true ]
        // R2   [false, false, true ]
        let m = KillMatrix::new(vec![
            vec![true, true, false],
            vec![false, true, true],
            vec![false, false, true],
        ]);
        // R0 covers {M0,M1}; then M2 is covered by R1 (also kills M1) or R2 (kills
        // only M2) — the more discriminating R2 wins the tie.
        assert_eq!(m.select(), vec![0, 2]);
    }

    /// With no mutants there is nothing to be noisy about: `is_noisy` is false.
    #[test]
    fn is_noisy_is_false_with_no_mutants() {
        let m = KillMatrix::new(vec![vec![]]);
        assert!(!m.is_noisy(0));
    }

    /// A mutant no relation kills is reported as a survivor (missing relation/DOF).
    #[test]
    fn surfaces_uncoverable_mutants() {
        //        M0     M1
        // R0   [true , false]
        let m = KillMatrix::new(vec![vec![true, false]]);
        assert_eq!(m.uncoverable(), vec![1]);
        assert_eq!(m.select(), vec![0]);
    }
}
