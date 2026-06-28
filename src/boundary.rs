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

// ----- capability at the TYPE level: markers + lattice --------------------
//
// The `Capability` enum above is the runtime REFLECTION; these markers put the same
// lattice in the TYPE system, so an effect ceiling can be DEMANDED as a bound
// (`where M::Capability: AtMost<Pure>`) — a compile-time effect contract the LSP can
// red-squiggle, the capability analog of the provenance contract. Each edge declares
// `type Capability`; the runtime `CAPABILITY` const is DERIVED from it by reflection,
// so every read site (the audit, the laws, the tests) is unchanged.

/// A type-level capability level, reflecting to the runtime `Capability`. Sealed: the
/// lattice is closed at exactly the four levels below.
pub trait Effect: sealed::Sealed {
    const VALUE: Capability;
}
/// The four levels as marker types (parallel to the `Capability` variants).
pub struct Pure;
pub struct Lossy;
pub struct Stateful;
pub struct Effectful;
impl sealed::Sealed for Pure {}
impl sealed::Sealed for Lossy {}
impl sealed::Sealed for Stateful {}
impl sealed::Sealed for Effectful {}
impl Effect for Pure {
    const VALUE: Capability = Capability::Pure;
}
impl Effect for Lossy {
    const VALUE: Capability = Capability::Lossy;
}
impl Effect for Stateful {
    const VALUE: Capability = Capability::Stateful;
}
impl Effect for Effectful {
    const VALUE: Capability = Capability::Effectful;
}

/// `Self` is at most `Ceiling` on the lattice — the bound that makes a ceiling
/// demandable. `where M::Capability: AtMost<Pure>` accepts only pure edges; a lossier
/// edge fails to satisfy it, at compile time.
pub trait AtMost<Ceiling> {}
impl AtMost<Pure> for Pure {}
impl AtMost<Lossy> for Pure {}
impl AtMost<Stateful> for Pure {}
impl AtMost<Effectful> for Pure {}
impl AtMost<Lossy> for Lossy {}
impl AtMost<Stateful> for Lossy {}
impl AtMost<Effectful> for Lossy {}
impl AtMost<Stateful> for Stateful {}
impl AtMost<Effectful> for Stateful {}
impl AtMost<Effectful> for Effectful {}

/// The type-level join (the composition rule): the higher of two levels. `Compose` and
/// `Then` use it, so a path's ceiling is computed in the type system — the type-level
/// twin of the const `join`.
pub trait Join<Other> {
    type Out: Effect;
}
macro_rules! join_impls {
    ($($a:ty, $b:ty => $out:ty);+ $(;)?) => {$(
        impl Join<$b> for $a { type Out = $out; }
    )+};
}
join_impls!(
    Pure, Pure => Pure; Pure, Lossy => Lossy; Pure, Stateful => Stateful; Pure, Effectful => Effectful;
    Lossy, Pure => Lossy; Lossy, Lossy => Lossy; Lossy, Stateful => Stateful; Lossy, Effectful => Effectful;
    Stateful, Pure => Stateful; Stateful, Lossy => Stateful; Stateful, Stateful => Stateful; Stateful, Effectful => Effectful;
    Effectful, Pure => Effectful; Effectful, Lossy => Effectful; Effectful, Stateful => Effectful; Effectful, Effectful => Effectful;
);

/// Apply a morphism only if its declared effect is at most `Ceiling` — a compile-time
/// effect contract, the capability twin of the provenance `Stamped` bound. The bound
/// `M::Capability: AtMost<Ceiling>` is checked by the type system at the CALL site: an
/// edge lossier than the ceiling fails to compile, so a coding agent gets LSP push-back
/// on the wrong effect shape before any test runs. `tests/compile_fail` pins the
/// negative (a `Lossy` edge rejected by an `AtMost<Pure>` call), and `run_pure` names
/// the common `Pure` ceiling.
pub fn run_within<Ceiling, M: Morphism>(m: &M, input: &M::In) -> (M::Out, M::Residual)
where
    M::Capability: AtMost<Ceiling>,
{
    m.forward(input)
}

/// `run_within` with the ceiling fixed at `Pure` — accepts only effect-free edges, so a
/// lossy, stateful, or effectful edge is rejected at compile time.
pub fn run_pure<M: Morphism>(m: &M, input: &M::In) -> (M::Out, M::Residual)
where
    M::Capability: AtMost<Pure>,
{
    run_within::<Pure, M>(m, input)
}

// ===== gradings: one monoid pattern, manifested at three levels ===========
//
// Each edge carries ANNOTATIONS that accumulate — a GRADING, i.e. a monoid (`EMPTY`
// for `id`, `combine` along the chain). The SAME pattern appears at all three levels
// of Rust's tower, each living where its guarantees require — and split by WHAT it
// annotates: residual and capability are properties of the OPERATOR (composed by
// `Compose`); provenance is the journey of a VALUE (accumulated by stamping):
//
//   - RESIDUAL — a TYPE-level monoid `(ValueObject, Pair, Unit)` on the OPERATOR. It
//     must be type-level so "discarded residual ⇒ not invertible" is a COMPILE error.
//   - CAPABILITY — a CONST-level monoid `(Capability, join, Pure)` on the OPERATOR. It
//     must be const so the effect ceiling is a compile-time fact (a `const fn` cannot
//     call a trait method, so it composes via the inherent `join`).
//   - PROVENANCE — a TYPE-level lineage `(Origin / Step, ++)` on the VALUE: each
//     `stamp` extends the value's path TYPE by the edge it crossed, so a value's type
//     proves where it came from (a compile-time provenance contract), reflectable to
//     the runtime `Provenance` via `Lineage`. Generative, so it accrues by an explicit
//     stamp call, not by `Compose`.
//
// The genus cannot be ONE Rust trait — type/const/value are different worlds, and
// associated-type defaults are unstable — but it is one structure; `Monoid` names the
// value-level reflection, and residual/capability are the type/const analogues.

/// The algebra a value-level grading accumulates in: `EMPTY` for the identity edge,
/// `combine` along composition. (Associative with `EMPTY` as unit — the monoid laws.)
pub trait Monoid {
    const EMPTY: Self;
    fn combine(self, other: Self) -> Self;
}

/// `Capability` IS such a monoid (`Pure` = empty, `join` = combine) — though `Compose`
/// accumulates it through the `const fn join` directly, since a const context cannot
/// call this trait method.
impl Monoid for Capability {
    const EMPTY: Self = Capability::Pure;
    fn combine(self, other: Self) -> Self {
        self.join(other)
    }
}

/// The runtime REFLECTION of a value's type-level lineage: the ordered edge labels it
/// flowed through. The lineage itself lives in the type (`Origin`/`Step`); `Lineage`
/// reflects it here when a human (or a log) wants to read it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Provenance(Vec<&'static str>);

impl Provenance {
    /// The lineage of a single edge, labelled by its operator type.
    pub fn single(label: &'static str) -> Self {
        Provenance(vec![label])
    }
    /// The ordered edge labels this value flowed through.
    pub fn steps(&self) -> &[&'static str] {
        &self.0
    }
}

impl Monoid for Provenance {
    const EMPTY: Self = Provenance(Vec::new());
    fn combine(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }
}

// ----- the type-level lineage and the stamped value that carries it --------

/// The empty lineage — a value that has crossed no edges yet (the `Nil` of the
/// type-level path).
pub struct Origin;

/// A lineage `Edge` then `Rest` — the type-level `Cons`. A value's path grows by
/// wrapping its previous path in a `Step` per edge crossed, so the whole journey is a
/// type, checkable at compile time.
pub struct Step<Edge, Rest>(PhantomData<(Edge, Rest)>);

/// Reflect a type-level lineage to the runtime `Provenance` (oldest edge first).
pub trait Lineage {
    fn reflect() -> Provenance;
}
impl Lineage for Origin {
    fn reflect() -> Provenance {
        Provenance::EMPTY
    }
}
impl<Edge, Rest: Lineage> Lineage for Step<Edge, Rest> {
    fn reflect() -> Provenance {
        Rest::reflect().combine(Provenance::single(core::any::type_name::<Edge>()))
    }
}

/// A value tagged with its type-level lineage `Path`. Its TYPE records every edge it
/// has crossed, so a consumer can demand a specific provenance (`Stamped<Step<Round,
/// Step<Aggregate, Origin>>, _>`) and a value with the wrong history will not compile.
/// `lineage()` reflects the path to a runtime `Provenance` when you want to read it.
pub struct Stamped<Path, T>(T, PhantomData<Path>);

impl<T> Stamped<Origin, T> {
    /// Enter a value into provenance tracking at the origin (no edges yet).
    pub fn origin(value: T) -> Self {
        Stamped(value, PhantomData)
    }
}

impl<Path, T> Stamped<Path, T> {
    /// The carried value (read-only).
    pub fn value(&self) -> &T {
        &self.0
    }
    /// The runtime lineage, reflected from the type-level `Path`.
    pub fn lineage(&self) -> Provenance
    where
        Path: Lineage,
    {
        Path::reflect()
    }
}

/// Run a morphism on a stamped value and EXTEND its lineage type by the edge crossed —
/// the type-level, n-ary value stamping. (Identity axis: the morphism's residual is
/// `run`'s concern, so it is dropped here.) Each call adds one `Step`, so the value's
/// type carries its whole journey.
pub fn stamp_through<Path, M: Morphism>(
    input: &Stamped<Path, M::In>,
    m: &M,
) -> Stamped<Step<M, Path>, M::Out> {
    Stamped(m.forward(input.value()).0, PhantomData)
}

// ===== degrees of freedom: coverage as a COMPILE-TIME obligation ==========
//
// `synth` reads a value object's degrees of freedom off its type and checks, at
// runtime, that a probe's instrument is appropriate to each. The type-level twin
// makes that completeness a BOUND: a value object declares its DOFs as a type-level
// set (`DofCons`/`DofNil`, the same cons-list shape as the provenance lineage), a
// check declares which DOFs it `Covers`, and `CoversAll` recurses the set — so a probe
// that omits a dimension fails to COMPILE, the way an unbalanced post does. The runtime
// `Dof` enumeration is then DERIVED from the type-level set, exactly as the runtime
// `CAPABILITY` is derived from `type Capability` (the third use of the reflect pattern).

/// The empty type-level DOF set — a probe covers it vacuously.
pub struct DofNil;
/// A cons cell of a type-level DOF set: head DOF `H`, tail set `T`.
pub struct DofCons<H, T>(PhantomData<(H, T)>);

/// A value object declares its degrees of freedom as a type-level set, so a probe's
/// completeness is demandable as a bound rather than enumerated at runtime. (The
/// runtime `synth::Dof` list reflects from `Self::Dofs`.)
pub trait HasDofs {
    /// The type-level DOF set (a `DofCons`/`DofNil` list).
    type Dofs;
}

/// A check that can SEE the degree of freedom `D` — it reaches the dimension AND fires
/// an instrument appropriate to it (per `synth`, reaching with the wrong check covers
/// nothing). The per-dimension witness a complete probe must exhibit.
pub trait Covers<D> {}

/// A check that covers EVERY DOF in the type-level set `L` — the completeness relation,
/// by recursion over the set. No overlap: `DofNil` and `DofCons` are disjoint head
/// constructors, so this is a clean bound on stable Rust.
pub trait CoversAll<L> {}
impl<C> CoversAll<DofNil> for C {}
impl<C, H, T> CoversAll<DofCons<H, T>> for C
where
    C: Covers<H>,
    C: CoversAll<T>,
{
}

/// The completeness obligation as a BOUND: this type-checks only when `C` covers every
/// DOF the value object `T` declares. A check that omits a dimension fails to compile —
/// the static twin of `synth`'s runtime coverage check, and the LSP push-back a coding
/// agent gets for an incomplete probe before any test runs. (`tests/compile_fail` pins
/// the negative: an output-only check that misses a real DOF is rejected.)
pub fn require_complete<T, C>(_check: &C)
where
    T: HasDofs,
    C: CoversAll<<T as HasDofs>::Dofs>,
{
}

// ===== DOF-driven probe SYNTHESIS: the declaration generates the suite =====
//
// `require_complete` checks, at compile time, that a probe COVERS the declared DOFs.
// This goes the other way: each DOF supplies the perturbation that reaches its
// dimension, so the runtime completeness suite is SYNTHESIZED from the type-level DOF
// set — declaring a degree of freedom generates its probe. Coverage is unified across
// the residual axis: a DOF is covered if perturbing along it is OBSERVABLE, either
// because the OUTPUT changes (the dimension survives into the output) or the RESIDUAL
// completely captures it (a lossy dimension witnessed). A dimension covered by NEITHER
// is silently dropped — exactly the bug the probe exists to find.

/// A degree of freedom that can be probed on morphism `M`: it supplies the perturbation
/// reaching its dimension. With this, the completeness suite is derived from the DOF set
/// rather than hand-written per dimension.
pub trait DofProbe<M: Morphism> {
    /// The perturbation that moves this dimension.
    type Perturb: Perturbation<M>;
    fn perturbation() -> Self::Perturb;
}

/// Is this DOF OBSERVABLE through `m` at `x`? `Some(true)` if the output responds or the
/// residual completely captures the perturbation; `Some(false)` if it is silently
/// dropped; `None` if the perturbation does not apply at `x`.
pub fn dof_covered<M, D>(m: &M, x: &M::In) -> Option<bool>
where
    M: Morphism,
    D: DofProbe<M>,
{
    let pr = probe(m, &D::perturbation(), x)?;
    Some(!pr.output_invariant || pr.residual_complete())
}

/// Walk a type-level DOF set, probing each DOF on `M` (oldest first). The recursion over
/// `DofCons`/`DofNil` is what turns the type-level declaration into a runtime sweep.
pub trait ProbeDofs<M: Morphism> {
    fn probe_each(m: &M, x: &M::In, out: &mut Vec<Option<bool>>);
}
impl<M: Morphism> ProbeDofs<M> for DofNil {
    fn probe_each(_m: &M, _x: &M::In, _out: &mut Vec<Option<bool>>) {}
}
impl<M, H, T> ProbeDofs<M> for DofCons<H, T>
where
    M: Morphism,
    H: DofProbe<M>,
    T: ProbeDofs<M>,
{
    fn probe_each(m: &M, x: &M::In, out: &mut Vec<Option<bool>>) {
        out.push(dof_covered::<M, H>(m, x));
        T::probe_each(m, x, out);
    }
}

/// Synthesize and run the completeness suite for EVERY degree of freedom the value
/// object `T` declares (`HasDofs`), against edge `M` — one verdict per DOF. The DOF
/// declaration generates the probes: a value object that adds a dimension to its
/// `HasDofs` set automatically gets it checked, with no new test code.
pub fn probe_declared_dofs<T, M>(m: &M, x: &M::In) -> Vec<Option<bool>>
where
    T: HasDofs,
    T::Dofs: ProbeDofs<M>,
    M: Morphism,
{
    let mut out = Vec::new();
    <T::Dofs as ProbeDofs<M>>::probe_each(m, x, &mut out);
    out
}

/// A possibly-lossy morphism whose `Residual` value object witnesses EXACTLY
/// what the forward map collapsed. Retaining the residual restores
/// invertibility: `backward(forward(x)) == x`.
pub trait Morphism: ValueOperator {
    /// The declared capability ceiling, at the TYPE level (a marker implementing
    /// `Effect`), so it is composable by `Join` and demandable as an `AtMost` bound.
    type Capability: Effect;
    /// The same ceiling reflected to a runtime value — DERIVED from `Capability`, so
    /// the audit/laws/tests read it unchanged. (Don't override; set `type Capability`.)
    const CAPABILITY: Capability = <Self::Capability as Effect>::VALUE;

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

// ===== oracle-free RELATIONS over an arbitrary map (any edge shape) ========

/// A metamorphic `Metamorphic<M>` relation is tied to a `Morphism`; but the value
/// frontier the interpreter exposed sits behind a `Guarded` edge (evaluation is admitted
/// by a `WellTyped` witness), and SHAPE laws — round-trip, completeness — certify that an
/// edge is structurally faithful, never that it computes the RIGHT value. A `Relation`
/// is the oracle-FREE way to pin value behaviour for ANY pure map regardless of edge
/// shape: it states that two routes to a result must AGREE — `apply(vary(x))` equals
/// `rewrite(apply(x))` — so it catches a wrong constant or a non-strict comparison
/// without ever naming the correct answer (the only structural attack on the VALUE
/// frontier, e.g. `eval(a + b) == eval(b + a)`). `apply` may itself drive a whole
/// `Parse`/`Check`/`Eval` pipeline; the relation does not care.
pub trait Relation {
    /// The relation's input domain.
    type In: Clone + Debug;
    /// The observed result the two routes must agree on.
    type Out: PartialEq + Debug;
    /// The map under test (an arbitrary pure function — eval, a pipeline, a morphism).
    fn apply(&self, x: &Self::In) -> Self::Out;
    /// Perturb the input along the relation's dimension (`None` if it does not apply).
    fn vary(&self, x: &Self::In) -> Option<Self::In>;
    /// The output transform the perturbation MUST induce (identity for a symmetry).
    fn rewrite(&self, y: Self::Out) -> Self::Out;
}

/// Oracle-free relation probe: `Some(true)` iff the two routes agree at `x` —
/// `apply(vary(x)) == rewrite(apply(x))`. `None` if the perturbation does not apply.
pub fn relation_holds<R: Relation>(r: &R, x: &R::In) -> Option<bool> {
    let varied = r.vary(x)?;
    Some(r.apply(&varied) == r.rewrite(r.apply(x)))
}

// ===== the COST grading: open keyed time/space budgets at the type level ===
//
// Cost is the fifth grading, and the first built as a PLUGGABLE grading: a grading is a
// marker `G`, an edge declares `Graded<G> { type Carrier }`, each grading names its own
// `Combine` rule, and ONE blanket `Compose` impl threads every grading. Adding a grading
// touches neither `Morphism` nor the other gradings — the level-heterogeneity that once
// blocked this was dissolved when capability was lifted to the type level.
//
// A cost is an OPEN keyed map from named SIZE AXES to a polynomial DEGREE (a type-level
// nat, so any n^k — no fixed lattice cap). Two resources ride the one map machinery as
// the gradings `TimeCost` and `SpaceCost`, differing only at iteration: mapping an edge
// per element multiplies BOTH (n results materialized) while folding multiplies only
// time (it streams). The point is the DEMAND: a path whose degree on some axis exceeds
// the budget is a COMPILE error — the pressure a complexity-blind agent otherwise lacks.

// --- type-level naturals: the open polynomial degree ---
pub struct Z;
pub struct S<N>(PhantomData<N>);
/// Reflect a type-level degree to its number (for the audit / diagnostics).
pub trait Degree {
    const N: u32;
}
impl Degree for Z {
    const N: u32 = 0;
}
impl<D: Degree> Degree for S<D> {
    const N: u32 = D::N + 1;
}

// --- type-level booleans + selection ---
pub struct True;
pub struct False;
/// Pick `T` on `True`, `E` on `False` — the type-level conditional.
pub trait Select<T, E> {
    type Out;
}
impl<T, E> Select<T, E> for True {
    type Out = T;
}
impl<T, E> Select<T, E> for False {
    type Out = E;
}

// --- nat equality / <= / max (overlap-free: disjoint head constructors) ---
pub trait NatEq<B> {
    type Out;
}
impl NatEq<Z> for Z {
    type Out = True;
}
impl<B> NatEq<S<B>> for Z {
    type Out = False;
}
impl<A> NatEq<Z> for S<A> {
    type Out = False;
}
impl<A, B> NatEq<S<B>> for S<A>
where
    A: NatEq<B>,
{
    type Out = <A as NatEq<B>>::Out;
}
/// `Self <= B` at the type level (reflected to a `Bool`).
pub trait Le<B> {
    type Out;
}
impl<B> Le<B> for Z {
    type Out = True;
}
impl<A> Le<Z> for S<A> {
    type Out = False;
}
impl<A, B> Le<S<B>> for S<A>
where
    A: Le<B>,
{
    type Out = <A as Le<B>>::Out;
}
/// The larger of two degrees.
pub trait Max<B> {
    type Out;
}
impl<B> Max<B> for Z {
    type Out = B;
}
impl<A> Max<Z> for S<A> {
    type Out = S<A>;
}
impl<A, B> Max<S<B>> for S<A>
where
    A: Max<B>,
{
    type Out = S<<A as Max<B>>::Out>;
}

// --- size axes: a named dimension of the input, with a unique type-level Id ---
/// A named size axis (e.g. AST nodes, nesting depth, type variables). The `Id`
/// distinguishes axes in the cost map's overlap-free lookup.
pub trait Axis {
    type Id;
}

// --- the cost map: a type-level multimap (axis, degree); lookup takes the MAX per ---
// axis, so sequential composition is just append (no map merge needed).
pub struct CostNil;
pub struct CostCons<Ax, Deg, Rest>(PhantomData<(Ax, Deg, Rest)>);

/// The degree a cost map assigns axis `Q` — the max over matching entries, `Z` if absent.
pub trait Lookup<Q> {
    type Deg;
}
impl<Q> Lookup<Q> for CostNil {
    type Deg = Z;
}
impl<Q, Ax, Deg, Rest> Lookup<Q> for CostCons<Ax, Deg, Rest>
where
    Q: Axis,
    Ax: Axis,
    <Q as Axis>::Id: NatEq<<Ax as Axis>::Id>,
    Rest: Lookup<Q>,
    Deg: Max<<Rest as Lookup<Q>>::Deg>,
    <<Q as Axis>::Id as NatEq<<Ax as Axis>::Id>>::Out:
        Select<<Deg as Max<<Rest as Lookup<Q>>::Deg>>::Out, <Rest as Lookup<Q>>::Deg>,
{
    type Deg = <<<Q as Axis>::Id as NatEq<<Ax as Axis>::Id>>::Out as Select<
        <Deg as Max<<Rest as Lookup<Q>>::Deg>>::Out,
        <Rest as Lookup<Q>>::Deg,
    >>::Out;
}

/// Append two cost maps (sequential composition's combine; per-axis max is folded into
/// `Lookup`, so this needs no merge).
pub trait AppendCost<B> {
    type Out;
}
impl<B> AppendCost<B> for CostNil {
    type Out = B;
}
impl<Ax, Deg, Rest, B> AppendCost<B> for CostCons<Ax, Deg, Rest>
where
    Rest: AppendCost<B>,
{
    type Out = CostCons<Ax, Deg, <Rest as AppendCost<B>>::Out>;
}

/// A cost map is WITHIN a `Ceiling` iff, for every axis the ceiling names, the cost's
/// degree for that axis is `<=` the ceiling's. The demandable budget bound.
pub trait WithinBudget<Ceiling> {}
impl<Cost> WithinBudget<CostNil> for Cost {}
impl<Cost, Ax, CDeg, Rest> WithinBudget<CostCons<Ax, CDeg, Rest>> for Cost
where
    Cost: Lookup<Ax>,
    <Cost as Lookup<Ax>>::Deg: Le<CDeg, Out = True>,
    Cost: WithinBudget<Rest>,
{
}

// --- the pluggable grading core ---
/// A grading KIND (a marker): residual, capability, cost, ... Each names a `Combine`
/// rule and edges declare a `Graded` carrier — which is what makes gradings PLUGGABLE: a
/// new grading needs no edit to `Morphism` or the other gradings.
pub trait Grading {}
/// Edge `Self` carries a type-level value for grading `G`.
pub trait Graded<G: Grading> {
    type Carrier;
}
/// How two carriers combine under grading `G` when edges run in SEQUENCE.
pub trait Combine<G: Grading, Rhs> {
    type Out;
}
/// ONE blanket impl threads EVERY present and future grading through `Compose`.
impl<G, F, T> Graded<G> for Compose<F, T>
where
    G: Grading,
    F: Graded<G>,
    T: Graded<G>,
    <F as Graded<G>>::Carrier: Combine<G, <T as Graded<G>>::Carrier>,
{
    type Carrier = <<F as Graded<G>>::Carrier as Combine<G, <T as Graded<G>>::Carrier>>::Out;
}

// --- cost as two pluggable gradings, time and space, over the one map ---
/// The time-complexity grading (carrier: a cost map).
pub struct TimeCost;
/// The space-complexity grading (carrier: a cost map).
pub struct SpaceCost;
impl Grading for TimeCost {}
impl Grading for SpaceCost {}
// Sequential composition appends the maps for BOTH resources (max folded into lookup).
impl<A, B> Combine<TimeCost, B> for A
where
    A: AppendCost<B>,
{
    type Out = <A as AppendCost<B>>::Out;
}
impl<A, B> Combine<SpaceCost, B> for A
where
    A: AppendCost<B>,
{
    type Out = <A as AppendCost<B>>::Out;
}

/// Demand a budget on grading `G`: a compile error unless `E`'s `G`-carrier is within
/// `Ceiling`. `require_within::<TimeCost, E, Budget>()` is the time demand; pass
/// `SpaceCost` for space. (Argument-free, so it works on cost-only marker combinators.)
pub fn require_within<G, E, Ceiling>()
where
    G: Grading,
    E: Graded<G>,
    <E as Graded<G>>::Carrier: WithinBudget<Ceiling>,
{
}

// --- iteration combinators: where time and space part ways ---
/// Apply an edge once per element along axis `A` and COLLECT the results: BOTH time and
/// space gain a degree on `A` (the n results are materialized).
pub struct MapCollect<E, A>(PhantomData<(E, A)>);
impl<E, A> Graded<TimeCost> for MapCollect<E, A>
where
    E: Graded<TimeCost>,
    <E as Graded<TimeCost>>::Carrier: Lookup<A>,
{
    type Carrier = CostCons<
        A,
        S<<<E as Graded<TimeCost>>::Carrier as Lookup<A>>::Deg>,
        <E as Graded<TimeCost>>::Carrier,
    >;
}
impl<E, A> Graded<SpaceCost> for MapCollect<E, A>
where
    E: Graded<SpaceCost>,
    <E as Graded<SpaceCost>>::Carrier: Lookup<A>,
{
    type Carrier = CostCons<
        A,
        S<<<E as Graded<SpaceCost>>::Carrier as Lookup<A>>::Deg>,
        <E as Graded<SpaceCost>>::Carrier,
    >;
}
/// Apply an edge once per element along axis `A` but FOLD (stream): time gains a degree
/// on `A`, space stays flat (results are consumed, not held) — "stream, don't
/// materialize" as a type-level fact.
pub struct Fold<E, A>(PhantomData<(E, A)>);
impl<E, A> Graded<TimeCost> for Fold<E, A>
where
    E: Graded<TimeCost>,
    <E as Graded<TimeCost>>::Carrier: Lookup<A>,
{
    type Carrier = CostCons<
        A,
        S<<<E as Graded<TimeCost>>::Carrier as Lookup<A>>::Deg>,
        <E as Graded<TimeCost>>::Carrier,
    >;
}
impl<E, A> Graded<SpaceCost> for Fold<E, A>
where
    E: Graded<SpaceCost>,
{
    type Carrier = <E as Graded<SpaceCost>>::Carrier;
}

/// The empirical honesty audit (the cost twin of the residual probe): the type level
/// checks declared costs COMPOSE within budget, but cannot see whether a leaf's declared
/// degree matches reality. `fits` does: it measures a deterministic step count at growing
/// sizes and rejects a `degree` lower than the observed growth — a `degree`-1 edge that
/// is secretly quadratic is caught. (Deterministic STEP count, not wall-clock, so the
/// verdict is reproducible; the bound for a degree-k claim is the exact doubling ratio
/// 2^k, which a real edge with positive lower-order terms ratios strictly below.)
pub fn fits(measure: impl Fn(usize) -> u64, degree: u32) -> bool {
    let sizes = [16usize, 32, 64, 128];
    let bound = 2f64.powi(degree as i32);
    let mut prev: Option<u64> = None;
    for &n in &sizes {
        let work = measure(n);
        if let Some(p) = prev {
            let ratio = work as f64 / (p.max(1)) as f64;
            if ratio > bound {
                return false;
            }
        }
        prev = Some(work);
    }
    true
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
    F::Capability: Join<G::Capability>,
{
    // the composite is as capable as its most-capable stage — the static join,
    // now computed in the TYPE system: the effect of the composite is the
    // type-level `Join` of the two stages' effects, so a mis-stated composite
    // ceiling is an LSP error, not a runtime surprise. The runtime `CAPABILITY`
    // const is still derived from it for the audit/laws.
    type Capability = <F::Capability as Join<G::Capability>>::Out;

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
    /// system just like a `Compose` chain's. Declared as a type-level `Effect`; the
    /// runtime const is derived so the audit/laws read it unchanged.
    type Capability: Effect;
    /// Reflected from `type Capability` — do not override.
    const CAPABILITY: Capability = <Self::Capability as Effect>::VALUE;

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
    C::Capability: Join<M::Capability>,
{
    // the path is as capable as its most-capable edge — the static join in the TYPE
    // system, exactly as `Compose` does for two morphisms.
    type Capability = <C::Capability as Join<M::Capability>>::Out;

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
    /// The declared capability ceiling, as on every edge — a type-level `Effect`.
    type Capability: Effect;
    /// Reflected from `type Capability` — do not override.
    const CAPABILITY: Capability = <Self::Capability as Effect>::VALUE;
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
    /// The declared capability ceiling, as on every edge — a type-level `Effect`.
    type Capability: Effect;
    /// Reflected from `type Capability` — do not override.
    const CAPABILITY: Capability = <Self::Capability as Effect>::VALUE;
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
            type Capability = $crate::boundary::Pure;
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
    // profiling is transparent to the effect — it just times the inner edge.
    type Capability = M::Capability;
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
