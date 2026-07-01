//! modularize — SELECT the right shapes out of a flat bag of functions.
//!
//! The whole discovery stack, turned around and pointed at the pathological case: one module with
//! everything crammed together — functions from several unrelated algebras all thrown in a heap, no
//! structure at all. `modularize` reads the structure back OUT. It partitions the functions by
//! law-connectivity, scores each cluster by how many discovered laws bind it, and ranks them: the
//! richest algebraic shapes first, the MISFITS (functions no law ever mentions) flagged at the bottom
//! and refused as modules. A good decomposition is not designed here, it is read off the algebra.
//!
//! The `soup` bag is four functions over three unrelated types — a `max` semilattice (`peak`), an
//! `and`/`or` lattice (`both`/`either`), and a structureless three-cycle (`rotate`). `#[algebra]`
//! synthesises the whole theory from just the functions; modularize proposes the two hidden algebras
//! and flags the cycle.
//!
//! Run `cargo run --example modularize`.

use boundary_algebra::discover::modularize::{render, soup::Soup};

fn main() {
    println!("Modularize — the structure hiding in an unstructured bag of functions:\n");
    print!("{}", render::<Soup>());
    println!(
        "\nThe algebra IS the selection criterion: a cluster the laws bind tightly is a real shape\n\
         (ranked by how many laws), and a function bound by no law is a MISFIT — the proposal will\n\
         not dress it up as a module. Nothing about this decomposition was written down."
    );
}
