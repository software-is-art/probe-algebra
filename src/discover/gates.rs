//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! gates — THE PIPELINE IS A LOCK: CI/CD subsumed into the declaration discipline.
//!
//! The last unvalidated artifact in this repository was its own pipeline. `ci.yml` was
//! hand-maintained prose sitting on the lowest rung of the assurance ladder (human
//! vigilance) — and it drifted exactly the way hand-maintained prose drifts: for its whole
//! life it ran `cargo test --all-targets` at a workspace root, silently testing only the
//! root package while the fixture it existed to guard went unexercised. The bug class is
//! structural: a pipeline that is not derived cannot be drift-gated.
//!
//! So the pipeline becomes what everything else here already is — a DECLARED, DERIVED,
//! RATIFIED artifact:
//!
//!   - [`GateRegistry::declared`] is the single source: every gate, with what it verifies,
//!     the exact command, its CADENCE (every change / per PR diff / default branch + weekly),
//!     and its capability (all current gates are `Pure` in the load-bearing sense: a
//!     deterministic function of the tree — executable on any machine, re-executable by CI,
//!     countersignable; an eventual deploy or live world-replay gate would be `Effectful`,
//!     and would wear that tag in the registry like any edge).
//!   - `spec/gates.spec` locks the human-readable inventory (WHAT is promised).
//!   - `.github/workflows/ci.yml` is ITSELF a lock (HOW GitHub Actions executes it): the
//!     execution steps are rendered from the registry, the file is drift-gated byte for
//!     byte, and the fix for a stale pipeline is never to edit the YAML — regenerate with
//!     `cargo run --example freeze_gates` and ratify the diff.
//!   - `cargo run --example gate` executes the every-change gates locally from the SAME
//!     declaration — CI stops being where verification is defined and becomes one more
//!     machine that executes it.
//!
//! What CI irreducibly keeps: countersigning (a green run the author didn't produce),
//! effects (edges the tree cannot contain), and economics (the full mutation sweep is too
//! expensive per keystroke, so it runs on the default branch and a weekly clock). All three
//! are now VISIBLE as registry data instead of implicit in YAML.

use std::path::PathBuf;

use spec_lock::Lock;

use crate::boundary::Capability;

/// The pinned Rust toolchain the workflow installs. Pinned, not `@stable`: the
/// `tests/compile_fail` trybuild suites match saved `.stderr` files, and rustc's diagnostic
/// rendering changes between versions. Bump this in lockstep with regenerating the
/// `.stderr` files (`TRYBUILD=overwrite`), then re-freeze the gates.
pub const TOOLCHAIN: &str = "1.94.1";

/// The weekly full-sweep schedule (Mondays 04:00 UTC) — the periodic whole-crate guarantee.
pub const FULL_SWEEP_CRON: &str = "0 4 * * 1";

/// When a gate runs — the ECONOMIC axis of the pipeline, explicit instead of implied by
/// which YAML job a step happens to sit in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cadence {
    /// Every push and every PR — cheap enough to pay per change.
    EveryChange,
    /// PRs only, scoped to the changed lines (mutation is expensive; the diff is the
    /// blast radius).
    PerDiff,
    /// The default branch, a weekly clock, and manual dispatch — the exhaustive sweep.
    DefaultBranchAndWeekly,
}

/// One declared gate of the pipeline.
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    /// The gate's name (also its step name where the workflow needs one).
    pub name: &'static str,
    /// What a pass PROMISES, in prose — the registry lock carries this, so the pipeline's
    /// claims are ratified text, not tribal knowledge.
    pub verifies: &'static str,
    /// The exact command, as argv — one source for the workflow render AND the local
    /// runner, so "green locally" and "green in CI" are the same claim.
    pub command: &'static [&'static str],
    pub cadence: Cadence,
    /// `Pure` = a deterministic function of the tree (cacheable, countersignable,
    /// executable anywhere). An eventual deploy / live-replay gate would be `Effectful`.
    pub effect: Capability,
}

impl Gate {
    /// The command as one shell-displayable line (argv joined — no shell interpretation
    /// anywhere in the pipeline's own machinery).
    pub fn command_line(&self) -> String {
        self.command.join(" ")
    }
}

/// The pipeline's registry — the single source both locks and the local runner derive
/// from. Associated fns per the no-rats-nest rule.
pub struct GateRegistry;

impl GateRegistry {
    /// Every declared gate, in execution order.
    pub fn declared() -> Vec<Gate> {
        vec![
            Gate {
                name: "format",
                verifies: "the whole workspace is rustfmt-canonical (generated members \
                           included)",
                command: &["cargo", "fmt", "--all", "--check"],
                cadence: Cadence::EveryChange,
                effect: Capability::Pure,
            },
            Gate {
                name: "lint",
                verifies: "clippy holds every workspace member, all targets and features, \
                           to deny-warnings",
                command: &[
                    "cargo",
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
                cadence: Cadence::EveryChange,
                effect: Capability::Pure,
            },
            Gate {
                name: "test",
                verifies: "every workspace member's suites: the enforcement passes and \
                           qualify censuses (ride the builds), the drift gates (module, \
                           system, shapes, and world locks), the distance gates, the \
                           probes, and the consumer fixtures",
                command: &["cargo", "test", "--workspace", "--all-targets"],
                cadence: Cadence::EveryChange,
                effect: Capability::Pure,
            },
            Gate {
                name: "mutation (changed lines)",
                verifies: "no mutant of the PR's changed lines survives the probe suite \
                           (timeouts are detections; ratified equivalents live in \
                           .cargo/mutants.toml)",
                command: &[".github/mutants-gate.sh", "--in-diff", "pr.diff"],
                cadence: Cadence::PerDiff,
                effect: Capability::Pure,
            },
            Gate {
                name: "mutation (full sweep)",
                verifies: "no mutant of the whole crate survives — the method's own \"the \
                           real test\", amortised to the default branch and a weekly clock",
                command: &[".github/mutants-gate.sh"],
                cadence: Cadence::DefaultBranchAndWeekly,
                effect: Capability::Pure,
            },
        ]
    }

    /// The human-readable inventory — what `spec/gates.spec` locks: one stanza per gate
    /// (name, cadence, capability, command, promise). The pipeline's CLAIMS, as ratified
    /// text.
    pub fn render_registry() -> String {
        let mut out = String::from(
            "# gate registry: the pipeline as a declaration — regenerate via `cargo run --example freeze_gates`; ratify the diff.\n\
             #\n\
             # Every gate below is a deterministic function of the tree (Pure): executable on\n\
             # any machine (`cargo run --example gate`), re-executed by CI from the DERIVED\n\
             # workflow (.github/workflows/ci.yml — itself a lock rendered from this registry),\n\
             # and therefore countersignable. CI keeps only what cannot shift left:\n\
             # countersigning, effects, and the economics of the expensive sweeps — all three\n\
             # visible here as cadence and capability instead of implicit in YAML.\n",
        );
        for gate in Self::declared() {
            let cadence = match gate.cadence {
                Cadence::EveryChange => "every change",
                Cadence::PerDiff => "per PR diff",
                Cadence::DefaultBranchAndWeekly => "default branch + weekly",
            };
            let effect = match gate.effect {
                Capability::Pure => "pure",
                Capability::Lossy => "lossy",
                Capability::Stateful => "stateful",
                Capability::Effectful => "EFFECTFUL",
            };
            out.push_str(&format!(
                "\n- {} ({cadence}; {effect})\n      {}\n      promises: {}\n",
                gate.name,
                gate.command_line(),
                gate.verifies
            ));
        }
        out
    }

    /// The GitHub Actions workflow — `.github/workflows/ci.yml` IS this render, drift-gated:
    /// the execution steps come from the registry (commands, cadences, the toolchain pin,
    /// the sweep schedule); the scaffolding (checkout, cache, tool install) is the template
    /// around them. Never edit the YAML by hand — regenerate and ratify.
    pub fn render_workflow() -> String {
        let gates = Self::declared();
        let every_change: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.cadence == Cadence::EveryChange)
            .collect();
        let per_diff: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.cadence == Cadence::PerDiff)
            .collect();
        let sweep: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.cadence == Cadence::DefaultBranchAndWeekly)
            .collect();

        let mut out = format!(
            "# GENERATED from the gate registry (`discover::gates`) — THE PIPELINE IS A LOCK.\n\
             # Never edit by hand: regenerate with `cargo run --example freeze_gates` and ratify\n\
             # the diff. The registry is the single source for the commands, cadences, the\n\
             # toolchain pin, and the sweep schedule; `spec/gates.spec` carries the promises.\n\
             name: ci\n\
             \n\
             # Dogfood everywhere, all the time. The fast gates run on every push and PR; the\n\
             # mutation gate — the method's own \"the real test\" — runs per-change on PRs (only\n\
             # the changed lines, since mutation is expensive) and as a full-crate sweep on the\n\
             # default branch and a weekly schedule.\n\
             #\n\
             # The Rust toolchain is PINNED (not `@stable`): the `tests/compile_fail` trybuild\n\
             # suites match saved `.stderr` files, and rustc's diagnostic rendering changes\n\
             # between versions. Bump the pin in `discover::gates::TOOLCHAIN` in lockstep with\n\
             # regenerating the `.stderr` files (TRYBUILD=overwrite), then re-freeze.\n\
             \n\
             on:\n\
             {sp2}push:\n\
             {sp4}branches: [main]\n\
             {sp2}pull_request:\n\
             {sp2}schedule:\n\
             {sp4}- cron: \"{cron}\" # the periodic whole-crate guarantee\n\
             {sp2}workflow_dispatch:\n\
             \n\
             env:\n\
             {sp2}CARGO_TERM_COLOR: always\n\
             {sp2}RUSTFLAGS: \"-D warnings\"\n\
             \n\
             jobs:\n\
             {sp2}check:\n\
             {sp4}name: fmt + clippy + test\n\
             {sp4}runs-on: ubuntu-latest\n\
             {sp4}steps:\n\
             {sp6}- uses: actions/checkout@v4\n\
             {sp6}- uses: dtolnay/rust-toolchain@{toolchain}\n\
             {sp8}with:\n\
             {sp10}components: rustfmt, clippy\n\
             {sp6}- uses: Swatinem/rust-cache@v2\n",
            cron = FULL_SWEEP_CRON,
            toolchain = TOOLCHAIN,
            sp2 = "  ",
            sp4 = "    ",
            sp6 = "      ",
            sp8 = "        ",
            sp10 = "          ",
        );
        for gate in &every_change {
            out.push_str(&format!("      - run: {}\n", gate.command_line()));
        }
        for gate in &per_diff {
            out.push_str(&format!(
                "\n  # Per-change dogfood: mutate only the lines this PR touches. New code that a\n\
                 \x20 # probe cannot kill fails the gate; pre-existing documented equivalents are\n\
                 \x20 # not in the diff, so they never re-trip it.\n\
                 \x20 mutants-diff:\n\
                 \x20   name: dogfood (changed lines)\n\
                 \x20   if: github.event_name == 'pull_request'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20       with:\n\
                 \x20         fetch-depth: 0\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants\n\
                 \x20     - name: mutate the diff\n\
                 \x20       run: |\n\
                 \x20         git diff \"origin/${{{{ github.base_ref }}}}...HEAD\" > pr.diff\n\
                 \x20         {}\n",
                gate.command_line()
            ));
        }
        for gate in &sweep {
            out.push_str(&format!(
                "\n  # Whole-crate dogfood: the green gate is \"0 MISSED\". Timeouts (non-termination\n\
                 \x20 # mutants) are DETECTIONS, not survivors — `.github/mutants-gate.sh` distinguishes\n\
                 \x20 # a timeout-only exit (pass) from a real survivor in `missed.txt` (fail). The\n\
                 \x20 # ratified equivalents are carved out by `.cargo/mutants.toml`.\n\
                 \x20 mutants-full:\n\
                 \x20   name: dogfood (full sweep)\n\
                 \x20   if: github.event_name != 'pull_request'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants\n\
                 \x20     - run: {}\n",
                gate.command_line()
            ));
        }
        out
    }

    /// The registry lock (`spec/gates.spec`) — the pipeline's PROMISES, ratified.
    pub fn registry_lock() -> Lock {
        Lock {
            name: "gate registry".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("gates.spec"),
            live: Self::render_registry(),
        }
    }

    /// The workflow lock — `.github/workflows/ci.yml` itself, held byte for byte to the
    /// registry's render. A hand edit to the YAML is DRIFT, caught by the same gate the
    /// YAML executes: the pipeline validates itself on every change it runs.
    pub fn workflow_lock() -> Lock {
        Lock {
            name: "ci workflow".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".github")
                .join("workflows")
                .join("ci.yml"),
            live: Self::render_workflow(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// BOTH pipeline locks are fresh: the committed inventory and the committed workflow
    /// match the registry's renders. A hand edit to `ci.yml` fails HERE — inside the very
    /// `cargo test` the workflow runs — so the pipeline can no longer drift silently the
    /// way it did (the `--workspace` gap lived undetected precisely because nothing held
    /// the YAML to a declaration).
    #[test]
    fn the_pipeline_locks_are_fresh() {
        let locks = [GateRegistry::registry_lock(), GateRegistry::workflow_lock()];
        if let Err(stale) = spec_lock::check(&locks) {
            panic!(
                "the pipeline drifted from its declaration: {}. Regenerate with \
                 `cargo run --example freeze_gates` and ratify the diff — never edit the \
                 YAML or the spec by hand.",
                stale.join(", ")
            );
        }
    }

    /// THE PAID-FOR BUG CLASS, as a census over the DATA: every cargo gate that runs tests
    /// or lints carries `--workspace` — the fixture-silently-untested gap cannot recur as
    /// a YAML editing accident, because the scope now lives in one reviewed declaration.
    #[test]
    fn workspace_scoped_gates_stay_workspace_scoped() {
        for gate in GateRegistry::declared() {
            let is_cargo_verifier = gate.command.first() == Some(&"cargo")
                && matches!(gate.command.get(1), Some(&"test") | Some(&"clippy"));
            if is_cargo_verifier {
                assert!(
                    gate.command.contains(&"--workspace"),
                    "gate `{}` runs a package-scoped verifier — the whole workspace is \
                     the unit of the pipeline: {:?}",
                    gate.name,
                    gate.command
                );
            }
        }
        // and fmt's workspace scope is spelled --all in cargo-fmt's dialect.
        let fmt = &GateRegistry::declared()[0];
        assert!(fmt.command.contains(&"--all"));
    }

    /// The registry's SHAPE is pinned: five gates — three every-change (all pure), one
    /// per-diff, one scheduled sweep — and every declared command reappears verbatim in
    /// the rendered workflow (nothing declared can fall out of execution).
    #[test]
    fn every_declared_gate_is_rendered_into_the_workflow() {
        let gates = GateRegistry::declared();
        assert_eq!(gates.len(), 5);
        assert_eq!(
            gates
                .iter()
                .filter(|g| g.cadence == Cadence::EveryChange)
                .count(),
            3
        );
        assert!(gates.iter().all(|g| g.effect == Capability::Pure));

        let workflow = GateRegistry::render_workflow();
        for gate in &gates {
            assert!(
                workflow.contains(&gate.command_line()),
                "gate `{}` is declared but not executed by the workflow",
                gate.name
            );
        }
        assert!(workflow.contains(&format!("dtolnay/rust-toolchain@{TOOLCHAIN}")));
        assert!(workflow.contains(&format!("- cron: \"{FULL_SWEEP_CRON}\"")));
        assert!(workflow.contains("GENERATED from the gate registry"));
    }

    /// The inventory render is pinned at its load-bearing points: the header names the
    /// regeneration path, and a stanza carries cadence, capability, command, and promise.
    #[test]
    fn the_registry_renders_the_promises() {
        let text = GateRegistry::render_registry();
        assert!(text.starts_with(
            "# gate registry: the pipeline as a declaration — regenerate via `cargo run --example freeze_gates`; ratify the diff.\n"
        ));
        assert!(text.contains(
            "\n- test (every change; pure)\n      cargo test --workspace --all-targets\n"
        ));
        assert!(text.contains("\n- mutation (full sweep) (default branch + weekly; pure)\n"));
    }
}
