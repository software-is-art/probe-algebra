//! build.rs — enforces the boundary discipline at COMPILE time, over an EXPLICIT, TOTAL partition.
//!
//! A boundary is a CATEGORY: value-object OBJECTS and value-operator MORPHISMS
//! (`Morphism` / `Construction` / `Branch` / `Guarded`), with typestates as object INDICES.
//!
//! Every source file declares its place in the partition with a `//! Tier: <NAME>` marker in its
//! header — there is no silent exemption. A new module that names no tier is a BUILD ERROR, so the
//! partition stays total and the choice is ratified in the diff. The four tiers:
//!
//! KERNEL   — the trusted floor: the grammar (`src/boundary.rs`), the discovery engine and the
//!            `theory!` macro, and crate-level tooling (`gdp`, `harness`, `capability`), plus the
//!            crate root. It DEFINES and RUNS the format, so it is exempt from the structural rules
//!            — but it is NAMED, not silently skipped.
//!
//! BOUNDARY — a domain's strict value-object surface (`src/<module>/boundary.rs`): TIER 1. May
//!            contain ONLY value objects, typestates, and value operators — no free functions,
//!            global state, submodules, traits, public fields, I/O, or `unsafe`.
//!
//! INTERIOR — the workshop / leaves (`internal.rs`, module glue): TIER 2. Mutation and raw
//!            collections are fine, but the INWARD rule holds: a function may not RETURN a raw
//!            primitive — `String`/`&str` or any numeric — because every domain primitive must be a
//!            value object with its own operators. `bool` is exempt (a predicate is control, not
//!            domain data); boundary accessors that unwrap to a primitive are the sanctioned hatch.
//!
//! ALGEBRA  — a discovered-law / report layer (the `theory!` domains and the `discover` meta): it
//!            renders human-facing reports (counts, prose, observations), so it is exempt from the
//!            inward rule. Its finer seam / capability-edge / leaf split is enforced WITHIN it by
//!            discovered laws and declared capabilities (see `discover::architect`), not by this
//!            structural pass.

use std::collections::HashSet;
use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Fields, FnArg, Item, ItemFn, Meta, ReturnType, Signature, Type, Visibility};

fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src = Path::new(&root).join("src");
    println!("cargo:rerun-if-changed={}", src.display());

    let mut violations = Vec::new();
    walk(&src, &src, &root, &mut violations);

    // EDGE-COMPLETENESS, by enumeration: every PRODUCTION boundary edge must carry a probe.
    // This closes the open-world residue the `edges!` list left — a new edge impl that no probe
    // can kill is now a BUILD ERROR, not a silently-missing line. `cargo-mutants` cannot reach
    // this (it mutates bodies, not the set of impls), so it is the type/build analogue of the
    // grading-law proofs.
    let mut edges: Vec<EdgeImpl> = Vec::new();
    let mut probed: HashSet<String> = HashSet::new();
    collect_edges_and_probes(&src, &mut edges, &mut probed);
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

    // QUALIFICATION CENSUS: boundary-hood as a COMPUTED property, not a file convention. For every
    // module, compute whether its functions form a discoverable algebra — operator-shaped (each
    // argument and the return a bare NAMED value type, no raw primitives, no I/O). A module qualifies
    // by STRUCTURE, wherever it lives and whatever it is named; `boundary.rs` is just one place that
    // happens to. The census is frozen into `spec/qualify.spec` and drift-gated, so the answer is
    // ratified in the diff (regenerate with `BLESS_QUALIFY=1 cargo build`).
    qualify_census(&src, &root, &mut violations);

    if !violations.is_empty() {
        violations.sort();
        violations.dedup();
        for v in &violations {
            println!("cargo:warning={}", v);
        }
        panic!(
            "boundary discipline enforcement failed: {} violation(s) — see warnings above",
            violations.len()
        );
    }
}

// ===== qualification census: which modules ARE algebras, by structure ====

/// Compute the algebra-qualification of every module, render it, and drift-check it against the
/// committed `spec/qualify.spec`.
fn qualify_census(src: &Path, manifest: &str, out: &mut Vec<String>) {
    let mut lines: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    collect_qualifications(src, manifest, &mut scanned, &mut lines);
    lines.sort();

    let mut report = String::from(
        "# qualify census — modules that meet the algebra spec by STRUCTURE: their functions are\n\
         # operator-shaped (every argument and the return a bare named value type, no primitives, no\n\
         # I/O). Boundary-hood is a COMPUTED property here, not the `boundary.rs` file convention — a\n\
         # module qualifies wherever it lives. Regenerate with `BLESS_QUALIFY=1 cargo build`.\n",
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

    let spec_path = Path::new(manifest).join("spec/qualify.spec");
    println!("cargo:rerun-if-changed={}", spec_path.display());
    if std::env::var("BLESS_QUALIFY").is_ok() {
        std::fs::write(&spec_path, &report).expect("write spec/qualify.spec");
        return;
    }
    let committed = std::fs::read_to_string(&spec_path).unwrap_or_default();
    if committed != report {
        out.push(
            "spec/qualify.spec is stale — the algebra-qualification census drifted. Regenerate \
             with `BLESS_QUALIFY=1 cargo build` and ratify the diff."
                .to_string(),
        );
    }
}

/// Walk `dir`, parsing each `.rs` file, and push a `QUALIFIES` line for every file whose functions
/// form an algebra.
fn collect_qualifications(dir: &Path, manifest: &str, scanned: &mut usize, out: &mut Vec<String>) {
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
        let mut ops: Vec<Op> = Vec::new();
        qualify_items(&file.items, &mut ops);
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
fn qualify_items(items: &[Item], out: &mut Vec<Op>) {
    for it in items {
        match it {
            Item::Fn(f) if !is_cfg_test(&f.attrs) => {
                if let Some(op) = operator_candidate(f) {
                    out.push(op);
                }
            }
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    qualify_items(inner, out);
                }
            }
            _ => {}
        }
    }
}

/// Is `f` an operator over value objects? Every argument and the return must be a BARE NAMED value
/// type (a path with no generics, not a raw primitive or `bool`), and the body must do no I/O — the
/// shape `#[algebra]` reads. A `&[Value]`-style evaluator, a primitive return, or an effect is not.
fn operator_candidate(f: &ItemFn) -> Option<Op> {
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
    let mut eff = EffectFinder { found: None };
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

fn walk(dir: &Path, _src_root: &Path, manifest: &str, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, _src_root, manifest, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

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
        // replaces the old path heuristics (filename `boundary.rs` → tier 1; a blanket `discover/`
        // skip) — so a new module cannot land silently un-categorized; placing it in the partition
        // is a build obligation, ratified in the diff.
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
            "KERNEL" => continue,
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
    }
}

/// The partition tiers — every source file declares exactly one (see `walk` for what each enforces).
const TIERS: &[&str] = &["KERNEL", "BOUNDARY", "INTERIOR", "ALGEBRA"];

/// The tier a file declares, via a `Tier: <NAME>` marker in its header (the module doc). `None` if
/// it declares none — which `walk` turns into a violation, so the partition stays total.
fn declared_tier(source: &str) -> Option<&'static str> {
    for line in source.lines().take(40) {
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

/// Walk EVERY `.rs` under `src/` (including the crate-root files `walk` exempts, since the
/// `Probed` impls live in `src/harness.rs`), collecting concrete production edges and the set of
/// types that carry an `impl Probed`.
fn collect_edges_and_probes(dir: &Path, edges: &mut Vec<EdgeImpl>, probed: &mut HashSet<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_edges_and_probes(&path, edges, probed);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let loc = path.display().to_string();
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
            Item::Mod(m) => {
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
// not a hidden dependency. This is the fine seam/capability/leaf partition (which v12 established by
// hand on `architect`) turned into a build obligation, the analogue of the file-level `Tier:` rule.

fn check_algebra(loc: &str, file: &syn::File, out: &mut Vec<String>) {
    check_algebra_items(loc, &file.items, out);
}

fn check_algebra_items(loc: &str, items: &[Item], out: &mut Vec<String>) {
    for item in items {
        match item {
            // a `#[cfg(test)]` module is test scaffolding, not a production edge — skip it.
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    check_algebra_items(loc, inner, out);
                }
            }
            Item::Fn(f) if !is_cfg_test(&f.attrs) => {
                let mut eff = EffectFinder { found: None };
                eff.visit_item_fn(f);
                if let Some(effect) = eff.found {
                    if !declares_capability(&f.attrs) {
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

/// Whether any doc attribute carries a `Capability:` declaration.
fn declares_capability(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        if let Meta::NameValue(nv) = &a.meta {
            if nv.path.is_ident("doc") {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    return s.value().contains("Capability:");
                }
            }
        }
        false
    })
}

/// Detects a world-touch: a path through `std::{fs,io,process,net,thread,env}`.
struct EffectFinder {
    found: Option<String>,
}

impl<'ast> Visit<'ast> for EffectFinder {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        let segs: Vec<String> = node.segments.iter().map(|s| s.ident.to_string()).collect();
        for w in segs.windows(2) {
            if w[0] == "std"
                && matches!(
                    w[1].as_str(),
                    "io" | "fs" | "process" | "net" | "thread" | "env"
                )
                && self.found.is_none()
            {
                self.found = Some(format!("std::{}", w[1]));
            }
        }
        visit::visit_path(self, node);
    }
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

/// Flags I/O and `unsafe` anywhere in a boundary file (tier 1 only).
struct PurityVisitor {
    loc: String,
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
        }
        visit::visit_macro(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let segs: Vec<String> = node.segments.iter().map(|s| s.ident.to_string()).collect();
        for w in segs.windows(2) {
            if w[0] == "std"
                && matches!(
                    w[1].as_str(),
                    "io" | "fs" | "process" | "net" | "thread" | "env"
                )
            {
                self.hits.push(format!(
                    "{}: `std::{}` — a boundary performs no I/O / side effects",
                    self.loc, w[1]
                ));
            }
        }
        visit::visit_path(self, node);
    }
}
