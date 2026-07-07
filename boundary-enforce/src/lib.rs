//! Build-time enforcement of the boundary discipline, over an EXPLICIT, TOTAL partition —
//! extracted from `probe-algebra`'s build script so ANY crate can attach the same discipline
//! from its own `build.rs`.
//!
//! A boundary is a CATEGORY: value-object OBJECTS and value-operator MORPHISMS
//! (`Morphism` / `Construction` / `Branch` / `Guarded`), with typestates as object INDICES.
//!
//! Every source file declares its place in the partition with a `//! Tier: <NAME>` marker in its
//! header — there is no silent exemption. A new module that names no tier is a BUILD ERROR, so the
//! partition stays total and the choice is ratified in the diff. The four tiers, and what each
//! enforces:
//!
//! * **KERNEL** — the trusted floor: it DEFINES and RUNS the format, so it is exempt from the
//!   structural rules — but it is NAMED, not silently skipped. Claiming kernel-hood claims
//!   exemption from every other rule, so it cannot be self-serve: the file must ALSO be on the
//!   consumer's [`Config::kernel_allowlist`], which lives in the consumer's OWN `build.rs` — every
//!   new kernel member is a reviewed diff in the consumer's tree, ratified, not asserted.
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
//! Finally, the **qualification census**: boundary-hood as a COMPUTED property. Every module whose
//! functions form a discoverable algebra is listed, the report is frozen to
//! [`Config::qualify_spec`] and drift-gated (regenerate by setting the [`Config::bless_env`]
//! environment variable, `BLESS_QUALIFY=1` by default).
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

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{Fields, FnArg, Item, ItemFn, Meta, ReturnType, Signature, Type, Visibility};

/// What to enforce, and where. Construct with [`Config::new`] and override fields as needed —
/// most importantly [`Config::kernel_allowlist`], which MUST live in the consumer's own
/// `build.rs` so every kernel admission is a reviewed diff in the consumer's tree.
pub struct Config {
    /// The source tree to walk (usually `<manifest>/src`).
    pub src_root: PathBuf,
    /// The manifest directory: violation locations are rendered relative to it, so messages
    /// never leak a machine's absolute path and stay stable across checkouts.
    pub manifest_dir: PathBuf,
    /// The RATIFIED kernel — the only files (manifest-relative paths) allowed to declare
    /// `Tier: KERNEL`. The marker alone is not enough: kernel-hood exempts a file from every
    /// structural rule, so admitting a new member must be a diff in the consumer's build.rs,
    /// where review cannot miss it.
    pub kernel_allowlist: Vec<String>,
    /// Where the qualification census is frozen and drift-gated. `None` skips the census
    /// freeze/drift pass entirely (the census is still computed and returned).
    pub qualify_spec: Option<PathBuf>,
    /// The environment variable that, when set, regenerates [`Config::qualify_spec`] instead of
    /// drift-checking it. Default: `BLESS_QUALIFY`.
    pub bless_env: String,
    /// Where the TIER census is frozen and drift-gated — the declared partition held against
    /// DERIVED evidence (ladder step one of tiers-as-a-lock; see the consumer's roadmap).
    /// `None` skips the freeze/drift pass (the census is still computed and returned).
    pub tiers_spec: Option<PathBuf>,
    /// The bless variable for [`Config::tiers_spec`]. Default: `BLESS_TIERS`.
    pub tiers_bless_env: String,
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
    /// The rendered tier census (what [`Config::tiers_spec`] freezes): per file, the declared
    /// tier held against the derived evidence — agreement, disagreement, or kernel decision.
    pub tiers_census: String,
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
        walk(
            &config.src_root,
            &config.manifest_dir,
            &config.kernel_allowlist,
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
            &mut rerun,
            &mut violations,
        );

        // TIER CENSUS: the declared partition held against DERIVED evidence — ladder step
        // one of tiers-as-a-lock. INTERIOR is derivable (pub-reachability from the crate
        // roots), BOUNDARY is derivable (the qualify bar: operator-shaped structure),
        // ALGEBRA is the reachable remainder; KERNEL is a DECISION (a privilege can never
        // be inferred from conduct), recorded and never judged. A DISAGREES row is the
        // honest distance between the declared partition and what structure can currently
        // derive — the freeze ratifies that distance; burning it down (or improving the
        // derivation) is the prerequisite for ladder step two.
        let tiers_census = render_tiers(
            &config.src_root,
            &config.manifest_dir,
            &config.tiers_bless_env,
        );
        freeze_or_gate(
            &config.tiers_spec,
            &tiers_census,
            &config.tiers_bless_env,
            "the tier census",
            &config.manifest_dir,
            &mut rerun,
            &mut violations,
        );

        violations.sort();
        violations.dedup();
        Enforcement {
            violations,
            qualify_census,
            tiers_census,
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
fn freeze_or_gate(
    spec_path: &Option<PathBuf>,
    census: &str,
    bless_env: &str,
    label: &str,
    manifest: &Path,
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
            violations.push(format!(
                "{} is stale — {label} drifted. Regenerate with `{bless_env}=1 cargo build` \
                 and ratify the diff.",
                rel.display(),
            ));
        }
    }
}

/// One file's row in the tier census: what it declares, and the derived evidence.
struct TierRow {
    loc: String,
    declared: Option<&'static str>,
    qualifies: bool,
    /// Child modules this file declares: `(name, is_pub)`.
    mods: Vec<(String, bool)>,
    /// The directory this file's child modules resolve under.
    child_dir: PathBuf,
}

/// The tier census: every file's declared tier held against the derived evidence.
fn render_tiers(src: &Path, manifest: &Path, bless_env: &str) -> String {
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

    let mut lines: Vec<String> = Vec::new();
    let (mut agree, mut disagree, mut kernel) = (0usize, 0usize, 0usize);
    for (path, row) in &rows {
        let is_reachable = reachable.contains(path);
        let derived = if !is_reachable {
            "INTERIOR"
        } else if row.qualifies {
            "BOUNDARY"
        } else {
            "ALGEBRA"
        };
        let evidence = match (is_reachable, row.qualifies) {
            (false, _) => "not pub-reachable",
            (true, true) => "pub-reachable, operator-shaped",
            (true, false) => "pub-reachable, not operator-shaped",
        };
        let line = match row.declared {
            Some("KERNEL") => {
                kernel += 1;
                format!(
                    "- {}: declared KERNEL — a decision (ratified in build.rs), never derived; \
                     evidence for the record: {evidence}",
                    row.loc
                )
            }
            Some(declared) if declared == derived => {
                agree += 1;
                format!(
                    "- {}: declared {declared}; derived {derived} ({evidence}) — agree",
                    row.loc
                )
            }
            Some(declared) => {
                disagree += 1;
                format!(
                    "- {}: declared {declared}; derived {derived} ({evidence}) — DISAGREES",
                    row.loc
                )
            }
            None => format!(
                "- {}: declares no tier (the partition pass refuses this separately)",
                row.loc
            ),
        };
        lines.push(line);
    }
    lines.sort();

    let mut report = format!(
        "# tier census — the declared partition held against DERIVED evidence: INTERIOR is\n\
         # non-pub reachability, BOUNDARY is operator-shaped structure (the qualify bar),\n\
         # ALGEBRA is the reachable remainder. KERNEL is a decision, recorded and never\n\
         # judged — a privilege cannot be inferred from conduct. A DISAGREES row is the\n\
         # honest distance between the declared partition and what structure can derive\n\
         # today; burn it down or improve the derivation before deleting any marker\n\
         # (the ladder: derive alongside, coherence-gate, then delete). Regenerate with\n\
         # `{bless_env}=1 cargo build`.\n",
    );
    report.push_str(&format!(
        "# {} files: {agree} agree, {disagree} disagree, {kernel} kernel decisions.\n\n",
        rows.len()
    ));
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
        let imports = std_effect_imports(&file.items);
        let mut ops: Vec<Op> = Vec::new();
        qualify_items(&file.items, &imports, &mut ops);
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
                declared: declared_tier(&source),
                qualifies: !ops.is_empty(),
                mods,
                child_dir,
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
        let imports = std_effect_imports(&file.items);
        let mut ops: Vec<Op> = Vec::new();
        qualify_items(&file.items, &imports, &mut ops);
        if ops.is_empty() {
            continue;
        }
        let loc = path.strip_prefix(manifest).unwrap_or(&path).display();
        let mut names: Vec<&str> = ops.iter().map(|o| o.name.as_str()).collect();
        names.sort_unstable();
        let mut sorts: Vec<&str> = ops
            .iter()
            .flat_map(|o| o.args.iter().chain(std::iter::once(&o.ret)))
            .map(|s| s.as_str())
            .collect();
        sorts.sort_unstable();
        sorts.dedup();
        out.push(format!(
            "{loc}: QUALIFIES — operators [{}] over sorts {{{}}}",
            names.join(", "),
            sorts.join(", ")
        ));
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
                if let Some(op) = operator_candidate(f, imports) {
                    out.push(op);
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

/// Is `f` an operator over value objects? Every argument and the return must be a BARE NAMED value
/// type (a path with no generics, not a raw primitive or `bool`), and the body must do no I/O — the
/// shape `#[algebra]` reads. A `&[Value]`-style evaluator, a primitive return, or an effect is not.
fn operator_candidate(f: &ItemFn, imports: &HashMap<String, String>) -> Option<Op> {
    let ret = match &f.sig.output {
        ReturnType::Type(_, ty) => named_value_type(ty)?,
        ReturnType::Default => return None,
    };
    let mut args = Vec::new();
    for arg in &f.sig.inputs {
        let FnArg::Typed(pt) = arg else {
            return None;
        };
        args.push(named_value_type(&pt.ty)?);
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
    eff.visit_item_fn(f);
    if eff.found.is_some() {
        return None;
    }
    Some(Op {
        name: f.sig.ident.to_string(),
        args,
        ret,
    })
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
    kernel_allowlist: &[String],
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
            walk(&path, manifest, kernel_allowlist, out, rerun);
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

        // THE PARTITION, made total and explicit: every source file must DECLARE its tier. This
        // replaces path heuristics (filename `boundary.rs` → tier 1; a blanket directory skip) —
        // so a new module cannot land silently un-categorized; placing it in the partition is a
        // build obligation, ratified in the diff.
        let Some(tier) = declared_tier(&source) else {
            out.push(format!(
                "{loc}: no `Tier:` declaration — every source file must name its place in the \
                 partition. Add a `//! Tier: <{}> — …` line to the module header.",
                TIERS.join(" | ")
            ));
            continue;
        };

        // dispatch the STRUCTURAL discipline on the declared tier:
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
                // Claiming kernel-hood is claiming EXEMPTION from every rule below, so it cannot
                // be self-serve: the file must ALSO be on the allowlist the CONSUMER's build.rs
                // passes in, making every new kernel member a diff in the consumer's own tree —
                // ratified, not asserted.
                if !kernel_allowlist.iter().any(|k| k == &loc) {
                    out.push(format!(
                        "{loc}: declares `Tier: KERNEL` but is not in KERNEL_ALLOWLIST — KERNEL \
                         tier must be ratified in build.rs. Add the file to the allowlist there \
                         (making the exemption a reviewed diff), or declare its real tier."
                    ));
                }
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
                    if !constant_operator && operator_candidate(f, imports).is_none() {
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

/// The partition tiers — every source file declares exactly one (see `walk` for what each enforces).
const TIERS: &[&str] = &["KERNEL", "BOUNDARY", "INTERIOR", "ALGEBRA"];

/// The tier a file declares, via a `//! Tier: <NAME>` marker in its header (the module doc). `None`
/// if it declares none — which `walk` turns into a violation, so the partition stays total. Only
/// lines that ARE module-doc lines (`//!`) count: a `Tier:` mention in a string literal, a code
/// comment, or prose cannot satisfy (or spoof) the declaration. We keep the simple fixed 40-line
/// window — every real header opens the file, so a marker below line 40 is a marker hidden from the
/// reader, and we would rather reject it than hunt for it.
fn declared_tier(source: &str) -> Option<&'static str> {
    for line in source.lines().take(40) {
        if !line.trim_start().starts_with("//!") {
            continue;
        }
        let Some((_, rest)) = line.split_once("Tier:") else {
            continue;
        };
        let word: String = rest
            .trim_start()
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect();
        if let Some(t) = TIERS.iter().find(|t| word.eq_ignore_ascii_case(t)) {
            return Some(t);
        }
    }
    None
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
