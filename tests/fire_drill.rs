//! The repo's own FIRE DRILL: known-bad fixtures proving the discipline's gates still fire.
//!
//! Every gate below guards this repository somewhere. A green suite proves each gate passes
//! good inputs; nothing in a green suite proves a gate would still go red — a rubber stamp
//! passes every positive test. So each drill plants a fixture that is KNOWN BAD and demands
//! the rejection arrive on cue. This is `fire-drill`'s dogfood (the crate exists because the
//! first production adoption hit three vacuous-pass failures in one system) and the process-
//! level twin of the witness shapes: "the gate actually acts" is only sayable as a red that
//! arrives when summoned.

use boundary_spec::discover::engine::{Engine, ShapeCatalog};
use boundary_spec::discover::expect::{Distance, Expectation};
use boundary_spec::discover::mutation::MutationReport;
use fire_drill::{Battery, Outcome};

// -- known-bad fixture: a LAWLESS theory (two distinguishable constants, no laws) -----------
// The algebra-mutation gate must report survivors for it: a spec that says nothing kills
// nothing, and the harness must say so.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct S;
#[derive(Clone)]
struct V(u8);
struct Lawless;
fn one(_: &[V]) -> Option<V> {
    Some(V(1))
}
fn two(_: &[V]) -> Option<V> {
    Some(V(2))
}

boundary_spec::theory! {
    Lawless : "lawless",
    Value = V,
    Obs = u8,
    Sort = S,
    sort_of = |_: &V| S,
    observe = |v: &V| v.0,
    vars { S => &["x", "y", "z"], }
    inhabit { S => vec![V(1), V(2)], }
    ops {
        Nullary "one" "one" () -> S = one;
        Nullary "two" "two" () -> S = two;
    }
}

// -- known-bad fixture: a theory that DECLARES a law its operators do not deliver ------------
// The distance gate must stay red for it.

struct Overclaim;
fn first(v: &[V]) -> Option<V> {
    Some(v[0].clone())
}

boundary_spec::theory! {
    Overclaim : "overclaim",
    Value = V,
    Obs = u8,
    Sort = S,
    sort_of = |_: &V| S,
    observe = |v: &V| v.0,
    vars { S => &["x", "y", "z"], }
    inhabit { S => vec![V(1), V(2), V(3)], }
    ops {
        Infix "first" "first" (S, S) -> S = first;
    }
    expects {
        commutative(first);
    }
}

fn the_battery(tamper_dir: &std::path::Path) -> Battery {
    // spec-lock's drift gate: a lock whose committed file was TAMPERED with, and one whose
    // file is MISSING, must both come back stale by name.
    let tampered = spec_lock::Lock {
        name: "tampered".into(),
        path: tamper_dir.join("tampered.spec"),
        live: "the derived truth".into(),
    };
    std::fs::write(&tampered.path, "a hand-edited lie").expect("plant the tampered lock");
    let missing = spec_lock::Lock {
        name: "missing".into(),
        path: tamper_dir.join("never-blessed.spec"),
        live: "anything".into(),
    };

    // the shape data gate: an identity declared with a constant of the WRONG sort must be
    // refused (sigs as sort NAMES, exactly how genesis validates declarations).
    let identity_gate = ShapeCatalog::inventory()
        .into_iter()
        .find(|s| s.name == "identity")
        .expect("identity is ratified")
        .gate_slots;
    let missorted: [(Vec<&str>, &str); 2] = [
        (vec!["credits", "credits"], "credits"),
        (vec![], "vouchers"),
    ];

    // the cross-lock anchor gate: a chain link whose live derivation drifted, one whose
    // anchor was re-ratified upstream, and one whose anchor is gone — each must be
    // refused WITH ITS OWN DIAGNOSIS (the three breaks are three different repairs).
    let baseline = "the shared instrument\n";
    let anchor_path = tamper_dir.join("anchor.spec");
    std::fs::write(&anchor_path, baseline).expect("plant the anchor");
    let cross = |name: &str, pinned_sha: String, live: &str| spec_lock::CrossLock {
        name: name.into(),
        foreign_path: anchor_path.clone(),
        pinned_sha,
        live: live.into(),
    };
    let drifted_live = cross(
        "drifted",
        spec_lock::sha256_hex(baseline.as_bytes()),
        "not the shared instrument\n",
    );
    let reratified = cross(
        "re-ratified",
        spec_lock::sha256_hex(b"an older ratification\n"),
        baseline,
    );
    let orphaned = spec_lock::CrossLock {
        name: "orphaned".into(),
        foreign_path: tamper_dir.join("never-existed.spec"),
        pinned_sha: spec_lock::sha256_hex(baseline.as_bytes()),
        live: baseline.into(),
    };
    let diagnosis = |lock: &spec_lock::CrossLock, fragment: &str| -> Outcome {
        match spec_lock::check_cross(std::slice::from_ref(lock)) {
            Err(broken) if broken[0].contains(fragment) => Outcome::Fired,
            _ => Outcome::Passed,
        }
    };

    Battery::named("the discipline's own gates")
        .requires([
            "spec-lock drift gate",
            "cross-lock anchor gate",
            "algebra mutation",
            "declared-expectation distance",
            "shape data gate",
            "expectation vocabulary",
        ])
        .drill(
            "spec-lock drift gate",
            "a committed lock file hand-edited away from the derived text",
            outcome(spec_lock::check(std::slice::from_ref(&tampered)).is_err()),
        )
        .drill(
            "spec-lock drift gate",
            "a lock whose committed file was never written (missing is stale, never fresh)",
            outcome(spec_lock::check(std::slice::from_ref(&missing)).is_err()),
        )
        .drill(
            "cross-lock anchor gate",
            "a chain link whose live derivation no longer reproduces its anchor — the \
             diagnosis must place the movement HERE, not upstream",
            diagnosis(&drifted_live, "live derivation drifted"),
        )
        .drill(
            "cross-lock anchor gate",
            "an anchor re-blessed upstream (pin no longer matches the file) — the \
             diagnosis must call for re-review, never a quiet re-pin",
            diagnosis(&reratified, "anchor re-ratified upstream"),
        )
        .drill(
            "cross-lock anchor gate",
            "an anchor file that is gone entirely",
            diagnosis(&orphaned, "anchor missing"),
        )
        .drill(
            "algebra mutation",
            "a lawless theory (two distinguishable constants, no laws) — its operator-table \
             mutants must SURVIVE and be reported",
            outcome(!MutationReport::of::<Lawless>().survivors().is_empty()),
        )
        .drill(
            "declared-expectation distance",
            "a theory declaring commutativity its left-projection operator does not have",
            outcome(!Distance::of::<Overclaim>().is_met()),
        )
        .drill(
            "shape data gate",
            "an identity declared with a constant of the wrong sort",
            outcome(
                identity_gate
                    .admit(
                        &missorted
                            .iter()
                            .map(|(i, o)| (i.clone(), *o))
                            .collect::<Vec<_>>(),
                        &["merge", "empty"],
                    )
                    .is_err(),
            ),
        )
        .drill(
            "expectation vocabulary",
            "a shape name the catalog never ratified (`symmetric` — `transitive` held \
             this drill until the guarded-law stanzas ratified it)",
            outcome(Expectation::canonical("symmetric").is_none()),
        )
}

fn outcome(fired: bool) -> Outcome {
    if fired {
        Outcome::Fired
    } else {
        Outcome::Passed
    }
}

/// Every gate fires on its planted known-bad fixture, and every required gate carries at
/// least one — the verdict names any rubber stamp.
#[test]
fn every_gate_still_fires() {
    let dir = std::env::temp_dir().join("probe-algebra-fire-drill");
    std::fs::create_dir_all(&dir).expect("a scratch dir for the planted locks");
    let battery = the_battery(&dir);
    if let Err(rot) = battery.verdict() {
        panic!(
            "a gate went vacuous:\n{rot}\n\nfull register:\n{}",
            battery.render()
        );
    }
    // and the lawless fixture's discovery really is silent — the drill planted what it
    // claimed to plant (a bad fixture that stops being bad invalidates its drill).
    assert!(Engine::<Lawless>::new().discover().laws.is_empty());
}
