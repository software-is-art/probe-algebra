# fire-drill

**Prove your gates still fire.**

Every pipeline accumulates gates — validators, reconciliation checks, drift gates, review
stamps — and every gate can rot into a rubber stamp. A vacuous gate is worse than no gate: it
keeps emitting the confidence while no longer doing the work, and its positive tests can't
tell, because a rubber stamp passes every positive test.

This crate is the mutation-testing move applied to processes: keep a standing battery of
**known-bad fixtures**, one or more per gate, and on every run demand that each gate REJECTS
its planted bad input. A drill that passes is a gate that has gone vacuous, named. A required
gate with no drill is UNPROVEN, so a gate cannot silently join the pipeline without a fixture
showing it can fail.

```rust
use fire_drill::{Battery, Outcome};

let battery = Battery::named("nightly gates")
    .requires(["reconciliation", "coverage manifest"])
    .drill("reconciliation", "an empty tree (zero items checked)", Outcome::Fired)
    .drill("coverage manifest", "a manifest naming a missing file", Outcome::Fired);

battery.verdict().expect("every gate still fires");
```

Substrate-free: a "gate" is anything whose verdict you can observe — a Rust function, a CLI,
a prose checklist. Run your gate over your bad fixture however you run it; hand this crate
the outcome. `render()` is deterministic text — freeze it with
[`spec-lock`](https://github.com/software-is-art/probe-algebra/tree/main/spec-lock) and the
battery itself is drift-gated, so removing a drill is a reviewed diff, never a quiet deletion.

Honest frame: a drill refutes vacuousness for its fixture only — the battery proves the alarm
rings when the button is pressed, never that the alarm hears everything. Grow it the way
incident registers grow: every vacuous-pass incident becomes a drill.

Part of [probe-algebra](https://github.com/software-is-art/probe-algebra) — the spec-lock
discipline: derive the spec by running the thing, freeze it, gate the drift.

## License

Dual-licensed under either of Apache-2.0 or MIT, at your option.
