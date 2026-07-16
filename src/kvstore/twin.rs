use std::collections::BTreeMap;

use crate::kvstore::store::{Clock, Key, Snapshot, Ttl, Val};
use crate::kvstore::theory::Sort;

/// The twin carrier: the same observable store over a genuinely different
/// representation — a key-ordered map from key to (value, ABSOLUTE expiry
/// instant), where the primary keeps a canonically sorted entry list with a
/// (born, ttl) split. No `Entry`, no `Vec`, no relative life stored anywhere.
/// It exists to be swapped under the ttl-store theory: if the discovered law
/// list moves, the spec language leaked representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TwinStore {
    slots: BTreeMap<Key, (Val, Clock)>,
    clock: Clock,
}

impl TwinStore {
    /// The empty store at the epoch.
    pub fn new() -> Self {
        TwinStore {
            slots: BTreeMap::new(),
            clock: Clock::start(),
        }
    }
    /// Bind `key`, replacing any existing binding; the expiry is absolute from
    /// the moment of writing. Dead bindings linger until the next tick, exactly
    /// as the primary's unswept entries do.
    pub fn put(&self, key: Key, val: Val, ttl: Ttl) -> TwinStore {
        let mut slots = self.slots.clone();
        slots.insert(key, (val, self.clock.advanced(ttl)));
        TwinStore {
            slots,
            clock: self.clock,
        }
    }
    /// Advance the clock and eagerly sweep everything no longer live (live is
    /// strictly before expiry — the same one comparison the primary hangs on).
    pub fn tick(&self, by: Ttl) -> TwinStore {
        let clock = self.clock.advanced(by);
        let slots = self
            .slots
            .iter()
            .filter(|(_, (_, expires))| clock < *expires)
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        TwinStore { slots, clock }
    }
    /// Right-biased union at the later clock: where both bind a key, `other`
    /// wins. Raw bindings merge — dead ones included — as in the primary.
    pub fn merge(&self, other: &TwinStore) -> TwinStore {
        let mut slots = self.slots.clone();
        for (k, v) in &other.slots {
            slots.insert(k.clone(), *v);
        }
        TwinStore {
            slots,
            clock: self.clock.max(other.clock),
        }
    }
    /// The canonical observation — identical Snapshot type, clock-relative
    /// remaining life, key-sorted by the map's own order.
    pub fn view(&self) -> Snapshot {
        Snapshot::from_rows(
            self.slots
                .iter()
                .filter(|(_, (_, expires))| self.clock < *expires)
                .map(|(k, (v, expires))| (k.clone(), *v, self.clock.until(*expires)))
                .collect(),
        )
    }
}

/// A value in the twin theory: a twin store or a duration (durations are shared
/// with the primary — the swap is on the store sort alone).
#[derive(Clone)]
pub enum TwinVal {
    Store(TwinStore),
    Dur(Ttl),
}

fn twin_store(v: &TwinVal) -> TwinStore {
    match v {
        TwinVal::Store(s) => s.clone(),
        TwinVal::Dur(_) => TwinStore::new(), // unreachable: the sorts keep arguments apart.
    }
}

fn twin_dur(v: &TwinVal) -> Ttl {
    match v {
        TwinVal::Dur(d) => *d,
        TwinVal::Store(_) => Ttl::zero(), // unreachable: the sorts keep arguments apart.
    }
}

fn empty(_: &[TwinVal]) -> Option<TwinVal> {
    Some(TwinVal::Store(TwinStore::new()))
}

fn merge(v: &[TwinVal]) -> Option<TwinVal> {
    Some(TwinVal::Store(twin_store(&v[0]).merge(&twin_store(&v[1]))))
}

/// The doctored merge the fire drill swaps in: FIRST write wins. A behaviour
/// change, not a representation change — the discovered bias law must move, or
/// the swap drill is decoration.
fn merge_first_wins(v: &[TwinVal]) -> Option<TwinVal> {
    Some(TwinVal::Store(twin_store(&v[1]).merge(&twin_store(&v[0]))))
}

fn tick(v: &[TwinVal]) -> Option<TwinVal> {
    Some(TwinVal::Store(twin_store(&v[0]).tick(twin_dur(&v[1]))))
}

fn zero(_: &[TwinVal]) -> Option<TwinVal> {
    Some(TwinVal::Dur(Ttl::zero()))
}

fn plus(v: &[TwinVal]) -> Option<TwinVal> {
    Some(TwinVal::Dur(twin_dur(&v[0]).plus(twin_dur(&v[1]))))
}

fn key(s: &str) -> Key {
    Key::new(s).expect("a valid key")
}

fn val(n: i64) -> Val {
    Val::new(n).expect("a valid value")
}

fn ttl(n: i64) -> Ttl {
    Ttl::new(n).expect("a valid ttl")
}

/// The primary's inhabitant spread, rebuilt through the twin's own edges — the
/// same conflicts and the same clock skew, so the grid refutes the same false
/// laws for the same reasons.
fn twin_stores() -> Vec<TwinVal> {
    let e = TwinStore::new();
    vec![
        e.clone(),
        e.put(key("a"), val(1), ttl(2)),
        e.put(key("a"), val(2), ttl(5)),
        e.put(key("b"), val(3), ttl(3)),
        e.put(key("a"), val(4), ttl(4))
            .put(key("b"), val(5), ttl(1)),
        e.put(key("a"), val(6), ttl(1)).tick(ttl(2)),
    ]
    .into_iter()
    .map(TwinVal::Store)
    .collect()
}

fn twin_durs() -> Vec<TwinVal> {
    [0i64, 1, 2, 5]
        .into_iter()
        .map(|n| TwinVal::Dur(ttl(n)))
        .collect()
}

// The twin theory declarations. Same sorts, same variable letters, same operator
// display names and symbols as the primary — so a discovered law renders to the
// same bytes, and the only thing that can differ is WHICH laws hold.
crate::theory! {
    TtlStoreTwin : "ttl store twin", Value = TwinVal, Obs = (u8, Snapshot, Ttl), Sort = Sort,
    sort_of = |v: &TwinVal| match v {
        TwinVal::Store(_) => Sort::Store,
        TwinVal::Dur(_) => Sort::Duration,
    },
    observe = |v: &TwinVal| match v {
        TwinVal::Store(s) => (0u8, s.view(), Ttl::zero()),
        TwinVal::Dur(d) => (1u8, Snapshot::empty(), *d),
    },
    vars {
        Sort::Store => &["s", "t", "u"],
        Sort::Duration => &["p", "q", "r"],
    }
    inhabit {
        Sort::Store => twin_stores(),
        Sort::Duration => twin_durs(),
    }
    ops {
        Nullary "Empty" "empty" () -> Sort::Store = empty;
        Infix   "Merge" "<+"    (Sort::Store, Sort::Store) -> Sort::Store = merge;
        Prefix  "Tick"  "tick"  (Sort::Store, Sort::Duration) -> Sort::Store = tick;
        Nullary "Zero"  "zero"  () -> Sort::Duration = zero;
        Infix   "Plus"  "+"     (Sort::Duration, Sort::Duration) -> Sort::Duration = plus;
    }
}

// The fire-drill theory: identical except merge is first-write-wins. If swapping
// THIS in leaves the law list fixed, the drill cannot fire and proves nothing.
crate::theory! {
    FirstWinsTwin : "ttl store twin (first write wins)", Value = TwinVal, Obs = (u8, Snapshot, Ttl), Sort = Sort,
    sort_of = |v: &TwinVal| match v {
        TwinVal::Store(_) => Sort::Store,
        TwinVal::Dur(_) => Sort::Duration,
    },
    observe = |v: &TwinVal| match v {
        TwinVal::Store(s) => (0u8, s.view(), Ttl::zero()),
        TwinVal::Dur(d) => (1u8, Snapshot::empty(), *d),
    },
    vars {
        Sort::Store => &["s", "t", "u"],
        Sort::Duration => &["p", "q", "r"],
    }
    inhabit {
        Sort::Store => twin_stores(),
        Sort::Duration => twin_durs(),
    }
    ops {
        Nullary "Empty" "empty" () -> Sort::Store = empty;
        Infix   "Merge" "<+"    (Sort::Store, Sort::Store) -> Sort::Store = merge_first_wins;
        Prefix  "Tick"  "tick"  (Sort::Store, Sort::Duration) -> Sort::Store = tick;
        Nullary "Zero"  "zero"  () -> Sort::Duration = zero;
        Infix   "Plus"  "+"     (Sort::Duration, Sort::Duration) -> Sort::Duration = plus;
    }
}

/// The twin theory — the same declaration as the primary over the swapped carrier.
pub struct TtlStoreTwin;

/// The doctored theory the fire drill discovers AGAINST — first write wins.
pub struct FirstWinsTwin;

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::engine::{DiscoveredLaw, Engine, Theory};
    use crate::kvstore::theory::TtlStore;

    fn laws<T: Theory>() -> Vec<DiscoveredLaw> {
        Engine::<T>::new().discover().laws
    }

    /// THE SWAP DRILL: the twin carrier — a different representation under the
    /// same operations, variables, and observation — must discover the
    /// byte-identical law list. A differing line is a representation leak in the
    /// spec language itself: the theory failed to name an equivalence class.
    #[test]
    fn the_twin_carrier_discovers_the_byte_identical_spec() {
        assert_eq!(TtlStoreTwin::name(), "ttl store twin");
        let primary: Vec<(String, String)> = laws::<TtlStore>()
            .into_iter()
            .map(|l| (l.prose, l.equation))
            .collect();
        let twin: Vec<(String, String)> = laws::<TtlStoreTwin>()
            .into_iter()
            .map(|l| (l.prose, l.equation))
            .collect();
        assert_eq!(
            primary, twin,
            "the spec language leaked representation: a law list moved under carrier swap"
        );
    }

    /// The drill can FIRE: a behaviour change (first write wins) must move the
    /// law list — the bias law names which side wins, so it cannot survive the
    /// flip. A swap drill that greens under a behaviour change is decoration.
    #[test]
    fn the_swap_drill_fires_on_a_behaviour_change() {
        let primary: Vec<(String, String)> = laws::<TtlStore>()
            .into_iter()
            .map(|l| (l.prose, l.equation))
            .collect();
        let doctored: Vec<(String, String)> = laws::<FirstWinsTwin>()
            .into_iter()
            .map(|l| (l.prose, l.equation))
            .collect();
        assert_ne!(
            primary, doctored,
            "the drill went vacuous: a first-write-wins merge left the law list fixed"
        );
        let bias = "the later operand wins";
        assert!(
            primary.iter().any(|(prose, _)| prose.contains(bias)),
            "the primary lost its bias law — the drill's teeth depend on it"
        );
        assert!(
            !doctored.iter().any(|(prose, _)| prose.contains(bias)),
            "first-write-wins still discovers a later-operand-wins bias — the flip did nothing"
        );
    }

    /// Equivalence at the observation level, spot-checked: the same edge
    /// sequence through both carriers yields the same snapshot. The law drill
    /// samples the theory's grid; this pins one concrete trace end to end.
    #[test]
    fn one_trace_agrees_end_to_end() {
        let p = crate::kvstore::store::Store::new()
            .put(key("a"), val(1), ttl(3))
            .put(key("b"), val(2), ttl(1))
            .tick(ttl(1))
            .put(key("a"), val(4), ttl(2));
        let t = TwinStore::new()
            .put(key("a"), val(1), ttl(3))
            .put(key("b"), val(2), ttl(1))
            .tick(ttl(1))
            .put(key("a"), val(4), ttl(2));
        assert_eq!(p.view(), t.view());
        assert_eq!(p.merge(&p).view(), t.merge(&t).view());
    }
}
