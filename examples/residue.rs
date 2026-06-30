//! residue — equivalent mutants surfaced as a simplification signal, not buried in config.
//!
//! Mutation testing leaves a residue no probe can kill: the EQUIVALENT mutant, behaviourally
//! identical to the original. Deciding equivalence is undecidable, so the residue is ratified by
//! hand — but it is a FINDING, not noise. A behaviourally-inert expression is either a REDUNDANT
//! guard (simplify it away — and the mutant is eliminated, not excluded) or a FREE CHOICE (accept it,
//! or tighten the spec). This classifies and surfaces them, redundant ones first, the same way
//! cohesion and layering surface their suggestions — and a drift gate keeps the list in lockstep with
//! the carve-outs the mutation gate actually applies.
//!
//! Run `cargo run --example residue`.

use boundary_algebra::discover::residue::{render, simplifiable};

fn main() {
    print!("{}", render());
    let simp = simplifiable();
    println!(
        "\n{} of them are REDUNDANT — simplify them away and the carve-out disappears; the rest are\n\
         genuine free choices the spec does not constrain. The drift gate fails CI if this list and\n\
         `.cargo/mutants.toml` ever disagree, so equivalents cannot accumulate undocumented.",
        simp.len()
    );
}
