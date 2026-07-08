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

- mutation (since green) (default branch, diff since mutants-green; pure)
      .github/mutants-gate.sh --in-diff since-green.diff
      promises: no mutant of anything changed since the last fully-certified tree (the mutants-green tag) survives — a merge re-verifies its drift, not the whole tree, and advances the tag on green

- mutation (full sweep) (weekly + manual, sharded; pure)
      .github/mutants-gate.sh
      promises: no mutant of the whole crate survives — the method's own "the real test", re-certifying the tree from scratch on a weekly clock (backstops incrementality's one gap: a test edit weakening kills for unchanged code)

- mutation (delta-render plumbing) (weekly + manual, sharded; pure)
      .github/mutants-gate.sh --package delta-render --config .github/delta-render-mutants.toml
      promises: no mutant of delta-render's plumbing (the license parser, the circuit validity rule, the render, the stream calculus) survives its lib probes — the workspace sweeps scope to the root crate, so the member that turns specs into generated code carries its own weekly verdict (config: .github/delta-render-mutants.toml; the drift-gate twins live in the lib precisely so this sweep can see them)

- statement bites (lean corpus) (weekly + manual, sharded; pure)
      .github/statement-bite.sh
      promises: no definition mutant of lean/ProbeBool.lean re-checks past its theorems, except the survivors ratified by key in lean/bites.register — mutation testing FOR the proof corpus: the kernel judges the mutants (the gate installs elan; the corpus is core-only), while the expected survivor set is pinned toolchain-free by discover::bite's mirror probe in every cargo test

- mutation (fire-drill plumbing) (weekly + manual, sharded; pure)
      .github/mutants-gate.sh --package fire-drill --config .github/fire-drill-mutants.toml
      promises: no mutant of fire-drill (the battery verdicts, the census refusals, both lockable renders) survives its lib probes — the workspace sweeps scope to the root crate, so the crate that proves gates can fire carries its own weekly verdict (config: .github/fire-drill-mutants.toml)

- mutation (layout-probe plumbing) (weekly + manual, sharded; pure)
      .github/mutants-gate.sh --package layout-probe --config .github/layout-probe-mutants.toml
      promises: no mutant of layout-probe (the two engines, the diagram edits, the visual census and its locality witness) survives its lib probes — the workspace sweeps scope to the root crate, so the second-domain miniature carries its own weekly verdict (config: .github/layout-probe-mutants.toml; the drift-gate twins live in the lib so this sweep can see them)

- release (certified tree) (on certification, when the mutants-green tag advances; EFFECTFUL)
      .github/release.sh
      promises: every certified default-branch tree publishes itself: the countersign's tag advance IS the release event, the version is CalVer (a date claims nothing about compatibility, which is honest), and the notes are DERIVED — commit subjects plus the ratified spec-lock diff, the uncompressed truth a semver integer would compress into an unchecked claim

- perimeter (settings drift) (weekly + manual, sharded; EFFECTFUL)
      .github/perimeter.sh
      promises: the LIVE repository perimeter — branch rules on the default branch, merge methods, private vulnerability reporting — still satisfies the declared floor (spec/perimeter.spec). Settings are configuration that drifts silently and that no one re-audits; this gate reads them back on the weekly clock and refuses by name. READ-ONLY: the write stays human — a privilege is ratified, never self-served

- substrate (git drift) (weekly + manual, sharded; EFFECTFUL)
      .github/substrate.sh
      promises: the LIVE repository's tags and history still satisfy the declared git substrate (spec/substrate.spec): the tags the machinery leans on exist and sit on the certified line, and the default branch stays linear after the declared epoch — the perimeter's squash-only rule, judged backward over the history that exists. READ-ONLY git plumbing against the checkout's own origin: the first world gate with no third-party API and no extra credential
