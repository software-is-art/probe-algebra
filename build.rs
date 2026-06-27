//! build.rs — enforces the boundary discipline at COMPILE time, in two tiers.
//!
//! TIER 1 — domain boundary files (`src/<module>/boundary.rs`): the strict
//! grammar. May contain ONLY value objects, typestates, and value operators —
//! no free functions, no global state, no submodules, no traits, no public
//! fields, no I/O / `unsafe`.
//!
//! TIER 2 — module-internal files (any other `.rs` inside a module directory,
//! e.g. `internal.rs`): the "workshop". Mutation and raw collections are fine
//! here, but the INWARD rule still holds: a function may not RETURN a raw
//! primitive — `String`/`&str` or any numeric (`i64`, `usize`, `f64`, ...) —
//! because every primitive that means something in the domain must be a value
//! object with its own operators. `bool` is exempt (a predicate is control, not
//! domain data). Accessors that unwrap to a primitive live at the boundary
//! (tier 1), the sanctioned exit hatch — they are not subject to this rule.
//!
//! EXEMPT — files directly under `src/` (`main.rs`, the grammar `boundary.rs`,
//! test files): the crate root / vocabulary definition, not a module interior.

use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Fields, Item, ReturnType, Signature, Type, Visibility};

fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src = Path::new(&root).join("src");
    println!("cargo:rerun-if-changed={}", src.display());

    let mut violations = Vec::new();
    walk(&src, &src, &root, &mut violations);

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

fn walk(dir: &Path, src_root: &Path, manifest: &str, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, src_root, manifest, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        // files directly under src/ are exempt (crate root / grammar / tests)
        if path.parent() == Some(src_root) {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());

        let loc = path
            .strip_prefix(manifest)
            .unwrap_or(&path)
            .display()
            .to_string();
        let file = match parse(&path) {
            Ok(f) => f,
            Err(msg) => {
                out.push(format!("{loc}: {msg}"));
                continue;
            }
        };

        if path.file_name().and_then(|n| n.to_str()) == Some("boundary.rs") {
            check_boundary(&loc, &file, out); // tier 1
        } else {
            check_internal(&loc, &file, out); // tier 2
        }
    }
}

fn parse(path: &Path) -> Result<syn::File, String> {
    let source = std::fs::read_to_string(path).map_err(|e| format!("cannot read ({e})"))?;
    syn::parse_file(&source).map_err(|e| format!("parse error ({e})"))
}

// ===== tier 1: the strict boundary grammar ===============================

fn check_boundary(loc: &str, file: &syn::File, out: &mut Vec<String>) {
    for item in &file.items {
        match item {
            Item::Use(_) | Item::Impl(_) | Item::Macro(_) | Item::Type(_) | Item::Const(_) => {}
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
    let visibilities = match fields {
        Fields::Named(n) => n.named.iter().map(|f| &f.vis).collect::<Vec<_>>(),
        Fields::Unnamed(u) => u.unnamed.iter().map(|f| &f.vis).collect::<Vec<_>>(),
        Fields::Unit => return,
    };
    for vis in visibilities {
        if !matches!(vis, Visibility::Inherited) {
            out.push(format!(
                "{loc}: `{name}` has a public field — a value object must not expose its internals"
            ));
            break;
        }
    }
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
                     with its own operators (e.g. Account / Cents / Balance), never returned un-typed",
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
/// such as `BTreeMap<Account, i64>` and tuples, and `&str`).
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
