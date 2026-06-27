//! build.rs — enforces the boundary grammar at COMPILE time.
//!
//! Every per-module boundary file (`src/<module>/boundary.rs`) is parsed with
//! `syn` and checked against the discipline: it may contain ONLY value objects,
//! typestates, and value operators — no free functions, no global state, no
//! submodules, no traits, no public fields, and no I/O / `unsafe` (boundaries
//! are pure). A violation fails the build with a pointed message.
//!
//! The universal grammar file `src/boundary.rs` is exempt — it DEFINES the
//! vocabulary (traits, the generic `probe`/`run` operators), so it is not a
//! domain boundary and is skipped.

use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{Fields, Item, Visibility};

fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let src = Path::new(&root).join("src");
    println!("cargo:rerun-if-changed={}", src.display());

    let mut boundaries = Vec::new();
    collect_boundaries(&src, &src, &mut boundaries);

    let mut violations = Vec::new();
    for path in &boundaries {
        println!("cargo:rerun-if-changed={}", path.display());
        check_file(path, &mut violations);
    }

    if !violations.is_empty() {
        violations.sort();
        violations.dedup();
        for v in &violations {
            println!("cargo:warning={}", v);
        }
        panic!(
            "boundary grammar enforcement failed: {} violation(s) — see warnings above",
            violations.len()
        );
    }
}

/// Collect `boundary.rs` files belonging to a module (i.e. NOT the top-level
/// `src/boundary.rs`, which is the grammar definition).
fn collect_boundaries(dir: &Path, src_root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_boundaries(&path, src_root, out);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("boundary.rs") {
            // skip the universal grammar file (parent == src root)
            if path.parent() != Some(src_root) {
                out.push(path);
            }
        }
    }
}

fn check_file(path: &Path, out: &mut Vec<String>) {
    let loc = path
        .strip_prefix(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .unwrap_or(path)
        .display()
        .to_string();

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            out.push(format!("{loc}: cannot read ({e})"));
            return;
        }
    };
    let file = match syn::parse_file(&source) {
        Ok(f) => f,
        Err(e) => {
            out.push(format!("{loc}: parse error ({e})"));
            return;
        }
    };

    for item in &file.items {
        match item {
            // allowed boundary citizens & supporting items
            Item::Use(_) | Item::Impl(_) | Item::Macro(_) | Item::Type(_) | Item::Const(_) => {}
            Item::Struct(s) => check_fields(&loc, &s.ident.to_string(), &s.fields, out),
            Item::Enum(e) => {
                for v in &e.variants {
                    check_fields(&loc, &format!("{}::{}", e.ident, v.ident), &v.fields, out);
                }
            }
            // disallowed at a domain boundary
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

    // cross-cutting purity scan over the whole file
    let mut purity = PurityVisitor {
        loc: loc.clone(),
        hits: Vec::new(),
    };
    purity.visit_file(&file);
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

/// Flags I/O, `unsafe`, and other impurity anywhere in a boundary file.
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
