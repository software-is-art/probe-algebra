//! genesis_relay — the committed sample declaration for the TRANSFORM-seam story.
//!
//! THIS FILE IS DATA, NOT CODE (see `genesis_demo.rs` for the pattern): a `system! { ... }`
//! block genesis parses from tokens, cfg'd out so cargo never expands it. Where the
//! `credit-app` sample demonstrates the transport seam and prose validity holes, this one
//! demonstrates the two newer bricks at once:
//!
//! * STRUCTURED validity rules — both values carry token ranges with a declared
//!   `saturating` re-entry policy, so their predicates, mints, and edge-seeking grids are
//!   GENERATED: the only meaning holes in the whole crate are the three operator interiors.
//! * A NAMED transform seam — `mixer -- gauge : transform on Signal via cook;` compiles end
//!   to end: the spanning theory, the compiled seam, the distance gate, the verdict test,
//!   and the preserved stanza in the system target lock.
//!
//! Generate the crate:
//!
//!     cargo run --example genesis -- examples/genesis_relay.rs <target-dir>
//!
//! The committed convergence lives in `relay-demo/` — generated from this file, its three
//! interior holes filled, its locks blessed.

#[cfg(any())]
system! {
    name: "relay-app",
    values {
        Signal = i64 where -8..=8 saturating;
        Level = i64 where 0..=16 saturating;
    }
    modules {
        mixer {
            ops {
                blend(Signal, Signal) -> Signal;
                cook(Signal) -> Level;
            }
            expects {
                commutative(blend);
                idempotent(blend);
            }
        }
        gauge {
            ops {
                fuse(Level, Level) -> Level;
            }
            expects {
                commutative(fuse);
            }
        }
    }
    seams {
        mixer -- gauge : transform on Signal via cook;
    }
}

fn main() {
    println!("genesis_relay is INPUT DATA — feed it to the generator:");
    println!("    cargo run --example genesis -- examples/genesis_relay.rs <target-dir>");
}
