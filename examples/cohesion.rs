//! cohesion — turn the method on its own architecture: read each module's discovered algebra for
//! DECOMPOSABILITY, and suggest where it wants to split (and what kind of seam the split would be).
//!
//! Nothing here constrains anything — it is a selection pressure toward cohesion, a suggestion a
//! human (or an agent, naming as it goes) ratifies. A module whose algebra is one connected whole
//! is cohesive; one whose operators fall into clusters no law connects is secretly several modules.
//!
//! Run `cargo run --example cohesion`.

use boundary_algebra::discover::arithmetic::Arithmetic;
use boundary_algebra::discover::cohesion::render;
use boundary_algebra::discover::date::Calendar;
use boundary_algebra::discover::router::Router;

fn main() {
    println!("Cohesion analysis — the discovered algebra read for decomposability:\n");
    print!("{}", render::<Arithmetic>());
    print!("{}", render::<Router>());
    print!("{}", render::<Calendar>());
    println!(
        "\nEach split is a SUGGESTION. A transport seam keeps the algebra (check it with coherence);\n\
         a transform seam changes it (check it with the homomorphism law) — the latent layer line."
    );
}
