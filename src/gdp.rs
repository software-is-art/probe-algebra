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
use std::collections::BTreeMap;

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

    /// Transform the value while KEEPING its name — the derived value INHERITS the
    /// source's identity. This is the coupling that lets a brand FLOW through an edge:
    /// a `Post`ed entry derived from a submitted one named `N` is itself named `N`, so
    /// "this output came from THAT input" is carried in the type, and a consumer can
    /// demand both under one name. (Sound as provenance only insofar as the value is
    /// actually computed FROM the source — the same "a proof is only as true as its
    /// mint" discipline as every GDP check; here the source is the sole input.)
    pub fn map<U>(&self, f: impl FnOnce(&T) -> U) -> Named<N, U> {
        Named(f(&self.0), PhantomData)
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

// ===== relational coupling: an index bound to its collection ==============
//
// The proofs above are SINGLE-name (a fact about one value). COUPLING is the
// two-name case: a proof relating a FRESH output to a DISTINCT existing named value.
// The canonical example (mononym's `exists!(LookupResult(idx): InBounds(Map, idx))`):
// a lookup yields a named index PLUS a proof it is in bounds of THAT map, so indexing
// needs no runtime check and an index for map A cannot index map B.

/// Predicate: the named index is a valid position in the collection named `Coll`. A
/// TWO-name relational proof — `Proof<Idx, InBounds<Coll>>` carries BOTH the index's
/// name and the collection's, tying them together.
pub struct InBounds<Coll>(PhantomData<Coll>);

/// Look up a key in a NAMED map: on a hit, return the value's position FRESHLY named
/// and an `InBounds` proof coupling it to THIS map. This is where the relation is
/// EARNED (the position is real); the proof carries it onward at no cost.
pub fn lookup<Coll, N: Name, K: Ord, V>(
    // consumed for the index's unique fresh name (affine).
    _seed: Seed<N>,
    map: &Named<Coll, BTreeMap<K, V>>,
    key: &K,
) -> Option<Witnessed<impl Name, usize, InBounds<Coll>>> {
    map.value()
        .keys()
        .position(|k| k == key)
        .map(|idx| Witnessed(Named::<N, _>(idx, PhantomData), Proof(PhantomData)))
}

/// Read a NAMED map at an index PROVEN in bounds of THAT map. The proof discharges
/// the bounds obligation, so this cannot be called with an index proven for a
/// different map, and the internal lookup is total by construction (the `expect` is
/// unreachable given the proof). The provenance complement to a runtime check.
pub fn get_in_bounds<'m, Coll, Idx, K, V>(
    map: &'m Named<Coll, BTreeMap<K, V>>,
    index: &Named<Idx, usize>,
    _proof: &Proof<Idx, InBounds<Coll>>,
) -> &'m V {
    map.value()
        .values()
        .nth(*index.value())
        .expect("the InBounds proof guarantees the index is valid")
}

// ===== a PERMUTATION proven valid for its collection ======================
//
// The `InBounds` proof bounds ONE index; reordering a whole sequence needs a
// stronger relation — that an index vector is a valid PERMUTATION (a bijection). With
// it proven, an un-permute is TOTAL: no per-index bounds check and no missing/dup
// slots. This is what lets a coupling-aware construction reconstruct without the
// runtime checks the plain `Construction::reconstruct` carries.

/// Predicate: the named `Vec<usize>` is a PERMUTATION of `0..len` — a bijection.
/// Earned by `prove_permutation`, and tied to the order's own name, so a permutation
/// proven for one vector cannot stand in for another.
pub struct PermutationOf;

/// Earn a `PermutationOf` proof by the real bijection check: right length, every
/// index in range, none repeated. `None` if the order is not a valid permutation.
pub fn prove_permutation<N>(
    order: &Named<N, Vec<usize>>,
    len: usize,
) -> Option<Proof<N, PermutationOf>> {
    let o = order.value();
    if o.len() != len {
        return None;
    }
    let mut seen = vec![false; len];
    for &i in o {
        if i >= len || core::mem::replace(&mut seen[i], true) {
            return None;
        }
    }
    Some(Proof(PhantomData))
}

/// Reorder `sorted` by a PROVEN permutation whose `order[k]` is the ORIGINAL index of
/// `sorted[k]`. TOTAL — the `PermutationOf` proof guarantees a bijection of the right
/// length, so every output slot is filled exactly once (the `expect` is unreachable).
pub fn unpermute<N, T: Clone>(
    order: &Named<N, Vec<usize>>,
    _proof: &Proof<N, PermutationOf>,
    sorted: &[T],
) -> Vec<T> {
    let mut out: Vec<Option<T>> = vec![None; sorted.len()];
    for (k, &orig) in order.value().iter().enumerate() {
        out[orig] = Some(sorted[k].clone());
    }
    out.into_iter()
        .map(|slot| slot.expect("PermutationOf proof guarantees every slot is filled"))
        .collect()
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
    use std::collections::BTreeMap;

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

    /// Relational coupling: `lookup` mints an index FRESHLY named and proven in bounds
    /// of THIS map, and `get_in_bounds` reads it with no runtime bounds check. The
    /// `InBounds<Coll>` proof ties the index to the one map — an index proven for a
    /// different map would not unify (a compile error), so it cannot index the wrong
    /// collection.
    #[test]
    fn lookup_couples_an_in_bounds_index_to_its_map() {
        with_seed(|seed| {
            let (s_map, s_idx) = seed.replicate();
            let mut m = BTreeMap::new();
            m.insert("a", 10i64);
            m.insert("b", 20);
            m.insert("c", 30);
            let named_map = s_map.new_named(m);
            match lookup(s_idx, &named_map, &"b") {
                Some(found) => {
                    let (idx, proof) = found.split();
                    assert_eq!(*idx.value(), 1, "b is the 2nd key in BTree order");
                    // read with the proof — no bounds check, and bound to THIS map.
                    assert_eq!(*get_in_bounds(&named_map, &idx, &proof), 20);
                    // get_in_bounds(&another_named_map, &idx, &proof) — would NOT
                    // compile: the proof's `Coll` is `named_map`'s name alone.
                }
                None => panic!("b is present"),
            }
        });
    }

    /// A miss returns `None` — no index, no proof minted for an absent key.
    #[test]
    fn lookup_miss_mints_no_proof() {
        with_seed(|seed| {
            let (s_map, s_idx) = seed.replicate();
            let mut m: BTreeMap<&str, i64> = BTreeMap::new();
            m.insert("a", 10);
            let named_map = s_map.new_named(m);
            assert!(lookup(s_idx, &named_map, &"z").is_none());
        });
    }

    /// `prove_permutation` accepts a bijection and rejects a wrong length, an
    /// out-of-range index, and a duplicate — earning the proof only for a real
    /// permutation (each rejection pins one branch of the check).
    #[test]
    fn prove_permutation_is_exact() {
        with_seed(|seed| {
            let (a, rest) = seed.replicate();
            let (b, rest) = rest.replicate();
            let (c, d) = rest.replicate();
            assert!(prove_permutation(&a.new_named(vec![2usize, 0, 1]), 3).is_some());
            assert!(prove_permutation(&b.new_named(vec![0usize, 1]), 3).is_none()); // wrong length
            assert!(prove_permutation(&c.new_named(vec![0usize, 1, 3]), 3).is_none()); // out of range
            assert!(prove_permutation(&d.new_named(vec![0usize, 0, 1]), 3).is_none());
            // duplicate
        });
    }

    /// `unpermute` scatters each item to its original index, totally, under the proof.
    #[test]
    fn unpermute_inverts_a_known_permutation() {
        with_seed(|seed| {
            // order[k] = original index of sorted[k]: sorted[0]->2, [1]->0, [2]->1.
            let order = seed.new_named(vec![2usize, 0, 1]);
            let proof = prove_permutation(&order, 3).expect("valid permutation");
            assert_eq!(
                unpermute(&order, &proof, &["a", "b", "c"]),
                vec!["b", "c", "a"]
            );
        });
    }
}
