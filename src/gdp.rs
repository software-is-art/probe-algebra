//! gdp — a spike on "Ghosts of Departed Proofs" (Noonan 2018) using mononym's
//! technique, hand-rolled (no dependency): unique TYPE-LEVEL names plus proofs
//! phrased about a named value.
//!
//! Why it earns a place next to the value objects: a value object enforces
//! SINGLE-VALUE invariants by construction (an `Int` is non-negative). GDP carries
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

use crate::boundary::{Construction, Morphism};

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

/// A proof that predicate `P` holds for the value named `N`. Its constructor is
/// private to this module, so a proof can be minted ONLY by the checks here —
/// and `N` ties it to one specific named value. (The relational predicates and the
/// proofs that carry domain facts across a seam are now demonstrated on the
/// interpreter — `interp`'s `WellTyped`/`IllTyped` are exactly such name-branded
/// proofs, minted only by the `Check` branch.)
pub struct Proof<N, P>(PhantomData<(N, P)>);

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Distinct seeds give distinct names: an index/proof minted under one cannot be
    /// used with another (crossing them would not compile). Exercised via `lookup` below.
    /// Relational coupling: `lookup` mints an index FRESHLY named and proven in bounds of
    /// THIS map, and `get_in_bounds` reads it with no runtime bounds check. The
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

    /// A REGION stamps values flowing through the dataflow with one brand, whatever the
    /// edge's shape. A raw source is stamped through the `Parse` CONSTRUCTION edge, then the
    /// parsed expression is pushed through the `ConstFold` MORPHISM edge with `stamp` — and
    /// the folded result still carries the SAME brand. `same_region` compiles only because
    /// every step shares `N`, so same-run provenance is demandable across the whole pipeline.
    #[test]
    fn a_region_stamps_a_value_across_construction_and_morphism_edges() {
        use crate::interp::boundary::{ConstFold, Expr, Parse};
        with_region(|r| {
            let expr = stamp_parse(&r, &Parse, &"(1 + 2)".to_string()).expect("valid source");
            assert_eq!(expr.value(), &Parse.parse_str("(1 + 2)").unwrap());
            // push the named expression through the ConstFold morphism, keeping the brand.
            let folded = stamp(&r, &ConstFold, &expr);
            assert_eq!(folded.value(), &Expr::int(3).unwrap()); // (1 + 2) folded to 3
            assert_eq!(folded.value().render(), "3");
            // a third value branded into the same region also shares the name `N`.
            let rendered = r.brand(expr.value().render());
            fn same_region<N, A, B>(_a: &Named<N, A>, _b: &Named<N, B>) {}
            same_region(&expr, &folded); // compiles only because both carry one brand
            same_region(&folded, &rendered);
            assert_eq!(rendered.value(), "(1 + 2)");
        });
    }
}
