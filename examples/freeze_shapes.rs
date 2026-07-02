//! freeze_shapes — regenerate the committed shape-catalog lock `spec/shapes.spec`.
//!
//! The catalog is the engine's LAW-LANGUAGE: every law any theory's discovered spec can state is
//! an instance of one of its shapes, so adding or changing a shape changes what EVERY consumer's
//! discovered spec contains. Run this when the template battery legitimately grows or changes;
//! review the diff it produces as the RATIFICATION of the new law-language (and note it in the
//! release contract). CI's drift gate (`engine::tests::the_committed_shape_catalog_is_fresh`)
//! fails if the committed lock is out of date, so a shape can never land silently. The write
//! itself is `spec_lock::bless` — the generic regeneration path; `ShapeCatalog::lock` supplies
//! this repo's artifact (path + rendered text).
//!
//! Run `cargo run --example freeze_shapes`.

use boundary_spec::discover::engine::ShapeCatalog;

fn main() {
    let lock = ShapeCatalog::lock();
    spec_lock::bless(std::slice::from_ref(&lock)).expect("write the shape catalog lock");
    println!(
        "froze {} ({} shapes)",
        lock.path.display(),
        ShapeCatalog::inventory().len()
    );
}
