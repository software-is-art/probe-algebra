//! Tier: KERNEL — the trusted floor — defines/runs the format, exempt from the structural rules.
//!
//! boundary.rs — the GRAMMAR of module boundaries.
//!
//! A module's boundary is the ONLY surface it exposes, and it is a CATEGORY: just
//! two things cross the seam —
//!
//!   - OBJECTS — the value objects (the nouns): immutable, validated, value-equality
//!     data, never exposing mutable internals. The raw primitives are the one set of
//!     objects OUTSIDE the domain — the source every construction flows out of.
//!   - MORPHISMS — the value operators (the verbs): pure maps with no I/O and no
//!     external mutation, in a few type-distinguished SHAPES that share one algebra
//!     (residual, backward reconstruction, probe): `Morphism` (a total edge between
//!     value objects), `Construction` (the PARTIAL entry edge from a raw primitive —
//!     "parse, don't validate"), `Branch` (a total edge into a coproduct), and
//!     `Guarded` (a partial edge admitted by a witness).
//!
//! A TYPESTATE is not a third citizen but an INDEX that distinguishes objects:
//! `Carried<M, Retained>` and `Carried<M, Discarded>` are two objects of the category —
//! same data, different edges (only the first can `invert`). This is why construction,
//! long left as a native `fn new` OUTSIDE the algebra, is just another morphism: the edge
//! INTO the domain, now probeable like every other.
//!
//! This file defines that grammar once, for the whole crate — and for CONSUMERS:
//!
//!   - marker traits `ValueObject` / `Typestate` / `ValueOperator`, registered only
//!     through the citizen macros (`value_object!` / `refined!` / `typestate!` /
//!     `value_operator!` / `proof_token!`), so no module invents a new KIND of
//!     citizen. A downstream crate mints its own citizens with the same macros; the
//!     closure that matters — every citizen is macro-registered, every edge probed,
//!     every file tiered — is enforced per-crate by `boundary-enforce`, not by a
//!     type-level seal (only the effect lattice stays truly sealed, because its
//!     laws are proven exhaustively over exactly four levels); and
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

use typewit::TypeEq;

/// The sealing module for the EFFECT LATTICE only. `Sealed` is `pub(crate)` because the
/// lattice's laws (`Join` commutativity/associativity/idempotence, `AtMost` agreement)
/// are proven EXHAUSTIVELY over exactly four levels — a fifth implementor would make
/// those proofs silently non-exhaustive, so no crate may add one.
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// The citizen-registration module — PUBLIC, unlike the effect seal, so a DOWNSTREAM
/// crate can mint its own value objects and operators with the same macros this crate
/// uses. `Registered` is the supertrait the citizen macros implement; registering by
/// hand instead of through a macro is possible but pointless — the structural closure
/// that actually protects a codebase (every citizen macro-registered, every edge
/// probed, every file tiered) is `boundary-enforce`'s job, per-crate, where it can see
/// the source. The kind-set stays closed: there are exactly three marker traits, and
/// no crate can add a fourth.
pub mod citizen {
    /// Implemented (via the citizen macros) by every boundary citizen.
    pub trait Registered {}
}

// ===== the boundary citizens: objects and morphisms ======================

/// Marker: an OBJECT of the category — an immutable, validated, value-equality datum.
pub trait ValueObject: Clone + PartialEq + Debug + citizen::Registered {}

/// Marker: an INDEX that distinguishes objects (a compile-time protocol position), so
/// illegal sequencing fails to compile. Not an object itself — `Carried<M, Retained>` is
/// the object; `Retained` is the index.
pub trait Typestate: citizen::Registered {}

/// Marker: a MORPHISM of the category — a pure operator-as-value over value objects
/// (`Morphism`, `Construction`, `Branch`, `Guarded`). (Free pure functions are morphisms
/// too; this marks the operator-as-object case.)
pub trait ValueOperator: citizen::Registered {}

/// Declarative sugar so per-module `boundary.rs` files read like a grammar:
/// `value_object!(Int, Expr, Value);`
#[macro_export]
macro_rules! value_object {
    ($($t:ty),+ $(,)?) => {$(
        impl $crate::boundary::citizen::Registered for $t {}
        impl $crate::boundary::ValueObject for $t {}
    )+};
}

/// `typestate!(Retained, Discarded);`
#[macro_export]
macro_rules! typestate {
    ($($t:ty),+ $(,)?) => {$(
        impl $crate::boundary::citizen::Registered for $t {}
        impl $crate::boundary::Typestate for $t {}
    )+};
}

/// `value_operator!(Parse, Check, Eval);`
#[macro_export]
macro_rules! value_operator {
    ($($t:ty),+ $(,)?) => {$(
        impl $crate::boundary::citizen::Registered for $t {}
        impl $crate::boundary::ValueOperator for $t {}
    )+};
}

/// Declare a module's edge set in ONE place — the single source `assert_all_probed` checks
/// completeness over: `type Edges = edges!(Parse, Check, Eval);` expands to
/// `EdgeCons<Parse, EdgeCons<Check, EdgeCons<Eval, EdgeNil>>>`. Giving edges a single
/// definition site is what makes "every edge is probed" as sound as DOF-completeness (whose
/// list `derive(Shaped)` reads from the type's one definition).
#[macro_export]
macro_rules! edges {
    () => { $crate::boundary::EdgeNil };
    ($head:ty $(, $tail:ty)* $(,)?) => {
        $crate::boundary::EdgeCons<$head, $crate::edges!($($tail),*)>
    };
}

/// A NAME-BRANDED PROOF TOKEN realized as a value object: zero data, branded by a
/// phantom `N`, so two tokens of the same name are the SAME fact and a token for name
/// A cannot stand in for B. Its field is private to the defining module, so it is
/// minted only there (a GDP "ghost" — `WellTyped<N>` / `IllTyped<N>`). This lifts the
/// ~17 lines those hand-rolled each into one line: `proof_token!(/// doc... WellTyped);`
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
        impl<N> $crate::boundary::citizen::Registered for $name<N> {}
        impl<N> $crate::boundary::ValueObject for $name<N> {}
    };
}

/// A REFINED value object: a newtype over a primitive admitted only by a validity rule —
/// the inward rule's whole premise ("every primitive that means something is a value object
/// with a smart constructor"), as one grammar line. The PREDICATE is the only content; the
/// struct, the parse-don't-validate `new`, and the `value_object!` registration are
/// generated, so a leaf cannot be declared without its validity rule:
///
/// ```ignore
/// refined! {
///     /// A non-negative integer.
///     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
///     pub struct Int(i64);
///     // `new` is given the raw input and returns the validated/normalized FIELD, or None.
///     fn new(n: i64) = (n >= 0).then_some(n);
/// }
/// ```
///
/// `new`'s body is an expression of type `Option<Field>` over the bound argument — pure
/// refinement (`Int`) or refinement-with-normalization (`Ident`: `&str` in, `String`
/// stored). Accessors and value OPERATORS stay in a normal `impl` (they are content).
#[macro_export]
macro_rules! refined {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ( $field:ty );
        fn new( $arg:ident : $argty:ty ) = $validate:expr;
    ) => {
        $(#[$meta])*
        $vis struct $name($field);
        impl $name {
            /// Parse-don't-validate: `Some` iff the raw input satisfies the rule.
            pub fn new($arg: $argty) -> ::core::option::Option<Self> {
                let __v: ::core::option::Option<$field> = $validate;
                __v.map($name)
            }
        }
        $crate::value_object!($name);
    };
}

/// Assign each named size axis a UNIQUE sequential Peano `Id`, left to right — so two axes
/// cannot accidentally share an id (the one mistake the open cost map's overlap-freedom
/// depends on). `axis!(Nodes, Depth)` gives `Nodes = Z`, `Depth = S<Z>`, ...
#[macro_export]
macro_rules! axis {
    ($($name:ident),+ $(,)?) => {
        $crate::axis!(@ $crate::boundary::Z; $($name),+);
    };
    (@ $cur:ty; $name:ident $(, $rest:ident)*) => {
        impl $crate::boundary::Axis for $name {
            type Id = $cur;
        }
        $crate::axis!(@ $crate::boundary::S<$cur>; $($rest),*);
    };
    (@ $cur:ty;) => {};
}

/// Declare an oracle-free `Relation` (see below) as a grammar line instead of a ~12-line
/// `impl`. Every value relation has the same skeleton — `apply` a map, `vary` the input,
/// `rewrite` the expected output — and only the three closures carry information; the
/// `type In`/`type Out`/method ceremony is pure repetition. This lifts that the way
/// `proof_token!` lifts a branded witness:
///
/// ```ignore
/// relation!(/// additive identity
///     AddIdentity: Expr => Value,
///     apply = |x| eval_closed(x),
///     vary = |x| Some(Expr::bin(Op::Add, x.clone(), Expr::int(0).unwrap())),
///     rewrite = |y| y);
/// ```
///
/// (Used only for the laws REGISTRY's relations. Edge SHAPES — the morphism families — are
/// deliberately left explicit: their per-variant `type Capability` and residual choices are
/// the demonstration, not boilerplate, so a macro would hide exactly what they teach.)
#[macro_export]
macro_rules! relation {
    ($(#[$meta:meta])* $name:ident : $In:ty => $Out:ty,
     apply = $apply:expr, vary = $vary:expr, rewrite = $rewrite:expr $(,)?) => {
        $(#[$meta])*
        struct $name;
        impl $crate::boundary::Relation for $name {
            type In = $In;
            type Out = $Out;
            fn apply(&self, x: &$In) -> $Out {
                ($apply)(x)
            }
            fn vary(&self, x: &$In) -> ::core::option::Option<$In> {
                ($vary)(x)
            }
            fn rewrite(&self, y: $Out) -> $Out {
                ($rewrite)(y)
            }
        }
    };
}

/// Declare an edge's TIME and SPACE cost carriers in one grammar line instead of two
/// near-identical `Graded` impls. The carrier types are the information; the
/// `impl Graded<TimeCost>/<SpaceCost>` scaffolding is repetition:
///
/// ```ignore
/// cost!(Eval, time = CostCons<Nodes, S<Z>, CostNil>, space = CostCons<Depth, S<Z>, CostNil>);
/// ```
#[macro_export]
macro_rules! cost {
    ($edge:ty, time = $time:ty, space = $space:ty $(,)?) => {
        impl $crate::boundary::Graded<$crate::boundary::TimeCost> for $edge {
            type Carrier = $time;
        }
        impl $crate::boundary::Graded<$crate::boundary::SpaceCost> for $edge {
            type Carrier = $space;
        }
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

/// The capability an INPUT type grants by its very shape — the capability's STATE FLOOR,
/// INFERRED from the input type rather than declared. An edge consuming a state-carrying value
/// object (`Bound`, which holds an `Env`) is at least `Stateful` whatever its body does, so a
/// demand (`run_pure` / `run_within<Ceiling>`) checks the input's own effect against the ceiling
/// too — not just the edge's *declared* `Capability`. That closes the dangerous case structurally:
/// a `Pure`-declared edge that secretly reads state cannot be demanded pure, because its INPUT
/// betrays the power the annotation hid. (An annotation can still over-claim — caught behaviourally
/// by the `capability` audit — and `Effectful`/I/O stays invisible to types, also the audit's job;
/// but the under-claim, the hidden dependency the type system would otherwise trust, is now a
/// compile error keyed on structure.) Each value object declares its input effect, default `Pure`.
pub trait InputEffect {
    /// The least capability that consuming `Self` grants. `Pure` for an ordinary value object.
    type Eff: Effect;
}

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
    // THE STATE FLOOR: the input's own inferred effect must also fit the ceiling, so an edge that
    // UNDER-declares (claims `Pure` over a stateful input) is rejected here on structure, not on
    // its (lie-able) annotation.
    M::In: InputEffect,
    <M::In as InputEffect>::Eff: AtMost<Ceiling>,
{
    m.forward(input)
}

/// `run_within` with the ceiling fixed at `Pure` — accepts only effect-free edges, so a
/// lossy, stateful, or effectful edge is rejected at compile time.
pub fn run_pure<M: Morphism>(m: &M, input: &M::In) -> (M::Out, M::Residual)
where
    M::Capability: AtMost<Pure>,
    M::In: InputEffect,
    <M::In as InputEffect>::Eff: AtMost<Pure>,
{
    run_within::<Pure, M>(m, input)
}

// ===== capability-lattice LAWS, proven at compile time =====================
//
// `Join` is a hand-written 4x4 table of associated types — it has NO runtime body, so the
// mutation sweep (which mutates fn bodies) cannot reach it: a mistyped cell, e.g. an
// asymmetric or non-maximal join, would compute a wrong effect ceiling SILENTLY. The
// `typewit::TypeEq` witnesses below close that gap. Each is built with `TypeEq::NEW`, which the
// compiler accepts ONLY if its two type arguments are literally the same type — so this section
// COMPILING is a proof that `Join` is a commutative, idempotent semilattice with `Pure` as
// identity, exhaustive over the closed, sealed 4-effect set. Flip any `=> Out` in `join_impls!`
// and the corresponding witness stops compiling. (This makes `typewit` part of the trust root,
// alongside rustc — a tiny price for a law the mutation gate cannot otherwise certify.)

/// A carried compile-time proof that `Join` COMMUTES on `(Self, B)`:
/// `<Self as Join<B>>::Out == <B as Join<Self>>::Out`. The impls are the proof; the witness is
/// public so a consumer can `to_right`-coerce a composed capability between join orders (see
/// the `join_law` example) — the GADT power `TypeEq` adds over a bare equality assertion.
pub trait JoinCommutes<B>: Join<B>
where
    Self: Sized,
    B: Join<Self>,
{
    /// Built with `TypeEq::NEW` — this impl type-checks only if the two join orders agree.
    const COMMUTES: TypeEq<<Self as Join<B>>::Out, <B as Join<Self>>::Out>;
}
macro_rules! prove_commutes {
    ($a:ty => $($b:ty),+ $(,)?) => {$(
        impl JoinCommutes<$b> for $a {
            const COMMUTES: TypeEq<<$a as Join<$b>>::Out, <$b as Join<$a>>::Out> = TypeEq::NEW;
        }
    )+};
}
prove_commutes!(Pure => Pure, Lossy, Stateful, Effectful);
prove_commutes!(Lossy => Pure, Lossy, Stateful, Effectful);
prove_commutes!(Stateful => Pure, Lossy, Stateful, Effectful);
prove_commutes!(Effectful => Pure, Lossy, Stateful, Effectful);

/// Each `const _` is a compile-time assertion that `<A as Join<B>>::Out` is exactly `Expected`.
macro_rules! prove_join_eq {
    ($($a:ty, $b:ty => $expected:ty);+ $(;)?) => {$(
        const _: TypeEq<<$a as Join<$b>>::Out, $expected> = TypeEq::NEW;
    )+};
}
// IDENTITY: `Pure` is the lattice identity — `Pure ⊔ A == A` and `A ⊔ Pure == A`.
prove_join_eq!(
    Pure, Lossy => Lossy; Pure, Stateful => Stateful; Pure, Effectful => Effectful;
    Lossy, Pure => Lossy; Stateful, Pure => Stateful; Effectful, Pure => Effectful;
);
// IDEMPOTENCE: `A ⊔ A == A`.
prove_join_eq!(
    Pure, Pure => Pure; Lossy, Lossy => Lossy;
    Stateful, Stateful => Stateful; Effectful, Effectful => Effectful;
);
// ASSOCIATIVITY: `(A⊔B)⊔C == A⊔(B⊔C)` over all 64 triples, generated by a cartesian macro over
// the closed effect set. Each triple is its own `TypeEq` witness; a non-associative cell fails.
macro_rules! prove_assoc {
    ([$($t:ty),+ $(,)?]) => { prove_assoc!(@a [$($t),+] [$($t),+]); };
    (@a [$a:ty $(, $arest:ty)*] [$($l:ty),+]) => {
        prove_assoc!(@b [$a] [$($l),+] [$($l),+]);
        prove_assoc!(@a [$($arest),*] [$($l),+]);
    };
    (@a [] [$($l:ty),+]) => {};
    (@b [$a:ty] [$b:ty $(, $brest:ty)*] [$($l:ty),+]) => {
        prove_assoc!(@c [$a] [$b] [$($l),+]);
        prove_assoc!(@b [$a] [$($brest),*] [$($l),+]);
    };
    (@b [$a:ty] [] [$($l:ty),+]) => {};
    (@c [$a:ty] [$b:ty] [$c:ty $(, $crest:ty)*]) => {
        const _: TypeEq<
            <<$a as Join<$b>>::Out as Join<$c>>::Out,
            <$a as Join<<$b as Join<$c>>::Out>>::Out,
        > = TypeEq::NEW;
        prove_assoc!(@c [$a] [$b] [$($crest),*]);
    };
    (@c [$a:ty] [$b:ty] []) => {};
}
prove_assoc!([Pure, Lossy, Stateful, Effectful]);

// TYPE↔VALUE CONSISTENCY: the type-level `Join` reflects to the SAME `Capability` the RUNTIME
// `Capability::join` computes from the reflected values — so the two parallel definitions (the
// `join_impls!` type table and the `Capability::join` const fn) cannot drift apart. A `const`
// assertion (the const-eval-as-typecheck technique; evaluated because it is a const item). This
// is the strongest single law — it pins every cell to the ground-truth rank order — but it is
// rank-RELATIVE (it trusts `rank`), so the pure type-level `typewit` laws above still earn their
// keep by checking the table's algebraic structure independently of any runtime value.
macro_rules! prove_join_matches_runtime {
    ($a:ty => $($b:ty),+ $(,)?) => {$(
        const _: () = {
            assert!(
                <<$a as Join<$b>>::Out as Effect>::VALUE.rank()
                    == <$a as Effect>::VALUE.join(<$b as Effect>::VALUE).rank()
            );
        };
    )+};
}
prove_join_matches_runtime!(Pure => Pure, Lossy, Stateful, Effectful);
prove_join_matches_runtime!(Lossy => Pure, Lossy, Stateful, Effectful);
prove_join_matches_runtime!(Stateful => Pure, Lossy, Stateful, Effectful);
prove_join_matches_runtime!(Effectful => Pure, Lossy, Stateful, Effectful);

// AtMost <-> Join CONSISTENCY: `AtMost` is the lattice ORDER and `Join` is its least upper
// bound, so the two must agree — every operand is `AtMost` the join (`A ⊑ A⊔B` and `B ⊑ A⊔B`),
// and the order is reflexive. `AtMost` is a bare marker trait (no associated type for `TypeEq`
// to compare), so the witness is a const that selects an impl GUARDED by the bound: referencing
// `OrderWitness::<A, C>::OK` forces `A: AtMost<C>` at compile time, and a missing edge of the
// order makes this section fail to build. Together with the semilattice laws above, this pins
// `AtMost` and `Join` to the SAME partial order — a `run_pure` ceiling check and a `Compose`
// ceiling computation cannot disagree about what "at most" means.
struct OrderWitness<A, C>(PhantomData<(A, C)>);
impl<A: AtMost<C>, C> OrderWitness<A, C> {
    const OK: () = ();
}
macro_rules! prove_atmost_join {
    ($a:ty => $($b:ty),+ $(,)?) => {$(
        // reflexivity, and both operands below their join.
        const _: () = OrderWitness::<$a, $a>::OK;
        const _: () = OrderWitness::<$a, <$a as Join<$b>>::Out>::OK;
        const _: () = OrderWitness::<$b, <$a as Join<$b>>::Out>::OK;
    )+};
}
prove_atmost_join!(Pure => Pure, Lossy, Stateful, Effectful);
prove_atmost_join!(Lossy => Pure, Lossy, Stateful, Effectful);
prove_atmost_join!(Stateful => Pure, Lossy, Stateful, Effectful);
prove_atmost_join!(Effectful => Pure, Lossy, Stateful, Effectful);

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

/// Concatenate two type-level lineages: `<Newer as AppendLineage<Older>>::Out` is `Older`'s
/// history followed by `Newer`'s (recursing on `Newer`, substituting its `Origin` base with
/// `Older`). This is the TYPE-level analogue of the runtime `Provenance::combine` — it lets two
/// values' provenances join into one path at compile time, the provenance twin of `AppendCost`.
///
/// It is a bodiless type function (associated type only), so the mutation sweep cannot reach it;
/// the `provenance_laws` proofs below certify it is an associative monoid with `Origin` as the
/// two-sided identity, AND `lineage_append_is_homomorphism` certifies it agrees with the runtime
/// `combine` (`reflect(append(N, O)) == reflect(O) ++ reflect(N)`), so the type-level path and
/// the `Provenance` it reflects to cannot drift.
pub trait AppendLineage<Older> {
    type Out;
}
impl<Older> AppendLineage<Older> for Origin {
    type Out = Older;
}
impl<Edge, Rest, Older> AppendLineage<Older> for Step<Edge, Rest>
where
    Rest: AppendLineage<Older>,
{
    type Out = Step<Edge, <Rest as AppendLineage<Older>>::Out>;
}

// ----- provenance composition LAWS, proven at compile time -----------------
mod provenance_laws {
    use super::{AppendLineage, Origin, Step};
    use typewit::TypeEq;

    // Proof-only edge markers (any distinct types; `reflect` reads their `type_name`).
    struct A;
    struct B;

    // Representative lineages of length 0..2 (newest edge outermost, as `stamp` builds them).
    type L0 = Origin;
    type L1 = Step<A, Origin>;
    type L2 = Step<B, Step<A, Origin>>;

    // `Step<E, _>` as a type-level function, to lift a `TypeEq` over lineage tails (the inductive
    // step for the lineage monoid — the provenance twin of `cost_laws::CostConsFn`).
    typewit::type_fn! {
        struct StepFn<E>;
        impl<T> T => Step<E, T>
    }

    // LEFT IDENTITY is definitional (`<Origin as AppendLineage<L>>::Out` IS `L`); the laws that
    // need induction are proven TOTAL below — for every lineage, not the `L0..L2` sample.

    /// `AppendLineage` is associative for EVERY triple of lineages (total). Induction on the first
    /// (`Origin` definitional, `Step` projects the IH through `StepFn`).
    #[allow(clippy::type_complexity)]
    pub trait AppendLineageAssoc<B_, A_>
    where
        Self: AppendLineage<B_>,
        <Self as AppendLineage<B_>>::Out: AppendLineage<A_>,
        B_: AppendLineage<A_>,
        Self: AppendLineage<<B_ as AppendLineage<A_>>::Out>,
    {
        const EQ: TypeEq<
            <<Self as AppendLineage<B_>>::Out as AppendLineage<A_>>::Out,
            <Self as AppendLineage<<B_ as AppendLineage<A_>>::Out>>::Out,
        >;
    }
    impl<B_, A_> AppendLineageAssoc<B_, A_> for Origin
    where
        B_: AppendLineage<A_>,
    {
        // both sides normalize to `<B_ as AppendLineage<A_>>::Out`.
        const EQ: TypeEq<
            <<Origin as AppendLineage<B_>>::Out as AppendLineage<A_>>::Out,
            <Origin as AppendLineage<<B_ as AppendLineage<A_>>::Out>>::Out,
        > = TypeEq::NEW;
    }
    #[allow(clippy::type_complexity)]
    impl<E, Rest, B_, A_> AppendLineageAssoc<B_, A_> for Step<E, Rest>
    where
        Rest: AppendLineageAssoc<B_, A_>,
        Rest: AppendLineage<B_>,
        <Rest as AppendLineage<B_>>::Out: AppendLineage<A_>,
        B_: AppendLineage<A_>,
        Rest: AppendLineage<<B_ as AppendLineage<A_>>::Out>,
    {
        const EQ: TypeEq<
            <<Step<E, Rest> as AppendLineage<B_>>::Out as AppendLineage<A_>>::Out,
            <Step<E, Rest> as AppendLineage<<B_ as AppendLineage<A_>>::Out>>::Out,
        > = Rest::EQ.project::<StepFn<E>>();
    }

    /// `AppendLineage<L, Origin> == L` for every lineage (right identity, total). Induction on `L`.
    pub trait AppendLineageRightId
    where
        Self: AppendLineage<Origin>,
    {
        const EQ: TypeEq<<Self as AppendLineage<Origin>>::Out, Self>;
    }
    impl AppendLineageRightId for Origin {
        const EQ: TypeEq<<Origin as AppendLineage<Origin>>::Out, Origin> = TypeEq::NEW;
    }
    impl<E, Rest> AppendLineageRightId for Step<E, Rest>
    where
        Rest: AppendLineageRightId + AppendLineage<Origin>,
    {
        const EQ: TypeEq<<Step<E, Rest> as AppendLineage<Origin>>::Out, Step<E, Rest>> =
            Rest::EQ.project::<StepFn<E>>();
    }

    // Force instantiation (smoke-test the induction; the generic impls prove it for all lineages).
    const _: () = {
        let _ = <L2 as AppendLineageAssoc<L1, L0>>::EQ;
        let _ = <L2 as AppendLineageRightId>::EQ;
    };

    // HOMOMORPHISM (type-level monoid -> runtime `Provenance` monoid): reflecting an appended
    // lineage equals combining the reflections. This is the one provenance law with a runtime
    // body (`reflect`/`combine`), so it is a `#[test]` (mutation-reachable) rather than a
    // `TypeEq` — it ties the two monoids together so neither can drift from the other.
    #[cfg(test)]
    #[test]
    fn lineage_append_is_homomorphism() {
        use super::{Lineage, Monoid};
        fn check<N, O>()
        where
            N: Lineage + AppendLineage<O>,
            O: Lineage,
            <N as AppendLineage<O>>::Out: Lineage,
        {
            let appended = <<N as AppendLineage<O>>::Out as Lineage>::reflect();
            let combined = O::reflect().combine(N::reflect());
            assert_eq!(appended, combined);
        }
        check::<L0, L0>();
        check::<L1, L0>();
        check::<L0, L2>();
        check::<L2, L1>();
        check::<L1, L2>();
    }
}

/// A value tagged with its type-level lineage `Path`. Its TYPE records every edge it
/// has crossed, so a consumer can demand a specific provenance (`Stamped<Step<ConstFold,
/// Origin>, _>`) and a value with the wrong history will not compile.
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

// ===== degrees of freedom: completeness as a COMPILE-TIME obligation =======
//
// A value object's degrees of freedom are a type-level set (`DofCons`/`DofNil`); a probe
// declares which it `Covers`, `CoversAll` recurses the set, and `require_complete` makes
// covering ALL of them a BOUND — an incomplete probe fails to COMPILE.
//
// The DOF SET is now DERIVED, not hand-declared: `#[derive(Shaped)]` emits `HasDofs` with
// one `Field<T, I>` marker per variant/field, and `Complete<T>` covers every `Field<T, I>`
// by construction. So the COMPLETE probe is generated — you cannot forget a dimension or
// under-specify it; the only thing left to get wrong (a hand-written PARTIAL `Covers` set)
// is still rejected (`tests/compile_fail/incomplete_probe_rejected`). The actual RUNTIME
// probing is the fused `Shaped` probe above, which is likewise complete by construction —
// so the per-dimension perturbation is derived, never hand-written.

/// The empty type-level DOF set — a probe covers it vacuously.
pub struct DofNil;
/// A cons cell of a type-level DOF set: head DOF `H`, tail set `T`.
pub struct DofCons<H, T>(PhantomData<(H, T)>);

/// The `I`-th DERIVED degree of freedom of value object `T` (one per variant / field).
/// Generated by `#[derive(Shaped)]`, so the DOF set is mechanical — there is no
/// hand-written marker to forget or mis-name.
pub struct Field<T, const I: usize>(PhantomData<T>);

/// A value object's degrees of freedom as a type-level set, so completeness is a bound
/// rather than enumerated by hand. DERIVED by `#[derive(Shaped)]`.
pub trait HasDofs {
    /// The type-level DOF set (a `DofCons`/`DofNil` list of `Field<Self, _>`).
    type Dofs;
}

/// A probe that can SEE the degree of freedom `D` — the per-dimension witness a complete
/// probe must exhibit.
pub trait Covers<D> {}

/// A probe that covers EVERY DOF in the type-level set `L` — the completeness relation, by
/// recursion. No overlap: `DofNil` and `DofCons` are disjoint head constructors.
pub trait CoversAll<L> {}
impl<C> CoversAll<DofNil> for C {}
impl<C, H, T> CoversAll<DofCons<H, T>> for C
where
    C: Covers<H>,
    C: CoversAll<T>,
{
}

/// The DERIVED complete probe for `T`: it `Covers` every `Field<T, I>` by construction, so
/// `require_complete::<T, Complete<T>>()` always type-checks — completeness is generated,
/// not asserted. (The hand-written path still exists as the backstop: a partial `Covers`
/// set is rejected, which `incomplete_probe_rejected` pins.)
pub struct Complete<T>(PhantomData<T>);
impl<T, const I: usize> Covers<Field<T, I>> for Complete<T> {}

/// The completeness obligation as a BOUND: type-checks only when `C` covers every DOF `T`
/// declares. With the DOF set derived and `Complete<T>` covering it by construction, the
/// normal path cannot be incomplete; a hand-written probe that omits a dimension still
/// fails to compile — the push-back a coding agent gets before any test runs.
pub fn require_complete<T, C>(_check: &C)
where
    T: HasDofs,
    C: CoversAll<<T as HasDofs>::Dofs>,
{
}

/// The DERIVED-completeness statement: `Complete<T>` covers every DOF `T` declares. It takes
/// no probe value — completeness is generated, so there is nothing to pass in and nothing to
/// get wrong. `assert_complete::<Expr>()` is the positive twin of `incomplete_probe_rejected`.
pub fn assert_complete<T>()
where
    T: HasDofs,
    Complete<T>: CoversAll<<T as HasDofs>::Dofs>,
{
}

// ===== edge completeness: every boundary edge must carry a probe ===========
//
// `Complete<T>` makes DOF-completeness SOUND because `derive(Shaped)` reads the DOF list from
// the type's ONE definition — list and reality are the same source, so they cannot diverge.
// Edges have NO single definition site (each is a separate `impl`), so the same idea needs the
// edge set declared once: the `edges!` macro. Over that single-source list, `AllProbed` makes
// "every edge is probed" a BOUND — an edge in the list that does not impl `Probed` fails to
// COMPILE, the same push-back a missing DOF gives, before any test runs. The open-world residue
// — a production edge `impl` written OUTSIDE the `edges!` list — is now CLOSED by `build.rs`,
// which enumerates every concrete edge impl in the source and rejects any without an
// `impl Probed` (a build error). The type system cannot enumerate impls; the build step can, so
// the two together make edge-completeness TOTAL: the list gives the in-language bound, the
// enumeration catches anything left off it. (Counterexamples/fixtures are `#[cfg(test)]`, so the
// enumeration skips them — they are not spec edges.)

/// An edge covered by a probe. Impling `Probed` means WRITING the probe (`fn probe`), so it
/// cannot be claimed without one; the probe's STRENGTH is then audited by mutation, exactly as
/// a declared `Capability` is audited against behaviour. `edges!` + `assert_all_probed` force
/// every declared edge to discharge this, so a new edge with no probe is a build error rather
/// than a surviving mutant discovered later.
///
/// The `#[diagnostic::on_unimplemented]` attribute (stable since 1.78) shapes the failure so a
/// forgotten probe reads as a spec violation, not a raw trait-bound error.
#[diagnostic::on_unimplemented(
    message = "boundary edge `{Self}` has no probe",
    label = "this edge is declared in `edges!` but has no `impl Probed`",
    note = "every boundary edge must carry a probe: add `impl Probed for {Self} {{ fn probe() {{ /* the edge's probe */ }} }}`",
    note = "a probe omitted here would otherwise surface only later, as a surviving mutant"
)]
pub trait Probed {
    /// Run this edge's probe.
    fn probe();
}

/// The empty type-level edge set.
pub struct EdgeNil;
/// A cons cell of a type-level edge set: head edge `E`, tail set `Rest`.
pub struct EdgeCons<E, Rest>(PhantomData<(E, Rest)>);

/// Every edge in the set `L` impls `Probed` — completeness over edges, by recursion (disjoint
/// head constructors, like `CoversAll` for DOFs).
pub trait AllProbed {}
impl AllProbed for EdgeNil {}
// NB: do NOT add `#[diagnostic::do_not_recommend]` here — on this recursive cons impl it
// SUPPRESSES the recursion into the `E: Probed` requirement, which discards the helpful
// `Probed` on_unimplemented message and degrades the error to a generic `AllProbed` failure.
// (do_not_recommend helps for flat blanket impls; it backfires for recursive machinery.)
impl<E: Probed, Rest: AllProbed> AllProbed for EdgeCons<E, Rest> {}

/// The edge-completeness statement: every edge in `L` is `Probed`. The edge analog of
/// `assert_complete`. `assert_all_probed::<edges!(Parse, Check, Eval)>()` type-checks only if
/// each edge impls `Probed`; a forgotten probe fails to compile
/// (`tests/compile_fail/forgotten_probe_rejected`).
pub fn assert_all_probed<L: AllProbed>() {}

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

// ===== the FUSED universal probe: structure derives the probe surface =====
//
// The probes above each target ONE bug class with a HAND-WRITTEN perturbation. `Shaped`
// derives the perturbation surface from a value object's own STRUCTURE, and the fused probe
// below collapses the structural, value, and semantic layers into a SINGLE operator: a map
// is faithful iff it responds to every derived degree of freedom. `#[derive(Shaped)]`
// (in `boundary-spec-macros`) generates it for composites; leaves with smart-constructor
// invariants impl it by hand.

/// A value object whose probe surface is its STRUCTURE. `inhabitant` is one canonical seed
/// (so generation needs no argument); `perturbation_classes` returns one neighbour-GROUP
/// per degree of freedom — the variant choice (STRUCTURE) and each field (VALUE / deeper
/// SEMANTICS). Derivable: `#[derive(Shaped)]` reads the variants and fields.
pub trait Shaped: Sized + Clone + PartialEq {
    /// One canonical inhabitant — a LEAF variant (the first whose fields don't mention the
    /// type itself, so the recursion bottoms out) / the struct of field inhabitants.
    fn inhabitant() -> Self;
    /// Neighbours grouped by degree of freedom (one group per variant-choice and per field).
    fn perturbation_classes(&self) -> Vec<Vec<Self>>;
    /// All neighbours, flattened across dimensions (what a field threads back upward).
    fn all_perturbations(&self) -> Vec<Self> {
        self.perturbation_classes().into_iter().flatten().collect()
    }
    /// The STRUCTURAL slice of the perturbation surface: neighbours that change WHICH
    /// constructor shape the value has (a variant swap, here or in any field), never how a
    /// leaf quantity is tuned. This is the CHEAP, type-level-finite partition of the space —
    /// for a non-recursive type its closure is a small finite set — so `shadow_grid` closes
    /// over it EXHAUSTIVELY before spending any budget on value neighbours. Derived by
    /// `#[derive(Shaped)]`; the default is empty (a leaf has no structural degree of
    /// freedom), which hand-written leaf impls inherit.
    fn structural_perturbations(&self) -> Vec<Self> {
        Vec::new()
    }
}

/// The grid a `Shaped` type GROWS from its own structure (the inhabitant closed under
/// perturbation, structure first, capped) — re-exported here because it is boundary
/// vocabulary in practice: a consumer's `Probed` impls and probe tests legitimately want the
/// derived grid, and `discover::engine::` reads engine-internal for what is really Shaped's
/// closure. `grid_gaps` is its audit: the one-step-reachable constructors a grid failed to
/// exhibit (empty for any completed closure).
pub use crate::discover::engine::{grid_gaps, shadow_grid};

/// A boolean varies one way: to its negation.
impl Shaped for bool {
    fn inhabitant() -> Self {
        false
    }
    fn perturbation_classes(&self) -> Vec<Vec<Self>> {
        vec![vec![!*self]]
    }
    /// A bool IS a two-variant sum, so its one degree of freedom is the variant choice —
    /// the negation is structural, not a value tune.
    fn structural_perturbations(&self) -> Vec<Self> {
        vec![!*self]
    }
}

/// A boxed value is shaped exactly as its contents (the box is transparent to probing).
impl<T: Shaped> Shaped for Box<T> {
    fn inhabitant() -> Self {
        Box::new(T::inhabitant())
    }
    fn perturbation_classes(&self) -> Vec<Vec<Self>> {
        (**self)
            .perturbation_classes()
            .into_iter()
            .map(|group| group.into_iter().map(Box::new).collect())
            .collect()
    }
    fn structural_perturbations(&self) -> Vec<Self> {
        (**self)
            .structural_perturbations()
            .into_iter()
            .map(Box::new)
            .collect()
    }
}

/// The FUSED universal probe: a map is FAITHFUL at `x` iff it is SENSITIVE to every degree
/// of freedom — for each derived perturbation class (the variant choice = STRUCTURE, each
/// field = VALUE / deeper SEMANTICS) SOME neighbour changes the output. One operator,
/// derived from `Shaped`, that fuses the three probe layers: a map silently collapsing ANY
/// dimension fails, because that dimension's class has no responding neighbour. Sensitivity
/// is PER-CLASS, not global injectivity, so a map MAY identify genuinely-equivalent inputs
/// within a class (e.g. eval and commutativity) and still be sensitive to the dimension.
/// (An empty class — a dimension with no neighbours — imposes no obligation.)
pub fn sensitive_to_all<T, Y>(map: impl Fn(&T) -> Y, x: &T) -> bool
where
    T: Shaped,
    Y: PartialEq,
{
    let fx = map(x);
    x.perturbation_classes()
        .into_iter()
        .filter(|class| !class.is_empty())
        .all(|class| class.iter().any(|n| map(n) != fx))
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

// ===== cost-grading LAWS, proven at compile time ==========================
//
// The cost grading is built entirely from type-level FUNCTIONS with no runtime body — `Max`,
// `NatEq`, `Le`, `Select` over the Peano degrees, and `Lookup` / `AppendCost` over the cost
// map. Exactly like the capability `Join` table, the mutation sweep mutates fn bodies, so it
// cannot reach any of these: a wrong cell (a `Max` that took the MIN, a `Lookup` that summed
// instead of maxed) would mis-state a path's complexity SILENTLY, and `WithinBudget` would
// then certify a budget against a lie. The `typewit::TypeEq` witnesses below close that gap the
// same way `JoinCommutes` & co. do for capability: each compiles only if its two type arguments
// are literally one type, so this section COMPILING is a proof of the cost algebra's laws.
//
// The Peano nats are an OPEN set (unlike the sealed 4-effect lattice). Rather than spot-check the
// laws over a finite sample, the core laws are proven TOTAL by STRUCTURAL INDUCTION: each is a
// trait carrying a `TypeEq`, whose `Z` impl is the base case and whose `S<N>` impl derives the
// witness from `N`'s — lifting it through the injective successor `SFn` with the const `project`.
// Rust type-checks the generic impl bodies at definition, so the law holds for EVERY degree, not a
// representative few. The `Max` semilattice (idempotence, identity, commutativity, associativity),
// `NatEq` reflexivity, and the `AppendCost` cost-monoid (associativity, identity) are total this
// way. What remains SAMPLED is noted at each site (and why): the load-bearing
// `Lookup`-distributes-over-`AppendCost` law, `NatEq` symmetry, `Le`, and the numeric type<->value
// reflection — each now resting on these total foundations.
mod cost_laws {
    use super::{AppendCost, Axis, CostCons, CostNil, Degree, Le, Lookup, Max, NatEq, True, S, Z};
    use typewit::TypeEq;

    // A representative closed sample of the open degree set: 0, 1, 2, 3.
    type D0 = Z;
    type D1 = S<Z>;
    type D2 = S<S<Z>>;
    type D3 = S<S<S<Z>>>;

    // The successor as an INJECTIVE type-level function, so a `TypeEq<X, Y>` can be lifted to a
    // `TypeEq<S<X>, S<Y>>` in a `const` (`project`). This is what turns a finite spot-check into a
    // genuine induction: the step case wraps the inductive hypothesis through `SFn`.
    typewit::inj_type_fn! {
        struct SFn;
        impl<T> T => S<T>;
    }

    // ----- INDUCTIVE totality: each law holds for EVERY degree, not a sample -----
    // A law is a trait carrying a `TypeEq` witness; its `Z` impl is the base case and its `S<N>`
    // impl derives the witness from `N`'s (the inductive step), so the trait being inhabited for
    // all nats IS the law's proof over the whole open set. `TypeEq::NEW` still does the checking;
    // induction supplies the witness the base case alone could not.

    /// `Max<A, A> == A` for every degree `A` (idempotence, total).
    pub trait MaxIdem
    where
        Self: Max<Self> + Sized,
    {
        const EQ: TypeEq<<Self as Max<Self>>::Out, Self>;
    }
    impl MaxIdem for Z {
        const EQ: TypeEq<<Z as Max<Z>>::Out, Z> = TypeEq::NEW;
    }
    impl<A: MaxIdem> MaxIdem for S<A> {
        // `<S<A> as Max<S<A>>>::Out` normalizes to `S<<A as Max<A>>::Out>`; project the IH.
        const EQ: TypeEq<<S<A> as Max<S<A>>>::Out, S<A>> = A::EQ.project::<SFn>();
    }

    /// `Max<Z, A> == A` for every `A` (left identity, total) — definitional in one blanket impl:
    /// `<Z as Max<A>>::Out` IS `A` by the `Max for Z` base case.
    pub trait MaxLeftId: Sized {
        const EQ: TypeEq<<Z as Max<Self>>::Out, Self>;
    }
    impl<A> MaxLeftId for A {
        const EQ: TypeEq<<Z as Max<A>>::Out, A> = TypeEq::NEW;
    }

    /// `Max<A, Z> == A` for every `A` (right identity, total). Both constructors are definitional
    /// (`<S<A> as Max<Z>>::Out` IS `S<A>`), so no induction is needed — two disjoint impls suffice.
    pub trait MaxRightId
    where
        Self: Max<Z>,
    {
        const EQ: TypeEq<<Self as Max<Z>>::Out, Self>;
    }
    impl MaxRightId for Z {
        const EQ: TypeEq<<Z as Max<Z>>::Out, Z> = TypeEq::NEW;
    }
    impl<A> MaxRightId for S<A> {
        const EQ: TypeEq<<S<A> as Max<Z>>::Out, S<A>> = TypeEq::NEW;
    }

    /// `Max<A, B> == Max<B, A>` for every `A`, `B` (commutativity, total). The three base cases
    /// (some operand `Z`) are definitional, split by constructor so each side normalizes; the
    /// `S`/`S` step projects the IH on the predecessors.
    pub trait MaxComm<B>
    where
        Self: Max<B> + Sized,
        B: Max<Self>,
    {
        const EQ: TypeEq<<Self as Max<B>>::Out, <B as Max<Self>>::Out>;
    }
    impl MaxComm<Z> for Z {
        const EQ: TypeEq<<Z as Max<Z>>::Out, <Z as Max<Z>>::Out> = TypeEq::NEW;
    }
    impl<B> MaxComm<S<B>> for Z {
        // `<Z as Max<S<B>>>::Out == S<B>` and `<S<B> as Max<Z>>::Out == S<B>`.
        const EQ: TypeEq<<Z as Max<S<B>>>::Out, <S<B> as Max<Z>>::Out> = TypeEq::NEW;
    }
    impl<A> MaxComm<Z> for S<A> {
        const EQ: TypeEq<<S<A> as Max<Z>>::Out, <Z as Max<S<A>>>::Out> = TypeEq::NEW;
    }
    impl<A, B> MaxComm<S<B>> for S<A>
    where
        A: MaxComm<B> + Max<B>,
        B: Max<A>,
    {
        // `<S<A> as Max<S<B>>>::Out` is `S<Max<A,B>>`, `<S<B> as Max<S<A>>>::Out` is `S<Max<B,A>>`.
        const EQ: TypeEq<<S<A> as Max<S<B>>>::Out, <S<B> as Max<S<A>>>::Out> =
            A::EQ.project::<SFn>();
    }

    /// `(A ⊔ B) ⊔ C == A ⊔ (B ⊔ C)` for every `A`, `B`, `C` (associativity, total). The three
    /// base cases (the first `Z` encountered at each position) are definitional; the all-`S` step
    /// projects the IH. The trait's `where` clauses are the bounds `Max` needs to even form both
    /// sides; the all-`S` impl restates `B`/`C`'s share of them (Rust does not imply a trait's
    /// `where` clauses at use sites).
    #[allow(clippy::type_complexity)]
    pub trait MaxAssoc<B, C>
    where
        Self: Max<B> + Sized,
        <Self as Max<B>>::Out: Max<C>,
        B: Max<C>,
        Self: Max<<B as Max<C>>::Out>,
    {
        const EQ: TypeEq<
            <<Self as Max<B>>::Out as Max<C>>::Out,
            <Self as Max<<B as Max<C>>::Out>>::Out,
        >;
    }
    impl<B: Max<C>, C> MaxAssoc<B, C> for Z {
        // both sides normalize to `<B as Max<C>>::Out`.
        const EQ: TypeEq<<<Z as Max<B>>::Out as Max<C>>::Out, <Z as Max<<B as Max<C>>::Out>>::Out> =
            TypeEq::NEW;
    }
    impl<A, C> MaxAssoc<Z, C> for S<A>
    where
        S<A>: Max<C>,
    {
        // both sides normalize to `<S<A> as Max<C>>::Out`.
        const EQ: TypeEq<
            <<S<A> as Max<Z>>::Out as Max<C>>::Out,
            <S<A> as Max<<Z as Max<C>>::Out>>::Out,
        > = TypeEq::NEW;
    }
    impl<A, B> MaxAssoc<S<B>, Z> for S<A>
    where
        A: Max<B>,
    {
        // both sides normalize to `S<Max<A,B>>`.
        const EQ: TypeEq<
            <<S<A> as Max<S<B>>>::Out as Max<Z>>::Out,
            <S<A> as Max<<S<B> as Max<Z>>::Out>>::Out,
        > = TypeEq::NEW;
    }
    #[allow(clippy::type_complexity)]
    impl<A, B, C> MaxAssoc<S<B>, S<C>> for S<A>
    where
        A: MaxAssoc<B, C>,
        A: Max<B>,
        <A as Max<B>>::Out: Max<C>,
        B: Max<C>,
        A: Max<<B as Max<C>>::Out>,
    {
        const EQ: TypeEq<
            <<S<A> as Max<S<B>>>::Out as Max<S<C>>>::Out,
            <S<A> as Max<<S<B> as Max<S<C>>>::Out>>::Out,
        > = A::EQ.project::<SFn>();
    }

    /// `NatEq<A, A> == True` for every `A` (reflexivity, total) — the equality `Lookup`'s axis
    /// matching rests on. The step needs no projection: `<S<A> as NatEq<S<A>>>::Out` already
    /// normalizes to `<A as NatEq<A>>::Out`, so the IH witness is reused directly.
    pub trait NatEqRefl
    where
        Self: NatEq<Self> + Sized,
    {
        const EQ: TypeEq<<Self as NatEq<Self>>::Out, True>;
    }
    impl NatEqRefl for Z {
        const EQ: TypeEq<<Z as NatEq<Z>>::Out, True> = TypeEq::NEW;
    }
    impl<A: NatEqRefl> NatEqRefl for S<A> {
        const EQ: TypeEq<<S<A> as NatEq<S<A>>>::Out, True> = A::EQ;
    }

    // `CostCons<Ax, Deg, _>` as a type-level function, to lift a `TypeEq` over cost-map TAILS the
    // same way `SFn` lifts over nat successors — the inductive step for the cost-map monoid laws.
    typewit::type_fn! {
        struct CostConsFn<Ax, Deg>;
        impl<T> T => CostCons<Ax, Deg, T>
    }

    /// `AppendCost` is associative for EVERY triple of maps (total) — the cost monoid `Compose`
    /// threads. Induction on the first map; `CostNil` is definitional, the `CostCons` step projects
    /// the IH through `CostConsFn`.
    #[allow(clippy::type_complexity)]
    pub trait AppendCostAssoc<B, C>
    where
        Self: AppendCost<B>,
        <Self as AppendCost<B>>::Out: AppendCost<C>,
        B: AppendCost<C>,
        Self: AppendCost<<B as AppendCost<C>>::Out>,
    {
        const EQ: TypeEq<
            <<Self as AppendCost<B>>::Out as AppendCost<C>>::Out,
            <Self as AppendCost<<B as AppendCost<C>>::Out>>::Out,
        >;
    }
    impl<B, C> AppendCostAssoc<B, C> for CostNil
    where
        B: AppendCost<C>,
    {
        // both sides normalize to `<B as AppendCost<C>>::Out`.
        const EQ: TypeEq<
            <<CostNil as AppendCost<B>>::Out as AppendCost<C>>::Out,
            <CostNil as AppendCost<<B as AppendCost<C>>::Out>>::Out,
        > = TypeEq::NEW;
    }
    #[allow(clippy::type_complexity)]
    impl<Ax, Deg, Rest, B, C> AppendCostAssoc<B, C> for CostCons<Ax, Deg, Rest>
    where
        Rest: AppendCostAssoc<B, C>,
        Rest: AppendCost<B>,
        <Rest as AppendCost<B>>::Out: AppendCost<C>,
        B: AppendCost<C>,
        Rest: AppendCost<<B as AppendCost<C>>::Out>,
    {
        const EQ: TypeEq<
            <<CostCons<Ax, Deg, Rest> as AppendCost<B>>::Out as AppendCost<C>>::Out,
            <CostCons<Ax, Deg, Rest> as AppendCost<<B as AppendCost<C>>::Out>>::Out,
        > = Rest::EQ.project::<CostConsFn<Ax, Deg>>();
    }

    /// `AppendCost<A, CostNil> == A` for every map (right identity, total; left identity is
    /// definitional). Induction on `A`, projecting the IH through `CostConsFn`.
    pub trait AppendCostRightId
    where
        Self: AppendCost<CostNil>,
    {
        const EQ: TypeEq<<Self as AppendCost<CostNil>>::Out, Self>;
    }
    impl AppendCostRightId for CostNil {
        const EQ: TypeEq<<CostNil as AppendCost<CostNil>>::Out, CostNil> = TypeEq::NEW;
    }
    impl<Ax, Deg, Rest> AppendCostRightId for CostCons<Ax, Deg, Rest>
    where
        Rest: AppendCostRightId + AppendCost<CostNil>,
    {
        const EQ: TypeEq<
            <CostCons<Ax, Deg, Rest> as AppendCost<CostNil>>::Out,
            CostCons<Ax, Deg, Rest>,
        > = Rest::EQ.project::<CostConsFn<Ax, Deg>>();
    }

    // The generic `impl` bodies above ARE the proofs — Rust type-checks them for all nats at
    // definition, so the laws hold over the whole open set. Instantiating at a representative depth
    // forces monomorphization of the recursive chain (a concrete smoke-test of the induction) and
    // marks the proof traits used.
    const _: () = {
        let _ = <D3 as MaxIdem>::EQ;
        let _ = <D3 as MaxLeftId>::EQ;
        let _ = <D3 as MaxRightId>::EQ;
        let _ = <D2 as MaxComm<D3>>::EQ;
        let _ = <D1 as MaxAssoc<D2, D3>>::EQ;
        let _ = <D3 as NatEqRefl>::EQ;
        let _ = <Ma as AppendCostAssoc<Mb, Mc>>::EQ;
        let _ = <Ma as AppendCostRightId>::EQ;
    };

    // --- cartesian expansion helpers: invoke a per-cell callback macro over a type list ---
    macro_rules! for_each {
        ($cb:ident; [$($t:ty),+ $(,)?]) => { $($cb!($t);)+ };
    }
    macro_rules! for_pairs {
        ($cb:ident; [$($t:ty),+ $(,)?]) => { for_pairs!(@a $cb; [$($t),+] [$($t),+]); };
        (@a $cb:ident; [$a:ty $(, $ar:ty)*] [$($l:ty),+]) => {
            for_pairs!(@b $cb; [$a] [$($l),+]);
            for_pairs!(@a $cb; [$($ar),*] [$($l),+]);
        };
        (@a $cb:ident; [] [$($l:ty),+]) => {};
        (@b $cb:ident; [$a:ty] [$b:ty $(, $br:ty)*]) => {
            $cb!($a, $b);
            for_pairs!(@b $cb; [$a] [$($br),*]);
        };
        (@b $cb:ident; [$a:ty] []) => {};
    }

    // The MAX semilattice laws (idempotence, identity, commutativity, associativity) and NATEQ
    // reflexivity are proven TOTAL by the inductive witnesses above — no sample needed. The
    // remaining helper laws below are still spot-checked over the representative sample: NATEQ
    // SYMMETRY and the LE order feed `Lookup`/`WithinBudget` but are not (yet) the load-bearing
    // structure, so they rest on the now-total `Max` foundation rather than re-deriving it.

    // NATEQ symmetry: `NatEq<A, B> == NatEq<B, A>` (decidable-equality symmetry, sampled).
    macro_rules! nateq_symm {
        ($a:ty, $b:ty) => {
            const _: TypeEq<<$a as NatEq<$b>>::Out, <$b as NatEq<$a>>::Out> = TypeEq::NEW;
        };
    }
    for_pairs!(nateq_symm; [D0, D1, D2, D3]);

    // LE is reflexive and consistent with `Max`: every operand is `<=` the max (the lub
    // property that makes `WithinBudget`'s per-axis `Le` check sound).
    macro_rules! le_refl {
        ($a:ty) => {
            const _: TypeEq<<$a as Le<$a>>::Out, True> = TypeEq::NEW;
        };
    }
    macro_rules! le_below_max {
        ($a:ty, $b:ty) => {
            const _: TypeEq<<$a as Le<<$a as Max<$b>>::Out>>::Out, True> = TypeEq::NEW;
            const _: TypeEq<<$b as Le<<$a as Max<$b>>::Out>>::Out, True> = TypeEq::NEW;
        };
    }
    for_each!(le_refl; [D0, D1, D2, D3]);
    for_pairs!(le_below_max; [D0, D1, D2, D3]);

    // TYPE<->VALUE CONSISTENCY: the type-level `Max` reflects (via `Degree::N`) to the SAME
    // number the obvious runtime maximum computes — so the Peano table and the `u32` degree it
    // stands for cannot drift (the const-eval-as-typecheck twin of `prove_join_matches_runtime`).
    const fn const_max(a: u32, b: u32) -> u32 {
        if a >= b {
            a
        } else {
            b
        }
    }
    macro_rules! max_reflects {
        ($a:ty, $b:ty) => {
            const _: () = {
                assert!(
                    <<$a as Max<$b>>::Out as Degree>::N
                        == const_max(<$a as Degree>::N, <$b as Degree>::N)
                );
            };
        };
    }
    for_pairs!(max_reflects; [D0, D1, D2, D3]);

    // --- the cost map laws, over representative maps and query axes ---
    // Proof-only axes (their `Id`s are distinct degrees, exactly as `axis!` would assign).
    struct Ax0;
    struct Ax1;
    struct Ax2;
    impl Axis for Ax0 {
        type Id = D0;
    }
    impl Axis for Ax1 {
        type Id = D1;
    }
    impl Axis for Ax2 {
        type Id = D2;
    }

    // Representative maps, including a DUPLICATE axis (`Ma` lists `Ax0` twice at different
    // degrees) so the per-axis `Max`-fold in `Lookup` is actually exercised.
    type Ma = CostCons<Ax0, D1, CostCons<Ax1, D2, CostCons<Ax0, D3, CostNil>>>;
    type Mb = CostCons<Ax1, D1, CostCons<Ax2, D2, CostNil>>;
    type Mc = CostCons<Ax2, D3, CostCons<Ax0, D1, CostNil>>;

    // APPENDCOST associativity and `CostNil` identity (the cost monoid `Compose` threads for both
    // `TimeCost` and `SpaceCost`) are proven TOTAL above by `AppendCostAssoc`/`AppendCostRightId`
    // — for every map, not this sample.

    // THE LOAD-BEARING LAW: `Lookup<Q>` distributes over `AppendCost` as `Max` —
    //   lookup(append(A, B), Q) == max(lookup(A, Q), lookup(B, Q))
    // for every query axis. This is exactly why sequential composition is plain append with no map
    // merge: the per-axis max is recovered at lookup time. Still SAMPLED (over `Ma`/`Mb` and each
    // axis): its total proof needs decidable-equality reflection — case-splitting `Lookup`'s
    // `Select` on `NatEq`, invoking the now-total `MaxAssoc` in the matching branch — which the
    // total `Max`/`AppendCost` foundations above now make a tractable next step. It rests on those
    // total foundations rather than re-deriving them.
    macro_rules! lookup_append_is_max {
        ($q:ty) => {
            const _: TypeEq<
                <<Ma as AppendCost<Mb>>::Out as Lookup<$q>>::Deg,
                <<Ma as Lookup<$q>>::Deg as Max<<Mb as Lookup<$q>>::Deg>>::Out,
            > = TypeEq::NEW;
        };
    }
    lookup_append_is_max!(Ax0);
    lookup_append_is_max!(Ax1);
    lookup_append_is_max!(Ax2);
}

// ===== composition: loss composes as a value object ======================

/// Product of two residuals — loss COMPOSES as a value object.
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<A, B>(pub A, pub B);
impl<A: ValueObject, B: ValueObject> citizen::Registered for Pair<A, B> {}
impl<A: ValueObject, B: ValueObject> ValueObject for Pair<A, B> {}

/// Sequential composition `g ∘ f` as a single morphism. Its residual is the
/// PRODUCT of the two residuals; retaining it keeps the composite invertible.
/// End-to-end invertibility flows THROUGH a lossy stage as long as its residual
/// is kept — loss only blocks propagation when the residual is DISCARDED.
pub struct Compose<F, G> {
    pub f: F,
    pub g: G,
}
impl<F, G> citizen::Registered for Compose<F, G> {}
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

// ===== residual monoid LAWS — the ONE grading proven UP TO ISOMORPHISM =====
//
// Residual is the OUTLIER among the four gradings. Capability (`Join`) and cost (`Max` /
// `AppendCost`) are STRICT type-level functions, so their monoid laws are exact type EQUALITIES
// a `TypeEq` can witness. Residual's combine is `Pair`, a genuine PRODUCT type — and a product
// is a monoid only UP TO ISOMORPHISM: `Pair<Unit, R>` is not literally `R`, and
// `Pair<Pair<A, B>, C>` is not literally `Pair<A, Pair<B, C>>`. So the strict-equality technique
// does NOT apply here — an honest finding, not a gap. The laws hold up to a canonical iso, and
// the witnesses below MAKE that iso executable: total, generic conversions whose round-trips are
// the unit and associativity laws. Unlike the type-level proofs, these have runtime bodies, so
// the mutation sweep judges them directly — and being fully generic (no `Default` to fabricate a
// return from), each admits no viable mutant but the correct one. The `residual_iso_laws`
// proptest exercises the round-trips so the laws are checked, not merely asserted.

/// Drop a left `Unit` residual: `Pair<Unit, R> ≅ R` (left identity), forward.
pub fn drop_unit_l<R>(p: Pair<Unit, R>) -> R {
    p.1
}
/// Re-introduce a left `Unit` residual (the inverse of `drop_unit_l`).
pub fn add_unit_l<R>(r: R) -> Pair<Unit, R> {
    Pair(Unit, r)
}
/// Drop a right `Unit` residual: `Pair<R, Unit> ≅ R` (right identity), forward.
pub fn drop_unit_r<R>(p: Pair<R, Unit>) -> R {
    p.0
}
/// Re-introduce a right `Unit` residual (the inverse of `drop_unit_r`).
pub fn add_unit_r<R>(r: R) -> Pair<R, Unit> {
    Pair(r, Unit)
}
/// Re-associate a composite residual: `Pair<Pair<A, B>, C> ≅ Pair<A, Pair<B, C>>`, forward —
/// the associativity iso. Lets a consumer normalize the residual of a left-nested `Compose`
/// chain into right-nested form without touching the carried data.
pub fn reassoc_residual<A, B, C>(p: Pair<Pair<A, B>, C>) -> Pair<A, Pair<B, C>> {
    let Pair(Pair(a, b), c) = p;
    Pair(a, Pair(b, c))
}
/// The inverse re-association (`Pair<A, Pair<B, C>> -> Pair<Pair<A, B>, C>`).
pub fn reassoc_residual_back<A, B, C>(p: Pair<A, Pair<B, C>>) -> Pair<Pair<A, B>, C> {
    let Pair(a, Pair(b, c)) = p;
    Pair(Pair(a, b), c)
}

#[cfg(test)]
mod residual_iso_laws {
    use super::{
        add_unit_l, add_unit_r, drop_unit_l, drop_unit_r, reassoc_residual, reassoc_residual_back,
        Pair, Unit,
    };
    use proptest::prelude::*;

    proptest! {
        /// LEFT IDENTITY iso: `drop_unit_l` and `add_unit_l` are mutually inverse.
        #[test]
        fn unit_l_is_an_iso(r in any::<i32>()) {
            prop_assert_eq!(drop_unit_l(add_unit_l(r)), r);
            let p = Pair(Unit, r);
            prop_assert_eq!(add_unit_l(drop_unit_l(p.clone())), p);
        }

        /// RIGHT IDENTITY iso: `drop_unit_r` and `add_unit_r` are mutually inverse.
        #[test]
        fn unit_r_is_an_iso(r in any::<i32>()) {
            prop_assert_eq!(drop_unit_r(add_unit_r(r)), r);
            let p = Pair(r, Unit);
            prop_assert_eq!(add_unit_r(drop_unit_r(p.clone())), p);
        }

        /// ASSOCIATIVITY iso: re-association round-trips in both directions.
        #[test]
        fn reassoc_is_an_iso(a in any::<i32>(), b in any::<u8>(), c in any::<bool>()) {
            let left = Pair(Pair(a, b), c);
            prop_assert_eq!(reassoc_residual_back(reassoc_residual(left.clone())), left);
            let right = Pair(a, Pair(b, c));
            prop_assert_eq!(reassoc_residual(reassoc_residual_back(right.clone())), right);
        }
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
impl<C, M> citizen::Registered for Then<C, M> {}
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
impl<M, T> citizen::Registered for Profiled<M, T> {}
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
