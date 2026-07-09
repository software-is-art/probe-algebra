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

- certify (mutants-green) (default branch, after the every-change gates; pure)
      git tag -f mutants-green
      promises: the default branch marks itself certified: once the every-change gates — which carry the WHOLE compiled-mutant population — pass on a push, the mutants-green tag advances to HEAD and the release mints. The per-merge cargo-mutants run is retired; the weekly sweeps keep the exempted remainder

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

- mutation (schemata) (every change; pure)
      cargo run --features schemata --example schemata -- sweep
      promises: no compiled expression-flip mutant survives the lib suite: every `#[mutate]` site (spec/schemata.spec — the judges, the router's classifier, the reliance judge) ships in ONE build behind the PROBE_MUTANT selector, and the sweep runs the suite once per site — a green run with a flip active is a survivor, ratified by key in spec/schemata.register or killed with a probe. The rebuild-per-mutant price is gone, so the whole population rides EVERY change (~a minute on a warm cache), not a weekly clock

- substrate (git drift) (weekly + manual, sharded; EFFECTFUL)
      .github/substrate.sh
      promises: the LIVE repository's tags and history still satisfy the declared git substrate (spec/substrate.spec): the tags the machinery leans on exist and sit on the certified line, every published root-crate version carries its v<version> marker (instances DERIVED from the crates.io index, never named in the declaration), and the default branch stays linear after the declared epoch — the perimeter's squash-only rule, judged backward. READ-ONLY and credential-free: git plumbing against the checkout's own origin plus one anonymous sparse-index read
