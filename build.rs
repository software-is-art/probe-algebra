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

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    // The RATIFIED kernel, read from the register: every exemption carries a
    // justification, and the Register grammar refuses bare keys, empty justifications,
    // and duplicates — so kernel-hood stays a reviewed decision, now with its reasons
    // committed next to it.
    let register = spec_lock::Register {
        name: "kernel".to_string(),
        path: manifest.join("spec/kernel.register"),
    };
    let kernel: Vec<String> = register
        .entries()
        .unwrap_or_else(|refusal| panic!("kernel register refused: {refusal}"))
        .into_iter()
        .map(|(path, _justification)| path)
        .collect();
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = kernel;
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.tiers_spec = Some(manifest.join("spec/tiers.spec"));
    Enforcement::enforce_or_panic(&config);
}
