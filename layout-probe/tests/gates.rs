//! gates — layout-probe's fire drills: known-bad engines proving the gates can FIRE.
//!
//! 1. a JITTERING engine (per-observation drift — the nondeterministic layout every
//!    agent loop fears) must NOT earn the `inert` law: discovery refusing the law IS
//!    the stability gate, so a planted unstable engine proves the refusal arm works;
//! 2. the VISUAL CENSUS must redden when the corpus grows past it (a wider rank) and
//!    when the committed file is missing — a census that cannot go red is decoration.

use boundary_spec::discover::engine::{Engine, Fixity, Operator, Theory};
use fire_drill::{Battery, Outcome};
use layout_probe::census;
use layout_probe::diagram::Diagram;
use layout_probe::layout::{layout, Policy};
use layout_probe::theories::source_grid;

// ===== the planted unstable engine =========================================

/// The jitter: every OBSERVATION shifts x by a fresh count — a layout that answers a
/// little differently each time anyone looks. Deterministic per process, unstable per
/// call: exactly the pathology the `inert` law exists to refuse.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Jd;
struct JitterLayout;

fn j_reorder(v: &[Diagram]) -> Option<Diagram> {
    Some(v[0].reorder())
}

impl Theory for JitterLayout {
    type Sort = Jd;
    type Value = Diagram;
    type Obs = Vec<(String, (i64, i64))>;
    fn name() -> &'static str {
        "jitter layout"
    }
    fn operators() -> Vec<Operator<Self>> {
        vec![Operator {
            name: "reorder",
            symbol: "reorder",
            fixity: Fixity::Prefix,
            inputs: vec![Jd],
            output: Jd,
            eval: j_reorder,
        }]
    }
    fn inhabitants(_: Jd) -> Vec<Diagram> {
        source_grid()
    }
    fn sort_of(_: &Diagram) -> Jd {
        Jd
    }
    fn observe(d: &Diagram) -> Vec<(String, (i64, i64))> {
        use std::cell::Cell;
        thread_local! {
            static CALLS: Cell<i64> = const { Cell::new(0) };
        }
        let jitter = CALLS.with(|c| {
            c.set(c.get() + 1);
            c.get()
        });
        layout(d, Policy::Stable)
            .placements()
            .iter()
            .map(|(n, (x, y))| (n.clone(), (*x + jitter, *y)))
            .collect()
    }
    fn sort_vars(_: Jd) -> &'static [&'static str] {
        &["x", "y", "z"]
    }
}

#[test]
fn every_gate_still_fires() {
    let dir = std::env::temp_dir().join("layout-probe-fire-drill");
    std::fs::create_dir_all(&dir).expect("a scratch dir for planted locks");

    // census drill fixtures: the committed census held against a GROWN corpus's live
    // text (the wider rank must redden it), and a never-written file (missing is
    // stale, never fresh).
    let mut grown = source_grid();
    grown.push(Diagram::of(&["p", "q", "r"], &[]));
    let committed_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec/visual.census");
    let grown_lock = spec_lock::Lock {
        name: "census vs grown corpus".into(),
        path: committed_path,
        live: census::render_over(&grown),
    };
    let missing_lock = spec_lock::Lock {
        name: "never frozen".into(),
        path: dir.join("never-frozen.census"),
        live: census::render(),
    };

    let jitter_spec = Engine::<JitterLayout>::new().discover();
    let jitter_awarded_stability = jitter_spec
        .laws
        .iter()
        .any(|l| l.prose.contains("leaves every value unchanged"));

    let battery = Battery::named("layout-probe's gates")
        .requires(["inert gate", "census gate"])
        .drill(
            "inert gate",
            "a jittering engine (every observation shifts) — discovery must REFUSE \
             the inert law a stable engine earns",
            if jitter_awarded_stability {
                Outcome::Passed
            } else {
                Outcome::Fired
            },
        )
        .drill(
            "census gate",
            "the corpus grown past the committed census (a three-node rank) — the \
             widest-rank row must redden the lock",
            if spec_lock::check(std::slice::from_ref(&grown_lock)).is_err() {
                Outcome::Fired
            } else {
                Outcome::Passed
            },
        )
        .drill(
            "census gate",
            "a census whose committed file was never written (missing is stale, \
             never fresh)",
            if spec_lock::check(std::slice::from_ref(&missing_lock)).is_err() {
                Outcome::Fired
            } else {
                Outcome::Passed
            },
        );

    if let Err(rot) = battery.verdict() {
        panic!(
            "a gate went vacuous:\n{rot}\n\nregister:\n{}",
            battery.render()
        );
    }
    // and the refusal is the INTERESTING half of a pair: the same discovery over the
    // stable engine DOES award the law (pinned in the committed spec) — so the drill
    // proves refusal is earned, not universal.
    assert!(
        !jitter_awarded_stability,
        "a jittering engine must never read as stable"
    );
}
