//! Tier: BOUNDARY — a domain's strict value-object surface (tier 1 grammar).
//!
//! select::boundary — choose a minimal, DISCRIMINATING relation set from a kill matrix,
//! specified as a boundary module.
//!
//! This is the SELF-HOST. `select` is part of the method's own kernel — given the kill
//! matrix (every candidate relation run against every mutant) it picks the small,
//! attributable suite that does the killing. Here that selector is specified in the very
//! discipline it serves: its data are value objects, its index is a citizen, and its
//! interior (`select::internal`) carries NO example tests — only the oracle-free probes in
//! `internal`'s test module, judged by the mutation sweep, certify it. The grammar is thus
//! turned on its own implementation, the way a compiler compiles itself.
//!
//! Two ideas drive the choice:
//!   - **Minimal cover** — the fewest relations that still kill every killable mutant
//!     (greedy set cover); and
//!   - **Discrimination preference** — among choices, prefer relations that kill FEW
//!     mutants: one that kills everything proves a bug exists but cannot ATTRIBUTE it; one
//!     that kills exactly one says precisely what broke.
//!
//! A mutant no relation kills is a survivor — the signal that a relation or a degree of
//! freedom is MISSING (`KillMatrix::uncoverable`).

// A position in the kill matrix — a relation row or a mutant column. The index CITIZEN, so
// no raw `usize` rides inside a `Cover` or the survivor list (the inward rule, applied to
// the selector's own data).
crate::refined! {
    /// A non-negative matrix position. Total: every `usize` is a valid index in the
    /// abstract (bounds are relative to a given matrix, checked where it is used).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Index(usize);
    fn new(i: usize) = Some(i);
}
impl Index {
    /// The raw offset — the sanctioned exit hatch for addressing the matrix.
    pub fn get(&self) -> usize {
        self.0
    }
}

/// A kill matrix: rows are relations, columns are mutants; cell `[r][m]` is true iff
/// relation `r` detects mutant `m`. `bool` is the one primitive the grammar exempts (a
/// detection is control, not domain data), so the grid composes without an extra citizen.
///
/// The honest reference implementation is deliberately NOT a column: a relation that flags
/// the correct implementation is unsound and is filtered out before it enters the matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillMatrix(Vec<Vec<bool>>);

impl KillMatrix {
    /// Parse-don't-validate: a kill matrix must be RECTANGULAR (every relation judged
    /// against the same mutant set). A ragged grid is rejected — `None` — rather than
    /// silently admitted, so a downstream out-of-bounds is unrepresentable.
    pub fn new(grid: Vec<Vec<bool>>) -> Option<Self> {
        let width = grid.first().map(Vec::len).unwrap_or(0);
        if grid.iter().all(|row| row.len() == width) {
            Some(KillMatrix(grid))
        } else {
            None
        }
    }

    /// The number of mutants (columns).
    pub fn mutants(&self) -> usize {
        self.0.first().map(Vec::len).unwrap_or(0)
    }

    /// The number of candidate relations (rows).
    pub fn relations(&self) -> usize {
        self.0.len()
    }

    /// A relation is NOISY if it kills every mutant: maximal recall, zero attribution.
    pub fn is_noisy(&self, relation: Index) -> bool {
        super::internal::is_noisy(self, relation)
    }

    /// Greedy set cover with the discrimination preference — the selected relations in pick
    /// order. Mutants no relation kills are skipped (and surfaced by `uncoverable`).
    pub fn select(&self) -> Cover {
        Cover(super::internal::select(self))
    }

    /// Mutants no relation kills — surviving mutants, each evidence that a relation or a
    /// degree of freedom is MISSING.
    pub fn uncoverable(&self) -> Cover {
        Cover(super::internal::uncoverable(self))
    }

    /// The raw grid — the sanctioned accessor the interior reads through.
    pub(crate) fn rows(&self) -> &[Vec<bool>] {
        &self.0
    }
}

crate::value_object!(KillMatrix);

/// A selected set of positions in pick order — the output of `select` (relation indices)
/// and of `uncoverable` (mutant indices). Both are ordered position lists, so one citizen
/// serves both.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cover(Vec<Index>);

impl Cover {
    /// The selected positions, in order.
    pub fn positions(&self) -> &[Index] {
        &self.0
    }

    /// Whether a given position is in the cover.
    pub fn contains(&self, position: Index) -> bool {
        self.0.contains(&position)
    }
}

crate::value_object!(Cover);
