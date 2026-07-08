//!
//! The interpreter's arithmetic, expressed as a `Theory` so its algebra discovers itself.
//!
//! Every operator routes through the REAL interpreter (`Check` then `Eval`), so the discovered laws,
//! re-probed, exercise the interpreter's interior — a mutant that breaks `eval` breaks a law.

use crate::gdp::with_seed;
use crate::interp::boundary::{Check, Eval, Expr, Int, Op, Value};

/// The sorts of the interpreter's value algebra.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Int,
    Bool,
}

/// The interpreter's arithmetic theory.
pub struct Arithmetic;

/// Evaluate a closed expression through the interpreter's own boundary (`Check` ⊳ `Eval`).
#[crate::mutate]
fn run(e: &Expr) -> Option<Value> {
    with_seed(|seed| {
        let named = seed.new_named(e.clone());
        let proof = Check.classify(&named).ok()?;
        Some(*Eval.run(&named, &proof).value())
    })
}

#[crate::mutate]
fn as_expr(v: &Value) -> Option<Expr> {
    match v {
        Value::Int(i) => Expr::int(i.get()),
        Value::Bool(b) => Some(Expr::boolean(*b)),
    }
}

#[crate::mutate]
fn bin(op: Op, vs: &[Value]) -> Option<Value> {
    run(&Expr::bin(op, as_expr(&vs[0])?, as_expr(&vs[1])?))
}

#[crate::mutate]
fn add(vs: &[Value]) -> Option<Value> {
    bin(Op::Add, vs)
}
#[crate::mutate]
fn mul(vs: &[Value]) -> Option<Value> {
    bin(Op::Mul, vs)
}
#[crate::mutate]
fn lt(vs: &[Value]) -> Option<Value> {
    bin(Op::Lt, vs)
}
#[crate::mutate]
fn int_const(n: i64) -> Option<Value> {
    Some(Value::Int(Int::new(n)?))
}
#[crate::mutate]
fn zero(_: &[Value]) -> Option<Value> {
    int_const(0)
}
#[crate::mutate]
fn one(_: &[Value]) -> Option<Value> {
    int_const(1)
}
#[crate::mutate]
fn fls(_: &[Value]) -> Option<Value> {
    Some(Value::Bool(false))
}

// The whole `Theory` impl is generated — even for the interpreter-backed theory, only the operator
// functions (which route through the real `Check` ⊳ `Eval`) and the `Sort` enum are authored.
crate::theory! {
    Arithmetic : "interpreter arithmetic", Value = Value, Obs = (u8, i64), Sort = Sort,
    sort_of = |v: &Value| match v {
        Value::Int(_) => Sort::Int,
        Value::Bool(_) => Sort::Bool,
    },
    observe = |v: &Value| match v {
        Value::Int(i) => (0u8, i.get()),
        Value::Bool(b) => (1u8, *b as i64),
    },
    vars {
        Sort::Int => &["x", "y", "z"],
        Sort::Bool => &["p", "q", "r"],
    }
    inhabit {
        Sort::Int => (0..8).map(|n| Value::Int(Int::new(n).expect("n >= 0"))).collect(),
        Sort::Bool => vec![Value::Bool(false), Value::Bool(true)],
    }
    ops {
        Nullary "Zero"           "0"     () -> Sort::Int = zero;
        Nullary "One"            "1"     () -> Sort::Int = one;
        Nullary "False"          "false" () -> Sort::Bool = fls;
        Infix   "Addition"       "+"     (Sort::Int, Sort::Int) -> Sort::Int = add;
        Infix   "Multiplication" "*"     (Sort::Int, Sort::Int) -> Sort::Int = mul;
        Infix   "less than"      "<"     (Sort::Int, Sort::Int) -> Sort::Bool = lt;
    }
}
