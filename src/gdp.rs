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

use crate::boundary::Morphism;
use crate::ledger::boundary::{AccountSummary, Balance, Round, Transaction};
use crate::linear::boundary::{Quantity, Scale};

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
}

/// Two values sharing ONE name — e.g. an output and its residual from a single
/// run. They can be separated (`split`) and stored apart, yet only re-paired with
/// their original partner, because the shared name will not unify with any other.
pub struct Paired<N, A, B>(Named<N, A>, Named<N, B>);
impl<N, A, B> Paired<N, A, B> {
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
/// Predicate: a transaction is NOT balanced.
pub struct Unbalanced;

/// Total classification — the paper's `classify`, NOT a `Maybe`. Every named
/// transaction is balanced or not, and BOTH branches carry a usable proof.
/// `Option<Proof<Balanced>>` would discard the negative witness; keeping it makes
/// the unbalanced case first-class (route it to a correction queue, etc.).
pub enum BalanceProof<N> {
    Balanced(Proof<N, Balanced>),
    Unbalanced(Proof<N, Unbalanced>),
}

/// Classify a named transaction by balance, minting the matching proof. This is
/// where the fact is EARNED; the name carries it onward at no cost.
pub fn classify_balance<N>(tx: &Named<N, Transaction>) -> BalanceProof<N> {
    let mut total = Balance::zero();
    for posting in tx.value().postings() {
        total = total.add_cents(*posting.amount());
    }
    if total == Balance::zero() {
        BalanceProof::Balanced(Proof(PhantomData))
    } else {
        BalanceProof::Unbalanced(Proof(PhantomData))
    }
}

/// Consumer of the POSITIVE branch: accepts a transaction ONLY with a proof that
/// the SAME named value is balanced — so an unbalanced, unbranded, or
/// wrong-named transaction will not type-check. A's classification discharges B's
/// precondition.
pub fn export_balanced<N>(tx: &Named<N, Transaction>, _proof: &Proof<N, Balanced>) -> usize {
    tx.value().postings().len()
}

/// Consumer of the NEGATIVE branch: accepts a transaction ONLY with a proof it is
/// UNBALANCED (e.g. route to a correction queue). The negative witness is carried,
/// not discarded — that is the upgrade over an `Option`-returning check.
pub fn quarantine_unbalanced<N>(
    tx: &Named<N, Transaction>,
    _proof: &Proof<N, Unbalanced>,
) -> usize {
    tx.value().postings().len()
}

// ===== a witness that an OPERATION occurred (the audit) ===================

/// A named value bundled with a proof about it under one shared name — e.g. an
/// output together with a witness of the operation that produced it.
pub struct Witnessed<N, T, P>(Named<N, T>, Proof<N, P>);
impl<N, T, P> Witnessed<N, T, P> {
    pub fn split(self) -> (Named<N, T>, Proof<N, P>) {
        (self.0, self.1)
    }
}

/// Witness: this summary is the result of `Round` (whole-dollar) reduction.
///
/// AUDIT finding: an operation's having-occurred is lost from the type system
/// exactly when the operation is an ENDO-map (input and output the same type).
/// `Round: AccountSummary -> AccountSummary` and `Scale: Quantity -> Quantity`
/// both erase the fact that they ran — a rounded summary is indistinguishable
/// from an un-rounded one. (Contrast `Calibrate: Sample -> Reading`, where the
/// type change itself witnesses the operation, so nothing is lost.) For an
/// endo-operation, capture the lost witness with a proof on the named output.
pub struct Rounded;

/// Run `Round`, capturing a witness — tied to the output's name — that rounding
/// occurred. The operation is no longer invisible: downstream can DEMAND it.
pub fn round_witnessed<N: Name>(
    // consumed for its unique name `N` (affine: the seed cannot be reused) — the
    // value and the proof are both branded with it below.
    _seed: Seed<N>,
    summary: &AccountSummary,
) -> Witnessed<impl Name, AccountSummary, Rounded> {
    let (rounded, _residual) = Round.forward(summary);
    Witnessed(Named::<N, _>(rounded, PhantomData), Proof(PhantomData))
}

/// A consumer requiring its input to have been ROUNDED — e.g. a whole-dollar
/// formatter. It cannot be called on an un-rounded or un-witnessed summary, so
/// "forgot to round" becomes a compile error rather than a wrong report.
pub fn whole_dollars<N>(summary: &Named<N, AccountSummary>, _proof: &Proof<N, Rounded>) -> usize {
    summary.value().totals().len()
}

/// Witness: this quantity is the output of the REFERENCE scaling (`Scale::honest`).
///
/// `Scale: Quantity -> Quantity` is the other endo-map from the audit, so "scaled"
/// is invisible in the type — and (the decisive negative result) a wrong-rate
/// `skew` output is indistinguishable from a right one. A value obtains this
/// witness ONLY by passing through `scale_witnessed`, which runs the honest rate;
/// the library offers no skew-witnessing counterpart, so a skew-scaled value can
/// never carry it. A consumer that requires `Scaled` therefore rejects skew (and
/// raw, un-scaled quantities) at COMPILE time — the provenance complement to the
/// runtime quantitative probe. (Sound because the witness is minted only on the
/// honest path; it is provenance, not verification of an arbitrary input.)
///
/// Together with `Scale::CAPABILITY` this captures BOTH facts an endo-operation
/// hides: its capability (`Pure`, a static const) and its having-occurred (this
/// witness).
pub struct Scaled;

/// Run the reference scaling, capturing a witness — tied to the output's name —
/// that it occurred at the honest rate.
pub fn scale_witnessed<N: Name>(
    // consumed for its unique name `N` (affine), used to brand the output below.
    _seed: Seed<N>,
    quantity: &Quantity,
) -> Witnessed<impl Name, Quantity, Scaled> {
    let (scaled, _unit) = Scale::honest().forward(quantity);
    Witnessed(Named::<N, _>(scaled, PhantomData), Proof(PhantomData))
}

/// A consumer requiring its input to be a reference-scaled figure. It cannot be
/// called on a raw quantity or a skew-scaled one — neither carries `Scaled`.
pub fn report_scaled<N>(quantity: &Named<N, Quantity>, _proof: &Proof<N, Scaled>) -> i64 {
    quantity.value().get()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::boundary::{Account, Aggregate, Cents, Posting};

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
        Transaction::new(vec![
            Posting::new(Account::new("Cash").unwrap(), Cents::new(10_000).unwrap()),
            Posting::new(Account::new("Fees").unwrap(), Cents::new(5_000).unwrap()),
        ])
        .unwrap()
    }

    /// Total classify, positive branch: the proof discharges B's precondition,
    /// tied to this value's name (a proof for another transaction would not unify).
    #[test]
    fn balanced_branch_discharges_export() {
        with_seed(|seed| {
            let named = seed.new_named(balanced());
            match classify_balance(&named) {
                BalanceProof::Balanced(proof) => assert_eq!(export_balanced(&named, &proof), 2),
                BalanceProof::Unbalanced(_) => panic!("the sample balances"),
            }
        });
    }

    /// Total classify, negative branch: the unbalanced case is NOT discarded — it
    /// carries a proof the correction path consumes.
    #[test]
    fn unbalanced_branch_carries_a_usable_witness() {
        with_seed(|seed| {
            let named = seed.new_named(unbalanced());
            match classify_balance(&named) {
                BalanceProof::Unbalanced(proof) => {
                    assert_eq!(quarantine_unbalanced(&named, &proof), 2)
                }
                BalanceProof::Balanced(_) => panic!("the sample does not balance"),
            }
        });
    }

    /// Distinct seeds give distinct names: a proof about one cannot be used with
    /// the other (crossing them would not compile).
    #[test]
    fn names_from_distinct_seeds_do_not_unify() {
        with_seed(|seed| {
            let (s1, s2) = seed.replicate();
            let a = s1.new_named(balanced());
            let b = s2.new_named(balanced());
            match (classify_balance(&a), classify_balance(&b)) {
                (BalanceProof::Balanced(pa), BalanceProof::Balanced(pb)) => {
                    assert_eq!(export_balanced(&a, &pa), 2);
                    assert_eq!(export_balanced(&b, &pb), 2);
                    // export_balanced(&a, &pb) — does NOT compile: names differ.
                }
                _ => panic!("both balance"),
            }
        });
    }

    /// The captured operation-witness: `round_witnessed` records that rounding
    /// occurred, and `whole_dollars` can be called ONLY with that witness — the
    /// endo-operation is no longer invisible in the type system.
    #[test]
    fn rounding_witness_is_captured_and_required() {
        with_seed(|seed| {
            let summary = Aggregate.forward(&balanced()).0;
            let (named, proof) = round_witnessed(seed, &summary).split();
            assert_eq!(whole_dollars(&named, &proof), 2);
            // whole_dollars(&some_unrounded_named, ...) — impossible: no Rounded
            // proof exists for a summary that did not pass round_witnessed.
        });
    }

    /// The other endo-map: `scale_witnessed` records that reference scaling ran,
    /// `report_scaled` requires the witness, and the witnessed op's capability is
    /// `Pure` — both type-invisible facts of the endo-operation now captured.
    #[test]
    fn scaling_witness_gates_consumer_and_pairs_with_capability() {
        use crate::boundary::Capability;
        use crate::linear::boundary::Scale;
        with_seed(|seed| {
            let q = Quantity::new(7).unwrap();
            let (named, proof) = scale_witnessed(seed, &q).split();
            // 7 * honest rate 3; report_scaled on a raw or skew-scaled Quantity is
            // impossible — neither carries a Scaled proof.
            assert_eq!(report_scaled(&named, &proof), 21);
            assert_eq!(<Scale as Morphism>::CAPABILITY, Capability::Pure);
        });
    }
}
