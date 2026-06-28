//! Tests for the boundary algebra, exercised through the interpreter — the sole
//! demonstration substrate. These import ONLY through `interp::boundary`, exactly as
//! another module would, and test the PUBLIC edges (`Parse`/`Check`/`Eval`). The
//! lexer/parser/checker/evaluator in `interp::internal` have no tests of their own; this
//! file plus the autogen `laws` registry are the entire rigour they get, and mutation
//! testing measures how much that buys.

use crate::boundary::{
    fits, require_complete, require_within, CostCons, CostNil, Fold, MapCollect, SpaceCost,
    TimeCost, S, Z,
};
use crate::gdp::with_seed;
use crate::interp::boundary::{
    Check, Depth, Eval, Expr, FullProbe, Ident, Int, Nodes, Op, Parse, Value,
};

fn name(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn int(v: i64) -> Expr {
    Expr::int(v).unwrap()
}
fn num(v: i64) -> Value {
    Value::Int(Int::new(v).unwrap())
}

/// Parse → Check → Eval through the boundary, returning the value. Panics if the
/// program is ill-typed (the test feeds only well-typed programs).
fn eval(expr: Expr) -> Value {
    with_seed(|seed| {
        let named = seed.new_named(expr);
        let proof = Check.classify(&named).expect("well-typed");
        *Eval.run(&named, &proof).value()
    })
}

/// Does the named expression type-check (the `Branch`'s positive arm)?
fn well_typed(expr: Expr) -> bool {
    with_seed(|seed| {
        let named = seed.new_named(expr);
        Check.classify(&named).is_ok()
    })
}

#[test]
fn evaluates_arithmetic() {
    assert_eq!(eval(int(2)), num(2));
    assert_eq!(eval(Expr::bin(Op::Add, int(2), int(3))), num(5));
    assert_eq!(eval(Expr::bin(Op::Mul, int(2), int(3))), num(6));
}

#[test]
fn evaluates_comparison_and_if() {
    assert_eq!(eval(Expr::bin(Op::Lt, int(1), int(2))), Value::Bool(true));
    assert_eq!(eval(Expr::bin(Op::Lt, int(2), int(1))), Value::Bool(false));
    // EQUAL operands pin `<` against `<=` (strict less-than).
    assert_eq!(eval(Expr::bin(Op::Lt, int(2), int(2))), Value::Bool(false));
    let cond = Expr::bin(Op::Lt, int(1), int(2));
    assert_eq!(eval(Expr::cond(cond.clone(), int(7), int(9))), num(7));
    let cond_false = Expr::bin(Op::Lt, int(2), int(1));
    assert_eq!(eval(Expr::cond(cond_false, int(7), int(9))), num(9));
}

#[test]
fn evaluates_let_and_var() {
    let prog = Expr::bind(
        name("x"),
        int(4),
        Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
    );
    assert_eq!(eval(prog), num(5));
    let shadow = Expr::bind(
        name("x"),
        int(1),
        Expr::bind(name("x"), int(2), Expr::var(name("x"))),
    );
    assert_eq!(eval(shadow), num(2));
    let square = Expr::bind(
        name("x"),
        int(3),
        Expr::bin(Op::Mul, Expr::var(name("x")), Expr::var(name("x"))),
    );
    assert_eq!(eval(square), num(9));
}

/// The value-object accessors and the canonical `render` report the REAL contents, not a
/// constant. (`render` is pinned here directly because the round-trip law's generator is
/// itself built from `render`.)
#[test]
fn accessors_and_render_report_the_real_value() {
    assert_eq!(Int::new(7).unwrap().get(), 7);
    assert_eq!(name("foo").get(), "foo");
    assert_eq!(Expr::bin(Op::Add, int(1), int(2)).render(), "(1 + 2)");
    assert_eq!(Expr::bin(Op::Mul, int(2), int(3)).render(), "(2 * 3)");
    assert_eq!(Expr::bin(Op::Lt, int(1), int(2)).render(), "(1 < 2)");
    assert_eq!(
        Expr::cond(Expr::boolean(true), int(1), int(2)).render(),
        "(if true then 1 else 2)"
    );
    assert_eq!(
        Expr::bind(name("x"), int(1), Expr::var(name("x"))).render(),
        "(let x = 1 in x)"
    );
}

#[test]
fn accepts_well_typed() {
    assert!(well_typed(Expr::bin(Op::Add, int(1), int(2))));
    assert!(well_typed(Expr::bin(Op::Lt, int(1), int(2))));
    let if_ok = Expr::cond(Expr::bin(Op::Lt, int(1), int(2)), int(1), int(2));
    assert!(well_typed(if_ok));
    let let_ok = Expr::bind(
        name("x"),
        int(1),
        Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
    );
    assert!(well_typed(let_ok));
}

#[test]
fn rejects_ill_typed() {
    assert!(!well_typed(Expr::bin(Op::Add, int(1), Expr::boolean(true))));
    assert!(!well_typed(Expr::bin(Op::Add, Expr::boolean(true), int(1))));
    assert!(!well_typed(Expr::bin(Op::Mul, int(1), Expr::boolean(true))));
    assert!(!well_typed(Expr::bin(Op::Lt, int(1), Expr::boolean(true))));
    assert!(!well_typed(Expr::cond(int(1), int(2), int(3))));
    assert!(!well_typed(Expr::cond(
        Expr::bin(Op::Lt, int(1), int(2)),
        int(1),
        Expr::boolean(true)
    )));
    assert!(!well_typed(Expr::var(name("y"))));
    let bad_let = Expr::bind(
        name("x"),
        Expr::boolean(true),
        Expr::bin(Op::Add, Expr::var(name("x")), int(1)),
    );
    assert!(!well_typed(bad_let));
}

#[test]
fn parses_and_runs_a_program() {
    let e = Parse
        .parse_str("(let x = 5 in (if (x < 10) then (x + 1) else 0))")
        .unwrap();
    assert_eq!(eval(e), num(6));
}

#[test]
fn parse_builds_the_expected_structure() {
    assert_eq!(
        Parse.parse_str("(1 + (2 * 3))").unwrap(),
        Expr::bin(Op::Add, int(1), Expr::bin(Op::Mul, int(2), int(3)))
    );
    assert_eq!(Parse.parse_str("true").unwrap(), Expr::boolean(true));
    assert_eq!(
        Parse.parse_str("(x < y)").unwrap(),
        Expr::bin(Op::Lt, Expr::var(name("x")), Expr::var(name("y")))
    );
}

#[test]
fn rejects_malformed_source() {
    assert!(Parse.parse_str("1 +").is_none());
    assert!(Parse.parse_str("(1 + 2").is_none());
    assert!(Parse.parse_str("(1 + 2) extra").is_none());
    assert!(Parse.parse_str("@").is_none());
    assert!(Parse.parse_str("()").is_none());
    assert!(Parse.parse_str("(1 2 3)").is_none());
}

// ===== cost grading on the interp edges ===================================

/// A deterministic measure for the `fits` audit: build the sum `1 + 1 + ... + 1` of `n`
/// ones as a left-nested chain and evaluate it THROUGH the boundary. Each `+` adds one to
/// both the node count and the result, so the measured result tracks `Eval`'s real step
/// count — non-tautological because it actually runs the evaluator end to end.
fn eval_steps(n: usize) -> u64 {
    let mut e = int(1);
    for _ in 1..n {
        e = Expr::bin(Op::Add, e, int(1));
    }
    match eval(e) {
        Value::Int(i) => i.get() as u64,
        Value::Bool(_) => unreachable!("a sum of ints evaluates to an int"),
    }
}

/// The empirical honesty check: `Eval`'s declared `Nodes` time degree is `S<Z>` (linear).
/// `fits` confirms the real growth is linear (degree 1) and NOT constant (degree 0), so the
/// leaf's declared degree matches reality — the audit the type level cannot perform itself.
#[test]
fn eval_is_empirically_linear_in_nodes() {
    assert!(
        fits(eval_steps, 1),
        "eval should grow linearly in node count"
    );
    assert!(
        !fits(eval_steps, 0),
        "eval is not constant — a degree-0 claim must be rejected"
    );
}

/// The per-axis map earns its keep here: `Eval`'s TIME is keyed on `Nodes` while its SPACE
/// is keyed on `Depth`, so the two budgets are demanded independently. A scalar cost could
/// not separate them. `require_within` is a compile-time demand; reaching this line means
/// both bounds hold.
#[test]
fn eval_time_and_space_axes_are_within_budget() {
    // Eval: linear in Nodes (time), linear in Depth (space).
    require_within::<TimeCost, Eval, CostCons<Nodes, S<Z>, CostNil>>();
    require_within::<SpaceCost, Eval, CostCons<Depth, S<Z>, CostNil>>();
    // Parse: linear in Nodes in both resources.
    require_within::<TimeCost, Parse, CostCons<Nodes, S<Z>, CostNil>>();
    require_within::<SpaceCost, Parse, CostCons<Nodes, S<Z>, CostNil>>();
    // Folding Eval per node STREAMS: time gains a Nodes degree (now quadratic) but space
    // stays linear in Depth — the "stream, don't materialize" fact, per-axis.
    require_within::<TimeCost, Fold<Eval, Nodes>, CostCons<Nodes, S<S<Z>>, CostNil>>();
    require_within::<SpaceCost, Fold<Eval, Nodes>, CostCons<Depth, S<Z>, CostNil>>();
    // Collecting Eval per node MATERIALIZES n results: BOTH time and space gain a Nodes
    // degree, so space is now quadratic in Nodes too.
    require_within::<TimeCost, MapCollect<Eval, Nodes>, CostCons<Nodes, S<S<Z>>, CostNil>>();
    require_within::<SpaceCost, MapCollect<Eval, Nodes>, CostCons<Nodes, S<S<Z>>, CostNil>>();
}

// ===== degrees of freedom: the static completeness demand =================

/// A probe that reaches BOTH of `Expr`'s declared DOFs (shape and literals) satisfies
/// `require_complete` — the positive of `tests/compile_fail/incomplete_probe_rejected`,
/// where a shape-only probe is rejected for missing the `Literals` dimension.
#[test]
fn a_complete_probe_covers_every_dof() {
    require_complete::<Expr, _>(&FullProbe);
}

/// The headline: the brand minted at `with_seed` threads through parse, the `Check`
/// branch, and the `Eval` guard. Only `Check` can mint the `WellTyped` witness `Eval`
/// demands, and the shared name ties them to THIS program (a proof for another program
/// will not type-check — pinned in `tests/compile_fail/eval_wrong_program`).
#[test]
fn the_brand_threads_parse_check_eval() {
    with_seed(|seed| {
        let named = seed.new_named(Parse.parse_str("(2 + 3)").unwrap());
        match Check.classify(&named) {
            Ok(proof) => {
                let result = Eval.run(&named, &proof);
                assert_eq!(result.value(), &num(5));
            }
            Err(_) => panic!("(2 + 3) is well-typed"),
        }
    });
}
