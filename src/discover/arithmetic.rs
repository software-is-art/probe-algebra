//! The interpreter's arithmetic, expressed as a `Theory` so its algebra discovers itself.
//!
//! Every operator routes through the REAL interpreter (`Check` then `Eval`), so the discovered laws,
//! re-probed, exercise the interpreter's interior — a mutant that breaks `eval` breaks a law.

use super::engine::{Fixity, Operator, Theory};
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
fn run(e: &Expr) -> Option<Value> {
    with_seed(|seed| {
        let named = seed.new_named(e.clone());
        let proof = Check.classify(&named).ok()?;
        Some(*Eval.run(&named, &proof).value())
    })
}

fn as_expr(v: &Value) -> Option<Expr> {
    match v {
        Value::Int(i) => Expr::int(i.get()),
        Value::Bool(b) => Some(Expr::boolean(*b)),
    }
}

fn bin(op: Op, vs: &[Value]) -> Option<Value> {
    run(&Expr::bin(op, as_expr(&vs[0])?, as_expr(&vs[1])?))
}

fn add(vs: &[Value]) -> Option<Value> {
    bin(Op::Add, vs)
}
fn mul(vs: &[Value]) -> Option<Value> {
    bin(Op::Mul, vs)
}
fn lt(vs: &[Value]) -> Option<Value> {
    bin(Op::Lt, vs)
}
fn int_const(n: i64) -> Option<Value> {
    Some(Value::Int(Int::new(n)?))
}
fn zero(_: &[Value]) -> Option<Value> {
    int_const(0)
}
fn one(_: &[Value]) -> Option<Value> {
    int_const(1)
}
fn fls(_: &[Value]) -> Option<Value> {
    Some(Value::Bool(false))
}

impl Theory for Arithmetic {
    type Sort = Sort;
    type Value = Value;
    type Obs = (u8, i64);

    fn name() -> &'static str {
        "interpreter arithmetic"
    }

    fn operators() -> Vec<Operator<Self>> {
        use Fixity::{Infix, Nullary};
        vec![
            Operator {
                name: "Zero",
                symbol: "0",
                fixity: Nullary,
                inputs: vec![],
                output: Sort::Int,
                eval: zero,
            },
            Operator {
                name: "One",
                symbol: "1",
                fixity: Nullary,
                inputs: vec![],
                output: Sort::Int,
                eval: one,
            },
            Operator {
                name: "False",
                symbol: "false",
                fixity: Nullary,
                inputs: vec![],
                output: Sort::Bool,
                eval: fls,
            },
            Operator {
                name: "Addition",
                symbol: "+",
                fixity: Infix,
                inputs: vec![Sort::Int, Sort::Int],
                output: Sort::Int,
                eval: add,
            },
            Operator {
                name: "Multiplication",
                symbol: "*",
                fixity: Infix,
                inputs: vec![Sort::Int, Sort::Int],
                output: Sort::Int,
                eval: mul,
            },
            Operator {
                name: "less than",
                symbol: "<",
                fixity: Infix,
                inputs: vec![Sort::Int, Sort::Int],
                output: Sort::Bool,
                eval: lt,
            },
        ]
    }

    fn inhabitants(sort: Self::Sort) -> Vec<Self::Value> {
        match sort {
            Sort::Int => (0..8)
                .map(|n| Value::Int(Int::new(n).expect("n >= 0")))
                .collect(),
            Sort::Bool => vec![Value::Bool(false), Value::Bool(true)],
        }
    }

    fn sort_of(value: &Self::Value) -> Self::Sort {
        match value {
            Value::Int(_) => Sort::Int,
            Value::Bool(_) => Sort::Bool,
        }
    }

    fn observe(value: &Self::Value) -> Self::Obs {
        match value {
            Value::Int(i) => (0, i.get()),
            Value::Bool(b) => (1, *b as i64),
        }
    }

    fn sort_vars(sort: Self::Sort) -> &'static [&'static str] {
        match sort {
            Sort::Int => &["x", "y", "z"],
            Sort::Bool => &["p", "q", "r"],
        }
    }
}
