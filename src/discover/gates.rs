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
//!     the exact command, its CADENCE (every change / per PR diff / default-branch drift
//!     since the certified tree / weekly sharded whole-tree sweep),
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
//! What CI irreducibly keeps: countersigning (a green run the author didn't produce — the
//! `mutants-green` tag is that signature made durable), effects (edges the tree cannot
//! contain), and economics (the whole-tree mutation sweep is too expensive per keystroke,
//! so merges pay only for their drift since the certified tree and the weekly clock pays
//! for everything, split across `FULL_SWEEP_SHARDS` parallel shards). All three are now
//! VISIBLE as registry data instead of implicit in YAML.

use std::path::{Path, PathBuf};

use spec_lock::Lock;

use crate::boundary::Capability;

/// The pinned Rust toolchain the workflow installs. Pinned, not `@stable`: the
/// `tests/compile_fail` trybuild suites match saved `.stderr` files, and rustc's diagnostic
/// rendering changes between versions. Bump this in lockstep with regenerating the
/// `.stderr` files (`TRYBUILD=overwrite`), then re-freeze the gates.
pub const TOOLCHAIN: &str = "1.94.1";

/// The weekly full-sweep schedule (Mondays 04:00 UTC) — the periodic whole-crate guarantee.
pub const FULL_SWEEP_CRON: &str = "0 4 * * 1";

/// The every-change job's display name — the status-check CONTEXT a branch ruleset
/// requires. One constant shared by the workflow renders and the perimeter declaration,
/// so renaming the job drifts the perimeter lock in the same diff: a rename can never
/// silently unprotect the default branch.
pub const CHECK_JOB: &str = "fmt + clippy + test";

/// How many parallel shards the weekly full sweep splits into. The whole workspace is
/// ~1,300 mutants; serially that is ~5½ hours — past what a single hosted runner reliably
/// survives (the sweep on PR #26's merge died at 5h16m with the gate step still running).
/// `cargo mutants --shard k/n` partitions the mutant list deterministically, so `n` jobs
/// in a matrix give the same total verdict in `1/n` the wall clock.
pub const FULL_SWEEP_SHARDS: u8 = 8;

/// The tag that marks the last tree the mutation gates have FULLY certified — the
/// incremental sweep on the default branch mutates only the diff since this tag and
/// advances it on green (CI's countersignature, the one effect in the pipeline). The
/// weekly sharded sweep re-certifies the whole tree from scratch, backstopping the one
/// gap incrementality has: a test edit weakening kills for UNCHANGED code.
pub const GREEN_TAG: &str = "mutants-green";

/// When a gate runs — the ECONOMIC axis of the pipeline, explicit instead of implied by
/// which YAML job a step happens to sit in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cadence {
    /// Every push and every PR — cheap enough to pay per change.
    EveryChange,
    /// PRs only, scoped to the changed lines (mutation is expensive; the diff is the
    /// blast radius).
    PerDiff,
    /// Default-branch pushes, scoped to the diff since the last fully-certified tree
    /// (`GREEN_TAG`) — a green sweep is a lock over the tree at that sha, so a merge
    /// re-verifies only what drifted, and the tag advances as the countersignature.
    DefaultBranch,
    /// The weekly clock and manual dispatch — the exhaustive whole-tree sweep, sharded
    /// `FULL_SWEEP_SHARDS` ways.
    Weekly,
    /// When the countersign advances the certified-tree tag (`GREEN_TAG`) — after the
    /// incremental gate on a default-branch merge, and after a green weekly sweep. The
    /// release cadence: a certified tree is the event, never a human decision.
    OnCertify,
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
    /// Weekly gates only: split across the `FULL_SWEEP_SHARDS` matrix (the whole-tree
    /// sweep's economics). An unsharded weekly gate renders as one plain job.
    pub sharded: bool,
}

impl Gate {
    /// The command as one shell-displayable line (argv joined — no shell interpretation
    /// anywhere in the pipeline's own machinery).
    pub fn command_line(&self) -> String {
        self.command.join(" ")
    }
}

/// A dogfood gate's RENDERED job name — the status-check CONTEXT GitHub reports (the
/// `mutation (...)` registry sugar unwrapped into the `dogfood (...)` display form).
/// Shared by the workflow render and [`GateRegistry::pr_checks`], so the perimeter's
/// required contexts and the executed job names are one computation, never two.
fn check_context(gate: &Gate) -> String {
    format!(
        "dogfood ({})",
        gate.name
            .strip_prefix("mutation (")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(gate.name)
    )
}

/// The registry inventory's stanza list — one stanza per gate (name, cadence, capability,
/// command, promise). ONE render for this repo's registry and every consumer pipeline, so
/// the two lock dialects cannot drift apart.
fn registry_stanzas(gates: &[Gate]) -> String {
    let mut out = String::new();
    for gate in gates {
        let cadence = match gate.cadence {
            Cadence::EveryChange => "every change",
            Cadence::PerDiff => "per PR diff",
            Cadence::DefaultBranch => "default branch, diff since mutants-green",
            Cadence::Weekly => "weekly + manual, sharded",
            Cadence::OnCertify => "on certification, when the mutants-green tag advances",
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

/// A GitHub Actions job id from a gate's name — job ids allow only alphanumerics and
/// dashes.
fn job_slug(label: &str) -> String {
    let mut id = String::new();
    for c in label.chars() {
        match c {
            c if c.is_ascii_alphanumeric() => id.push(c),
            _ if id.ends_with('-') || id.is_empty() => {}
            _ => id.push('-'),
        }
    }
    id.trim_end_matches('-').to_string()
}

/// A DOWNSTREAM pipeline declaration — the CONSUMER form of the gate discipline, the
/// sibling of `Spec::lock_in`. This repo's own [`GateRegistry`] stays bespoke (its
/// workflow carries the green-tag countersign and the sharded-sweep economics, sized to
/// this workspace); a consumer declares its OWN gates here and derives both locks rooted
/// in ITS repository:
///
/// ```ignore
/// let pipeline = Pipeline::starter(); // or a hand-declared Pipeline { .. }
/// spec_lock::bless(&pipeline.locks_in(Path::new(env!("CARGO_MANIFEST_DIR")))?)?;
/// ```
///
/// The render covers the tiers a downstream pipeline generally needs — every-change
/// gates (one `check` job), per-diff mutation, and unsharded weekly jobs on a declared
/// schedule. The two bespoke tiers REFUSE by name rather than render wrong YAML: the
/// default-branch incremental cadence is green-tag countersign economics, and sharding
/// is sweep economics sized to a mutant count — both live in this repo's own render as
/// the reference for a consumer who grows into them.
pub struct Pipeline {
    /// The workflow's name (`name:` in the YAML, and the workflow file's stem).
    pub name: &'static str,
    /// The pinned toolchain the workflow installs.
    pub toolchain: &'static str,
    /// The command that regenerates both locks — named in the lock headers so the fix
    /// for drift is always spelled where the drift is reported.
    pub regen: &'static str,
    /// The weekly schedule, when any weekly gate is declared.
    pub cron: Option<&'static str>,
    /// The declared gates, in execution order.
    pub gates: Vec<Gate>,
}

impl Pipeline {
    /// The STARTER pipeline — the three every-change gates every crate in this
    /// discipline runs (format, lint, test — workspace-scoped, the paid-for lesson),
    /// pinned to the toolchain this library is tested against. Genesis emits generated
    /// crates' pipelines as a call to this constructor, so the starter is ONE
    /// declaration, not a restatement per crate; an adopter outgrowing it replaces the
    /// call with a hand-declared `Pipeline { .. }`.
    pub fn starter() -> Pipeline {
        Pipeline {
            name: "ci",
            toolchain: TOOLCHAIN,
            regen: "cargo run --example freeze_gates",
            cron: None,
            gates: vec![
                Gate {
                    name: "format",
                    verifies: "the whole workspace is rustfmt-canonical",
                    command: &["cargo", "fmt", "--all", "--check"],
                    cadence: Cadence::EveryChange,
                    effect: Capability::Pure,
                    sharded: false,
                },
                Gate {
                    name: "lint",
                    verifies: "clippy holds every workspace member, all targets and \
                               features, to deny-warnings",
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
                    sharded: false,
                },
                Gate {
                    name: "test",
                    verifies: "every suite: the drift gates (module, system, and gates \
                               locks), the distance gates, and the probes",
                    command: &["cargo", "test", "--workspace", "--all-targets"],
                    cadence: Cadence::EveryChange,
                    effect: Capability::Pure,
                    sharded: false,
                },
            ],
        }
    }

    /// The human-readable inventory — what the consumer's `spec/gates.spec` locks. Same
    /// stanza dialect as this repo's registry (one shared render), consumer header.
    pub fn render_registry(&self) -> String {
        let mut out = format!(
            "# gate registry: the pipeline as a declaration — regenerate via `{}`; ratify the diff.\n\
             #\n\
             # Every gate below is a declared, executable claim: the workflow CI runs is\n\
             # RENDERED from this same declaration and drift-gated byte for byte, so \"green\n\
             # locally\" and \"green in CI\" are one claim. Cadence and capability are visible\n\
             # here instead of implicit in YAML.\n",
            self.regen
        );
        out.push_str(&registry_stanzas(&self.gates));
        out
    }

    /// The consumer workflow — every-change gates as one `check` job, per-diff mutation
    /// and unsharded weekly jobs as declared. The bespoke tiers refuse by name (see the
    /// type docs); a duplicate job id refuses rather than emitting colliding YAML.
    pub fn render_workflow(&self) -> Result<String, String> {
        for gate in &self.gates {
            if gate.cadence == Cadence::DefaultBranch {
                return Err(format!(
                    "gate `{}` declares the default-branch incremental cadence — that tier \
                     is green-tag countersign economics, implemented bespoke in this \
                     library's own pipeline (`GateRegistry::render_workflow` is the \
                     reference); a consumer pipeline declares EveryChange, PerDiff, or an \
                     unsharded Weekly gate",
                    gate.name
                ));
            }
            if gate.cadence == Cadence::OnCertify {
                return Err(format!(
                    "gate `{}` declares the on-certification cadence — that tier rides \
                     the green-tag countersign, implemented bespoke in this library's \
                     own pipeline (`GateRegistry::render_workflow` is the reference); a \
                     consumer pipeline declares EveryChange, PerDiff, or an unsharded \
                     Weekly gate",
                    gate.name
                ));
            }
            if gate.sharded {
                return Err(format!(
                    "gate `{}` declares sharding — the shard matrix is whole-tree sweep \
                     economics sized to a specific mutant count (`GateRegistry::\
                     render_workflow` is the reference); declare the gate unsharded or \
                     render a bespoke workflow",
                    gate.name
                ));
            }
        }
        let weekly: Vec<&Gate> = self
            .gates
            .iter()
            .filter(|g| g.cadence == Cadence::Weekly)
            .collect();
        if !weekly.is_empty() && self.cron.is_none() {
            return Err(format!(
                "gate `{}` is weekly but the pipeline declares no schedule — set \
                 `Pipeline::cron`",
                weekly[0].name
            ));
        }
        let per_diff: Vec<&Gate> = self
            .gates
            .iter()
            .filter(|g| g.cadence == Cadence::PerDiff)
            .collect();
        let mut job_ids = vec!["check".to_string()];
        for gate in per_diff
            .iter()
            .map(|g| ("diff", g))
            .chain(weekly.iter().map(|g| ("weekly", g)))
        {
            let id = format!("{}-{}", gate.0, job_slug(gate.1.name));
            if job_ids.contains(&id) {
                return Err(format!(
                    "two gates render the same workflow job id `{id}` — rename one"
                ));
            }
            job_ids.push(id);
        }

        let mut out = format!(
            "# GENERATED from this crate's gate declaration — THE PIPELINE IS A LOCK.\n\
             # Never edit by hand: regenerate with `{regen}` and ratify the diff. The\n\
             # declaration is the single source for the commands, the cadences, and the\n\
             # toolchain pin; `spec/gates.spec` carries the promises.\n\
             name: {name}\n\
             \n\
             on:\n\
             {sp2}push:\n\
             {sp4}branches: [main]\n\
             {sp2}pull_request:\n",
            regen = self.regen,
            name = self.name,
            sp2 = "  ",
            sp4 = "    ",
        );
        if let (Some(cron), false) = (self.cron, weekly.is_empty()) {
            out.push_str(&format!(
                "  schedule:\n    - cron: \"{cron}\"\n  workflow_dispatch:\n"
            ));
        }
        out.push_str(&format!(
            "\n\
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
            toolchain = self.toolchain,
            sp2 = "  ",
            sp4 = "    ",
            sp6 = "      ",
            sp8 = "        ",
            sp10 = "          ",
        ));
        for gate in self
            .gates
            .iter()
            .filter(|g| g.cadence == Cadence::EveryChange)
        {
            out.push_str(&format!("      - run: {}\n", gate.command_line()));
        }
        for gate in &per_diff {
            out.push_str(&format!(
                "\n  # Per-change dogfood: mutate only the lines this PR touches — see the\n\
                 \x20 # gate registry for what it promises.\n\
                 \x20 diff-{slug}:\n\
                 \x20   name: {label}\n\
                 \x20   if: github.event_name == 'pull_request'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20       with:\n\
                 \x20         fetch-depth: 0\n\
                 \x20     - uses: dtolnay/rust-toolchain@{toolchain}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants,cargo-nextest\n\
                 \x20     - name: mutate the diff\n\
                 \x20       run: |\n\
                 \x20         git diff \"origin/${{{{ github.base_ref }}}}...HEAD\" > pr.diff\n\
                 \x20         {command}\n",
                slug = job_slug(gate.name),
                label = gate.name,
                toolchain = self.toolchain,
                command = gate.command_line(),
            ));
        }
        for gate in &weekly {
            out.push_str(&format!(
                "\n  # Weekly gate: one plain job on the declared schedule — see the gate\n\
                 \x20 # registry for what it promises.\n\
                 \x20 weekly-{slug}:\n\
                 \x20   name: {label}\n\
                 \x20   if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - uses: dtolnay/rust-toolchain@{toolchain}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants,cargo-nextest\n\
                 \x20     - run: {command}\n",
                slug = job_slug(gate.name),
                label = gate.name,
                toolchain = self.toolchain,
                command = gate.command_line(),
            ));
        }
        Ok(out)
    }

    /// BOTH locks, rooted in a caller-supplied repository root: the registry inventory at
    /// `spec/gates.spec` and the workflow at `.github/workflows/<name>.yml` — the exact
    /// sibling of `Spec::lock_in`, one call for a consumer's whole pipeline freeze. A
    /// declaration the render refuses surfaces here as the named refusal, never as a
    /// half-written lock list.
    pub fn locks_in(&self, root: &Path) -> Result<Vec<Lock>, String> {
        let workflow = self.render_workflow()?;
        Ok(vec![
            Lock {
                name: format!("{} gate registry", self.name),
                path: root.join("spec").join("gates.spec"),
                live: self.render_registry(),
            },
            Lock {
                name: format!("{} workflow", self.name),
                path: root
                    .join(".github")
                    .join("workflows")
                    .join(format!("{}.yml", self.name)),
                live: workflow,
            },
        ])
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
                sharded: false,
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
                sharded: false,
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
                sharded: false,
            },
            Gate {
                name: "mutation (changed lines)",
                verifies: "no mutant of the PR's changed lines survives the probe suite \
                           (timeouts are detections; ratified equivalents live in \
                           .cargo/mutants.toml)",
                command: &[".github/mutants-gate.sh", "--in-diff", "pr.diff"],
                cadence: Cadence::PerDiff,
                effect: Capability::Pure,
                sharded: false,
            },
            Gate {
                name: "mutation (since green)",
                verifies: "no mutant of anything changed since the last fully-certified \
                           tree (the mutants-green tag) survives — a merge re-verifies \
                           its drift, not the whole tree, and advances the tag on green",
                command: &[".github/mutants-gate.sh", "--in-diff", "since-green.diff"],
                cadence: Cadence::DefaultBranch,
                effect: Capability::Pure,
                sharded: false,
            },
            Gate {
                name: "mutation (full sweep)",
                verifies: "no mutant of the whole crate survives — the method's own \"the \
                           real test\", re-certifying the tree from scratch on a weekly \
                           clock (backstops incrementality's one gap: a test edit \
                           weakening kills for unchanged code)",
                command: &[".github/mutants-gate.sh"],
                cadence: Cadence::Weekly,
                effect: Capability::Pure,
                sharded: true,
            },
            Gate {
                name: "mutation (delta-render plumbing)",
                verifies: "no mutant of delta-render's plumbing (the license parser, the \
                           circuit validity rule, the render, the stream calculus) \
                           survives its lib probes — the workspace sweeps scope to the \
                           root crate, so the member that turns specs into generated \
                           code carries its own weekly verdict (config: \
                           .github/delta-render-mutants.toml; the drift-gate twins live \
                           in the lib precisely so this sweep can see them)",
                command: &[
                    ".github/mutants-gate.sh",
                    "--package",
                    "delta-render",
                    "--config",
                    ".github/delta-render-mutants.toml",
                ],
                cadence: Cadence::Weekly,
                effect: Capability::Pure,
                sharded: false,
            },
            Gate {
                name: "statement bites (lean corpus)",
                verifies: "no definition mutant of lean/ProbeBool.lean re-checks past its \
                           theorems, except the survivors ratified by key in \
                           lean/bites.register — mutation testing FOR the proof corpus: \
                           the kernel judges the mutants (the gate installs elan; the \
                           corpus is core-only), while the expected survivor set is \
                           pinned toolchain-free by discover::bite's mirror probe in \
                           every cargo test",
                command: &[".github/statement-bite.sh"],
                cadence: Cadence::Weekly,
                effect: Capability::Pure,
                sharded: false,
            },
            Gate {
                name: "mutation (fire-drill plumbing)",
                verifies: "no mutant of fire-drill (the battery verdicts, the census \
                           refusals, both lockable renders) survives its lib probes — \
                           the workspace sweeps scope to the root crate, so the crate \
                           that proves gates can fire carries its own weekly verdict \
                           (config: .github/fire-drill-mutants.toml)",
                command: &[
                    ".github/mutants-gate.sh",
                    "--package",
                    "fire-drill",
                    "--config",
                    ".github/fire-drill-mutants.toml",
                ],
                cadence: Cadence::Weekly,
                effect: Capability::Pure,
                sharded: false,
            },
            Gate {
                name: "mutation (layout-probe plumbing)",
                verifies: "no mutant of layout-probe (the two engines, the diagram \
                           edits, the visual census and its locality witness) survives \
                           its lib probes — the workspace sweeps scope to the root \
                           crate, so the second-domain miniature carries its own weekly \
                           verdict (config: .github/layout-probe-mutants.toml; the \
                           drift-gate twins live in the lib so this sweep can see them)",
                command: &[
                    ".github/mutants-gate.sh",
                    "--package",
                    "layout-probe",
                    "--config",
                    ".github/layout-probe-mutants.toml",
                ],
                cadence: Cadence::Weekly,
                effect: Capability::Pure,
                sharded: false,
            },
            Gate {
                name: "release (certified tree)",
                verifies: "every certified default-branch tree publishes itself: the \
                           countersign's tag advance IS the release event, the version \
                           is CalVer (a date claims nothing about compatibility, which \
                           is honest), and the notes are DERIVED — commit subjects plus \
                           the ratified spec-lock diff, the uncompressed truth a semver \
                           integer would compress into an unchecked claim",
                command: &[".github/release.sh"],
                cadence: Cadence::OnCertify,
                effect: Capability::Effectful,
                sharded: false,
            },
            Gate {
                name: "perimeter (settings drift)",
                verifies: "the LIVE repository perimeter — branch rules on the default \
                           branch, merge methods, private vulnerability reporting — still \
                           satisfies the declared floor (spec/perimeter.spec). Settings \
                           are configuration that drifts silently and that no one \
                           re-audits; this gate reads them back on the weekly clock and \
                           refuses by name. READ-ONLY: the write stays human — a \
                           privilege is ratified, never self-served",
                command: &[".github/perimeter.sh"],
                cadence: Cadence::Weekly,
                effect: Capability::Effectful,
                sharded: false,
            },
            Gate {
                name: "substrate (git drift)",
                verifies: "the LIVE repository's tags and history still satisfy the \
                           declared git substrate (spec/substrate.spec): the tags the \
                           machinery leans on exist and sit on the certified line, \
                           every published root-crate version carries its v<version> \
                           marker (instances DERIVED from the crates.io index, never \
                           named in the declaration), and the default branch stays \
                           linear after the declared epoch — the perimeter's \
                           squash-only rule, judged backward. READ-ONLY and \
                           credential-free: git plumbing against the checkout's own \
                           origin plus one anonymous sparse-index read",
                command: &[".github/substrate.sh"],
                cadence: Cadence::Weekly,
                effect: Capability::Effectful,
                sharded: false,
            },
        ]
    }

    /// The status-check contexts a PR must pass before merging — the every-change job
    /// plus each per-diff gate, by their RENDERED job names. The perimeter declaration
    /// consumes this, which is the point of deriving it: a gate added, renamed, or
    /// re-cadenced here moves `spec/perimeter.spec` in the same diff, and the weekly
    /// read-back then holds the live branch rules to it.
    pub fn pr_checks() -> Vec<String> {
        let mut checks = vec![CHECK_JOB.to_string()];
        checks.extend(
            Self::declared()
                .iter()
                .filter(|g| g.cadence == Cadence::PerDiff)
                .map(check_context),
        );
        checks
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
        out.push_str(&registry_stanzas(&Self::declared()));
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
        let default_branch: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.cadence == Cadence::DefaultBranch)
            .collect();
        let weekly: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.cadence == Cadence::Weekly)
            .collect();
        let on_certify: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.cadence == Cadence::OnCertify)
            .collect();
        // the release steps render after BOTH tag-advance sites — the countersign IS
        // the release event, so wherever the tag moves, the certified tree publishes.
        let release_steps = |out: &mut String| {
            for gate in &on_certify {
                out.push_str(&format!(
                    "      - name: release — the certified tree publishes itself\n\
                     \x20       env:\n\
                     \x20         GH_TOKEN: ${{{{ github.token }}}}\n\
                     \x20         CARGO_REGISTRY_TOKEN: ${{{{ secrets.CARGO_REGISTRY_TOKEN }}}}\n\
                     \x20       run: {}\n",
                    gate.command_line()
                ));
            }
        };

        let mut out = format!(
            "# GENERATED from the gate registry (`discover::gates`) — THE PIPELINE IS A LOCK.\n\
             # Never edit by hand: regenerate with `cargo run --example freeze_gates` and ratify\n\
             # the diff. The registry is the single source for the commands, cadences, the\n\
             # toolchain pin, and the sweep schedule; `spec/gates.spec` carries the promises.\n\
             name: ci\n\
             \n\
             # Default-deny at the top; the two tag-advance/release jobs elevate to\n\
             # `contents: write` explicitly. (Scorecard's token-permissions criterion,\n\
             # and simply the capability-honesty rule applied to CI itself.)\n\
             permissions:\n\
             {sp2}contents: read\n\
             \n\
             # Dogfood everywhere, all the time. The fast gates run on every push and PR; the\n\
             # mutation gate — the method's own \"the real test\" — runs per-change on PRs (only\n\
             # the changed lines), incrementally on the default branch (only the diff since the\n\
             # last fully-certified tree, marked by the `mutants-green` tag, advanced on green),\n\
             # and as a sharded whole-crate sweep on the weekly clock.\n\
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
                 \x20   name: {context}\n\
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
                 \x20         tool: cargo-mutants,cargo-nextest\n\
                 \x20     - name: mutate the diff\n\
                 \x20       run: |\n\
                 \x20         git diff \"origin/${{{{ github.base_ref }}}}...HEAD\" > pr.diff\n\
                 \x20         {}\n",
                gate.command_line(),
                context = check_context(gate)
            ));
        }
        for gate in &default_branch {
            out.push_str(&format!(
                "\n  # Incremental dogfood on the default branch: a green sweep is a lock over the\n\
                 \x20 # tree at that sha (the `{GREEN_TAG}` tag), so a merge mutates only the diff\n\
                 \x20 # since it. On green the tag advances — CI's countersignature, the pipeline's\n\
                 \x20 # one effect. A red gate leaves the tag put, so the next merge re-verifies the\n\
                 \x20 # accumulated drift. The weekly sharded sweep re-certifies from scratch.\n\
                 \x20 mutants-incremental:\n\
                 \x20   name: dogfood (since green)\n\
                 \x20   if: github.event_name == 'push'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   permissions:\n\
                 \x20     contents: write\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20       with:\n\
                 \x20         fetch-depth: 0\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants,cargo-nextest\n\
                 \x20     - name: mutate the drift since the certified tree\n\
                 \x20       run: |\n\
                 \x20         git rev-parse -q --verify refs/tags/{GREEN_TAG} >/dev/null || {{ echo \"::error::no {GREEN_TAG} tag - bootstrap by dispatching this workflow (Actions -> ci -> Run workflow): the full sweep's countersign plants the tag it earns\"; exit 1; }}\n\
                 \x20         git diff \"{GREEN_TAG}...HEAD\" > since-green.diff\n\
                 \x20         {}\n\
                 \x20     - name: countersign — advance the certified-tree tag\n\
                 \x20       run: |\n\
                 \x20         git tag -f {GREEN_TAG}\n\
                 \x20         git push -f origin {GREEN_TAG}\n",
                gate.command_line()
            ));
            release_steps(&mut out);
        }
        // weekly certification: the sharded whole-tree sweep, any unsharded weekly
        // companions (each a plain job), and ONE countersign that needs them all — the
        // tag is the whole weekly verdict, not one job's.
        // a companion's display label: the `mutation (...)` sugar unwrapped when present,
        // the full gate name otherwise (a non-mutation weekly gate keeps its own words).
        let companion_label = |gate: &Gate| -> &str {
            gate.name
                .strip_prefix("mutation (")
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or(gate.name)
        };
        let weekly_job_id = |gate: &Gate| -> String {
            match gate.sharded {
                true => "mutants-full".to_string(),
                false => {
                    // job ids allow only alphanumerics and dashes: slug the label.
                    let mut id = String::from("mutants-");
                    for c in companion_label(gate).chars() {
                        match c {
                            c if c.is_ascii_alphanumeric() => id.push(c),
                            _ if id.ends_with('-') => {}
                            _ => id.push('-'),
                        }
                    }
                    id.trim_end_matches('-').to_string()
                }
            }
        };
        for gate in weekly
            .iter()
            .filter(|g| !g.sharded && g.effect != Capability::Effectful)
        {
            out.push_str(&format!(
                "\n  # Weekly companion gate: one plain job on the weekly clock, feeding the\n\
                 \x20 # same countersign — see the gate registry for what it verifies.\n\
                 \x20 {job_id}:\n\
                 \x20   name: dogfood ({label})\n\
                 \x20   if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants,cargo-nextest\n\
                 \x20     - run: {}\n",
                gate.command_line(),
                job_id = weekly_job_id(gate),
                label = companion_label(gate),
            ));
        }
        // WORLD gates: weekly Effectful reads of state the tree cannot contain (the
        // repository perimeter). They ride the weekly clock but never feed the
        // countersign — a world fact is not evidence about the TREE, and the certified
        // tag must never wait on someone's settings page. GH_TOKEN is the workflow's
        // ordinary read token; the world gates only ever read.
        for gate in weekly
            .iter()
            .filter(|g| !g.sharded && g.effect == Capability::Effectful)
        {
            out.push_str(&format!(
                "\n  # World gate: read state the tree cannot contain, hold it to the declared\n\
                 \x20 # floor — see the gate registry for what it verifies. Not a countersign\n\
                 \x20 # input: a world fact is not evidence about the tree.\n\
                 \x20 world-{slug}:\n\
                 \x20   name: {label}\n\
                 \x20   if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   env:\n\
                 \x20     GH_TOKEN: ${{{{ github.token }}}}\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - run: {}\n",
                gate.command_line(),
                slug = job_slug(gate.name),
                label = gate.name,
            ));
        }
        let weekly_needs: Vec<String> = weekly
            .iter()
            .filter(|g| g.effect != Capability::Effectful)
            .map(|g| weekly_job_id(g))
            .collect();
        for gate in weekly.iter().filter(|g| g.sharded) {
            let shards: Vec<String> = (0..FULL_SWEEP_SHARDS).map(|s| s.to_string()).collect();
            out.push_str(&format!(
                "\n  # Whole-crate dogfood: the green gate is \"0 MISSED\". Timeouts (non-termination\n\
                 \x20 # mutants) are DETECTIONS, not survivors — `.github/mutants-gate.sh` distinguishes\n\
                 \x20 # a timeout-only exit (pass) from a real survivor in `missed.txt` (fail). The\n\
                 \x20 # ratified equivalents are carved out by `.cargo/mutants.toml`. Sharded: the\n\
                 \x20 # ~1,300-mutant sweep is ~5½ serial hours, past a hosted runner's patience.\n\
                 \x20 mutants-full:\n\
                 \x20   name: dogfood (full sweep)\n\
                 \x20   if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   strategy:\n\
                 \x20     fail-fast: false\n\
                 \x20     matrix:\n\
                 \x20       shard: [{shard_list}]\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - uses: taiki-e/install-action@v2\n\
                 \x20       with:\n\
                 \x20         tool: cargo-mutants,cargo-nextest\n\
                 \x20     - run: {} --shard ${{{{ matrix.shard }}}}/{FULL_SWEEP_SHARDS}\n\
                 \n\
                 \x20 # The tag is only ever planted or advanced by a run that EARNED it: the\n\
                 \x20 # incremental gate advances it per-merge, and this countersign advances it when\n\
                 \x20 # EVERY shard of the full sweep is green — which is also the BOOTSTRAP (no tag\n\
                 \x20 # yet? dispatch this workflow once; the certification plants it) and the\n\
                 \x20 # recovery after a red stretch (the weekly green re-anchors the diff).\n\
                 \x20 # MAIN-ONLY: a branch dispatch runs the sweeps as evidence, but the tag names\n\
                 \x20 # the certified DEFAULT-BRANCH tree — a branch tip must never claim it.\n\
                 \x20 mutants-full-countersign:\n\
                 \x20   name: countersign (full sweep)\n\
                 \x20   needs: [{needs_list}]\n\
                 \x20   if: (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') && github.ref == 'refs/heads/main'\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   permissions:\n\
                 \x20     contents: write\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - name: countersign — advance the certified-tree tag\n\
                 \x20       run: |\n\
                 \x20         git tag -f {GREEN_TAG}\n\
                 \x20         git push -f origin {GREEN_TAG}\n",
                gate.command_line(),
                shard_list = shards.join(", "),
                needs_list = weekly_needs.join(", ")
            ));
            release_steps(&mut out);
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

    /// The registry's SHAPE is pinned: thirteen gates — three every-change, one
    /// per-diff, one default-branch incremental, seven weekly (the sharded whole-tree
    /// sweep, four member/corpus companions, and the two world gates), and the release
    /// gate on the certification cadence — and every declared command reappears
    /// verbatim in the rendered workflow (nothing declared can fall out of execution).
    /// Every gate is Pure except exactly three, each wearing the EFFECTFUL tag for its
    /// own reason: the release WRITES the world (tags, a GitHub release, crates.io),
    /// the perimeter READS state the tree cannot contain (the live repository
    /// settings), and the substrate READS the repository's own tags and history —
    /// which is also why no world gate feeds the countersign.
    #[test]
    fn every_declared_gate_is_rendered_into_the_workflow() {
        let gates = GateRegistry::declared();
        assert_eq!(gates.len(), 13);
        assert_eq!(
            gates
                .iter()
                .filter(|g| g.cadence == Cadence::Weekly)
                .count(),
            7
        );
        assert_eq!(
            gates.iter().filter(|g| g.sharded).count(),
            1,
            "exactly one gate fans out over the shard matrix"
        );
        assert_eq!(
            gates
                .iter()
                .filter(|g| g.cadence == Cadence::EveryChange)
                .count(),
            3
        );
        let effectful: Vec<&Gate> = gates
            .iter()
            .filter(|g| g.effect == Capability::Effectful)
            .collect();
        assert_eq!(
            effectful.len(),
            3,
            "the release (a world write), the perimeter and the substrate (world reads)"
        );
        assert_eq!(effectful[0].name, "release (certified tree)");
        assert_eq!(effectful[0].cadence, Cadence::OnCertify);
        assert_eq!(effectful[1].name, "perimeter (settings drift)");
        assert_eq!(effectful[1].cadence, Cadence::Weekly);
        assert_eq!(effectful[2].name, "substrate (git drift)");
        assert_eq!(effectful[2].cadence, Cadence::Weekly);

        // the world gates ride the weekly clock but are NOT countersign inputs, and
        // their jobs carry the read token: a world fact is not evidence about the tree.
        let workflow = GateRegistry::render_workflow();
        for world_job in [
            "world-perimeter-settings-drift",
            "world-substrate-git-drift",
        ] {
            assert!(workflow.contains(&format!("{world_job}:")), "{workflow}");
            assert!(
                !workflow.contains(&format!("{world_job},"))
                    && !workflow.contains(&format!(", {world_job}")),
                "`{world_job}` must never appear in the countersign's needs list"
            );
        }

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

    /// The SWEEP ECONOMICS are executed as declared: the weekly job fans out over exactly
    /// `FULL_SWEEP_SHARDS` matrix shards (and passes its shard to cargo-mutants), and the
    /// incremental job diffs against `GREEN_TAG` and advances it on green — the
    /// countersignature step exists and pushes the tag.
    #[test]
    fn the_sweep_economics_are_rendered() {
        let workflow = GateRegistry::render_workflow();
        let shard_list: Vec<String> = (0..FULL_SWEEP_SHARDS).map(|s| s.to_string()).collect();
        assert!(workflow.contains(&format!("shard: [{}]", shard_list.join(", "))));
        assert!(workflow.contains(&format!(
            "--shard ${{{{ matrix.shard }}}}/{FULL_SWEEP_SHARDS}"
        )));
        assert!(workflow.contains(&format!(
            "git diff \"{GREEN_TAG}...HEAD\" > since-green.diff"
        )));
        assert!(workflow.contains(&format!("git push -f origin {GREEN_TAG}")));
        // the full sweep runs on the clock and the button, never per-merge (that is the
        // incremental job's cadence — the whole point of the split).
        assert!(workflow.contains(
            "if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'"
        ));
        // the tag has NO manual path: it is planted and advanced only by runs that earned
        // it — the full sweep's countersign (also the bootstrap: dispatch the workflow
        // once), gated on every shard via `needs`, and the incremental gate per-merge.
        assert!(workflow.contains("mutants-full-countersign:"));
        // ... and the countersign is MAIN-ONLY: a branch dispatch runs the sweeps as
        // evidence, but the tag names the certified default-branch tree, so a branch
        // tip must never claim it (found live: a branch dispatch nearly moved the tag).
        assert!(workflow.contains(
            "if: (github.event_name == 'schedule' || github.event_name == \
             'workflow_dispatch') && github.ref == 'refs/heads/main'"
        ));
        // ... and it needs EVERY weekly job: the tag is the whole weekly verdict, so a
        // red companion gate withholds the countersign exactly like a red shard.
        assert!(workflow.contains(
            "needs: [mutants-full, mutants-delta-render-plumbing, \
             mutants-statement-bites-lean-corpus, mutants-fire-drill-plumbing, \
             mutants-layout-probe-plumbing]"
        ));
        let advances = workflow
            .matches(&format!("git push -f origin {GREEN_TAG}"))
            .count();
        assert_eq!(
            advances, 2,
            "the tag must advance in exactly two places: the incremental gate and the \
             full sweep's countersign"
        );
        // and the RELEASE rides both advances — wherever the tag moves, the certified
        // tree publishes itself, with the token in scope for `gh release create`.
        assert_eq!(
            workflow.matches("run: .github/release.sh").count(),
            2,
            "one release step per tag-advance site"
        );
        assert_eq!(
            workflow
                .matches("- name: release — the certified tree publishes itself")
                .count(),
            2
        );
        assert!(workflow.contains("GH_TOKEN: ${{ github.token }}"));
    }

    /// Everywhere cargo-mutants is installed, cargo-nextest rides along: the mutation
    /// gates run their per-mutant suites through nextest's fail-fast runner
    /// (`test_tool = "nextest"` in .cargo/mutants.toml), so a killed mutant stops at its
    /// first failing probe instead of running out the whole suite.
    #[test]
    fn every_mutation_job_installs_the_fail_fast_runner() {
        let workflow = GateRegistry::render_workflow();
        let installs: Vec<&str> = workflow
            .lines()
            .filter(|l| l.trim_start().starts_with("tool: ") && l.contains("cargo-mutants"))
            .collect();
        assert!(!installs.is_empty());
        for line in installs {
            assert!(
                line.contains("cargo-nextest"),
                "a mutation job installs cargo-mutants without its test runner: {line}"
            );
        }
        let config = include_str!("../../.cargo/mutants.toml");
        assert!(
            config.contains("test_tool = \"nextest\""),
            "the mutants config no longer selects nextest — the fail-fast speedup is gone"
        );
    }

    /// THE STARTER PIPELINE, byte-pinned end to end — this is exactly what a genesis
    /// crate is born with, so its shape is a product surface: three every-change gates,
    /// no schedule block (no weekly gates declared), one `check` job carrying every
    /// declared command.
    #[test]
    fn the_starter_pipeline_renders_exactly() {
        let starter = Pipeline::starter();
        assert_eq!(
            starter.render_registry(),
            "# gate registry: the pipeline as a declaration — regenerate via `cargo run --example freeze_gates`; ratify the diff.\n\
             #\n\
             # Every gate below is a declared, executable claim: the workflow CI runs is\n\
             # RENDERED from this same declaration and drift-gated byte for byte, so \"green\n\
             # locally\" and \"green in CI\" are one claim. Cadence and capability are visible\n\
             # here instead of implicit in YAML.\n\
             \n\
             - format (every change; pure)\n\
             \x20     cargo fmt --all --check\n\
             \x20     promises: the whole workspace is rustfmt-canonical\n\
             \n\
             - lint (every change; pure)\n\
             \x20     cargo clippy --workspace --all-targets --all-features -- -D warnings\n\
             \x20     promises: clippy holds every workspace member, all targets and features, to deny-warnings\n\
             \n\
             - test (every change; pure)\n\
             \x20     cargo test --workspace --all-targets\n\
             \x20     promises: every suite: the drift gates (module, system, and gates locks), the distance gates, and the probes\n"
        );
        assert_eq!(
            starter.render_workflow().expect("the starter renders"),
            format!(
                "# GENERATED from this crate's gate declaration — THE PIPELINE IS A LOCK.\n\
                 # Never edit by hand: regenerate with `cargo run --example freeze_gates` and ratify the diff. The\n\
                 # declaration is the single source for the commands, the cadences, and the\n\
                 # toolchain pin; `spec/gates.spec` carries the promises.\n\
                 name: ci\n\
                 \n\
                 on:\n\
                 \x20 push:\n\
                 \x20   branches: [main]\n\
                 \x20 pull_request:\n\
                 \n\
                 env:\n\
                 \x20 CARGO_TERM_COLOR: always\n\
                 \x20 RUSTFLAGS: \"-D warnings\"\n\
                 \n\
                 jobs:\n\
                 \x20 check:\n\
                 \x20   name: fmt + clippy + test\n\
                 \x20   runs-on: ubuntu-latest\n\
                 \x20   steps:\n\
                 \x20     - uses: actions/checkout@v4\n\
                 \x20     - uses: dtolnay/rust-toolchain@{TOOLCHAIN}\n\
                 \x20       with:\n\
                 \x20         components: rustfmt, clippy\n\
                 \x20     - uses: Swatinem/rust-cache@v2\n\
                 \x20     - run: cargo fmt --all --check\n\
                 \x20     - run: cargo clippy --workspace --all-targets --all-features -- -D warnings\n\
                 \x20     - run: cargo test --workspace --all-targets\n"
            )
        );
    }

    /// The consumer form GROWS to the general tiers: a per-diff mutation gate and a
    /// weekly gate render as their own jobs (per-diff on pull requests, weekly on the
    /// declared schedule — which also switches on the workflow's schedule block).
    #[test]
    fn the_consumer_tiers_render_their_jobs() {
        let mut pipeline = Pipeline::starter();
        pipeline.cron = Some("0 4 * * 1");
        pipeline.gates.push(Gate {
            name: "mutation (changed lines)",
            verifies: "no mutant of the PR's changed lines survives",
            command: &["cargo", "mutants", "--in-diff", "pr.diff"],
            cadence: Cadence::PerDiff,
            effect: Capability::Pure,
            sharded: false,
        });
        pipeline.gates.push(Gate {
            name: "mutation (full sweep)",
            verifies: "no mutant of the whole crate survives",
            command: &["cargo", "mutants"],
            cadence: Cadence::Weekly,
            effect: Capability::Pure,
            sharded: false,
        });
        let workflow = pipeline.render_workflow().expect("all tiers supported");
        assert!(workflow.contains("  schedule:\n    - cron: \"0 4 * * 1\"\n  workflow_dispatch:\n"));
        assert!(workflow.contains("  diff-mutation-changed-lines:\n"));
        assert!(workflow.contains("    if: github.event_name == 'pull_request'\n"));
        assert!(workflow
            .contains("        git diff \"origin/${{ github.base_ref }}...HEAD\" > pr.diff\n"));
        assert!(workflow.contains("  weekly-mutation-full-sweep:\n"));
        assert!(workflow.contains(
            "    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'\n"
        ));
        // every declared command reappears in execution — the census the root registry
        // pins, held for consumers too.
        for gate in &pipeline.gates {
            assert!(workflow.contains(&gate.command_line()));
        }
    }

    /// The bespoke tiers REFUSE by name — wrong YAML is never rendered, and each refusal
    /// points at the reference implementation. A weekly gate without a schedule and a
    /// job-id collision refuse the same way.
    #[test]
    fn the_bespoke_tiers_are_named_refusals() {
        let with = |gate: Gate| {
            let mut p = Pipeline::starter();
            p.gates.push(gate);
            p
        };
        let incremental = with(Gate {
            name: "mutation (since green)",
            verifies: "",
            command: &["cargo", "mutants"],
            cadence: Cadence::DefaultBranch,
            effect: Capability::Pure,
            sharded: false,
        });
        let refusal = incremental.render_workflow().unwrap_err();
        assert!(
            refusal.contains("green-tag countersign economics"),
            "{refusal}"
        );
        assert!(
            incremental.locks_in(Path::new(".")).is_err(),
            "a refused render never half-writes the lock list"
        );

        let sharded = with(Gate {
            name: "mutation (full sweep)",
            verifies: "",
            command: &["cargo", "mutants"],
            cadence: Cadence::Weekly,
            effect: Capability::Pure,
            sharded: true,
        });
        assert!(sharded
            .render_workflow()
            .unwrap_err()
            .contains("sweep economics"));

        let certify = with(Gate {
            name: "release (certified tree)",
            verifies: "",
            command: &[".github/release.sh"],
            cadence: Cadence::OnCertify,
            effect: Capability::Effectful,
            sharded: false,
        });
        assert!(certify
            .render_workflow()
            .unwrap_err()
            .contains("rides the green-tag countersign"));

        let unscheduled = with(Gate {
            name: "audit",
            verifies: "",
            command: &["cargo", "audit"],
            cadence: Cadence::Weekly,
            effect: Capability::Pure,
            sharded: false,
        });
        assert!(unscheduled
            .render_workflow()
            .unwrap_err()
            .contains("declares no schedule"));

        let mut colliding = with(Gate {
            name: "audit!",
            verifies: "",
            command: &["cargo", "audit"],
            cadence: Cadence::Weekly,
            effect: Capability::Pure,
            sharded: false,
        });
        colliding.cron = Some("0 4 * * 1");
        colliding.gates.push(Gate {
            name: "audit?",
            verifies: "",
            command: &["cargo", "audit"],
            cadence: Cadence::Weekly,
            effect: Capability::Pure,
            sharded: false,
        });
        assert!(colliding
            .render_workflow()
            .unwrap_err()
            .contains("same workflow job id `weekly-audit`"));
    }

    /// `locks_in` roots BOTH locks in the caller's repository: the registry under
    /// `spec/`, the workflow under `.github/workflows/<name>.yml` — the consumer twin
    /// of this repo's `registry_lock`/`workflow_lock`.
    #[test]
    fn the_consumer_locks_land_in_the_callers_repo() {
        let locks = Pipeline::starter()
            .locks_in(Path::new("/downstream"))
            .expect("the starter renders");
        let [registry, workflow] = locks.as_slice() else {
            panic!("two locks, got {}", locks.len());
        };
        assert_eq!(registry.name, "ci gate registry");
        assert_eq!(
            registry.path,
            Path::new("/downstream").join("spec").join("gates.spec")
        );
        assert_eq!(registry.live, Pipeline::starter().render_registry());
        assert_eq!(workflow.name, "ci workflow");
        assert_eq!(
            workflow.path,
            Path::new("/downstream")
                .join(".github")
                .join("workflows")
                .join("ci.yml")
        );
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
        assert!(text.contains(
            "\n- mutation (since green) (default branch, diff since mutants-green; pure)\n"
        ));
        assert!(text.contains("\n- mutation (full sweep) (weekly + manual, sharded; pure)\n"));
        // the release stanza wears both distinctions: the certification cadence and the
        // registry's one EFFECTFUL capability.
        assert!(text.contains(
            "\n- release (certified tree) (on certification, when the mutants-green tag \
             advances; EFFECTFUL)\n"
        ));
    }
}
