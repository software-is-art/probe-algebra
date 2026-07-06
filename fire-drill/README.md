# fire-drill

**Prove your gates still fire.**

Every pipeline accumulates gates — validators, reconciliation checks, drift gates, review
stamps — and every gate can rot into a rubber stamp. A vacuous gate is worse than no gate: it
keeps emitting the confidence while no longer doing the work, and its positive tests can't
tell, because a rubber stamp passes every positive test.

**Start with the census.** `requires([...])` declares the gates that must each carry a
known-bad fixture; a required gate with no drill is UNPROVEN and fails the verdict. That
default — a gate cannot join the pipeline without a fixture proving it can fail — is the
sleeper feature: in systems where gates accrete fast, it is worth more than the drills
themselves. The drills catch rot; the census prevents gates being born rotten.

The drills are the mutation-testing move applied to processes: a standing battery of
**known-bad fixtures**, one or more per gate, and on every run each gate must REJECT its
planted bad input. A drill that passes is a gate that has gone vacuous, named — validated in
production on day zero of this crate's first consumption, by an incident that predated
reading it: two committed regression fixtures both carried the degenerate case, so a 95-test
green suite could not see a real defect the first non-degenerate job hit. Not a gate that
stopped firing — a fixture set that never pressed the button.

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
the outcome. That handoff is where consumers quietly cheat, so map it STRICTLY: `Fired` only
when the gate failed AND its verdict names the planted defect (a usage error or an unrelated
failure is a harness bug — panic, don't count it), and panic when a mutation helper cannot
find the text it plans to corrupt (a mutation that misses its target makes the drill vacuous
the wrong way round). The crate docs carry the worked example. `render()` is deterministic text — freeze it with
[`spec-lock`](https://github.com/software-is-art/probe-algebra/tree/main/spec-lock) and the
battery itself is drift-gated, so removing a drill is a reviewed diff, never a quiet deletion.

Honest frame: a drill refutes vacuousness for its fixture only — the battery proves the alarm
rings when the button is pressed, never that the alarm hears everything. Grow it the way
incident registers grow: every vacuous-pass incident becomes a drill.

Part of [probe-algebra](https://github.com/software-is-art/probe-algebra) — the spec-lock
discipline: derive the spec by running the thing, freeze it, gate the drift.

## License

Dual-licensed under either of Apache-2.0 or MIT, at your option.
