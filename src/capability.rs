//!
//! capability — the behavioural side of the capability lattice: reconcile what an edge
//! DECLARES with what it DOES.
//!
//! ```text
//! effectful  ⊃  stateful  ⊃  lossy  ⊃  pure
//! ```
//!
//! Each step rightward (toward pure) adds a verification guarantee, so you want every edge
//! as far right as its behaviour allows. The TYPE level already carries the declaration
//! (`Morphism::Capability`) and can DEMAND a ceiling (`run_pure`), but a declaration is a
//! CLAIM the compiler trusts. This module verifies it: perturb a declared capability
//! SOURCE and observe whether the output (or residual) responds. Comparing the claim to
//! the response catches BOTH error directions —
//!
//!   - OVER-claiming (declared but never used) → capability-slop: needless guards and a
//!     false barrier to composition; the edge can move right; and
//!   - UNDER-claiming (used but undeclared) → a HIDDEN DEPENDENCY, the dangerous case: a
//!     "pure" edge that secretly reads state or the world, which the type system accepts.
//!
//! Crate-level tooling, exempt from the per-module boundary STRUCTURAL discipline (it audits
//! edges rather than being one), but SELF-HOSTED by verification: its example tests are
//! replaced with oracle-free property probes and it is kept in the mutation sweep. It is
//! demonstrated on the interpreter's `Resolve` family.

use crate::boundary::{Capability, Morphism, Perturbation};

/// Whether perturbing a source moved the output and/or the residual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Response {
    pub output_moved: bool,
    pub residual_moved: bool,
}

/// Perturb `x` along a source and report what moved — the capability lens over the same
/// forward/perturb machinery the residual probe uses.
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

/// The kind of source a perturbation targets — a DECLARED role (the probe cannot infer
/// whether a channel is data, carried state, or a world reading; it verifies the declared
/// dependency holds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    LostInput,
    State,
    World,
}

/// Whether a source is LIVE given what its perturbation moved.
fn is_live(source: Source, r: &Response) -> bool {
    match source {
        // a genuine lost dimension: the residual must keep what the output drops.
        Source::LostInput => !r.output_moved && r.residual_moved,
        // reads carried state, or depends on the world.
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

/// The capability a source contributes given what its perturbation moved. A source that
/// moved nothing is not live → it contributes `Pure` (it is slop).
pub fn source_capability(source: Source, r: &Response) -> Capability {
    if is_live(source, r) {
        cap_of(source)
    } else {
        Capability::Pure
    }
}

/// True when the edge was declared MORE capable than it behaves — it can move right
/// (drop the unused capability). Slop in the capability dimension.
pub fn over_declared(declared: Capability, actual: Capability) -> bool {
    actual.rank() < declared.rank()
}

// ===== claim vs behaviour: detect over- AND under-claiming ================

/// A capability CLAIM bound to an edge: the sources it declares it uses.
pub trait Declares {
    fn declared_sources(&self) -> &'static [Source];
}

/// One audited channel: a source, whether the edge CLAIMED it, and whether the behaviour
/// shows it LIVE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel {
    pub source: Source,
    pub claimed: bool,
    pub live: bool,
}

/// Audit one channel: observe the edge along the source's perturbation and record the
/// edge's own claim for that source.
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

/// A capability audit over an edge's candidate channels: it reconciles the edge's claim
/// with its probed behaviour.
pub struct Audit {
    channels: Vec<Channel>,
}
impl Audit {
    pub fn new(channels: Vec<Channel>) -> Self {
        Audit { channels }
    }

    /// Sources declared but not used — capability-slop. Move right by dropping them.
    pub fn over_claimed(&self) -> Vec<Source> {
        self.channels
            .iter()
            .filter(|c| c.claimed && !c.live)
            .map(|c| c.source)
            .collect()
    }

    /// Sources used but not declared — a HIDDEN DEPENDENCY. The dangerous case: a
    /// declared-pure edge that actually reads state or the world.
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

// ===== the interpreter's Resolve family, audited =========================
//
// `Resolve` and its two dishonest twins all share one channel — the carried `State` — and
// one perturbation (bump a binding). The audit over that single channel separates the
// honest edge from the over- and under-claiming ones.

use crate::interp::boundary::{Bound, Ident, Int, Resolve};

impl Declares for Resolve {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::State] // honest: it reads the carried environment.
    }
}

// The OVER-claiming subject is a TEST-ONLY counterexample exercising the audit's detection logic
// (it is not a production edge). The under-claiming twin is gone — v4 catches under-claiming
// structurally via the input-effect floor (see `boundary::InputEffect`), so the audit's remaining
// job is the over-claim a negative type fact can't express.
#[cfg(test)]
use crate::interp::boundary::ResolveIgnoresEnv;
#[cfg(test)]
impl Declares for ResolveIgnoresEnv {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::State] // OVER-claim: demands state it never reads.
    }
}

/// Perturb the carried STATE: bump the binding for a named variable. If the edge reads it,
/// the substituted output moves; if it ignores it, nothing moves.
pub struct BumpBinding(pub Ident);
crate::value_operator!(BumpBinding);
impl<M: Morphism<In = Bound>> Perturbation<M> for BumpBinding {
    fn perturb(&self, b: &Bound) -> Option<Bound> {
        let current = b.env().get(&self.0)?;
        let bumped = b
            .env()
            .clone()
            .bind(self.0.clone(), current.plus(Int::new(1)?));
        Some(Bound::new(bumped, b.expr().clone()))
    }
}

#[cfg(test)]
mod tests {
    //! Self-hosted: no example tests. The audit algebra (`is_live`, `cap_of`,
    //! `source_capability`, `over_declared`, and the `Audit` combinators) is certified by the
    //! ORACLE-FREE probes below — each recomputes the answer INDEPENDENTLY (the source rule,
    //! an exhaustive enumeration, the rank order, a re-derived join) and compares — and the
    //! `Resolve` family is audited over RANDOM bindings rather than one fixed example.
    //! `cargo mutants` (now in scope over this file) judges whether that suffices.
    use super::*;
    use crate::interp::boundary::{Env, Expr, Op};
    use proptest::prelude::*;

    fn name(s: &str) -> Ident {
        Ident::new(s).expect("valid identifier")
    }

    fn source() -> impl Strategy<Value = Source> {
        prop_oneof![
            Just(Source::LostInput),
            Just(Source::State),
            Just(Source::World)
        ]
    }
    fn capability() -> impl Strategy<Value = Capability> {
        prop_oneof![
            Just(Capability::Pure),
            Just(Capability::Lossy),
            Just(Capability::Stateful),
            Just(Capability::Effectful),
        ]
    }
    fn response() -> impl Strategy<Value = Response> {
        (any::<bool>(), any::<bool>()).prop_map(|(o, r)| Response {
            output_moved: o,
            residual_moved: r,
        })
    }
    fn channel() -> impl Strategy<Value = Channel> {
        (source(), any::<bool>(), any::<bool>()).prop_map(|(source, claimed, live)| Channel {
            source,
            claimed,
            live,
        })
    }

    proptest! {
        /// LIVENESS rule, restated independently: a lost input is live only when the residual
        /// moves and the output does not; a state/world source when either moves.
        #[test]
        fn is_live_matches_the_source_rule(s in source(), r in response()) {
            let expected = match s {
                Source::LostInput => !r.output_moved && r.residual_moved,
                Source::State | Source::World => r.output_moved || r.residual_moved,
            };
            prop_assert_eq!(is_live(s, &r), expected);
        }

        /// A source contributes `cap_of` when live, `Pure` otherwise — pins the gate.
        #[test]
        fn source_capability_gates_on_liveness(s in source(), r in response()) {
            let expected = if is_live(s, &r) { cap_of(s) } else { Capability::Pure };
            prop_assert_eq!(source_capability(s, &r), expected);
        }

        /// `over_declared` holds iff the actual capability ranks BELOW the declared one.
        #[test]
        fn over_declared_iff_actual_ranks_below_declared(d in capability(), a in capability()) {
            prop_assert_eq!(over_declared(d, a), a.rank() < d.rank());
        }

        /// The AUDIT reconciles claim with behaviour: over-claimed = claimed-not-live,
        /// under-claimed = live-not-claimed, honest = neither, and observed/declared are the
        /// joins over the live/claimed channels — each recomputed independently here.
        #[test]
        fn audit_reconciles_claim_and_behaviour(channels in prop::collection::vec(channel(), 0..6)) {
            let audit = Audit::new(channels.clone());
            let over: Vec<Source> = channels.iter().filter(|c| c.claimed && !c.live).map(|c| c.source).collect();
            let under: Vec<Source> = channels.iter().filter(|c| c.live && !c.claimed).map(|c| c.source).collect();
            prop_assert_eq!(audit.over_claimed(), over.clone());
            prop_assert_eq!(audit.under_claimed(), under.clone());
            prop_assert_eq!(audit.is_honest(), over.is_empty() && under.is_empty());

            let observed = channels.iter().filter(|c| c.live)
                .fold(Capability::Pure, |acc, c| acc.join(cap_of(c.source)));
            let declared = channels.iter().filter(|c| c.claimed)
                .fold(Capability::Pure, |acc, c| acc.join(cap_of(c.source)));
            prop_assert_eq!(audit.observed(), observed);
            prop_assert_eq!(audit.declared(), declared);
        }

        /// END-TO-END over random bindings: auditing the real `Resolve` family along the `State`
        /// channel separates the honest edge from its OVER-claiming twin — pinning `observe`,
        /// `audit_channel`, `BumpBinding`, and the `Declares` claims. (The under-claiming twin is
        /// gone: v4 catches under-claiming structurally via the input-effect floor, so the audit's
        /// end-to-end job here is the over-claim a negative type fact can't express. The
        /// claimed/live detection logic itself is pinned exhaustively by the per-channel property
        /// test above, over arbitrary `claimed`×`live` combinations.)
        #[test]
        fn the_resolve_family_audits_as_specified(v in 0i64..10_000) {
            let x = name("x");
            let env = Env::new().bind(x.clone(), Int::new(v).unwrap());
            let bound = Bound::new(env, Expr::bin(Op::Add, Expr::var(x.clone()), Expr::int(1).unwrap()));
            let one = |c| Audit::new(vec![c]);

            let honest = one(audit_channel(Source::State, &Resolve, &BumpBinding(x.clone()), &bound).unwrap());
            let over = one(audit_channel(Source::State, &ResolveIgnoresEnv, &BumpBinding(x.clone()), &bound).unwrap());

            prop_assert!(honest.is_honest());
            prop_assert_eq!(honest.observed(), Capability::Stateful);
            prop_assert_eq!(honest.declared(), Capability::Stateful);

            prop_assert_eq!(over.over_claimed(), vec![Source::State]);
            prop_assert!(over.under_claimed().is_empty());
            prop_assert_eq!(over.observed(), Capability::Pure);
            prop_assert!(over_declared(over.declared(), over.observed()));
        }
    }

    /// EXHAUSTIVE: the three sources map to three distinct capabilities — pins each `cap_of`
    /// arm against a constant-return mutant (a total enumeration of a finite domain).
    #[test]
    fn cap_of_distinguishes_the_three_sources() {
        assert_eq!(cap_of(Source::LostInput), Capability::Lossy);
        assert_eq!(cap_of(Source::State), Capability::Stateful);
        assert_eq!(cap_of(Source::World), Capability::Effectful);
    }
}
