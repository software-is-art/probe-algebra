//!
//! demo — exercises the boundary algebra THROUGH the interpreter, the sole demonstration
//! substrate. The bin names only `boundary_spec::boundary` (the grammar) and
//! `boundary_spec::interp::boundary` (the interpreter's interface); the lexer, parser,
//! type checker, and evaluator in `interp::internal` are private and unreachable here.

use boundary_spec::gdp::with_seed;
use boundary_spec::interp::boundary::{Check, Eval, Parse, Value};

fn banner(s: &str) {
    println!("\n=== {s} ===");
}
fn yn(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "NO"
    }
}

fn main() {
    banner("BOUNDARY: the interpreter's seam is a category of edges");
    println!("  Parse  : Construction  String -> Expr        (parse, don't validate)");
    println!("  Check  : Branch        Expr   -> WellTyped + IllTyped");
    println!("  Eval   : Guarded       Expr   -> Value        (needs a WellTyped witness)");
    println!("  interp::internal (lexer/parser/checker/evaluator) is PRIVATE — unreachable.");

    let src = "(let x = 5 in (if (x < 10) then (x + 1) else 0))";

    banner("CONSTRUCTION: parse the canonical source, and render it back");
    let expr = Parse.parse_str(src).expect("valid program");
    println!("  source     : {src}");
    println!("  re-rendered : {}", expr.render());
    println!("  render . parse == id : {}", yn(expr.render() == src));

    banner("BRANCH then GUARDED: type-check mints the witness eval demands");
    with_seed(|seed| {
        let named = seed.new_named(expr.clone());
        match Check.classify(&named) {
            Ok(proof) => {
                let value = Eval.run(&named, &proof);
                let shown = match value.value() {
                    Value::Int(n) => n.get().to_string(),
                    Value::Bool(b) => b.to_string(),
                };
                println!("  well-typed -> evaluates to : {shown}");
            }
            Err(_) => println!("  ill-typed (unreachable for this program)"),
        }
    });

    banner("WELL-TYPED PROGRAMS DON'T GO WRONG (as a compile-time fact)");
    println!("  Eval::run requires a WellTyped<N> for the SAME brand as the expression,");
    println!("  so an unchecked or ill-typed program cannot be evaluated at all —");
    println!("  see tests/compile_fail/eval_wrong_program (a proof for A cannot eval B).");

    banner("REJECTION: an ill-typed program never type-checks");
    let bad = Parse.parse_str("(1 + true)").expect("parses");
    let well_typed = with_seed(|seed| Check.classify(&seed.new_named(bad)).is_ok());
    println!("  (1 + true) type-checks : {}", yn(well_typed));

    banner("DONE");
}
