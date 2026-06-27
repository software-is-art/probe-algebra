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

use crate::boundary::{Capability, Compose, Morphism, Perturbation};
use crate::effect::boundary::{IgnoresClock, SecretStamp, Stamp};
use crate::journal::boundary::Add;
use crate::ledger::boundary::{Aggregate, Round};

// `Capability` (the chain, with `rank`/`join`) now lives in the grammar
// (`crate::boundary`), because a morphism declares it as `Morphism::CAPABILITY`.
// This module keeps the behavioural side: probing sources and auditing claims.

/// COMPILE-TIME ceiling. The ledger pipeline (`Aggregate` then `Round`) is a lossy
/// transform that never reaches state or the world. Its statically composed
/// capability — `Compose` joins the stages' `CAPABILITY` at compile time — must
/// stay at most `Lossy`. Promote any stage to `Stateful`/`Effectful` and this
/// stops compiling: an exceeded ceiling is a BUILD error, not a test failure.
const _: () =
    assert!(<Compose<Aggregate, Round> as Morphism>::CAPABILITY.rank() <= Capability::Lossy.rank());

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

/// Whether a source is LIVE given what its perturbation moved.
fn is_live(source: Source, r: &Response) -> bool {
    match source {
        // a genuine lost dimension: the residual must keep what the output drops
        Source::LostInput => !r.output_moved && r.residual_moved,
        // reads/forgets carried state, or depends on the world
        Source::State | Source::World => r.output_moved || r.residual_moved,
    }
}

/// The capability a live source contributes.
fn cap_of(source: Source) -> Capability {
    match source {
        Source::LostInput => Capability::Lossy,
        Source::State => Capability::Stateful,
        Source::World => Capability::Effectful,
    }
}

/// The capability a source contributes given what its perturbation moved. A
/// source that moved nothing is not live → it contributes `Pure` (it is slop).
pub fn source_capability(source: Source, r: &Response) -> Capability {
    if is_live(source, r) {
        cap_of(source)
    } else {
        Capability::Pure
    }
}

/// True when the operator was declared MORE capable than it behaves — it can move
/// right (drop the unused capability). This is slop in the capability dimension.
pub fn over_declared(declared: Capability, actual: Capability) -> bool {
    actual.rank() < declared.rank()
}

// ===== claim vs behaviour: detect over- AND under-claiming ================

/// A capability CLAIM bound to an operator: the sources it declares it uses.
/// Comparing the claim to the probed behaviour catches BOTH error directions:
///   - over-claiming (declared but unused) → slop: needless guards and tests;
///   - under-claiming (used but undeclared) → a hidden dependency, a latent bug
///     (e.g. a "pure" call that secretly reads the world).
pub trait Declares {
    fn declared_sources(&self) -> &'static [Source];
}

impl Declares for Stamp {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::World] // honest: it reads the clock
    }
}
impl Declares for IgnoresClock {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::World] // OVER-claim: demands a clock it never uses
    }
}
impl Declares for SecretStamp {
    fn declared_sources(&self) -> &'static [Source] {
        &[] // UNDER-claim: hides that it reads the clock
    }
}
impl Declares for Add {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::State] // honest: it reads the carried state
    }
}

/// One audited channel: a source, whether the operator CLAIMED it, and whether the
/// behaviour shows it LIVE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel {
    pub source: Source,
    pub claimed: bool,
    pub live: bool,
}

/// Audit one channel: observe the operator along the source's perturbation and
/// record the operator's own claim for that source.
pub fn audit_channel<M, P>(source: Source, m: &M, p: &P, x: &M::In) -> Option<Channel>
where
    M: Morphism + Declares,
    P: Perturbation<M>,
{
    let r = observe(m, p, x)?;
    Some(Channel {
        source,
        claimed: m.declared_sources().contains(&source),
        live: is_live(source, &r),
    })
}

/// A capability audit over an operator's candidate channels: it reconciles the
/// operator's claim with its probed behaviour.
pub struct Audit {
    channels: Vec<Channel>,
}
impl Audit {
    pub fn new(channels: Vec<Channel>) -> Self {
        Audit { channels }
    }

    /// Sources declared but not used — capability-slop (over-testing, needless
    /// guards, false barriers to composition). Move right by dropping them.
    pub fn over_claimed(&self) -> Vec<Source> {
        self.channels
            .iter()
            .filter(|c| c.claimed && !c.live)
            .map(|c| c.source)
            .collect()
    }

    /// Sources used but not declared — a hidden dependency. The dangerous case: a
    /// declared-pure operator that actually reads state or the world.
    pub fn under_claimed(&self) -> Vec<Source> {
        self.channels
            .iter()
            .filter(|c| c.live && !c.claimed)
            .map(|c| c.source)
            .collect()
    }

    /// True iff the claim matches the behaviour exactly (no slop, no hidden deps).
    pub fn is_honest(&self) -> bool {
        self.over_claimed().is_empty() && self.under_claimed().is_empty()
    }

    /// The capability the behaviour actually exhibits.
    pub fn observed(&self) -> Capability {
        self.channels
            .iter()
            .filter(|c| c.live)
            .fold(Capability::Pure, |acc, c| acc.join(cap_of(c.source)))
    }

    /// The capability the declaration claims.
    pub fn declared(&self) -> Capability {
        self.channels
            .iter()
            .filter(|c| c.claimed)
            .fold(Capability::Pure, |acc, c| acc.join(cap_of(c.source)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boundary::{Compose, Pair};
    use crate::effect::boundary::{Clock, IgnoresClock, Message, NudgeReading, SecretStamp, Stamp};
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

    fn env() -> Pair<Message, Clock> {
        Pair(Message::new(5).unwrap(), Clock::new(100).unwrap())
    }

    /// An honest claim matches the behaviour: no over-claim, no under-claim.
    #[test]
    fn honest_claim_passes_the_audit() {
        let audit = Audit::new(vec![audit_channel(
            Source::World,
            &Stamp,
            &NudgeReading,
            &env(),
        )
        .unwrap()]);
        assert!(audit.is_honest());
        assert_eq!(audit.declared(), Capability::Effectful);
        assert_eq!(audit.observed(), Capability::Effectful);
    }

    /// OVER-claim: IgnoresClock declares the world but ignores it. Flagged as
    /// over-claimed (slop); declared sits above observed.
    #[test]
    fn over_claim_is_detected() {
        let audit = Audit::new(vec![audit_channel(
            Source::World,
            &IgnoresClock,
            &NudgeReading,
            &env(),
        )
        .unwrap()]);
        assert_eq!(audit.over_claimed(), vec![Source::World]);
        assert!(audit.under_claimed().is_empty());
        assert!(!audit.is_honest());
        assert_eq!(audit.declared(), Capability::Effectful);
        assert_eq!(audit.observed(), Capability::Pure);
    }

    /// UNDER-claim: SecretStamp reads the clock but declares nothing. Flagged as
    /// under-claimed (a hidden dependency); observed sits above declared.
    #[test]
    fn under_claim_is_detected() {
        let audit = Audit::new(vec![audit_channel(
            Source::World,
            &SecretStamp,
            &NudgeReading,
            &env(),
        )
        .unwrap()]);
        assert_eq!(audit.under_claimed(), vec![Source::World]);
        assert!(audit.over_claimed().is_empty());
        assert!(!audit.is_honest());
        assert_eq!(audit.declared(), Capability::Pure);
        assert_eq!(audit.observed(), Capability::Effectful);
        // the hidden dependency is real: SecretStamp behaves like Stamp and
        // round-trips (it is not a no-op that merely looks effectful).
        let (out, emission) = SecretStamp.forward(&env());
        assert_eq!(SecretStamp.backward(&out, &emission), Some(env()));
    }

    /// The grammar `CAPABILITY` const records each operator's declared ceiling,
    /// and `Compose` joins them at compile time — so a path's capability is a type
    /// fact. (The const and the per-source `Declares` are two views of the same
    /// claim; the audit is what checks either against behaviour.)
    #[test]
    fn grammar_ceiling_and_static_join() {
        assert_eq!(<Stamp as Morphism>::CAPABILITY, Capability::Effectful);
        assert_eq!(<Add as Morphism>::CAPABILITY, Capability::Stateful);
        assert_eq!(<Aggregate as Morphism>::CAPABILITY, Capability::Lossy);
        // SecretStamp's declared ceiling agrees with its (empty) source claim —
        // both say Pure — yet the audit shows both are wrong vs behaviour.
        assert_eq!(<SecretStamp as Morphism>::CAPABILITY, Capability::Pure);
        // the composite's capability is the join of its stages, computed statically
        assert_eq!(
            <Compose<Aggregate, Round> as Morphism>::CAPABILITY,
            Capability::Lossy
        );
    }

    /// The audit is not effect-specific: Add honestly declares and uses State.
    #[test]
    fn audit_generalizes_across_domains() {
        let audit = Audit::new(vec![audit_channel(
            Source::State,
            &Add::by(Register::new(5).unwrap()),
            &NudgeState,
            &Register::new(7).unwrap(),
        )
        .unwrap()]);
        assert!(audit.is_honest());
        assert_eq!(audit.observed(), Capability::Stateful);
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
