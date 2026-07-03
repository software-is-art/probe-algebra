//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
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
//! expect   := shape "(" Ident ("," Ident)? ")" ";"
//! shape    := "commutative" | "associative" | "idempotent" | "bias_later" | "bias_earlier"
//!           | "identity" | "annihilation"
//!             — the declared LAW EXPECTATIONS, a subset of the engine's shape catalog
//!               (`ShapeCatalog::inventory`) restricted to homogeneous binary operators:
//!               one-argument shapes take the operator; `identity`/`annihilation` take the
//!               operator and a declared nullary constant of its sort. The keys are the same
//!               ones `discover::expect` and the `#[algebra]` macro's `expects(...)` speak.
//! seams    := "seams" "{" seam* "}"
//! seam     := Ident "--" Ident ":" ("transport" | "transform") "on" Ident ";"
//!             — a declared seam between two modules on a shared value: `transport` promises
//!               the sort is the SAME type on both sides (discharged by construction — genesis
//!               defines each value once); `transform` promises a conversion that must be a
//!               HOMOMORPHISM, emitted as a test obligation.
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
use crate::discover::engine::{ShapeCatalog, ShapeInfo};
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

/// A declared seam between two modules on a shared value.
pub struct SeamDecl {
    pub left: String,
    pub right: String,
    pub kind: SeamKindDecl,
    pub on: String,
}

/// The declared seam kind (parse-side twin of `cohesion::SeamKind`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeamKindDecl {
    /// The shared sort is the SAME type on both sides — preserved by construction.
    Transport,
    /// A conversion crosses the seam and must be a HOMOMORPHISM — an emitted obligation.
    Transform,
}

/// The v1 expectation vocabulary: the shape catalog's homogeneous-binary rows by declaration
/// key, each with the argument count it takes (the operator, plus a nullary constant for the
/// last two). The keys are exactly the ones `discover::expect` speaks, so a parsed line is an
/// `Expectation`, not a genesis-private twin.
const V1_EXPECT_KEYS: &[(&str, usize)] = &[
    ("commutative", 1),
    ("associative", 1),
    ("idempotent", 1),
    ("bias_later", 1),
    ("bias_earlier", 1),
    ("identity", 2),
    ("annihilation", 2),
];

/// The catalog row a (validated) expectation instantiates.
fn shape_info(e: &Expectation) -> ShapeInfo {
    ShapeCatalog::inventory()
        .into_iter()
        .find(|shape| shape.name == e.shape)
        .expect("an Expectation's shape is always a ratified catalog name")
}

/// The shape-catalog rank — the order the engine tries (and therefore renders) the shapes, so
/// the target lock's law order matches a discovery that confirms the declaration.
fn shape_rank(e: &Expectation) -> usize {
    ShapeCatalog::inventory()
        .iter()
        .position(|shape| shape.name == e.shape)
        .expect("an Expectation's shape is always a ratified catalog name")
}

/// Does the expectation's shape take a constant (its second declared name)?
fn takes_constant(e: &Expectation) -> bool {
    matches!(e.shape, "identity" | "annihilation")
}

/// The declared law as the lock will state it: `(prose, equation)`. The PROSE is the ratified
/// shape catalog's own template (`ShapeInfo::instantiate`) — one source, never restated — over
/// the declared names; the EQUATION is the canonical form the engine renders for the shape's
/// discovered instance (default variable names `x`, `y`, `z`, the operator infix under its own
/// name — the byte-exact render pin in the tests holds this side in sync).
fn law(e: &Expectation) -> (String, String) {
    let op = e.ops[0].as_str();
    let prose = match e.ops.get(1) {
        Some(konst) => shape_info(e).instantiate(&[("op", op), ("const", konst)]),
        None => shape_info(e).instantiate(&[("op", op)]),
    };
    let equation = match e.shape {
        "commutativity" => format!("(x {op} y) = (y {op} x)"),
        "associativity" => format!("((x {op} y) {op} z) = (x {op} (y {op} z))"),
        "idempotence" => format!("(x {op} x) = x"),
        "bias (right-regular)" => format!("((x {op} y) {op} x) = (y {op} x)"),
        "bias (left-regular)" => format!("((x {op} y) {op} x) = (x {op} y)"),
        "identity" => format!("({} {op} x) = x", e.ops[1]),
        "annihilation" => format!("({konst} {op} x) = {konst}", konst = e.ops[1]),
        other => unreachable!("v1 admits no `{other}` expectation"),
    };
    (prose, equation)
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
}

/// A raw type's canonical text: `quote`'s token render with its inter-token spacing
/// collapsed back to source form (`Vec < u8 >` → `Vec<u8>`), so the generated newtypes read
/// as written.
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
fn parse_bound(input: ParseStream) -> syn::Result<i128> {
    let negative = input.peek(Token![-]);
    if negative {
        input.parse::<Token![-]>()?;
    }
    let lit: syn::LitInt = input.parse()?;
    let magnitude: i128 = lit.base10_parse()?;
    Ok(if negative { -magnitude } else { magnitude })
}

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

fn parse_expect(input: ParseStream) -> syn::Result<Expectation> {
    let shape: Ident = input.parse()?;
    let args;
    parenthesized!(args in input);
    let idents: Punctuated<Ident, Token![,]> = args.parse_terminated(Ident::parse, Token![,])?;
    input.parse::<Token![;]>()?;
    let names: Vec<String> = idents.iter().map(Ident::to_string).collect();
    let known = V1_EXPECT_KEYS
        .iter()
        .find(|(key, arity)| shape == key && *arity == names.len());
    let Some((key, arity)) = known else {
        return Err(syn::Error::new(
            shape.span(),
            format!(
                "unknown expectation `{shape}` with {} argument(s); the v1 vocabulary is \
                 commutative(op), associative(op), idempotent(op), bias_later(op), \
                 bias_earlier(op), identity(op, const), annihilation(op, const)",
                names.len()
            ),
        ));
    };
    if *arity == 2 && names[0] == names[1] {
        // `Expectation` normalises to DISTINCT names (a discovered law's fingerprint does the
        // same), so a self-paired declaration would silently lose its constant — refuse it.
        return Err(syn::Error::new(
            shape.span(),
            format!(
                "expectation `{key}({0}, {0})` names the same operator twice — the constant \
                 must be a distinct nullary operator",
                names[0]
            ),
        ));
    }
    Ok(Expectation::of(key, names))
}

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
    input.parse::<Token![;]>()?;
    Ok(SeamDecl {
        left: left.to_string(),
        right: right.to_string(),
        kind,
        on: on.to_string(),
    })
}

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
fn parse_declaration(source: &str) -> Result<SystemDecl, String> {
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
    parse_system
        .parse2(mac.tokens)
        .map_err(|e| format!("genesis: system! declaration: {e}"))
}

// ===== validation (the declaration must be coherent before anything is emitted) =============

/// Reject an incoherent declaration with a message naming the exact production at fault —
/// generation only ever runs over a validated system.
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
            let Some(op) = m.ops.iter().find(|o| o.name == e.ops[0]) else {
                return err(format!(
                    "expectation `{}` names `{}`, which module `{}` does not declare",
                    e.render(),
                    e.ops[0],
                    m.name
                ));
            };
            let homogeneous_binary =
                op.inputs.len() == 2 && op.inputs[0] == op.output && op.inputs[1] == op.output;
            if !homogeneous_binary {
                return err(format!(
                    "expectation `{}` needs a homogeneous binary operator (s × s → s); \
                     `{}` is `{}({}) -> {}`",
                    e.render(),
                    op.name,
                    op.name,
                    op.inputs.join(", "),
                    op.output
                ));
            }
            if takes_constant(e) {
                let konst = &e.ops[1];
                let ok = m
                    .ops
                    .iter()
                    .any(|o| o.name == *konst && o.inputs.is_empty() && o.output == op.output);
                if !ok {
                    return err(format!(
                        "expectation `{}` needs `{konst}` to be a declared constant \
                         (a nullary operator) of sort `{}` in module `{}`",
                        e.render(),
                        op.output,
                        m.name
                    ));
                }
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
    Ok(())
}

// ===== naming and placement =================================================================

/// The module that OWNS a value: the first module (declaration order) whose operators mention
/// it. The value object is defined in that module's boundary file; everyone else imports it —
/// which is exactly what makes a declared `transport` seam true by construction.
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
fn crate_ident(name: &str) -> String {
    name.replace('-', "_")
}

/// The generated crate's qualify-census bless variable (`credit-app` →
/// `BLESS_CREDIT_APP_QUALIFY`) — renamed per crate so a workspace-wide bless cannot silently
/// re-bless two censuses at once (the fixture's own convention).
fn bless_env(name: &str) -> String {
    format!("BLESS_{}_QUALIFY", name.to_uppercase().replace('-', "_"))
}

/// Escape text for embedding inside a generated double-quoted string literal.
fn esc_lit(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
}

/// One line of validity prose safe for a generated doc comment.
fn doc_safe(s: &str) -> String {
    s.replace('\n', " ")
}

const ARG_NAMES: [&str; 6] = ["a", "b", "c", "d", "e", "f"];

/// The `//! GENERATED …` banner every emitted `.rs` carries under its tier line.
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
fn expects_for<'a>(m: &'a ModuleDecl, op: &str) -> Vec<&'a Expectation> {
    let mut found: Vec<&Expectation> = m.expects.iter().filter(|e| e.ops[0] == op).collect();
    found.sort_by_key(|e| shape_rank(e));
    found
}

// ===== emission (one function per generated file) ===========================================

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

fn emit_build_rs(sys: &SystemDecl) -> String {
    let template = r#"//! build.rs — the enforcement shim genesis emitted: attach the whole structural discipline
//! from `boundary-enforce`, with a config that is THIS crate's own.
//!
//! Two decisions live here and must live here:
//!
//! * **The kernel allowlist.** Claiming `Tier: KERNEL` exempts a file from every structural
//!   rule, so it cannot be self-serve — the file must ALSO be named here, where admitting a
//!   member is a reviewed diff in this crate's tree. The generated kernel is exactly
//!   `src/lib.rs` (the module roster).
//!
//! * **The qualification census.** FIRST BUILD: run `@BLESS@=1 cargo build`
//!   once to mint `spec/qualify.spec` — a missing lock is stale, never fresh, so an unblessed
//!   tree refuses to build. From then on the census is drift-gated; regenerate with the same
//!   variable and ratify the diff.

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// The RATIFIED kernel of THIS crate — the only files allowed to declare `Tier: KERNEL`.
const KERNEL_ALLOWLIST: &[&str] = &["src/lib.rs"];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = KERNEL_ALLOWLIST.iter().map(|s| s.to_string()).collect();
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.bless_env = "@BLESS@".to_string();
    Enforcement::enforce_or_panic(&config);
}
"#;
    template.replace("@BLESS@", &bless_env(&sys.name))
}

fn emit_lib_rs(sys: &SystemDecl) -> String {
    let mut out = String::from(
        "//! Tier: KERNEL — the crate's trusted floor: the module roster, and nothing else.\n",
    );
    out.push_str(&banner(&sys.name));
    out.push_str(&format!(
        "//!\n\
         //! # {name} — a crate whose layout was DERIVED from one declaration\n\
         //!\n\
         //! Everything structural in this tree — the tier-marked files, the operator plumbing,\n\
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
        "//! 2. First build: `{bless}=1 cargo build` — mints `spec/qualify.spec`, which the\n\
         //!    enforcement shim (`build.rs`) drift-gates from then on.\n\
         //! 3. `cargo test` — the expectations gate names each module's DISTANCE from its\n\
         //!    declaration, and the freeze gate holds the LIVE discovered spec (module laws and\n\
         //!    the seam graph) against the TARGET locks; red means the meaning does not yet earn\n\
         //!    the declaration.\n\
         //! 4. `cargo run --example freeze` — regenerate the locks from discovery and read the\n\
         //!    diff against the targets. That diff IS the review: ratify it, or fix the meaning.\n",
        bless = bless_env(&sys.name)
    ));
    out.push('\n');
    for m in &sys.modules {
        out.push_str(&format!("pub mod {};\n", m.name));
    }
    out.push_str("pub mod ops;\npub mod system;\n\n");
    for m in &sys.modules {
        out.push_str(&format!("mod {}_internal;\n", m.name));
    }
    out
}

/// `use crate::<owner>::<Value>;` lines for every value `m`'s operators touch, deduplicated
/// and sorted (imports are mechanical; the set is exactly what the signatures name).
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
             {indent}    ({lo}..={hi}).contains(&raw).then(|| {name}(raw))\n"
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

fn emit_boundary(sys: &SystemDecl, m: &ModuleDecl) -> String {
    let owned: Vec<&ValueDecl> = sys
        .values
        .iter()
        .filter(|v| owner_of(sys, &v.name) == Some(m.name.as_str()))
        .collect();

    let mut out = format!(
        "//! Tier: BOUNDARY — `{m}`'s strict value-object surface (the tier-1 grammar).\n",
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

fn emit_internal(sys: &SystemDecl, m: &ModuleDecl) -> String {
    let mut out = format!(
        "//! Tier: INTERIOR — the workshop `{m}`'s boundary delegates to (the tier-2 inward\n\
         //! rule holds: nothing here returns a raw primitive).\n",
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

fn emit_ops(sys: &SystemDecl) -> String {
    let mut out = String::from(
        "//! Tier: ALGEBRA — the discovered-law layer: each declared module's operators, as the\n\
         //! theory `#[algebra]` synthesises from ordinary function signatures.\n",
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
        // imports (indented one level inside the module).
        for line in value_imports(sys, m, false).lines() {
            out.push_str(&format!("    {line}\n"));
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
    out
}

/// The TARGET lock: the DECLARED laws in the exact committed lock format
/// (`discover::freeze::render` — header, law lines, coverage line), so the generated crate's
/// drift gate is red until discovery re-derives precisely what was declared.
fn emit_target_lock(m: &ModuleDecl) -> String {
    let mut out = format!(
        "# discovered spec: {} — a behaviour lock; regenerate via this repo's freeze path and ratify the diff.\n\n",
        m.name
    );
    for op in &m.ops {
        for e in expects_for(m, &op.name) {
            let (prose, equation) = law(e);
            out.push_str(&format!("- {prose}\n      {equation}\n"));
        }
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

/// The compiled `system!` twin of the declaration (see `discover::system`): the marker, the
/// module registry, and every TRANSPORT seam — discharged by construction, because genesis
/// defines each value object exactly once (the macro's compile-time witness pins it). A
/// declared TRANSFORM seam is NOT compiled here: its conversion is meaning genesis cannot
/// name, so it stays a loud hole in `tests/seams.rs` until a spanning theory exists.
fn emit_system(sys: &SystemDecl) -> String {
    let marker = camel(&sys.name);
    let transports: Vec<&SeamDecl> = sys
        .seams
        .iter()
        .filter(|s| s.kind == SeamKindDecl::Transport)
        .collect();
    let transforms = sys.seams.len() - transports.len();

    let mut out = String::from(
        "//! Tier: ALGEBRA — the compiled `system!` graph: the application-level spec as a \
         checked artifact.\n",
    );
    out.push_str(&banner(&sys.name));
    out.push_str(
        "//!\n\
         //! The graph IS the registry: `modules()` is the list the freeze loop records and the\n\
         //! drift gate checks (a module cannot silently fall out of it), and the rendered graph\n\
         //! — modules, seams, each seam's obligation and status — freezes into the committed\n\
         //! `spec/*.system.spec` lock. Declared transport seams are discharged BY CONSTRUCTION:\n\
         //! each shared value object is defined exactly once, and the `system!` macro emits the\n\
         //! compile-time witness that keeps that true.\n",
    );
    if transforms > 0 {
        out.push_str(&format!(
            "//!\n\
             //! NOTE: {transforms} declared TRANSFORM seam(s) are not compiled into this graph —\n\
             //! a transform's conversion is meaning, not structure. Each remains a loud hole in\n\
             //! `tests/seams.rs`; once a spanning theory exists, declare the seam here as\n\
             //! `Left -- Right : transform on Value via conversion in SpanningTheory;`.\n"
        ));
    }
    out.push('\n');

    // imports: every module's theory marker, plus each transport seam's value (for the witness).
    let mut imports = BTreeSet::new();
    for m in &sys.modules {
        imports.insert(format!(
            "use crate::ops::{}_ops::{};\n",
            m.name,
            camel(&m.name)
        ));
    }
    for s in &transports {
        let owner = owner_of(sys, &s.on).expect("validated: seam value has an owner");
        imports.insert(format!("use crate::{owner}::{};\n", s.on));
    }
    for line in &imports {
        out.push_str(line);
    }

    out.push_str(&format!(
        "\n/// The system marker — `SystemReport::of::<{marker}>()` is the graph the lock \
         freezes.\npub struct {marker};\n\nboundary_spec::system! {{\n    {marker} : \
         \"{name}\",\n    modules {{\n",
        name = sys.name
    ));
    for m in &sys.modules {
        out.push_str(&format!("        {};\n", camel(&m.name)));
    }
    out.push_str("    }\n");
    if !transports.is_empty() {
        out.push_str("    seams {\n");
        for s in &transports {
            out.push_str(&format!(
                "        {} -- {} : transport on {} by construction;\n",
                camel(&s.left),
                camel(&s.right),
                s.on
            ));
        }
        out.push_str("    }\n");
    }
    out.push_str("}\n");
    out
}

/// The TARGET system lock: the declared graph in the exact committed lock format
/// (`discover::system::SystemReport::render` — keep the two in step; the render pin in the
/// tests holds this side). Transport seams are born discharged (by construction), so this
/// lock goes green the moment the crate compiles and discovery runs.
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
    let transports: Vec<&SeamDecl> = sys
        .seams
        .iter()
        .filter(|s| s.kind == SeamKindDecl::Transport)
        .collect();
    if transports.is_empty() {
        out.push_str("seams: none — no module pair declares a shared-value obligation.\n");
        return out;
    }
    out.push_str("seams (each edge: its obligation, then the verdict its checker returned):\n");
    for s in &transports {
        out.push_str(&format!(
            "- {} -- {} : transport on {}\n",
            s.left, s.right, s.on
        ));
        out.push_str("      obligation: the modules share this value and must agree on its laws\n");
        out.push_str(
            "      status: discharged by construction — the shared value is one type on both \
             sides (the declaration carries the compile-time witness)\n",
        );
    }
    out
}

/// The DECLARED-LAWS gate: one distance test per module that declares expectations —
/// `Distance::of` names exactly what is missing, so the gate is red WITH A WORKLIST until
/// the meaning earns the declaration. Emitted only when some module declares `expects`.
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
    out
}

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

fn emit_seams(sys: &SystemDecl) -> String {
    let krate = crate_ident(&sys.name);
    let mut out = format!(
        "//! seams — the DECLARED seam obligations, one test per seam.\n\
         //!\n\
         //! GENERATED by genesis for `{}`. A seam names what a module split must preserve:\n\
         //! a `transport` seam shares a sort that is the SAME type on both sides (preserved by\n\
         //! construction — genesis defines each value object exactly once); a `transform` seam\n\
         //! carries a conversion that must be a HOMOMORPHISM, and that check is a meaning hole.\n",
        sys.name
    );
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
            SeamKindDecl::Transform => out.push_str(&format!(
                "\n/// TRANSFORM seam `{l} -- {r}` on `{v}`: a conversion crosses it, and the \
                 conversion must be\n/// a HOMOMORPHISM — `h(a op b) == h(a) op' h(b)` — or \
                 the cut is bad. Naming `h` and the two\n/// operators is meaning; probing the \
                 equation over the grid is the discharge.\n#[test]\nfn {test}() {{\n    \
                 todo!(\"MEANING: name the conversion across `{l} -- {r}` and probe h(a op b) \
                 == h(a) op'(h(b))\")\n}}\n",
                l = s.left,
                r = s.right,
                v = s.on,
            )),
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
    /// Every file to write, target-root-relative, in emission order.
    pub edits: Vec<FileEdit>,
}

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

impl Genesis {
    /// The PURE half: parse the declaration source (`syn`, from the `system!` token stream),
    /// validate it, and derive every file. No I/O — the same plan can be inspected, tested, or
    /// applied.
    pub fn plan(declaration: &str, deps: &Deps) -> Result<Plan, String> {
        let system = parse_declaration(declaration)?;
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
            contents: emit_system(&system),
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
        if system.modules.iter().any(|m| !m.expects.is_empty()) {
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
        Ok(Plan { system, edits })
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
            ]
        );
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

    /// The COMPILED `system!` twin: the marker, the module registry, and the transport seam
    /// as a by-construction declaration (genesis defines each value once); the roster admits
    /// the module. A transform seam would stay OUT of the compiled graph (meaning, not
    /// structure) — pinned by the second declaration.
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
        assert!(system.contains("boundary_spec::system! {"));
        assert!(system.contains("CreditApp : \"credit-app\","));
        assert!(system.contains("        Meter;\n        Billing;\n"));
        assert!(system.contains("Meter -- Billing : transport on Credits by construction;"));
        assert!(system.contains("use crate::meter::Credits;"));

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
        assert!(system.contains("1 declared TRANSFORM seam(s) are not compiled"));
        assert!(!system.contains("seams {"), "no compiled seam block");
        // ... while the seam test hole is still emitted.
        assert!(plan.listing().contains(&"tests/seams.rs"));
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

        assert!(boundary.contains("(0..=20).contains(&raw).then(|| C(raw))"));
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
                "nullary operator",
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
}
