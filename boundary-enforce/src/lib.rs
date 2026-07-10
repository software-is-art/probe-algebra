//! Build-time enforcement of the boundary discipline, over a DERIVED, TOTAL partition —
//! extracted from `probe-algebra`'s build script so ANY crate can attach the same discipline
//! from its own `build.rs`.
//!
//! A boundary is a CATEGORY: value-object OBJECTS and value-operator MORPHISMS
//! (`Morphism` / `Construction` / `Branch` / `Guarded`), with typestates as object INDICES.
//!
//! No source file declares its tier — the partition is COMPUTED from structure, so it is total
//! by construction: a new module is categorized the moment it exists, and nothing can opt out
//! by silence. The derivation (see `derive_partition`): INTERIOR is non-pub reachability from
//! the crate roots; BOUNDARY is being a DOOR — carrying production edge impls, or FRONTING an
//! interior sibling (delegating into a non-reachable module in its own directory: the tier-2
//! relation read backwards); ALGEBRA is the reachable remainder; pure glue (module declarations
//! and re-exports only) takes its reachability's tier. The one tier that is a DECISION rather
//! than evidence is KERNEL, and it is ratified, never inferred. The four tiers, and what each
//! enforces:
//!
//! * **KERNEL** — the trusted floor: it DEFINES and RUNS the format, so it is exempt from the
//!   structural rules — but it is NAMED, not silently skipped. Kernel-hood exempts a file from
//!   every other rule, so it cannot be self-serve or derived from conduct: the file must be on
//!   the consumer's [`Config::kernel_allowlist`], fed from the consumer's OWN `build.rs` — every
//!   new kernel member is a reviewed diff in the consumer's tree (in `probe-algebra`, a
//!   justified line in `spec/kernel.register`). A registered file that no longer exists is a
//!   stale ratification, refused.
//!
//! * **BOUNDARY** — a domain's strict value-object surface: TIER 1. May contain ONLY value
//!   objects, typestates, and value operators — no free functions, global state, submodules,
//!   traits, re-exports (`pub use`), public fields, I/O, or `unsafe` (block, `fn`, or `impl`).
//!   A raw primitive may appear only as the lone field of a newtype wrapper (`Ident(String)`).
//!   Effect detection sees fully-written `std::{io,fs,process,net,thread,env}` paths, names
//!   reached through the file's `use std::…` imports (including renames and groups), the printing
//!   macros, and `std :: <eff>` sequences smuggled inside ANY macro invocation's tokens.
//!
//! * **INTERIOR** — the workshop / leaves: TIER 2. Mutation and raw collections are fine, but the
//!   INWARD rule holds: no function may RETURN a raw primitive — `String`/`&str` or any numeric —
//!   because every domain primitive must be a value object with its own operators. `bool` is
//!   exempt (a predicate is control, not domain data).
//!
//! * **ALGEBRA** — a discovered-law / report layer: exempt from the inward rule (it renders
//!   human-facing reports), but its effects must be HONEST: a function whose body reaches
//!   `std::{fs,io,process,net,env,thread}` must carry a `Capability: <Pure|Lossy|Stateful|Effectful>`
//!   line in its doc, and a `Capability:` line naming none of those is itself a violation.
//!
//! Two rules cut across the non-kernel tiers:
//!
//! * **No rats-nest** (INTERIOR + ALGEBRA): a fully-public top-level `fn` must be operator-shaped
//!   (bare named value types in and out, no effects) — an operator IS attached, to its value
//!   objects. Anything else public is loose plumbing: localize it or hang it off the type it serves.
//! * **Edge-probe completeness** (every file): each concrete, non-test `impl` of
//!   `Morphism`/`Construction`/`Branch`/`Guarded` must have a matching `impl Probed`.
//!
//! Finally, two frozen censuses. The **qualification census**: boundary-hood as a COMPUTED
//! property — every module whose functions form a discoverable algebra is listed, the report
//! frozen to [`Config::qualify_spec`] and drift-gated (regenerate via [`Config::bless_env`],
//! `BLESS_QUALIFY=1` by default). And the **tier lock**: the derived partition itself, one line
//! per file with its evidence, frozen to [`Config::tiers_spec`] — the SAME rows the rule
//! dispatch consumes, so what the lock says and what the rules enforce cannot diverge; a
//! partition move (a file changing tier because its structure changed) is a ratified diff.
//!
//! # Wiring it up (a consumer's `build.rs`)
//!
//! ```no_run
//! use boundary_enforce::{Config, Enforcement};
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
//!     let mut config = Config::new(&manifest);
//!     // The kernel allowlist stays HERE, in your build.rs — a reviewed diff in your own tree.
//!     config.kernel_allowlist = vec!["src/lib.rs".into()];
//!     config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
//!     config.tiers_spec = Some(manifest.join("spec/tiers.spec"));
//!     Enforcement::enforce_or_panic(&config);
//! }
//! ```
//!
//! # Known limitations
//!
//! Holes we know about and have not closed, named here so they are a documented residue rather
//! than a false claim of totality:
//!
//! * Probe/edge matching is by BARE type name (the last path segment), so two same-named edge
//!   types in different modules would share one probe obligation — a cross-module name collision
//!   satisfies the check without covering both edges.
//! * Effect detection is SYNTACTIC and local to a function body: a helper function that does the
//!   I/O and is merely CALLED is invisible (transitive effects are not traced), and only paths
//!   that lexically reach `std::{io,fs,process,net,thread,env}` — directly or via the file's
//!   `use std::…` imports — are seen. (A glob — `use std::fs::*;` — names nothing we can map;
//!   it remains a known gap.)
//! * A type alias (`type Name = String;`) can smuggle a raw primitive past the inward rule and
//!   the boundary field checks, because we match names, not resolved types.
//!
//! The integration tests in `tests/rules.rs` are the crate's executable spec: one fixture per
//! rule, readable as the rule inventory.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{Fields, FnArg, Item, Meta, ReturnType, Signature, Type, Visibility};

/// What to enforce, and where. Construct with [`Config::new`] and override fields as needed —
/// most importantly [`Config::kernel_allowlist`], which MUST live in the consumer's own
/// `build.rs` so every kernel admission is a reviewed diff in the consumer's tree.
pub struct Config {
    /// The source tree to walk (usually `<manifest>/src`).
    pub src_root: PathBuf,
    /// The manifest directory: violation locations are rendered relative to it, so messages
    /// never leak a machine's absolute path and stay stable across checkouts.
    pub manifest_dir: PathBuf,
    /// The RATIFIED kernel — the only files (manifest-relative paths) the partition places
    /// in KERNEL. Kernel-hood exempts a file from every structural rule, so it is never
    /// derived from the file itself: admitting a new member must be a diff in the consumer's
    /// own tree (its build.rs, or a register file that build.rs parses), where review cannot
    /// miss it. An entry naming a file that no longer exists is a violation.
    pub kernel_allowlist: Vec<String>,
    /// Where the qualification census is frozen and drift-gated. `None` skips the census
    /// freeze/drift pass entirely (the census is still computed and returned).
    pub qualify_spec: Option<PathBuf>,
    /// The environment variable that, when set, regenerates [`Config::qualify_spec`] instead of
    /// drift-checking it. Default: `BLESS_QUALIFY`.
    pub bless_env: String,
    /// Where the TIER lock is frozen and drift-gated — the derived partition itself, the
    /// same rows the rule dispatch consumes, so a file changing tier is a ratified diff.
    /// `None` skips the freeze/drift pass (the partition is still computed and returned).
    pub tiers_spec: Option<PathBuf>,
    /// The bless variable for [`Config::tiers_spec`]. Default: `BLESS_TIERS`.
    pub tiers_bless_env: String,
    /// Where the REASON census is frozen and drift-gated — WHY each non-qualifying file
    /// refuses the algebra census (the qualify census's complement, derived by the same
    /// walk). `None` skips the freeze/drift pass (the census is still computed and
    /// returned).
    pub reasons_spec: Option<PathBuf>,
    /// The bless variable for [`Config::reasons_spec`]. Default: `BLESS_REASONS`.
    pub reasons_bless_env: String,
    /// THE BLESS LOOP, DISSOLVED: when a census drifts, regenerate it INTO THE WORKING
    /// TREE and fail the build once — the diff is the ratification (commit it) or the
    /// refusal (revert it); the homework ("go run the bless command") disappears while
    /// the gate's authority is untouched. Default: on everywhere except CI (`CI` env
    /// set), because a fresh checkout must never mutate itself — there, the refusal
    /// only reports. The bless variables remain for explicit regeneration.
    pub autofix_stale: bool,
}

impl Config {
    /// A config for `manifest_dir` with the defaults: `src_root = <manifest>/src`, an EMPTY
    /// kernel allowlist, no qualify spec, and `BLESS_QUALIFY` as the bless variable.
    pub fn new(manifest_dir: impl Into<PathBuf>) -> Config {
        let manifest_dir = manifest_dir.into();
        Config {
            src_root: manifest_dir.join("src"),
            manifest_dir,
            kernel_allowlist: Vec::new(),
            qualify_spec: None,
            bless_env: "BLESS_QUALIFY".to_string(),
            tiers_spec: None,
            tiers_bless_env: "BLESS_TIERS".to_string(),
            reasons_spec: None,
            reasons_bless_env: "BLESS_REASONS".to_string(),
            autofix_stale: std::env::var_os("CI").is_none(),
        }
    }
}

/// The result of running every pass: the (sorted, deduped) violations, the rendered
/// qualification census, and the paths a build script should emit `cargo:rerun-if-changed`
/// lines for. Use [`Enforcement::enforce_or_panic`] from a build.rs for the standard wiring.
pub struct Enforcement {
    /// Every violation, sorted and deduplicated — empty means the discipline holds.
    pub violations: Vec<String>,
    /// The rendered qualification census (what [`Config::qualify_spec`] freezes).
    pub qualify_census: String,
    /// The rendered tier lock (what [`Config::tiers_spec`] freezes): the derived partition,
    /// one line per file with the evidence that placed it.
    pub tiers_census: String,
    /// The rendered REASON census (what [`Config::reasons_spec`] freezes): one line per
    /// non-qualifying file with the mechanical blocker classes its functions exhibit.
    pub reasons_census: String,
    /// The files and directories whose change must re-trigger enforcement, in emission order.
    pub rerun_paths: Vec<PathBuf>,
}

impl Enforcement {
    /// Run ALL passes over `config.src_root` and return the outcome without printing or
    /// panicking. If [`Config::bless_env`] is set in the environment and
    /// [`Config::qualify_spec`] is `Some`, the census is (re)written there instead of
    /// drift-checked — that is the one side effect this method can have.
    pub fn run(config: &Config) -> Enforcement {
        let mut rerun: Vec<PathBuf> = vec![config.src_root.clone()];
        let mut violations = Vec::new();

        // THE PARTITION, derived once and consumed twice: the rule dispatch below and
        // the tier lock both read this map, so what the lock says and what the rules
        // enforce cannot diverge. KERNEL comes only from the ratified allowlist; a
        // registered file that no longer exists is a stale ratification, refused.
        let partition = derive_partition(
            &config.src_root,
            &config.manifest_dir,
            &config.kernel_allowlist,
        );
        for registered in &config.kernel_allowlist {
            if !partition.iter().any(|(_, loc, _, _)| loc == registered) {
                violations.push(format!(
                    "{registered}: registered as KERNEL but no such file exists — a stale \
                     ratification is a lie; delete its register line."
                ));
            }
        }
        let tier_of: HashMap<&str, &'static str> = partition
            .iter()
            .map(|(_, loc, tier, _)| (loc.as_str(), *tier))
            .collect();
        walk(
            &config.src_root,
            &config.manifest_dir,
            &tier_of,
            &mut violations,
            &mut rerun,
        );

        // EDGE-COMPLETENESS, by enumeration: every PRODUCTION boundary edge must carry a probe.
        // This closes the open-world residue an `edges!` list leaves — a new edge impl that no
        // probe can kill is a BUILD ERROR, not a silently-missing line. `cargo-mutants` cannot
        // reach this (it mutates bodies, not the set of impls), so it is the type/build analogue
        // of the grading-law proofs.
        let mut edges: Vec<EdgeImpl> = Vec::new();
        let mut probed: HashSet<String> = HashSet::new();
        collect_edges_and_probes(
            &config.src_root,
            &config.manifest_dir,
            &mut edges,
            &mut probed,
        );
        for e in &edges {
            if !probed.contains(&e.ty) {
                violations.push(format!(
                    "{}: boundary edge `{}` has no `impl Probed` — every production edge must carry a \
                     probe. Add `impl Probed for {}` with its oracle-free probe, or, if it is a \
                     counterexample/fixture rather than a spec edge, gate it behind `#[cfg(test)]`.",
                    e.loc, e.ty, e.ty
                ));
            }
        }

        // QUALIFICATION CENSUS: boundary-hood as a COMPUTED property, not a file convention. For
        // every module, compute whether its functions form a discoverable algebra — operator-shaped
        // (each argument and the return a bare NAMED value type, no raw primitives, no I/O). A
        // module qualifies by STRUCTURE, wherever it lives and whatever it is named; `boundary.rs`
        // is just one place that happens to. The census is frozen into `Config::qualify_spec` and
        // drift-gated, so the answer is ratified in the diff (regenerate with the bless env var).
        let qualify_census =
            render_census(&config.src_root, &config.manifest_dir, &config.bless_env);
        freeze_or_gate(
            &config.qualify_spec,
            &qualify_census,
            &config.bless_env,
            "the algebra-qualification census",
            &config.manifest_dir,
            config.autofix_stale,
            &mut rerun,
            &mut violations,
        );

        // TIER LOCK: the derived partition, frozen. The same rows the dispatch above
        // consumed, rendered one line per file with the evidence that placed it — so a
        // file changing tier (its structure moved it through a door, or out of reach)
        // is a diff a reviewer ratifies, and the lock can never disagree with what was
        // enforced.
        let tiers_census = render_tiers(&partition, &config.tiers_bless_env);
        freeze_or_gate(
            &config.tiers_spec,
            &tiers_census,
            &config.tiers_bless_env,
            "the tier census",
            &config.manifest_dir,
            config.autofix_stale,
            &mut rerun,
            &mut violations,
        );

        // REASON CENSUS: the qualify census's COMPLEMENT — for every file that does NOT
        // qualify, the mechanical blocker classes its functions exhibit (no functions,
        // primitive signatures, borrowed types, effects…). Same walk, same rule,
        // second render: the census is evidence, the reading of it (value-object debt vs
        // missing vocabulary vs principled refusal) stays a ratification.
        let reasons_census = render_reasons(
            &config.src_root,
            &config.manifest_dir,
            &config.reasons_bless_env,
        );
        freeze_or_gate(
            &config.reasons_spec,
            &reasons_census,
            &config.reasons_bless_env,
            "the qualify-reason census",
            &config.manifest_dir,
            config.autofix_stale,
            &mut rerun,
            &mut violations,
        );

        violations.sort();
        violations.dedup();
        Enforcement {
            violations,
            qualify_census,
            tiers_census,
            reasons_census,
            rerun_paths: rerun,
        }
    }

    /// The standard build.rs wiring: run every pass, print the `cargo:rerun-if-changed` lines,
    /// and — if anything is violated — print each violation as a `cargo:warning` and panic with
    /// the violation count, failing the build.
    pub fn enforce_or_panic(config: &Config) -> Enforcement {
        let enforcement = Enforcement::run(config);
        for p in &enforcement.rerun_paths {
            println!("cargo:rerun-if-changed={}", p.display());
        }
        if !enforcement.violations.is_empty() {
            for v in &enforcement.violations {
                println!("cargo:warning={}", v);
            }
            panic!(
                "boundary discipline enforcement failed: {} violation(s) — see warnings above",
                enforcement.violations.len()
            );
        }
        enforcement
    }
}

// ===== qualification census: which modules ARE algebras, by structure ====

/// Compute the algebra-qualification of every module and render the census report.
/// Freeze `census` at `spec_path` (when the bless env is set) or drift-gate it against the
/// committed text — the ONE freeze/drift shape both censuses share, so the two cannot
/// diverge in mechanics. `None` spec path skips entirely.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn freeze_or_gate(
    spec_path: &Option<PathBuf>,
    census: &str,
    bless_env: &str,
    label: &str,
    manifest: &Path,
    autofix: bool,
    rerun: &mut Vec<PathBuf>,
    violations: &mut Vec<String>,
) {
    let Some(spec_path) = spec_path else { return };
    rerun.push(spec_path.clone());
    if std::env::var(bless_env).is_ok() {
        if let Some(parent) = spec_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("create {} ({e})", parent.display()));
        }
        std::fs::write(spec_path, census)
            .unwrap_or_else(|e| panic!("write {} ({e})", spec_path.display()));
    } else {
        let committed = std::fs::read_to_string(spec_path).unwrap_or_default();
        if committed != census {
            let rel = spec_path.strip_prefix(manifest).unwrap_or(spec_path);
            // THE BLESS LOOP, DISSOLVED (the standing question's first catch:
            // regeneration is pure derivation; only committing the diff is the
            // signature). With `autofix` on — the default everywhere except CI — the
            // refusal RUNS its fixing derivation: the regenerated census lands in the
            // working tree and the build still fails ONCE, so the gate's authority is
            // untouched (an unratified drift never builds green) while the homework
            // ("go run the bless command") disappears. Commit the diff to ratify, or
            // revert it to refuse. In CI there is no tree to ratify into, so the
            // refusal only reports — a fresh checkout must never mutate itself.
            if autofix {
                if let Some(parent) = spec_path.parent() {
                    std::fs::create_dir_all(parent)
                        .unwrap_or_else(|e| panic!("create {} ({e})", parent.display()));
                }
                std::fs::write(spec_path, census)
                    .unwrap_or_else(|e| panic!("write {} ({e})", spec_path.display()));
                violations.push(format!(
                    "{} was stale — {label} drifted, and the regenerated text is now IN \
                     YOUR WORKING TREE. The diff is the ratification: commit it, or \
                     revert it to refuse the drift. (This build fails once so the \
                     movement is never silent.)",
                    rel.display(),
                ));
            } else {
                violations.push(format!(
                    "{} is stale — {label} drifted. Regenerate with `{bless_env}=1 cargo \
                     build` and ratify the diff.",
                    rel.display(),
                ));
            }
        }
    }
}

/// One file's row in the derived partition.
struct TierRow {
    loc: String,
    /// Does the file carry anything beyond module declarations and re-exports? Pure
    /// glue has no evidence to judge, so its declared tier stands.
    substance: bool,
    /// Child modules this file declares: `(name, is_pub)`.
    mods: Vec<(String, bool)>,
    /// The directory this file's child modules resolve under.
    child_dir: PathBuf,
    /// The source text, kept for the fronting check (does this file delegate into an
    /// interior sibling?).
    source: String,
}

/// The DERIVED partition: every file's tier, computed — the single source both the rule
/// dispatch and the tier lock consume. INTERIOR is non-pub reachability from the crate
/// roots; BOUNDARY is being a DOOR (production edge impls, or fronting an interior
/// sibling — the tier-2 relation read backwards); ALGEBRA is the reachable remainder;
/// pure glue takes its reachability's tier (its content gives the rules nothing to
/// judge either way). KERNEL is never derived: it comes from the ratified allowlist the
/// consumer's build.rs feeds (in this workspace, parsed from `spec/kernel.register`,
/// where every exemption carries a justification).
fn derive_partition(
    src: &Path,
    manifest: &Path,
    kernel_allowlist: &[String],
) -> Vec<(PathBuf, String, &'static str, String)> {
    let mut rows: Vec<(PathBuf, TierRow)> = Vec::new();
    collect_tier_rows(src, manifest, &mut rows);

    // pub-reachability, from the crate roots: a file is PUB-REACHABLE when a chain of
    // `pub mod` declarations connects a root (lib.rs / main.rs) to it. Roots count as
    // reachable themselves.
    let mut reachable: Vec<PathBuf> = rows
        .iter()
        .map(|(p, _)| p.clone())
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            (name == "lib.rs" || name == "main.rs") && p.parent() == Some(src)
        })
        .collect();
    let mut frontier = reachable.clone();
    while let Some(path) = frontier.pop() {
        let Some((_, row)) = rows.iter().find(|(p, _)| p == &path) else {
            continue;
        };
        for (name, is_pub) in &row.mods {
            if !is_pub {
                continue;
            }
            for candidate in [
                row.child_dir.join(format!("{name}.rs")),
                row.child_dir.join(name).join("mod.rs"),
            ] {
                if rows.iter().any(|(p, _)| p == &candidate) && !reachable.contains(&candidate) {
                    reachable.push(candidate.clone());
                    frontier.push(candidate);
                }
            }
        }
    }

    // the BOUNDARY evidence, half one: production edge impls (Morphism/Construction/
    // Branch/Guarded, non-generic, non-test) — the same collector the probe-completeness
    // pass runs. Operator-shape stays the QUALIFY census's fact: a discoverable algebra
    // wherever it lives is not a tier claim.
    let mut edges: Vec<EdgeImpl> = Vec::new();
    let mut probed: HashSet<String> = HashSet::new();
    collect_edges_and_probes(src, manifest, &mut edges, &mut probed);
    let edge_locs: HashSet<&str> = edges.iter().map(|e| e.loc.as_str()).collect();

    // half two, the FRONTING relation — the tier system's own semantics as evidence:
    // INTERIOR is not merely unreachable, it is fronted, and the front is the file that
    // delegates into it. A pub-reachable file that references an interior sibling (a
    // non-pub-reachable module in its own directory) by path is a door. Doors are
    // boundaries.
    let interior_stems: Vec<(PathBuf, String)> = rows
        .iter()
        .filter(|(p, _)| !reachable.contains(p))
        .filter_map(|(p, _)| {
            Some((
                p.parent()?.to_path_buf(),
                p.file_stem()?.to_str()?.to_string(),
            ))
        })
        .collect();

    rows.into_iter()
        .map(|(path, row)| {
            if kernel_allowlist.iter().any(|k| k == &row.loc) {
                return (
                    path,
                    row.loc,
                    "KERNEL",
                    "registered — a decision, never derived".to_string(),
                );
            }
            let is_reachable = reachable.contains(&path);
            let carries_edges = edge_locs.contains(row.loc.as_str());
            let fronts = interior_stems.iter().any(|(dir, stem)| {
                Some(dir.as_path()) == path.parent() && row.source.contains(&format!("{stem}::"))
            });
            let (tier, evidence): (&'static str, String) = if !row.substance {
                let t = if is_reachable { "ALGEBRA" } else { "INTERIOR" };
                (
                    t,
                    "glue — module declarations and re-exports only; tier by reachability"
                        .to_string(),
                )
            } else if !is_reachable {
                ("INTERIOR", "not pub-reachable".to_string())
            } else if carries_edges {
                (
                    "BOUNDARY",
                    "pub-reachable, carries production edges".to_string(),
                )
            } else if fronts {
                (
                    "BOUNDARY",
                    "pub-reachable, fronts an interior sibling".to_string(),
                )
            } else {
                (
                    "ALGEBRA",
                    "pub-reachable, no production edges, fronts nothing".to_string(),
                )
            };
            (path, row.loc, tier, evidence)
        })
        .collect()
}

/// The tier lock's render: the derived partition, one line per file. What
/// [`Config::tiers_spec`] freezes — THE partition, not a coherence report; there is no
/// declared column left to disagree with.
/// The one-line meaning of each tier's membership — what belonging to it forbids or exempts.
/// Canonical HERE, in the enforcer that decides those rules, and rendered into the tier lock
/// (`render_tiers`) so downstream readers — the edit hook above all — recite it from the
/// committed text instead of carrying a copy that silently drifts from what is enforced.
fn tier_rule(tier: &str) -> &'static str {
    match tier {
        "KERNEL" => "the trusted floor — exempt from the structural rules; a ratified privilege",
        "BOUNDARY" => "tier 1 — a domain's strict value-object surface; no loose `pub fn`",
        "INTERIOR" => {
            "tier 2 — the workshop; mutation and raw collections allowed; no loose `pub fn`"
        }
        "ALGEBRA" => {
            "the discovered-law / report layer; exempt from the inward rule; no loose `pub fn`"
        }
        _ => "see boundary-enforce for this tier's rules",
    }
}

fn render_tiers(partition: &[(PathBuf, String, &'static str, String)], bless_env: &str) -> String {
    let count = |t: &str| {
        partition
            .iter()
            .filter(|(_, _, tier, _)| *tier == t)
            .count()
    };
    let mut report = format!(
        "# the tier partition, DERIVED — the single source the rule dispatch and this lock\n\
         # both consume. INTERIOR is non-pub reachability; BOUNDARY is being a DOOR\n\
         # (production edge impls, or fronting an interior sibling — the tier-2 relation\n\
         # read backwards); ALGEBRA is the reachable remainder; glue takes its\n\
         # reachability's tier. KERNEL is a decision, never derived: it is ratified in the\n\
         # consumer's own tree (the build.rs allowlist, or a register it parses).\n\
         # Regenerate with `{bless_env}=1 cargo build`.\n",
    );
    report.push_str(&format!(
        "# {} files: {} boundary, {} interior, {} algebra, {} kernel.\n",
        partition.len(),
        count("BOUNDARY"),
        count("INTERIOR"),
        count("ALGEBRA"),
        count("KERNEL"),
    ));
    // the RULES LEGEND: one line per tier saying what its membership means. Rendered here —
    // the enforcer owns what a tier forbids — so every reader recites it from the committed
    // text, never from its own compiled copy. The edit hook (`probe-hook::tier_voice`) reads
    // exactly these `# rule <TIER>:` lines, so a lock regenerated by a newer enforcer updates
    // what the hook says with no new binary.
    for tier in ["KERNEL", "BOUNDARY", "INTERIOR", "ALGEBRA"] {
        report.push_str(&format!("# rule {tier}: {}\n", tier_rule(tier)));
    }
    report.push('\n');
    let mut lines: Vec<String> = partition
        .iter()
        .map(|(_, loc, tier, evidence)| format!("- {loc}: {tier} ({evidence})"))
        .collect();
    lines.sort();
    for l in &lines {
        report.push_str(l);
        report.push('\n');
    }
    report
}

/// Walk `dir` collecting each file's declared tier, qualify evidence, and child-module
/// declarations (for the reachability pass).
fn collect_tier_rows(dir: &Path, manifest: &Path, rows: &mut Vec<(PathBuf, TierRow)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_tier_rows(&path, manifest, rows);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(file) = parse(&path) else { continue };
        let substance = file
            .items
            .iter()
            .any(|it| !matches!(it, Item::Mod(_) | Item::Use(_)));
        let mods = file
            .items
            .iter()
            .filter_map(|it| match it {
                Item::Mod(m) if m.content.is_none() => Some((
                    m.ident.to_string(),
                    matches!(m.vis, syn::Visibility::Public(_)),
                )),
                _ => None,
            })
            .collect();
        // lib.rs / main.rs / mod.rs resolve children in their own directory; a plain
        // `foo.rs` resolves them under `foo/`.
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let child_dir = if matches!(name, "lib.rs" | "main.rs" | "mod.rs") {
            path.parent().unwrap_or(dir).to_path_buf()
        } else {
            path.with_extension("")
        };
        rows.push((
            path.clone(),
            TierRow {
                loc: path
                    .strip_prefix(manifest)
                    .unwrap_or(&path)
                    .display()
                    .to_string(),
                substance,
                mods,
                child_dir,
                source,
            },
        ));
    }
}

fn render_census(src: &Path, manifest: &Path, bless_env: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    collect_qualifications(src, manifest, &mut scanned, &mut lines);
    lines.sort();

    let mut report = format!(
        "# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are\n\
         # operator-shaped (every argument and the return a bare named value type, no primitives, no\n\
         # I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a\n\
         # module qualifies wherever it lives. Regenerate with `{bless_env}=1 cargo build`.\n",
    );
    report.push_str(&format!(
        "# {} files scanned, {} qualify.\n\n",
        scanned,
        lines.len()
    ));
    for l in &lines {
        report.push_str(l);
        report.push('\n');
    }
    report
}

/// Walk `dir`, parsing each `.rs` file, and push a `QUALIFIES` line for every file whose functions
/// form an algebra.
fn collect_qualifications(dir: &Path, manifest: &Path, scanned: &mut usize, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_qualifications(&path, manifest, scanned, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // `tests.rs` is the crate's `#[cfg(test)] mod tests` body — test scaffolding, not a domain.
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        let Ok(file) = parse(&path) else { continue };
        *scanned += 1;
        let loc = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        if let Some(line) = qualify_line_from_items(&file.items, &loc) {
            out.push(line);
        }
    }
}

/// The qualify census line one file's items contribute, or `None` if its functions do not form an
/// algebra — the EXACT per-file emitter [`collect_qualifications`] walks with, factored to ONE
/// source so the operator-shape rule is stated once and served both to the frozen census and to
/// any caller that needs the same line from a file's items. `loc` is the path as it appears in
/// the census line.
fn qualify_line_from_items(items: &[Item], loc: &str) -> Option<String> {
    let imports = std_effect_imports(items);
    let mut ops: Vec<Op> = Vec::new();
    qualify_items(items, &imports, &mut ops);
    if ops.is_empty() {
        return None;
    }
    let mut names: Vec<&str> = ops.iter().map(|o| o.name.as_str()).collect();
    names.sort_unstable();
    let mut sorts: Vec<&str> = ops
        .iter()
        .flat_map(|o| o.args.iter().chain(std::iter::once(&o.ret)))
        .map(|s| s.as_str())
        .collect();
    sorts.sort_unstable();
    sorts.dedup();
    Some(format!(
        "{loc}: QUALIFIES — operators [{}] over sorts {{{}}}",
        names.join(", "),
        sorts.join(", ")
    ))
}

/// The qualify census line a file's SOURCE TEXT would contribute — the edit-time entry point.
///
/// This is [`collect_qualifications`]'s per-file emitter reached from text rather than a walked
/// path, so an editor hook can compute exactly the line the frozen census would carry for a file
/// it just changed and narrate the movement BEFORE the build — one vocabulary with the census, the
/// operator-shape rule never restated. `loc` is the path as it should appear in the line
/// (repo-relative). The three outcomes are distinct on purpose: `Ok(Some(line))` qualifies,
/// `Ok(None)` parses but forms no algebra (a real "does not / no longer qualifies"), and `Err`
/// is a file that does not parse — a half-written edit, which a caller must treat as "no signal
/// yet", never as a qualification change.
pub fn qualify_line(source: &str, loc: &str) -> Result<Option<String>, String> {
    let file = syn::parse_file(source).map_err(|e| format!("unparseable: {e}"))?;
    Ok(qualify_line_from_items(&file.items, loc))
}

/// The live qualify census BODY — every `QUALIFIES` line a source tree currently produces, sorted,
/// without the header/count comments. This is exactly what [`render_census`] walks (same tree, same
/// `tests.rs` skip, same per-file emitter), minus the prose, so the edit-time drift ledger can hold
/// it against the committed `spec/qualify.spec` and show every currently-stale line at once —
/// accumulating as files drift, empty the moment a re-bless reconciles them. `manifest` is the
/// prefix stripped to form each line's path (so lines read `src/…` as the committed census does).
pub fn qualify_census_lines(src: &Path, manifest: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let mut scanned = 0usize;
    collect_qualifications(src, manifest, &mut scanned, &mut lines);
    lines.sort();
    lines
}

// ===== the REASON census: why each non-qualifying module refuses =====

/// Render the qualify-REASON census: one line per file whose functions form no algebra,
/// carrying the mechanical blocker classes they exhibit — the qualify census's complement,
/// derived by the same walk (same `tests.rs` skip, same operator-shape rule, stated once).
/// The classes are EVIDENCE, deliberately mechanical; reading them into value-object debt,
/// missing vocabulary, or a principled refusal is a ratification, not a derivation.
fn render_reasons(src: &Path, manifest: &Path, bless_env: &str) -> String {
    let mut scanned = 0usize;
    let mut lines: Vec<String> = Vec::new();
    collect_reasons(src, manifest, &mut scanned, &mut lines);
    lines.sort();
    let mut report = format!(
        "# qualify reasons — WHY each module refuses the algebra census: the mechanical blockers,\n\
         # per file, derived by the same walk as the qualify census (one rule, two renders; free\n\
         # functions AND impl methods, the receiver resolved to the typestate). Classes:\n\
         # no functions | unit returns | primitive signatures | borrowed types | parameterised\n\
         # types | unshaped types | zero-argument constants | effectful bodies | mutating\n\
         # receivers. The classes are evidence; reading them into value-object debt, missing\n\
         # vocabulary, or a principled refusal is the ratification's job. Regenerate with\n\
         # `{bless_env}=1 cargo build`.\n",
    );
    report.push_str(&format!(
        "# {} files scanned, {} qualify, {} refuse.\n\n",
        scanned,
        scanned - lines.len(),
        lines.len()
    ));
    for l in &lines {
        report.push_str(l);
        report.push('\n');
    }
    report
}

/// Walk `dir` and push a `REFUSES` line for every parsed file whose functions form no
/// algebra — the same walk (and skips) as [`collect_qualifications`].
fn collect_reasons(dir: &Path, manifest: &Path, scanned: &mut usize, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_reasons(&path, manifest, scanned, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("tests.rs") {
            continue;
        }
        let Ok(file) = parse(&path) else { continue };
        *scanned += 1;
        let classes = refusal_classes_from_items(&file.items);
        if classes.is_empty() {
            continue;
        }
        let loc = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        out.push(format!("{loc}: REFUSES — {}", classes.join(", ")));
    }
}

/// The blocker classes one file's items exhibit — EMPTY exactly when the file qualifies
/// (some function is operator-shaped, the same predicate [`qualify_line_from_items`]
/// answers), so the two censuses partition the scanned files by construction. Free
/// functions and impl methods are judged alike (the receiver resolved to the typestate —
/// the "impl-attached surface only" blind spot this census exposed on its first minting is
/// dissolved by the walk now SEEING impls); a file with no functions at all is its own
/// class; otherwise the classes aggregate every blocker across the functions, sorted.
fn refusal_classes_from_items(items: &[Item]) -> Vec<&'static str> {
    let imports = std_effect_imports(items);
    let mut ops: Vec<Op> = Vec::new();
    qualify_items(items, &imports, &mut ops);
    if !ops.is_empty() {
        return Vec::new();
    }
    let mut fns: Vec<FnRow> = Vec::new();
    collect_fns(items, &mut fns);
    if fns.is_empty() {
        return vec!["no functions"];
    }
    let mut classes: BTreeSet<&'static str> = BTreeSet::new();
    for row in fns {
        // the same admission `operator_candidate` makes, read for its refusal class: `Self`
        // resolves to the typestate (or reads as its target's blocker), a type PARAMETER is
        // generic machinery, and everything else classifies by structure.
        let class_of = |ty: &Type| -> Option<&'static str> {
            if let Some(n) = named_value_type(ty) {
                if n == "Self" {
                    return match &row.receiver {
                        Some(Some(_)) => None,
                        Some(None) => Some("parameterised types"),
                        None => Some("unshaped types"),
                    };
                }
                if row.type_params.contains(&n) {
                    return Some("parameterised types");
                }
                return None;
            }
            type_class(ty)
        };
        match &row.sig.output {
            ReturnType::Default => {
                classes.insert("unit returns");
            }
            ReturnType::Type(_, ty) => {
                if let Some(class) = class_of(ty) {
                    classes.insert(class);
                }
            }
        }
        if row.sig.inputs.is_empty() {
            classes.insert("zero-argument constants");
        }
        for arg in &row.sig.inputs {
            match arg {
                FnArg::Receiver(r) => {
                    if r.reference.is_some() && r.mutability.is_some() {
                        classes.insert("mutating receivers");
                    } else if r.colon_token.is_some() {
                        classes.insert("unshaped types");
                    } else if row.receiver.as_ref().is_some_and(|target| target.is_none()) {
                        // `self` on a generic/exotic impl target: the receiver's own type
                        // is not a bare named value type.
                        classes.insert("parameterised types");
                    }
                }
                FnArg::Typed(pt) => {
                    if let Some(class) = class_of(&pt.ty) {
                        classes.insert(class);
                    }
                }
            }
        }
        let mut eff = EffectFinder {
            found: None,
            imports: &imports,
        };
        eff.visit_block(row.body);
        if eff.found.is_some() {
            classes.insert("effectful bodies");
        }
    }
    classes.into_iter().collect()
}

/// One function the qualify walk judges, with the context its types resolve in: the receiver
/// is `None` for a free function, `Some(target)` for a method (where `target` is the impl's
/// bare named value type, or `None` when the impl target is generic/exotic — a receiver whose
/// type is not a sort), and `type_params` are the in-scope type variables (impl + fn).
struct FnRow<'a> {
    sig: &'a syn::Signature,
    receiver: Option<Option<String>>,
    type_params: BTreeSet<String>,
    body: &'a syn::Block,
}

/// Every function the qualify walk judges — free functions and impl methods, descending into
/// non-test inline modules: the same traversal as [`qualify_items`], as [`FnRow`]s.
fn collect_fns<'a>(items: &'a [Item], out: &mut Vec<FnRow<'a>>) {
    for it in items {
        match it {
            Item::Fn(f) if !is_cfg_test(&f.attrs) => out.push(FnRow {
                sig: &f.sig,
                receiver: None,
                type_params: type_params(&f.sig.generics),
                body: &f.block,
            }),
            Item::Impl(i) if !is_cfg_test(&i.attrs) => {
                let target = named_value_type(&i.self_ty);
                let impl_params = type_params(&i.generics);
                for item in &i.items {
                    if let syn::ImplItem::Fn(m) = item {
                        if !is_cfg_test(&m.attrs) {
                            let mut params = impl_params.clone();
                            params.extend(type_params(&m.sig.generics));
                            out.push(FnRow {
                                sig: &m.sig,
                                receiver: Some(target.clone()),
                                type_params: params,
                                body: &m.block,
                            });
                        }
                    }
                }
            }
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    collect_fns(inner, out);
                }
            }
            _ => {}
        }
    }
}

/// Why a type is not a bare named value type — `None` when it IS one (the same admission
/// [`named_value_type`] makes, so the classifier and the rule cannot disagree about a type).
fn type_class(ty: &Type) -> Option<&'static str> {
    if named_value_type(ty).is_some() {
        return None;
    }
    match ty {
        Type::Path(tp) => match tp.path.segments.last() {
            Some(seg) if !matches!(seg.arguments, syn::PathArguments::None) => {
                Some("parameterised types")
            }
            Some(_) => Some("primitive signatures"),
            None => Some("unshaped types"),
        },
        Type::Reference(_) => Some("borrowed types"),
        Type::Tuple(t) if t.elems.is_empty() => Some("unit returns"),
        _ => Some("unshaped types"),
    }
}

/// An operator read off a function: name, argument sorts, return sort.
struct Op {
    name: String,
    args: Vec<String>,
    ret: String,
}

/// Collect operator functions from items, descending into non-test inline modules.
fn qualify_items(items: &[Item], imports: &HashMap<String, String>, out: &mut Vec<Op>) {
    for it in items {
        match it {
            Item::Fn(f) if !is_cfg_test(&f.attrs) => {
                let params = type_params(&f.sig.generics);
                if let Some(op) = operator_candidate(&f.sig, None, &params, &f.block, imports) {
                    out.push(op);
                }
            }
            // ASSOCIATED FUNCTIONS ARE OPERATORS (the spike's first brick): the no-rats-nest
            // rule pushes every public callable onto a typestate, so a census that read only
            // free functions manufactured its own largest blind spot ("impl-attached surface
            // only", 15 files on the day the reason census minted). A method is judged by the
            // same one rule with `self`/`&self`/`Self` resolved to the impl target — Rust's
            // calling convention for a value-object operator, not a shape difference. The
            // impl target must itself be a bare named value type (a generic target is not a
            // sort), and methods key `Type::method`, the sixth sense's identity convention.
            Item::Impl(im) if !is_cfg_test(&im.attrs) => {
                let target = named_value_type(&im.self_ty);
                let impl_params = type_params(&im.generics);
                for item in &im.items {
                    if let syn::ImplItem::Fn(m) = item {
                        if is_cfg_test(&m.attrs) {
                            continue;
                        }
                        let mut params = impl_params.clone();
                        params.extend(type_params(&m.sig.generics));
                        if let Some(op) = operator_candidate(
                            &m.sig,
                            target.as_deref(),
                            &params,
                            &m.block,
                            imports,
                        ) {
                            out.push(op);
                        }
                    }
                }
            }
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    qualify_items(inner, imports, out);
                }
            }
            _ => {}
        }
    }
}

/// Is this signature an operator over value objects? Every argument and the return must be a BARE
/// NAMED value type (a path with no generics, not a raw primitive or `bool`, not a type PARAMETER —
/// a variable is not a sort), and the body must do no I/O — the shape `#[algebra]` reads. A
/// `&[Value]`-style evaluator, a primitive return, or an effect is not. `receiver` is the impl
/// target for a method: `self`, `&self`, and `Self` in the signature all resolve to it (calling
/// convention and spelling, not shape — the borrow of the carrier is how Rust spells "operator on
/// a value object"); `&mut self` is mutation, refused; an explicitly-typed receiver
/// (`self: Box<Self>`) is not bare, refused. Method operators key `Type::method` — the sixth
/// sense's identity convention, and what keeps two typestates' `new`s distinct in the census.
fn operator_candidate(
    sig: &syn::Signature,
    receiver: Option<&str>,
    type_params: &BTreeSet<String>,
    body: &syn::Block,
    imports: &HashMap<String, String>,
) -> Option<Op> {
    let sort_of = |ty: &Type| -> Option<String> {
        let n = named_value_type(ty)?;
        if n == "Self" {
            return receiver.map(str::to_string);
        }
        (!type_params.contains(&n)).then_some(n)
    };
    let ret = match &sig.output {
        ReturnType::Type(_, ty) => sort_of(ty)?,
        ReturnType::Default => return None,
    };
    let mut args = Vec::new();
    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(r) => {
                if r.colon_token.is_some() || (r.reference.is_some() && r.mutability.is_some()) {
                    return None;
                }
                args.push(receiver?.to_string());
            }
            FnArg::Typed(pt) => args.push(sort_of(&pt.ty)?),
        }
    }
    if args.is_empty() {
        // an arity-0 constant only counts toward an algebra alongside real operators; on its own a
        // bare `fn() -> T` is not enough signal, so require at least one argument here.
        return None;
    }
    let mut eff = EffectFinder {
        found: None,
        imports,
    };
    eff.visit_block(body);
    if eff.found.is_some() {
        return None;
    }
    Some(Op {
        name: match receiver {
            Some(target) => format!("{target}::{}", sig.ident),
            None => sig.ident.to_string(),
        },
        args,
        ret,
    })
}

/// The in-scope type-parameter names of a generics clause — the variables an operator's sorts
/// must not be.
fn type_params(generics: &syn::Generics) -> BTreeSet<String> {
    generics
        .type_params()
        .map(|p| p.ident.to_string())
        .collect()
}

/// A bare named value type — a path with no generic arguments that is not a raw primitive or `bool`.
/// `Tri`/`Date` qualify; `i64`, `bool`, `Option<T>`, `&[T]`, `&str` do not.
fn named_value_type(ty: &Type) -> Option<String> {
    let Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    let n = seg.ident.to_string();
    if RAW_PRIMITIVES.contains(&n.as_str()) || n == "bool" {
        return None;
    }
    Some(n)
}

fn walk(
    dir: &Path,
    manifest: &Path,
    tier_of: &HashMap<&str, &'static str>,
    out: &mut Vec<String>,
    rerun: &mut Vec<PathBuf>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Register the DIRECTORY too, not just the files in it — a brand-new file in a
            // subdirectory changes the directory entry, so it must re-trigger the build script;
            // watching only the files that existed at the last build would let a new module land
            // unchecked until something else happened to dirty the build.
            rerun.push(path.clone());
            walk(&path, manifest, tier_of, out, rerun);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        rerun.push(path.clone());

        let loc = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        let source = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                out.push(format!("{loc}: cannot read ({e})"));
                continue;
            }
        };

        // THE PARTITION, total by DERIVATION: every file's tier comes from the derived
        // map (reachability, doors, glue — see `derive_partition`), so a new module is
        // categorized the moment it exists; nothing is declared and nothing can be
        // forgotten. KERNEL appears in the map only via the ratified allowlist.
        let Some(tier) = tier_of.get(loc.as_str()).copied() else {
            // unparseable files never enter the partition; the parse pass reports them.
            continue;
        };

        // dispatch the STRUCTURAL discipline on the derived tier:
        //   KERNEL   — the trusted floor (the grammar, the engine, the macros, crate tooling):
        //              defines/runs the format, so it is exempt — but NAMED, not silently skipped.
        //   BOUNDARY — a domain's strict value-object surface: the tier-1 grammar.
        //   INTERIOR — the workshop / leaves: the tier-2 inward rule (no raw primitive escapes).
        //   ALGEBRA  — a discovered-law / report layer (`theory!` domains + the discover meta): it
        //              renders human-facing reports (counts, prose, observations), so it is exempt
        //              from the inward rule. But its effects must be HONEST: a function that touches
        //              the world (`std::fs`/`io`/`process`/`net`/`env`/`thread`) must DECLARE a
        //              capability — the fine seam/capability/leaf split, made a build obligation.
        match tier {
            "KERNEL" => {
                // exemption from every rule below — reached only through the ratified
                // allowlist (fed, in this workspace, from spec/kernel.register, where
                // every entry carries a justification). Named, never silently skipped.
                continue;
            }
            "BOUNDARY" | "INTERIOR" | "ALGEBRA" => {}
            _ => continue,
        }

        let file = match syn::parse_file(&source) {
            Ok(f) => f,
            Err(e) => {
                out.push(format!("{loc}: parse error ({e})"));
                continue;
            }
        };
        match tier {
            "BOUNDARY" => check_boundary(&loc, &file, out), // tier 1
            "INTERIOR" => check_internal(&loc, &file, out), // tier 2
            "ALGEBRA" => check_algebra(&loc, &file, out),   // the capability-honesty rule
            _ => {}
        }
        // the NO-RATS-NEST rule, on every non-kernel tier: a fully-public function must be
        // ATTACHED — a method/associated fn on a typestate (impls are untouched here) — or be
        // OPERATOR-SHAPED (bare named value types in and out, the same shape `#[algebra]` and
        // the qualify census read: an operator IS attached, to its value objects). Anything
        // else public is loose plumbing: localize it (`fn` / `pub(super)` / `pub(crate)` are
        // free — "local defs"), or hang it off the type it serves. BOUNDARY already bans all
        // top-level fns (stricter), so this adds teeth to INTERIOR and ALGEBRA.
        if matches!(tier, "INTERIOR" | "ALGEBRA") {
            check_loose_pub_fns(&loc, &file, out);
        }
    }
}

/// Flag fully-public top-level functions that are neither operator-shaped nor localized —
/// the rats-nest rule. Recurses into non-`cfg(test)` inline modules (an `#[algebra]` module's
/// operators are inside one), skipping test scaffolding like every other pass.
fn check_loose_pub_fns(loc: &str, file: &syn::File, out: &mut Vec<String>) {
    let imports = std_effect_imports(&file.items);
    fn go(
        loc: &str,
        items: &[Item],
        imports: &HashMap<String, String>,
        in_algebra: bool,
        out: &mut Vec<String>,
    ) {
        for item in items {
            match item {
                Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                    if let Some((_, inner)) = &m.content {
                        // an `#[algebra]` module's public fns ARE its operator table — inside
                        // one, an arity-0 fn returning a value type is a CONSTANT operator
                        // (the identity/annihilator laws need it), not loose plumbing.
                        let algebra = in_algebra
                            || m.attrs.iter().any(|a| {
                                a.path()
                                    .segments
                                    .last()
                                    .is_some_and(|s| s.ident == "algebra")
                            });
                        go(loc, inner, imports, algebra, out);
                    }
                }
                Item::Fn(f)
                    if !is_cfg_test(&f.attrs) && matches!(f.vis, syn::Visibility::Public(_)) =>
                {
                    let constant_operator = in_algebra
                        && f.sig.inputs.is_empty()
                        && match &f.sig.output {
                            syn::ReturnType::Type(_, ty) => named_value_type(ty).is_some(),
                            syn::ReturnType::Default => false,
                        };
                    if !constant_operator
                        && operator_candidate(
                            &f.sig,
                            None,
                            &type_params(&f.sig.generics),
                            &f.block,
                            imports,
                        )
                        .is_none()
                    {
                        out.push(format!(
                            "{loc}: `pub fn {}` is a LOOSE public function — neither attached to \
                             a typestate nor operator-shaped. Make it a method/associated fn on \
                             the value object it serves, give it the operator shape (bare named \
                             value types in and out), or localize it (`pub(super)`/`pub(crate)`).",
                            f.sig.ident
                        ));
                    }
                }
                _ => {}
            }
        }
    }
    go(loc, &file.items, &imports, false, &mut *out);
}

fn parse(path: &Path) -> Result<syn::File, String> {
    let source = std::fs::read_to_string(path).map_err(|e| format!("cannot read ({e})"))?;
    syn::parse_file(&source).map_err(|e| format!("parse error ({e})"))
}

// ===== edge-completeness: enumerate edges, require a probe for each =======

/// A concrete (non-generic, non-test) boundary edge impl: the type it is for, and where.
struct EdgeImpl {
    ty: String,
    loc: String,
}

/// The edge marker traits — an `impl` of any of these (for a concrete type) is a boundary edge.
const EDGE_TRAITS: &[&str] = &["Morphism", "Construction", "Branch", "Guarded"];

/// Walk EVERY `.rs` under the source root (including the crate-root files `walk` exempts, since
/// the `Probed` impls may live in kernel files), collecting concrete production edges and the set
/// of types that carry an `impl Probed`.
fn collect_edges_and_probes(
    dir: &Path,
    manifest: &Path,
    edges: &mut Vec<EdgeImpl>,
    probed: &mut HashSet<String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_edges_and_probes(&path, manifest, edges, probed);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // manifest-relative, like every other pass — a violation message must not leak the
        // machine's absolute path, and the sorted/deduped output must be stable across checkouts.
        let loc = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        if let Ok(file) = parse(&path) {
            collect_items(&file.items, &loc, edges, probed);
        }
    }
}

/// Recurse over items (descending into inline modules, where the `Probed` impls live), recording
/// concrete edge impls and `Probed` impls.
fn collect_items(
    items: &[Item],
    loc: &str,
    edges: &mut Vec<EdgeImpl>,
    probed: &mut HashSet<String>,
) {
    for item in items {
        match item {
            // a `#[cfg(test)]` module is test scaffolding, not a production edge — skip it whole,
            // matching `qualify_items` and `check_algebra_items`; checking only the impl's own
            // attrs would let a test-module edge masquerade as production.
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    collect_items(inner, loc, edges, probed);
                }
            }
            Item::Impl(im) => {
                let Some((_, path, _)) = &im.trait_ else {
                    continue;
                };
                let Some(trait_name) = path.segments.last().map(|s| s.ident.to_string()) else {
                    continue;
                };
                let Some(ty) = self_type_name(&im.self_ty) else {
                    continue;
                };
                if trait_name == "Probed" {
                    probed.insert(ty);
                } else if EDGE_TRAITS.contains(&trait_name.as_str())
                    // skip GENERIC combinators (`Compose`, `Then`, `Profiled`, …) — they are
                    // parametric, probed through the leaves they compose, not in their own right.
                    && im.generics.type_params().next().is_none()
                    // skip `#[cfg(test)]` impls — counterexamples/fixtures are not spec edges.
                    && !is_cfg_test(&im.attrs)
                {
                    edges.push(EdgeImpl {
                        ty,
                        loc: loc.to_string(),
                    });
                }
            }
            _ => {}
        }
    }
}

/// The bare type name an `impl ... for T` is for (the last path segment of `T`), or `None` for a
/// non-path self type.
fn self_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

/// Whether any attribute is `#[cfg(test)]`.
fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        matches!(&a.meta, Meta::List(l) if l.path.is_ident("cfg") && l.tokens.to_string().contains("test"))
    })
}

// ===== tier 1: the strict boundary grammar ===============================

fn check_boundary(loc: &str, file: &syn::File, out: &mut Vec<String>) {
    for item in &file.items {
        match item {
            // A boundary may IMPORT privately (to implement its citizens) but may
            // not RE-EXPORT: `pub use` forwards another module's citizen as if it
            // were this boundary's own, defeating "one place to look" and letting
            // a parent's surface silently become its whole subtree. A parent must
            // instead DEFINE a narrower citizen that delegates inward.
            Item::Use(u) => {
                if !matches!(u.vis, Visibility::Inherited) {
                    out.push(format!(
                        "{loc}: re-export (`pub use`) — a boundary must DEFINE its citizens, \
                         not forward another module's; collapse the child surface into an \
                         operator or value object owned here"
                    ));
                }
            }
            Item::Impl(_) | Item::Macro(_) | Item::Type(_) | Item::Const(_) => {}
            Item::Struct(s) => check_fields(loc, &s.ident.to_string(), &s.fields, out),
            Item::Enum(e) => {
                for v in &e.variants {
                    check_fields(loc, &format!("{}::{}", e.ident, v.ident), &v.fields, out);
                }
            }
            Item::Fn(f) => out.push(format!(
                "{loc}: free function `{}` — value operators must be types implementing \
                 ValueOperator; put pure helpers in a private module",
                f.sig.ident
            )),
            Item::Static(s) => out.push(format!(
                "{loc}: `static {}` — a boundary may not hold global state",
                s.ident
            )),
            Item::Mod(m) => out.push(format!(
                "{loc}: submodule `{}` — a boundary is flat (move it to a private sibling)",
                m.ident
            )),
            Item::Trait(t) => out.push(format!(
                "{loc}: trait `{}` — traits belong in the grammar (crate::boundary), \
                 not a domain boundary",
                t.ident
            )),
            Item::Union(u) => out.push(format!(
                "{loc}: union `{}` — not a boundary citizen",
                u.ident
            )),
            other => out.push(format!(
                "{loc}: disallowed item `{}` at the boundary",
                describe(other)
            )),
        }
    }

    let mut purity = PurityVisitor {
        loc: loc.to_string(),
        imports: std_effect_imports(&file.items),
        hits: Vec::new(),
    };
    purity.visit_file(file);
    out.extend(purity.hits);
}

fn check_fields(loc: &str, name: &str, fields: &Fields, out: &mut Vec<String>) {
    let field_types: Vec<(&Visibility, &Type)> = match fields {
        Fields::Named(n) => n.named.iter().map(|f| (&f.vis, &f.ty)).collect(),
        Fields::Unnamed(u) => u.unnamed.iter().map(|f| (&f.vis, &f.ty)).collect(),
        Fields::Unit => return,
    };

    // A value object must not expose its internals.
    if field_types
        .iter()
        .any(|(vis, _)| !matches!(vis, Visibility::Inherited))
    {
        out.push(format!(
            "{loc}: `{name}` has a public field — a value object must not expose its internals"
        ));
    }

    // A raw primitive may appear ONLY as the lone field of a newtype WRAPPER
    // (`Int(i64)`, `Ident(String)`). Anywhere else — nested in a collection,
    // a named field, a multi-field tuple — it is a value object DOWNGRADED to a
    // primitive (e.g. `BTreeMap<String, _>` instead of `BTreeMap<Ident, _>`),
    // which is what forces re-parsing later. Make that impossible here.
    if is_primitive_newtype(fields) {
        return;
    }
    for (_, ty) in &field_types {
        let mut finder = PrimitiveFinder { offender: None };
        finder.visit_type(ty);
        if let Some(prim) = finder.offender {
            out.push(format!(
                "{loc}: `{name}` has a field containing raw `{prim}` — a value object must compose \
                 value objects; a primitive may appear only as the lone field of a newtype wrapper \
                 (e.g. `Ident(String)`), never downgraded inside one"
            ));
        }
    }
}

/// A single-field tuple struct whose field is a bare primitive — the sanctioned
/// wrapper (`Int(i64)`, `Ident(String)`). The one place a primitive is allowed.
fn is_primitive_newtype(fields: &Fields) -> bool {
    matches!(fields, Fields::Unnamed(u) if u.unnamed.len() == 1 && is_bare_primitive(&u.unnamed[0].ty))
}

/// A type that IS a raw primitive at the top level (no generic arguments), as
/// opposed to one that merely contains a primitive nested inside generics.
fn is_bare_primitive(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        if let Some(seg) = tp.path.segments.last() {
            return RAW_PRIMITIVES.contains(&seg.ident.to_string().as_str())
                && matches!(seg.arguments, syn::PathArguments::None);
        }
    }
    false
}

fn describe(item: &Item) -> &'static str {
    match item {
        Item::ExternCrate(_) => "extern crate",
        Item::ForeignMod(_) => "extern block",
        Item::TraitAlias(_) => "trait alias",
        Item::Verbatim(_) => "unparsed tokens",
        _ => "item",
    }
}

// ===== ALGEBRA: effects must be declared capabilities ====================
//
// An ALGEBRA-tier file may touch the world (a report tool writes files, reads sources) — but not
// SILENTLY. A function whose body reaches `std::fs`/`io`/`process`/`net`/`env`/`thread` must DECLARE
// a capability in its doc (`Capability: Effectful`), so the world-touch is an honest, named edge and
// not a hidden dependency. This is the fine seam/capability/leaf partition turned into a build
// obligation, the analogue of the file-level `Tier:` rule.

fn check_algebra(loc: &str, file: &syn::File, out: &mut Vec<String>) {
    let imports = std_effect_imports(&file.items);
    check_algebra_items(loc, &file.items, &imports, out);
}

fn check_algebra_items(
    loc: &str,
    items: &[Item],
    imports: &HashMap<String, String>,
    out: &mut Vec<String>,
) {
    for item in items {
        match item {
            // a `#[cfg(test)]` module is test scaffolding, not a production edge — skip it.
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    check_algebra_items(loc, inner, imports, out);
                }
            }
            Item::Fn(f) if !is_cfg_test(&f.attrs) => {
                let decl = capability_declaration(&f.attrs);
                if let CapabilityDecl::Malformed(text) = &decl {
                    out.push(format!(
                        "{loc}: `{}` has a `Capability:` line that names no capability (`{text}`) — \
                         the word after `Capability:` must be one of {} for the declaration to be a \
                         contract rather than decoration.",
                        f.sig.ident,
                        CAPABILITY_NAMES.join(" / ")
                    ));
                }
                let mut eff = EffectFinder {
                    found: None,
                    imports,
                };
                eff.visit_item_fn(f);
                if let Some(effect) = eff.found {
                    if !matches!(decl, CapabilityDecl::Declared) {
                        out.push(format!(
                            "{loc}: `{}` touches the world (`{effect}`) but declares no capability — \
                             an ALGEBRA-tier effect must be an honest edge. Add a `Capability: \
                             Effectful` line to its doc, or move the effect behind a declared edge.",
                            f.sig.ident
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

/// The capability names a declaration may claim (mirrors the grammar's `Capability` lattice).
const CAPABILITY_NAMES: &[&str] = &["Pure", "Lossy", "Stateful", "Effectful"];

/// What a function's doc says about its capability.
enum CapabilityDecl {
    /// no `Capability:` line at all.
    Absent,
    /// a `Capability:` line whose first word is one of `CAPABILITY_NAMES`.
    Declared,
    /// a `Capability:` line naming NONE of them (the offending text is carried for the message).
    /// A bare-substring acceptance would let `Capability: whatever` satisfy the honesty rule —
    /// the declaration must NAME a capability, or it declares nothing.
    Malformed(String),
}

/// Read the `Capability:` declaration off a function's doc attributes, requiring the word after
/// the marker to be a real capability name.
fn capability_declaration(attrs: &[syn::Attribute]) -> CapabilityDecl {
    let mut decl = CapabilityDecl::Absent;
    for a in attrs {
        let Meta::NameValue(nv) = &a.meta else {
            continue;
        };
        if !nv.path.is_ident("doc") {
            continue;
        }
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        else {
            continue;
        };
        let text = s.value();
        let Some((_, rest)) = text.split_once("Capability:") else {
            continue;
        };
        let word: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect();
        if CAPABILITY_NAMES.contains(&word.as_str()) {
            return CapabilityDecl::Declared;
        }
        decl = CapabilityDecl::Malformed(rest.trim().to_string());
    }
    decl
}

/// The `std` modules whose reach constitutes a world-touch.
const EFFECT_MODULES: &[&str] = &["io", "fs", "process", "net", "thread", "env"];

/// The per-file map from a locally-visible name to the `std` effect module it reaches: `use
/// std::fs;` maps `fs`, `use std::fs::write;` maps `write`, `use std::fs as f;` maps `f`, and a
/// grouped `use std::{fs, io};` maps both. Without this map, an import was a loophole — the path
/// visitors only see two-segment `std::…` windows, so `use std::fs;` followed by `fs::write(…)`
/// was invisible. We take the UNION over the whole file (nested modules included): an
/// over-approximation we accept, because a shadowing name at worst flags a call for review, never
/// hides one. (A glob — `use std::fs::*;` — names nothing we can map; it remains a known gap.)
fn std_effect_imports(items: &[Item]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    collect_std_effect_imports(items, &mut map);
    map
}

fn collect_std_effect_imports(items: &[Item], map: &mut HashMap<String, String>) {
    for item in items {
        match item {
            Item::Use(u) => record_use_tree(&u.tree, &mut Vec::new(), map),
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_std_effect_imports(inner, map);
                }
            }
            _ => {}
        }
    }
}

/// Walk one `use` tree, recording every leaf whose path passes through a `std` effect module.
fn record_use_tree(
    tree: &syn::UseTree,
    prefix: &mut Vec<String>,
    map: &mut HashMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            record_use_tree(&p.tree, prefix, map);
            prefix.pop();
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                record_use_tree(t, prefix, map);
            }
        }
        syn::UseTree::Name(n) => {
            record_import(prefix, &n.ident.to_string(), &n.ident.to_string(), map)
        }
        syn::UseTree::Rename(r) => {
            record_import(prefix, &r.ident.to_string(), &r.rename.to_string(), map)
        }
        syn::UseTree::Glob(_) => {}
    }
}

/// Record one imported leaf: `local` is the name the file will use, `ident` the item imported.
fn record_import(prefix: &[String], ident: &str, local: &str, map: &mut HashMap<String, String>) {
    if prefix.first().map(String::as_str) != Some("std") {
        return;
    }
    let module = match prefix.get(1) {
        // an item WITHIN an effect module: `use std::fs::write;`, `use std::io::Write as W;`.
        Some(second) if EFFECT_MODULES.contains(&second.as_str()) => second.as_str(),
        Some(_) => return,
        // the effect module ITSELF: `use std::fs;` / `use std::fs as f;`.
        None if EFFECT_MODULES.contains(&ident) => ident,
        None => return,
    };
    // `use std::fs::{self};` imports the MODULE under its own name, not a leaf called `self`.
    let local = if local == "self" {
        prefix.last().map(String::as_str).unwrap_or(local)
    } else {
        local
    };
    map.insert(local.to_string(), format!("std::{module}"));
}

/// Detects a world-touch: a path through `std::{fs,io,process,net,thread,env}` — written out in
/// full, or reaching the module through one of the file's `use std::…` imports.
struct EffectFinder<'a> {
    found: Option<String>,
    imports: &'a HashMap<String, String>,
}

impl<'ast> Visit<'ast> for EffectFinder<'_> {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if self.found.is_none() {
            self.found = effect_in_path(node, self.imports);
        }
        visit::visit_path(self, node);
    }
}

/// The effect module a path reaches, if any: a `std::<eff>` window anywhere in the path, or a
/// FIRST segment an import maps into one (`fs::write` after `use std::fs;`, bare `write(…)` after
/// `use std::fs::write;`).
fn effect_in_path(node: &syn::Path, imports: &HashMap<String, String>) -> Option<String> {
    let segs: Vec<String> = node.segments.iter().map(|s| s.ident.to_string()).collect();
    for w in segs.windows(2) {
        if w[0] == "std" && EFFECT_MODULES.contains(&w[1].as_str()) {
            return Some(format!("std::{}", w[1]));
        }
    }
    if let Some(module) = segs.first().and_then(|s| imports.get(s.as_str())) {
        return Some(format!("{module} (reached via `use`)"));
    }
    None
}

// ===== tier 2: the inward rule (parse-don't-validate) ====================

fn check_internal(loc: &str, file: &syn::File, out: &mut Vec<String>) {
    let mut v = RuleAVisitor {
        loc: loc.to_string(),
        hits: Vec::new(),
    };
    v.visit_file(file);
    out.extend(v.hits);
}

struct RuleAVisitor {
    loc: String,
    hits: Vec<String>,
}

impl<'ast> Visit<'ast> for RuleAVisitor {
    fn visit_signature(&mut self, sig: &'ast Signature) {
        if let ReturnType::Type(_, ty) = &sig.output {
            let mut finder = PrimitiveFinder { offender: None };
            finder.visit_type(ty);
            if let Some(prim) = finder.offender {
                self.hits.push(format!(
                    "{}: `{}` returns a raw `{}` — every domain primitive must be a value object \
                     with its own operators (e.g. Int / Ident), never returned un-typed",
                    self.loc, sig.ident, prim
                ));
            }
        }
        visit::visit_signature(self, sig);
    }
}

/// A raw primitive that must not escape a module-internal function via its return
/// type. `bool` is intentionally absent — a predicate is control, not domain data.
const RAW_PRIMITIVES: &[&str] = &[
    "String", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128",
    "usize", "f32", "f64", "char",
];

/// Detects a raw primitive anywhere within a return type (incl. inside generics
/// such as `BTreeMap<Ident, i64>` and tuples, and `&str`).
struct PrimitiveFinder {
    offender: Option<String>,
}

impl<'ast> Visit<'ast> for PrimitiveFinder {
    fn visit_path(&mut self, p: &'ast syn::Path) {
        if let Some(seg) = p.segments.last() {
            let id = seg.ident.to_string();
            if self.offender.is_none() && RAW_PRIMITIVES.contains(&id.as_str()) {
                self.offender = Some(id);
            }
        }
        visit::visit_path(self, p);
    }

    fn visit_type_reference(&mut self, r: &'ast syn::TypeReference) {
        if let Type::Path(tp) = &*r.elem {
            if self.offender.is_none()
                && tp
                    .path
                    .segments
                    .last()
                    .map(|s| s.ident == "str")
                    .unwrap_or(false)
            {
                self.offender = Some("str".to_string());
            }
        }
        visit::visit_type_reference(self, r);
    }
}

/// Flags I/O and `unsafe` anywhere in a boundary file (tier 1 only). `imports` is the file's
/// `use std::…` effect-import map (see `std_effect_imports`), so an imported name is as visible
/// to this pass as a fully written path.
struct PurityVisitor {
    loc: String,
    imports: HashMap<String, String>,
    hits: Vec<String>,
}

impl<'ast> Visit<'ast> for PurityVisitor {
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.hits.push(format!(
            "{}: `unsafe` block — boundaries are safe code",
            self.loc
        ));
        visit::visit_expr_unsafe(self, node);
    }

    // `unsafe` is a KEYWORD in three positions, and a block is only one of them — an `unsafe fn`
    // (free or in an impl) and an `unsafe impl` must be caught too, or the rule reads "no unsafe"
    // while enforcing "no unsafe *blocks*".
    fn visit_signature(&mut self, sig: &'ast Signature) {
        if sig.unsafety.is_some() {
            self.hits.push(format!(
                "{}: `unsafe fn {}` — boundaries are safe code",
                self.loc, sig.ident
            ));
        }
        visit::visit_signature(self, sig);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if node.unsafety.is_some() {
            self.hits.push(format!(
                "{}: `unsafe impl` — boundaries are safe code",
                self.loc
            ));
        }
        visit::visit_item_impl(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(seg) = node.path.segments.last() {
            let id = seg.ident.to_string();
            if matches!(
                id.as_str(),
                "println" | "print" | "eprintln" | "eprint" | "dbg"
            ) {
                self.hits.push(format!(
                    "{}: `{}!` — a boundary performs no I/O (it is a pure value layer)",
                    self.loc, id
                ));
            }
            // A macro's TOKENS are unparsed, so the path visitor never sees them — a
            // `std::fs::write(…)` smuggled inside any macro invocation would slip through the
            // name check alone. A token-level scan closes that: any `std :: <eff>` sequence in
            // the flattened token stream is a world-reach. (The stream stringifies with a space
            // around `::`, so the needle is `std :: fs` etc.)
            let tokens = node.tokens.to_string();
            for eff in EFFECT_MODULES {
                if tokens.contains(&format!("std :: {eff}")) {
                    self.hits.push(format!(
                        "{}: `{}!` carries `std::{}` in its tokens — a boundary performs no \
                         I/O / side effects",
                        self.loc, id, eff
                    ));
                }
            }
        }
        visit::visit_macro(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(eff) = effect_in_path(node, &self.imports) {
            self.hits.push(format!(
                "{}: `{}` — a boundary performs no I/O / side effects",
                self.loc, eff
            ));
        }
        visit::visit_path(self, node);
    }
}

#[cfg(test)]
mod qualify_line_tests {
    use super::qualify_line;

    /// The public per-file emitter produces EXACTLY the census line — the operator names and
    /// sorts sorted and deduped, the same format `collect_qualifications` pushes — so the
    /// edit-time hook and the frozen census can never disagree about one file.
    #[test]
    fn qualify_line_matches_the_census_format() {
        let src = "pub struct A; pub struct B;\n\
                   pub fn f(x: A) -> B { todo!() }\n\
                   pub fn g(y: B) -> A { todo!() }\n";
        assert_eq!(
            qualify_line(src, "src/m.rs").unwrap().as_deref(),
            Some("src/m.rs: QUALIFIES — operators [f, g] over sorts {A, B}"),
        );
    }

    /// A module whose functions are not operator-shaped (a primitive return, an effect) forms no
    /// algebra: `Ok(None)`, the honest "does not / no longer qualify" the census reflects by the
    /// absence of a line.
    #[test]
    fn a_non_algebra_module_is_ok_none() {
        assert_eq!(
            qualify_line("pub fn n(x: u32) -> u32 { x }\n", "src/m.rs"),
            Ok(None)
        );
        assert_eq!(qualify_line("// just a comment\n", "src/m.rs"), Ok(None));
    }

    /// A file that does not PARSE (a half-written edit) is `Err`, distinct from `Ok(None)` — a
    /// caller must treat it as "no signal yet", never as a qualification change.
    #[test]
    fn an_unparseable_file_is_err_not_none() {
        assert!(qualify_line("pub fn broken( -> {", "src/m.rs").is_err());
    }

    /// ASSOCIATED FUNCTIONS ARE OPERATORS: `self`/`&self`/`Self` all resolve to the
    /// typestate, methods key `Type::method` (two typestates' `new`s stay distinct), and
    /// the census line carries the resolved sorts — no `Self` ever leaks as a sort.
    #[test]
    fn impl_methods_qualify_with_the_receiver_resolved() {
        let src = "pub struct A;\n\
                   impl A {\n\
                       pub fn f(self, o: A) -> A { o }\n\
                       pub fn g(&self, o: Self) -> Self { let _ = o; A }\n\
                   }\n";
        assert_eq!(
            qualify_line(src, "src/m.rs").unwrap().as_deref(),
            Some("src/m.rs: QUALIFIES — operators [A::f, A::g] over sorts {A}"),
        );
    }

    /// The refusals that keep the method rule honest: `&mut self` is mutation, a type
    /// PARAMETER is a variable not a sort (on the method, the impl, or a free fn), and a
    /// generic impl target has no typestate to resolve to.
    #[test]
    fn method_shape_refusals() {
        assert_eq!(
            qualify_line(
                "pub struct A;\nimpl A { pub fn f(&mut self) -> A { A } }\n",
                "src/m.rs"
            ),
            Ok(None)
        );
        assert_eq!(
            qualify_line(
                "pub struct A;\nimpl A { pub fn f<T>(self, x: T) -> A { let _ = x; A } }\n",
                "src/m.rs"
            ),
            Ok(None)
        );
        assert_eq!(
            qualify_line("pub fn id<T>(x: T) -> T { x }\n", "src/m.rs"),
            Ok(None)
        );
        assert_eq!(
            qualify_line(
                "pub struct W<T>(T);\npub struct A;\n\
                 impl<T> W<T> { pub fn f(&self, a: A) -> A { a } }\n",
                "src/m.rs"
            ),
            Ok(None)
        );
    }
}

#[cfg(test)]
mod reason_census_tests {
    use super::refusal_classes_from_items;

    fn classes(src: &str) -> Vec<&'static str> {
        refusal_classes_from_items(&syn::parse_file(src).expect("parses").items)
    }

    /// A QUALIFYING file has NO refusal classes — the two censuses partition the scanned
    /// files by construction (this is the same predicate the qualify emitter answers).
    #[test]
    fn a_qualifying_file_has_no_classes() {
        assert!(classes("pub struct A;\npub fn f(x: A) -> A { todo!() }\n").is_empty());
    }

    /// A file with no functions at all (types, data, glue) is its own class — and an
    /// impl-attached operator now QUALIFIES its file (no refusal classes): the census
    /// reads impls, so the blind spot the first minting exposed is dissolved, not renamed.
    #[test]
    fn no_functions_is_a_class_and_impl_operators_qualify() {
        assert_eq!(classes("pub struct A;\n"), vec!["no functions"]);
        assert!(classes("pub struct A;\nimpl A { pub fn f(&self) -> A { A } }\n").is_empty());
    }

    /// Methods are classified by the same rule as free functions, with the receiver
    /// resolved: `&mut self` is its own class (mutation is not operator shape), and a
    /// method on a GENERIC impl target reads as a parameterised receiver.
    #[test]
    fn method_receivers_are_classified() {
        assert_eq!(
            classes("pub struct A;\nimpl A { pub fn f(&mut self) -> A { A } }\n"),
            vec!["mutating receivers"]
        );
        assert_eq!(
            classes(
                "pub struct W<T>(T);\npub struct A;\n\
                 impl<T> W<T> { pub fn f(&self) -> A { A } }\n"
            ),
            vec!["parameterised types"]
        );
    }

    /// The signature classes, one probe each: primitives, borrows, parameterised types,
    /// unit returns, zero-argument constants, unshaped types — aggregated and sorted
    /// across a file's free functions.
    #[test]
    fn signature_classes_name_each_blocker() {
        assert_eq!(
            classes("pub fn n(x: u32) -> u32 { x }\n"),
            vec!["primitive signatures"]
        );
        assert_eq!(
            classes("pub struct A;\npub fn f(x: &A) -> A { todo!() }\n"),
            vec!["borrowed types"]
        );
        assert_eq!(
            classes("pub struct A;\npub fn f(x: Option<A>) -> A { todo!() }\n"),
            vec!["parameterised types"]
        );
        assert_eq!(
            classes("pub struct A;\npub fn f(x: A) { let _ = x; }\n"),
            vec!["unit returns"]
        );
        assert_eq!(
            classes("pub struct A;\npub fn f() -> A { A }\n"),
            vec!["zero-argument constants"]
        );
        assert_eq!(
            classes("pub struct A;\npub fn f(x: (A, A)) -> A { x.0 }\n"),
            vec!["unshaped types"]
        );
        // aggregation is a sorted union across functions:
        assert_eq!(
            classes(
                "pub struct A;\npub fn f(x: u32) -> u32 { x }\npub fn g(x: &A) -> A { todo!() }\n"
            ),
            vec!["borrowed types", "primitive signatures"]
        );
    }

    /// An effectful body blocks an otherwise operator-shaped function, and the class
    /// names it — the I/O half of the operator-shape rule, same evidence the qualify
    /// walk reads.
    #[test]
    fn an_effectful_body_is_its_own_class() {
        assert_eq!(
            classes(
                "use std::fs;\npub struct A;\n\
                 pub fn f(x: A) -> A { let _ = fs::read(\"p\"); x }\n"
            ),
            vec!["effectful bodies"]
        );
    }
}
