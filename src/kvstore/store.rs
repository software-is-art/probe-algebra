//!
//! kvstore::store — a TTL key-value store as a boundary CATEGORY.
//!
//! NOTE THE FILENAME. Every other domain calls this file `boundary.rs`; this one is
//! `store.rs` ON PURPOSE, to demonstrate that boundary-hood is DERIVED structure — this
//! file is pub-reachable and carries production edges, which is what places it BOUNDARY
//! in `spec/tiers.spec` and subjects it to the full tier-1 grammar in `build.rs` — and
//! not a filename convention. (The qualification census makes the same point from the
//! other side: a module qualifies as an algebra by STRUCTURE, wherever it lives.) If
//! renaming the file changed what the rules enforced, the rigidity would live in the
//! path, not the boundary.
//!
//! The domain is the crate's first STATEFUL one, and the design keeps `Stateful` honest
//! without `Effectful`: the `Store` value object carries its own logical `Clock`, and time
//! advances ONLY through the explicit `Tick` edge — there is no ambient now, so every
//! path is deterministic and replayable. An entry is live while `clock < born + ttl`
//! (strictly), so expiry is a fact about two value objects, not about the world.
//!
//! The edges:
//!   - `ParseKey` — the parse-don't-validate ENTRY edge (a pure `Construction`);
//!   - `Put` — a `Stateful` `Morphism` whose `Displaced` residual witnesses EXACTLY the
//!     binding the write clobbered, so overwriting is invertible;
//!   - `Tick` — a `Stateful` `Morphism` that advances the clock and EAGERLY SWEEPS the
//!     newly-dead entries; its `Swept` residual carries them, so even expiry reverses;
//!   - `Get`/`Read` — the `Check`/`Eval` pattern on state: `Get` is a `Branch` minting a
//!     name-branded `Live`/`Gone` liveness witness, and `Read` is the `Guarded` edge that
//!     is UNCALLABLE without a `Live` proof for the SAME lookup — "reads of dead entries
//!     don't happen" is a compile-time fact, not a runtime `Option`.

use core::marker::PhantomData;

use crate::boundary::{Branch, Construction, Guarded, InputEffect, Morphism, Pure, Stateful, Unit};
use crate::gdp::Named;

// ===== value objects: the store's nouns ====================================

// The validity rule is the only content; `refined!` generates the rest.
crate::refined! {
    /// A key: a non-empty, all-ASCII-alphabetic identifier (the `Ident` rule, minus the
    /// interpreter's keyword list — this domain has no keywords to collide with).
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Key(String);
    fn new(s: &str) = {
        let ok = !s.is_empty() && s.chars().all(|c| c.is_ascii_alphabetic());
        ok.then(|| s.to_string())
    };
}
impl Key {
    /// The raw text — the sanctioned exit hatch.
    pub fn get(&self) -> &str {
        &self.0
    }
}

crate::refined! {
    /// A stored value: a non-negative integer (the `Int` pattern — the payload is kept
    /// scalar so the state machinery, not the payload, is what this domain exercises).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Val(i64);
    fn new(n: i64) = (n >= 0).then_some(n);
}
impl Val {
    pub fn get(&self) -> i64 {
        self.0
    }
    /// The default the `Live` proof rules out — `Read`'s total-function fallback for the
    /// branch that cannot be reached under the witness (see `internal::lookup`).
    pub fn zero() -> Self {
        Val(0)
    }
}

crate::refined! {
    /// A duration in logical ticks — an entry's time-to-live at `Put`, and the step a
    /// `Tick` advances by. One value object wears both hats deliberately: the tick monoid
    /// the discovery engine finds (`Plus`/`zero`) is the SAME algebra the TTLs live in.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Ttl(i64);
    fn new(n: i64) = (n >= 0).then_some(n);
}
impl Ttl {
    pub fn get(&self) -> i64 {
        self.0
    }
    /// The monoid identity: the tick that changes nothing, the TTL that never lives.
    pub fn zero() -> Self {
        Ttl(0)
    }
    /// Saturating addition (total; two non-negatives stay non-negative).
    pub fn plus(self, other: Ttl) -> Ttl {
        Ttl(self.0.saturating_add(other.0))
    }
}

crate::refined! {
    /// A logical instant: how many ticks this store has lived. MONOTONE by construction —
    /// the only operator that moves it is `advanced`, and only the `Tick` edge calls it —
    /// which is what keeps the whole domain deterministic (`Stateful`, never `Effectful`).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Clock(i64);
    fn new(n: i64) = (n >= 0).then_some(n);
}
impl Clock {
    /// The epoch — a fresh store's clock.
    pub fn start() -> Self {
        Clock(0)
    }
    pub fn get(&self) -> i64 {
        self.0
    }
    /// This instant moved forward by a duration (saturating: the clock never wraps).
    pub fn advanced(self, by: Ttl) -> Clock {
        Clock(self.0.saturating_add(by.get()))
    }
    /// The inverse of `advanced`, where it exists — `Tick::backward`'s rewind. `None` if
    /// rewinding would cross the epoch (no genuine tick output ever does).
    pub fn rewound(self, by: Ttl) -> Option<Clock> {
        Clock::new(self.0 - by.get())
    }
    /// The duration from this instant to a later one (zero if `later` is not later).
    pub fn until(self, later: Clock) -> Ttl {
        Ttl((later.0 - self.0).max(0))
    }
}

/// One stored binding: a key, its value, the instant it was written (`born`), and its
/// time-to-live. Liveness is DERIVED, never stored — an entry is live at `at` iff
/// `at < born + ttl` (strict, so a zero TTL is dead on arrival) — which is what lets the
/// eager sweep and the lazy view be two routes to one answer (see `probes`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    key: Key,
    val: Val,
    born: Clock,
    ttl: Ttl,
}
impl Entry {
    /// Assembled only by the interior (`internal::put`) — an entry's `born` is always the
    /// writing store's clock, never a caller-supplied instant.
    pub(crate) fn new(key: Key, val: Val, born: Clock, ttl: Ttl) -> Self {
        Entry {
            key,
            val,
            born,
            ttl,
        }
    }
    pub fn key(&self) -> &Key {
        &self.key
    }
    pub fn val(&self) -> Val {
        self.val
    }
    pub(crate) fn ttl(&self) -> Ttl {
        self.ttl
    }
    /// The instant this entry stops being live.
    pub fn expires_at(&self) -> Clock {
        self.born.advanced(self.ttl)
    }
    /// Is this entry live at `at`? Strictly before expiry — the one comparison the whole
    /// expiry semantics hangs on (the probes pin it from both sides).
    pub fn live_at(&self, at: Clock) -> bool {
        at < self.expires_at()
    }
    /// The life this entry has left at `at` (zero once dead) — the clock-RELATIVE datum
    /// the canonical `Snapshot` observes, so two stores at different absolute clocks can
    /// still look the same.
    pub fn remaining_at(&self, at: Clock) -> Ttl {
        at.until(self.expires_at())
    }
}

/// THE state: the entries plus the store's own logical clock, as one immutable value
/// object. Every edge takes a store and returns a NEW store — carried state is a citizen,
/// not a mutable global — so `Stateful` here means "output depends on carried state",
/// never "something happened". Entries are kept canonically sorted by key with at most
/// one entry per key, so structural equality is semantic equality of the representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    entries: Vec<Entry>,
    clock: Clock,
}
impl Default for Store {
    fn default() -> Self {
        Store::new()
    }
}
impl Store {
    /// The empty store at the epoch — the identity of `merge`, as the discovered spec
    /// certifies.
    pub fn new() -> Self {
        Store {
            entries: Vec::new(),
            clock: Clock::start(),
        }
    }
    pub fn clock(&self) -> Clock {
        self.clock
    }
    /// The canonical observation: the LIVE entries at this store's clock, sorted, with
    /// clock-relative remaining life. This is the `observe` the discovery engine judges
    /// laws through, so expiry is VISIBLE to the algebra.
    pub fn view(&self) -> Snapshot {
        super::internal::view(self)
    }
    /// Right-biased union — last write wins where both stores bind a key — at the LATER
    /// of the two clocks (so merging with a later store can silently expire your entries:
    /// a consequence the engine can see through `view`). A value operator, not an edge:
    /// its whole specification is the discovered monoid in `theory`.
    pub fn merge(&self, other: &Store) -> Store {
        super::internal::merge(self, other)
    }
    /// Convenience routed THROUGH the `Put` edge (so builders and generators exercise the
    /// probed path, never a private shortcut).
    pub fn put(&self, key: Key, val: Val, ttl: Ttl) -> Store {
        Put.forward(&Write::new(self.clone(), key, val, ttl)).0
    }
    /// Convenience routed THROUGH the `Tick` edge.
    pub fn tick(&self, by: Ttl) -> Store {
        Tick.forward(&Advance::new(self.clone(), by)).0
    }
    /// The raw entries — the sanctioned accessor the interior (and the probes'
    /// independent recomputations) read through.
    pub(crate) fn entries(&self) -> &[Entry] {
        &self.entries
    }
    /// Assembled only by the interior, which owns the sorted-unique-keys invariant.
    pub(crate) fn assemble(entries: Vec<Entry>, clock: Clock) -> Store {
        Store { entries, clock }
    }
}

/// The canonical live snapshot: `(key, value, remaining life)` rows, sorted by key. The
/// store's OBSERVATION — what the discovery engine fingerprints and what two-route probes
/// compare — kept clock-relative so observational equality is about what a reader could
/// ever see, not about internal bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snapshot(Vec<(Key, Val, Ttl)>);
impl Snapshot {
    /// The empty observation (also the theory's placeholder for the non-store sort).
    pub fn empty() -> Self {
        Snapshot(Vec::new())
    }
    pub fn rows(&self) -> &[(Key, Val, Ttl)] {
        &self.0
    }
    pub(crate) fn from_rows(rows: Vec<(Key, Val, Ttl)>) -> Self {
        Snapshot(rows)
    }
}

/// A store PAIRED with the binding to write — `Put`'s input. Bundling the state INTO the
/// input keeps `Morphism`'s signature while modelling a stateful edge (the `Bound`
/// pattern from `interp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Write {
    store: Store,
    key: Key,
    val: Val,
    ttl: Ttl,
}
impl Write {
    pub fn new(store: Store, key: Key, val: Val, ttl: Ttl) -> Self {
        Write {
            store,
            key,
            val,
            ttl,
        }
    }
    pub fn store(&self) -> &Store {
        &self.store
    }
    pub fn key(&self) -> &Key {
        &self.key
    }
    pub fn val(&self) -> Val {
        self.val
    }
    pub fn ttl(&self) -> Ttl {
        self.ttl
    }
}

/// A store paired with the key to look up — the input `Get` classifies and `Read`
/// consumes (under `Get`'s witness).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lookup {
    store: Store,
    key: Key,
}
impl Lookup {
    pub fn new(store: Store, key: Key) -> Self {
        Lookup { store, key }
    }
    pub fn store(&self) -> &Store {
        &self.store
    }
    pub fn key(&self) -> &Key {
        &self.key
    }
}

/// A store paired with the duration to advance by — `Tick`'s input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advance {
    store: Store,
    by: Ttl,
}
impl Advance {
    pub fn new(store: Store, by: Ttl) -> Self {
        Advance { store, by }
    }
    pub fn store(&self) -> &Store {
        &self.store
    }
    pub fn by(&self) -> Ttl {
        self.by
    }
}

/// `Put`'s residual: the binding the write DISPLACED — the written key plus the prior
/// entry it clobbered (`None` when the key was vacant). This is exactly what overwriting
/// loses, so keeping it restores invertibility: `backward` puts the prior binding back
/// and recovers the write that was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Displaced {
    key: Key,
    prior: Option<Entry>,
}

/// `Tick`'s residual: the step taken plus the entries the eager sweep EVICTED. Expiry is
/// the one genuinely destructive act in the domain; carrying the corpses is what lets
/// `backward` rewind the clock and resurrect them exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Swept {
    by: Ttl,
    evicted: Vec<Entry>,
}

// `Key`/`Val`/`Ttl`/`Clock` register themselves via `refined!`; the rest register here.
crate::value_object!(Entry, Store, Snapshot, Write, Lookup, Advance, Displaced, Swept);

// Capability STATE FLOOR, inferred from the input type (see `boundary::InputEffect`):
// each of these carries a `Store`, so consuming one is at least `Stateful` whatever the
// body does — an under-declared edge over them cannot be demanded pure.
impl InputEffect for Write {
    type Eff = Stateful;
}
impl InputEffect for Lookup {
    type Eff = Stateful;
}
impl InputEffect for Advance {
    type Eff = Stateful;
}

// ===== the proof tokens: the liveness witnesses ============================

crate::proof_token!(
    /// A proof that the lookup named `N` is LIVE: its key is bound and unexpired at the
    /// store's own clock. Branded, so a proof for one lookup cannot authorize reading
    /// another; minted ONLY by `Get::classify`. It is the witness `Read` demands — the
    /// type-level statement of "reads of dead entries don't happen".
    Live
);
crate::proof_token!(
    /// A proof that the lookup named `N` is GONE — absent or expired, deliberately ONE
    /// witness: at the store's clock the two are indistinguishable to any reader, and
    /// pretending otherwise would leak bookkeeping the boundary does not promise. The
    /// NEGATIVE witness is kept (not discarded as a `None`) so the rejection path is
    /// first-class.
    Gone
);

// ===== the boundary edges ==================================================

/// The ENTRY edge: parse raw text into a `Key`, or reject it. A pure refinement — no
/// normalization — so its residual is `Unit` and reconstruction is the key's own text.
/// Modelled as a `Construction`, so even key admission is inside the probe space.
pub struct ParseKey;
/// The writer: a `Stateful` `Morphism` `Write -> Store` that binds (or overwrites) a key
/// at the store's current clock; its `Displaced` residual witnesses the clobbered binding.
pub struct Put;
/// The classifier: a `Branch` minting `Live<N> + Gone<N>` for a named `Lookup`, running
/// the REAL liveness check (a proof is only as true as its mint). `Stateful` — it reads
/// the carried store.
pub struct Get;
/// The reader: a `Guarded` edge `Lookup -> Val` admitted by a `Live<N>` for the SAME
/// name. You cannot read what has not been proven live — the witness comes from `Get`,
/// exactly as `Eval` demands `Check`'s.
pub struct Read;
/// Time itself, as an edge: advance the store's clock by a duration and eagerly sweep
/// the entries that die on the way. The ONLY operator that moves a `Clock` — which is
/// the whole determinism story — and the action the discovered spec certifies
/// (`tick(tick(s, p), q) = tick(s, p + q)`).
pub struct Tick;
crate::value_operator!(ParseKey, Put, Get, Read, Tick);

impl Construction for ParseKey {
    type Capability = Pure;
    type Raw = String;
    type Refined = Key;
    type Residual = Unit;

    fn parse(&self, raw: &String) -> Option<(Key, Unit)> {
        Key::new(raw).map(|k| (k, Unit))
    }
    fn reconstruct(&self, refined: &Key, _residual: &Unit) -> Option<String> {
        Some(refined.get().to_string())
    }
}

impl Morphism for Put {
    type Capability = Stateful;
    type In = Write;
    type Out = Store;
    // The residual witnesses EXACTLY what the write destroyed: the prior binding.
    type Residual = Displaced;

    fn forward(&self, input: &Write) -> (Store, Displaced) {
        let (store, prior) =
            super::internal::put(input.store(), input.key(), input.val(), input.ttl());
        (
            store,
            Displaced {
                key: input.key().clone(),
                prior,
            },
        )
    }

    /// Undo the write: read the written value and TTL off the output's own entry (its
    /// `born` is the clock, so the entry is identifiable by key alone), then restore the
    /// displaced binding. Total for genuine outputs; `None` only if `out` does not carry
    /// the residual's key at all — then it is not this edge's output.
    fn backward(&self, out: &Store, residual: &Displaced) -> Option<Write> {
        let written = out.entries().iter().find(|e| e.key() == &residual.key)?;
        let (val, ttl) = (written.val(), written.ttl());
        let store = super::internal::unput(out, &residual.key, residual.prior.clone());
        Some(Write::new(store, residual.key.clone(), val, ttl))
    }
}

impl Morphism for Tick {
    type Capability = Stateful;
    type In = Advance;
    type Out = Store;
    // The residual witnesses EXACTLY what expiry destroyed: the swept entries + the step.
    type Residual = Swept;

    fn forward(&self, input: &Advance) -> (Store, Swept) {
        let (store, evicted) = super::internal::tick(input.store(), input.by());
        (
            store,
            Swept {
                by: input.by(),
                evicted,
            },
        )
    }

    /// Undo time: rewind the clock and resurrect the swept entries. `None` only if the
    /// rewind would cross the epoch, which no genuine output's residual asks for.
    fn backward(&self, out: &Store, residual: &Swept) -> Option<Advance> {
        let clock = out.clock().rewound(residual.by)?;
        let store = super::internal::restore(out, &residual.evicted, clock);
        Some(Advance::new(store, residual.by))
    }
}

impl Branch for Get {
    type Capability = Stateful;
    type In<N> = Named<N, Lookup>;
    type Left<N> = Live<N>;
    type Right<N> = Gone<N>;

    fn branch<N>(&self, lookup: &Named<N, Lookup>) -> Result<Live<N>, Gone<N>> {
        let l = lookup.value();
        if super::internal::lookup(l.store(), l.key()).is_some() {
            Ok(Live(PhantomData))
        } else {
            Err(Gone(PhantomData))
        }
    }
}

impl Get {
    /// Classify a named lookup — the ergonomic name for the `Branch` edge. Both arms
    /// carry a proof; a dead read is a `Gone` witness, never a silent `None`.
    pub fn classify<N>(&self, lookup: &Named<N, Lookup>) -> Result<Live<N>, Gone<N>> {
        self.branch(lookup)
    }
}

impl Guarded for Read {
    type Capability = Stateful;
    type In<N> = Named<N, Lookup>;
    type Proof<N> = Live<N>;
    // Reading KEEPS the lookup's brand: the value is provably the value of the lookup
    // that was classified live, not some other.
    type Out<N> = Named<N, Val>;

    fn guard<N>(&self, lookup: &Named<N, Lookup>, _proof: &Live<N>) -> Named<N, Val> {
        // The `unwrap_or` arm is unreachable under the `Live` proof (the witness rules
        // out absence); a defined default keeps the edge total, exactly like `eval`'s
        // proof-dead arms.
        lookup.map(|l| super::internal::lookup(l.store(), l.key()).unwrap_or(Val::zero()))
    }
}

impl Read {
    /// Read a proven-live, named lookup — the ergonomic name for the `Guarded` edge.
    /// Requires a `Live<N>` for the same name; there is no other way to reach a `Val`.
    pub fn run<N>(&self, lookup: &Named<N, Lookup>, proof: &Live<N>) -> Named<N, Val> {
        self.guard(lookup, proof)
    }
}
