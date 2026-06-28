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

use crate::boundary::{
    probe, require_complete, Covers, DofCons, DofNil, DofProbe, HasDofs, Morphism,
};
use crate::ledger::boundary::{
    AccountSummary, Aggregate, AggregateDropsAmounts, NudgePosting, Split, Transaction,
};

/// The degrees of freedom of a `Transaction`, as read off its type. A multiset
/// of postings has per-account TOTALS and MULTIPLICITY (how each total splits
/// into the individual postings that produced it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dof {
    Totals,
    Multiplicity,
}

// ===== the DOFs at the TYPE level: the set is declared, not enumerated =====
//
// Each DOF is a marker type reflecting to the runtime `Dof` (the same reflect pattern
// the grammar uses for capability and provenance). A value object declares its DOFs as
// a type-level set via `HasDofs`; `transaction_dofs()` is then DERIVED from that set,
// and a probe's completeness is the compile-time `require_complete` bound.

/// A type-level degree of freedom, reflecting to the runtime `Dof`.
pub trait DegreeOfFreedom {
    /// The runtime tag this marker reflects to.
    const DOF: Dof;
}
/// The per-account totals dimension.
pub struct Totals;
/// The multiplicity dimension (how each total splits into individual postings).
pub struct Multiplicity;
impl DegreeOfFreedom for Totals {
    const DOF: Dof = Dof::Totals;
}
impl DegreeOfFreedom for Multiplicity {
    const DOF: Dof = Dof::Multiplicity;
}

/// Reflect a type-level DOF set to the runtime `Vec<Dof>` (declaration order). The
/// local twin of the grammar's `Lineage::reflect`, defined here because the runtime
/// `Dof` enum is domain-specific tooling, not grammar.
pub trait ReflectDofs {
    fn reflect(out: &mut Vec<Dof>);
}
impl ReflectDofs for DofNil {
    fn reflect(_out: &mut Vec<Dof>) {}
}
impl<H: DegreeOfFreedom, T: ReflectDofs> ReflectDofs for DofCons<H, T> {
    fn reflect(out: &mut Vec<Dof>) {
        out.push(H::DOF);
        T::reflect(out);
    }
}

/// A `Transaction`'s degrees of freedom, declared at the TYPE level: TOTALS then
/// MULTIPLICITY. This is the single source of truth `transaction_dofs()` reflects and
/// `require_complete` demands coverage of.
impl HasDofs for Transaction {
    type Dofs = DofCons<Totals, DofCons<Multiplicity, DofNil>>;
}

// Each DOF supplies the perturbation that reaches its dimension, so the completeness
// suite is SYNTHESIZED from the type-level set by `probe_declared_dofs` — generic over
// any aggregation-shaped morphism, so it probes the honest and buggy edges alike.
impl<M: Morphism<In = Transaction, Out = AccountSummary>> DofProbe<M> for Totals {
    // Totals survive into the OUTPUT, so nudging an amount makes the summary respond.
    type Perturb = NudgePosting;
    fn perturbation() -> NudgePosting {
        NudgePosting
    }
}
impl<M: Morphism<In = Transaction, Out = AccountSummary>> DofProbe<M> for Multiplicity {
    // Multiplicity is COLLAPSED into the residual, so `Split` reaches it there.
    type Perturb = Split;
    fn perturbation() -> Split {
        Split
    }
}

/// Enumerate the DOFs a `Transaction` exposes — the dimensions any check claiming
/// completeness must each be able to see. DERIVED from the type-level `HasDofs`
/// declaration, so the runtime list cannot drift from the static obligation.
pub fn transaction_dofs() -> Vec<Dof> {
    let mut v = Vec::new();
    <<Transaction as HasDofs>::Dofs as ReflectDofs>::reflect(&mut v);
    v
}

// ===== coverage as a COMPILE-TIME obligation ===============================
//
// The two candidate instruments as type-level checks. Their `Covers` impls are the
// static statement of what the runtime probes below DEMONSTRATE: the output-only check
// sees only totals (blind to multiplicity), while the synthesized residual check (Split
// + the residual round-trip) reaches and covers BOTH. `require_complete` then accepts
// only a check whose `Covers` set spans every DOF `Transaction` declares.

/// The naive instrument: compare the aggregation OUTPUT. It sees per-account totals but
/// is structurally blind to multiplicity (the summary is invariant under a
/// multiplicity-only perturbation) — so it covers only `Totals`.
pub struct OutputOnlyCheck;
impl Covers<Totals> for OutputOnlyCheck {}

/// The synthesized instrument: reach multiplicity with `Split` and fire the residual
/// round-trip there. It covers BOTH the totals and the multiplicity dimension.
pub struct ResidualCheck;
impl Covers<Totals> for ResidualCheck {}
impl Covers<Multiplicity> for ResidualCheck {}

/// Statically assert the synthesized residual check is a COMPLETE probe of a
/// `Transaction`: it must `Covers` every DOF the type declares, or this fails to
/// compile. The compile-time twin of the runtime coverage tests below.
pub fn assert_residual_check_is_complete() {
    require_complete::<Transaction, _>(&ResidualCheck);
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

/// Analyse the MULTIPLICITY DOF for a morphism `m`, reached by the synthesized
/// `Split` operator. Reuses the generic `probe`: `output_invariant` is exactly
/// "the output-only check is blind to this dimension", while coverage requires
/// the residual to BOTH respond AND round-trip — a responding-but-incomplete
/// residual (e.g. the count-only bug) does not actually cover the dimension.
pub fn multiplicity_coverage<M>(m: &M, x: &Transaction) -> Option<Coverage>
where
    M: Morphism<In = Transaction, Out = crate::ledger::boundary::AccountSummary>,
{
    let pr = probe(m, &Split, x)?;
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

    /// The runtime DOF list is DERIVED from the type-level `HasDofs` declaration, so it
    /// cannot drift from the static obligation: declaration order, both dimensions.
    #[test]
    fn dofs_are_reflected_from_the_type_level_set() {
        assert_eq!(transaction_dofs(), vec![Dof::Totals, Dof::Multiplicity]);
    }

    /// SYNTHESIS: the completeness suite is generated from the type-level DOF set. For
    /// the honest aggregation, EVERY declared DOF is observable — totals through the
    /// output, multiplicity through the residual — so all verdicts are `Some(true)`,
    /// with no per-dimension test code.
    #[test]
    fn synthesized_suite_covers_every_declared_dof() {
        use crate::boundary::probe_declared_dofs;
        let x = sample();
        let verdicts = probe_declared_dofs::<Transaction, _>(&Aggregate, &x);
        assert_eq!(verdicts, vec![Some(true), Some(true)]);
    }

    /// SYNTHESIS catches a silently-dropped dimension: `AggregateDropsAmounts` keeps
    /// totals in the output but collapses multiplicity with no complete residual, so the
    /// generated suite flags the MULTIPLICITY DOF as uncovered — a bug found with no
    /// hand-written probe for that edge.
    #[test]
    fn synthesized_suite_flags_a_dropped_dof() {
        use crate::boundary::probe_declared_dofs;
        let x = sample();
        let verdicts = probe_declared_dofs::<Transaction, _>(&AggregateDropsAmounts, &x);
        assert_eq!(
            verdicts,
            vec![Some(true), Some(false)],
            "totals survive in the output; multiplicity is silently dropped"
        );
    }

    /// Completeness is a COMPILE-TIME fact: `assert_residual_check_is_complete` only
    /// type-checks because `ResidualCheck` `Covers` every DOF `Transaction` declares.
    /// An instrument that omits a dimension (the output-only check) is rejected by the
    /// same bound — pinned in `tests/compile_fail/incomplete_probe_rejected`.
    #[test]
    fn residual_check_is_statically_complete() {
        assert_residual_check_is_complete();
    }

    /// The output-only check is blind to multiplicity; the synthesized residual
    /// check covers it. Reaching (Split) + dimension-appropriate check = coverage.
    #[test]
    fn synthesized_residual_check_covers_multiplicity() {
        let x = sample();
        let cov = multiplicity_coverage(&Aggregate, &x).unwrap();
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

    /// A responding-but-incomplete residual does NOT cover the dimension: the
    /// count-only bug's residual responds to Split but cannot round-trip, so
    /// coverage requires BOTH (responds AND round-trips), not either.
    #[test]
    fn responding_but_incomplete_residual_does_not_cover() {
        let x = sample();
        let cov = multiplicity_coverage(&AggregateDropsAmounts, &x).unwrap();
        assert!(cov.output_check_blind);
        assert!(
            !cov.residual_check_covers,
            "a residual that responds but cannot round-trip does not cover the DOF"
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
