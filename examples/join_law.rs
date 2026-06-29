//! join_law — demonstrate that the capability-lattice laws are PROVEN at compile time, and
//! USE the carried witness to coerce (the GADT power `typewit` adds over a bare assertion).
//!
//! The proofs themselves now live in the grammar (`boundary`): the `JoinCommutes` impls and the
//! identity/idempotence `const _` assertions COMPILE only if the type-level `Join` table is a
//! commutative, idempotent semilattice with `Pure` as identity. Because `Join` has no runtime
//! body, the mutation sweep cannot reach it — these `TypeEq` witnesses are its only certifier.
//!
//! This example shows the witness in USE: `recompose` carries `JoinCommutes::COMMUTES` and
//! `to_right`-coerces a composed capability between the two join orders, in generic code where
//! the two `Out` types are not otherwise known to be equal.

use boundary_algebra::boundary::{Join, JoinCommutes, Lossy, Pure};

/// Coerce a composed capability from the `Join<A,B>` order to the `Join<B,A>` order in GENERIC
/// code, using the carried commutativity witness. Impossible without the proof — the two `Out`
/// types differ generically; `TypeEq::to_right` discharges it.
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
        "capability-lattice laws (commutativity, identity, idempotence): proven at compile time \
         in `boundary`; a Pure⊔Lossy capability was coerced through the witness into Lossy⊔Pure."
    );
}
