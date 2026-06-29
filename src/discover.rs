//! discover — the laws WRITE (and READ) themselves.
//!
//! The interpreter's algebraic spec is no longer hand-listed. `discover_laws()` takes the
//! operators (`Add`, `Mul`, `Lt`) and the value objects' canonical constants, mechanically
//! INSTANTIATES the universal algebraic shapes over them (identity, commutativity, associativity,
//! annihilation, distributivity, irreflexivity), and keeps exactly the ones that HOLD — discovered
//! by RUNNING `eval` over a deterministic grid of inputs, not asserted by a human. The author
//! names no structure and writes no law; which laws hold falls out of the operators' behaviour.
//!
//! Each held law carries a NON-MATHY sentence (`Adding zero leaves a value unchanged`), so the
//! discovered set is a spec a non-mathematical stakeholder can audit — and ratify: a law you
//! expect but DON'T see in the output (e.g. a folder that doubled would have no additive identity)
//! is a bug surfaced. The held laws also feed the `harness` as probes, where mutation judges their
//! kill power; `discover` itself is kept in the mutation sweep, certified by the probes below.
//!
//! Reference frame: discovery's oracle is the baseline, so its power is deviation-catching
//! (mutation) plus expectation-checking (ratification) — it cannot conjure a law the operators
//! don't exhibit. That is the precise edge where "the tests write themselves" meets "what did you
//! mean".

use crate::gdp::with_seed;
use crate::interp::boundary::{Check, Eval, Expr, Op, Value};

/// A law that held on every tested input, with its plain-language and symbolic renderings and a
/// `schema` that rebuilds both sides for any inputs (so the harness can re-probe it and mutation
/// can judge it).
pub struct Law {
    /// Non-mathy statement, for the readable spec.
    pub prose: String,
    /// The equation in symbols, for readers who want it.
    pub equation: String,
    /// Build the two sides of the equation for concrete inputs `(x, y, z)`.
    pub schema: Box<dyn Fn(i64, i64, i64) -> (Expr, Expr)>,
}

/// A non-negative literal (grid inputs and constants are non-negative, so `Int::new` succeeds).
fn lit(n: i64) -> Expr {
    Expr::int(n).expect("grid inputs are non-negative")
}

/// Evaluate a closed, well-typed expression through the boundary (`Check` then `Eval`).
fn eval_closed(e: &Expr) -> Value {
    with_seed(|seed| {
        let named = seed.new_named(e.clone());
        let proof = Check
            .classify(&named)
            .expect("a closed int/bool expr is well-typed");
        *Eval.run(&named, &proof).value()
    })
}

/// Does the equation hold on every grid assignment? The discovery test: run both sides through
/// `eval` over a fixed grid and compare. A law survives only if it holds EVERYWHERE on the grid.
fn holds(schema: &dyn Fn(i64, i64, i64) -> (Expr, Expr)) -> bool {
    const GRID: &[i64] = &[0, 1, 2, 3, 5, 7];
    for &a in GRID {
        for &b in GRID {
            for &c in GRID {
                let (l, r) = schema(a, b, c);
                if eval_closed(&l) != eval_closed(&r) {
                    return false;
                }
            }
        }
    }
    true
}

/// The arithmetic operators and the words that render them.
fn arithmetic() -> [(Op, &'static str, &'static str, &'static str); 2] {
    // (op, symbol, gerund, "identity phrase")
    [
        (Op::Add, "+", "Addition", "Adding zero"),
        (Op::Mul, "*", "Multiplication", "Multiplying by one"),
    ]
}

/// The CANDIDATE catalog: the universal algebraic shapes, mechanically instantiated over the
/// operator set. The author writes none of these — they are the search space; `discover_laws`
/// keeps only those that run true.
fn candidates() -> Vec<Law> {
    let mut v: Vec<Law> = Vec::new();

    for (op, sym, word, ident_phrase) in arithmetic() {
        let identity = if op == Op::Add { 0 } else { 1 };

        // IDENTITY: `x op e == x`.
        v.push(Law {
            prose: format!("{ident_phrase} leaves a value unchanged."),
            equation: format!("x {sym} {identity} = x"),
            schema: Box::new(move |a, _, _| (Expr::bin(op, lit(a), lit(identity)), lit(a))),
        });

        // COMMUTATIVITY: `x op y == y op x`.
        v.push(Law {
            prose: format!("{word} gives the same result in either order."),
            equation: format!("x {sym} y = y {sym} x"),
            schema: Box::new(move |a, b, _| {
                (Expr::bin(op, lit(a), lit(b)), Expr::bin(op, lit(b), lit(a)))
            }),
        });

        // ASSOCIATIVITY: `(x op y) op z == x op (y op z)`.
        v.push(Law {
            prose: format!("When combining three values with {word}, the grouping doesn't matter."),
            equation: format!("(x {sym} y) {sym} z = x {sym} (y {sym} z)"),
            schema: Box::new(move |a, b, c| {
                (
                    Expr::bin(op, Expr::bin(op, lit(a), lit(b)), lit(c)),
                    Expr::bin(op, lit(a), Expr::bin(op, lit(b), lit(c))),
                )
            }),
        });
    }

    // ANNIHILATION: `x * 0 == 0`.
    v.push(Law {
        prose: "Multiplying by zero always gives zero.".to_string(),
        equation: "x * 0 = 0".to_string(),
        schema: Box::new(|a, _, _| (Expr::bin(Op::Mul, lit(a), lit(0)), lit(0))),
    });

    // DISTRIBUTIVITY: `x * (y + z) == x*y + x*z`.
    v.push(Law {
        prose: "Multiplying a sum is the same as multiplying each part and adding the results."
            .to_string(),
        equation: "x * (y + z) = (x * y) + (x * z)".to_string(),
        schema: Box::new(|a, b, c| {
            (
                Expr::bin(Op::Mul, lit(a), Expr::bin(Op::Add, lit(b), lit(c))),
                Expr::bin(
                    Op::Add,
                    Expr::bin(Op::Mul, lit(a), lit(b)),
                    Expr::bin(Op::Mul, lit(a), lit(c)),
                ),
            )
        }),
    });

    // IRREFLEXIVITY of `<`: `x < x == false`.
    v.push(Law {
        prose: "A value is never less than itself.".to_string(),
        equation: "x < x = false".to_string(),
        schema: Box::new(|a, _, _| (Expr::bin(Op::Lt, lit(a), lit(a)), Expr::boolean(false))),
    });

    v
}

/// The discovered spec: every candidate that RAN true. The author supplied only the operators and
/// constants (already in `interp`); the law set is found by running, not declared.
pub fn discover_laws() -> Vec<Law> {
    candidates()
        .into_iter()
        .filter(|l| holds(&l.schema))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Discovery RUNS: the catalog's true shapes are found. Pins `holds`/`eval_closed`/the catalog
    /// against a constant-return mutant — a `holds`-always-false mutant empties the set.
    #[test]
    fn discovery_finds_the_arithmetic_laws() {
        let found: Vec<String> = discover_laws().into_iter().map(|l| l.equation).collect();
        for expected in [
            "x + 0 = x",
            "x * 1 = x",
            "x + y = y + x",
            "x * y = y * x",
            "(x + y) + z = x + (y + z)",
            "x * 0 = 0",
            "x * (y + z) = (x * y) + (x * z)",
            "x < x = false",
        ] {
            assert!(found.iter().any(|e| e == expected), "missing: {expected}");
        }
    }

    /// Discovery DISCRIMINATES: a FALSE candidate is rejected — so a `holds`-always-true mutant
    /// (which would keep everything) is killed. `x + 1 == x` is false, and `x + x == x` (a bogus
    /// idempotence) is false, so neither may appear.
    #[test]
    fn discovery_rejects_false_laws() {
        // `x + 1 == x` is false for every x.
        assert!(!holds(&|a, _, _| (
            Expr::bin(Op::Add, lit(a), lit(1)),
            lit(a)
        )));
        // `x * (y + z) == x*y + z` (a broken distributivity) is false.
        assert!(!holds(&|a, b, c| {
            (
                Expr::bin(Op::Mul, lit(a), Expr::bin(Op::Add, lit(b), lit(c))),
                Expr::bin(Op::Add, Expr::bin(Op::Mul, lit(a), lit(b)), lit(c)),
            )
        }));
    }

    /// `Add` is NOT the same shape as `Mul`: addition has no annihilator, so `x + 0 == 0` is false
    /// — pinning that the grid actually varies `x` (a single-point grid would pass it vacuously).
    #[test]
    fn the_grid_actually_varies_the_input() {
        assert!(!holds(&|a, _, _| (
            Expr::bin(Op::Add, lit(a), lit(0)),
            lit(0)
        )));
    }

    /// Every discovered law renders to a non-empty plain sentence AND a symbolic equation — pins
    /// the prose/equation against an empty-string mutant.
    #[test]
    fn discovered_laws_render_readably() {
        for law in discover_laws() {
            assert!(!law.prose.is_empty(), "a law has no prose");
            assert!(law.equation.contains('='), "a law has no equation");
            assert!(
                law.prose.ends_with('.'),
                "prose should be a sentence: {}",
                law.prose
            );
        }
    }
}
