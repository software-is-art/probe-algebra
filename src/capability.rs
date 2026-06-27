//! capability — measure how far LEFT on the chain a morphism actually sits, and
//! detect when it is declared further left than it needs to be.
//!
//! ```text
//! effect  ⊃  state  ⊃  pure-with-loss  ⊃  pure
//! ```
//!
//! Each step adds a capability and subtracts a verification guarantee, so you
//! want every operator as far RIGHT (as close to pure) as its behaviour allows.
//! This module makes "as far right as behaviour allows" *measurable*: perturb each
//! declared capability SOURCE and observe whether the output or residual responds.
//! A declared source that the morphism ignores is capability-slop — the operator
//! can move right by dropping it.
//!
//! - LostInput  → the residual responds while the output is invariant (a genuine
//!   lost dimension): Lossy.
//! - State      → the output or residual responds to a perturbation of the carried
//!   state: Stateful.
//! - World      → the output or residual responds to a perturbation of a world
//!   reading: Effectful.
//!
//! Composition takes the JOIN: a path is as capable as its most-capable stage.
//!
//! Crate-level tooling (like `select`/`synth`), exempt from the boundary discipline.

use crate::boundary::{Morphism, Perturbation};

/// The capability chain, least-power first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Pure,
    Lossy,
    Stateful,
    Effectful,
}

impl Capability {
    fn rank(self) -> u8 {
        match self {
            Capability::Pure => 0,
            Capability::Lossy => 1,
            Capability::Stateful => 2,
            Capability::Effectful => 3,
        }
    }

    /// The class of a composition: as capable as the most-capable stage.
    pub fn join(self, other: Capability) -> Capability {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}

/// Whether perturbing a source moved the output and/or the residual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Response {
    pub output_moved: bool,
    pub residual_moved: bool,
}

/// Perturb `x` along a source and report what moved — the capability lens over the
/// same forward/perturb machinery the residual probe uses.
pub fn observe<M, P>(m: &M, p: &P, x: &M::In) -> Option<Response>
where
    M: Morphism,
    P: Perturbation<M>,
{
    let px = p.perturb(x)?;
    let (ox, rx) = m.forward(x);
    let (opx, rpx) = m.forward(&px);
    Some(Response {
        output_moved: ox != opx,
        residual_moved: rx != rpx,
    })
}

/// The kind of source a perturbation targets — a DECLARED role (the probe cannot
/// infer whether an input channel is data, carried state, or a world reading; it
/// can only verify the declared dependency holds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    LostInput,
    State,
    World,
}

/// The capability a source contributes given what its perturbation moved. A
/// source that moved nothing is not live → it contributes `Pure` (it is slop).
pub fn source_capability(source: Source, r: &Response) -> Capability {
    let live = match source {
        // a genuine lost dimension: the residual must keep what the output drops
        Source::LostInput => !r.output_moved && r.residual_moved,
        // reads/forgets carried state, or depends on the world
        Source::State | Source::World => r.output_moved || r.residual_moved,
    };
    if !live {
        return Capability::Pure;
    }
    match source {
        Source::LostInput => Capability::Lossy,
        Source::State => Capability::Stateful,
        Source::World => Capability::Effectful,
    }
}

/// True when the operator was declared MORE capable than it behaves — it can move
/// right (drop the unused capability). This is slop in the capability dimension.
pub fn over_declared(declared: Capability, actual: Capability) -> bool {
    actual.rank() < declared.rank()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{Compose, Pair};
    use crate::effect::boundary::{Clock, IgnoresClock, Message, NudgeReading, Stamp};
    use crate::journal::boundary::{Add, NudgeState, Register};
    use crate::ledger::boundary::{Account, Aggregate, Cents, Posting, Round, Split, Transaction};

    fn tx() -> Transaction {
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(6000).unwrap()),
            Posting::new(Account::new("Cash").unwrap(), Cents::new(4000).unwrap()),
        ])
        .unwrap()
    }

    /// A pure transformation: input moves the output, no residual. No source is
    /// live → Pure.
    #[test]
    fn pure_when_no_source_is_live() {
        let pure_like = Response {
            output_moved: true,
            residual_moved: false,
        };
        assert_eq!(
            source_capability(Source::LostInput, &pure_like),
            Capability::Pure
        );
    }

    /// A source where BOTH output and residual move is not a clean lost dimension
    /// (the output is not invariant), so it does not count as Lossy.
    #[test]
    fn both_moving_is_not_a_clean_lost_dimension() {
        let both = Response {
            output_moved: true,
            residual_moved: true,
        };
        assert_eq!(
            source_capability(Source::LostInput, &both),
            Capability::Pure
        );
    }

    /// Not over-declared when the declared ceiling matches or sits below the
    /// observed behaviour.
    #[test]
    fn matched_or_higher_behaviour_is_not_over_declared() {
        assert!(!over_declared(Capability::Lossy, Capability::Lossy));
        assert!(!over_declared(Capability::Pure, Capability::Effectful));
    }

    /// Aggregation: perturbing the lost dimension leaves the output invariant and
    /// moves the residual → Lossy.
    #[test]
    fn aggregate_is_lossy() {
        let r = observe(&Aggregate, &Split, &tx()).unwrap();
        assert_eq!(source_capability(Source::LostInput, &r), Capability::Lossy);
    }

    /// Add reads the carried state (its output moves with the prior) → Stateful.
    #[test]
    fn add_is_stateful() {
        let r = observe(
            &Add::by(Register::new(5).unwrap()),
            &NudgeState,
            &Register::new(7).unwrap(),
        )
        .unwrap();
        assert!(r.output_moved, "output depends on the carried state");
        assert_eq!(source_capability(Source::State, &r), Capability::Stateful);
    }

    /// Stamp's output depends on the world reading → Effectful.
    #[test]
    fn stamp_is_effectful() {
        let env = Pair(Message::new(5).unwrap(), Clock::new(100).unwrap());
        let r = observe(&Stamp, &NudgeReading, &env).unwrap();
        assert!(r.output_moved, "output depends on the world reading");
        assert_eq!(source_capability(Source::World, &r), Capability::Effectful);
    }

    /// IgnoresClock DEMANDS a clock but ignores it: declared Effectful, behaves
    /// Pure → the probe flags the over-declaration.
    #[test]
    fn over_declared_effect_is_detected() {
        let env = Pair(Message::new(5).unwrap(), Clock::new(100).unwrap());
        let r = observe(&IgnoresClock, &NudgeReading, &env).unwrap();
        let actual = source_capability(Source::World, &r);
        assert_eq!(actual, Capability::Pure, "the clock is never used");
        assert!(
            over_declared(Capability::Effectful, actual),
            "it can move right: drop the clock demand"
        );
    }

    /// Composition takes the join: a two-stage lossy path is Lossy, and probing
    /// the composite confirms it.
    #[test]
    fn composition_takes_the_join() {
        assert_eq!(Capability::Pure.join(Capability::Lossy), Capability::Lossy);
        assert_eq!(
            Capability::Lossy.join(Capability::Effectful),
            Capability::Effectful
        );
        assert_eq!(
            Capability::Stateful.join(Capability::Lossy),
            Capability::Stateful
        );

        // the ledger composite Aggregate then Round, probed, is Lossy = join(Lossy, Lossy)
        let composite = Compose {
            f: Aggregate,
            g: Round,
        };
        let r = observe(&composite, &Split, &tx()).unwrap();
        assert_eq!(
            source_capability(Source::LostInput, &r),
            Capability::Lossy.join(Capability::Lossy)
        );
    }
}
