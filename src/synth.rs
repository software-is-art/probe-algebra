//! synth — type-driven synthesis: read a value object's degrees of freedom off
//! its TYPE, find one the available checks cannot see, and synthesize the
//! reaching operator plus a DIMENSION-APPROPRIATE check for it.
//!
//! The spec's finding (`synth/`): the synthesized operator's job is *reaching*
//! the dimension; the check that fires there must itself be appropriate to that
//! dimension — structural equality for multiplicity, NOT totals. Reaching + an
//! appropriate check = coverage; reaching with the wrong check covers nothing.
//!
//! Worked here on the ledger's `Transaction` (a multiset of postings):
//!   - its DOFs, read off the type, are TOTALS and MULTIPLICITY;
//!   - aggregation's OUTPUT keeps totals but collapses multiplicity, so an
//!     output-only (totals) check is structurally blind to the multiplicity DOF;
//!   - the synthesized coverage is `Split` (reaches multiplicity) + the residual
//!     check (structural equality via the residual + round-trip).
//!
//! Crate-level tooling, like `select` — exempt from the boundary discipline.

use crate::boundary::{probe, Morphism};
use crate::ledger::boundary::{Aggregate, AggregateDropsAmounts, Split, Transaction};

/// The degrees of freedom of a `Transaction`, as read off its type. A multiset
/// of postings has per-account TOTALS and MULTIPLICITY (how each total splits
/// into the individual postings that produced it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dof {
    Totals,
    Multiplicity,
}

/// Enumerate the DOFs a `Transaction` exposes — the dimensions any check claiming
/// completeness must each be able to see.
pub fn transaction_dofs() -> Vec<Dof> {
    vec![Dof::Totals, Dof::Multiplicity]
}

/// Coverage of one DOF by two candidate checks: the naive OUTPUT-only check and
/// the synthesized RESIDUAL check, both probed through a reaching operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coverage {
    pub dof: Dof,
    /// The output-only check cannot see this DOF (the output is invariant under a
    /// perturbation along it) — it is the wrong instrument.
    pub output_check_blind: bool,
    /// The synthesized residual check DOES see it (the residual responds and the
    /// perturbed input still round-trips).
    pub residual_check_covers: bool,
}

/// Analyse the MULTIPLICITY DOF for aggregation, reached by the synthesized
/// `Split` operator. Reuses the generic `probe`: `output_invariant` is exactly
/// "the output-only check is blind to this dimension", while a responding
/// residual that still round-trips is the dimension-appropriate coverage.
pub fn multiplicity_coverage(x: &Transaction) -> Option<Coverage> {
    let pr = probe(&Aggregate, &Split, x)?;
    Some(Coverage {
        dof: Dof::Multiplicity,
        output_check_blind: pr.output_invariant,
        residual_check_covers: pr.residual_responds && pr.round_trips,
    })
}

/// The naive output-only check accepts a multiplicity bug: `AggregateDropsAmounts`
/// produces the SAME summary as the honest morphism, so comparing outputs reports
/// "fine". This is why the check must be synthesized to match the dimension.
pub fn output_only_accepts_multiplicity_bug(x: &Transaction) -> bool {
    let honest = Aggregate.forward(x).0;
    let buggy = AggregateDropsAmounts.forward(x).0;
    honest == buggy
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::boundary::{Account, Cents, Posting};

    fn sample() -> Transaction {
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(6000).unwrap()),
            Posting::new(Account::new("Cash").unwrap(), Cents::new(4000).unwrap()),
            Posting::new(
                Account::new("Revenue").unwrap(),
                Cents::new(-10000).unwrap(),
            ),
        ])
        .unwrap()
    }

    /// Multiplicity is a real DOF of the input type.
    #[test]
    fn multiplicity_is_a_degree_of_freedom() {
        assert!(transaction_dofs().contains(&Dof::Multiplicity));
    }

    /// The output-only check is blind to multiplicity; the synthesized residual
    /// check covers it. Reaching (Split) + dimension-appropriate check = coverage.
    #[test]
    fn synthesized_residual_check_covers_multiplicity() {
        let x = sample();
        let cov = multiplicity_coverage(&x).unwrap();
        assert_eq!(cov.dof, Dof::Multiplicity);
        assert!(
            cov.output_check_blind,
            "the summary is invariant under a multiplicity-only perturbation"
        );
        assert!(
            cov.residual_check_covers,
            "the residual responds and the perturbed input round-trips"
        );
    }

    /// The payoff: the wrong instrument (output-only) accepts the multiplicity
    /// bug, while the synthesized residual check catches it.
    #[test]
    fn wrong_instrument_is_fooled_where_synthesized_check_catches() {
        let x = sample();
        assert!(
            output_only_accepts_multiplicity_bug(&x),
            "comparing outputs cannot see a collapsed multiplicity"
        );
        assert!(
            !probe(&AggregateDropsAmounts, &Split, &x)
                .unwrap()
                .residual_complete(),
            "the synthesized residual check catches what the output check missed"
        );
    }
}
