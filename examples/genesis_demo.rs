//! genesis_demo — the committed SAMPLE DECLARATION for the blank-slate generator.
//!
//! THIS FILE IS DATA, NOT CODE. It is the input document `discover::genesis` parses:
//! one compact `system! { ... }` declaration — value objects, modules, operator signatures,
//! declared law expectations, seams — from which the generator derives an entire crate layout
//! (the `downstream-fixture` shape). There is no `system!` macro to expand; the block below is
//! cfg'd out (`any()` is never true) so cargo never tries, and genesis reads the TOKENS with
//! `syn`. The `main` at the bottom exists only so cargo accepts the file as an example.
//!
//! Generate the crate:
//!
//!     cargo run --example genesis -- examples/genesis_demo.rs /tmp/credit-app
//!
//! Grammar: see the header of `src/discover/genesis.rs` — every production is documented there.
//! The `where` strings here are validity-rule HOLES: prose carried verbatim into the generated
//! stubs as doc'd `todo!()` predicates — this sample keeps them prose ON PURPOSE, to exercise
//! the hole-filling story end to end (the committed `genesis-demo/` is its convergence).
//! STRUCTURED rules exist too and generate their own transcription: `Credits = i64 where
//! 0..=20 saturating;` would derive the predicate, the clamping `mint`, and the edge-seeking
//! `Shaped` grid, leaving only the operator interiors as meaning. Meaning is never generated.

#[cfg(any())]
system! {
    name: "credit-app",
    values {
        Credits = i64 where "0..=20";
        Receipt = String where "non-empty";
    }
    modules {
        meter {
            ops {
                zero() -> Credits;
                grant(Credits, Credits) -> Credits;
                renew(Credits, Credits) -> Credits;
            }
            expects {
                commutative(grant);
                identity(grant, zero);
                bias_later(renew);
            }
        }
        billing {
            ops {
                charge(Credits, Receipt) -> Credits;
            }
        }
    }
    seams {
        meter -- billing : transport on Credits;
    }
}

fn main() {
    println!("genesis_demo is INPUT DATA — feed it to the generator:");
    println!("    cargo run --example genesis -- examples/genesis_demo.rs <target-dir>");
}
