//!
//! gdp — "Ghosts of Departed Proofs" (Noonan 2018) via mononym's technique, hand-rolled (no
//! dependency): unique TYPE-LEVEL names plus proofs phrased about a named value.
//!
//! Why it earns a place next to the value objects: a value object enforces SINGLE-VALUE
//! invariants by construction (an `Int` is non-negative). GDP carries RELATIONAL / provenance
//! facts the value's type cannot — "this index is in bounds of THAT matrix", "this output
//! came from THAT input" — and TIES the proof to one named value, so a proof minted for A
//! cannot be used with B.
//!
//! This is LOAD-BEARING grammar, not a spike. Two live consumers exercise it: the
//! interpreter's `WellTyped`/`IllTyped` are gdp proofs (minted only by `Check`, demanded by
//! `Eval`), and the kill-matrix selector reads its matrix entirely through the `InBounds`
//! relational proof (`positions` ⇒ `get`), so an unproven read is not writable at all.
//!
//! Division of labour: the name uniqueness is sound (it bottoms out in the GhostCell HRTB +
//! invariant-lifetime trick at `with_seed` / `with_region`) — but a NAME is not a VALUE: a
//! region brands many values, and `map` derives new values under an old name, so a name alone
//! cannot pin "the proof is about THIS collection". The relational proofs therefore hold
//! their subject's BORROW (see the coupling note below); the brand carries provenance, the
//! borrow carries identity. And the PROOF is only as true as the code that mints it —
//! `prove_permutation` earns its bijection proof with a real check. We never mint a proof
//! from a statistical probe; that would carry an unproven fact in the type.
//!
//! Cost to note: everything runs inside one `with_seed` / `with_region` continuation — fine
//! for a program, an imposition on a library's callers.

use core::marker::PhantomData;
use std::collections::BTreeMap;

use crate::boundary::{Construction, Morphism};

// ===== the name machinery (mononym's technique, hand-rolled) ==============

/// A unique, opaque type-level name.
pub trait Name {}

/// A name backed by an INVARIANT phantom lifetime — the GhostCell trick. Two
/// `Life<'a>` / `Life<'b>` are distinct types even when the lifetimes overlap.
pub struct Life<'name>(PhantomData<*mut &'name ()>);
impl<'name> Name for Life<'name> {}

/// A value tagged with a name. A name is unique to its minting scope, but NOT to a
/// value: a `Seed`-minted `Named` starts as a singleton, while a region deliberately
/// stamps many values with one brand and `map` derives new values under an old name.
/// The brand is therefore an identity of PROVENANCE (which scope/dataflow this value
/// belongs to), not of value — facts that must pin one specific value carry its
/// borrow (see the relational proofs below).
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

// ===== a REGION: arbitrarily many values under one brand ==================
//
// `Seed`/`new_named` brands ONE value (affine). `Paired` brands two. A REGION brands
// ARBITRARILY MANY under one name — the n-ary generalization. All values stamped in a
// region share its brand, so they recombine only with each other and a value from
// another region does not unify. This is the GhostCell-style per-SCOPE brand, and it
// is what lets every morphism STAMP the values flowing through it, whatever its shape:
// `stamp` is generic over `Morphism`, so identity threads through the entire dataflow
// graph (the orthogonal axis to `run`'s residual/invertibility).

/// A reusable brand for a region — unlike the affine `Seed`, it stamps many values.
pub struct Brander<N>(PhantomData<N>);
impl<N> Brander<N> {
    /// Stamp a value into this region (brand it with the region's name).
    pub fn brand<T>(&self, value: T) -> Named<N, T> {
        Named(value, PhantomData)
    }
}

/// Enter a region with a fresh, unique brand (the `for<'name>` HRTB makes it unique to
/// this call, exactly as `with_seed`). Every value stamped inside shares the brand.
pub fn with_region<R>(cont: impl for<'name> FnOnce(Brander<Life<'name>>) -> R) -> R {
    cont(Brander(PhantomData))
}

/// The entry-edge stamp: parse a raw input and stamp the refined value into the
/// region (`None` if the parse rejects). A whole pipeline can thus start from a raw
/// primitive and carry one region brand all the way through.
pub fn stamp_parse<N, C: Construction>(
    r: &Brander<N>,
    c: &C,
    raw: &C::Raw,
) -> Option<Named<N, C::Refined>> {
    c.parse(raw).map(|(refined, _residual)| r.brand(refined))
}

/// Push a NAMED value through a `Morphism` and re-stamp the result into the SAME region:
/// the brand is the IDENTITY of the dataflow, threaded across an edge. The input and
/// output share `N`, so a value carried through one region cannot be confused with
/// another's — `same_region` (the region demo) checks exactly that. (The residual is
/// dropped here; `run`/`Carried` is the residual-keeping form.)
pub fn stamp<N, M: Morphism>(r: &Brander<N>, m: &M, input: &Named<N, M::In>) -> Named<N, M::Out> {
    r.brand(m.forward(input.value()).0)
}

// ===== a proof carried across a seam ======================================
//
// A SINGLE-name proof — "predicate P holds for the value named N" — is realized as a
// branded zero-data token minted only by its check: that is `boundary`'s
// `proof_token!` (`interp`'s `WellTyped`/`IllTyped`, minted only by the `Check`
// branch and demanded by `Eval`). The RELATIONAL proofs below are the two-value
// case, and they carry their subject's BORROW rather than only its brand — see the
// coupling note.

// ===== relational coupling: an index bound to its collection ==============
//
// The proofs above are SINGLE-name (a fact about one value). COUPLING relates a
// position to a DISTINCT collection. An honest finding forced a redesign here: a
// BRAND is not value-unique — `Brander::brand` stamps arbitrarily many values with
// one name, and `Named::map` derives a new value under the old name — so a proof
// keyed only on a phantom brand could be minted for collection A and discharged
// against a same-branded collection B, making the "unreachable" reads reachable.
// The fix is that a coupling proof HOLDS THE BORROW of the collection it certifies:
// reading goes through the proof itself, so applying it to any other collection is
// not a convention but unrepresentable, and the collection cannot change while the
// proof lives (the borrow checker is the jailer). "A proof for A cannot be used
// with B" is thereby restored as a type-level fact, not a usage discipline.

/// A key's hit in ONE map — the position, plus the borrow of THAT map. Minted only by
/// `lookup` (where the relation is EARNED: the position is real), and the only way to
/// read through it is `get`, so the proof cannot be pointed at another map and the map
/// cannot change under it.
pub struct Found<'m, Coll, K, V> {
    map: &'m Named<Coll, BTreeMap<K, V>>,
    idx: usize,
}

impl<'m, Coll, K, V> Found<'m, Coll, K, V> {
    /// The position of the found key in the map's ordered key sequence.
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Read the found value. Total by construction: `idx` was a real position when
    /// minted and the held borrow keeps the map unchanged, so the read cannot miss.
    pub fn get(&self) -> &'m V {
        self.map
            .value()
            .values()
            .nth(self.idx)
            .expect("Found holds the map's borrow, so the minted position stays valid")
    }
}

/// Look up a key in a NAMED map: on a hit, return the value's position coupled to THIS
/// map by borrow. The provenance complement to a runtime check.
pub fn lookup<'m, Coll, K: Ord, V>(
    map: &'m Named<Coll, BTreeMap<K, V>>,
    key: &K,
) -> Option<Found<'m, Coll, K, V>> {
    map.value()
        .keys()
        .position(|k| k == key)
        .map(|idx| Found { map, idx })
}

// ===== SEQUENCE positions proven in bounds (the n-ary form) ===============
//
// `lookup` couples ONE index to a map by key. A whole ALGORITHM that indexes a sequence
// repeatedly wants the n-ary form: every valid position of a sequence, each holding the
// borrow of THAT sequence, minted together so a loop indexes with no bounds check and a
// position from one sequence cannot read another. This is what lets `select` (the kill-
// matrix kernel) read its matrix entirely by proof — the relational proof made load-bearing.

/// A position proven in bounds of ONE sequence — the proof holds the sequence's borrow,
/// so it reads only THAT sequence and the sequence cannot change while it lives.
pub struct InBounds<'s, Coll, T> {
    seq: &'s Named<Coll, Vec<T>>,
    idx: usize,
}

impl<'s, Coll, T> InBounds<'s, Coll, T> {
    /// The proven position.
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Read the element at the proven position. Total by construction: the position was
    /// in range when minted and the held borrow keeps the sequence unchanged.
    pub fn get(&self) -> &'s T {
        &self.seq.value()[self.idx]
    }
}

/// Every position of a named sequence, each holding the sequence's own borrow. Turns a
/// runtime length into a set of in-bounds positions: a loop over them needs no bounds
/// check, and a position minted for sequence A cannot read sequence B.
pub fn positions<Coll, T>(seq: &Named<Coll, Vec<T>>) -> Vec<InBounds<'_, Coll, T>> {
    (0..seq.value().len())
        .map(|idx| InBounds { seq, idx })
        .collect()
}

/// Prove a single position is in bounds of a named sequence — the checked analog of
/// `lookup` (by position, not by key). `None` if out of range; on success the proof
/// holds THIS sequence's borrow.
pub fn prove_position<Coll, T>(
    seq: &Named<Coll, Vec<T>>,
    pos: usize,
) -> Option<InBounds<'_, Coll, T>> {
    (pos < seq.value().len()).then_some(InBounds { seq, idx: pos })
}

// ===== a PERMUTATION proven valid for its collection ======================
//
// The `InBounds` proof bounds ONE index; reordering a whole sequence needs a
// stronger relation — that an index vector is a valid PERMUTATION (a bijection). With
// it proven, an un-permute needs no per-index bounds check and no missing/dup
// slots. This is what lets a coupling-aware construction reconstruct without the
// runtime checks the plain `Construction::reconstruct` carries.

/// Proof that a `Vec<usize>` is a PERMUTATION of `0..len` — a bijection. Earned by
/// `prove_permutation` with the real check, and holding the order's borrow, so a
/// permutation proven for one vector cannot stand in for another (and the vector
/// cannot change while the proof lives).
pub struct PermutationOf<'o, N> {
    order: &'o Named<N, Vec<usize>>,
}

/// Earn a `PermutationOf` proof by the real bijection check: right length, every
/// index in range, none repeated. `None` if the order is not a valid permutation.
pub fn prove_permutation<N>(
    order: &Named<N, Vec<usize>>,
    len: usize,
) -> Option<PermutationOf<'_, N>> {
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
    Some(PermutationOf { order })
}

impl<N> PermutationOf<'_, N> {
    /// Reorder `sorted` by the proven permutation, whose `order[k]` is the ORIGINAL index
    /// of `sorted[k]`. The proof guarantees a bijection over the held order, so every
    /// output slot is filled exactly once (those `expect`s are unreachable); the one fact
    /// a borrow cannot carry is about the OTHER argument, so a `sorted` of the wrong
    /// length is an explicit `None`, not a panic.
    pub fn unpermute<T: Clone>(&self, sorted: &[T]) -> Option<Vec<T>> {
        let order = self.order.value();
        if sorted.len() != order.len() {
            return None;
        }
        let mut out: Vec<Option<T>> = vec![None; sorted.len()];
        for (k, &orig) in order.iter().enumerate() {
            out[orig] = Some(sorted[k].clone());
        }
        Some(
            out.into_iter()
                .map(|slot| slot.expect("the bijection fills every slot exactly once"))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    //! Like every self-hosted module, gdp carries ZERO example tests: its proof machinery is
    //! certified only by the ORACLE-FREE probes below (round-trips and exact partitions,
    //! each compared against an INDEPENDENT computation — the raw `Vec`, the `BTreeMap`, the
    //! identity), judged by the mutation sweep. The probes brand inside a `with_seed` /
    //! `with_region` scope and assert on values returned OUT of it (a brand cannot escape).
    use super::*;
    use crate::boundary::Morphism;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    /// A found position IS the key's rank in the ordered key sequence — pinned at rank 2
    /// over a three-key map, so neither constant (0 or 1) can impersonate it.
    #[test]
    fn a_found_position_is_the_keys_rank() {
        let map: BTreeMap<i16, i64> = [(10, 0), (20, 0), (30, 0)].into_iter().collect();
        let idx = with_seed(|seed| {
            let named = seed.new_named(map);
            lookup(&named, &30).expect("present").index()
        });
        assert_eq!(idx, 2);
    }

    /// A genuine permutation of `0..n`: stable-sort the identity by random keys, so the
    /// result is always a rearrangement of `0..n` (no `prop_shuffle` dependency).
    fn permutation() -> impl Strategy<Value = Vec<usize>> {
        (1usize..=6).prop_flat_map(|n| {
            prop::collection::vec(any::<u8>(), n).prop_map(move |keys| {
                let mut order: Vec<usize> = (0..n).collect();
                order.sort_by_key(|&i| keys[i]);
                order
            })
        })
    }

    /// A possibly-invalid order vector plus a target length, for the exactness probe.
    fn maybe_order() -> impl Strategy<Value = (Vec<usize>, usize)> {
        (1usize..=6).prop_flat_map(|n| (prop::collection::vec(0usize..8, 0..=6), Just(n)))
    }

    /// Simple closed Add-expressions, for the region-threading probe.
    fn int_expr() -> impl Strategy<Value = crate::interp::boundary::Expr> {
        use crate::interp::boundary::{Expr, Op};
        let leaf = (0i64..100).prop_map(|n| Expr::int(n).unwrap());
        leaf.prop_recursive(3, 12, 2, |inner| {
            (inner.clone(), inner).prop_map(|(a, b)| Expr::bin(Op::Add, a, b))
        })
    }

    proptest! {
        /// POSITIONS round-trip: branding a sequence yields one proven position per element,
        /// in order, and reading through the proof yields exactly the raw element — the
        /// n-ary InBounds the selector consumes, checked against the raw `Vec`.
        #[test]
        fn positions_index_every_element_in_order(xs in prop::collection::vec(any::<i64>(), 0..6)) {
            let (shape_ok, reads): (bool, Vec<i64>) = with_region(|r| {
                let named = r.brand(xs.clone());
                let ps = positions(&named);
                let ordered = ps.len() == xs.len()
                    && ps.iter().enumerate().all(|(i, w)| w.index() == i);
                let reads = ps.iter().map(|w| *w.get()).collect::<Vec<_>>();
                (ordered, reads)
            });
            prop_assert!(shape_ok, "positions must enumerate 0..len in order");
            prop_assert_eq!(reads, xs, "an InBounds read must return the raw element");
        }

        /// PROVE_POSITION exactness: a position is provable iff it is in range — every index
        /// below the length succeeds, the length itself and beyond fail.
        #[test]
        fn prove_position_holds_iff_in_range(
            xs in prop::collection::vec(any::<i64>(), 0..6),
            past in 0usize..4,
        ) {
            let len = xs.len();
            let (all_in, one_out): (bool, bool) = with_region(|r| {
                let named = r.brand(xs.clone());
                let all_in = (0..len).all(|i| prove_position(&named, i).is_some());
                let one_out = prove_position(&named, len + past).is_some();
                (all_in, one_out)
            });
            prop_assert!(all_in, "every in-range position must be provable");
            prop_assert!(!one_out, "no out-of-range position may be provable");
        }

        /// PERMUTATION round-trip: with `sorted[k] = xs[order[k]]`, the proven `unpermute`
        /// restores `xs` exactly — `permute ∘ unpermute == id`, oracle = the original.
        /// And the one fact the proof cannot carry — `sorted`'s length — is an explicit
        /// `None`, not a panic.
        #[test]
        fn unpermute_inverts_any_permutation(order in permutation()) {
            let n = order.len();
            let xs: Vec<i64> = (0..n as i64).collect();
            let (short_refused, restored) = with_seed(|seed| {
                let named = seed.new_named(order.clone());
                let proof = prove_permutation(&named, n).expect("a genuine permutation");
                let sorted: Vec<i64> = order.iter().map(|&i| xs[i]).collect();
                let short = proof.unpermute(&sorted[..sorted.len().saturating_sub(1)]);
                (
                    n == 0 || short.is_none(),
                    proof.unpermute(&sorted).expect("lengths match by construction"),
                )
            });
            prop_assert!(short_refused, "a short `sorted` must be refused, not read");
            prop_assert_eq!(restored, xs, "unpermute did not invert the permutation");
        }

        /// PROVE_PERMUTATION exactness: the proof is minted iff the order is a genuine
        /// bijection of `0..n` — compared against an independent bijection check, so each
        /// rejection branch (wrong length, out of range, duplicate) is pinned.
        #[test]
        fn prove_permutation_accepts_exactly_bijections((order, n) in maybe_order()) {
            let is_bijection = order.len() == n && {
                let mut seen = vec![false; n];
                order.iter().all(|&i| i < n && !core::mem::replace(&mut seen[i], true))
            };
            let proven = with_seed(|seed| prove_permutation(&seed.new_named(order.clone()), n).is_some());
            prop_assert_eq!(proven, is_bijection, "prove_permutation disagreed with the bijection check");
        }

        /// LOOKUP round-trip: a coupled lookup-then-read returns exactly what the raw
        /// `BTreeMap` would, and a miss yields `None` — oracle = `BTreeMap::get`.
        #[test]
        fn lookup_round_trips_against_the_map(
            entries in prop::collection::vec((any::<i16>(), any::<i64>()), 0..6),
            probe in any::<i16>(),
        ) {
            let map: BTreeMap<i16, i64> = entries.into_iter().collect();
            let expected = map.get(&probe).copied();
            let got = with_seed(|seed| {
                let named = seed.new_named(map.clone());
                lookup(&named, &probe).map(|found| *found.get())
            });
            prop_assert_eq!(got, expected, "coupled lookup disagreed with BTreeMap::get");
        }

        /// REGION threading: one brand carried across a `Construction` (`stamp_parse`) and a
        /// `Morphism` (`stamp`) edge, each carrying the value the edge itself computes — and
        /// the same-brand demand `same_region` compiles only because both share `N`.
        #[test]
        fn a_region_threads_one_brand_across_edge_shapes(e in int_expr()) {
            use crate::interp::boundary::{ConstFold, Parse};
            let src = e.render();
            let (parsed_ok, folded_ok) = with_region(|r| {
                let expr = stamp_parse(&r, &Parse, &src).expect("rendered source parses");
                let parsed_ok = expr.value() == &Parse.parse_str(&src).unwrap();
                let folded = stamp(&r, &ConstFold, &expr);
                let folded_ok = folded.value() == &ConstFold.forward(expr.value()).0;
                fn same_region<N, A, B>(_a: &Named<N, A>, _b: &Named<N, B>) {}
                same_region(&expr, &folded); // compiles only under one shared brand
                (parsed_ok, folded_ok)
            });
            prop_assert!(parsed_ok, "stamp_parse must carry the parsed value");
            prop_assert!(folded_ok, "stamp must carry the morphism's result");
        }
    }
}
