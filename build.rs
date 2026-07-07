//! build.rs — enforces the boundary discipline at COMPILE time, over an EXPLICIT, TOTAL partition.
//!
//! The passes themselves — the `//! Tier:` partition (KERNEL/BOUNDARY/INTERIOR/ALGEBRA), the
//! strict tier-1 boundary grammar, the tier-2 inward rule, ALGEBRA capability honesty, the
//! no-rats-nest rule, edge-probe completeness, and the drift-gated qualify census — now live in
//! the workspace crate `boundary-enforce` (see its crate doc for what each tier enforces, the
//! consumer wiring, and the known limitations). This file is a thin shim: it exists so the ONE
//! thing that must stay in the consumer's own tree does — the KERNEL allowlist. Claiming
//! kernel-hood is claiming exemption from every structural rule, so it cannot be self-serve:
//! admitting a member must be a diff HERE, in this repo's build script, where review cannot miss
//! it. A file declaring `Tier: KERNEL` that is not on this list is a build error.
//!
//! The qualify census is frozen into `spec/qualify.spec` (`BLESS_QUALIFY=1 cargo build`);
//! the tier census — the declared partition held against derived evidence, ladder step one
//! of tiers-as-a-lock — into `spec/tiers.spec` (`BLESS_TIERS=1 cargo build`).

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// The RATIFIED kernel — the only files allowed to declare `Tier: KERNEL` (manifest-relative
/// paths). Kept here, not in the `boundary-enforce` crate, so the exemption is a reviewed diff
/// in THIS tree.
const KERNEL_ALLOWLIST: &[&str] = &[
    "src/boundary.rs",
    "src/capability.rs",
    "src/discover/engine.rs",
    "src/discover/expect.rs",
    "src/discover/mod.rs",
    "src/gdp.rs",
    "src/harness.rs",
    "src/lib.rs",
    "src/main.rs",
    "src/tests.rs",
];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = KERNEL_ALLOWLIST.iter().map(|s| s.to_string()).collect();
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.tiers_spec = Some(manifest.join("spec/tiers.spec"));
    Enforcement::enforce_or_panic(&config);
}
