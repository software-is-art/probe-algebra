//! gates — the pipeline drift gate: CI validates its own declaration on every run.

use std::path::PathBuf;

use credit_app::gates::Ci;

/// BOTH pipeline locks are fresh: the committed inventory and the committed workflow match
/// the declaration's renders — a hand edit to the YAML fails inside the very `cargo test`
/// the workflow runs.
#[test]
fn the_pipeline_locks_are_fresh() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let locks = Ci::pipeline()
        .locks_in(&root)
        .expect("the declared pipeline renders");
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "the pipeline drifted from its declaration: {}. Regenerate with \
             `cargo run --example freeze_gates` and ratify the diff — never edit the YAML \
             or the spec by hand.",
            stale.join(", ")
        );
    }
}

/// Nothing declared can fall out of execution: every gate's command reappears verbatim in
/// the rendered workflow.
#[test]
fn every_declared_gate_is_rendered_into_the_workflow() {
    let p = Ci::pipeline();
    let workflow = p.render_workflow().expect("the declared pipeline renders");
    for gate in &p.gates {
        assert!(
            workflow.contains(&gate.command_line()),
            "gate `{}` is declared but not executed by the workflow",
            gate.name
        );
    }
}
