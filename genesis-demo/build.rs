//! build.rs — the enforcement shim genesis emitted: attach the whole structural discipline
//! from `boundary-enforce`, with a config that is THIS crate's own.
//!
//! Two decisions live here and must live here:
//!
//! * **The kernel allowlist.** KERNEL exempts a file from every structural rule, so it is a
//!   RATIFICATION, never derived from the file itself — membership is named here, where
//!   admitting a member is a reviewed diff in this crate's tree. The generated kernel is
//!   exactly `src/lib.rs` (the module roster).
//!
//! * **The two censuses.** FIRST BUILD: run `BLESS_CREDIT_APP_QUALIFY=1 BLESS_CREDIT_APP_TIERS=1 cargo build`
//!   once to mint `spec/qualify.spec` (the algebra-qualification census) and
//!   `spec/tiers.spec` (the DERIVED tier partition — reachability, doors, glue; no file
//!   declares a tier) — a missing lock is stale, never fresh, so an unblessed tree refuses
//!   to build. From then on both are drift-gated; regenerate with the same variables and
//!   ratify the diff.

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// The RATIFIED kernel of THIS crate — the only files the partition places in KERNEL.
const KERNEL_ALLOWLIST: &[&str] = &["src/lib.rs"];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = KERNEL_ALLOWLIST.iter().map(|s| s.to_string()).collect();
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.bless_env = "BLESS_CREDIT_APP_QUALIFY".to_string();
    config.tiers_spec = Some(manifest.join("spec/tiers.spec"));
    config.tiers_bless_env = "BLESS_CREDIT_APP_TIERS".to_string();
    Enforcement::enforce_or_panic(&config);
}
