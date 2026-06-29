//! discovered_spec — print the interpreter's algebraic spec, DISCOVERED by running the operators.
//!
//! Nothing here is hand-written: `discover::discover_laws()` instantiated the universal algebraic
//! shapes over the operators and kept the ones that ran true. The output is a plain-language
//! contract a non-mathematical stakeholder can read and ratify — and an *expected* law that is
//! ABSENT (e.g. if the folder doubled, there would be no "adding zero leaves a value unchanged")
//! is a bug surfaced.
//!
//! Run `cargo run --example discovered_spec`.

use boundary_algebra::discover::interpreter_spec;

fn main() {
    let spec = interpreter_spec();
    println!(
        "The {} obeys these laws (discovered by running it):\n",
        spec.theory
    );
    for law in &spec.laws {
        println!("  • {}", law.prose());
        println!("      {}", law.equation());
    }
    println!(
        "\n{} named laws, none hand-written — found by the generic engine enumerating terms over \
         the operators and keeping the equalities that ran true.",
        spec.laws.len()
    );
    println!(
        "({} further equalities were discovered too — every one a consequence of the laws above, \
         so they are counted, not listed.)",
        spec.consequences
    );
    if spec.uncovered_ops.is_empty() {
        println!("Every operator participates in a law.");
    } else {
        println!(
            "Operators in no law (where the spec is silent): {:?}",
            spec.uncovered_ops
        );
    }
}
