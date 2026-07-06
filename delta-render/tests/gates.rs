//! fire_drill — known-bad fixtures proving the CLASSIFIER and the END GATE still fire.
//!
//! The licenses and the render are gates; every gate can rot into a rubber stamp. Each
//! drill below plants a fixture that is KNOWN BAD and demands the rejection arrive:
//!
//! 1. an ALMOST-linear operator (additive on non-negative weights only) — the
//!    classifier must DENY the license, because the grid's negative-weight instances
//!    are load-bearing; and if a pruned grid is substituted to FORGE the license, the
//!    end law must fail instead (both arms tested);
//! 2. a HAND-EDITED generated circuit — the render drift gate must redden;
//! 3. a FORGED license (`distinct → linear`, bypassing derivation) — the end gate must
//!    fire on a named stream;
//! 4. (the floor, as a plain test) an all-generic-fallback derivation must PASS the
//!    end gate — correctness never depended on any license;
//! 5. (the committed witness, as a drift gate) the min-retraction red instance —
//!    `spec/min.retraction.spec` — is fresh, and the witness itself still refutes.

use std::path::PathBuf;

use boundary_spec::discover::engine::{Fixity, Operator, Theory};
use boundary_spec::discover::Spec;
use delta_render::circuit::{circuit_locks, demo_circuit, fallback1, linear1};
use delta_render::license::{Classification, License, Registry};
use delta_render::ops::min_retraction_witness;
use delta_render::stream::{grid as stream_grid, Stream};
use delta_render::zset::{Row, ZSet, Zs};
use fire_drill::{Battery, Outcome};

// ===== the almost-linear operator: additive ONLY on non-negative weights ===

/// Clamp: drop every retraction (negative weight). Additive on insert-only histories,
/// broken the moment a delta retracts — the classic almost-license.
fn clamped(x: &ZSet) -> ZSet {
    ZSet::of(
        &x.entries()
            .into_iter()
            .filter(|(_, w)| *w > 0)
            .collect::<Vec<_>>(),
    )
}

fn op_zero(_: &[ZSet]) -> Option<ZSet> {
    Some(ZSet::empty())
}
fn op_plus(v: &[ZSet]) -> Option<ZSet> {
    Some(v[0].plus(&v[1]))
}
fn op_neg(v: &[ZSet]) -> Option<ZSet> {
    Some(v[0].neg())
}
fn op_clamped(v: &[ZSet]) -> Option<ZSet> {
    Some(clamped(&v[0]))
}

/// The drill theory over a grid: group + `clamp`, judged exhaustively. `HONEST` keeps
/// the carrier's negative-weight instances; the forged twin prunes them.
macro_rules! clamp_theory {
    ($thy:ident, $name:literal, $grid:expr) => {
        struct $thy;
        impl Theory for $thy {
            type Sort = Zs;
            type Value = ZSet;
            type Obs = Vec<(Row, i64)>;
            fn name() -> &'static str {
                $name
            }
            fn operators() -> Vec<Operator<Self>> {
                vec![
                    Operator {
                        name: "zero",
                        symbol: "zero",
                        fixity: Fixity::Nullary,
                        inputs: vec![],
                        output: Zs,
                        eval: op_zero,
                    },
                    Operator {
                        name: "plus",
                        symbol: "plus",
                        fixity: Fixity::Infix,
                        inputs: vec![Zs, Zs],
                        output: Zs,
                        eval: op_plus,
                    },
                    Operator {
                        name: "neg",
                        symbol: "neg",
                        fixity: Fixity::Prefix,
                        inputs: vec![Zs],
                        output: Zs,
                        eval: op_neg,
                    },
                    Operator {
                        name: "clamp",
                        symbol: "clamp",
                        fixity: Fixity::Prefix,
                        inputs: vec![Zs],
                        output: Zs,
                        eval: op_clamped,
                    },
                ]
            }
            fn inhabitants(_: Zs) -> Vec<ZSet> {
                $grid
            }
            fn sort_of(_: &ZSet) -> Zs {
                Zs
            }
            fn observe(v: &ZSet) -> Vec<(Row, i64)> {
                v.entries()
            }
            fn sort_vars(_: Zs) -> &'static [&'static str] {
                &["x", "y", "z"]
            }
            fn grid_size() -> usize {
                512 // covers both grids' full spaces (8³ and 5³)
            }
        }
    };
}

fn honest_grid() -> Vec<ZSet> {
    let a = Row::new(0);
    let b = Row::new(1);
    vec![
        ZSet::empty(),
        ZSet::of(&[(a, 1)]),
        ZSet::of(&[(a, -1)]),
        ZSet::of(&[(a, 3)]),
        ZSet::of(&[(a, 2)]),
        ZSet::of(&[(a, -2)]),
        ZSet::of(&[(a, 1), (b, 1)]),
        ZSet::of(&[(a, 1), (b, -2)]),
    ]
}

/// The FORGE: the same operator over a grid with every retraction pruned — on
/// insert-only histories `clamp` really is additive, so discovery certifies the
/// license the full space refutes. This is what "the grid's negative-weight instances
/// are load-bearing" means, made executable.
fn pruned_grid() -> Vec<ZSet> {
    honest_grid()
        .into_iter()
        .filter(|z| z.entries().iter().all(|(_, w)| *w > 0))
        .collect()
}

clamp_theory!(ClampHonest, "clamp-honest", honest_grid());
clamp_theory!(ClampForged, "clamp-forged", pruned_grid());

fn license_of<T: Theory>(op: &str) -> License {
    let lock = Spec::of::<T>().lock_in(std::path::Path::new("spec"));
    License::read(op, "spec/clamp.spec", &lock.live)
}

fn outcome(fired: bool) -> Outcome {
    if fired {
        Outcome::Fired
    } else {
        Outcome::Passed
    }
}

/// Does the end law fail somewhere on the stream grid for a unary operator run under
/// the LINEAR rule? (`I(linear1(D(s), f)) = lift-tickwise f(s)` — false is a firing.)
fn end_law_fires_for_linear_rule(f: fn(&ZSet) -> ZSet) -> bool {
    stream_grid().into_iter().any(|s| {
        let batch = Stream::of(&s.ticks().iter().map(f).collect::<Vec<_>>());
        let incremental = linear1(&s.differentiate(), f).integrate();
        incremental != batch
    })
}

#[test]
fn every_gate_still_fires() {
    let dir = std::env::temp_dir().join("delta-render-fire-drill");
    std::fs::create_dir_all(&dir).expect("a scratch dir for the planted locks");

    // drill 2's fixture: the committed render, hand-edited in a planted copy.
    let registry = Registry::derive();
    let [gen_lock, _] = circuit_locks(&demo_circuit(&registry), &registry, &PathBuf::from("."));
    let tampered = spec_lock::Lock {
        name: "tampered render".into(),
        path: dir.join("demo_incremental.rs"),
        live: gen_lock.live.clone(),
    };
    std::fs::write(&tampered.path, format!("{}// hand edit\n", gen_lock.live))
        .expect("plant the tampered render");

    // drill 3's fixture: the forged registry — `distinct` hard-coded linear, bypassing
    // derivation entirely.
    let mut forged = registry.clone();
    for l in &mut forged.licenses {
        if l.operator == "distinct" {
            l.classification = Classification::Linear;
        }
    }
    let circuit = demo_circuit(&registry);
    let forged_end_gate_fires = stream_grid().into_iter().any(|s| {
        let batch = circuit.batch(std::slice::from_ref(&s));
        let incremental = circuit
            .incremental_with(&forged, &[s.differentiate()])
            .integrate();
        incremental != batch
    });

    let battery = Battery::named("delta-render's gates")
        .requires(["license classifier", "render drift gate", "end gate"])
        .drill(
            "license classifier",
            "an almost-linear operator (drops retractions) over the honest grid — the \
             negative-weight instances must deny the license",
            outcome(license_of::<ClampHonest>("clamp").classification != Classification::Linear),
        )
        .drill(
            "end gate",
            "the same operator with its license FORGED by a pruned, insert-only grid — \
             discovery certifies it (the forge works), so the end law must fail instead",
            outcome(
                license_of::<ClampForged>("clamp").classification == Classification::Linear
                    && end_law_fires_for_linear_rule(clamped),
            ),
        )
        .drill(
            "render drift gate",
            "a hand-edited copy of the generated incremental circuit",
            outcome(spec_lock::check(std::slice::from_ref(&tampered)).is_err()),
        )
        .drill(
            "render drift gate",
            "a generated circuit whose file was never written (missing is stale, never fresh)",
            outcome(
                spec_lock::check(std::slice::from_ref(&spec_lock::Lock {
                    name: "never rendered".into(),
                    path: dir.join("never-rendered.rs"),
                    live: gen_lock.live.clone(),
                }))
                .is_err(),
            ),
        )
        .drill(
            "end gate",
            "a forged license (`distinct → linear`, hard-coded past the derivation) run \
             through the interpreter twin",
            outcome(forged_end_gate_fires),
        )
        .drill(
            "license classifier",
            "a spec text carrying only the zero fixed point (no additivity) — must not \
             read as any license",
            outcome(
                License::read("d", "spec/d.spec", "- d leaves zero fixed.\n").classification
                    == Classification::Neither,
            ),
        );

    if let Err(rot) = battery.verdict() {
        panic!(
            "a gate went vacuous:\n{rot}\n\nregister:\n{}",
            battery.render()
        );
    }
}

/// THE FLOOR: an all-generic-fallback derivation (every license stripped to NEITHER)
/// still meets the end law on every grid stream — correctness never depended on a
/// license; upgrades only ever buy cost.
#[test]
fn the_all_fallback_floor_meets_the_end_law() {
    let registry = Registry::derive();
    let mut floor = registry.clone();
    for l in &mut floor.licenses {
        l.classification = Classification::Neither;
        l.citations.clear();
    }
    let circuit = demo_circuit(&registry);
    for s in stream_grid() {
        let batch = circuit.batch(std::slice::from_ref(&s));
        let incremental = circuit
            .incremental_with(&floor, &[s.differentiate()])
            .integrate();
        assert_eq!(
            incremental, batch,
            "the generic fallback must always be correct"
        );
    }
}

/// The committed min-retraction witness is FRESH, and its instance still refutes:
/// the fallback route disagrees with the linear route for `min` on the stream grid
/// (the executable half of what the frozen artifact says in prose).
#[test]
fn the_min_retraction_witness_is_fresh_and_still_refutes() {
    let lock = spec_lock::Lock {
        name: "min retraction witness".into(),
        path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec/min.retraction.spec"),
        live: min_retraction_witness(),
    };
    if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
        panic!(
            "the min-retraction witness drifted: {}. Regenerate \
             (`cargo run -p delta-render --example freeze`) and ratify the diff.",
            stale.join(", ")
        );
    }
    // and executably: running min under the linear rule breaks the end law somewhere.
    assert!(
        end_law_fires_for_linear_rule(delta_render::ops::least),
        "min under the linear rule must fail the end law — that failure IS the witness"
    );
    // while the honest fallback holds it.
    for s in stream_grid() {
        let batch = Stream::of(
            &s.ticks()
                .iter()
                .map(delta_render::ops::least)
                .collect::<Vec<_>>(),
        );
        let incremental = fallback1(&s.differentiate(), delta_render::ops::least).integrate();
        assert_eq!(incremental, batch, "the fallback must hold min's end law");
    }
}
