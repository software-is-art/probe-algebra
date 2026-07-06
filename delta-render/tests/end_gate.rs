//! end_gate — the INDEPENDENT gate: `I ∘ Q^Δ ∘ D = Q` over the stream grid.
//!
//! The licenses are bounded evidence; the product gets a gate that does not trust
//! them. For the declared circuit: feed every grid stream's DELTAS through the
//! incremental form, integrate, and demand equality with the plain batch recompute at
//! every tick. A forged or gapped license must surface HERE as a named failing stream
//! — this law holds regardless of what `spec/licenses.spec` says.
//!
//! (The world-level slot, declared now per the design: when a SQL frontend is
//! attached, the batch oracle becomes the emulator and this same equation becomes the
//! emulator-differential tier. Nothing else here depends on it.)

use delta_render::audit_incremental::audit_incremental;
use delta_render::circuit::{audit_circuit, circuit_locks, demo_circuit};
use delta_render::demo_incremental::demo_incremental;
use delta_render::license::Registry;
use delta_render::stream::grid;

/// THE END LAW, on the GENERATED code: `I(demo_incremental(D(s))) = batch(s)` for
/// every stream on the grid — incremental-then-integrate equals batch recompute.
#[test]
fn the_generated_circuit_meets_the_end_law_on_every_grid_stream() {
    let registry = Registry::derive();
    let circuit = demo_circuit(&registry);
    for (i, s) in grid().into_iter().enumerate() {
        let batch = circuit.batch(std::slice::from_ref(&s));
        let incremental = demo_incremental(&s.differentiate()).integrate();
        assert_eq!(
            incremental, batch,
            "I(Q^Δ(D(s))) = Q(s) failed on grid stream {i}: {s:?}"
        );
    }
}

/// THE END LAW on the AUDIT circuit's generated code — two sources, a fan-out, and
/// the non-commutative bilinear, over every grid PAIR: the DAG shapes the demo cannot
/// show, held to the same law.
#[test]
fn the_generated_audit_circuit_meets_the_end_law_on_every_grid_pair() {
    let registry = Registry::derive();
    let circuit = audit_circuit(&registry);
    for (i, s) in grid().into_iter().enumerate() {
        for (j, t) in grid().into_iter().enumerate() {
            let batch = circuit.batch(&[s.clone(), t.clone()]);
            let incremental = audit_incremental(&s.differentiate(), &t.differentiate()).integrate();
            assert_eq!(
                incremental, batch,
                "I(Q^Δ(D(s))) = Q(s) failed on grid pair ({i}, {j})"
            );
        }
    }
}

/// The generated code and its interpreter twin are the SAME derivation: byte-level
/// drift is caught by the render gate below; semantic drift is caught here.
#[test]
fn the_generated_code_agrees_with_the_interpreter_twin() {
    let registry = Registry::derive();
    let circuit = demo_circuit(&registry);
    for s in grid() {
        let deltas = [s.differentiate()];
        assert_eq!(
            demo_incremental(&deltas[0]),
            circuit.incremental_with(&registry, &deltas),
            "the rendered code and the interpreter diverged"
        );
    }
}

/// THE RENDER DRIFT GATES: both committed render artifacts — the generated Rust and
/// the plain-language derivation — are re-rendered live and held byte for byte. A
/// hand edit to `gen/demo_incremental.rs` fails here; a license demotion regenerates
/// both with a diff that NAMES the rule change.
#[test]
fn the_committed_renders_are_fresh() {
    let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let registry = Registry::derive();
    let [a, b] = circuit_locks(&demo_circuit(&registry), &registry, &crate_root);
    let [c, d] = circuit_locks(&audit_circuit(&registry), &registry, &crate_root);
    let locks = [a, b, c, d];
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "a rendered circuit artifact drifted: {}. Never hand-edit — run \
             `cargo run -p delta-render --example freeze` and ratify the diff.",
            stale.join(", ")
        );
    }
}
