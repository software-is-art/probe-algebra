//! join_law — a `typewit` prototype: lift a LAW about the type-level capability lattice into a
//! COMPILE-TIME proof (research open problems #3/#4 — relational proofs beyond single values).
//!
//! `Join` (the capability composition rule) is a hand-written 4×4 table. That it is
//! COMMUTATIVE — `<A as Join<B>>::Out == <B as Join<A>>::Out` — is currently only *trusted* (the
//! author wrote symmetric entries; a mistyped cell would compute a wrong ceiling silently). Here
//! each pair gets a `typewit::TypeEq` witness built with `TypeEq::NEW`, which the compiler
//! accepts ONLY if the two `Out` types are literally the same. So this file COMPILING *is* a
//! proof of commutativity — exhaustive over the closed 4-element capability set (sealed
//! `Effect`); an asymmetric table entry would fail to build.
//!
//! Unlike a plain identity-coercion equality check, the `TypeEq` is a first-class VALUE:
//! `recompose` CARRIES the witness and USES it (`to_right`) to coerce a capability between the
//! two join orders in GENERIC code — the GADT-style power typewit adds over phantom equality,
//! and the reason it (not a bare `PhantomData`) is the right tool here.

use boundary_algebra::boundary::{Effectful, Join, Lossy, Pure, Stateful};
use typewit::TypeEq;

/// A compile-time witness that `Join` commutes on `(Self, B)`.
trait JoinCommutes<B>: Join<B>
where
    B: Join<Self>,
    Self: Sized,
{
    /// Built with `TypeEq::NEW`, so the impl compiles ONLY if the two join orders agree.
    const COMMUTES: TypeEq<<Self as Join<B>>::Out, <B as Join<Self>>::Out>;
}

macro_rules! prove_commutes {
    ($($a:ty, $b:ty);+ $(;)?) => {$(
        impl JoinCommutes<$b> for $a {
            const COMMUTES: TypeEq<<$a as Join<$b>>::Out, <$b as Join<$a>>::Out> = TypeEq::NEW;
        }
    )+};
}

// All 16 ordered pairs — the exhaustive commutativity proof over the closed capability set.
// (Flip any single `=> Out` in `boundary::join_impls!` to a non-symmetric value and the
// corresponding line below stops compiling.)
prove_commutes!(
    Pure, Pure; Pure, Lossy; Pure, Stateful; Pure, Effectful;
    Lossy, Pure; Lossy, Lossy; Lossy, Stateful; Lossy, Effectful;
    Stateful, Pure; Stateful, Lossy; Stateful, Stateful; Stateful, Effectful;
    Effectful, Pure; Effectful, Lossy; Effectful, Stateful; Effectful, Effectful;
);

/// Coerce a composed capability from the `Join<A,B>` order to the `Join<B,A>` order in GENERIC
/// code, using the carried witness. Impossible without the proof — the two `Out` types differ
/// generically; `TypeEq::to_right` discharges it.
fn recompose<A, B>(cap: <A as Join<B>>::Out) -> <B as Join<A>>::Out
where
    A: JoinCommutes<B>,
    B: Join<A>,
{
    A::COMMUTES.to_right(cap)
}

fn main() {
    // `<Pure as Join<Lossy>>::Out` is `Lossy`; recompose to `<Lossy as Join<Pure>>::Out` (also
    // `Lossy`) THROUGH the witness — a real value coercion driven by the type-level proof.
    let composed: <Pure as Join<Lossy>>::Out = Lossy;
    let _recomposed: <Lossy as Join<Pure>>::Out = recompose::<Pure, Lossy>(composed);
    println!(
        "Join commutativity: proven at compile time for all 16 capability pairs; \
         a Pure⊔Lossy capability was coerced through the witness into the Lossy⊔Pure order."
    );
}
