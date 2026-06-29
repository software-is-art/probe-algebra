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
    Branch, Construction, CostCons, CostNil, Guarded, InputEffect, Lossy, Morphism, Pure, Stateful,
    Unit, S, Z,
};
use crate::gdp::Named;
use crate::Shaped; // the `#[derive(Shaped)]` macro (the trait is `crate::boundary::Shaped`)

// ===== value objects: the language's nouns ================================

// `refined!` generates the newtype, the parse-don't-validate `new`, and the value-object
// registration; the validity rule is the only content.
crate::refined! {
    /// A non-negative integer literal / result, range-checked (the language has no unary
    /// minus, so negatives have no source form to round-trip).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Int(i64);
    fn new(n: i64) = (n >= 0).then_some(n);
}
impl Int {
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

// The validity rule is the only content (it refines a `&str` and NORMALIZES it into the
// stored `String`); `refined!` generates the rest.
crate::refined! {
    /// A variable identifier: a non-empty, all-alphabetic name that is not a keyword. (Named
    /// `Ident`, not `Name`, to avoid colliding with the GDP `Name` brand trait.)
    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct Ident(String);
    fn new(s: &str) = {
        let ok = !s.is_empty()
            && s.chars().all(|c| c.is_ascii_alphabetic())
            && !matches!(s, "if" | "then" | "else" | "let" | "in" | "true" | "false");
        ok.then(|| s.to_string())
    };
}
impl Ident {
    pub fn get(&self) -> &str {
        &self.0
    }
}

/// A binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
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
#[derive(Debug, Clone, PartialEq, Eq, Shaped)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
pub enum Value {
    Int(Int),
    Bool(bool),
}

/// An ENVIRONMENT: variable bindings carried as STATE. It is a value object (the carried
/// state is a citizen, not a loose `HashMap`), and it is what makes `Resolve` `Stateful` —
/// an edge that reads it produces output depending on more than its expression argument.
/// Last binding for a name wins, so `bind` models shadowing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Env(Vec<(Ident, Int)>);
impl Env {
    pub fn new() -> Self {
        Env(Vec::new())
    }
    /// Bind `name` to `val`, shadowing any earlier binding for the same name.
    pub fn bind(mut self, name: Ident, val: Int) -> Self {
        self.0.push((name, val));
        self
    }
    /// The value bound to `name` (the most recent), or `None` if unbound.
    pub fn get(&self, name: &Ident) -> Option<Int> {
        self.0
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, v)| *v)
    }
}

/// An expression PAIRED with the environment it is to be resolved against — the input to
/// the `Stateful` `Resolve` edge. Bundling the state INTO the input keeps `Morphism`'s pure
/// `forward` signature while still modelling an edge whose result depends on carried state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bound {
    env: Env,
    expr: Expr,
}
impl Bound {
    pub fn new(env: Env, expr: Expr) -> Self {
        Bound { env, expr }
    }
    pub fn env(&self) -> &Env {
        &self.env
    }
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
}

/// The raw source text as a value object — the lexer's substrate. Modelling it (rather
/// than passing a bare `&str`) is what lets the scanner read characters through a
/// sanctioned ACCESSOR (`at`) instead of a primitive-returning helper the inward rule
/// would forbid: char handling is confined to this citizen, the same way `Int`
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

// `Int` and `Ident` register themselves via `refined!`; the rest are registered here.
crate::value_object!(Op, Lit, Ty, Expr, Value, Env, Bound, Source, Pos);

// Capability STATE FLOOR, INFERRED from the input type (see `boundary::InputEffect`): `Bound`
// carries an `Env`, so consuming it is at least `Stateful` whatever the body does; an ordinary
// expression grants nothing extra (`Pure`). `run_pure`/`run_within` read these to reject an edge
// that UNDER-declares its capability, on structure rather than its annotation.
impl InputEffect for Expr {
    type Eff = Pure;
}
impl InputEffect for Bound {
    type Eff = Stateful;
}

// The probe surface (`Shaped`) is DERIVED for the composites (`Op`, `Lit`, `Value`, `Expr`)
// but the two leaves carry smart-constructor INVARIANTS the derive cannot see (an `Int` is
// non-negative; an `Ident` is non-empty, alphabetic, non-keyword), so their inhabitants and
// perturbations are written by hand to stay inside the valid set — the one place structure
// alone is not enough.
impl crate::boundary::Shaped for Int {
    fn inhabitant() -> Self {
        Int::new(0).expect("0 is a valid Int")
    }
    fn perturbation_classes(&self) -> Vec<Vec<Self>> {
        // bump and halve — two valid neighbours that move the value in both directions.
        vec![vec![
            Int::new(self.0 + 1).expect("successor stays non-negative"),
            Int::new(self.0 / 2).expect("halving stays non-negative"),
        ]]
    }
}
impl crate::boundary::Shaped for Ident {
    fn inhabitant() -> Self {
        Ident::new("x").expect("x is a valid identifier")
    }
    fn perturbation_classes(&self) -> Vec<Vec<Self>> {
        // a different valid, non-keyword name (distinct from the canonical inhabitant).
        let other = if self.0 == "y" { "z" } else { "y" };
        vec![vec![Ident::new(other).expect("valid identifier")]]
    }
}

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
/// autogen round-trip law (`laws::construction_round_trips`) certifies `render . parse == id`.
pub struct Parse;
/// The classifier: a `Branch` that type-checks a named expression into
/// `WellTyped<N> + IllTyped<N>`, running real inference and keeping BOTH witnesses.
/// Pure (a read-only analysis).
pub struct Check;
/// The evaluator: a `Guarded` edge `Expr -> Value` admitted by a `WellTyped<N>` for the
/// SAME name. You cannot evaluate an expression that has not been proven well-typed —
/// the witness comes from `Check`, exactly as a `Construction` mints its own. Pure.
pub struct Eval;
crate::value_operator!(Parse, Check, Eval, ConstFold, Resolve);

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

// ===== ConstFold: a LOSSY optimization edge, and the capability lattice ====
//
// `Parse`/`Check`/`Eval` are all `Pure`; `ConstFold` exercises a higher rung of the
// capability lattice and the `Morphism` shape (lossy projection + residual witness). It is an
// optimizer `Expr -> Expr` that collapses constant subexpressions — a real loss (you cannot
// recover `(2 + 3)` from `5`), so it is declared `Lossy` and keeps a `Residual` that witnesses
// what it collapsed, restoring invertibility. Because it is `Lossy`, `run_pure` REFUSES it at
// compile time (`tests/compile_fail/run_pure_rejects_lossy`): the ceiling is enforced, not a
// comment. Its probe (`harness`) is oracle-free: folding PRESERVES the value (`eval . fold ==
// eval`, which a wrong constant fails) and its residual RECONSTRUCTS the input.
//
// (The old hand-rolled `ConstFold` counterexamples — a doubling folder, a residual-forgetful
// folder — were RETIRED: they were manual stand-ins for mutants, and `cargo-mutants` plants
// equivalent body-mutants automatically. The blind-spot map is now read off the real mutation
// kill matrix instead of asserted with crafted bugs; see `examples/suite_audit`.)

/// The honest constant folder: a `Lossy` `Morphism` whose residual is the ORIGINAL
/// expression, so `backward` reconstructs exactly.
pub struct ConstFold;

impl ConstFold {
    /// The honest arithmetic at a reducible node — the reference combiner the engine
    /// (`internal::fold`) is parameterized over.
    fn combine(op: Op, x: Int, y: Int) -> Option<Lit> {
        match op {
            Op::Add => Some(Lit::Int(x.plus(y))),
            Op::Mul => Some(Lit::Int(x.times(y))),
            Op::Lt => Some(Lit::Bool(x.less_than(y))),
        }
    }
}

impl Morphism for ConstFold {
    type Capability = Lossy;
    type In = Expr;
    type Out = Expr;
    // The residual witnesses EXACTLY what folding collapsed: the original expression.
    type Residual = Expr;

    fn forward(&self, input: &Expr) -> (Expr, Expr) {
        (
            super::internal::fold(input, &ConstFold::combine),
            input.clone(),
        )
    }
    fn backward(&self, _out: &Expr, residual: &Expr) -> Option<Expr> {
        Some(residual.clone())
    }
}

// ===== Resolve: a STATEFUL edge, and the claim-vs-behaviour audit ==========
//
// `ConstFold` reached the `Lossy` rung; `Resolve` reaches `Stateful`. It substitutes a
// carried `Env` into an expression (`internal::subst`), so its output depends on more than
// its expression argument — the definition of stateful. The capability LATTICE is now
// populated Pure → Lossy → Stateful on the one substrate. Its probe (`harness`) is the
// oracle-free two-route law: substitute-then-eval equals let-bind-then-eval.
//
// A declared capability is only a CLAIM. v4 splits the two ways it can lie:
//
//   - UNDER-claiming (declare `Pure`, secretly read state) — the dangerous case — is now caught
//     STRUCTURALLY: `Bound` carries state (`impl InputEffect for Bound = Stateful`), so the
//     capability's STATE FLOOR is read from the input type and `run_pure` refuses any edge whose
//     input grants more than the ceiling, whatever its annotation says. The under-claiming
//     counterexample is RETIRED; the rejection is pinned by `tests/compile_fail`.
//   - OVER-claiming (declare `Stateful`, ignore the env) is a NEGATIVE the type system can't
//     prove ("this body does not read state"), so it stays the behavioural audit's job —
//     `ResolveIgnoresEnv` below remains its `#[cfg(test)]` subject. (Inference reads state THREADED
//     THROUGH a type; state hidden elsewhere — a global, I/O — also stays the audit's, and
//     `Effectful` stays invisible to types entirely. Inference and audit are complementary.)

/// The honest variable resolver: a `Stateful` `Morphism` that reads its carried `Env`.
pub struct Resolve;

impl Morphism for Resolve {
    type Capability = Stateful;
    type In = Bound;
    type Out = Expr;
    type Residual = Unit;

    fn forward(&self, input: &Bound) -> (Expr, Unit) {
        (super::internal::subst(input.expr(), input.env()), Unit)
    }
    fn backward(&self, _out: &Expr, _residual: &Unit) -> Option<Bound> {
        None // substitution discards which leaves were variables — irreversible.
    }
}

/// Capability-audit COUNTEREXAMPLE — an OVER-claiming `Resolve` twin, TEST-ONLY (flat, so the
/// boundary stays flat). It declares `Stateful` but ignores the env; the audit must catch that an
/// edge claims more than it uses. It carries no probe and is not a production edge, so `build.rs`'s
/// enumeration does not demand one. (The under-claiming twin is gone — inference retired it.)
#[cfg(test)]
pub struct ResolveIgnoresEnv;

#[cfg(test)]
crate::value_operator!(ResolveIgnoresEnv);

#[cfg(test)]
impl Morphism for ResolveIgnoresEnv {
    type Capability = Stateful; // DECLARED stateful…
    type In = Bound;
    type Out = Expr;
    type Residual = Unit;

    fn forward(&self, input: &Bound) -> (Expr, Unit) {
        (input.expr().clone(), Unit) // …but it never reads the env — over-claim.
    }
    fn backward(&self, _out: &Expr, _residual: &Unit) -> Option<Bound> {
        None
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
/// Size axis: the nesting depth (`Expr::depth`).
pub struct Depth;
// `axis!` assigns each a unique sequential Peano `Id` (Nodes = Z, Depth = S<Z>), so two
// axes cannot collide — the overlap-freedom the cost-map lookup relies on, by construction.
crate::axis!(Nodes, Depth);

// `Parse` scans the source once and builds a tree of `Nodes` cells: linear in `Nodes` in
// both time and space.
crate::cost!(Parse, time = CostCons<Nodes, S<Z>, CostNil>, space = CostCons<Nodes, S<Z>, CostNil>);
// `Eval` walks the tree once (linear TIME in `Nodes`) but holds only the current path's
// recursion and bindings (linear SPACE in `Depth`, NOT in `Nodes`).
crate::cost!(Eval, time = CostCons<Nodes, S<Z>, CostNil>, space = CostCons<Depth, S<Z>, CostNil>);
// The degree-of-freedom set of every value object (and thus the completeness obligation)
// is now DERIVED by `#[derive(Shaped)]` — `Expr`'s DOFs are one `Field<Expr, I>` per
// variant, and `Complete<Expr>` covers them by construction. There is no hand-written DOF
// marker to forget: see `crate::boundary::{Field, Complete, require_complete}`.
