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
//! Crate-level tooling, exempt from the per-module boundary discipline (it audits edges
//! rather than being one). It is demonstrated on the interpreter's `Resolve` family.

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

use crate::interp::boundary::{Bound, Ident, Int, Resolve, ResolveIgnoresEnv, ResolvePretendsPure};

impl Declares for Resolve {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::State] // honest: it reads the carried environment.
    }
}
impl Declares for ResolveIgnoresEnv {
    fn declared_sources(&self) -> &'static [Source] {
        &[Source::State] // OVER-claim: demands state it never reads.
    }
}
impl Declares for ResolvePretendsPure {
    fn declared_sources(&self) -> &'static [Source] {
        &[] // UNDER-claim: hides that it reads the environment.
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
    use super::*;
    use crate::interp::boundary::{Env, Expr, Op};

    fn name(s: &str) -> Ident {
        Ident::new(s).expect("valid identifier")
    }
    fn int(v: i64) -> Expr {
        Expr::int(v).expect("valid literal")
    }

    /// `(x + 1)` in an environment binding `x = 5`, audited along the `State` channel.
    fn audit_state<M: Morphism<In = Bound> + Declares>(m: &M) -> Audit {
        let x = name("x");
        let env = Env::new().bind(x.clone(), Int::new(5).unwrap());
        let bound = Bound::new(env, Expr::bin(Op::Add, Expr::var(x.clone()), int(1)));
        let channel = audit_channel(Source::State, m, &BumpBinding(x), &bound).expect("applies");
        Audit::new(vec![channel])
    }

    #[test]
    fn honest_resolve_claim_matches_behaviour() {
        let audit = audit_state(&Resolve);
        assert!(audit.is_honest());
        assert!(audit.over_claimed().is_empty());
        assert!(audit.under_claimed().is_empty());
        assert_eq!(audit.observed(), Capability::Stateful);
        assert_eq!(audit.declared(), Capability::Stateful);
    }

    /// OVER-claim: declares `State` but the perturbation moves nothing — slop.
    #[test]
    fn over_claim_is_flagged_as_slop() {
        let audit = audit_state(&ResolveIgnoresEnv);
        assert!(!audit.is_honest());
        assert_eq!(audit.over_claimed(), vec![Source::State]);
        assert!(audit.under_claimed().is_empty());
        // declared further left is possible: behaviour is pure, declaration stateful.
        assert!(over_declared(audit.declared(), audit.observed()));
        assert_eq!(audit.observed(), Capability::Pure);
    }

    /// UNDER-claim: declared `Pure` yet the output moves when state is perturbed — the
    /// hidden dependency the type system trusted and only the behavioural audit catches.
    #[test]
    fn under_claim_exposes_a_hidden_dependency() {
        let audit = audit_state(&ResolvePretendsPure);
        assert!(!audit.is_honest());
        assert_eq!(audit.under_claimed(), vec![Source::State]);
        assert!(audit.over_claimed().is_empty());
        assert_eq!(audit.declared(), Capability::Pure);
        assert_eq!(audit.observed(), Capability::Stateful);
    }
}
