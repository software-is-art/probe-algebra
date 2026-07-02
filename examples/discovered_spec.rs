//! discovered_spec — print every theory's algebraic spec, DISCOVERED by running its operators.
//!
//! Nothing here is hand-written: the generic engine (`discover::engine`) instantiated the universal
//! algebraic shapes over each domain's operators and kept the ones that ran true. The interpreter's
//! arithmetic, a non-commutative router monoid, and a multi-sorted date calculus all fall out of the
//! SAME mechanism — and each renders as a plain-language contract a non-mathematical stakeholder can
//! ratify. An expected law that is ABSENT (or, for the router, a commutativity that must NOT appear)
//! is a bug surfaced. The committed `spec/*.spec` locks freeze these; CI fails if they drift.
//!
//! Run `cargo run --example discovered_spec`.

use boundary_spec::discover::all_specs;

fn main() {
    for spec in all_specs() {
        println!("══ {} ══\n", spec.theory);
        for law in &spec.laws {
            println!("  • {}", law.prose());
            println!("      {}", law.equation());
        }
        println!(
            "\n  {} named laws (none hand-written) + {} further consequence equalities.",
            spec.laws.len(),
            spec.consequences
        );
        if spec.uncovered_ops.is_empty() {
            println!("  Every operator participates in a law.\n");
        } else {
            println!(
                "  Operators in no law (where the spec is silent): {}\n",
                spec.uncovered_ops.join(", ")
            );
        }
    }
    println!(
        "All discovered by running the operators — one engine, three very different algebras."
    );
}
