//! boundary.rs — the GRAMMAR of module boundaries.
//!
//! A module's boundary is the ONLY surface it exposes, and it is a CATEGORY: just
//! two things cross the seam —
//!
//!   - OBJECTS — the value objects (the nouns): immutable, validated, value-equality
//!     data, never exposing mutable internals. The raw primitives are the one set of
//!     objects OUTSIDE the domain — the source every construction flows out of.
//!   - MORPHISMS — the value operators (the verbs): pure maps with no I/O and no
//!     external mutation. They come in a few type-distinguished SHAPES that share one
//!     algebra (residual, backward reconstruction, probe): `Morphism` (a total edge
//!     between value objects), `Construction` (the PARTIAL entry edge from a raw
//!     primitive into a value object — "parse, don't validate"), and transitions
//!     (a `Morphism` declared by `transition!`).
//!
//! A TYPESTATE is not a third citizen but an INDEX that distinguishes objects:
//! `Entry<Draft>` and `Entry<Submitted>` are two objects of the category — same data,
//! different edges. This is why construction, long left as a native `fn new` OUTSIDE
//! the algebra, is just another morphism: the edge INTO the domain, now probeable
//! like every other.
//!
//! This file defines that grammar once, for the whole crate:
//!
//!   - sealed marker traits `ValueObject` / `Typestate` / `ValueOperator`, so the set
//!     of citizens is CLOSED — no module can invent a new kind, and no external crate
//!     can implement them; and
//!   - the generic `Morphism` / `Construction` algebra `… -> (Out, Residual)` with
//!     `backward` / `reconstruct`, the generic completeness probes (`probe`,
//!     `reconstructs`), residual `Compose`-ition / `Then`-composition (loss as a
//!     `Pair` value object), and the retention typestate that makes "residual
//!     discarded => not invertible" a COMPILE error.
//!
//! Every per-module `boundary.rs` then defines only types carrying one of the
//! markers; non-boundary logic lives in private sibling modules.

use core::fmt::Debug;
use core::marker::PhantomData;

/// The sealing module. `Sealed` is `pub(crate)`, so only THIS crate can satisfy
/// the marker supertraits — downstream crates cannot mint boundary citizens.
pub(crate) mod sealed {
    pub trait Sealed {}
}

// ===== the boundary citizens: objects and morphisms ======================

/// Marker: an OBJECT of the category — an immutable, validated, value-equality datum.
pub trait ValueObject: Clone + PartialEq + Debug + sealed::Sealed {}

/// Marker: an INDEX that distinguishes objects (a compile-time protocol position), so
/// illegal sequencing fails to compile. Not an object itself — `Entry<Draft>` is the
/// object; `Draft` is the index.
pub trait Typestate: sealed::Sealed {}

/// Marker: a MORPHISM of the category — a pure operator-as-value over value objects
/// (`Morphism`, `Construction`, transitions). (Free pure functions are morphisms too;
/// this marks the operator-as-object case.)
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

/// A NAME-BRANDED PROOF TOKEN realized as a value object: zero data, branded by a
/// phantom `N`, so two tokens of the same name are the SAME fact and a token for name
/// A cannot stand in for B. Its field is private to the defining module, so it is
/// minted only there (a GDP "ghost" — `Cleared<N>` / `Flagged<N>`). This lifts the
/// ~17 lines those hand-rolled each into one line: `proof_token!(/// doc... Cleared);`
#[macro_export]
macro_rules! proof_token {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        pub struct $name<N>(::core::marker::PhantomData<N>);
        impl<N> ::core::clone::Clone for $name<N> {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl<N> ::core::marker::Copy for $name<N> {}
        impl<N> ::core::cmp::PartialEq for $name<N> {
            fn eq(&self, _other: &Self) -> bool {
                true // a token of a given name carries no distinguishing data.
            }
        }
        impl<N> ::core::fmt::Debug for $name<N> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(stringify!($name))
            }
        }
        impl<N> $crate::boundary::sealed::Sealed for $name<N> {}
        impl<N> $crate::boundary::ValueObject for $name<N> {}
    };
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

// ===== tagged primitives: one operator set per KIND, not per type ========

/// A primitive (`i64`) tagged with a zero-size KIND `U`. Two domain concepts over
/// the same primitive do NOT unify (`Qty<A>` != `Qty<B>`), yet they share ONE
/// generic operator set defined here — so a new concept costs a tag plus a
/// validity rule, not a hand-written operator set. The low-boilerplate form of
/// "every primitive is a value object with operators": a kind's operators are
/// written once, and a named-but-unvalidated concept gets all of them for free.
pub struct Qty<U>(i64, PhantomData<U>);

// Manual impls (no `U: Trait` bounds) so any zero-size tag works as a kind without
// deriving anything on it.
impl<U> Clone for Qty<U> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U> Copy for Qty<U> {}
impl<U> PartialEq for Qty<U> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<U> Eq for Qty<U> {}
impl<U> PartialOrd for Qty<U> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<U> Ord for Qty<U> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl<U> Debug for Qty<U> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Qty({})", self.0)
    }
}
impl<U> sealed::Sealed for Qty<U> {}
impl<U> ValueObject for Qty<U> {}

/// The KIND of a tagged primitive: which raw values it admits (its partition).
pub trait Kind {
    fn admits(raw: i64) -> bool;
}

/// A kind whose valid set is the WHOLE primitive domain — so its arithmetic is
/// total (no `Option`). Most named-but-unvalidated domain concepts are `Total`.
pub trait Total: Kind {}

impl<U: Kind> Qty<U> {
    /// Parse-don't-validate: `None` outside the kind's valid set.
    pub fn new(raw: i64) -> Option<Self> {
        if U::admits(raw) {
            Some(Qty(raw, PhantomData))
        } else {
            None
        }
    }
    /// The sanctioned exit hatch back to the primitive.
    pub fn get(self) -> i64 {
        self.0
    }
    /// Range-checked addition (partial — stays within the kind's valid set).
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Self::new(self.0 + other.0)
    }
    /// Range-checked subtraction (partial).
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        Self::new(self.0 - other.0)
    }
}

impl<U: Total> Qty<U> {
    /// The additive identity (a total kind admits 0).
    pub fn zero() -> Self {
        Qty(0, PhantomData)
    }
    /// Total addition.
    pub fn plus(self, other: Self) -> Self {
        Qty(self.0 + other.0, PhantomData)
    }
    /// Total subtraction.
    pub fn minus(self, other: Self) -> Self {
        Qty(self.0 - other.0, PhantomData)
    }
    /// Total negation.
    pub fn negate(self) -> Self {
        Qty(-self.0, PhantomData)
    }
}

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

/// How a retention state STORES the residual: `Retained` keeps it, `Discarded`
/// drops it. This makes "Retained always carries a residual" a STRUCTURAL fact —
/// there is no `Option` and no runtime `expect` re-asserting an invariant the
/// typestate is meant to guarantee. (The typestate as its own proof, not a
/// runtime check — the whole point of the project, applied to itself.)
pub trait Retention: Typestate {
    type Carry<R>;
}
impl Retention for Retained {
    type Carry<R> = R;
}
impl Retention for Discarded {
    type Carry<R> = ();
}

/// The output of a morphism, INDEXED by whether its residual is retained.
///
/// `Carried<M, Retained>` can `invert`; `Carried<M, Discarded>` structurally
/// cannot — there is no such method, so discarding then inverting will not
/// compile. The retention state also decides whether the residual is even stored
/// (`Retained` keeps `M::Residual`; `Discarded` keeps `()`).
pub struct Carried<M: Morphism, S: Retention> {
    out: M::Out,
    residual: S::Carry<M::Residual>,
    _state: PhantomData<S>,
}

impl<M: Morphism, S: Retention> Carried<M, S> {
    /// The forward output is available regardless of retention state.
    pub fn out(&self) -> &M::Out {
        &self.out
    }
}

impl<M: Morphism> Carried<M, Retained> {
    fn new(out: M::Out, residual: M::Residual) -> Self {
        Carried {
            out,
            residual,
            _state: PhantomData,
        }
    }

    /// The retained residual — structurally present, so no runtime check.
    pub fn residual(&self) -> &M::Residual {
        &self.residual
    }

    /// Reconstruct the input — only available while the residual is retained.
    pub fn invert(&self, m: &M) -> Option<M::In> {
        m.backward(&self.out, &self.residual)
    }

    /// Irreversibly drop the residual, moving to the `Discarded` typestate.
    /// After this, `invert` is no longer in scope.
    pub fn discard(self) -> Carried<M, Discarded> {
        Carried {
            out: self.out,
            residual: (),
            _state: PhantomData,
        }
    }
}

/// Run a morphism and capture its output WITH its residual retained.
pub fn run<M: Morphism>(m: &M, input: &M::In) -> Carried<M, Retained> {
    let (out, residual) = m.forward(input);
    Carried::new(out, residual)
}

// ===== constructions: the ENTRY edge into the value-object world =========

/// A CONSTRUCTION: the entry edge of the value-object category — the morphism FROM
/// a raw primitive INTO a value object ("parse, don't validate"). It is the sibling
/// of `Morphism`, sharing its residual algebra, and differs only in the two ways the
/// boundary makes special:
///
///   - its source `Raw` is OUTSIDE the domain, so it carries NO `ValueObject` bound —
///     it is the one state that is not a citizen (a bare `i64`, `String`, `Vec<_>`); and
///   - it is PARTIAL: a parse either ADMITS the raw (yielding the refined value plus
///     the residual it normalized away) or REJECTS it (`None`) — the branching shape.
///
/// Keeping the `Residual` is what brings construction INTO the probe space and keeps
/// it HONEST. A pure refinement (a range check) loses nothing, so its residual is
/// `Unit`; a NORMALIZING parse (trimming, sorting) discards a real dimension, so its
/// residual must witness exactly that dimension and `reconstruct` recovers the
/// original raw. A constructor that silently normalizes therefore CANNOT claim a
/// `Unit` residual: the `reconstructs` round-trip catches it, the same way `probe`
/// catches an incomplete `Morphism` residual. This is why the value-object pattern's
/// smart constructor was the one edge outside the testing space — modelled as a
/// construction, it is back inside it.
pub trait Construction: ValueOperator {
    /// The declared capability ceiling, exactly as on `Morphism` — a pure refinement
    /// (range check, `Unit` residual) is `Pure`; a NORMALIZING parse (trimming,
    /// sorting) collapses a dimension and is `Lossy`. `Then` joins it with the
    /// morphism's, so a primitive-to-output path's capability is computed by the type
    /// system just like a `Compose` chain's.
    const CAPABILITY: Capability;

    /// The raw input — a primitive, NOT a value object: the only state outside the
    /// domain. Bounded just enough to probe the round-trip (equality + diagnostics).
    type Raw: Clone + PartialEq + Debug;
    /// The validated value object the parse produces.
    type Refined: ValueObject;
    /// What normalization discarded: `Unit` for a pure refinement, a real witness for
    /// a normalizing parse.
    type Residual: ValueObject;

    /// Parse the raw input: `Some((refined, residual))` if admitted, `None` if rejected.
    fn parse(&self, raw: &Self::Raw) -> Option<(Self::Refined, Self::Residual)>;

    /// Reconstruct the raw input from the refined value plus the residual. It mirrors
    /// `Morphism::backward` (an `Option`, so a construction composed onto a lossy
    /// morphism can thread that morphism's `backward` through). For an admitted `x`, a
    /// COMPLETE residual gives `reconstruct(parse(x)) == Some(x)`.
    fn reconstruct(&self, refined: &Self::Refined, residual: &Self::Residual) -> Option<Self::Raw>;
}

/// Construction round-trip probe (the entry-edge analog of `probe`): a parse that
/// ADMITS `raw` must reconstruct it EXACTLY from the refined value plus the residual.
/// `Some(true)` iff it does, `Some(false)` if the residual is too lossy to recover the
/// raw, `None` if the raw is rejected (no obligation). It catches a constructor that
/// normalizes without a complete residual — including one that claims a `Unit`
/// residual but silently drops a dimension.
pub fn reconstructs<C: Construction>(c: &C, raw: &C::Raw) -> Option<bool> {
    let (refined, residual) = c.parse(raw)?;
    Some(c.reconstruct(&refined, &residual).as_ref() == Some(raw))
}

/// A RAW perturbation: nudges a construction's raw input along one dimension — the
/// entry-edge analog of `Perturbation`, used to probe whether the parse's residual
/// captures that dimension.
pub trait RawPerturbation<C: Construction>: ValueOperator {
    fn perturb(&self, raw: &C::Raw) -> Option<C::Raw>;
}

/// Construction COMPLETENESS probe — the entry-edge analog of `probe`, and the
/// upgrade over the bare `reconstructs` round-trip. Perturb the raw along a dimension
/// the parse NORMALIZES away; a COMPLETE residual then:
///   (1) leaves the REFINED value invariant (the parse really does normalize it),
///   (2) makes the RESIDUAL respond (it records the perturbed dimension), and
///   (3) still ROUND-TRIPS on the perturbed raw.
/// A `Unit`-residual parse that secretly normalizes fails (2) and (3) — exactly how
/// `probe` catches an incomplete `Morphism` residual.
pub fn construction_probe<C, P>(c: &C, p: &P, raw: &C::Raw) -> Option<ProbeResult>
where
    C: Construction,
    P: RawPerturbation<C>,
{
    let praw = p.perturb(raw)?;
    let (refined_x, res_x) = c.parse(raw)?;
    let (refined_px, res_px) = c.parse(&praw)?;
    Some(ProbeResult {
        output_invariant: refined_x == refined_px,
        residual_responds: res_x != res_px,
        round_trips: c.reconstruct(&refined_px, &res_px).as_ref() == Some(&praw),
    })
}

/// Sequential composition of a CONSTRUCTION with a `Morphism` — the proof that
/// construction lives in the SAME category as every other edge. A path FROM a raw
/// primitive, THROUGH the parse, THROUGH a value-object morphism, is itself one
/// construction: its `Raw` is the parse's, its `Refined` is the morphism's output, and
/// its residual is the PRODUCT of the two residuals, so the whole primitive-to-output
/// path stays reconstructible. (The entry-edge twin of `Compose`.)
pub struct Then<C, M> {
    pub construct: C,
    pub then: M,
}
impl<C, M> sealed::Sealed for Then<C, M> {}
impl<C, M> ValueOperator for Then<C, M> {}
impl<C, M> Construction for Then<C, M>
where
    C: Construction,
    M: Morphism<In = C::Refined>,
{
    // the path is as capable as its most-capable edge — the static join, exactly as
    // `Compose` does for two morphisms.
    const CAPABILITY: Capability = C::CAPABILITY.join(M::CAPABILITY);

    type Raw = C::Raw;
    type Refined = M::Out;
    type Residual = Pair<C::Residual, M::Residual>;

    fn parse(&self, raw: &Self::Raw) -> Option<(Self::Refined, Self::Residual)> {
        let (refined, rc) = self.construct.parse(raw)?;
        let (out, rm) = self.then.forward(&refined);
        Some((out, Pair(rc, rm)))
    }

    fn reconstruct(&self, out: &Self::Refined, residual: &Self::Residual) -> Option<Self::Raw> {
        let refined = self.then.backward(out, &residual.1)?;
        self.construct.reconstruct(&refined, &residual.0)
    }
}

// ===== branching & guarded edges: completing the category ================
//
// A `Morphism`/`Construction` is a SINGLE-target edge between value objects. Two
// remaining edge shapes complete the category, so every hop of a multistate path is
// an edge of the algebra rather than a hand-written seam: a BRANCH lands in a
// COPRODUCT, and a GUARDED edge is a partial map admitted by an external witness.
// Both are name-BRANDED (a per-call brand `N`, GDP-style) so a proof minted for one
// value cannot discharge an edge on another — hence the generic-associated `<N>`.

/// A BRANCHING edge — a total morphism into a COPRODUCT. Where a `Morphism` lands in
/// one object, a `Branch` lands in one of two (`Left` / `Right`) and KEEPS the witness
/// of which: the categorical case-split (`classify`), not a `Maybe` that throws the
/// negative arm away. The brand `N` ties each produced proof to the one value
/// classified, so it can only discharge a `Guarded` edge on that SAME value. In the
/// grammar it declares a `CAPABILITY` like any edge, so a path through a branch keeps
/// a computed ceiling.
pub trait Branch: ValueOperator {
    /// The declared capability ceiling, as on every edge.
    const CAPABILITY: Capability;
    /// The branded input (e.g. `Named<N, _>`); the brand is not itself a citizen.
    type In<N>;
    /// The positive arm — a value object (often a proof realized as one).
    type Left<N>: ValueObject;
    /// The negative arm, KEPT as a first-class value object, never discarded.
    type Right<N>: ValueObject;
    /// Classify the input into one arm of the coproduct, branded to its value.
    fn branch<N>(&self, input: &Self::In<N>) -> Result<Self::Left<N>, Self::Right<N>>;
}

/// A GUARDED edge — the categorical SIBLING of `Construction`. Both are "a morphism
/// plus an admissibility witness", differing only in WHERE the witness is born: a
/// `Construction` MINTS its own (the parse succeeds, residual in hand), while a
/// `Guarded` edge DEMANDS one minted elsewhere — a name-branded `Proof` for the SAME
/// value, typically a `Branch`'s output. With the witness present the map is total,
/// so "you forgot the precondition" is a COMPILE error rather than a runtime check,
/// and no other path can reach `Out`. (A `Construction` reverses via its residual; a
/// `Guarded` edge consumes its brand and is one-way — reversal, when wanted, is its
/// own edge, the way `Void` reverses `Post`.)
pub trait Guarded: ValueOperator {
    /// The declared capability ceiling, as on every edge.
    const CAPABILITY: Capability;
    /// The branded input being admitted.
    type In<N>;
    /// The unforgeable witness required for the SAME brand `N`.
    type Proof<N>;
    /// The state reached once admitted — itself BRANDED by `N`, so the input's
    /// identity FLOWS through the edge (provenance coupling): the output is provably
    /// the image of the value that was admitted, not some other. (The underlying datum
    /// is a value object; the `N` wrapper carries its lineage.)
    type Out<N>;
    /// Cross the edge, consuming the proof of admissibility; the output keeps `N`.
    fn guard<N>(&self, input: &Self::In<N>, proof: &Self::Proof<N>) -> Self::Out<N>;
}

// ===== state machines: a boundary as a transition graph ==================

/// A boundary modelled EXPLICITLY as a state machine: one piece of `Data` carried
/// at a phantom protocol state. `At<S>` is that data viewed at state `S`; `at` and
/// `data` move it between states without re-implementing the move. Implement this
/// once per machine, then declare each legal REVERSIBLE edge with `transition!`.
///
/// This generalizes what `ledger` does ad hoc over distinct value-object types
/// (`Round: Summary -> Summary` cannot be applied to a `Transaction`): the type
/// graph IS a state machine. `At<S>` is the tool for the one case a structural type
/// change cannot express — the data shape is INVARIANT but the permitted next moves
/// change (`At<Draft>` and `At<Submitted>` are the same `Data`, different edges).
/// States are typestates; the carrier and the payload are value objects.
pub trait StateMachine {
    /// The payload carried through every state — invariant across transitions.
    type Data: ValueObject;
    /// The value object at protocol state `S`.
    type At<S: Typestate>: ValueObject;
    /// Seat the payload at state `S` — the only way to move it between states, so
    /// every transition routes through here.
    fn at<S: Typestate>(data: Self::Data) -> Self::At<S>;
    /// Read the payload, regardless of state (the sanctioned cross-state accessor).
    fn data<S: Typestate>(at: &Self::At<S>) -> &Self::Data;
}

/// Declare a named, REVERSIBLE transition `From => To` of a `StateMachine` as a
/// `Morphism`. The body is mechanical — preserve the payload, retag the state,
/// `Unit` residual, `Pure` — so this generates it, and the SOURCE carries only the
/// irreducible content: the edge's name and endpoints. Writing this macro IS adding
/// the edge to the graph; an edge you do not declare has no operator and cannot be
/// called, so the legal transition graph stays exactly the set of declarations.
///
/// It is ONLY for free, data-preserving edges. A GUARDED transition (a precondition
/// — see a GDP proof) or a BRANCHING one (several targets) is not mechanical — its
/// content is the guard or the branch — so it stays hand-written.
#[macro_export]
macro_rules! transition {
    ($(#[$meta:meta])* $name:ident : $machine:ty, $from:ty => $to:ty) => {
        $(#[$meta])*
        pub struct $name;
        $crate::value_operator!($name);
        impl $crate::boundary::Morphism for $name {
            const CAPABILITY: $crate::boundary::Capability = $crate::boundary::Capability::Pure;
            type In = <$machine as $crate::boundary::StateMachine>::At<$from>;
            type Out = <$machine as $crate::boundary::StateMachine>::At<$to>;
            type Residual = $crate::boundary::Unit;

            fn forward(&self, input: &Self::In) -> (Self::Out, $crate::boundary::Unit) {
                let payload = <$machine as $crate::boundary::StateMachine>::data(input).clone();
                (
                    <$machine as $crate::boundary::StateMachine>::at::<$to>(payload),
                    $crate::boundary::Unit,
                )
            }

            fn backward(
                &self,
                out: &Self::Out,
                _residual: &$crate::boundary::Unit,
            ) -> ::core::option::Option<Self::In> {
                let payload = <$machine as $crate::boundary::StateMachine>::data(out).clone();
                ::core::option::Option::Some(
                    <$machine as $crate::boundary::StateMachine>::at::<$from>(payload),
                )
            }
        }
    };
}

/// Declare a state machine's CARRIER and descriptor in one line. The carrier is a
/// phantom-indexed value object `Carrier<S>` over a `Payload`: its value semantics
/// (`Clone`/`PartialEq`/`Eq`/`Debug`) delegate to the payload and the phantom `S` is
/// erased, so the index costs nothing at runtime. The descriptor `Flow` is the
/// `StateMachine` whose `at`/`data` wrap and read the payload — the only operations
/// every `transition!` is built from.
///
/// This lifts the ~18 lines of identical carrier boilerplate each machine used to
/// hand-write (see `Entry`/`Gauge`). Declaring a machine is then: `typestate!` (the
/// states), `state_machine!` (carrier + descriptor), and one `transition!` per
/// reversible edge — the carrier's field stays module-private, so the defining module
/// still writes its own entry constructors and accessors.
#[macro_export]
macro_rules! state_machine {
    ($flow:ident, $carrier:ident, $payload:ty) => {
        pub struct $carrier<S>($payload, ::core::marker::PhantomData<S>);
        impl<S> ::core::clone::Clone for $carrier<S> {
            fn clone(&self) -> Self {
                $carrier(
                    ::core::clone::Clone::clone(&self.0),
                    ::core::marker::PhantomData,
                )
            }
        }
        impl<S> ::core::cmp::PartialEq for $carrier<S> {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl<S> ::core::cmp::Eq for $carrier<S> {}
        impl<S> ::core::fmt::Debug for $carrier<S> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_tuple(stringify!($carrier)).field(&self.0).finish()
            }
        }
        impl<S> $crate::boundary::sealed::Sealed for $carrier<S> {}
        impl<S> $crate::boundary::ValueObject for $carrier<S> {}

        pub struct $flow;
        $crate::value_operator!($flow);
        impl $crate::boundary::StateMachine for $flow {
            type Data = $payload;
            type At<S: $crate::boundary::Typestate> = $carrier<S>;

            fn at<S: $crate::boundary::Typestate>(data: $payload) -> $carrier<S> {
                $carrier(data, ::core::marker::PhantomData)
            }

            fn data<S: $crate::boundary::Typestate>(at: &$carrier<S>) -> &$payload {
                &at.0
            }
        }
    };
}

// ===== instrumentation: the morphism as the annotation point =============

/// A hook called around every metered morphism step. The default (`NoMeter`) does
/// nothing and costs nothing; a causal profiler (e.g. Coz) implements it to open
/// latency scopes and mark throughput progress points.
///
/// The usual friction with a causal profiler is deciding *which* methods to
/// annotate. The `Morphism` answers that: every dataflow step is one, so the
/// annotation set is DETERMINED by the algebra and labelled by the operator's
/// type — you never hand-pick instrumentation points. A Coz adapter is one impl,
/// behind a feature so the `coz` dependency stays optional:
///
/// ```ignore
/// pub struct CozMeter;
/// impl Meter for CozMeter {
///     fn measured<R>(&self, label: &'static str, body: impl FnOnce() -> R) -> R {
///         coz::scope!(label); // RAII latency region for this stage
///         body()
///     }
///     fn progress(&self, label: &'static str) { coz::progress!(label); }
/// }
/// ```
pub trait Meter {
    /// Run `body` as a measured region labelled `label`.
    fn measured<R>(&self, label: &'static str, body: impl FnOnce() -> R) -> R;
    /// Mark a unit of end-to-end progress (throughput) labelled `label`.
    fn progress(&self, label: &'static str);
}

/// Instrumentation off: zero cost, fully transparent.
pub struct NoMeter;
impl Meter for NoMeter {
    fn measured<R>(&self, _label: &'static str, body: impl FnOnce() -> R) -> R {
        body()
    }
    fn progress(&self, _label: &'static str) {}
}

impl<T: Meter + ?Sized> Meter for &T {
    fn measured<R>(&self, label: &'static str, body: impl FnOnce() -> R) -> R {
        (**self).measured(label, body)
    }
    fn progress(&self, label: &'static str) {
        (**self).progress(label)
    }
}

/// Wrap any morphism so each `forward` / `backward` is metered, labelled by the
/// operator's TYPE. Because every dataflow step is a `Morphism`, ONE wrapper
/// instruments the whole graph (including nested `Compose`s) at uniform
/// granularity — the same altitude-agnosticism that lets one `probe` test every
/// level. Metering is pure overhead, so the capability ceiling is unchanged, and
/// with `NoMeter` it compiles to the bare morphism.
pub struct Profiled<M, T = NoMeter> {
    inner: M,
    meter: T,
}
impl<M> Profiled<M, NoMeter> {
    pub fn new(inner: M) -> Self {
        Profiled {
            inner,
            meter: NoMeter,
        }
    }
}
impl<M, T> Profiled<M, T> {
    pub fn metered(inner: M, meter: T) -> Self {
        Profiled { inner, meter }
    }
}
impl<M, T> sealed::Sealed for Profiled<M, T> {}
impl<M, T> ValueOperator for Profiled<M, T> {}
impl<M: Morphism, T: Meter> Morphism for Profiled<M, T> {
    const CAPABILITY: Capability = M::CAPABILITY;
    type In = M::In;
    type Out = M::Out;
    type Residual = M::Residual;

    fn forward(&self, input: &Self::In) -> (Self::Out, Self::Residual) {
        let label = core::any::type_name::<M>();
        let out = self.meter.measured(label, || self.inner.forward(input));
        // a completed forward is a unit of this stage's throughput; a coarser
        // end-to-end progress point can be marked by the caller at the run boundary.
        self.meter.progress(label);
        out
    }

    fn backward(&self, out: &Self::Out, residual: &Self::Residual) -> Option<Self::In> {
        self.meter.measured(core::any::type_name::<M>(), || {
            self.inner.backward(out, residual)
        })
    }
}
