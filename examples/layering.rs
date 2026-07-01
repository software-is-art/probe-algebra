//! layering — the OTHER architectural pressure: read each module's discovered algebra for SPRAWL.
//!
//! `cohesion` finds modules that are secretly several (disconnected algebras → split). `layering`
//! finds modules that are one connected algebra but hold together only through a load-bearing
//! operator — a HINGE (a graph articulation point). The hinge is the natural seam to introduce a
//! layer. A component with no hinge is atomic; keep it. Like cohesion, it constrains nothing — it is
//! a suggestion a human (or an agent, naming as it goes) ratifies, read off the algebra structurally
//! with no threshold to tune.
//!
//! Run `cargo run --example layering`.

use boundary_algebra::discover::arithmetic::Arithmetic;
use boundary_algebra::discover::date::Calendar;
use boundary_algebra::discover::layering::render;
use boundary_algebra::discover::router::Router;

fn main() {
    println!("Layering analysis — the discovered algebra read for sprawl:\n");
    print!("{}", render::<Arithmetic>());
    print!("{}", render::<Router>());
    print!("{}", render::<Calendar>());
    println!(
        "\nA HINGE is an operator whose removal would disconnect the algebra — the rest holds\n\
         together only through it, so it is where a layer wants to go. Atomic components have none."
    );
}
