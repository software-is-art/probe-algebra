//! scaffold — turn a cohesion suggestion into the actual split: emit the `theory!` sub-modules and
//! the seam obligation that keeps the cut honest. The action half of the cohesion loop.
//!
//! Run `cargo run --example scaffold`.

use boundary_algebra::discover::date::Calendar;
use boundary_algebra::discover::scaffold::Scaffold;

fn main() {
    print!("{}", Scaffold::render::<Calendar>());
}
