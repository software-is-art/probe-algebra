//! scorecard — print the two engines' distance reports side by side: the same three
//! declared laws, each engine red on exactly its own axis. The comparability is the
//! product: swap the engine, keep the law set, and the tradeoff reads as a diff.
//!
//!     cargo run -p layout-probe --example scorecard

use boundary_spec::discover::expect::Distance;
use layout_probe::theories::{EagerLayout, StableLayout};
fn main() {
    println!("STABLE: {}", Distance::of::<StableLayout>().render());
    println!("EAGER:  {}", Distance::of::<EagerLayout>().render());
}
