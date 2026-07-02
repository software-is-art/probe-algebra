//! Tier: INTERIOR — the workshop / leaves (tier 2 inward rule).
//!
//! internal — PRIVATE implementation of the interpreter: lexer, parser, type
//! checker, and evaluator. Other modules cannot name anything here.
//!
//! This is the "relax inside" half of the experiment, taken to its limit: this file has
//! **zero tests of its own**. Nothing here is exercised directly — the lexer, parser,
//! type checker, and evaluator are reached ONLY transitively, through the boundary edges
//! in `boundary.rs`: the autogen parse round-trip law (`laws::construction_round_trips`, which
//! certifies `render . parse == id` over generated source) plus the behavioural tests of
//! `Parse`/`Check`/`Eval` in `tests.rs`.
//!
//! Unlike the other modules' internals, this file is deliberately KEPT IN the mutation
//! sweep, precisely so we can MEASURE the result: cargo-mutants on this file quantifies
//! how much internal correctness the boundary rigour buys for free. The surviving
//! mutants are the exact measure of what the boundary contracts do NOT pin — and the
//! evaluator's `_` arms (unreachable under the `WellTyped` guard) are provably-equivalent
//! survivors, the guard turning whole branches dead. The INWARD rule still holds: no
//! function returns a raw primitive — every result is an `Expr`, `Ty`, `Value`, or a
//! `Token`/collection of value objects.

use std::collections::HashMap;

use crate::interp::boundary::{Env, Expr, Ident, Int, Lit, Op, Pos, Source, Ty, Value};

// ===== lexer ==============================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    LParen,
    RParen,
    Op(Op),
    Eq,
    If,
    Then,
    Else,
    Let,
    In,
    Num(Int),
    Bool(bool),
    Ident(Ident),
}

/// Tokenize a `Source`. `None` on any unrecognized character or out-of-range literal
/// (so the parse rejects it rather than admitting garbage). Characters are read through
/// the `Source::at` accessor and the cursor is a `Pos` value object — no raw `char`/
/// `usize` helpers, so the scanner composes typed citizens (the inward rule, satisfied
/// by MODELLING the substrate rather than exempting it).
fn lex(source: &Source) -> Option<Vec<Token>> {
    let mut pos = Pos::start();
    let mut out = Vec::new();
    while let Some(c) = source.at(&pos) {
        match c {
            ' ' => pos = pos.next(),
            '(' => {
                out.push(Token::LParen);
                pos = pos.next();
            }
            ')' => {
                out.push(Token::RParen);
                pos = pos.next();
            }
            '+' => {
                out.push(Token::Op(Op::Add));
                pos = pos.next();
            }
            '*' => {
                out.push(Token::Op(Op::Mul));
                pos = pos.next();
            }
            '<' => {
                out.push(Token::Op(Op::Lt));
                pos = pos.next();
            }
            '=' => {
                out.push(Token::Eq);
                pos = pos.next();
            }
            _ if c.is_ascii_digit() => {
                let mut s = String::new();
                while let Some(d) = source.at(&pos).filter(|d| d.is_ascii_digit()) {
                    s.push(d);
                    pos = pos.next();
                }
                let n: i64 = s.parse().ok()?;
                out.push(Token::Num(Int::new(n)?));
            }
            _ if c.is_ascii_alphabetic() => {
                let mut s = String::new();
                while let Some(a) = source.at(&pos).filter(|a| a.is_ascii_alphabetic()) {
                    s.push(a);
                    pos = pos.next();
                }
                out.push(match s.as_str() {
                    "if" => Token::If,
                    "then" => Token::Then,
                    "else" => Token::Else,
                    "let" => Token::Let,
                    "in" => Token::In,
                    "true" => Token::Bool(true),
                    "false" => Token::Bool(false),
                    _ => Token::Ident(Ident::new(&s)?),
                });
            }
            _ => return None,
        }
    }
    Some(out)
}

// ===== parser (recursive descent over the canonical, fully-parenthesized form) =====

struct Parser {
    toks: Vec<Token>,
    pos: usize,
}
impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.pos)
    }
    fn bump(&mut self) -> Option<Token> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }
    fn eat(&mut self, want: &Token) -> bool {
        if self.peek() == Some(want) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expr(&mut self) -> Option<Expr> {
        let t = self.peek()?.clone();
        match t {
            Token::Num(n) => {
                self.pos += 1;
                Some(Expr::Lit(Lit::Int(n)))
            }
            Token::Bool(b) => {
                self.pos += 1;
                Some(Expr::boolean(b))
            }
            Token::Ident(name) => {
                self.pos += 1;
                Some(Expr::Var(name))
            }
            Token::LParen => {
                self.pos += 1;
                let e = match self.peek()? {
                    Token::If => self.if_form()?,
                    Token::Let => self.let_form()?,
                    _ => self.bin_form()?,
                };
                if self.eat(&Token::RParen) {
                    Some(e)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn bin_form(&mut self) -> Option<Expr> {
        let a = self.expr()?;
        let op = match self.bump()? {
            Token::Op(o) => o,
            _ => return None,
        };
        let b = self.expr()?;
        Some(Expr::bin(op, a, b))
    }

    fn if_form(&mut self) -> Option<Expr> {
        self.pos += 1; // skip `if`
        let c = self.expr()?;
        if !self.eat(&Token::Then) {
            return None;
        }
        let t = self.expr()?;
        if !self.eat(&Token::Else) {
            return None;
        }
        let e = self.expr()?;
        Some(Expr::cond(c, t, e))
    }

    fn let_form(&mut self) -> Option<Expr> {
        self.pos += 1; // skip `let`
        let name = match self.bump()? {
            Token::Ident(n) => n,
            _ => return None,
        };
        if !self.eat(&Token::Eq) {
            return None;
        }
        let v = self.expr()?;
        if !self.eat(&Token::In) {
            return None;
        }
        let body = self.expr()?;
        Some(Expr::bind(name, v, body))
    }
}

/// Parse canonical source into an `Expr`, or `None` if it is not a complete, valid
/// program (the `Construction`'s partial parse).
pub(super) fn parse(src: &str) -> Option<Expr> {
    let toks = lex(&Source::new(src))?;
    let mut p = Parser { toks, pos: 0 };
    let e = p.expr()?;
    if p.pos == p.toks.len() {
        Some(e)
    } else {
        None // trailing tokens — not a single well-formed expression
    }
}

// ===== type checker (the witness `Check` mints) ===========================

fn check(e: &Expr, env: &HashMap<Ident, Ty>) -> Option<Ty> {
    match e {
        Expr::Lit(Lit::Int(_)) => Some(Ty::Int),
        Expr::Lit(Lit::Bool(_)) => Some(Ty::Bool),
        Expr::Var(name) => env.get(name).copied(),
        Expr::Bin(Op::Add | Op::Mul, a, b) => {
            if check(a, env) == Some(Ty::Int) && check(b, env) == Some(Ty::Int) {
                Some(Ty::Int)
            } else {
                None
            }
        }
        Expr::Bin(Op::Lt, a, b) => {
            if check(a, env) == Some(Ty::Int) && check(b, env) == Some(Ty::Int) {
                Some(Ty::Bool)
            } else {
                None
            }
        }
        Expr::If(c, t, f) => {
            if check(c, env) != Some(Ty::Bool) {
                return None;
            }
            let tt = check(t, env)?;
            let tf = check(f, env)?;
            if tt == tf {
                Some(tt)
            } else {
                None
            }
        }
        Expr::Let(name, v, body) => {
            let tv = check(v, env)?;
            let mut env2 = env.clone();
            env2.insert(name.clone(), tv);
            check(body, &env2)
        }
    }
}

/// Infer the type of a CLOSED expression (empty environment). `Some(ty)` iff it is
/// well-typed — no type mismatch and no unbound variable.
pub(super) fn infer(e: &Expr) -> Option<Ty> {
    check(e, &HashMap::new())
}

// ===== evaluator (total for well-typed input — the guard guarantees it) =====

fn ev(e: &Expr, env: &HashMap<Ident, Value>) -> Value {
    // The `_` arms below are unreachable for a `WellTyped` expression (the proof rules
    // out the type mismatch / unbound variable); a defined default keeps `ev` total.
    match e {
        Expr::Lit(Lit::Int(n)) => Value::Int(*n),
        Expr::Lit(Lit::Bool(b)) => Value::Bool(*b),
        Expr::Var(name) => env.get(name).copied().unwrap_or(Value::Int(Int::zero())),
        Expr::Bin(Op::Add, a, b) => match (ev(a, env), ev(b, env)) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x.plus(y)),
            _ => Value::Int(Int::zero()),
        },
        Expr::Bin(Op::Mul, a, b) => match (ev(a, env), ev(b, env)) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x.times(y)),
            _ => Value::Int(Int::zero()),
        },
        Expr::Bin(Op::Lt, a, b) => match (ev(a, env), ev(b, env)) {
            (Value::Int(x), Value::Int(y)) => Value::Bool(x.less_than(y)),
            _ => Value::Bool(false),
        },
        Expr::If(c, t, f) => match ev(c, env) {
            Value::Bool(b) => {
                if b {
                    ev(t, env)
                } else {
                    ev(f, env)
                }
            }
            _ => ev(t, env),
        },
        Expr::Let(name, v, body) => {
            let val = ev(v, env);
            let mut env2 = env.clone();
            env2.insert(name.clone(), val);
            ev(body, &env2)
        }
    }
}

/// Evaluate a CLOSED, well-typed expression to a `Value`.
pub(super) fn eval(e: &Expr) -> Value {
    ev(e, &HashMap::new())
}

// ===== constant folding (the Lossy `ConstFold` edge's engine) ==============

/// Constant-fold an expression: any binary node whose folded operands are BOTH integer
/// literals collapses to the literal `combine` computes. The recursion (which nodes are
/// reducible, how the tree is rebuilt) is honest and SHARED; only `combine` — the
/// arithmetic at a reducible node — is injected, so a wrong-coefficient edge differs from
/// the honest one by its combiner alone, not by a forked traversal. A node `combine`
/// declines (`None`) is left unfolded.
pub(super) fn fold(e: &Expr, combine: &dyn Fn(Op, Int, Int) -> Option<Lit>) -> Expr {
    match e {
        Expr::Lit(_) | Expr::Var(_) => e.clone(),
        Expr::Bin(op, a, b) => {
            let fa = fold(a, combine);
            let fb = fold(b, combine);
            if let (Expr::Lit(Lit::Int(x)), Expr::Lit(Lit::Int(y))) = (&fa, &fb) {
                if let Some(lit) = combine(*op, *x, *y) {
                    return Expr::Lit(lit);
                }
            }
            Expr::bin(*op, fa, fb)
        }
        Expr::If(c, t, f) => Expr::cond(fold(c, combine), fold(t, combine), fold(f, combine)),
        Expr::Let(name, v, body) => Expr::bind(name.clone(), fold(v, combine), fold(body, combine)),
    }
}

// ===== substitution (the Stateful `Resolve` edge's engine) =================

/// Substitute the bindings carried in `env` into an expression: every FREE `Var(name)`
/// for which `env` has a value becomes that integer literal; unbound variables and all
/// other nodes are left as-is. "Free" is load-bearing: a `let` SHADOWS its name inside its
/// body, so occurrences bound by an interior `let` must survive untouched — the walk
/// carries the set of currently-shadowed names and skips them. (Rewriting a bound
/// occurrence is variable capture, the classic substitution bug; the `Resolve` two-route
/// probe caught exactly that here once its generator learned to emit shadowing `let`s.)
/// This is the `Resolve` edge READING its carried state, which is exactly why that edge is
/// `Stateful` — its output depends on the environment, not the expression alone.
pub(super) fn subst(e: &Expr, env: &Env) -> Expr {
    fn go(e: &Expr, env: &Env, shadowed: &[Ident]) -> Expr {
        match e {
            Expr::Var(name) => {
                if shadowed.contains(name) {
                    return e.clone(); // bound by an interior `let` — not ours to rewrite
                }
                match env.get(name) {
                    Some(v) => Expr::Lit(Lit::Int(v)),
                    None => e.clone(),
                }
            }
            Expr::Lit(_) => e.clone(),
            Expr::Bin(op, a, b) => Expr::bin(*op, go(a, env, shadowed), go(b, env, shadowed)),
            Expr::If(c, t, f) => Expr::cond(
                go(c, env, shadowed),
                go(t, env, shadowed),
                go(f, env, shadowed),
            ),
            Expr::Let(name, v, body) => {
                // the bound VALUE is evaluated outside the binding, so it substitutes
                // under the outer shadow set; only the BODY sees the new name.
                let v2 = go(v, env, shadowed);
                let mut inner = shadowed.to_vec();
                inner.push(name.clone());
                Expr::bind(name.clone(), v2, go(body, env, &inner))
            }
        }
    }
    go(e, env, &[])
}
