# gate registry: the pipeline as a declaration — regenerate via `cargo run --example freeze_gates`; ratify the diff.
#
# Every gate below is a deterministic function of the tree (Pure): executable on
# any machine (`cargo run --example gate`), re-executed by CI from the DERIVED
# workflow (.github/workflows/ci.yml — itself a lock rendered from this registry),
# and therefore countersignable. CI keeps only what cannot shift left:
# countersigning, effects, and the economics of the expensive sweeps — all three
# visible here as cadence and capability instead of implicit in YAML.

- format (every change; pure)
      cargo fmt --all --check
      promises: the whole workspace is rustfmt-canonical (generated members included)

- lint (every change; pure)
      cargo clippy --workspace --all-targets --all-features -- -D warnings
      promises: clippy holds every workspace member, all targets and features, to deny-warnings

- test (every change; pure)
      cargo test --workspace --all-targets
      promises: every workspace member's suites: the enforcement passes and qualify censuses (ride the builds), the drift gates (module, system, shapes, and world locks), the distance gates, the probes, and the consumer fixtures

- mutation (changed lines) (per PR diff; pure)
      .github/mutants-gate.sh --in-diff pr.diff
      promises: no mutant of the PR's changed lines survives the probe suite (timeouts are detections; ratified equivalents live in .cargo/mutants.toml)

- mutation (full sweep) (default branch + weekly; pure)
      .github/mutants-gate.sh
      promises: no mutant of the whole crate survives — the method's own "the real test", amortised to the default branch and a weekly clock
