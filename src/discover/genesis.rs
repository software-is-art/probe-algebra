//!
//! genesis — the BLANK-SLATE layout generator: one compact declaration in, a whole crate
//! layout out.
//!
//! `scaffold` (this module's closest relative) regenerates structure from an EXISTING engine —
//! it needs code to read. Genesis serves the day before that: an agent starting a NEW
//! application writes ONE declaration file — the whole app's structure: value objects, modules,
//! operators with signatures, declared law expectations, seams — and genesis mechanically emits
//! the entire crate layout the `downstream-fixture` models by hand. The declaration fits in a
//! context window; the layout is DERIVED from it, never transcribed, so the declaration→files
//! translation stops being a manual, unchecked step.
//!
//! # The input is Rust, not a DSL
//!
//! The declaration is a `.rs` file containing a `system! { ... }`-shaped macro invocation. It
//! need not expand (there is no `system!` macro to expand — the committed sample cfg's the block
//! out); genesis PARSES it with `syn` from the token stream. The v1 grammar, every production:
//!
//! ```text
//! system   := "name" ":" LitStr ","  values  modules  seams?
//! values   := "values"  "{" value+ "}"
//! value    := Ident "=" Type "where" rule ";"
//! rule     := LitStr
//!           | Int "..=" Int ("saturating")?
//!             — a value object: its name, its raw representation (any Rust type), and its
//!               validity rule. A PROSE rule (the string form) is a HOLE: words carried
//!               verbatim into the generated stubs as doc'd `todo!()`s — genesis never
//!               generates meaning. A STRUCTURED rule is tokens, not words, so everything it
//!               mechanically implies is GENERATED rather than left for the author to
//!               re-derive by hand: an integer range emits the validity predicate and the
//!               edge-seeking `Shaped` grid, and naming the `saturating` re-entry policy
//!               additionally emits the clamping `mint` (the policy is meaning — clamp vs
//!               reject vs panic — so an unnamed policy leaves `mint` a hole).
//! modules  := "modules" "{" module+ "}"
//! module   := Ident "{" ops expects? "}"
//! ops      := "ops" "{" op+ "}"
//! op       := Ident "(" (Ident ("," Ident)*)? ")" "->" Ident ";"
//!             — an operator signature over declared value names. Zero inputs declares a
//!               CONSTANT (a nullary operator, e.g. `zero() -> Credits`).
//! expects  := "expects" "{" expect+ "}"
//! expect   := shape "(" Ident ("," Ident)* ")" ";"
//! shape    := any `discover::expect` declaration key except "irreflexive"
//!             — the declared LAW EXPECTATIONS: the engine's WHOLE ratified shape catalog
//!               (`ShapeCatalog::inventory`), each line validated against the shape's DATA
//!               gate (`ShapeGate::admit`) — the same gate discovery tries operators
//!               against, so declarable and discoverable are one vocabulary. Arities and
//!               sort relations come from the gate: `commutative(op)`,
//!               `identity(op, constant)`, `monoid_action(action, combiner)`,
//!               `round_trip(outer, inner)`, `homomorphism(conversion, from_op, to_op)`, …
//!               (`irreflexive` alone is refused here: its witness constant must render as
//!               `false`, which no operator identifier can spell — declare
//!               `self_application` or leave it to discovery.) The keys are the same ones
//!               `discover::expect` and the `#[algebra]` macro's `expects(...)` speak.
//! seams    := "seams" "{" seam* "}"
//! seam     := Ident "--" Ident ":" ("transport" | "transform") "on" Ident ("via" Ident)? ";"
//!             — a declared seam between two modules on a shared value: `transport` promises
//!               the sort is the SAME type on both sides (discharged by construction — genesis
//!               defines each value once); `transform` promises a conversion that must be a
//!               HOMOMORPHISM. Naming the conversion (`via h`, a declared unary operator from
//!               the seam's value into the other side's) turns the whole obligation into
//!               STRUCTURE: genesis emits the seam's SPANNING theory (source operator,
//!               conversion, target operator, with the homomorphism expectation riding the
//!               attribute), compiles the seam into the `system!` graph, gates the declared
//!               law in `tests/expectations.rs`, and renders the preserved stanza into the
//!               system target lock. Unnamed, the obligation stays a `tests/seams.rs` hole.
//! ```
//!
//! # The output is the fixture's shape
//!
//! For a blank target directory, `Genesis::apply` emits exactly the layout `downstream-fixture`
//! hand-builds: a `Cargo.toml` (deps on `boundary-spec`, `spec-lock`, `boundary-enforce` —
//! path-parameterised or by version, per [`Deps`]), a `build.rs` enforcement shim, a KERNEL
//! `src/lib.rs` roster, and per module a BOUNDARY surface file (value-object newtype stubs,
//! operator methods delegating inward), an INTERIOR `<m>_internal.rs` workshop, and a shared
//! ALGEBRA `src/ops.rs` whose `#[algebra]` modules carry the declared signatures — with the
//! declared expectations passed through AS WRITTEN in an `expects(...)` attribute argument.
//! `spec/` receives TARGET lock files: the DECLARED laws rendered in the exact committed lock
//! format (`discover::freeze::render`'s header, law lines from the shape catalog's prose
//! templates, coverage line) so the drift gate is RED until discovery re-earns the declaration.
//! `tests/freeze_gate.rs` + `examples/freeze.rs` are the lock loop, `tests/probes.rs` the edge
//! stub, and `tests/seams.rs` the declared seam obligations.
//!
//! # Generation is honest
//!
//! Everything mechanical is generated; every hole is loud and greppable — a `todo!()` whose
//! message starts with `MEANING:`, naming exactly what a human/agent supplies (validity
//! predicates, interior operator bodies, constants, grid shapes, edge observations). The
//! generated tree is guaranteed SYN-CLEAN (every emitted `.rs` parses; the unit tests hold the
//! generator to that) but is NOT promised to compile or pass: `cargo build` is gated by the
//! emitted enforcement shim until its qualify census is blessed, the freeze gate is red until
//! discovery matches the declaration, and every reached hole panics with its `MEANING:` label.
//! Wrong-but-compiling stub semantics would be worse than loud incompleteness, so genesis never
//! fakes a body. One nuance it inherits from the engine: a declared `identity(op, e)` renders
//! the canonical left form `(e op x) = x`; if the meaning only earns the right-hand identity,
//! the bless diff shows the correction — that diff is the review.
//!
//! Entry point: `cargo run --example genesis -- <declaration.rs> <target-dir>` (sample
//! declaration: `examples/genesis_demo.rs`). Per the no-rats-nest rule the public callables
//! hang off the [`Genesis`] typestate: `Genesis::plan` is the PURE half (parse + validate +
//! generate, no I/O), `Genesis::apply` the one Effectful edge — writes confined to the target
//! root via `Architect::apply`'s path-traversal rejection.

use std::collections::BTreeSet;
use std::path::Path;

use syn::parse::{Parse, ParseStream, Parser as _};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, Ident, LitStr, Token};

use crate::discover::architect::{Architect, CodeAction, FileEdit};
use crate::discover::engine::{Fixity, Polarity, ShapeCatalog, ShapeInfo, Slot};
use crate::discover::expect::Expectation;

// ===== the parse-side representation ========================================================
//
// Declared law expectations are NOT genesis's own vocabulary: a parsed `expects { ... }` line
// becomes a `discover::expect::Expectation` — the same value the `theory!` macro's clause and
// `#[algebra(..., expects(...))]` construct, and the same identity a discovered law carries.
// Genesis only restricts WHICH shapes v1 admits (`V1_EXPECT_KEYS`: the catalog's homogeneous
// binary rows) and renders the target-lock law lines through the catalog's own prose
// templates (`ShapeInfo::instantiate`), so declared and discovered cannot drift apart.

/// The whole parsed declaration — everything `system! { ... }` states.
pub struct SystemDecl {
    /// The crate name (`name: "credit-app"`).
    pub name: String,
    /// The declared value objects, in declaration order.
    pub values: Vec<ValueDecl>,
    /// The declared modules, in declaration order.
    pub modules: Vec<ModuleDecl>,
    /// The declared seams (possibly empty).
    pub seams: Vec<SeamDecl>,
}

/// One value object: name, raw representation (any Rust type, rendered back to text), and
/// its validity rule.
pub struct ValueDecl {
    pub name: String,
    pub raw: String,
    pub rule: Rule,
}

/// A declared validity rule. PROSE is a hole (meaning is never generated from words);
/// a STRUCTURED rule is tokens, so the artifacts it mechanically implies are generated —
/// re-deriving them by hand would be exactly the transcription genesis exists to delete.
pub enum Rule {
    /// Words, carried verbatim into doc'd `todo!()` holes.
    Prose(String),
    /// An inclusive integer range `lo..=hi`; `saturating` names the re-entry policy (the
    /// interior `mint` clamps). Without the policy the predicate and the edge-seeking grid
    /// are still generated, but `mint` stays a hole — clamp vs reject vs panic is meaning.
    Range {
        lo: i128,
        hi: i128,
        saturating: bool,
    },
}

#[crate::mutate]
impl Rule {
    /// The rule as one doc-safe line — what the generated doc comments and hole messages
    /// quote (`0..=20 (saturating)`, or the prose itself).
    fn doc(&self) -> String {
        match self {
            Rule::Prose(words) => doc_safe(words),
            Rule::Range {
                lo,
                hi,
                saturating: true,
            } => format!("{lo}..={hi} (saturating)"),
            Rule::Range { lo, hi, .. } => format!("{lo}..={hi}"),
        }
    }
}

/// One module: its operators and its declared law expectations.
pub struct ModuleDecl {
    pub name: String,
    pub ops: Vec<OpDecl>,
    pub expects: Vec<Expectation>,
}

/// One operator signature over declared value names. Zero inputs is a constant.
pub struct OpDecl {
    pub name: String,
    pub inputs: Vec<String>,
    pub output: String,
}

/// A declared seam between two modules on a shared value. A TRANSFORM seam may name the
/// conversion that crosses it (`via h`) — with the conversion named, the whole obligation
/// becomes structure genesis can emit (a spanning theory, a compiled seam, a verdict test);
/// without it, the obligation stays a meaning hole in `tests/seams.rs`.
pub struct SeamDecl {
    pub left: String,
    pub right: String,
    pub kind: SeamKindDecl,
    pub on: String,
    /// The crossing conversion's operator name (transform seams only).
    pub via: Option<String>,
}

/// The declared seam kind (parse-side twin of `cohesion::SeamKind`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeamKindDecl {
    /// The shared sort is the SAME type on both sides — preserved by construction.
    Transport,
    /// A conversion crosses the seam and must be a HOMOMORPHISM — an emitted obligation.
    Transform,
}

/// The catalog row a (validated) expectation instantiates.
#[crate::mutate]
fn shape_info(e: &Expectation) -> ShapeInfo {
    ShapeCatalog::inventory()
        .into_iter()
        .find(|shape| shape.name == e.shape)
        .expect("an Expectation's shape is always a ratified catalog name")
}

/// The shape-catalog rank — the order the engine tries (and therefore renders) the shapes, so
/// the target lock's law order matches a discovery that confirms the declaration.
#[crate::mutate]
fn shape_rank(e: &Expectation) -> usize {
    ShapeCatalog::inventory()
        .iter()
        .position(|shape| shape.name == e.shape)
        .expect("an Expectation's shape is always a ratified catalog name")
}

/// The declared law as the lock will state it: `(prose, equation)`. BOTH halves come from the
/// ratified shape catalog — the PROSE via `ShapeInfo::instantiate` over the declared names,
/// the EQUATION via `ShapeInfo::equation` over the shape's canonical terms — one source,
/// never restated. The fixities are the `#[algebra]` render conventions for a generated
/// theory's operators (binaries, actions, and relations infix under their own names, unaries
/// prefix, constants bare) and every sort's variables are `x`, `y`, `z` — so the derived
/// equation is byte-for-byte what the engine renders for the shape's discovered instance.
/// The dynamic sync test (`the_target_lock_reproduces_discovery_byte_for_byte`) holds this
/// to the freeze's actual render.
#[crate::mutate]
fn law(e: &Expectation) -> (String, String) {
    let op = e.ops[0].as_str();
    let subs: Vec<(&str, &str)> = match e.shape {
        "identity" | "annihilation" | "action identity" | "self-application" | "non-constancy" => {
            vec![("op", op), ("const", e.ops[1].as_str())]
        }
        "distributivity" | "absorption" | "monoid action" | "round-trip" => {
            vec![("op", op), ("other", e.ops[1].as_str())]
        }
        "homomorphism" => vec![
            ("op", op),
            ("other", e.ops[1].as_str()),
            ("via", e.ops[2].as_str()),
        ],
        _ => vec![("op", op)],
    };
    let info = shape_info(e);
    let prose = info.instantiate(&subs);
    let ops: Vec<(&str, Fixity)> = info
        .gate_slots
        .slots
        .iter()
        .zip(&e.ops)
        .map(|(slot, name)| {
            let fixity = match slot {
                Slot::Constant(_) => Fixity::Nullary,
                Slot::Unary(..) => Fixity::Prefix,
                Slot::Binary(_) | Slot::Action(..) | Slot::Relation(..) => Fixity::Infix,
            };
            (name.as_str(), fixity)
        })
        .collect();
    let vars: &[&[&str]] = &[&["x", "y", "z"], &["x", "y", "z"]];
    (prose, info.equation(&ops, vars))
}

/// Where the generated crate's dependencies point — the flag `cargo run --example genesis`
/// takes. `Path` writes path dependencies into a `probe-algebra` checkout (the given directory
/// must be the checkout root; `spec-lock` and `boundary-enforce` live under it); `Version`
/// writes registry versions.
pub enum Deps {
    Path(String),
    Version(String),
}

// ===== the parser (syn over the macro's token stream) =======================================

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(values);
    syn::custom_keyword!(modules);
    syn::custom_keyword!(ops);
    syn::custom_keyword!(expects);
    syn::custom_keyword!(seams);
    syn::custom_keyword!(on);
    syn::custom_keyword!(transport);
    syn::custom_keyword!(transform);
    syn::custom_keyword!(via);
}

/// A raw type's canonical text: `quote`'s token render with its inter-token spacing
/// collapsed back to source form (`Vec < u8 >` → `Vec<u8>`), so the generated newtypes read
/// as written.
#[crate::mutate]
fn type_text(ty: &syn::Type) -> String {
    quote::quote!(#ty)
        .to_string()
        .replace(" :: ", "::")
        .replace("< ", "<")
        .replace(" <", "<")
        .replace(" >", ">")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" ,", ",")
}

/// One end of a range rule: an integer literal, optionally negated.
#[crate::mutate]
fn parse_bound(input: ParseStream) -> syn::Result<i128> {
    let negative = input.peek(Token![-]);
    if negative {
        input.parse::<Token![-]>()?;
    }
    let lit: syn::LitInt = input.parse()?;
    let magnitude: i128 = lit.base10_parse()?;
    Ok(if negative { -magnitude } else { magnitude })
}

#[crate::mutate]
fn parse_value(input: ParseStream) -> syn::Result<ValueDecl> {
    let name: Ident = input.parse()?;
    input.parse::<Token![=]>()?;
    let raw: syn::Type = input.parse()?;
    input.parse::<Token![where]>()?;
    let rule = if input.peek(LitStr) {
        Rule::Prose(input.parse::<LitStr>()?.value())
    } else {
        let lo = parse_bound(input)?;
        input.parse::<Token![..=]>()?;
        let hi = parse_bound(input)?;
        let saturating = input.peek(syn::Ident) && {
            let policy: Ident = input.parse()?;
            if policy != "saturating" {
                return Err(syn::Error::new(
                    policy.span(),
                    format!(
                        "unknown re-entry policy `{policy}` — the structured vocabulary is \
                         `saturating` (or omit the policy to leave `mint` a hole)"
                    ),
                ));
            }
            true
        };
        Rule::Range { lo, hi, saturating }
    };
    input.parse::<Token![;]>()?;
    Ok(ValueDecl {
        name: name.to_string(),
        raw: type_text(&raw),
        rule,
    })
}

#[crate::mutate]
fn parse_op(input: ParseStream) -> syn::Result<OpDecl> {
    let name: Ident = input.parse()?;
    let args;
    parenthesized!(args in input);
    let inputs: Punctuated<Ident, Token![,]> = args.parse_terminated(Ident::parse, Token![,])?;
    input.parse::<Token![->]>()?;
    let output: Ident = input.parse()?;
    input.parse::<Token![;]>()?;
    Ok(OpDecl {
        name: name.to_string(),
        inputs: inputs.iter().map(Ident::to_string).collect(),
        output: output.to_string(),
    })
}

#[crate::mutate]
fn parse_expect(input: ParseStream) -> syn::Result<Expectation> {
    let shape: Ident = input.parse()?;
    let args;
    parenthesized!(args in input);
    let idents: Punctuated<Ident, Token![,]> = args.parse_terminated(Ident::parse, Token![,])?;
    input.parse::<Token![;]>()?;
    let names: Vec<String> = idents.iter().map(Ident::to_string).collect();
    if shape == "irreflexive" {
        // Declarable in `#[algebra]` (its witness constant is a string literal there), but
        // not here: the shape's witness must RENDER as `false`, and no genesis operator
        // identifier can spell that keyword. Refuse with the alternative named.
        return Err(syn::Error::new(
            shape.span(),
            "`irreflexive` is not declarable in a genesis declaration — its witness constant \
             must render as `false`, which no operator identifier can spell. Declare \
             `self_application(op, constant)` instead, or leave the law to discovery",
        ));
    }
    // arity per key, from the catalog's DATA gate — the same source validation checks.
    let arity_of = |key: &'static str| {
        let canonical = Expectation::canonical(key).expect("vocabulary key");
        ShapeCatalog::inventory()
            .into_iter()
            .find(|info| info.name == canonical)
            .map(|info| info.gate_slots.slots.len())
            .expect("the vocabulary and the catalog move in lockstep")
    };
    let key = Expectation::vocabulary_keys()
        .into_iter()
        .find(|key| shape == key && arity_of(key) == names.len());
    let Some(key) = key else {
        let vocabulary: Vec<String> = Expectation::vocabulary_keys()
            .into_iter()
            .filter(|key| *key != "irreflexive")
            .map(|key| format!("{key}/{}", arity_of(key)))
            .collect();
        return Err(syn::Error::new(
            shape.span(),
            format!(
                "unknown expectation `{shape}` with {} argument(s); the declarable vocabulary \
                 (key/arity) is {}",
                names.len(),
                vocabulary.join(", ")
            ),
        ));
    };
    let mut distinct = names.clone();
    distinct.sort();
    distinct.dedup();
    if distinct.len() != names.len() {
        // `Expectation` normalises to DISTINCT names (a discovered law's fingerprint does
        // the same), so a repeated name would silently drop a slot — refuse it.
        return Err(syn::Error::new(
            shape.span(),
            format!(
                "expectation `{shape}({})` names the same operator twice — each slot needs a \
                 distinct operator in a genesis declaration",
                names.join(", ")
            ),
        ));
    }
    Ok(Expectation::of(key, names))
}

#[crate::mutate]
fn parse_module(input: ParseStream) -> syn::Result<ModuleDecl> {
    let name: Ident = input.parse()?;
    let body;
    braced!(body in input);
    body.parse::<kw::ops>()?;
    let ops_body;
    braced!(ops_body in body);
    let mut ops = Vec::new();
    while !ops_body.is_empty() {
        ops.push(parse_op(&ops_body)?);
    }
    let mut expects = Vec::new();
    if body.peek(kw::expects) {
        body.parse::<kw::expects>()?;
        let expects_body;
        braced!(expects_body in body);
        while !expects_body.is_empty() {
            expects.push(parse_expect(&expects_body)?);
        }
    }
    if !body.is_empty() {
        return Err(
            body.error("unexpected tokens in module body (v1: `ops { }` then `expects { }`)")
        );
    }
    Ok(ModuleDecl {
        name: name.to_string(),
        ops,
        expects,
    })
}

#[crate::mutate]
fn parse_seam(input: ParseStream) -> syn::Result<SeamDecl> {
    let left: Ident = input.parse()?;
    input.parse::<Token![-]>()?;
    input.parse::<Token![-]>()?;
    let right: Ident = input.parse()?;
    input.parse::<Token![:]>()?;
    let kind = if input.peek(kw::transport) {
        input.parse::<kw::transport>()?;
        SeamKindDecl::Transport
    } else if input.peek(kw::transform) {
        input.parse::<kw::transform>()?;
        SeamKindDecl::Transform
    } else {
        return Err(input.error("seam kind must be `transport` or `transform`"));
    };
    input.parse::<kw::on>()?;
    let on: Ident = input.parse()?;
    let via = if input.peek(kw::via) {
        let via_kw = input.parse::<kw::via>()?;
        if kind == SeamKindDecl::Transport {
            return Err(syn::Error::new(
                via_kw.span,
                "only a transform seam names a conversion — a transport seam shares the one \
                 type unchanged",
            ));
        }
        Some(input.parse::<Ident>()?.to_string())
    } else {
        None
    };
    input.parse::<Token![;]>()?;
    Ok(SeamDecl {
        left: left.to_string(),
        right: right.to_string(),
        kind,
        on: on.to_string(),
        via,
    })
}

#[crate::mutate]
fn parse_system(input: ParseStream) -> syn::Result<SystemDecl> {
    input.parse::<kw::name>()?;
    input.parse::<Token![:]>()?;
    let name: LitStr = input.parse()?;
    input.parse::<Token![,]>()?;

    input.parse::<kw::values>()?;
    let values_body;
    braced!(values_body in input);
    let mut values = Vec::new();
    while !values_body.is_empty() {
        values.push(parse_value(&values_body)?);
    }

    input.parse::<kw::modules>()?;
    let modules_body;
    braced!(modules_body in input);
    let mut modules = Vec::new();
    while !modules_body.is_empty() {
        modules.push(parse_module(&modules_body)?);
    }

    let mut seams = Vec::new();
    if input.peek(kw::seams) {
        input.parse::<kw::seams>()?;
        let seams_body;
        braced!(seams_body in input);
        while !seams_body.is_empty() {
            seams.push(parse_seam(&seams_body)?);
        }
    }
    if !input.is_empty() {
        return Err(input.error("unexpected tokens after the last section"));
    }
    Ok(SystemDecl {
        name: name.value(),
        values,
        modules,
        seams,
    })
}

/// Find the `system! { ... }` invocation in the declaration file and parse its token stream.
#[crate::mutate]
fn parse_declaration(source: &str) -> Result<(SystemDecl, String), String> {
    let file = syn::parse_file(source)
        .map_err(|e| format!("genesis: the declaration is not parseable Rust: {e}"))?;
    let mac = file
        .items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Macro(m)
                if m.mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "system") =>
            {
                Some(m.mac)
            }
            _ => None,
        })
        .ok_or_else(|| {
            "genesis: no `system! { ... }` invocation found in the declaration".to_string()
        })?;
    let system = parse_system
        .parse2(mac.tokens.clone())
        .map_err(|e| format!("genesis: system! declaration: {e}"))?;
    // the ORIGINAL text between the macro's braces, author formatting intact — what the
    // generated `src/system.rs` splices verbatim (one declaration, two lifecycle stages).
    let text = match &mac.delimiter {
        syn::MacroDelimiter::Brace(brace) => {
            let open = brace.span.open().end();
            let close = brace.span.close().start();
            slice_by_line_column(source, open, close)
                .trim_matches(|c| c == '\n' || c == '\r')
                .to_string()
        }
        // paren/bracket invocations lose author formatting; fall back to the token render.
        _ => mac.tokens.to_string(),
    };
    Ok((system, text))
}

/// The byte slice of `source` between two proc-macro2 line/column positions (lines are
/// 1-based, columns are UTF-8-character offsets within the line).
#[crate::mutate]
fn slice_by_line_column(
    source: &str,
    start: proc_macro2::LineColumn,
    end: proc_macro2::LineColumn,
) -> &str {
    &source[byte_offset(source, start)..byte_offset(source, end)]
}

/// The byte offset of a proc-macro2 line/column position in `source` (lines 1-based, columns
/// UTF-8-character offsets within the line) — the span→text bridge, stated once: genesis's
/// declaration splice and the bundle's item segmentation (`discover::bundle`) both cut with it.
#[crate::mutate]
pub(crate) fn byte_offset(source: &str, pos: proc_macro2::LineColumn) -> usize {
    let mut remaining_lines = pos.line - 1;
    let mut byte = 0;
    for (i, c) in source.char_indices() {
        if remaining_lines == 0 {
            let line_start = byte;
            return source[line_start..]
                .char_indices()
                .nth(pos.column)
                .map(|(o, _)| line_start + o)
                .unwrap_or(source.len());
        }
        if c == '\n' {
            remaining_lines -= 1;
            byte = i + 1;
        }
    }
    source.len()
}

// ===== validation (the declaration must be coherent before anything is emitted) =============

/// Reject an incoherent declaration with a message naming the exact production at fault —
/// generation only ever runs over a validated system.
#[crate::mutate]
fn validate(sys: &SystemDecl) -> Result<(), String> {
    let err = |msg: String| Err(format!("genesis: {msg}"));

    if sys.name.is_empty()
        || !sys
            .name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        || !sys
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return err(format!(
            "`{}` is not a usable crate name (ascii alphanumeric plus `-`/`_`, letter first)",
            sys.name
        ));
    }
    if sys.values.is_empty() {
        return err("a system needs at least one value object".to_string());
    }
    if sys.modules.is_empty() {
        return err("a system needs at least one module".to_string());
    }

    let mut value_names = BTreeSet::new();
    for v in &sys.values {
        if !value_names.insert(v.name.as_str()) {
            return err(format!("value `{}` is declared twice", v.name));
        }
        if let Rule::Range { lo, hi, .. } = &v.rule {
            const SIGNED: &[&str] = &["i8", "i16", "i32", "i64", "i128", "isize"];
            const UNSIGNED: &[&str] = &["u8", "u16", "u32", "u64", "u128", "usize"];
            let raw = v.raw.as_str();
            if !SIGNED.contains(&raw) && !UNSIGNED.contains(&raw) {
                return err(format!(
                    "value `{}` declares a range rule, but its raw type `{raw}` is not a \
                     primitive integer — write the rule as prose (a hole) instead",
                    v.name
                ));
            }
            if UNSIGNED.contains(&raw) && *lo < 0 {
                return err(format!(
                    "value `{}` ranges from {lo}, but `{raw}` is unsigned",
                    v.name
                ));
            }
            if lo > hi {
                return err(format!(
                    "value `{}` declares the empty range {lo}..={hi}",
                    v.name
                ));
            }
        }
    }

    let mut module_names = BTreeSet::new();
    let mut op_names = BTreeSet::new();
    for m in &sys.modules {
        if m.name == "ops" || m.name == "lib" || m.name.ends_with("_internal") {
            return err(format!(
                "module name `{}` collides with a generated file (`ops`, `lib`, `*_internal` \
                 are reserved)",
                m.name
            ));
        }
        if !module_names.insert(m.name.as_str()) {
            return err(format!("module `{}` is declared twice", m.name));
        }
        if m.ops.is_empty() {
            return err(format!("module `{}` declares no operators", m.name));
        }
        for op in &m.ops {
            // one namespace across the system: operators become surface METHODS on their first
            // input's value object, and two same-named methods on one type cannot coexist.
            if !op_names.insert(op.name.as_str()) {
                return err(format!(
                    "operator `{}` is declared twice (operator names are one namespace in v1)",
                    op.name
                ));
            }
            if op.inputs.len() > 6 {
                return err(format!("operator `{}` has more than 6 inputs", op.name));
            }
            for t in op.inputs.iter().chain(std::iter::once(&op.output)) {
                if !value_names.contains(t.as_str()) {
                    return err(format!(
                        "operator `{}` names `{t}`, which is not a declared value",
                        op.name
                    ));
                }
            }
        }
        for e in &m.expects {
            // every named operator must be this module's; then the shape's DATA gate — the
            // same one discovery tries operators against — judges the signatures.
            let mut sigs: Vec<(Vec<String>, String)> = Vec::new();
            for op_name in &e.ops {
                let Some(op) = m.ops.iter().find(|o| o.name == *op_name) else {
                    return err(format!(
                        "expectation `{}` names `{op_name}`, which module `{}` does not declare",
                        e.render(),
                        m.name
                    ));
                };
                sigs.push((op.inputs.clone(), op.output.clone()));
            }
            let names: Vec<&str> = e.ops.iter().map(String::as_str).collect();
            if let Err(why) = shape_info(e).gate_slots.admit(&sigs, &names) {
                return err(format!(
                    "expectation `{}` in module `{}`: {why}",
                    e.render(),
                    m.name
                ));
            }
        }
    }

    // every value must have an owner: the first module whose operators touch it.
    for v in &sys.values {
        if owner_of(sys, &v.name).is_none() {
            return err(format!(
                "value `{}` is used by no operator — genesis cannot place it",
                v.name
            ));
        }
    }

    for s in &sys.seams {
        if s.left == s.right {
            return err(format!(
                "seam `{} -- {}` joins a module to itself",
                s.left, s.right
            ));
        }
        for m in [&s.left, &s.right] {
            if !module_names.contains(m.as_str()) {
                return err(format!("seam names `{m}`, which is not a declared module"));
            }
        }
        if !value_names.contains(s.on.as_str()) {
            return err(format!(
                "seam names `{}`, which is not a declared value",
                s.on
            ));
        }
        if let Some(via) = &s.via {
            // a NAMED conversion: the seam becomes emit-able structure, so its pieces are
            // validated here — the conversion itself and the one endpoint binary per side
            // the spanning theory will carry.
            let module = |name: &String| {
                sys.modules
                    .iter()
                    .find(|m| m.name == *name)
                    .expect("checked above")
            };
            let (left, right) = (module(&s.left), module(&s.right));
            let Some(h) = left.ops.iter().chain(&right.ops).find(|o| o.name == *via) else {
                return err(format!(
                    "seam `{} -- {}` names conversion `{via}`, which neither module declares",
                    s.left, s.right
                ));
            };
            if h.inputs.len() != 1 {
                return err(format!(
                    "seam conversion `{via}` must be unary — `{via}({}) -> {}` is not a \
                     conversion",
                    h.inputs.join(", "),
                    h.output
                ));
            }
            if h.inputs[0] != s.on {
                return err(format!(
                    "seam conversion `{via}` converts from `{}`, but the seam is on `{}`",
                    h.inputs[0], s.on
                ));
            }
            if h.output == s.on {
                return err(format!(
                    "seam conversion `{via}` returns `{}` — a transform must land on a \
                     DIFFERENT value than it leaves",
                    s.on
                ));
            }
            let from: Vec<&OpDecl> = left
                .ops
                .iter()
                .filter(|o| is_binary_on_value(o, &s.on))
                .collect();
            let to: Vec<&OpDecl> = right
                .ops
                .iter()
                .filter(|o| is_binary_on_value(o, &h.output))
                .collect();
            for (found, side, sort) in [(&from, &s.left, &s.on), (&to, &s.right, &h.output)] {
                if found.len() != 1 {
                    return err(format!(
                        "the `{} -- {}` seam needs module `{side}` to declare exactly one \
                         homogeneous binary on `{sort}` for the homomorphism (found {})",
                        s.left,
                        s.right,
                        found.len()
                    ));
                }
            }
        } else {
            for side in [&s.left, &s.right] {
                let module = sys
                    .modules
                    .iter()
                    .find(|m| m.name == *side)
                    .expect("checked");
                let touches = module
                    .ops
                    .iter()
                    .any(|op| op.inputs.iter().any(|t| t == &s.on) || op.output == s.on);
                if !touches {
                    return err(format!(
                        "seam on `{}` claims module `{side}` shares it, but no operator of \
                         `{side}` touches it",
                        s.on
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Is `op` a homogeneous binary over declared value `v` (`v × v → v`)?
#[crate::mutate]
fn is_binary_on_value(op: &OpDecl, v: &str) -> bool {
    op.inputs.len() == 2 && op.inputs[0] == v && op.inputs[1] == v && op.output == v
}

/// The resolved pieces of a `via` transform seam — the conversion and the two endpoint
/// binaries the spanning theory carries: `(h, from_op, to_op)`. Only callable on a
/// VALIDATED system (every lookup was checked by `validate`).
#[crate::mutate]
fn via_seam_parts<'a>(sys: &'a SystemDecl, s: &SeamDecl) -> (&'a OpDecl, &'a OpDecl, &'a OpDecl) {
    let module = |name: &String| {
        sys.modules
            .iter()
            .find(|m| m.name == *name)
            .expect("validated: seam modules exist")
    };
    let (left, right) = (module(&s.left), module(&s.right));
    let via = s.via.as_deref().expect("a via seam");
    let h = left
        .ops
        .iter()
        .chain(&right.ops)
        .find(|o| o.name == via)
        .expect("validated: the conversion is declared");
    let from = left
        .ops
        .iter()
        .find(|o| is_binary_on_value(o, &s.on))
        .expect("validated: exactly one source binary");
    let to = right
        .ops
        .iter()
        .find(|o| is_binary_on_value(o, &h.output))
        .expect("validated: exactly one target binary");
    (h, from, to)
}

/// A `via` seam's spanning-theory naming: `(ops module, theory marker, display name)` —
/// `meter`/`billing` → (`meter_billing_seam_ops`, `MeterBillingSeam`, "meter-billing seam").
#[crate::mutate]
fn seam_theory_names(s: &SeamDecl) -> (String, String, String) {
    (
        format!("{}_{}_seam_ops", s.left, s.right),
        format!("{}{}Seam", camel(&s.left), camel(&s.right)),
        format!("{}-{} seam", s.left, s.right),
    )
}

// ===== naming and placement =================================================================

/// The module that OWNS a value: the first module (declaration order) whose operators mention
/// it. The value object is defined in that module's boundary file; everyone else imports it —
/// which is exactly what makes a declared `transport` seam true by construction.
#[crate::mutate]
fn owner_of<'a>(sys: &'a SystemDecl, value: &str) -> Option<&'a str> {
    sys.modules
        .iter()
        .find(|m| {
            m.ops
                .iter()
                .any(|op| op.inputs.iter().any(|t| t == value) || op.output == value)
        })
        .map(|m| m.name.as_str())
}

/// `credit_meter` → `CreditMeter`: the theory type name for a module.
#[crate::mutate]
fn camel(s: &str) -> String {
    s.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

/// The crate name as a Rust identifier (`credit-app` → `credit_app`).
#[crate::mutate]
fn crate_ident(name: &str) -> String {
    name.replace('-', "_")
}

/// The generated crate's qualify-census bless variable (`credit-app` →
/// `BLESS_CREDIT_APP_QUALIFY`) — renamed per crate so a workspace-wide bless cannot silently
/// re-bless two censuses at once (the fixture's own convention).
#[crate::mutate]
fn bless_env(name: &str) -> String {
    format!("BLESS_{}_QUALIFY", name.to_uppercase().replace('-', "_"))
}

/// The per-crate bless variable for the derived tier partition (`spec/tiers.spec`).
#[crate::mutate]
fn bless_tiers_env(name: &str) -> String {
    format!("BLESS_{}_TIERS", name.to_uppercase().replace('-', "_"))
}

/// Escape text for embedding inside a generated double-quoted string literal.
#[crate::mutate]
fn esc_lit(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

/// One line of validity prose safe for a generated doc comment.
#[crate::mutate]
fn doc_safe(s: &str) -> String {
    s.replace('\n', " ")
}

const ARG_NAMES: [&str; 6] = ["a", "b", "c", "d", "e", "f"];

/// The `//! GENERATED …` banner every emitted `.rs` carries under its tier line.
#[crate::mutate]
fn banner(app: &str) -> String {
    format!(
        "//!\n\
         //! GENERATED by genesis from the `{app}` system declaration. STRUCTURE here is derived\n\
         //! and regenerable; MEANING is not — every hole is a loud `todo!(\"MEANING: …\")`.\n\
         //! Grep `MEANING:` for the complete list of what a human/agent still owes this crate.\n"
    )
}

/// The expectations that name `op` as their operator, in shape-catalog order — for doc lines
/// and the target lock.
#[crate::mutate]
fn expects_for<'a>(m: &'a ModuleDecl, op: &str) -> Vec<&'a Expectation> {
    let mut found: Vec<&Expectation> = m.expects.iter().filter(|e| e.ops[0] == op).collect();
    found.sort_by_key(|e| shape_rank(e));
    found
}

// ===== emission (one function per generated file) ===========================================

#[crate::mutate]
fn emit_cargo_toml(sys: &SystemDecl, deps: &Deps) -> String {
    let (bspec, slock, benforce) = match deps {
        Deps::Path(root) => {
            let root = root.trim_end_matches('/');
            (
                format!("{{ path = \"{root}\" }}"),
                format!("{{ path = \"{root}/spec-lock\" }}"),
                format!("{{ path = \"{root}/boundary-enforce\" }}"),
            )
        }
        Deps::Version(v) => (format!("\"{v}\""), format!("\"{v}\""), format!("\"{v}\"")),
    };
    format!(
        "# GENERATED by genesis from the `{name}` system declaration — the layout is DERIVED,\n\
         # not transcribed. Regenerate from the declaration rather than hand-porting structure.\n\
         [package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\
         \n\
         # Stand alone even when generated inside another workspace's directory tree.\n\
         [workspace]\n\
         \n\
         [dependencies]\n\
         # The discipline under consumption: value objects, edges, discovery, `#[algebra]`.\n\
         boundary-spec = {bspec}\n\
         \n\
         [dev-dependencies]\n\
         # The freeze/drift-gate mechanics — only `tests/freeze_gate.rs` and `examples/freeze.rs`\n\
         # touch it, so a dev-dependency suffices.\n\
         spec-lock = {slock}\n\
         \n\
         [build-dependencies]\n\
         # The build-time discipline (tier partition, boundary grammar, inward rule, rats-nest,\n\
         # edge-probe completeness, qualify census), attached from this crate's own build.rs.\n\
         boundary-enforce = {benforce}\n",
        name = sys.name
    )
}

#[crate::mutate]
fn emit_build_rs(sys: &SystemDecl) -> String {
    let template = r#"//! build.rs — the enforcement shim genesis emitted: attach the whole structural discipline
//! from `boundary-enforce`, with a config that is THIS crate's own.
//!
//! Two decisions live here and must live here:
//!
//! * **The kernel allowlist.** KERNEL exempts a file from every structural rule, so it is a
//!   RATIFICATION, never derived from the file itself — membership is named here, where
//!   admitting a member is a reviewed diff in this crate's tree. The generated kernel is
//!   exactly `src/lib.rs` (the module roster).
//!
//! * **The two censuses.** FIRST BUILD: run `@BLESS@=1 @BLESS_TIERS@=1 cargo build`
//!   once to mint `spec/qualify.spec` (the algebra-qualification census) and
//!   `spec/tiers.spec` (the DERIVED tier partition — reachability, doors, glue; no file
//!   declares a tier) — a missing lock is stale, never fresh, so an unblessed tree refuses
//!   to build. From then on both are drift-gated; regenerate with the same variables and
//!   ratify the diff.

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// The RATIFIED kernel of THIS crate — the only files the partition places in KERNEL.
const KERNEL_ALLOWLIST: &[&str] = &["src/lib.rs"];

#[crate::mutate]
fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = KERNEL_ALLOWLIST.iter().map(|s| s.to_string()).collect();
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.bless_env = "@BLESS@".to_string();
    config.tiers_spec = Some(manifest.join("spec/tiers.spec"));
    config.tiers_bless_env = "@BLESS_TIERS@".to_string();
    Enforcement::enforce_or_panic(&config);
}
"#;
    template
        .replace("@BLESS_TIERS@", &bless_tiers_env(&sys.name))
        .replace("@BLESS@", &bless_env(&sys.name))
}

#[crate::mutate]
fn emit_lib_rs(sys: &SystemDecl) -> String {
    let mut out = String::from(
        "//! The crate's trusted floor: the module roster, and nothing else. KERNEL is a\n\
         //! REGISTRATION (this crate's build.rs names it), never an assertion in a file.\n",
    );
    out.push_str(&banner(&sys.name));
    out.push_str(&format!(
        "//!\n\
         //! # {name} — a crate whose layout was DERIVED from one declaration\n\
         //!\n\
         //! Everything structural in this tree — the tiered layout, the operator plumbing,\n\
         //! the lock loop — was emitted by `genesis` from a single `system! {{ ... }}` declaration\n\
         //! that fits in a context window. The declaration→files translation was mechanical, so\n\
         //! reviewing THIS crate means reviewing meaning, not transcription.\n\
         //!\n\
         //! ## The module map\n\
         //!\n",
        name = sys.name
    ));
    for m in &sys.modules {
        out.push_str(&format!(
            "//! * [`{m}`] (BOUNDARY) — value-object surface; `{m}_internal` (INTERIOR, private) —\n\
             //!   the workshop its operators delegate to, where the meaning holes live.\n",
            m = m.name
        ));
    }
    out.push_str(
        "//! * [`ops`] (ALGEBRA) — each module's operators as an `#[algebra]` theory; the declared\n\
         //!   expectations ride the attribute, and discovery must re-earn them.\n\
         //! * [`system`] (ALGEBRA) — the COMPILED `system!` graph: the module registry (the graph\n\
         //!   IS the registry the freeze loop reads) and the declared seams with their checkers.\n\
         //! * [`gates`] (ALGEBRA) — the pipeline as a declaration: `spec/gates.spec` and the CI\n\
         //!   workflow are BOTH its renders, drift-gated (`cargo run --example freeze_gates`).\n\
         //! * `spec/` — TARGET locks: the DECLARED laws (and the declared seam graph) in the exact\n\
         //!   lock format, committed RED on purpose. The drift gate stays red until discovery\n\
         //!   matches the declaration.\n\
         //!\n\
         //! ## The runbook — from generated skeleton to ratified system\n\
         //!\n\
         //! 1. `grep -rn \"MEANING:\" src tests` — the complete list of holes: validity\n\
         //!    predicates, interior operator bodies, constants, grid shapes, edge probes.\n",
    );
    out.push_str(&format!(
        "//! 2. First build: `{bless}=1 {bless_tiers}=1 cargo build` — mints `spec/qualify.spec`\n\
         //!    and `spec/tiers.spec` (the derived tier partition), which the enforcement shim\n\
         //!    (`build.rs`) drift-gates from then on.\n\
         //! 3. `cargo test` — the expectations gate names each module's DISTANCE from its\n\
         //!    declaration, and the freeze gate holds the LIVE discovered spec (module laws and\n\
         //!    the seam graph) against the TARGET locks; red means the meaning does not yet earn\n\
         //!    the declaration.\n\
         //! 4. `cargo run --example freeze` — regenerate the locks from discovery and read the\n\
         //!    diff against the targets. That diff IS the review: ratify it, or fix the meaning.\n",
        bless = bless_env(&sys.name),
        bless_tiers = bless_tiers_env(&sys.name)
    ));
    out.push('\n');
    for m in &sys.modules {
        out.push_str(&format!("pub mod {};\n", m.name));
    }
    out.push_str("pub mod gates;\npub mod ops;\npub mod system;\n\n");
    for m in &sys.modules {
        out.push_str(&format!("mod {}_internal;\n", m.name));
    }
    out
}

/// `use crate::<owner>::<Value>;` lines for every value `m`'s operators touch, deduplicated
/// and sorted (imports are mechanical; the set is exactly what the signatures name).
#[crate::mutate]
fn value_imports(sys: &SystemDecl, m: &ModuleDecl, exclude_own: bool) -> String {
    let mut lines = BTreeSet::new();
    for op in &m.ops {
        for t in op.inputs.iter().chain(std::iter::once(&op.output)) {
            let owner = owner_of(sys, t).expect("validated: every value has an owner");
            if exclude_own && owner == m.name {
                continue;
            }
            lines.insert(format!("use crate::{owner}::{t};\n"));
        }
    }
    lines.into_iter().collect()
}

/// One value object's newtype, constructor discipline, and probe surface. What is
/// generated vs left as a hole follows the rule's FORM: prose implies nothing mechanically,
/// so everything stays a `MEANING:` hole; a structured range implies its predicate and its
/// edge-seeking grid (generated), and the `saturating` policy — when declared — its
/// clamping `mint`. The policy itself is meaning, so a range WITHOUT one keeps the hole.
#[crate::mutate]
fn emit_value_object(v: &ValueDecl) -> String {
    let (name, raw) = (&v.name, &v.raw);
    let rule_doc = v.rule.doc();
    let indent = "    ";

    let new_body = match &v.rule {
        Rule::Prose(words) => {
            let lit = esc_lit(words);
            format!(
                "{indent}    // MEANING: the validity predicate for \"{rule_doc}\" — genesis carries the\n\
                 {indent}    // words, a human/agent supplies the predicate. Admit with `Some({name}(raw))`.\n\
                 {indent}    todo!(\"MEANING: validity of {name} — \\\"{lit}\\\"\")\n"
            )
        }
        Rule::Range { lo, hi, .. } => format!(
            "{indent}    // GENERATED from the declared rule `{rule_doc}` — a structured rule is\n\
             {indent}    // tokens, so its predicate is derived, never re-transcribed by hand.\n\
             {indent}    ({lo}..={hi}).contains(&raw).then_some({name}(raw))\n"
        ),
    };

    let mint = match &v.rule {
        Rule::Prose(words) => {
            let lit = esc_lit(words);
            format!(
                "{indent}/// The interior's way back into validity from a raw computation. Choosing the\n\
                 {indent}/// discipline (clamp? reject? panic?) is domain meaning, not structure.\n\
                 {indent}pub(crate) fn mint(raw: {raw}) -> {name} {{\n\
                 {indent}    todo!(\"MEANING: interior constructor for {name} (re-enter \\\"{lit}\\\")\")\n\
                 {indent}}}\n"
            )
        }
        Rule::Range {
            lo,
            hi,
            saturating: true,
        } => format!(
            "{indent}/// The interior's way back into validity — the DECLARED re-entry policy:\n\
             {indent}/// a computation may overshoot either edge of `{rule_doc}`; the value saturates.\n\
             {indent}pub(crate) fn mint(raw: {raw}) -> {name} {{\n\
             {indent}    {name}(raw.clamp({lo}, {hi}))\n\
             {indent}}}\n"
        ),
        Rule::Range { .. } => format!(
            "{indent}/// The interior's way back into validity from a raw computation.\n\
             {indent}pub(crate) fn mint(raw: {raw}) -> {name} {{\n\
             {indent}    // MEANING: the range `{rule_doc}` is declared, but the re-entry DISCIPLINE\n\
             {indent}    // is not — clamp? reject? panic? Declare `saturating` to generate the clamp.\n\
             {indent}    todo!(\"MEANING: interior constructor for {name} (re-enter {rule_doc})\")\n\
             {indent}}}\n"
        ),
    };

    let shaped = match &v.rule {
        Rule::Prose(words) => {
            let lit = esc_lit(words);
            format!(
                "/// The probe/grid surface: discovery can only refute what this grid reaches, so step\n\
                 /// the perturbations toward the validity rule's load-bearing points.\n\
                 impl Shaped for {name} {{\n\
                 {indent}fn inhabitant() -> Self {{\n\
                 {indent}    todo!(\"MEANING: the canonical valid {name}\")\n\
                 {indent}}}\n\
                 {indent}fn perturbation_classes(&self) -> Vec<Vec<Self>> {{\n\
                 {indent}    todo!(\"MEANING: neighbours of a {name}, stepping toward the edges of \\\"{lit}\\\"\")\n\
                 {indent}}}\n\
                 }}\n\n"
            )
        }
        Rule::Range { lo, hi, .. } => format!(
            "/// The probe/grid surface — GENERATED from the declared range: the grid seeds at the\n\
             /// lower edge and steps toward BOTH edges, so discovery judges laws at the rule's\n\
             /// load-bearing points (the closure reaches the entire range, cap permitting).\n\
             impl Shaped for {name} {{\n\
             {indent}fn inhabitant() -> Self {{\n\
             {indent}    {name}({lo})\n\
             {indent}}}\n\
             {indent}fn perturbation_classes(&self) -> Vec<Vec<Self>> {{\n\
             {indent}    vec![\n\
             {indent}        vec![{name}(self.0.saturating_add(1).clamp({lo}, {hi})), {name}({hi})],\n\
             {indent}        vec![{name}(self.0.saturating_sub(1).clamp({lo}, {hi})), {name}({lo})],\n\
             {indent}    ]\n\
             {indent}}}\n\
             }}\n\n"
        ),
    };

    let rule_kind = match &v.rule {
        Rule::Prose(_) => "carried verbatim — never generated",
        Rule::Range { .. } => "structured — the implied artifacts below are generated",
    };
    format!(
        "/// `{name}` — `{raw}` refined by a declared validity rule.\n\
         ///\n\
         /// VALIDITY (declared, {rule_kind}): \"{rule_doc}\"\n\
         #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]\n\
         pub struct {name}({raw});\n\
         \n\
         impl {name} {{\n\
         {indent}/// Parse, don't validate: `Some` iff `raw` satisfies the declared rule.\n\
         {indent}pub fn new(raw: {raw}) -> Option<{name}> {{\n\
         {new_body}\
         {indent}}}\n\
         \n\
         {indent}/// The raw value — the sanctioned exit hatch.\n\
         {indent}#[allow(clippy::clone_on_copy)] // the generic form: raw may or may not be Copy\n\
         {indent}pub fn get(&self) -> {raw} {{\n\
         {indent}    self.0.clone()\n\
         {indent}}}\n\
         \n\
         {mint}\
         }}\n\
         \n\
         {shaped}"
    )
}

#[crate::mutate]
fn emit_boundary(sys: &SystemDecl, m: &ModuleDecl) -> String {
    let owned: Vec<&ValueDecl> = sys
        .values
        .iter()
        .filter(|v| owner_of(sys, &v.name) == Some(m.name.as_str()))
        .collect();

    let mut out = format!(
        "//! `{m}`'s strict value-object surface — a DOOR by structure: its operator methods\n\
         //! front the interior workshop, which derives this file BOUNDARY (the tier-1 grammar).\n",
        m = m.name
    );
    out.push_str(&banner(&sys.name));
    out.push_str(
        "//!\n\
         //! The newtypes below are STRUCTURE; their validity rules are carried VERBATIM from the\n\
         //! declaration as meaning holes. Operator methods delegate inward — the tier-1 grammar\n\
         //! (no free functions, no public fields, no I/O) is enforced by this crate's build.rs.\n\n",
    );

    if !owned.is_empty() {
        out.push_str("use boundary_spec::boundary::Shaped;\n");
    }
    let imports = value_imports(sys, m, true);
    out.push_str(&imports);
    out.push('\n');

    for v in &owned {
        out.push_str(&emit_value_object(v));
    }

    if !owned.is_empty() {
        let names: Vec<&str> = owned.iter().map(|v| v.name.as_str()).collect();
        out.push_str(&format!(
            "boundary_spec::value_object!({});\n",
            names.join(", ")
        ));
    }

    // operator methods, grouped by receiver (the first input's value object).
    let with_inputs: Vec<&OpDecl> = m.ops.iter().filter(|op| !op.inputs.is_empty()).collect();
    if !with_inputs.is_empty() {
        out.push_str(
            "\n// ===== operator methods — mechanical delegation; the meaning lives in the\n\
             // interior workshop =====\n",
        );
        let mut receivers: Vec<&str> = Vec::new();
        for op in &with_inputs {
            let recv = op.inputs[0].as_str();
            if !receivers.contains(&recv) {
                receivers.push(recv);
            }
        }
        for recv in receivers {
            out.push_str(&format!("\nimpl {recv} {{\n"));
            let mut first = true;
            for op in with_inputs.iter().filter(|op| op.inputs[0] == recv) {
                if !first {
                    out.push('\n');
                }
                first = false;
                let params: Vec<String> = op.inputs[1..]
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!(", {}: {t}", ARG_NAMES[i + 1]))
                    .collect();
                let args: Vec<String> = (1..op.inputs.len())
                    .map(|i| format!(", {}", ARG_NAMES[i]))
                    .collect();
                let expected = expects_for(m, &op.name);
                let expect_doc = if expected.is_empty() {
                    String::new()
                } else {
                    let list: Vec<String> = expected.iter().map(|e| e.render()).collect();
                    format!(
                        "    ///\n    /// Declared expectations (the target lock restates \
                         them): {}.\n",
                        list.join("; ")
                    )
                };
                out.push_str(&format!(
                    "    /// `{op_name}` — declared `{op_name}({sig}) -> {ret}`, delegated to \
                     the interior\n    /// (`crate::{module}_internal::{op_name}`), where its \
                     MEANING hole lives.\n{expect_doc}    pub fn {op_name}(self{params}) -> \
                     {ret} {{\n        crate::{module}_internal::{op_name}(self{args})\n    \
                     }}\n",
                    op_name = op.name,
                    sig = op.inputs.join(", "),
                    ret = op.output,
                    module = m.name,
                    params = params.join(""),
                    args = args.join(""),
                ));
            }
            out.push_str("}\n");
        }
    }
    out
}

#[crate::mutate]
fn emit_internal(sys: &SystemDecl, m: &ModuleDecl) -> String {
    let mut out = format!(
        "//! The workshop `{m}`'s boundary delegates to — not pub-reachable, deriving INTERIOR\n\
         //! (the tier-2 inward rule holds: nothing here returns a raw primitive).\n",
        m = m.name
    );
    out.push_str(&banner(&sys.name));
    out.push_str(
        "//!\n\
         //! Every body below is THE meaning hole for its operator. The interior is free — raw\n\
         //! representation arithmetic, any style — but the only way back out is a value object's\n\
         //! `mint`, so a quantity cannot leave un-validated.\n\n",
    );
    out.push_str(&value_imports(sys, m, false));

    for op in m.ops.iter().filter(|op| !op.inputs.is_empty()) {
        let params: Vec<String> = op
            .inputs
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}: {t}", ARG_NAMES[i]))
            .collect();
        let expected = expects_for(m, &op.name);
        let expect_doc = if expected.is_empty() {
            "/// No laws were declared for it — whatever discovery finds is the spec.\n".to_string()
        } else {
            let list: Vec<String> = expected.iter().map(|e| e.render()).collect();
            format!(
                "/// Declared expectations to honour (discovery must re-earn them): {}.\n",
                list.join("; ")
            )
        };
        out.push_str(&format!(
            "\n/// `{op_name}` — the operator's interior.\n{expect_doc}\
             pub(crate) fn {op_name}({params}) -> {ret} {{\n    \
             todo!(\"MEANING: body of {op_name}({sig}) -> {ret}\")\n}}\n",
            op_name = op.name,
            params = params.join(", "),
            sig = op.inputs.join(", "),
            ret = op.output,
        ));
    }
    out
}

#[crate::mutate]
fn emit_ops(sys: &SystemDecl) -> String {
    let mut out = String::from(
        "//! The discovered-law layer: each declared module's operators, as the theory\n\
         //! `#[algebra]` synthesises from ordinary function signatures.\n",
    );
    out.push_str(&banner(&sys.name));
    out.push_str(
        "//!\n\
         //! Nothing about the algebras is asserted here. The `expects(...)` attribute argument\n\
         //! carries the DECLARED expectations through AS WRITTEN in the declaration; discovery\n\
         //! must re-earn them by running these functions, and the committed `spec/*.spec` target\n\
         //! locks stay red until it does.\n\n\
         use boundary_spec::algebra;\n",
    );
    for m in &sys.modules {
        let theory = camel(&m.name);
        let attr = if m.expects.is_empty() {
            format!("#[algebra({theory}, \"{}\")]", m.name)
        } else {
            let written: Vec<String> = m.expects.iter().map(Expectation::render).collect();
            format!(
                "#[algebra({theory}, \"{}\", expects({}))]",
                m.name,
                written.join(", ")
            )
        };
        out.push_str(&format!(
            "\n/// `{m}`'s operator surface for discovery: the theory (`{theory}`, named \
             \"{m}\") is what\n/// `#[algebra]` reads off the signatures below.\n{attr}\n\
             pub mod {m}_ops {{\n",
            m = m.name,
        ));
        // imports (indented one level inside the module) — PUBLIC re-exports, so the
        // full-grammar `system!` block can name every sort through the module the
        // declaration mentions (`crate::ops::meter_ops::Credits`).
        for line in value_imports(sys, m, false).lines() {
            out.push_str(&format!("    pub {line}\n"));
        }
        for op in &m.ops {
            if op.inputs.is_empty() {
                out.push_str(&format!(
                    "\n    /// The declared constant `{n}` — a MEANING hole: the witness the \
                     declared laws name.\n    pub fn {n}() -> {ret} {{\n        \
                     todo!(\"MEANING: the constant {n}() -> {ret}\")\n    }}\n",
                    n = op.name,
                    ret = op.output,
                ));
            } else {
                let params: Vec<String> = op
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}: {t}", ARG_NAMES[i]))
                    .collect();
                let args: Vec<&str> = (1..op.inputs.len()).map(|i| ARG_NAMES[i]).collect();
                out.push_str(&format!(
                    "\n    /// `{n}` — mechanical delegation to the boundary surface.\n    \
                     pub fn {n}({params}) -> {ret} {{\n        a.{n}({args})\n    }}\n",
                    n = op.name,
                    params = params.join(", "),
                    ret = op.output,
                    args = args.join(", "),
                ));
            }
        }
        out.push_str("}\n");
    }

    // one SPANNING theory per `via` transform seam: the two endpoint operators and the
    // crossing conversion in ONE operator table, so discovery can find — and the compiled
    // seam demand — the homomorphism law. Same mechanical delegation as the module theories.
    for s in sys.seams.iter().filter(|s| s.via.is_some()) {
        let (module_name, marker, display) = seam_theory_names(s);
        let (h, from, to) = via_seam_parts(sys, s);
        let (source, target) = (&s.on, &h.output);
        out.push_str(&format!(
            "\n/// The SPANNING theory for the `{l} -- {r}` transform seam: source operator, \
             conversion, and\n/// target operator in one table — the homomorphism expectation \
             rides the attribute, and the\n/// compiled seam (`src/system.rs`) holds discovery \
             to it.\n#[algebra({marker}, \"{display}\", expects(homomorphism({h}, {from}, \
             {to})))]\npub mod {module_name} {{\n",
            l = s.left,
            r = s.right,
            h = h.name,
            from = from.name,
            to = to.name,
        ));
        let mut imports = BTreeSet::new();
        for value in [source, target] {
            let owner = owner_of(sys, value).expect("validated: every value has an owner");
            imports.insert(format!("    pub use crate::{owner}::{value};\n"));
        }
        for line in &imports {
            out.push_str(line);
        }
        out.push_str(&format!(
            "\n    /// `{from}` — the source-side operator, delegated to the boundary surface.\n    \
             pub fn {from}(a: {v}, b: {v}) -> {v} {{\n        a.{from}(b)\n    }}\n\n    \
             /// `{h}` — the crossing conversion, delegated to the boundary surface.\n    \
             pub fn {h}(a: {v}) -> {w} {{\n        a.{h}()\n    }}\n\n    \
             /// `{to}` — the target-side operator, delegated to the boundary surface.\n    \
             pub fn {to}(a: {w}, b: {w}) -> {w} {{\n        a.{to}(b)\n    }}\n",
            from = from.name,
            h = h.name,
            to = to.name,
            v = source,
            w = target,
        ));
        out.push_str("}\n");
    }
    out
}

/// The TARGET lock: the DECLARED laws in the exact committed lock format
/// (`discover::freeze::render` — header, law lines, coverage line), so the generated crate's
/// drift gate is red until discovery re-derives precisely what was declared.
#[crate::mutate]
fn emit_target_lock(m: &ModuleDecl) -> String {
    let mut out = format!(
        "# discovered spec: {} — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.\n\n",
        m.name
    );
    // laws in the order a confirming discovery renders them: EQUATIONS first, WITNESS laws
    // last (the engine emits its inequation band after the whole equational one), each band
    // grouped by the operator the engine FIRES the shape on — uniformly the first declared
    // op, the shape's slot 0, since the generic driver replaced the hand-written battery —
    // then in catalog order, declaration order breaking ties. The dynamic sync test holds
    // this to the freeze's actual render.
    let fire_index = |e: &Expectation| {
        m.ops
            .iter()
            .position(|o| o.name == e.ops[0])
            .expect("validated: every named operator is declared")
    };
    let band = |e: &Expectation| match shape_info(e).polarity {
        Polarity::Equal => 0,
        Polarity::Differs => 1,
    };
    let mut declared: Vec<&Expectation> = m.expects.iter().collect();
    declared.sort_by_key(|e| (band(e), fire_index(e), shape_rank(e)));
    for e in declared {
        let (prose, equation) = law(e);
        out.push_str(&format!("- {prose}\n      {equation}\n"));
    }
    out.push('\n');
    let covered: BTreeSet<&str> = m
        .expects
        .iter()
        .flat_map(|e| e.ops.iter().map(String::as_str))
        .collect();
    let uncovered: Vec<&str> = m
        .ops
        .iter()
        .map(|o| o.name.as_str())
        .filter(|n| !covered.contains(n))
        .collect();
    let coverage = if uncovered.is_empty() {
        "none — every operator participates in a law".to_string()
    } else {
        uncovered.join(", ")
    };
    out.push_str(&format!(
        "# operators in no law (where the spec is silent): {coverage}\n"
    ));
    out
}

/// The compiled `system!` stage of the declaration — ONE artifact, two lifecycle stages
/// (see `discover::system`'s full-grammar form): the marker struct plus the ORIGINAL
/// declaration tokens, spliced VERBATIM into a `boundary_spec::system! { Marker: ... }`
/// invocation. The macro maps module names to their genesis-conventional theories, turns
/// every declared operator signature into a compile-time witness, discharges transport
/// seams by construction, and checks named transform seams in their spanning theories — so
/// declaration↔code drift is a COMPILE error whose message points back at the declaration.
/// A via-less transform seam is skipped by the macro; its hole lives in `tests/seams.rs`.
#[crate::mutate]
fn emit_system(sys: &SystemDecl, declaration: &str) -> String {
    let marker = camel(&sys.name);
    let holes = sys
        .seams
        .iter()
        .filter(|s| s.kind == SeamKindDecl::Transform && s.via.is_none())
        .count();

    let mut out =
        String::from("//! The compiled `system!` graph: ONE declaration, two lifecycle stages.\n");
    out.push_str(&banner(&sys.name));
    out.push_str(
        "//!\n\
         //! The block below is the ORIGINAL system declaration, spliced VERBATIM — the same\n\
         //! tokens genesis derived this whole crate from, now COMPILING: `modules()` (the\n\
         //! registry the freeze loop reads) resolves each module name to its theory, every\n\
         //! declared operator signature is a compile-time witness against the code, and the\n\
         //! seams wire to their checkers. Edit the declaration HERE and the crate follows or\n\
         //! fails to build — the declaration cannot go stale.\n",
    );
    if holes > 0 {
        out.push_str(&format!(
            "//!\n\
             //! NOTE: {holes} declared TRANSFORM seam(s) name no conversion (`via h`), so the\n\
             //! macro skips them — each remains a loud hole in `tests/seams.rs`. Name the\n\
             //! conversion in the declaration below to compile the check.\n"
        ));
    }
    out.push_str(&format!(
        "\n/// The system marker — `SystemReport::of::<{marker}>()` is the graph the lock \
         freezes.\npub struct {marker};\n\nboundary_spec::system! {{\n    {marker}:\n\
         {declaration}\n}}\n"
    ));
    out
}

/// The TARGET system lock: the declared graph in the exact committed lock format
/// (`discover::system::SystemReport::render` — keep the two in step; the render pin in the
/// tests holds this side). Transport seams are born discharged (by construction), so this
/// lock goes green the moment the crate compiles and discovery runs.
#[crate::mutate]
fn emit_system_lock(sys: &SystemDecl) -> String {
    let mut out = format!(
        "# system spec: {} — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.\n\n",
        sys.name
    );
    out.push_str("modules (the ratified registry — one committed module lock each):\n");
    for m in &sys.modules {
        out.push_str(&format!("- {}\n", m.name));
    }
    out.push('\n');
    let compiled: Vec<&SeamDecl> = sys
        .seams
        .iter()
        .filter(|s| s.kind == SeamKindDecl::Transport || s.via.is_some())
        .collect();
    if compiled.is_empty() {
        out.push_str("seams: none — no module pair declares a shared-value obligation.\n");
        return out;
    }
    out.push_str("seams (each edge: its obligation, then the verdict its checker returned):\n");
    for s in &compiled {
        match &s.via {
            None => {
                out.push_str(&format!(
                    "- {} -- {} : transport on {}\n",
                    s.left, s.right, s.on
                ));
                out.push_str(
                    "      obligation: the modules share this value and must agree on its laws\n",
                );
                out.push_str(
                    "      status: discharged by construction — the shared value is one type on both \
                     sides (the declaration carries the compile-time witness)\n",
                );
            }
            Some(via) => {
                // the CONVERGED target: once the conversion's meaning earns the declared
                // homomorphism, the live verdict renders exactly this (no composites — a
                // discovered chain shows up in the bless diff, which is the ratification).
                let (_, _, display) = seam_theory_names(s);
                let (h, from, to) = via_seam_parts(sys, s);
                let _ = h;
                out.push_str(&format!(
                    "- {} -- {} : transform on {}\n",
                    s.left, s.right, s.on
                ));
                out.push_str(
                    "      obligation: the conversion across the seam must be a homomorphism\n",
                );
                out.push_str(&format!(
                    "      status: preserved — the conversion `{via}` is a discovered homomorphism \
                     (spanning theory: {display}):\n"
                ));
                out.push_str(&format!(
                    "        * {via} turns {} into {}.\n",
                    from.name, to.name
                ));
            }
        }
    }
    out
}

/// The DECLARED-LAWS gate: one distance test per module that declares expectations —
/// `Distance::of` names exactly what is missing, so the gate is red WITH A WORKLIST until
/// the meaning earns the declaration. Emitted only when some module declares `expects`.
#[crate::mutate]
fn emit_expectations(sys: &SystemDecl) -> String {
    let krate = crate_ident(&sys.name);
    let mut out = String::from(
        "//! expectations — the DECLARED-LAWS gate: each module's DISTANCE from its declared\n\
         //! algebra (`Distance::of`), red until discovery earns every declared law.\n",
    );
    out.push_str(&banner(&sys.name));
    out.push_str(
        "//!\n\
         //! RED BY DESIGN until the meaning holes are filled: the failure message IS the\n\
         //! worklist (\"MISSING: ...\"). Surprises (discovered, never declared) do not fail —\n\
         //! ratify them into the declaration or refute the operator that produced them.\n\n\
         use boundary_spec::discover::expect::Distance;\n\n",
    );
    for m in sys.modules.iter().filter(|m| !m.expects.is_empty()) {
        out.push_str(&format!(
            "use {krate}::ops::{}_ops::{};\n",
            m.name,
            camel(&m.name)
        ));
    }
    for seam in sys.seams.iter().filter(|s| s.via.is_some()) {
        let (module_name, marker, _) = seam_theory_names(seam);
        out.push_str(&format!("use {krate}::ops::{module_name}::{marker};\n"));
    }
    for m in sys.modules.iter().filter(|m| !m.expects.is_empty()) {
        out.push_str(&format!(
            "\n/// `{m}` declares {n} law(s); the distance report names any that discovery has\n\
             /// not (yet) found true of the meaning.\n\
             #[test]\n\
             fn {m}_meets_its_declared_expectations() {{\n    \
             let distance = Distance::of::<{theory}>();\n    \
             assert!(distance.is_met(), \"{{}}\", distance.render());\n}}\n",
            m = m.name,
            n = m.expects.len(),
            theory = camel(&m.name),
        ));
    }
    for seam in sys.seams.iter().filter(|s| s.via.is_some()) {
        let (_, marker, display) = seam_theory_names(seam);
        out.push_str(&format!(
            "\n/// The `{display}` spanning theory declares the seam's homomorphism; red names\n\
             /// exactly the law the conversion has not yet earned.\n\
             #[test]\n\
             fn {l}_{r}_seam_meets_its_declared_expectations() {{\n    \
             let distance = Distance::of::<{marker}>();\n    \
             assert!(distance.is_met(), \"{{}}\", distance.render());\n}}\n",
            l = seam.left,
            r = seam.right,
        ));
    }
    out
}

#[crate::mutate]
fn emit_freeze_example(sys: &SystemDecl) -> String {
    let template = r#"//! freeze — the BLESS path: regenerate `spec/*.spec` from the live, discovered algebra.
//!
//!     cargo run --example freeze
//!
//! This is the ONE sanctioned writer of the lock files. Genesis committed TARGET locks (the
//! DECLARED laws and the declared seam graph); the drift gate stays red until discovery
//! matches them. Once the meaning holes are filled, run this and read the diff against the
//! targets — a clean diff means the code earned exactly what was declared; any other diff is
//! the conversation to have in review. Never edit a lock by hand.
//!
//! The lock list is READ OFF THE GRAPH (`@MARKER@::modules()` plus the system lock): the
//! declaration is the registry, so a module cannot silently fall out of the freeze loop.

use std::path::PathBuf;

use boundary_spec::discover::system::{System, SystemReport};
use @KRATE@::system::@MARKER@;

#[crate::mutate]
fn main() {
    let spec_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
    let mut locks: Vec<spec_lock::Lock> = @MARKER@::modules()
        .iter()
        .map(|spec| spec.lock_in(&spec_dir))
        .collect();
    locks.push(SystemReport::of::<@MARKER@>().lock_in(&spec_dir));
    spec_lock::bless(&locks).expect("write the spec locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
    println!("now diff spec/ against genesis's targets — that diff is the ratification.");
}
"#;
    template
        .replace("@KRATE@", &crate_ident(&sys.name))
        .replace("@MARKER@", &camel(&sys.name))
}

#[crate::mutate]
fn emit_freeze_gate(sys: &SystemDecl) -> String {
    let template = r#"//! freeze_gate — the DRIFT GATE, as a plain integration test.
//!
//! Re-derive the live discovered spec and hold it against the committed locks. Genesis
//! committed TARGET locks — the DECLARED laws and the declared seam graph, in the exact lock
//! format — so this gate is RED BY DESIGN until the meaning holes are filled and discovery
//! re-earns the declaration. The fix is never to hand-edit a lock: fill the meaning, run
//! `cargo run --example freeze`, and ratify the diff against the targets in review.

use std::path::PathBuf;

use boundary_spec::discover::system::{System, SystemReport};
use @KRATE@::system::@MARKER@;

#[crate::mutate]
fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

/// The committed locks are FRESH: the live discovered algebra (every module in the graph's
/// registry, and the graph itself) matches what was ratified — which, until the first bless,
/// is the DECLARED target.
#[test]
fn the_committed_specs_are_fresh() {
    let spec_dir = spec_dir();
    let mut locks: Vec<spec_lock::Lock> = @MARKER@::modules()
        .iter()
        .map(|spec| spec.lock_in(&spec_dir))
        .collect();
    locks.push(SystemReport::of::<@MARKER@>().lock_in(&spec_dir));
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "discovered spec differs from the committed lock for: {}. If discovery now matches \
             the declaration, run `cargo run --example freeze` and ratify the (empty) diff; \
             otherwise the meaning does not yet earn the declared laws.",
            stale.join(", ")
        );
    }
}
"#;
    template
        .replace("@KRATE@", &crate_ident(&sys.name))
        .replace("@MARKER@", &camel(&sys.name))
}

#[crate::mutate]
fn emit_gates_module(sys: &SystemDecl) -> String {
    let template = r#"//! A discovered-law / report layer (exempt from the inward rule).
//!
//! gates — THE PIPELINE IS A LOCK, the consumer form: this crate's CI is a declaration.
//!
//! GENERATED by genesis for `@APP@`. `spec/gates.spec` (the promises) and
//! `.github/workflows/ci.yml` (the execution) are BOTH rendered from the declaration
//! below and drift-gated byte for byte (`tests/gates.rs`), so the pipeline cannot be
//! hand-edited into silent decay — the motivating bug class upstream was a hand-written
//! workflow that quietly tested the wrong scope for its whole life. Regenerate with
//! `cargo run --example freeze_gates` and ratify the diff.

use boundary_spec::discover::gates::Pipeline;

/// The pipeline declaration, as a TYPESTATE — the crate's CI surface (every public
/// callable hangs off a typestate; this crate's own enforcement shim holds this file to
/// that rule too).
pub struct Ci;

#[crate::mutate]
impl Ci {
    /// This crate's pipeline: the STARTER declaration (format, lint, test — every
    /// change), pinned to the toolchain the upstream library is tested against. Outgrow
    /// it by replacing this call with a hand-declared `Pipeline { .. }` — per-diff
    /// mutation and weekly gates render too, and the locks drift-gate whichever
    /// declaration stands.
    pub fn pipeline() -> Pipeline {
        Pipeline::starter()
    }
}
"#;
    template.replace("@APP@", &sys.name)
}

#[crate::mutate]
fn emit_freeze_gates_example(sys: &SystemDecl) -> String {
    let template = r#"//! freeze_gates — regenerate the pipeline locks from this crate's gate declaration.
//!
//!     cargo run --example freeze_gates
//!
//! The ONE sanctioned writer of `spec/gates.spec` and `.github/workflows/ci.yml`. Both are
//! renders of `src/gates.rs`'s declaration; the committed diff is the ratification. Never
//! edit either by hand.

use std::path::PathBuf;

use @KRATE@::gates::Ci;

#[crate::mutate]
fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let locks = Ci::pipeline()
        .locks_in(&root)
        .expect("the declared pipeline renders");
    spec_lock::bless(&locks).expect("write the pipeline locks");
    for lock in &locks {
        println!("blessed `{}` -> {}", lock.name, lock.path.display());
    }
}
"#;
    template.replace("@KRATE@", &crate_ident(&sys.name))
}

#[crate::mutate]
fn emit_gates_gate(sys: &SystemDecl) -> String {
    let template = r#"//! gates — the pipeline drift gate: CI validates its own declaration on every run.

use std::path::PathBuf;

use @KRATE@::gates::Ci;

/// BOTH pipeline locks are fresh: the committed inventory and the committed workflow match
/// the declaration's renders — a hand edit to the YAML fails inside the very `cargo test`
/// the workflow runs.
#[test]
fn the_pipeline_locks_are_fresh() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let locks = Ci::pipeline()
        .locks_in(&root)
        .expect("the declared pipeline renders");
    if let Err(stale) = spec_lock::check(&locks) {
        panic!(
            "the pipeline drifted from its declaration: {}. Regenerate with \
             `cargo run --example freeze_gates` and ratify the diff — never edit the YAML \
             or the spec by hand.",
            stale.join(", ")
        );
    }
}

/// Nothing declared can fall out of execution: every gate's command reappears verbatim in
/// the rendered workflow.
#[test]
fn every_declared_gate_is_rendered_into_the_workflow() {
    let p = Ci::pipeline();
    let workflow = p.render_workflow().expect("the declared pipeline renders");
    for gate in &p.gates {
        assert!(
            workflow.contains(&gate.command_line()),
            "gate `{}` is declared but not executed by the workflow",
            gate.name
        );
    }
}
"#;
    template.replace("@KRATE@", &crate_ident(&sys.name))
}

#[crate::mutate]
fn emit_probes_stub(sys: &SystemDecl) -> String {
    let template = r#"//! probes — the edge-probe half of the contract: the build shim proves every edge HAS an
//! `impl Probed`; this file is where the probes RUN in CI.
//!
//! GENERATED by genesis for `@APP@` as a STUB: genesis emits no edges, because an edge (a
//! Construction's parse, a Branch's classification, a Guarded edge's witness) is meaning, not
//! structure. Mint the domain's edges on the boundary surface and probe them here — the
//! downstream-fixture's `tests/probes.rs` is the reference shape (entry parse, branch, guarded
//! chain, driven end to end).

/// MEANING: mint this system's edges and drive them. Until then this test fails loudly —
/// a skeleton with no observations is incomplete, not green.
#[test]
fn the_edges_are_probed() {
    todo!("MEANING: mint and probe this system's edges (see downstream-fixture/tests/probes.rs)")
}
"#;
    template.replace("@APP@", &sys.name)
}

#[crate::mutate]
fn emit_seams(sys: &SystemDecl) -> String {
    let krate = crate_ident(&sys.name);
    let mut out = format!(
        "//! seams — the DECLARED seam obligations, one test per seam.\n\
         //!\n\
         //! GENERATED by genesis for `{}`. A seam names what a module split must preserve:\n\
         //! a `transport` seam shares a sort that is the SAME type on both sides (preserved by\n\
         //! construction — genesis defines each value object exactly once); a `transform` seam\n\
         //! carries a conversion that must be a HOMOMORPHISM — checked by discovery in the\n\
         //! seam's spanning theory when the conversion is NAMED (`via h`), a meaning hole when\n\
         //! it is not.\n",
        sys.name
    );
    if sys.seams.iter().any(|s| s.via.is_some()) {
        out.push_str(&format!(
            "\nuse boundary_spec::discover::system::{{SeamKind, SystemReport}};\n\n\
             use {krate}::system::{};\n",
            camel(&sys.name)
        ));
    }
    for s in &sys.seams {
        let owner = owner_of(sys, &s.on).expect("validated: seam value has an owner");
        let test = format!(
            "seam_{}_{}_{}_on_{}",
            s.left,
            s.right,
            match s.kind {
                SeamKindDecl::Transport => "transport",
                SeamKindDecl::Transform => "transform",
            },
            s.on.to_lowercase()
        );
        match s.kind {
            SeamKindDecl::Transport => out.push_str(&format!(
                "\n/// TRANSPORT seam `{l} -- {r}` on `{v}`: both modules must use the one \
                 `{v}` type, so the\n/// algebra crosses the seam unchanged. Discharged by \
                 construction — `{v}` is defined once,\n/// in `{krate}::{owner}` — and \
                 witnessed here by a compile-time identity: if the sides ever\n/// diverge \
                 into two types, this stops compiling.\n#[test]\nfn {test}() {{\n    let \
                 _witness: fn({krate}::{owner}::{v}) -> {krate}::{owner}::{v} = |value| \
                 value;\n}}\n",
                l = s.left,
                r = s.right,
                v = s.on,
            )),
            SeamKindDecl::Transform => match &s.via {
                Some(via) => out.push_str(&format!(
                    "\n/// TRANSFORM seam `{l} -- {r}` on `{v}` via `{via}`: the conversion must \
                     be a HOMOMORPHISM,\n/// checked by DISCOVERY in the seam's spanning theory \
                     (`ops::{span}`) and verdict-bearing in\n/// the compiled graph — this test \
                     holds the verdict, and the system lock freezes it.\n#[test]\nfn {test}() \
                     {{\n    let seam = SystemReport::of::<{marker}>()\n        .seams\n        \
                     .into_iter()\n        .find(|s| (s.left, s.right, s.kind) == (\"{l}\", \
                     \"{r}\", SeamKind::Transform))\n        .expect(\"declared in \
                     src/system.rs\");\n    assert!(\n        seam.status.is_met(),\n        \
                     \"the conversion `{via}` does not yet preserve the algebra: {{:?}}\",\n        \
                     seam.status\n    );\n}}\n",
                    l = s.left,
                    r = s.right,
                    v = s.on,
                    span = seam_theory_names(s).0,
                    marker = camel(&sys.name),
                )),
                None => out.push_str(&format!(
                    "\n/// TRANSFORM seam `{l} -- {r}` on `{v}`: a conversion crosses it, and the \
                     conversion must be\n/// a HOMOMORPHISM — `h(a op b) == h(a) op' h(b)` — or \
                     the cut is bad. Naming `h` and the two\n/// operators is meaning; probing the \
                     equation over the grid is the discharge (or name the\n/// conversion in the \
                     declaration — `via h` — and regenerate to compile the check).\n#[test]\nfn \
                     {test}() {{\n    \
                     todo!(\"MEANING: name the conversion across `{l} -- {r}` and probe h(a op b) \
                     == h(a) op'(h(b))\")\n}}\n",
                    l = s.left,
                    r = s.right,
                    v = s.on,
                )),
            },
        }
    }
    out
}

// ===== the typestate =========================================================================

/// The generation plan: the validated declaration plus every file it derives — pure data, no
/// I/O performed or promised. Feed it to [`Genesis::apply`] to materialise.
pub struct Plan {
    /// The parsed, validated declaration.
    pub system: SystemDecl,
    /// The ORIGINAL declaration text (the tokens between `system! {` and `}`), verbatim —
    /// spliced into the generated `src/system.rs` so one artifact serves both lifecycle
    /// stages.
    pub declaration: String,
    /// Every file to write, target-root-relative, in emission order.
    pub edits: Vec<FileEdit>,
}

#[crate::mutate]
impl Plan {
    /// The target-root-relative paths this plan will write, in emission order.
    pub fn listing(&self) -> Vec<&str> {
        self.edits.iter().map(|e| e.path.as_str()).collect()
    }
}

/// `Genesis::apply`'s declared effect — the same confined-write discipline as the architect's
/// code actions (the write itself is `Architect::apply`, path-traversal rejection included).
pub const GENESIS_APPLY_CAPABILITY: crate::boundary::Capability =
    crate::boundary::Capability::Effectful;

/// The blank-slate generator, as a TYPESTATE — plan and apply hang off it per the no-rats-nest
/// rule (every public callable hangs off a typestate).
pub struct Genesis;

#[crate::mutate]
impl Genesis {
    /// The PURE half: parse the declaration source (`syn`, from the `system!` token stream),
    /// validate it, and derive every file. No I/O — the same plan can be inspected, tested, or
    /// applied.
    pub fn plan(declaration: &str, deps: &Deps) -> Result<Plan, String> {
        let (system, declaration_text) = parse_declaration(declaration)?;
        validate(&system)?;

        let mut edits = vec![
            FileEdit {
                path: "Cargo.toml".to_string(),
                contents: emit_cargo_toml(&system, deps),
            },
            FileEdit {
                path: "build.rs".to_string(),
                contents: emit_build_rs(&system),
            },
            FileEdit {
                path: "src/lib.rs".to_string(),
                contents: emit_lib_rs(&system),
            },
        ];
        for m in &system.modules {
            edits.push(FileEdit {
                path: format!("src/{}.rs", m.name),
                contents: emit_boundary(&system, m),
            });
            edits.push(FileEdit {
                path: format!("src/{}_internal.rs", m.name),
                contents: emit_internal(&system, m),
            });
        }
        edits.push(FileEdit {
            path: "src/ops.rs".to_string(),
            contents: emit_ops(&system),
        });
        edits.push(FileEdit {
            path: "src/system.rs".to_string(),
            contents: emit_system(&system, &declaration_text),
        });
        for m in &system.modules {
            edits.push(FileEdit {
                path: format!("spec/{}.spec", m.name),
                contents: emit_target_lock(m),
            });
        }
        edits.push(FileEdit {
            path: format!("spec/{}.system.spec", system.name),
            contents: emit_system_lock(&system),
        });
        edits.push(FileEdit {
            path: "examples/freeze.rs".to_string(),
            contents: emit_freeze_example(&system),
        });
        edits.push(FileEdit {
            path: "tests/freeze_gate.rs".to_string(),
            contents: emit_freeze_gate(&system),
        });
        if system.modules.iter().any(|m| !m.expects.is_empty())
            || system.seams.iter().any(|s| s.via.is_some())
        {
            edits.push(FileEdit {
                path: "tests/expectations.rs".to_string(),
                contents: emit_expectations(&system),
            });
        }
        edits.push(FileEdit {
            path: "tests/probes.rs".to_string(),
            contents: emit_probes_stub(&system),
        });
        if !system.seams.is_empty() {
            edits.push(FileEdit {
                path: "tests/seams.rs".to_string(),
                contents: emit_seams(&system),
            });
        }
        // THE PIPELINE: the declaration module (a call to the one starter — never a
        // restatement), its freeze example and drift gate, and the two initial artifacts
        // rendered HERE from the same starter the emitted module calls — so the generated
        // crate's pipeline locks are fresh from birth (no target-vs-earned gap: the
        // declaration fully determines the render).
        let pipeline = super::gates::Pipeline::starter();
        edits.push(FileEdit {
            path: "src/gates.rs".to_string(),
            contents: emit_gates_module(&system),
        });
        edits.push(FileEdit {
            path: "examples/freeze_gates.rs".to_string(),
            contents: emit_freeze_gates_example(&system),
        });
        edits.push(FileEdit {
            path: "tests/gates.rs".to_string(),
            contents: emit_gates_gate(&system),
        });
        edits.push(FileEdit {
            path: "spec/gates.spec".to_string(),
            contents: pipeline.render_registry(),
        });
        edits.push(FileEdit {
            path: ".github/workflows/ci.yml".to_string(),
            contents: pipeline
                .render_workflow()
                .expect("the starter pipeline renders"),
        });
        Ok(Plan {
            system,
            declaration: declaration_text,
            edits,
        })
    }

    /// The one EFFECTFUL edge: write every planned file under `root`, through
    /// `Architect::apply` — so the effect is CONFINED to `root` (absolute and `..` paths are
    /// rejected, not written) and the confinement is enforced machinery, not a fresh claim.
    ///
    /// Capability: Effectful — writes files to disk (confined to `root`; see
    /// `GENESIS_APPLY_CAPABILITY`).
    pub fn apply(plan: &Plan, root: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        let action = CodeAction {
            title: format!("genesis: materialise `{}`", plan.system.name),
            preferred: false,
            edits: plan
                .edits
                .iter()
                .map(|e| FileEdit {
                    path: e.path.clone(),
                    contents: e.contents.clone(),
                })
                .collect(),
        };
        Architect::apply(&action, root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed sample declaration — the same file the `genesis` example points at.
    const SAMPLE: &str = include_str!("../../examples/genesis_demo.rs");

    fn sample_plan() -> Plan {
        Genesis::plan(SAMPLE, &Deps::Path("../probe-algebra".to_string()))
            .expect("the committed sample declaration must plan")
    }

    /// The sample declaration derives EXACTLY this tree — the whole fixture shape, from one
    /// declaration. Pins the file list so a generation change is a reviewed diff here.
    #[test]
    fn the_sample_declaration_plans_the_fixture_shape() {
        let plan = sample_plan();
        assert_eq!(plan.system.name, "credit-app");
        assert_eq!(
            plan.listing(),
            vec![
                "Cargo.toml",
                "build.rs",
                "src/lib.rs",
                "src/meter.rs",
                "src/meter_internal.rs",
                "src/billing.rs",
                "src/billing_internal.rs",
                "src/ops.rs",
                "src/system.rs",
                "spec/meter.spec",
                "spec/billing.spec",
                "spec/credit-app.system.spec",
                "examples/freeze.rs",
                "tests/freeze_gate.rs",
                "tests/expectations.rs",
                "tests/probes.rs",
                "tests/seams.rs",
                "src/gates.rs",
                "examples/freeze_gates.rs",
                "tests/gates.rs",
                "spec/gates.spec",
                ".github/workflows/ci.yml",
            ]
        );
    }

    /// The generated PIPELINE is fresh from birth: the emitted artifacts are renders of
    /// the same starter declaration the emitted `src/gates.rs` calls — no target-vs-earned
    /// gap, because the declaration fully determines the render. (The starter's exact
    /// shape is byte-pinned in `discover::gates`; this pins the emission wiring.)
    #[test]
    fn the_generated_pipeline_is_fresh_from_birth() {
        let plan = sample_plan();
        let body = |path: &str| {
            &plan
                .edits
                .iter()
                .find(|e| e.path == path)
                .unwrap_or_else(|| panic!("`{path}` is planned"))
                .contents
        };
        let starter = crate::discover::gates::Pipeline::starter();
        assert_eq!(body("spec/gates.spec"), &starter.render_registry());
        assert_eq!(
            body(".github/workflows/ci.yml"),
            &starter.render_workflow().expect("the starter renders")
        );
        assert!(body("src/gates.rs").contains("Pipeline::starter()"));
        assert!(body("tests/gates.rs").contains("the_pipeline_locks_are_fresh"));
        assert!(body("src/lib.rs").contains("pub mod gates;\n"));
    }

    /// Every generated `.rs` file PARSES — the syn-clean contract. (Compiling is deliberately
    /// NOT promised: the holes are `todo!()`s, and the emitted build shim gates the first
    /// build on a census bless.)
    #[test]
    fn every_generated_rust_file_is_syn_clean() {
        for edit in &sample_plan().edits {
            if edit.path.ends_with(".rs") {
                syn::parse_file(&edit.contents).unwrap_or_else(|e| {
                    panic!(
                        "generated `{}` does not parse: {e}\n{}",
                        edit.path, edit.contents
                    )
                });
            }
        }
    }

    /// The TARGET lock is the declared laws in the EXACT committed lock format — byte-for-byte
    /// the render a confirming discovery would freeze, so a meaning that earns the declaration
    /// exactly turns the gate green with an empty bless diff. Spot-checks one whole file body.
    #[test]
    fn the_meter_target_lock_renders_the_declared_laws_exactly() {
        let plan = sample_plan();
        let lock = plan
            .edits
            .iter()
            .find(|e| e.path == "spec/meter.spec")
            .expect("meter lock");
        let expected = "\
# discovered spec: meter — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.

- grant gives the same result in either order.
      (x grant y) = (y grant x)
- grant with zero leaves a value unchanged.
      (zero grant x) = x
- With renew, the later operand wins where the two disagree — re-applying an earlier one cannot overwrite it.
      ((x renew y) renew x) = (y renew x)

# operators in no law (where the spec is silent): none — every operator participates in a law
";
        assert_eq!(lock.contents, expected);

        // billing declares no laws, so its target lock is silent — and says so in the
        // coverage line, exactly as the engine would render an undiscovered operator.
        let billing = plan
            .edits
            .iter()
            .find(|e| e.path == "spec/billing.spec")
            .expect("billing lock");
        assert!(billing
            .contents
            .ends_with("# operators in no law (where the spec is silent): charge\n"));
        assert!(!billing.contents.contains("- "), "no laws were declared");
    }

    /// The boundary surface carries the validity HOLE verbatim and the mechanical delegation;
    /// the ops file passes the declared expectations through AS WRITTEN. Spot-checks bodies.
    #[test]
    fn the_generated_bodies_carry_holes_and_delegation() {
        let plan = sample_plan();
        let body = |path: &str| {
            &plan
                .edits
                .iter()
                .find(|e| e.path == path)
                .unwrap_or_else(|| panic!("missing {path}"))
                .contents
        };

        let meter = body("src/meter.rs");
        assert!(meter.contains("pub struct Credits(i64);"));
        assert!(meter.contains("todo!(\"MEANING: validity of Credits — \\\"0..=20\\\"\")"));
        assert!(meter.contains("crate::meter_internal::grant(self, b)"));

        // billing's operator is a method ON the foreign value object, imported from its owner.
        let billing = body("src/billing.rs");
        assert!(billing.contains("use crate::meter::Credits;"));
        assert!(billing.contains("pub struct Receipt(String);"));
        assert!(billing.contains("crate::billing_internal::charge(self, b)"));

        let ops = body("src/ops.rs");
        assert!(ops.contains(
            "#[algebra(Meter, \"meter\", expects(commutative(grant), identity(grant, zero), \
             bias_later(renew)))]"
        ));
        assert!(ops.contains("#[algebra(Billing, \"billing\")]"));
        assert!(ops.contains("a.grant(b)"));

        // the transport seam is discharged by construction, as a compile-time witness.
        let seams = body("tests/seams.rs");
        assert!(seams.contains("fn seam_meter_billing_transport_on_credits()"));
        assert!(seams.contains("fn(credit_app::meter::Credits) -> credit_app::meter::Credits"));

        // the lock loop is read off the GRAPH — the registry, plus the system lock.
        let gate = body("tests/freeze_gate.rs");
        assert!(gate.contains("use credit_app::system::CreditApp;"));
        assert!(gate.contains("CreditApp::modules()"));
        assert!(gate.contains("SystemReport::of::<CreditApp>().lock_in(&spec_dir)"));
        let freeze = body("examples/freeze.rs");
        assert!(freeze.contains("CreditApp::modules()"));
        assert!(freeze.contains("SystemReport::of::<CreditApp>().lock_in(&spec_dir)"));

        // the expectations gate covers exactly the modules that declare laws.
        let expectations = body("tests/expectations.rs");
        assert!(expectations.contains("fn meter_meets_its_declared_expectations()"));
        assert!(expectations.contains("Distance::of::<Meter>()"));
        assert!(
            !expectations.contains("Billing"),
            "billing declares no laws — no distance gate for it"
        );
    }

    /// The boundary emission is CONDITIONAL grammar, pinned from both sides: a module that
    /// owns values gets the `Shaped` import and the `value_object!` registration; one that
    /// owns none gets neither (the sample cannot see this — both its modules own a value,
    /// so the sweep watched these guards flip unpunished). Each operator's doc restates its
    /// OWN declared expectations in catalog order, methods sit one blank line apart, and a
    /// system with no expectations and no via seams plans no expectations gate at all.
    #[test]
    fn the_boundary_emission_is_conditional_grammar_pinned_from_both_sides() {
        let plan = sample_plan();
        let meter = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/meter.rs")
            .expect("meter boundary")
            .contents;
        assert!(meter.contains("use boundary_spec::boundary::Shaped;"));
        assert!(meter.contains("boundary_spec::value_object!(Credits);"));
        // the doc must sit ON ITS OWN operator (adjacency, not mere presence — a flipped
        // filter swaps the lists between `grant` and `renew` while both strings still
        // appear somewhere in the file).
        assert!(meter.contains(
            "Declared expectations (the target lock restates them): commutative(grant); \
             identity(grant, zero).\n    pub fn grant("
        ));
        assert!(meter.contains(
            "Declared expectations (the target lock restates them): bias_later(renew).\n    \
             pub fn renew("
        ));
        // one blank line BETWEEN methods, none after the impl header.
        assert!(meter.contains("    }\n\n    /// `renew`"));
        assert!(!meter.contains("impl Credits {\n\n"));

        // the ownerless twin: `pay` touches only meter's value, so its boundary file must
        // carry NO registration grammar — and this lawless, seamless system plans no
        // `tests/expectations.rs`.
        let lean = Genesis::plan(
            r#"system! {
                name: "lean-app",
                values { Credits = i64 where "0..=20"; }
                modules {
                    meter { ops { grant(Credits, Credits) -> Credits; } }
                    pay { ops { spend(Credits, Credits) -> Credits; } }
                }
            }"#,
            &Deps::Version("0".into()),
        )
        .expect("the lean declaration must plan");
        let pay = &lean
            .edits
            .iter()
            .find(|e| e.path == "src/pay.rs")
            .expect("pay boundary")
            .contents;
        assert!(!pay.contains("Shaped"));
        assert!(!pay.contains("value_object!"));
        assert!(!lean.listing().contains(&"tests/expectations.rs"));
    }

    /// The captured declaration text is trimmed EDGE TO EDGE: exactly the author's tokens,
    /// with the newlines hugging the braces stripped and nothing else — so the splice sits
    /// flush under the marker line with the author's own indentation. Pinned at both edges
    /// (a trim that under- or over-strips changes the emitted block's first line).
    #[test]
    fn the_declaration_text_is_captured_edge_to_edge() {
        let plan = sample_plan();
        assert!(
            plan.declaration.starts_with("    name: \"credit-app\","),
            "no leading newline, indentation intact: {:?}",
            &plan.declaration[..40.min(plan.declaration.len())]
        );
        assert!(
            plan.declaration.ends_with('}'),
            "no trailing newline: {:?}",
            &plan.declaration[plan.declaration.len().saturating_sub(20)..]
        );
        // and therefore the splice sits flush under the marker line.
        let system = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/system.rs")
            .expect("system module")
            .contents;
        assert!(system.contains("    CreditApp:\n    name: \"credit-app\","));
    }

    /// The COMPILED `system!` stage is the ORIGINAL declaration, spliced VERBATIM after the
    /// marker — one artifact, two lifecycle stages: the sample's own tokens (its name line,
    /// its lowercase module names, its seam line) appear unchanged, and the ops modules
    /// PUB-re-export their sorts so the macro can name every type through the declaration.
    /// A via-less transform seam is skipped by the macro — pinned by the second declaration,
    /// whose doc note names the fix.
    #[test]
    fn the_compiled_system_declaration_is_emitted() {
        let plan = sample_plan();
        let system = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/system.rs")
            .expect("system module")
            .contents;
        assert!(system.contains("pub struct CreditApp;"));
        assert!(system.contains("boundary_spec::system! {\n    CreditApp:\n"));
        // the sample's ORIGINAL tokens, verbatim — not a re-render.
        assert!(system.contains("name: \"credit-app\","));
        assert!(system.contains("Credits = i64 where \"0..=20\";"));
        assert!(system.contains("meter -- billing : transport on Credits;"));
        let ops = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/ops.rs")
            .expect("ops")
            .contents;
        assert!(
            ops.contains("pub use crate::meter::Credits;"),
            "ops modules re-export their sorts for the macro's paths"
        );

        let lib = plan
            .edits
            .iter()
            .find(|e| e.path == "src/lib.rs")
            .expect("roster");
        assert!(lib.contents.contains("pub mod system;\n"));

        // a TRANSFORM seam is not compiled into the graph — it stays a tests/seams.rs hole,
        // and the emitted module says so.
        let decl =
            "system! { name: \"pipey\", values { V = i64 where \"any\"; W = i64 where \"any\"; } \
                    modules { a { ops { fst(V, V) -> V; } } b { ops { snd(V, W) -> W; } } } \
                    seams { a -- b : transform on V; } }";
        let plan = Genesis::plan(decl, &Deps::Version("0".to_string())).expect("plan");
        let system = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/system.rs")
            .expect("system module")
            .contents;
        assert!(system.contains("1 declared TRANSFORM seam(s) name no conversion"));
        // the verbatim block still CARRIES the seam tokens (the macro skips them) ...
        assert!(system.contains("a -- b : transform on V;"));
        // ... while the seam test hole is still emitted.
        assert!(plan.listing().contains(&"tests/seams.rs"));
    }

    /// A transform seam with a NAMED conversion (`via h`) is compiled END TO END: the
    /// spanning theory (source op, conversion, target op, homomorphism expectation) lands in
    /// `src/ops.rs`; the compiled seam wires it in `src/system.rs`; the expectations gate
    /// covers the seam theory; `tests/seams.rs` holds the verdict instead of a hole; and the
    /// system target lock renders the PRESERVED stanza (kept in step with
    /// `SystemReport::render` — the byte pin here is the sync point).
    #[test]
    fn a_named_conversion_compiles_the_transform_seam_end_to_end() {
        let decl = "system! { name: \"pipe-app\", \
                    values { Raw = i64 where \"any\"; Cooked = i64 where \"any\"; } \
                    modules { \
                    source { ops { blend(Raw, Raw) -> Raw; cook(Raw) -> Cooked; } } \
                    sink { ops { fuse(Cooked, Cooked) -> Cooked; } } } \
                    seams { source -- sink : transform on Raw via cook; } }";
        let plan = Genesis::plan(decl, &Deps::Version("0".to_string())).expect("plan");
        let body = |path: &str| {
            &plan
                .edits
                .iter()
                .find(|e| e.path == path)
                .unwrap_or_else(|| panic!("missing {path}"))
                .contents
        };

        let ops = body("src/ops.rs");
        assert!(ops.contains(
            "#[algebra(SourceSinkSeam, \"source-sink seam\", \
             expects(homomorphism(cook, blend, fuse)))]"
        ));
        assert!(ops.contains("pub mod source_sink_seam_ops {"));
        assert!(ops.contains("pub fn cook(a: Raw) -> Cooked {"));
        assert!(ops.contains("pub fn fuse(a: Cooked, b: Cooked) -> Cooked {"));

        let system = body("src/system.rs");
        assert!(
            system.contains("PipeApp:"),
            "the marker heads the verbatim block"
        );
        assert!(
            system.contains("source -- sink : transform on Raw via cook;"),
            "the ORIGINAL seam tokens are spliced verbatim — the macro derives the spanning \
             theory path itself"
        );
        assert!(
            !system.contains("name no conversion"),
            "no hole note — the seam compiled"
        );

        let expectations = body("tests/expectations.rs");
        assert!(expectations.contains("use pipe_app::ops::source_sink_seam_ops::SourceSinkSeam;"));
        assert!(expectations.contains("fn source_sink_seam_meets_its_declared_expectations()"));
        assert!(expectations.contains("Distance::of::<SourceSinkSeam>()"));

        let seams = body("tests/seams.rs");
        assert!(seams.contains("fn seam_source_sink_transform_on_raw()"));
        assert!(seams
            .contains("(s.left, s.right, s.kind) == (\"source\", \"sink\", SeamKind::Transform)"));
        assert!(
            !seams.contains("todo!"),
            "the verdict test replaced the hole"
        );

        let lock = body("spec/pipe-app.system.spec");
        let expected = "\
# system spec: pipe-app — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.

modules (the ratified registry — one committed module lock each):
- source
- sink

seams (each edge: its obligation, then the verdict its checker returned):
- source -- sink : transform on Raw
      obligation: the conversion across the seam must be a homomorphism
      status: preserved — the conversion `cook` is a discovered homomorphism (spanning theory: source-sink seam):
        * cook turns blend into fuse.
";
        assert_eq!(lock, expected);

        for edit in &plan.edits {
            if edit.path.ends_with(".rs") {
                syn::parse_file(&edit.contents)
                    .unwrap_or_else(|e| panic!("generated `{}` does not parse: {e}", edit.path));
            }
        }
    }

    /// Malformed `via` declarations are refused by name: an undeclared conversion, a
    /// non-unary one, a wrong source, a transform landing where it left, a missing endpoint
    /// binary, and a transport claiming a conversion.
    #[test]
    fn malformed_via_seams_are_refused_by_name() {
        let wrap = |seam: &str| {
            format!(
                "system! {{ name: \"app\", \
                 values {{ V = i64 where \"any\"; W = i64 where \"any\"; }} \
                 modules {{ \
                 a {{ ops {{ blend(V, V) -> V; cook(V) -> W; mix(V, V) -> V; }} }} \
                 b {{ ops {{ fuse(W, W) -> W; }} }} }} \
                 seams {{ {seam} }} }}"
            )
        };
        let cases: Vec<(String, &str)> = vec![
            (
                wrap("a -- b : transform on V via boil;"),
                "neither module declares",
            ),
            (wrap("a -- b : transform on V via blend;"), "must be unary"),
            (
                wrap("a -- b : transform on W via cook;"),
                "converts from `V`, but the seam is on `W`",
            ),
            (
                wrap("a -- b : transform on V via cook;"),
                "exactly one homogeneous binary on `V`",
            ),
            (
                wrap("a -- b : transport on V via cook;"),
                "only a transform seam names a conversion",
            ),
        ];
        for (decl, expected) in cases {
            let err = Genesis::plan(&decl, &Deps::Version("0".to_string()))
                .err()
                .unwrap_or_else(|| panic!("should refuse: {decl}"));
            assert!(
                err.contains(expected),
                "expected `{expected}` in the refusal, got: {err}"
            );
        }
        // a self-landing conversion is refused too.
        let decl = "system! { name: \"app\", values { V = i64 where \"any\"; } \
                    modules { a { ops { blend(V, V) -> V; spin(V) -> V; } } \
                    b { ops { merge(V, V) -> V; } } } \
                    seams { a -- b : transform on V via spin; } }";
        let err = Genesis::plan(decl, &Deps::Version("0".to_string()))
            .err()
            .unwrap();
        assert!(err.contains("must land on a DIFFERENT value"), "got: {err}");
    }

    /// The TARGET system lock is the declared graph in the EXACT committed lock format —
    /// byte-for-byte what `SystemReport::render` produces once the crate compiles, so the
    /// system gate goes green with an empty bless diff. (Keep in step with
    /// `discover::system::SystemReport::render` — this pin is the sync point.)
    #[test]
    fn the_system_target_lock_renders_the_declared_graph_exactly() {
        let plan = sample_plan();
        let lock = plan
            .edits
            .iter()
            .find(|e| e.path == "spec/credit-app.system.spec")
            .expect("system lock");
        let expected = "\
# system spec: credit-app — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.

modules (the ratified registry — one committed module lock each):
- meter
- billing

seams (each edge: its obligation, then the verdict its checker returned):
- meter -- billing : transport on Credits
      obligation: the modules share this value and must agree on its laws
      status: discharged by construction — the shared value is one type on both sides (the declaration carries the compile-time witness)
";
        assert_eq!(lock.contents, expected);
    }

    /// HONESTY: every hole is loud and greppable — each `todo!(` in the generated tree opens a
    /// `MEANING:` message, and there is at least one per meaning site (validity, mint, grid,
    /// interior body, constant, probes).
    #[test]
    fn every_generated_hole_is_a_greppable_meaning_marker() {
        let mut holes = 0;
        for edit in &sample_plan().edits {
            if !edit.path.ends_with(".rs") {
                continue;
            }
            for (i, _) in edit.contents.match_indices("todo!(") {
                holes += 1;
                let tail = &edit.contents[i..];
                assert!(
                    tail.starts_with("todo!(\"MEANING:"),
                    "`{}` has a todo!() without a MEANING label: {}",
                    edit.path,
                    &tail[..tail.len().min(60)]
                );
            }
        }
        assert!(
            holes >= 10,
            "the sample should carry many holes, got {holes}"
        );
    }

    /// A STRUCTURED rule generates the transcription prose leaves as holes: the range
    /// yields the predicate and the edge-seeking grid, and the declared `saturating` policy
    /// yields the clamping `mint` — so the value object carries NO meaning holes at all,
    /// while the operator interiors (genuine meaning) still do. The emitted file stays
    /// syn-clean.
    #[test]
    fn a_structured_rule_generates_what_prose_leaves_as_holes() {
        let decl = "system! { name: \"cap\", \
                    values { C = i64 where 0..=20 saturating; } \
                    modules { m { ops { zero() -> C; pool(C, C) -> C; } \
                    expects { commutative(pool); identity(pool, zero); } } } }";
        let plan = Genesis::plan(decl, &Deps::Version("0".to_string())).expect("plan");
        let boundary = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/m.rs")
            .expect("boundary")
            .contents;

        assert!(boundary.contains("(0..=20).contains(&raw).then_some(C(raw))"));
        assert!(boundary.contains("C(raw.clamp(0, 20))"));
        assert!(
            boundary.contains("C(0)"),
            "the grid seeds at the lower edge"
        );
        assert!(boundary.contains("vec![C(self.0.saturating_add(1).clamp(0, 20)), C(20)],"));
        assert!(boundary.contains("vec![C(self.0.saturating_sub(1).clamp(0, 20)), C(0)],"));
        // no holes in CODE lines — the banner's prose legitimately quotes the marker.
        let code_holes = boundary
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .filter(|line| line.contains("todo!"))
            .count();
        assert_eq!(
            code_holes, 0,
            "a fully-structured value object owes nothing: {boundary}"
        );
        syn::parse_file(boundary).expect("the generated boundary parses");

        // the genuine meaning is still owed: the operator interiors and the constant.
        let interior = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/m_internal.rs")
            .expect("interior")
            .contents;
        assert!(interior.contains("todo!(\"MEANING: body of pool(C, C) -> C\")"));

        // negative bounds render as declared (signed raw).
        let signed = "system! { name: \"deg\", values { T = i64 where -5..=5 saturating; } \
                      modules { m { ops { f(T, T) -> T; } } } }";
        let plan = Genesis::plan(signed, &Deps::Version("0".to_string())).expect("plan");
        let boundary = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/m.rs")
            .unwrap()
            .contents;
        assert!(boundary.contains("(-5..=5).contains(&raw)"));
        assert!(boundary.contains("T(raw.clamp(-5, 5))"));
    }

    /// A range WITHOUT a policy generates the predicate and grid but keeps `mint` a hole —
    /// the range is declared; the re-entry discipline (clamp vs reject vs panic) is meaning.
    #[test]
    fn an_unpoliced_range_keeps_the_reentry_hole() {
        let decl = "system! { name: \"cap\", values { C = i64 where 0..=20; } \
                    modules { m { ops { pool(C, C) -> C; } } } }";
        let plan = Genesis::plan(decl, &Deps::Version("0".to_string())).expect("plan");
        let boundary = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/m.rs")
            .unwrap()
            .contents;
        assert!(
            boundary.contains("(0..=20).contains(&raw)"),
            "predicate generated"
        );
        assert!(
            boundary.contains("todo!(\"MEANING: interior constructor for C (re-enter 0..=20)\")"),
            "mint stays a hole"
        );
        assert!(boundary.contains("Declare `saturating` to generate the clamp"));
        assert!(
            boundary.contains("C(self.0.saturating_add(1)"),
            "grid still generated"
        );
    }

    /// The raw representation is any Rust TYPE, rendered back as written — the v1
    /// plain-path restriction is gone.
    #[test]
    fn raw_types_are_arbitrary_types() {
        let decl = "system! { name: \"blob\", \
                    values { R = Vec<u8> where \"non-empty\"; P = (i64, u8) where \"any\"; } \
                    modules { m { ops { join(R, R) -> R; tag(R, P) -> R; } } } }";
        let plan = Genesis::plan(decl, &Deps::Version("0".to_string())).expect("plan");
        let boundary = &plan
            .edits
            .iter()
            .find(|e| e.path == "src/m.rs")
            .unwrap()
            .contents;
        assert!(boundary.contains("pub struct R(Vec<u8>);"));
        assert!(boundary.contains("pub struct P((i64, u8));"));
        assert!(boundary.contains("pub fn new(raw: Vec<u8>) -> Option<R>"));
        syn::parse_file(boundary).expect("generic raws still emit parseable Rust");
    }

    /// Structured rules are validated at plan time, each refusal naming the fault: a range
    /// on a non-integer raw, an empty range, a negative bound on an unsigned raw, and an
    /// unknown re-entry policy.
    #[test]
    fn malformed_structured_rules_are_refused_by_name() {
        let wrap = |values: &str| {
            format!("system! {{ name: \"app\", values {{ {values} }} modules {{ m {{ ops {{ f(V, V) -> V; }} }} }} }}")
        };
        let cases: Vec<(String, &str)> = vec![
            (wrap("V = String where 0..=20;"), "not a primitive integer"),
            (wrap("V = i64 where 5..=1;"), "empty range"),
            (wrap("V = u8 where -1..=5;"), "unsigned"),
            (
                wrap("V = i64 where 0..=20 clamping;"),
                "unknown re-entry policy",
            ),
        ];
        for (decl, expected) in cases {
            let err = Genesis::plan(&decl, &Deps::Version("0".to_string()))
                .err()
                .unwrap_or_else(|| panic!("should refuse: {decl}"));
            assert!(
                err.contains(expected),
                "expected `{expected}` in the refusal, got: {err}"
            );
        }
    }

    // ===== the FULL-VOCABULARY sync: genesis's declared-law render IS the freeze's ==========
    //
    // Three fixture theories mirroring what `#[algebra]` emits (operator name == symbol,
    // default `x`/`y`/`z` variables, binaries infix, unaries prefix), chosen so every
    // newly-declarable shape fires: a lattice (distributivity, absorption), a two-sort
    // conversion domain (involution, round-trip, homomorphism), and an action domain
    // (action identity, monoid action, self-application). The test below declares EVERY law
    // discovery finds and demands the emitted target lock equal the freeze's render byte
    // for byte — order, prose, and equations at once.

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct L;
    struct Latt;
    fn and_op(v: &[bool]) -> Option<bool> {
        Some(v[0] && v[1])
    }
    fn or_op(v: &[bool]) -> Option<bool> {
        Some(v[0] || v[1])
    }
    crate::theory! {
        Latt : "latt", Value = bool, Obs = bool, Sort = L,
        sort_of = |_: &bool| L,
        observe = |v: &bool| *v,
        vars { L => &["x", "y", "z"], }
        inhabit { L => vec![false, true], }
        ops {
            Infix "and" "and" (L, L) -> L = and_op;
            Infix "or"  "or"  (L, L) -> L = or_op;
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum C2 {
        A,
        B,
    }
    struct Conv;
    fn cat_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, v[0].1.max(v[1].1)))
    }
    fn glue_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1.max(v[1].1)))
    }
    fn esc_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1))
    }
    fn unesc_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, v[0].1))
    }
    fn twist_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, -v[0].1))
    }
    crate::theory! {
        Conv : "conv", Value = (u8, i64), Obs = (u8, i64), Sort = C2,
        sort_of = |v: &(u8, i64)| if v.0 == 0 { C2::A } else { C2::B },
        observe = |v: &(u8, i64)| *v,
        vars { C2::A => &["x", "y", "z"], C2::B => &["x", "y", "z"], }
        inhabit {
            C2::A => vec![(0, -1), (0, 0), (0, 1)],
            C2::B => vec![(1, -1), (1, 0), (1, 1)],
        }
        ops {
            Infix  "cat"   "cat"   (C2::A, C2::A) -> C2::A = cat_op;
            Infix  "glue"  "glue"  (C2::B, C2::B) -> C2::B = glue_op;
            Prefix "esc"   "esc"   (C2::A) -> C2::B = esc_op;
            Prefix "unesc" "unesc" (C2::B) -> C2::A = unesc_op;
            Prefix "twist" "twist" (C2::A) -> C2::A = twist_op;
        }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum SC {
        C,
        P,
    }
    struct Score;
    fn bump_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, v[0].1 + v[1].1))
    }
    fn plus_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1 + v[1].1))
    }
    fn unit_op(_: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, 0))
    }
    fn cmp_op(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1 - v[1].1))
    }
    crate::theory! {
        Score : "score", Value = (u8, i64), Obs = (u8, i64), Sort = SC,
        sort_of = |v: &(u8, i64)| if v.0 == 0 { SC::C } else { SC::P },
        observe = |v: &(u8, i64)| *v,
        vars { SC::C => &["x", "y", "z"], SC::P => &["x", "y", "z"], }
        inhabit {
            SC::C => vec![(0, 0), (0, 1), (0, 2)],
            SC::P => vec![(1, 0), (1, 1), (1, 2)],
        }
        ops {
            Infix   "bump" "bump" (SC::C, SC::P) -> SC::C = bump_op;
            Infix   "plus" "plus" (SC::P, SC::P) -> SC::P = plus_op;
            Nullary "unit" "unit" () -> SC::P = unit_op;
            Infix   "cmp"  "cmp"  (SC::C, SC::C) -> SC::P = cmp_op;
        }
    }

    /// THE SYNC PIN for the whole declarable vocabulary: declare every law discovery finds
    /// over a fixture, and the emitted TARGET lock must equal the freeze's live render byte
    /// for byte — same laws, same prose (the catalog template), same equations (the engine's
    /// canonical render), same ORDER (the fire-operator emulation). Any drift between
    /// genesis's renderer and the engine's fails here, per shape, by name.
    #[test]
    fn the_target_lock_reproduces_discovery_byte_for_byte() {
        use crate::discover::engine::{Engine, ShapeCatalog, Theory};

        fn check<T: Theory>(expect_shapes: &[&str]) {
            let engine = Engine::<T>::new();
            let sigs = engine.signatures();
            let symbols: Vec<&'static str> = sigs.iter().map(|(s, _, _)| *s).collect();
            let inventory = ShapeCatalog::inventory();
            let mut expects = Vec::new();
            let mut fired: Vec<&str> = Vec::new();
            for law in engine.discover().laws {
                let ops = law.ops(&symbols);
                let slots = inventory
                    .iter()
                    .find(|i| i.name == law.shape)
                    .expect("ratified shape")
                    .gate_slots
                    .slots
                    .len();
                assert_eq!(
                    ops.len(),
                    slots,
                    "fixture law `{}` coincides slots — pick a cleaner fixture",
                    law.prose
                );
                fired.push(law.shape);
                expects.push(Expectation {
                    shape: law.shape,
                    ops: ops.into_iter().map(String::from).collect(),
                });
            }
            for shape in expect_shapes {
                assert!(
                    fired.contains(shape),
                    "fixture `{}` was chosen to fire `{shape}`, but it did not",
                    T::name()
                );
            }
            let module = ModuleDecl {
                name: T::name().to_string(),
                ops: sigs
                    .iter()
                    .map(|(symbol, inputs, output)| OpDecl {
                        name: symbol.to_string(),
                        inputs: inputs.iter().map(|s| format!("{s:?}")).collect(),
                        output: format!("{output:?}"),
                    })
                    .collect(),
                expects,
            };
            assert_eq!(
                emit_target_lock(&module),
                crate::discover::Spec::of::<T>()
                    .lock_in(Path::new("spec"))
                    .live,
                "genesis's declared-law render diverged from the freeze for `{}`",
                T::name()
            );
        }

        check::<Latt>(&["distributivity", "absorption"]);
        check::<Conv>(&["involution", "round-trip", "homomorphism"]);
        check::<Score>(&["action identity", "monoid action", "self-application"]);
    }

    /// The dependency flag switches the manifest between a path-parameterised checkout and
    /// registry versions — the two consumer stories.
    #[test]
    fn the_deps_flag_parameterises_the_manifest() {
        let by_path = sample_plan();
        let manifest = &by_path.edits[0].contents;
        assert!(manifest.contains("boundary-spec = { path = \"../probe-algebra\" }"));
        assert!(manifest.contains("spec-lock = { path = \"../probe-algebra/spec-lock\" }"));

        let by_version = Genesis::plan(SAMPLE, &Deps::Version("0.1.0".to_string())).expect("plan");
        let manifest = &by_version.edits[0].contents;
        assert!(manifest.contains("boundary-spec = \"0.1.0\""));
        assert!(manifest.contains("boundary-enforce = \"0.1.0\""));
    }

    /// An incoherent declaration is REFUSED with a message naming the fault — generation never
    /// runs over an unvalidated system. One probe per validation family.
    #[test]
    fn incoherent_declarations_are_refused_by_name() {
        let wrap = |body: &str| format!("system! {{ name: \"app\", {body} }}");
        let cases: Vec<(String, &str)> = vec![
            (
                wrap(
                    "values { V = i64 where \"any\"; } modules { m { ops { f(V, W) -> V; } } }",
                ),
                "not a declared value",
            ),
            (
                wrap(
                    "values { V = i64 where \"any\"; } modules { m { ops { f(V) -> V; } \
                     expects { commutative(f); } } }",
                ),
                "homogeneous binary",
            ),
            (
                wrap(
                    "values { V = i64 where \"any\"; } modules { m { ops { f(V, V) -> V; } \
                     expects { identity(f, zero); } } }",
                ),
                "which module `m` does not declare",
            ),
            (
                wrap(
                    "values { V = i64 where \"any\"; W = i64 where \"any\"; } modules { m { \
                     ops { f(V, V) -> V; wrong(W, W) -> W; unit() -> W; } \
                     expects { identity(f, unit); } } }",
                ),
                "`unit` must be a nullary constant of the matching sort",
            ),
            (
                wrap(
                    "values { V = i64 where \"any\"; } modules { m { ops { f(V, V) -> V; } } } \
                     seams { m -- n : transport on V; }",
                ),
                "not a declared module",
            ),
            (
                wrap("values { V = i64 where \"any\"; W = i64 where \"any\"; } modules { m { ops { f(V, V) -> V; } } }"),
                "used by no operator",
            ),
        ];
        for (decl, expected) in cases {
            let err = Genesis::plan(&decl, &Deps::Version("0".to_string()))
                .err()
                .unwrap_or_else(|| panic!("should refuse: {decl}"));
            assert!(
                err.contains(expected),
                "expected `{expected}` in the refusal, got: {err}"
            );
        }
        // and a file with no system! block at all is refused up front.
        let err = Genesis::plan("fn main() {}", &Deps::Version("0".to_string()))
            .err()
            .unwrap();
        assert!(err.contains("no `system!"));
    }

    /// `apply` materialises the plan on disk, CONFINED to the root (the architect's
    /// path-traversal rejection is the enforcement) — and what lands is what was planned.
    #[test]
    fn apply_writes_the_planned_tree_confined_to_root() {
        let plan = sample_plan();
        let dir = std::env::temp_dir().join(format!("genesis-test-{}", std::process::id()));
        let written = Genesis::apply(&plan, &dir).expect("write the tree");
        assert_eq!(written.len(), plan.edits.len());
        assert!(
            written.iter().all(|p| p.starts_with(&dir)),
            "a write escaped the root: {written:?}"
        );
        assert_eq!(
            GENESIS_APPLY_CAPABILITY,
            crate::boundary::Capability::Effectful
        );
        let lock = std::fs::read_to_string(dir.join("spec/meter.spec")).expect("read back");
        assert!(lock.contains("grant gives the same result in either order."));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The naming helpers are EXACT — a deaf `String::new()` empties the bless variable,
    /// the doc line, or the banner, and each is pinned here.
    #[test]
    fn the_naming_helpers_are_exact() {
        assert_eq!(bless_env("credit-app"), "BLESS_CREDIT_APP_QUALIFY");
        assert_eq!(bless_tiers_env("credit-app"), "BLESS_CREDIT_APP_TIERS");
        assert_eq!(doc_safe("a\nb\nc"), "a b c");
        assert!(banner("credit-app").contains("GENERATED by genesis from the `credit-app`"));
    }

    /// `is_binary_on_value` is TRUE for exactly a two-input operator whose inputs and
    /// output are all the value — every single-fact departure is false, so no `&&` can
    /// weaken to `||` unseen.
    #[test]
    fn is_binary_on_value_needs_every_slot() {
        let op = |inputs: &[&str], output: &str| OpDecl {
            name: "f".to_string(),
            inputs: inputs.iter().map(|s| s.to_string()).collect(),
            output: output.to_string(),
        };
        assert!(is_binary_on_value(&op(&["V", "V"], "V"), "V"));
        assert!(!is_binary_on_value(&op(&["V"], "V"), "V"), "arity");
        assert!(
            !is_binary_on_value(&op(&["W", "V"], "V"), "V"),
            "first input"
        );
        assert!(
            !is_binary_on_value(&op(&["V", "W"], "V"), "V"),
            "second input"
        );
        assert!(!is_binary_on_value(&op(&["V", "V"], "W"), "V"), "output");
    }

    /// The emitted file BODIES carry their content — a deaf emitter returns `""`, which
    /// still parses (so `every_generated_rust_file_is_syn_clean` misses it); this pins
    /// the characteristic prose of each.
    #[test]
    fn the_emitted_bodies_carry_their_content() {
        let plan = sample_plan();
        let body = |path: &str| {
            plan.edits
                .iter()
                .find(|e| e.path == path)
                .unwrap_or_else(|| panic!("`{path}` is planned"))
                .contents
                .as_str()
        };
        assert!(body("build.rs").contains("the enforcement shim genesis emitted"));
        assert!(body("tests/probes.rs").contains("the edge-probe half of the contract"));
        assert!(body("examples/freeze_gates.rs").contains("regenerate the pipeline locks"));
    }

    /// A minimal VALID system — the baseline the rejection battery perturbs.
    fn valid_system() -> SystemDecl {
        SystemDecl {
            name: "app".to_string(),
            values: vec![ValueDecl {
                name: "V".to_string(),
                raw: "u8".to_string(),
                rule: Rule::Prose("a value".to_string()),
            }],
            modules: vec![ModuleDecl {
                name: "core".to_string(),
                ops: vec![OpDecl {
                    name: "combine".to_string(),
                    inputs: vec!["V".to_string(), "V".to_string()],
                    output: "V".to_string(),
                }],
                expects: vec![],
            }],
            seams: vec![],
        }
    }

    /// `validate` REJECTS every ill-formed system: the baseline is accepted, and each
    /// single-fact corruption is refused. The name/reserved/duplicate guards are `||`/`==`
    /// chains, so a battery that trips each operand in turn is what keeps a flipped
    /// connective from silently admitting a malformed declaration.
    #[test]
    fn validate_rejects_every_malformed_system() {
        assert!(validate(&valid_system()).is_ok(), "the baseline is valid");

        let reject = |mutate: &dyn Fn(&mut SystemDecl), why: &str| {
            let mut sys = valid_system();
            mutate(&mut sys);
            assert!(validate(&sys).is_err(), "should reject: {why}");
        };

        // crate name (the `is_empty() || bad-first || bad-chars` chain):
        reject(&|s| s.name = String::new(), "empty name");
        reject(&|s| s.name = "1app".to_string(), "digit-first name");
        reject(&|s| s.name = "a b".to_string(), "space in name");
        // the shape floors:
        reject(&|s| s.values.clear(), "no values");
        reject(&|s| s.modules.clear(), "no modules");
        // duplicate value / module / operator names:
        reject(
            &|s| {
                s.values.push(ValueDecl {
                    name: "V".to_string(),
                    raw: "u8".to_string(),
                    rule: Rule::Prose("dup".to_string()),
                })
            },
            "duplicate value",
        );
        reject(
            &|s| {
                s.modules.push(ModuleDecl {
                    name: "core".to_string(),
                    ops: vec![OpDecl {
                        name: "other".to_string(),
                        inputs: vec!["V".to_string()],
                        output: "V".to_string(),
                    }],
                    expects: vec![],
                })
            },
            "duplicate module",
        );
        reject(
            &|s| {
                s.modules[0].ops.push(OpDecl {
                    name: "combine".to_string(),
                    inputs: vec!["V".to_string()],
                    output: "V".to_string(),
                })
            },
            "duplicate operator",
        );
        // reserved module names (the `== "ops" || == "lib" || ends_with("_internal")` chain):
        reject(&|s| s.modules[0].name = "ops".to_string(), "reserved: ops");
        reject(&|s| s.modules[0].name = "lib".to_string(), "reserved: lib");
        reject(
            &|s| s.modules[0].name = "x_internal".to_string(),
            "reserved: *_internal",
        );
        // a module with no operators, and an operator naming an undeclared value:
        reject(&|s| s.modules[0].ops.clear(), "module with no operators");
        reject(
            &|s| s.modules[0].ops[0].output = "Missing".to_string(),
            "operator names an undeclared value",
        );
        // range-rule integrity:
        reject(
            &|s| {
                s.values[0].raw = "String".to_string();
                s.values[0].rule = Rule::Range {
                    lo: 0,
                    hi: 9,
                    saturating: false,
                };
            },
            "range on a non-integer raw",
        );
        reject(
            &|s| {
                s.values[0].rule = Rule::Range {
                    lo: -1,
                    hi: 9,
                    saturating: false,
                }
            },
            "unsigned range with a negative floor",
        );
        reject(
            &|s| {
                s.values[0].rule = Rule::Range {
                    lo: 9,
                    hi: 0,
                    saturating: false,
                }
            },
            "empty range (lo > hi)",
        );
    }

    /// The expectation and seam-`via` lookups REFUSE an unknown name: these are `find`
    /// closures (`o.name == *op_name`, `o.name == *via`), so a flipped `==` would pick a
    /// wrong operator instead of failing — feeding a name that exists NOWHERE forces the
    /// original to return `None` (rejection) where the flip would find a stand-in.
    #[test]
    fn plan_refuses_unknown_expectation_and_via_names() {
        let plan = |decl: &str| Genesis::plan(decl, &Deps::Version("0".to_string()));

        // an expectation naming an operator the module does not declare:
        assert!(plan(
            "system! { name: \"app\", values { V = i64 where \"any\"; } \
             modules { m { ops { f(V, V) -> V; } expects { commutative(ghost); } } } }"
        )
        .is_err());

        // an UNKNOWN expectation shape: the error lists the declarable vocabulary, which
        // is every key EXCEPT `irreflexive` (`*key != "irreflexive"`) — so the message must
        // name a real shape; a flipped filter would leave only `irreflexive` and drop it.
        let err = match plan(
            "system! { name: \"app\", values { V = i64 where \"any\"; } \
             modules { m { ops { f(V, V) -> V; } expects { bogus(f); } } } }",
        ) {
            Ok(_) => panic!("an unknown expectation shape must be refused"),
            Err(e) => e,
        };
        assert!(
            err.contains("commutative"),
            "the vocabulary list must name the real shapes: {err}"
        );

        // a via-seam naming a conversion neither module declares:
        assert!(plan(
            "system! { name: \"app\", \
             values { A = i64 where \"any\"; B = i64 where \"any\"; } \
             modules { l { ops { p(A, A) -> A; } } r { ops { q(B, B) -> B; } } } \
             seams { l -- r : transform on A via ghost; } }"
        )
        .is_err());

        // the endpoint-binary check: a via conversion whose sides lack the binary the
        // spanning theory needs is refused (exercises the module-lookup closure with two
        // distinct modules, so a mis-resolved side is observable).
        assert!(plan(
            "system! { name: \"app\", \
             values { A = i64 where \"any\"; B = i64 where \"any\"; } \
             modules { l { ops { conv(A) -> B; } } r { ops { q(B, B) -> B; } } } \
             seams { l -- r : transform on A via conv; } }"
        )
        .is_err());

        // a TRANSPORT seam whose right module does not touch the shared value: the
        // via-less branch must find each named side and check IT (not a stand-in) touches
        // the seam value — so a flipped `input == on` or `output == on` would wrongly
        // accept a module that shares nothing.
        assert!(plan(
            "system! { name: \"app\", \
             values { A = i64 where \"any\"; B = i64 where \"any\"; } \
             modules { l { ops { f(A, A) -> A; } } r { ops { g(B, B) -> B; } } } \
             seams { l -- r : transport on A; } }"
        )
        .is_err());

        // a VALID transport seam with a non-participating module declared FIRST: the seam
        // must check the named sides, not the first module — a flipped `m.name == *side`
        // in the lookup would test `aux` (which touches nothing shared) and wrongly reject.
        assert!(plan(
            "system! { name: \"app\", \
             values { A = i64 where \"any\"; C = i64 where \"any\"; } \
             modules { aux { ops { c(C, C) -> C; } } \
             l { ops { f(A, A) -> A; } } r { ops { g(A, A) -> A; } } } \
             seams { l -- r : transport on A; } }"
        )
        .is_ok());
    }
}
