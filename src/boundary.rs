//! boundary.rs — the GRAMMAR of module boundaries.
//!
//! A module's boundary is the ONLY surface it exposes to other modules, and it
//! may contain exactly three kinds of citizen:
//!
//!   1. VALUE OBJECTS — immutable, validated, value-equality data, built via smart constructors, never exposing mutable internals.
//!   2. TYPESTATES — (near) zero-data types encoding WHERE in a protocol a value sits, so illegal sequencing fails to compile.
//!   3. VALUE OPERATORS — pure morphisms over value objects (total or `-> Option`), with no I/O and no external mutation.
//!
//! This file defines that grammar once, for the whole crate:
//!
//!   - sealed marker traits `ValueObject` / `Typestate` / `ValueOperator`, so the
//!     set of boundary citizens is CLOSED — no module can invent a fourth kind,
//!     and no external crate can implement them; and
//!   - the generic `Morphism` abstraction `In -> (Out, Residual)` with `backward`
//!     reconstruction, the generic completeness `probe`, residual `Compose`-ition
//!     (loss as a `Pair` value object), and the retention typestate that makes
//!     "residual discarded => not invertible" a COMPILE error.
//!
//! Every per-module `boundary.rs` then re-exports only types carrying one of the
//! three markers; non-boundary logic lives in private sibling modules.

use core::fmt::Debug;
use core::marker::PhantomData;

/// The sealing module. `Sealed` is `pub(crate)`, so only THIS crate can satisfy
/// the marker supertraits — downstream crates cannot mint boundary citizens.
pub(crate) mod sealed {
    pub trait Sealed {}
}

// ===== the three boundary citizens =======================================

/// Marker: an immutable, validated, value-equality datum.
pub trait ValueObject: Clone + PartialEq + Debug + sealed::Sealed {}

/// Marker: a type encoding a position in a protocol (compile-time state).
pub trait Typestate: sealed::Sealed {}

/// Marker: a pure morphism-as-value over value objects. (Free pure functions are
/// value operators too; this marks the operator-as-object case.)
pub trait ValueOperator: sealed::Sealed {}

/// Declarative sugar so per-module `boundary.rs` files read like a grammar:
/// `value_object!(Cents, Account, Posting);`
#[macro_export]
macro_rules! value_object {
    ($($t:ty),+ $(,)?) => {$(
        impl $crate::boundary::sealed::Sealed for $t {}
        impl $crate::boundary::ValueObject for $t {}
    )+};
}

/// `typestate!(Retained, Discarded);`
#[macro_export]
macro_rules! typestate {
    ($($t:ty),+ $(,)?) => {$(
        impl $crate::boundary::sealed::Sealed for $t {}
        impl $crate::boundary::Typestate for $t {}
    )+};
}

/// `value_operator!(Aggregate, Round, Split);`
#[macro_export]
macro_rules! value_operator {
    ($($t:ty),+ $(,)?) => {$(
        impl $crate::boundary::sealed::Sealed for $t {}
        impl $crate::boundary::ValueOperator for $t {}
    )+};
}

// ===== generic morphism: In -> (Out, Residual) ===========================

/// The capability chain, least-power first: each step adds a capability and
/// subtracts a verification guarantee. A morphism declares its `CAPABILITY` as a
/// compile-time ceiling; `Compose` joins the ceilings of its stages, so a path's
/// capability is computed by the type system. (The behavioural audit in
/// `crate::capability` verifies a declared ceiling against actual behaviour.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Pure,
    Lossy,
    Stateful,
    Effectful,
}

impl Capability {
    /// Position on the chain (higher = more capable). `const` so it composes and
    /// can be asserted at compile time.
    pub const fn rank(self) -> u8 {
        match self {
            Capability::Pure => 0,
            Capability::Lossy => 1,
            Capability::Stateful => 2,
            Capability::Effectful => 3,
        }
    }

    /// The capability of a composition: as capable as the most-capable stage.
    pub const fn join(self, other: Capability) -> Capability {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}

/// A possibly-lossy morphism whose `Residual` value object witnesses EXACTLY
/// what the forward map collapsed. Retaining the residual restores
/// invertibility: `backward(forward(x)) == x`.
pub trait Morphism: ValueOperator {
    /// The declared capability ceiling — the furthest-LEFT class on the chain this
    /// operator claims (see `Capability`). The behavioural audit checks it; the
    /// type system composes it.
    const CAPABILITY: Capability;

    type In: ValueObject;
    type Out: ValueObject;
    type Residual: ValueObject;

    /// The lossy projection PLUS the typed witness of what it lost.
    fn forward(&self, input: &Self::In) -> (Self::Out, Self::Residual);

    /// Reconstruct the input from output + residual. Total iff the residual is
    /// COMPLETE; `None` when the residual cannot reconstruct a valid input.
    fn backward(&self, out: &Self::Out, residual: &Self::Residual) -> Option<Self::In>;
}

/// The empty residual: a LOSSLESS transport collapses nothing, so its witness of
/// loss is trivial. `Morphism<Residual = Unit>` marks a transport (an invertible
/// map with no discarded dimension) — the case where structural commutation is
/// vacuous and the quantitative probe becomes load-bearing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit;
crate::value_object!(Unit);

/// A perturbation is a partial value operator `In -> In` that nudges the input
/// along ONE dimension — used to probe whether a residual captures it.
pub trait Perturbation<M: Morphism>: ValueOperator {
    fn perturb(&self, input: &M::In) -> Option<M::In>;
}

/// Result of a residual-completeness probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    /// Output unchanged under perturbation — the loss really is in this dimension.
    pub output_invariant: bool,
    /// Residual changed — it captures the perturbed dimension.
    pub residual_responds: bool,
    /// Round-trip holds on the perturbed input — residual complete enough.
    pub round_trips: bool,
}
impl ProbeResult {
    pub fn residual_complete(&self) -> bool {
        self.output_invariant && self.residual_responds && self.round_trips
    }
}
crate::value_object!(ProbeResult);

/// Generic residual-completeness probe (a pure value operator).
///
/// Perturb the input along a dimension. A COMPLETE residual:
///   (1) leaves the OUTPUT invariant (the loss really is in that dimension),
///   (2) makes the RESIDUAL respond (it records the perturbed dimension), and
///   (3) still ROUND-TRIPS on the perturbed input (it is complete enough to
///       reconstruct).
/// An incomplete residual fails (2) and/or (3).
pub fn probe<M, P>(m: &M, p: &P, x: &M::In) -> Option<ProbeResult>
where
    M: Morphism,
    P: Perturbation<M>,
{
    let px = p.perturb(x)?;
    let (out_x, res_x) = m.forward(x);
    let (out_px, res_px) = m.forward(&px);
    Some(ProbeResult {
        output_invariant: out_x == out_px,
        residual_responds: res_x != res_px,
        round_trips: m
            .backward(&out_px, &res_px)
            .map(|r| r == px)
            .unwrap_or(false),
    })
}

// ===== layer 2: structural / variational probes (reference-FREE) ==========

/// A METAMORPHIC RELATION (structural / variational): an input perturbation
/// paired with the output transform it MUST induce. The morphism *commutes* with
/// the relation iff `forward(input_op(x)) == output_op(forward(x))`.
///
/// This is REFERENCE-FREE: it checks HOW the output varies under a known input
/// change without knowing the correct output value, so it works on a fully
/// opaque `forward`. Its blind spot is the *coefficient*: a uniform bug that
/// respects the relation (e.g. a halving respects scaling) survives — that is
/// what the quantitative probe below exists to catch.
pub trait Metamorphic<M: Morphism>: ValueOperator {
    /// Perturb the input along the relation's dimension.
    fn input_op(&self, x: &M::In) -> Option<M::In>;
    /// The output transform the perturbation must induce.
    fn output_op(&self, y: &M::Out) -> M::Out;
}

/// Structural commutation probe (reference-free): `Some(true)` iff the morphism
/// commutes with the relation at `x`. `None` if the perturbation does not apply.
pub fn commutes<M, R>(m: &M, r: &R, x: &M::In) -> Option<bool>
where
    M: Morphism,
    R: Metamorphic<M>,
{
    let perturbed = r.input_op(x)?;
    let (y, _) = m.forward(x);
    let (y_perturbed, _) = m.forward(&perturbed);
    Some(y_perturbed == r.output_op(&y))
}

// ===== layer 3: quantitative / coefficient probes (reference-BEARING) ======

/// A QUANTITATIVE / COEFFICIENT relation (reference-BEARING): apply a KNOWN unit
/// step to the input and require a KNOWN output delta — the forward map's actual
/// coefficient. This PINS values, catching a right-shape / wrong-constant bug
/// that every structural check is blind to.
///
/// It needs an external correctness criterion (a spec or an independent
/// reference) to supply `expected_delta` — without a referent "wrong coefficient"
/// is undefined. This is the absolute invariant, decomposed coefficient-by-
/// coefficient: it compares output *deltas* to reference *coefficients* instead
/// of comparing whole outputs to a reference.
pub trait Coefficient<M: Morphism>: ValueOperator {
    /// The output-delta value object (often `M::Out`, kept abstract).
    type Delta: ValueObject;
    /// Apply one known unit step to the input.
    fn unit_step(&self, x: &M::In) -> Option<M::In>;
    /// The output delta the step MUST produce (the reference coefficient).
    fn expected_delta(&self) -> Self::Delta;
    /// Observe the delta between two outputs.
    fn observed_delta(&self, before: &M::Out, after: &M::Out) -> Self::Delta;
}

/// Quantitative coefficient probe (reference-bearing): `Some(true)` iff the
/// observed unit-response equals the reference coefficient. `None` if the unit
/// step does not apply at `x`.
pub fn coefficient_holds<M, C>(m: &M, c: &C, x: &M::In) -> Option<bool>
where
    M: Morphism,
    C: Coefficient<M>,
{
    let stepped = c.unit_step(x)?;
    let (y, _) = m.forward(x);
    let (y_stepped, _) = m.forward(&stepped);
    Some(c.observed_delta(&y, &y_stepped) == c.expected_delta())
}

// ===== composition: loss composes as a value object ======================

/// Product of two residuals — loss COMPOSES as a value object.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<A, B>(pub A, pub B);
impl<A: ValueObject, B: ValueObject> sealed::Sealed for Pair<A, B> {}
impl<A: ValueObject, B: ValueObject> ValueObject for Pair<A, B> {}

/// Sequential composition `g ∘ f` as a single morphism. Its residual is the
/// PRODUCT of the two residuals; retaining it keeps the composite invertible.
/// End-to-end invertibility flows THROUGH a lossy stage as long as its residual
/// is kept — loss only blocks propagation when the residual is DISCARDED.
pub struct Compose<F, G> {
    pub f: F,
    pub g: G,
}
impl<F, G> sealed::Sealed for Compose<F, G> {}
impl<F, G> ValueOperator for Compose<F, G> {}
impl<F, G> Morphism for Compose<F, G>
where
    F: Morphism,
    G: Morphism<In = F::Out>,
{
    // the composite is as capable as its most-capable stage — the static join,
    // computed by the type system at compile time.
    const CAPABILITY: Capability = F::CAPABILITY.join(G::CAPABILITY);

    type In = F::In;
    type Out = G::Out;
    type Residual = Pair<F::Residual, G::Residual>;

    fn forward(&self, input: &Self::In) -> (Self::Out, Self::Residual) {
        let (mid, rf) = self.f.forward(input);
        let (out, rg) = self.g.forward(&mid);
        (out, Pair(rf, rg))
    }

    fn backward(&self, out: &Self::Out, r: &Self::Residual) -> Option<Self::In> {
        let mid = self.g.backward(out, &r.1)?;
        self.f.backward(&mid, &r.0)
    }
}

// ===== retention typestate: discarded residual => not invertible =========

/// Typestate: the residual is still carried (the result is invertible).
pub struct Retained;
/// Typestate: the residual has been dropped (invertibility is GONE).
pub struct Discarded;
crate::typestate!(Retained, Discarded);

/// The output of a morphism, INDEXED by whether its residual is retained.
///
/// `Carried<M, Retained>` can `invert`; `Carried<M, Discarded>` structurally
/// cannot — there is no such method, so discarding then inverting will not
/// compile. The typestate makes irreversibility a fact the compiler enforces.
pub struct Carried<M: Morphism, S: Typestate> {
    out: M::Out,
    residual: Option<M::Residual>,
    _state: PhantomData<S>,
}

impl<M: Morphism, S: Typestate> Carried<M, S> {
    /// The forward output is available regardless of retention state.
    pub fn out(&self) -> &M::Out {
        &self.out
    }
}

impl<M: Morphism> Carried<M, Retained> {
    fn new(out: M::Out, residual: M::Residual) -> Self {
        Carried {
            out,
            residual: Some(residual),
            _state: PhantomData,
        }
    }

    /// The retained residual.
    pub fn residual(&self) -> &M::Residual {
        self.residual
            .as_ref()
            .expect("Retained always carries a residual")
    }

    /// Reconstruct the input — only available while the residual is retained.
    pub fn invert(&self, m: &M) -> Option<M::In> {
        m.backward(&self.out, self.residual())
    }

    /// Irreversibly drop the residual, moving to the `Discarded` typestate.
    /// After this, `invert` is no longer in scope.
    pub fn discard(self) -> Carried<M, Discarded> {
        Carried {
            out: self.out,
            residual: None,
            _state: PhantomData,
        }
    }
}

/// Run a morphism and capture its output WITH its residual retained.
pub fn run<M: Morphism>(m: &M, input: &M::In) -> Carried<M, Retained> {
    let (out, residual) = m.forward(input);
    Carried::new(out, residual)
}
