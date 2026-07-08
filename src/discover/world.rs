//!
//! world — EFFECTS AS THEORIES, and the WORLD LOCK: the freeze discipline pointed outward.
//!
//! The library keeps I/O at the edges because effects resist the mental model. The kvstore
//! found the crack in that wall — TIME became a value (`Tick` is an explicit edge) and a
//! stateful domain became just another discovered algebra. This module walks the same move
//! outward, to an EXTERNAL dependency, in the project's own vocabulary: *derive the spec by
//! running the thing, freeze it, gate the drift.*
//!
//!   1. **Effects are values.** A dependency is modeled by a [`Command`] value object and a
//!      [`Trace`] — a sequence of commands — which is itself a value with a concatenation
//!      algebra.
//!   2. **The mental model is an interpreter, and the interpreter is the observation.** The
//!      pure [`StoreModel`] plays the role `observe` plays everywhere else, so the EXISTING
//!      engine discovers PROTOCOL laws with no new machinery: `idempotent(++)` literally IS
//!      replay/retry safety, `bias_later(++)` IS last-write-wins, `identity(++, empty)` says
//!      an empty batch is harmless. The declared expectations gate them like any theory's.
//!   3. **The probe battery derives itself.** `#[derive(Shaped)]` on [`Command`] gives the
//!      canonical battery of traces via `shadow_grid` — structure-first, so every command
//!      constructor appears before value tuning spends the cap.
//!   4. **The world lock.** You cannot ratify the world, but you CAN ratify your assumptions
//!      about it: [`WorldReport`] records an observer's conduct over the derived battery and
//!      freezes it into `spec/<dependency>.world.spec`. Two seams are then drift-gated —
//!      MODEL vs the committed lock (did our beliefs change?) and ADAPTER vs the same lock
//!      (does reality still honour them?) — and a divergence names the exact trace where the
//!      world left our assumptions.
//!
//! The demonstration dependency here ([`FakeRemoteStore`]) is a deliberately INDEPENDENT
//! implementation — event-sourced internals standing in for a vendor — so its conformance
//! replay is pure and runs in every test pass. Against a REAL remote dependency the replay
//! is the one deliberately-Effectful gate: run it behind a bless flag in integration CI,
//! record, ratify the diff. Honest frame unchanged: the battery is grid-bounded, so the
//! lock refutes conformance, it never proves it.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use spec_lock::Lock;

use crate::boundary::{shadow_grid, Shaped};

// ===== the modeled dependency: commands, traces, and the pure world model ==================

/// A key of the modeled store — a tiny closed sort, so the derived battery stays exhaustive
/// where the type is finite (the partitioned grid's sweet spot).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, crate::Shaped)]
pub enum Key {
    A,
    B,
}

/// A value of the modeled store.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, crate::Shaped)]
pub enum Val {
    V0,
    V1,
}

/// One effect-describing value: what a client can ask the dependency to do. The DERIVED
/// probe surface of this enum is what makes the integration battery write itself.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, crate::Shaped)]
pub enum Command {
    /// Write `Val` at `Key` (the vendor promises overwrite).
    Put(Key, Val),
    /// Remove `Key` (absent keys tolerated).
    Del(Key),
}

/// A TRACE: the commands a session issued, in order — the value the protocol algebra ranges
/// over. `++` is concatenation, `empty` the idle session.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Trace(pub Vec<Command>);

/// The grid grows a trace two ways: APPEND each reachable command (via `Command`'s own
/// derived grid — every constructor, both keys, both values), and DROP the last command
/// (back toward `empty`). The closure under these reaches every short session shape.
#[crate::mutate]
impl Shaped for Trace {
    fn inhabitant() -> Self {
        Trace(Vec::new())
    }
    fn perturbation_classes(&self) -> Vec<Vec<Self>> {
        let grown: Vec<Trace> = shadow_grid::<Command>(8)
            .into_iter()
            .map(|command| {
                let mut commands = self.0.clone();
                commands.push(command);
                Trace(commands)
            })
            .collect();
        let shrunk: Vec<Trace> = if self.0.is_empty() {
            Vec::new()
        } else {
            vec![Trace(self.0[..self.0.len() - 1].to_vec())]
        };
        vec![grown, shrunk]
    }
}

/// What a session leaves behind: the store's final contents. The OBSERVATION of the
/// protocol theory — deterministic, ordered, diffable.
pub type State = BTreeMap<Key, Val>;

/// The pure WORLD MODEL — our beliefs about the dependency, as an interpreter. This is the
/// mental model made checkable: the protocol theory observes traces through it, and the
/// world lock freezes its predictions.
pub struct StoreModel;

#[crate::mutate]
impl StoreModel {
    /// Replay a trace against the model: `Put` overwrites, `Del` removes (absent tolerated).
    pub fn replay(trace: &Trace) -> State {
        let mut state = State::new();
        for command in &trace.0 {
            match command {
                Command::Put(key, val) => {
                    state.insert(*key, *val);
                }
                Command::Del(key) => {
                    state.remove(key);
                }
            }
        }
        state
    }

    /// The model's BELIEFS as a world report: its predicted observation for every battery
    /// trace. Freeze this (`examples/freeze_spec.rs`) — the committed file is the ratified
    /// assumption set the conformance gate holds reality to.
    pub fn beliefs() -> WorldReport {
        WorldReport::record("store model", BATTERY_CAP, StoreModel::replay)
    }
}

#[crate::mutate]
fn empty_trace(_: &[Trace]) -> Option<Trace> {
    Some(Trace(Vec::new()))
}
#[crate::mutate]
fn concat_traces(v: &[Trace]) -> Option<Trace> {
    let mut commands = v[0].0.clone();
    commands.extend(v[1].0.iter().copied());
    Some(Trace(commands))
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct P;

/// The protocol THEORY marker: traces under `++`/`empty`, observed through the model.
pub struct StoreProtocol;

crate::theory! {
    StoreProtocol : "store protocol",
    Value = Trace,
    Obs = State,
    Sort = P,
    sort_of = |_: &Trace| P,
    observe = |t: &Trace| StoreModel::replay(t),
    ops {
        Nullary "empty" "empty" () -> P = empty_trace;
        Infix   "++"    "++"    (P, P) -> P = concat_traces;
    }
    expects {
        associative("++");
        idempotent("++");
        bias_later("++");
        identity("++", empty);
    }
}

// ===== the world lock =======================================================================

/// How many traces the canonical battery carries (`shadow_grid` over [`Trace`], structure
/// first): the empty session, every single-command session, and a spread of two-command
/// sessions — enough to refute overwrite, delete, and ordering misbeliefs.
pub const BATTERY_CAP: usize = 24;

/// An observer's recorded conduct over the derived battery — the world lock's value object.
/// `record` it twice: once through the MODEL (beliefs — what freezes into
/// `spec/<dependency>.world.spec`) and once through the real ADAPTER (reality — what the
/// conformance gate holds against the same committed text).
pub struct WorldReport {
    /// Whose conduct this is ("store model", "fake remote store", a vendor's name).
    pub observer: &'static str,
    /// One row per battery trace: `(the trace, the observation)`, both rendered.
    pub rows: Vec<(String, String)>,
}

#[crate::mutate]
impl WorldReport {
    /// Record an observer over the canonical battery of `V` (derived from the value's own
    /// `Shaped` surface, structure-first). Deterministic given a deterministic observer —
    /// the lock discipline's one hard requirement.
    pub fn record<V, S>(
        observer: &'static str,
        cap: usize,
        observe: impl Fn(&V) -> S,
    ) -> WorldReport
    where
        V: Shaped + core::fmt::Debug,
        S: core::fmt::Debug,
    {
        WorldReport {
            observer,
            rows: shadow_grid::<V>(cap)
                .iter()
                .map(|value| (format!("{value:?}"), format!("{:?}", observe(value))))
                .collect(),
        }
    }

    /// The rows where two reports DISAGREE — each rendered with the trace and both
    /// observations, so a conformance failure names exactly where the world left the
    /// model's assumptions. Empty iff the observers agree over the whole battery.
    pub fn disagreements(&self, other: &WorldReport) -> Vec<String> {
        if self.rows.len() != other.rows.len() {
            return vec![format!(
                "the batteries differ in size ({} vs {}) — the reports were not recorded \
                 over the same derived battery",
                self.rows.len(),
                other.rows.len()
            )];
        }
        self.rows
            .iter()
            .zip(&other.rows)
            .filter(|((_, ours), (_, theirs))| ours != theirs)
            .map(|((trace, ours), (_, theirs))| {
                format!(
                    "on {trace}: {} observes {ours}, {} observes {theirs}",
                    self.observer, other.observer
                )
            })
            .collect()
    }

    /// The canonical text — deterministic, human-readable, diffable: one stanza per battery
    /// row. This is what `spec/<slug>.world.spec` locks.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# world lock: {} — recorded conduct over the derived trace battery; regenerate via this repo's freeze path and ratify the diff.\n\
             #\n\
             # You cannot ratify the world, but you can ratify your ASSUMPTIONS about it: each\n\
             # row is a canonical trace (derived from the command type's own structure) and the\n\
             # observation the model predicts. The conformance gate replays the same battery\n\
             # against the real dependency and holds it to this text — a divergence names the\n\
             # exact trace where the world left the assumptions. Grid-bounded, like all\n\
             # discovery: the battery refutes conformance, it never proves it.\n",
            self.observer
        );
        for (trace, observation) in &self.rows {
            out.push_str(&format!("\n- {trace}\n      -> {observation}\n"));
        }
        out
    }

    /// This report as a `spec_lock::Lock` rooted in a caller-supplied spec directory — the
    /// sibling of `Spec::lock_in` / `SystemReport::lock_in`, at
    /// `spec_dir/<slugified-observer>.world.spec`.
    pub fn lock_in(&self, spec_dir: &Path) -> Lock {
        let slug: String = self
            .observer
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        Lock {
            name: format!("{} world", self.observer),
            path: spec_dir.join(format!("{slug}.world.spec")),
            live: self.render(),
        }
    }

    /// This report as a lock in THIS repo's `spec/` directory (consumers use
    /// [`WorldReport::lock_in`] with their own directory).
    pub fn lock(&self) -> Lock {
        self.lock_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec"))
    }
}

// ===== the demonstration dependency: an INDEPENDENT implementation =========================

/// The stand-in vendor: an EVENT-SOURCED store — it journals every command and derives its
/// contents by folding the journal, sharing no code with [`StoreModel`]'s direct-mutation
/// interpreter. Equivalent conduct through different machinery is exactly what the
/// conformance gate exists to establish (and a real vendor is this, behind a network).
#[derive(Default)]
pub struct FakeRemoteStore {
    journal: Vec<Command>,
}

#[crate::mutate]
impl FakeRemoteStore {
    /// Accept a command (the vendor's write path: append-only).
    pub fn apply(&mut self, command: Command) {
        self.journal.push(command);
    }

    /// The vendor's read path: fold the journal into current contents.
    pub fn snapshot(&self) -> State {
        let mut contents = std::collections::HashMap::new();
        for command in &self.journal {
            match command {
                Command::Put(key, val) => {
                    contents.insert(*key, *val);
                }
                Command::Del(key) => {
                    contents.remove(key);
                }
            }
        }
        contents.into_iter().collect()
    }

    /// The dependency's CONDUCT over the canonical battery: drive a fresh store through
    /// each trace and snapshot it. Pure here (the whole point of a fixture vendor); against
    /// a real dependency this is the one deliberately-Effectful gate, run behind a bless
    /// flag in integration CI.
    pub fn conduct() -> WorldReport {
        WorldReport::record("fake remote store", BATTERY_CAP, |trace: &Trace| {
            let mut store = FakeRemoteStore::default();
            for command in &trace.0 {
                store.apply(*command);
            }
            store.snapshot()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::expect::Distance;

    /// THE PROTOCOL LAWS, discovered by the existing engine through the model observation
    /// and gated by the declaration: replay safety (`idempotent`), last-write-wins
    /// (`bias_later`), batch associativity, and the harmless empty session — met exactly,
    /// with no surprises.
    #[test]
    fn the_protocol_laws_are_discovered_and_met() {
        let distance = Distance::of::<StoreProtocol>();
        assert!(distance.is_met(), "{}", distance.render());
        assert_eq!(
            distance.render(),
            "store protocol: 4 of 4 declared laws hold; no surprises"
        );
    }

    /// The derived battery has the shape the lock leans on: the idle session first
    /// (structure seeds at the inhabitant), every single-command session (all constructors,
    /// keys, and values), and two-command sessions filling the cap.
    #[test]
    fn the_battery_derives_from_the_command_type() {
        let battery = shadow_grid::<Trace>(BATTERY_CAP);
        assert_eq!(battery.len(), BATTERY_CAP);
        assert_eq!(
            battery[0],
            Trace(Vec::new()),
            "the idle session seeds the battery"
        );
        let singles: Vec<&Trace> = battery.iter().filter(|t| t.0.len() == 1).collect();
        assert_eq!(
            singles.len(),
            6,
            "every command appears as a session of one"
        );
        assert!(
            battery.iter().any(|t| t.0.len() == 2),
            "the cap leaves room for ordering probes"
        );
    }

    /// The committed WORLD LOCK is fresh: the model's live beliefs match what was ratified.
    /// A drift here means OUR assumptions changed — regenerate with
    /// `cargo run --example freeze_spec` and ratify the diff.
    #[test]
    fn the_committed_world_lock_is_fresh() {
        let lock = StoreModel::beliefs().lock();
        if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
            panic!(
                "the world lock drifted for: {}. Run `cargo run --example freeze_spec` \
                 and ratify the diff.",
                stale.join(", ")
            );
        }
    }

    /// CONFORMANCE: the independent, event-sourced vendor honours every assumption the
    /// model froze — zero disagreements over the whole battery. (With the lock fresh, this
    /// transitively holds reality to the COMMITTED text.)
    #[test]
    fn the_fake_vendor_conforms_to_the_ratified_beliefs() {
        let disagreements = StoreModel::beliefs().disagreements(&FakeRemoteStore::conduct());
        assert_eq!(disagreements, Vec::<String>::new());
    }

    /// THE GATE HAS TEETH: a vendor whose `Put` is first-write-wins (a real bug class —
    /// "insert" where the contract says "upsert") is caught, and the disagreement NAMES the
    /// exact trace and both observations. You cannot ratify the world, but you are told, by
    /// name, when it leaves your assumptions.
    #[test]
    fn a_first_write_wins_vendor_is_named_by_trace() {
        let broken = WorldReport::record("broken vendor", BATTERY_CAP, |trace: &Trace| {
            let mut contents = State::new();
            for command in &trace.0 {
                match command {
                    Command::Put(key, val) => {
                        contents.entry(*key).or_insert(*val);
                    }
                    Command::Del(key) => {
                        contents.remove(key);
                    }
                }
            }
            contents
        });
        let disagreements = StoreModel::beliefs().disagreements(&broken);
        assert!(
            !disagreements.is_empty(),
            "an overwrite misbelief must not conform"
        );
        assert!(
            disagreements
                .iter()
                .any(|d| d.contains("Put(A, V0), Put(A, V1)")
                    && d.contains("store model observes")
                    && d.contains("broken vendor observes")),
            "the divergence names the trace and both observations: {disagreements:?}"
        );
    }

    /// Mismatched batteries are refused, never row-matched by luck.
    #[test]
    fn reports_over_different_batteries_are_refused() {
        let small = WorldReport::record("small", 4, StoreModel::replay);
        let disagreements = StoreModel::beliefs().disagreements(&small);
        assert_eq!(disagreements.len(), 1);
        assert!(disagreements[0].contains("batteries differ in size"));
    }

    /// The lock lands in its own namespace, and the render is pinned at the load-bearing
    /// points (header, a row's stanza shape) so the committed artifact's format is a
    /// reviewed decision.
    #[test]
    fn the_world_lock_renders_and_lands_in_its_namespace() {
        let report = StoreModel::beliefs();
        let lock = report.lock_in(Path::new("spec"));
        assert_eq!(lock.name, "store model world");
        assert_eq!(lock.path, Path::new("spec").join("store-model.world.spec"));
        let text = report.render();
        assert!(text.starts_with(
            "# world lock: store model — recorded conduct over the derived trace battery;"
        ));
        assert!(text.contains("\n- Trace([])\n      -> {}\n"));
        assert!(text.contains("\n- Trace([Put(A, V0)])\n      -> {A: V0}\n"));
    }
}
