//! Tier: INTERIOR — the workshop / leaves (tier 2 inward rule).
//!
//! probes — the store's per-module probe registry, mirroring the crate `harness`: every
//! edge declared in `edges!` MUST impl `Probed` (a compile-time obligation, enforced
//! again by `build.rs`'s enumeration), and every probe is ORACLE-FREE — two-route laws,
//! residual round-trips, independent recomputation — never an example table. The file
//! carries no `cfg` markers; `mod.rs` registers it as `#[cfg(test)] mod probes;`, the
//! same trick `lib.rs` plays with `harness`.
//!
//! What is NOT derived is DISCLOSED, exactly as in `src/tests.rs`: the negative tests at
//! the bottom (malformed keys rejected; expired or absent reads classify as `Gone`) are
//! the hand-written irreducible base — "X is rejected" cannot be derived from the thing
//! under test.
//!
//! Division of labour with `theory`: the discovered laws pin the STATE ALGEBRA (merge
//! monoid, tick action); the probes here pin what the algebra cannot see — the residual
//! witnesses, the witness-guarded read path, and the exact expiry inequality.

use proptest::prelude::*;
use proptest::test_runner::TestRunner;

use crate::boundary::{assert_all_probed, Construction, Morphism, Probed};
use crate::gdp::with_seed;
use crate::harness::{construction_capability_matches_residual, construction_round_trips};
use crate::kvstore::store::{
    Advance, Get, Key, Lookup, ParseKey, Put, Read, Store, Tick, Ttl, Val, Write,
};

// ===== input distributions, built through the public smart constructors ====

fn key(s: &str) -> Key {
    Key::new(s).expect("a valid key")
}
fn val(n: i64) -> Val {
    Val::new(n).expect("a valid value")
}
fn ttl(n: i64) -> Ttl {
    Ttl::new(n).expect("a valid duration")
}

/// Short keys over a TINY alphabet, so generated stores collide on keys often — the
/// overwrite path (and thus the `Displaced` residual) is exercised, not just fresh
/// inserts.
fn keys() -> impl Strategy<Value = Key> {
    proptest::collection::vec(proptest::char::range('a', 'd'), 1..3)
        .prop_map(|cs| key(&cs.into_iter().collect::<String>()))
}
fn vals() -> impl Strategy<Value = Val> {
    (0i64..=9).prop_map(val)
}
/// TTLs INCLUDING zero (dead on arrival), for store generation.
fn ttls() -> impl Strategy<Value = Ttl> {
    (0i64..=6).prop_map(ttl)
}
/// Strictly positive TTLs, for probes that need the written entry observable.
fn live_ttls() -> impl Strategy<Value = Ttl> {
    (1i64..=6).prop_map(ttl)
}

/// One step of store history.
#[derive(Debug, Clone)]
enum Step {
    Put(Key, Val, Ttl),
    Tick(Ttl),
}

/// Stores generated as HISTORIES — a fold of random puts and ticks over the empty store,
/// routed through the real edges — so the distribution includes overwritten keys, ticked
/// clocks, near-expiry entries, and dead-on-arrival entries, never a hand-assembled state.
fn stores() -> impl Strategy<Value = Store> {
    let step = prop_oneof![
        (keys(), vals(), ttls()).prop_map(|(k, v, t)| Step::Put(k, v, t)),
        (0i64..=3).prop_map(|n| Step::Tick(ttl(n))),
    ];
    proptest::collection::vec(step, 0..8).prop_map(|steps| {
        steps.into_iter().fold(Store::new(), |s, step| match step {
            Step::Put(k, v, t) => s.put(k, v, t),
            Step::Tick(by) => s.tick(by),
        })
    })
}

/// The full witness-guarded read path: classify, then read under the `Live` proof.
/// `None` iff `Get` minted `Gone` — the only way to observe "no value" from outside.
fn read(s: &Store, k: &Key) -> Option<Val> {
    with_seed(|seed| {
        let named = seed.new_named(Lookup::new(s.clone(), k.clone()));
        match Get.classify(&named) {
            Ok(proof) => Some(*Read.run(&named, &proof).value()),
            Err(_) => None,
        }
    })
}

/// Does the lookup classify `Live`?
fn is_live(s: &Store, k: &Key) -> bool {
    read(s, k).is_some()
}

// ===== edges declared once; each MUST carry a probe ========================

type Edges = crate::edges!(ParseKey, Put, Get, Read, Tick);

/// COMPLETENESS: every edge in `Edges` impls `Probed` — the same obligation the crate
/// harness discharges for the interpreter's edge set.
#[test]
fn every_edge_is_probed() {
    assert_all_probed::<Edges>();
}

/// The entry edge: `reconstruct . parse == id` over generated valid key text (rendered
/// from generated `Key`s, so the generator is independent of the parser), and the `Pure`
/// claim is checked against the `Unit` residual. A parse that normalized (trimmed,
/// lowercased) would fail the round-trip.
impl Probed for ParseKey {
    fn probe() {
        construction_round_trips(&ParseKey, keys().prop_map(|k| k.get().to_string()));
        construction_capability_matches_residual::<ParseKey>();
    }
}
#[test]
fn parse_key_is_probed() {
    ParseKey::probe();
}

/// `Put` — the stateful writer. Three oracle-free laws over generated histories:
///   1. the `Displaced` residual reconstructs the EXACT input (`backward . forward ==
///      id`) — killing a residual that forgets the prior binding, records the wrong key,
///      or an `unput` that fails to restore (the tiny key alphabet makes real
///      displacement frequent, so the `prior: Some(_)` arm is exercised, not vacuous);
///   2. put-then-read coherence: a binding written with a live TTL is `Live` and reads
///      back the value written — killing wrong-value storage, wrong-key storage, and a
///      `born` stamped at the wrong instant;
///   3. last write wins: overwriting the same key reads the SECOND value — killing an
///      append-instead-of-replace interior (whose stale first entry `lookup` would find).
impl Probed for Put {
    fn probe() {
        TestRunner::default()
            .run(
                &(stores(), keys(), vals(), vals(), live_ttls(), live_ttls()),
                |(s, k, v1, v2, t1, t2)| {
                    let w = Write::new(s, k.clone(), v1, t1);
                    let (out, residual) = Put.forward(&w);
                    prop_assert_eq!(
                        Put.backward(&out, &residual),
                        Some(w),
                        "the displaced-binding residual did not reconstruct the write"
                    );
                    prop_assert_eq!(
                        read(&out, &k),
                        Some(v1),
                        "a live-TTL write must read back as written"
                    );
                    let twice = out.put(k.clone(), v2, t2);
                    prop_assert_eq!(read(&twice, &k), Some(v2), "last write must win");
                    Ok(())
                },
            )
            .unwrap();
    }
}
#[test]
fn put_is_probed() {
    Put::probe();
}

/// `Get` — the liveness `Branch`. The classification is pinned from BOTH sides of the
/// expiry inequality by construction (no oracle: the inputs are built so the answer is
/// forced, the way the harness builds closed int expressions for `Check`): a binding
/// with TTL `t` is `Live` after `t - 1` ticks and `Gone` after `t` — so a mutant
/// relaxing the strict `<` to `<=` (or tightening it) dies deterministically.
impl Probed for Get {
    fn probe() {
        TestRunner::default()
            .run(&(stores(), keys(), vals(), live_ttls()), |(s, k, v, t)| {
                let written = s.put(k.clone(), v, t);
                prop_assert!(is_live(&written, &k), "a fresh live-TTL binding is Live");
                let brink = written.tick(ttl(t.get() - 1));
                prop_assert!(is_live(&brink, &k), "one tick short of the TTL is Live");
                let dead = written.tick(t);
                prop_assert!(!is_live(&dead, &k), "exactly the TTL later is Gone");
                Ok(())
            })
            .unwrap();
    }
}
#[test]
fn get_is_probed() {
    Get::probe();
}

/// `Read` — the guarded reader. The FRAME law, two routes to one value: reading `k` is
/// invariant under a write to any OTHER key (route 1: read, then write elsewhere, read
/// again; route 2: the first read). Kills a lookup that matches the wrong key, returns
/// another entry's value, or a `put` that perturbs neighbours. (`Read` under the proof
/// returning the RIGHT value is pinned by `Put`'s coherence law above.)
impl Probed for Read {
    fn probe() {
        TestRunner::default()
            .run(
                &(
                    stores(),
                    keys(),
                    keys(),
                    vals(),
                    vals(),
                    live_ttls(),
                    live_ttls(),
                ),
                |(s, k, other, v, v2, t, t2)| {
                    // force the second key distinct by extension (still a valid key), so
                    // the frame condition is non-vacuous on every generated case.
                    let other = if other == k {
                        key(&format!("{}x", other.get()))
                    } else {
                        other
                    };
                    let base = s.put(k.clone(), v, t);
                    let before = read(&base, &k);
                    let after = read(&base.put(other, v2, t2), &k);
                    prop_assert_eq!(after, before, "a write to another key moved this read");
                    Ok(())
                },
            )
            .unwrap();
    }
}
#[test]
fn read_is_probed() {
    Read::probe();
}

/// `Tick` — time as an edge. Three oracle-free laws over generated histories:
///   1. the `Swept` residual reconstructs the EXACT pre-tick store (`backward . forward
///      == id`) — killing a sweep that loses corpses or a rewind that mis-restores;
///   2. eager agrees with lazy: the ticked store's view equals an INDEPENDENT
///      recomputation that never sweeps — filtering the ORIGINAL entries by liveness at
///      the advanced clock — so evicting a live entry (or surfacing a dead one) diverges
///      the routes; and the swept store carries NO dead entry, so a sweep that silently
///      keeps corpses (invisible to any view) is killed structurally;
///   3. the clock moves by exactly the step, checked against raw integer arithmetic
///      (bounded operands, so saturation never blurs it) — killing off-by-one and
///      wrong-operator mutants in the advance.
impl Probed for Tick {
    fn probe() {
        TestRunner::default()
            .run(&(stores(), 0i64..=6), |(s, n)| {
                let by = ttl(n);
                let a = Advance::new(s.clone(), by);
                let (out, residual) = Tick.forward(&a);
                prop_assert_eq!(
                    Tick.backward(&out, &residual),
                    Some(a),
                    "the swept residual did not reconstruct the pre-tick store"
                );
                let advanced = s.clock().advanced(by);
                let lazy: Vec<(Key, Val, Ttl)> = s
                    .entries()
                    .iter()
                    .filter(|e| e.live_at(advanced))
                    .map(|e| (e.key().clone(), e.val(), e.remaining_at(advanced)))
                    .collect();
                prop_assert_eq!(
                    out.view().rows().to_vec(),
                    lazy,
                    "the eager sweep diverged from the lazy specification"
                );
                prop_assert!(
                    out.entries().iter().all(|e| e.live_at(out.clock())),
                    "the sweep left a dead entry behind"
                );
                prop_assert_eq!(
                    out.clock().get(),
                    s.clock().get() + n,
                    "the clock did not advance by exactly the step"
                );
                Ok(())
            })
            .unwrap();
    }
}
#[test]
fn tick_is_probed() {
    Tick::probe();
}

/// `merge` — the value operator the discovered spec certifies as a monoid. But the
/// universal shapes CANNOT name the DIRECTION of the bias: a first-write-wins union is
/// also associative, idempotent, and `empty`-neutral, so a hand-planted left-biased
/// mutant SURVIVED the golden spec (a real blind-spot finding — the algebra sees that
/// merge is a monoid, not which write wins). This probe closes it, oracle-free: the
/// merged read of any key must agree with an INDEPENDENT per-key recomputation off the
/// raw entries — the RIGHT store's binding where both bind, judged live at the later
/// clock — and the merged clock must be the raw integer max.
#[test]
fn merge_is_right_biased_at_the_later_clock() {
    TestRunner::default()
        .run(&(stores(), stores(), keys()), |(a, b, k)| {
            let merged = a.merge(&b);
            prop_assert_eq!(
                merged.clock().get(),
                a.clock().get().max(b.clock().get()),
                "the merged clock is not the later of the two"
            );
            let winner = b
                .entries()
                .iter()
                .find(|e| e.key() == &k)
                .or_else(|| a.entries().iter().find(|e| e.key() == &k));
            let expected = winner
                .filter(|e| e.live_at(merged.clock()))
                .map(|e| e.val());
            prop_assert_eq!(
                read(&merged, &k),
                expected,
                "merge did not let the right-hand store's binding win"
            );
            Ok(())
        })
        .unwrap();
}

// ===== NEGATIVE: the disclosed, hand-written irreducible base ==============
//
// Mirroring `src/tests.rs`: rejection cannot be derived from the thing under test, so
// the blind-spot side of each edge is pinned by hand and disclosed as such.

/// The entry edge REJECTS malformed keys — empty, non-alphabetic, non-ASCII — through
/// both the smart constructor and the `Construction` (one mint, two doors).
#[test]
fn rejects_malformed_keys() {
    assert!(Key::new("").is_none());
    assert!(Key::new("k1").is_none());
    assert!(Key::new("two words").is_none());
    assert!(Key::new("naïve").is_none());
    assert!(ParseKey.parse(&String::new()).is_none());
    assert!(ParseKey.parse(&"a b".to_string()).is_none());
}

/// A dead lookup classifies `Gone` — for every way of being dead: never written,
/// expired at exactly its TTL, and dead on arrival (TTL zero, since liveness is
/// STRICTLY before expiry). The negative witness is a first-class token, so these pin
/// the `Err` arm `Get` must mint, not a `None` it might fall into.
#[test]
fn dead_lookups_classify_gone() {
    let k = key("k");
    let empty = Store::new();
    assert!(!is_live(&empty, &k), "an absent key is Gone");
    let expired = empty.put(k.clone(), val(7), ttl(2)).tick(ttl(2));
    assert!(
        !is_live(&expired, &k),
        "an entry at exactly its TTL is Gone"
    );
    let doa = empty.put(k.clone(), val(7), ttl(0));
    assert!(!is_live(&doa, &k), "a zero-TTL entry is dead on arrival");
}
