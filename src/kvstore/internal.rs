//!
//! internal — PRIVATE implementation of the store: entry storage, lookup, the expiry
//! sweep, and the right-biased merge. Other modules cannot name anything here.
//!
//! This is the "relax inside" half of the experiment, on STATE this time: this file has
//! **zero tests of its own**. Ordinary imperative code — clones, loops, a `HashMap` in
//! `merge` — reached only transitively through the edges in `store.rs`, and verified only
//! by the oracle-free probes in `probes` plus the discovered laws in `theory`. Like
//! `interp::internal`, it is deliberately KEPT IN the mutation sweep, so cargo-mutants
//! quantifies how much internal correctness the boundary contracts buy for a stateful
//! interior. The INWARD rule still holds: no function returns a raw primitive — every
//! result is a `Store`, `Snapshot`, `Entry`, or a collection of value objects.
//!
//! One representation invariant is owned here (and only here, because only this file
//! calls `Store::assemble`): entries are SORTED BY KEY with at most one entry per key, so
//! a store's structural equality is canonical — which is what makes the `Put`/`Tick`
//! residual round-trips exact rather than up-to-reordering.

use std::collections::HashMap;

use crate::kvstore::store::{Clock, Entry, Key, Snapshot, Store, Ttl, Val};

/// Bind `key` to `val` with `ttl` at the store's current clock, replacing any existing
/// binding. Returns the new store plus the DISPLACED prior entry (`None` if the key was
/// vacant) — the witness `Put`'s residual carries.
pub(super) fn put(store: &Store, key: &Key, val: Val, ttl: Ttl) -> (Store, Option<Entry>) {
    let mut entries: Vec<Entry> = Vec::new();
    let mut prior = None;
    for e in store.entries() {
        if e.key() == key {
            prior = Some(e.clone());
        } else {
            entries.push(e.clone());
        }
    }
    entries.push(Entry::new(key.clone(), val, store.clock(), ttl));
    entries.sort_by(|a, b| a.key().cmp(b.key()));
    (Store::assemble(entries, store.clock()), prior)
}

/// Undo a `put`: drop the entry written for `key` and restore the displaced `prior` (if
/// any). The inverse `Put::backward` threads its residual through.
pub(super) fn unput(after: &Store, key: &Key, prior: Option<Entry>) -> Store {
    let mut entries: Vec<Entry> = after
        .entries()
        .iter()
        .filter(|e| e.key() != key)
        .cloned()
        .collect();
    if let Some(p) = prior {
        entries.push(p);
        entries.sort_by(|a, b| a.key().cmp(b.key()));
    }
    Store::assemble(entries, after.clock())
}

/// The value bound to `key`, iff its entry is LIVE at the store's own clock. `None` for
/// absent AND expired alike — the coarsening `Get` turns into the single `Gone` witness.
pub(super) fn lookup(store: &Store, key: &Key) -> Option<Val> {
    store
        .entries()
        .iter()
        .find(|e| e.key() == key)
        .filter(|e| e.live_at(store.clock()))
        .map(|e| e.val())
}

/// Advance the clock by `by` and EAGERLY SWEEP: entries no longer live at the new clock
/// are removed and returned (the corpses `Tick`'s residual carries). Keeping the sweep
/// eager is a real implementation choice — the probes hold it to the LAZY specification
/// (filtering by liveness), so the two routes must agree.
pub(super) fn tick(store: &Store, by: Ttl) -> (Store, Vec<Entry>) {
    let clock = store.clock().advanced(by);
    let mut kept: Vec<Entry> = Vec::new();
    let mut evicted: Vec<Entry> = Vec::new();
    for e in store.entries() {
        if e.live_at(clock) {
            kept.push(e.clone());
        } else {
            evicted.push(e.clone());
        }
    }
    (Store::assemble(kept, clock), evicted)
}

/// Undo a `tick`: resurrect the evicted entries at the rewound `clock`. Evicted keys are
/// disjoint from the kept ones (the sweep removed them), so re-sorting restores the
/// exact original representation.
pub(super) fn restore(after: &Store, evicted: &[Entry], clock: Clock) -> Store {
    let mut entries: Vec<Entry> = after.entries().to_vec();
    entries.extend(evicted.iter().cloned());
    entries.sort_by(|a, b| a.key().cmp(b.key()));
    Store::assemble(entries, clock)
}

/// Right-biased union at the LATER clock: where both stores bind a key, `b`'s binding
/// wins (last write wins). Ordinary `HashMap` code — the canonical sorted `Vec` is
/// rebuilt at the end, so the representation invariant survives the detour.
pub(super) fn merge(a: &Store, b: &Store) -> Store {
    let mut by_key: HashMap<Key, Entry> = HashMap::new();
    for e in a.entries().iter().chain(b.entries()) {
        by_key.insert(e.key().clone(), e.clone());
    }
    let mut entries: Vec<Entry> = by_key.into_values().collect();
    entries.sort_by(|a, b| a.key().cmp(b.key()));
    let clock = a.clock().max(b.clock());
    Store::assemble(entries, clock)
}

/// The canonical observation: the live entries at the store's own clock, as sorted
/// `(key, value, remaining life)` rows. Entries are already key-sorted, so filtering
/// preserves the canonical order.
pub(super) fn view(store: &Store) -> Snapshot {
    let rows: Vec<(Key, Val, Ttl)> = store
        .entries()
        .iter()
        .filter(|e| e.live_at(store.clock()))
        .map(|e| (e.key().clone(), e.val(), e.remaining_at(store.clock())))
        .collect();
    Snapshot::from_rows(rows)
}
