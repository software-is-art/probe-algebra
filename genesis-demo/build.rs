//! build.rs — the enforcement shim genesis emitted: attach the whole structural discipline
//! from `boundary-enforce`, with a config that is THIS crate's own.
//!
//! Two decisions live here and must live here:
//!
//! * **The kernel allowlist.** Claiming `Tier: KERNEL` exempts a file from every structural
//!   rule, so it cannot be self-serve — the file must ALSO be named here, where admitting a
//!   member is a reviewed diff in this crate's tree. The generated kernel is exactly
//!   `src/lib.rs` (the module roster).
//!
//! * **The qualification census.** FIRST BUILD: run `BLESS_CREDIT_APP_QUALIFY=1 cargo build`
//!   once to mint `spec/qualify.spec` — a missing lock is stale, never fresh, so an unblessed
//!   tree refuses to build. From then on the census is drift-gated; regenerate with the same
//!   variable and ratify the diff.

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// The RATIFIED kernel of THIS crate — the only files allowed to declare `Tier: KERNEL`.
const KERNEL_ALLOWLIST: &[&str] = &["src/lib.rs"];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = KERNEL_ALLOWLIST.iter().map(|s| s.to_string()).collect();
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.bless_env = "BLESS_CREDIT_APP_QUALIFY".to_string();
    Enforcement::enforce_or_panic(&config);
}
