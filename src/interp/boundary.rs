//! interp::boundary — a tiny expression language as a boundary CATEGORY.
//!
//! This module is the crate's "cold" use case: a domain the grammar was NOT designed
//! around, built to test whether the abstraction earns its keep. The thesis it puts on
//! trial is the one that justifies all the type-level machinery — **prove it at the
//! boundary, relax inside**:
//!
//!   - the BOUNDARY edges (`Parse`, `Check`, `Eval`) carry the full discipline — a
//!     parse-don't-validate `Construction` with a certified round-trip, a `Branch` that
//!     mints a type-correctness witness, and a `Guarded` evaluation that is UNCALLABLE
//!     without that witness — so "well-typed programs don't go wrong" is a COMPILE-time
//!     fact, not a runtime hope; while
//!   - the INTERNALS (the lexer, parser, type checker, and evaluator in `internal.rs`)
//!     are ordinary code, validated by a few example tests and deliberately left OUT of
//!     the mutation sweep (`.cargo/mutants.toml`). The boundary contracts constrain
//!     them, so they need not be exhaustively re-tested — the over-testing the structure
//!     is meant to eliminate.
//!
//! The language: non-negative integer and boolean literals, `+`/`*`/`<`, `if/then/else`,
//! and `let x = e in e`, written in a fully-parenthesized CANONICAL form so the parse is
//! exactly invertible.

use core::marker::PhantomData;

use crate::boundary::{
    Axis, Branch, Construction, CostCons, CostNil, Covers, DofCons, DofNil, Graded, Guarded,
    HasDofs, Pure, SpaceCost, TimeCost, Unit, S, Z,
};
use crate::gdp::Named;

// ===== value objects: the language's nouns ================================

/// A non-negative integer literal / result, range-checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Int(i64);
impl Int {
    /// Smart constructor: a literal must be non-negative (the language has no unary
    /// minus, so negatives have no source form to round-trip).
    pub fn new(n: i64) -> Option<Self> {
        if n >= 0 {
            Some(Int(n))
        } else {
            None
        }
    }
    pub fn get(&self) -> i64 {
        self.0
    }
    /// The additive identity — the well-typed evaluator's default for a branch the
    /// `WellTyped` proof rules out (see `internal::eval`).
    pub fn zero() -> Self {
        Int(0)
    }
    /// Wrapping addition (total, so evaluation never panics).
    pub fn plus(self, other: Int) -> Int {
        Int(self.0.wrapping_add(other.0).max(0))
    }
    /// Wrapping multiplication (total).
    pub fn times(self, other: Int) -> Int {
        Int(self.0.wrapping_mul(other.0).max(0))
    }
    /// Strict less-than (a predicate is control, not domain data, so it returns `bool`).
    pub fn less_than(self, other: Int) -> bool {
        self.0 < other.0
    }
}

/// A variable identifier: a non-empty, all-alphabetic name that is not a keyword.
/// (Named `Ident`, not `Name`, to avoid colliding with the GDP `Name` brand trait.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Ident(String);
impl Ident {
    /// Smart constructor — rejects empty, non-alphabetic, or keyword identifiers.
    pub fn new(s: &str) -> Option<Self> {
        let ok = !s.is_empty()
            && s.chars().all(|c| c.is_ascii_alphabetic())
            && !matches!(s, "if" | "then" | "else" | "let" | "in" | "true" | "false");
        if ok {
            Some(Ident(s.to_string()))
        } else {
            None
        }
    }
    pub fn get(&self) -> &str {
        &self.0
    }
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Add,
    Mul,
    Lt,
}
impl Op {
    /// The canonical source symbol (a boundary accessor — the sanctioned exit hatch).
    pub fn sym(&self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Mul => "*",
            Op::Lt => "<",
        }
    }
}

/// A literal: an integer or a boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lit {
    Int(Int),
    Bool(bool),
}

/// A monomorphic type in the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Int,
    Bool,
}

/// The abstract syntax tree — the central value object the boundary edges transform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Lit(Lit),
    Var(Ident),
    Bin(Op, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(Ident, Box<Expr>, Box<Expr>),
}
impl Expr {
    /// Ergonomic constructors (also used by the probe generators).
    pub fn int(n: i64) -> Option<Expr> {
        Int::new(n).map(|i| Expr::Lit(Lit::Int(i)))
    }
    pub fn boolean(b: bool) -> Expr {
        Expr::Lit(Lit::Bool(b))
    }
    pub fn var(name: Ident) -> Expr {
        Expr::Var(name)
    }
    pub fn bin(op: Op, a: Expr, b: Expr) -> Expr {
        Expr::Bin(op, Box::new(a), Box::new(b))
    }
    pub fn cond(c: Expr, t: Expr, e: Expr) -> Expr {
        Expr::If(Box::new(c), Box::new(t), Box::new(e))
    }
    pub fn bind(name: Ident, value: Expr, body: Expr) -> Expr {
        Expr::Let(name, Box::new(value), Box::new(body))
    }

    /// The number of AST nodes — the `Nodes` size axis the cost grading is keyed on.
    /// `Eval` visits each node once, so this is also its step count (the empirical
    /// `fits` audit measures growth against this).
    pub fn node_count(&self) -> usize {
        match self {
            Expr::Lit(_) | Expr::Var(_) => 1,
            Expr::Bin(_, a, b) => 1 + a.node_count() + b.node_count(),
            Expr::If(c, t, e) => 1 + c.node_count() + t.node_count() + e.node_count(),
            Expr::Let(_, v, body) => 1 + v.node_count() + body.node_count(),
        }
    }

    /// The maximum nesting depth — the `Depth` size axis. `Eval`'s recursion (and the
    /// environment it threads) grows with this, so it is `Eval`'s SPACE axis, distinct
    /// from its `Nodes` TIME axis: a tree-walker is linear in nodes but only linear in
    /// depth in space.
    pub fn depth(&self) -> usize {
        match self {
            Expr::Lit(_) | Expr::Var(_) => 1,
            Expr::Bin(_, a, b) => 1 + a.depth().max(b.depth()),
            Expr::If(c, t, e) => 1 + c.depth().max(t.depth()).max(e.depth()),
            Expr::Let(_, v, body) => 1 + v.depth().max(body.depth()),
        }
    }

    /// Render to the CANONICAL source form — the exact inverse of the parse, so
    /// `Parse::reconstruct` round-trips. Fully parenthesized, single-space delimited.
    /// (A method, not a free function, so it is admissible at the boundary; returning a
    /// raw `String` here is the sanctioned exit hatch, like any accessor.)
    pub fn render(&self) -> String {
        match self {
            Expr::Lit(Lit::Int(n)) => n.get().to_string(),
            Expr::Lit(Lit::Bool(b)) => b.to_string(),
            Expr::Var(name) => name.get().to_string(),
            Expr::Bin(op, a, b) => format!("({} {} {})", a.render(), op.sym(), b.render()),
            Expr::If(c, t, e) => {
                format!(
                    "(if {} then {} else {})",
                    c.render(),
                    t.render(),
                    e.render()
                )
            }
            Expr::Let(name, v, body) => {
                format!("(let {} = {} in {})", name.get(), v.render(), body.render())
            }
        }
    }
}

/// A runtime value — the result of evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(Int),
    Bool(bool),
}

/// The raw source text as a value object — the lexer's substrate. Modelling it (rather
/// than passing a bare `&str`) is what lets the scanner read characters through a
/// sanctioned ACCESSOR (`at`) instead of a primitive-returning helper the inward rule
/// would forbid: char handling is confined to this citizen, the same way a `Cents`
/// confines `i64`. `pub(crate)` — it is internal substrate, not domain API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Source(String);
impl Source {
    pub(crate) fn new(text: &str) -> Self {
        Source(text.to_string())
    }
    /// The character at position `p`, or `None` past the end. The one place a `char`
    /// crosses out — a boundary accessor, the substrate's sanctioned exit hatch. The
    /// canonical source is ASCII (the language has no other glyphs), so a byte index is
    /// O(1) and exact; a non-ASCII byte maps to a char no token arm accepts, so the lex
    /// rejects it anyway.
    pub(crate) fn at(&self, p: &Pos) -> Option<char> {
        self.0.as_bytes().get(p.index()).map(|&b| b as char)
    }
}

/// A position into a `Source` — a value object, so the scanner's cursor is a citizen
/// rather than a raw `usize` (which the boundary would reject as a primitive field).
/// Advancing is functional: `next` returns the successor position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Pos(usize);
impl Pos {
    pub(crate) fn start() -> Self {
        Pos(0)
    }
    pub(crate) fn next(self) -> Self {
        Pos(self.0 + 1)
    }
    /// The raw offset — the sanctioned exit hatch for indexing.
    pub(crate) fn index(&self) -> usize {
        self.0
    }
}

crate::value_object!(Int, Ident, Op, Lit, Ty, Expr, Value, Source, Pos);

// ===== the proof tokens: the type-correctness witnesses ===================

crate::proof_token!(
    /// A proof that the expression named `N` is WELL-TYPED (it type-checks in the empty
    /// environment: no type mismatch, no unbound variable). Branded with the
    /// expression's name, so a proof for program A cannot authorize evaluating B; minted
    /// ONLY by `Check::classify`. It is the witness `Eval` demands — the type-level
    /// statement of "well-typed programs don't go wrong".
    WellTyped
);
crate::proof_token!(
    /// A proof that the expression named `N` is ILL-TYPED — the NEGATIVE witness, kept
    /// (not discarded as a `None`) so the rejection path is first-class. A statistical
    /// check must never mint it: `Check` runs the real inference (a proof is only as true
    /// as its mint).
    IllTyped
);

// ===== the boundary edges: Construction, Branch, Guarded ==================

/// The ENTRY edge: parse canonical source into an `Expr`, or reject it. A pure
/// refinement — the canonical form carries no formatting to normalize away — so its
/// residual is `Unit` and `reconstruct` (via `Expr::render`) recovers the source
/// exactly. Modelled as a `Construction`, the parser is INSIDE the probe space: the
/// autogen round-trip law (`laws::construction_laws`) certifies `render . parse == id`.
pub struct Parse;
/// The classifier: a `Branch` that type-checks a named expression into
/// `WellTyped<N> + IllTyped<N>`, running real inference and keeping BOTH witnesses.
/// Pure (a read-only analysis).
pub struct Check;
/// The evaluator: a `Guarded` edge `Expr -> Value` admitted by a `WellTyped<N>` for the
/// SAME name. You cannot evaluate an expression that has not been proven well-typed —
/// the witness comes from `Check`, exactly as a `Construction` mints its own. Pure.
pub struct Eval;
crate::value_operator!(Parse, Check, Eval);

impl Construction for Parse {
    type Capability = Pure;
    type Raw = String;
    type Refined = Expr;
    type Residual = Unit;

    fn parse(&self, raw: &String) -> Option<(Expr, Unit)> {
        super::internal::parse(raw).map(|e| (e, Unit))
    }

    fn reconstruct(&self, refined: &Expr, _residual: &Unit) -> Option<String> {
        Some(refined.render())
    }
}

impl Parse {
    /// Smart-constructor facade over the construction: parse canonical source into an
    /// `Expr`, dropping the (empty) residual.
    pub fn parse_str(&self, src: &str) -> Option<Expr> {
        self.parse(&src.to_string()).map(|(e, _)| e)
    }
}

impl Branch for Check {
    type Capability = Pure;
    type In<N> = Named<N, Expr>;
    type Left<N> = WellTyped<N>;
    type Right<N> = IllTyped<N>;

    fn branch<N>(&self, expr: &Named<N, Expr>) -> Result<WellTyped<N>, IllTyped<N>> {
        if super::internal::infer(expr.value()).is_some() {
            Ok(WellTyped(PhantomData))
        } else {
            Err(IllTyped(PhantomData))
        }
    }
}

impl Check {
    /// Type-check a named expression — the ergonomic name for the `Branch` edge. Both
    /// arms carry a proof; an ill-typed program is an `IllTyped` witness, never a silent
    /// `None`.
    pub fn classify<N>(&self, expr: &Named<N, Expr>) -> Result<WellTyped<N>, IllTyped<N>> {
        self.branch(expr)
    }
}

impl Guarded for Eval {
    type Capability = Pure;
    type In<N> = Named<N, Expr>;
    type Proof<N> = WellTyped<N>;
    // Evaluation KEEPS the expression's brand `N`: the resulting value is provably the
    // value of the program named `N`.
    type Out<N> = Named<N, Value>;

    fn guard<N>(&self, expr: &Named<N, Expr>, _proof: &WellTyped<N>) -> Named<N, Value> {
        expr.map(super::internal::eval)
    }
}

impl Eval {
    /// Evaluate a well-typed, named expression — the ergonomic name for the `Guarded`
    /// edge. Requires a `WellTyped<N>` for the same name, so an unchecked or ill-typed
    /// program will not type-check, and there is no other way to reach a `Value`. The
    /// result keeps the brand `N`.
    pub fn run<N>(&self, expr: &Named<N, Expr>, proof: &WellTyped<N>) -> Named<N, Value> {
        self.guard(expr, proof)
    }
}

// ===== the COST grading on the interp edges ===============================
//
// The pluggable cost grading from `crate::boundary` keyed on this language's two size
// axes. `Eval` is the case the open keyed map exists for: a tree-walker whose TIME and
// SPACE diverge — linear in `Nodes` (one visit each) but only linear in `Depth` in space
// (the recursion + environment stack), not linear in nodes. A single scalar "cost" could
// not say that; the per-axis map can.

/// Size axis: the number of AST nodes (`Expr::node_count`).
pub struct Nodes;
impl Axis for Nodes {
    type Id = Z;
}
/// Size axis: the nesting depth (`Expr::depth`).
pub struct Depth;
impl Axis for Depth {
    type Id = S<Z>;
}

// `Parse` scans the source once and builds a tree of `Nodes` cells: linear in `Nodes` in
// both time and space.
impl Graded<TimeCost> for Parse {
    type Carrier = CostCons<Nodes, S<Z>, CostNil>;
}
impl Graded<SpaceCost> for Parse {
    type Carrier = CostCons<Nodes, S<Z>, CostNil>;
}
// `Eval` walks the tree once (linear TIME in `Nodes`) but holds only the current path's
// recursion and bindings (linear SPACE in `Depth`, NOT in `Nodes`).
impl Graded<TimeCost> for Eval {
    type Carrier = CostCons<Nodes, S<Z>, CostNil>;
}
impl Graded<SpaceCost> for Eval {
    type Carrier = CostCons<Depth, S<Z>, CostNil>;
}

// ===== degrees of freedom of an `Expr`, and the coverage demand ===========
//
// The static completeness obligation from `crate::boundary`, re-homed onto `Expr`. An
// expression varies along two independent dimensions, and a probe suite that claims to be
// complete must reach BOTH or fail to compile.

/// DOF: the STRUCTURE — which constructor, which operator, the tree shape. A probe sees it
/// by perturbing the shape (swap `Add` for `Mul`, an `If` for a `Let`).
pub struct Shape;
/// DOF: the LITERAL payloads — the `Int`/`Bool` values at the leaves. A probe sees it by
/// nudging a literal while holding the shape fixed.
pub struct Literals;

impl HasDofs for Expr {
    type Dofs = DofCons<Shape, DofCons<Literals, DofNil>>;
}

/// A complete probe of `Expr`: it reaches BOTH the shape and the leaf values, so
/// `require_complete::<Expr, _>(&FullProbe)` type-checks.
pub struct FullProbe;
impl Covers<Shape> for FullProbe {}
impl Covers<Literals> for FullProbe {}

/// An INCOMPLETE probe: it perturbs the tree shape but never varies a literal, so it is
/// blind to the `Literals` dimension. `require_complete` rejects it at compile time
/// (pinned in `tests/compile_fail/incomplete_probe_rejected`) — the LSP push-back a coding
/// agent gets for a probe with a hole, before any test runs.
pub struct ShapeOnlyProbe;
impl Covers<Shape> for ShapeOnlyProbe {}
