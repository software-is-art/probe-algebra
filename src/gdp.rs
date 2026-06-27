//! gdp — a spike on "Ghosts of Departed Proofs" (Noonan 2018) using mononym's
//! technique, hand-rolled (no dependency): unique TYPE-LEVEL names plus proofs
//! phrased about a named value.
//!
//! Why it earns a place next to the value objects: a value object enforces
//! SINGLE-VALUE invariants by construction (a `Cents` is in range). GDP carries
//! RELATIONAL / provenance facts the value's type cannot — here, "this specific
//! transaction is balanced" — across a module seam, and TIES the proof to one
//! named value so a proof minted for transaction A cannot be used with B.
//!
//! Division of labour: the name uniqueness is sound (it bottoms out in the
//! GhostCell HRTB + invariant-lifetime trick at `with_seed`). The PROOF is only
//! as true as the code that mints it — `prove_balanced` earns it with a real
//! check. We never mint a proof from a statistical probe; that would carry an
//! unproven fact in the type.
//!
//! Cost to note: everything runs inside one `with_seed` continuation — fine for a
//! program, an imposition on a library's callers.

use core::marker::PhantomData;

use crate::ledger::boundary::{Balance, Transaction};

// ===== the name machinery (mononym's technique, hand-rolled) ==============

/// A unique, opaque type-level name.
pub trait Name {}

/// A name backed by an INVARIANT phantom lifetime — the GhostCell trick. Two
/// `Life<'a>` / `Life<'b>` are distinct types even when the lifetimes overlap.
pub struct Life<'name>(PhantomData<*mut &'name ()>);
impl<'name> Name for Life<'name> {}

/// A value tagged with a unique name. A `Named<N, T>` is effectively a singleton:
/// Rust's affine types mean no two values can share the same `N`.
pub struct Named<N, T>(T, PhantomData<N>);
impl<N, T> Named<N, T> {
    pub fn value(&self) -> &T {
        &self.0
    }
    pub fn into_value(self) -> T {
        self.0
    }
}

/// Two values sharing ONE name — e.g. an output and its residual from a single
/// run. They can be separated (`split`) and stored apart, yet only re-paired with
/// their original partner, because the shared name will not unify with any other.
pub struct Paired<N, A, B>(Named<N, A>, Named<N, B>);
impl<N, A, B> Paired<N, A, B> {
    pub fn left(&self) -> &Named<N, A> {
        &self.0
    }
    pub fn right(&self) -> &Named<N, B> {
        &self.1
    }
    pub fn split(self) -> (Named<N, A>, Named<N, B>) {
        (self.0, self.1)
    }
}

/// An affine seed: consumed to mint a name, so the same seed cannot mint two.
pub struct Seed<N>(PhantomData<N>);
impl<N: Name> Seed<N> {
    /// Tag a value with a fresh unique name (consumes the seed).
    pub fn new_named<T>(self, value: T) -> Named<impl Name, T> {
        Named::<N, T>(value, PhantomData)
    }

    /// Tag TWO values with one shared fresh name (consumes the seed).
    pub fn new_paired<A, B>(self, a: A, b: B) -> Paired<impl Name, A, B> {
        Paired(Named::<N, A>(a, PhantomData), Named::<N, B>(b, PhantomData))
    }

    /// Split one seed into two with distinct names.
    pub fn replicate(self) -> (Seed<impl Name>, Seed<impl Name>) {
        (Seed(PhantomData::<N>), Seed(PhantomData::<N>))
    }
}

/// Enter a scope with a fresh seed. The `for<'name>` bound makes `Life<'name>`
/// unique to this call, so every name derived from the seed is unique.
pub fn with_seed<R>(cont: impl for<'name> FnOnce(Seed<Life<'name>>) -> R) -> R {
    cont(Seed(PhantomData))
}

// ===== a proof carried across a seam ======================================

/// A proof that predicate `P` holds for the value named `N`. Its constructor is
/// private to this module, so a proof can be minted ONLY by the checks here —
/// and `N` ties it to one specific named value.
pub struct Proof<N, P>(PhantomData<(N, P)>);

/// Predicate: a transaction is balanced (its postings sum to zero — double entry).
pub struct Balanced;

/// Module A's check: mint `Balanced` IFF the named transaction really balances.
/// This is where the fact is EARNED; the name carries it onward at no cost.
pub fn prove_balanced<N>(tx: &Named<N, Transaction>) -> Option<Proof<N, Balanced>> {
    let mut total = Balance::zero();
    for posting in tx.value().postings() {
        total = total.add_cents(*posting.amount());
    }
    if total == Balance::zero() {
        Some(Proof(PhantomData))
    } else {
        None
    }
}

/// Module B's precondition: it accepts a transaction ONLY together with a proof
/// that the SAME named transaction is balanced. It cannot be called with an
/// unbalanced one (no proof exists), an unbranded one (not `Named`), or a proof
/// minted for a different transaction (`N` will not unify). A's check discharges
/// B's precondition at the type level.
pub fn export_balanced<N>(tx: &Named<N, Transaction>, _proof: &Proof<N, Balanced>) -> usize {
    tx.value().postings().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::boundary::{Account, Cents, Posting};

    fn balanced() -> Transaction {
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(10_000).unwrap()),
            Posting::new(
                Account::new("Revenue").unwrap(),
                Cents::new(-10_000).unwrap(),
            ),
        ])
        .unwrap()
    }

    fn unbalanced() -> Transaction {
        Transaction::new(vec![Posting::new(
            Account::new("Cash").unwrap(),
            Cents::new(10_000).unwrap(),
        )])
        .unwrap()
    }

    /// A's check mints the proof; the SAME named value + proof discharges B's
    /// precondition. (The proof is tied to this value's name; one minted for
    /// another transaction would not type-check here — `N` cannot unify.)
    #[test]
    fn balance_proof_discharges_the_export_precondition() {
        with_seed(|seed| {
            let named = seed.new_named(balanced());
            let proof = prove_balanced(&named).expect("the sample balances");
            assert_eq!(export_balanced(&named, &proof), 2);
        });
    }

    /// An unbalanced transaction yields NO proof, so `export_balanced` is
    /// unreachable for it — there is nothing to pass as the second argument.
    #[test]
    fn unbalanced_transaction_has_no_proof() {
        with_seed(|seed| {
            let named = seed.new_named(unbalanced());
            assert!(prove_balanced(&named).is_none());
        });
    }

    /// Distinct seeds give distinct names: a proof about one cannot be confused
    /// with the other. (Using `proof_a` with `named_b` below would not compile.)
    #[test]
    fn names_from_distinct_seeds_do_not_unify() {
        with_seed(|seed| {
            let (s1, s2) = seed.replicate();
            let named_a = s1.new_named(balanced());
            let named_b = s2.new_named(balanced());
            let proof_a = prove_balanced(&named_a).unwrap();
            let proof_b = prove_balanced(&named_b).unwrap();
            // each proof discharges only its own value's precondition
            assert_eq!(export_balanced(&named_a, &proof_a), 2);
            assert_eq!(export_balanced(&named_b, &proof_b), 2);
            // export_balanced(&named_a, &proof_b) — does NOT compile: N mismatch.
        });
    }
}
