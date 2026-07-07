# gate registry: the pipeline as a declaration — regenerate via `cargo run --example freeze_gates`; ratify the diff.
#
# Every gate below is a declared, executable claim: the workflow CI runs is
# RENDERED from this same declaration and drift-gated byte for byte, so "green
# locally" and "green in CI" are one claim. Cadence and capability are visible
# here instead of implicit in YAML.

- format (every change; pure)
      cargo fmt --all --check
      promises: the whole workspace is rustfmt-canonical

- lint (every change; pure)
      cargo clippy --workspace --all-targets --all-features -- -D warnings
      promises: clippy holds every workspace member, all targets and features, to deny-warnings

- test (every change; pure)
      cargo test --workspace --all-targets
      promises: every suite: the drift gates (module, system, and gates locks), the distance gates, and the probes
