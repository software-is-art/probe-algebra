//! build.rs — the CONSUMER's enforcement shim: attach the whole structural discipline to this
//! crate from `boundary-enforce`, with a config that is OURS, not the library's.
//!
//! This is the piece a consumer copies first. Two decisions live here and must live here:
//!
//! * **The kernel allowlist.** Claiming `Tier: KERNEL` exempts a file from every structural
//!   rule, so it cannot be self-serve — the file must ALSO be named here, in the consumer's own
//!   build script, where admitting a member is a reviewed diff in the consumer's tree. This
//!   fixture's whole kernel is `src/lib.rs` (the module roster and the re-export shim the
//!   `#[algebra]` macro needs — see its header for why).
//!
//! * **The qualification census, opted IN.** `Config::qualify_spec = None` would skip the
//!   freeze/drift of the census entirely; we point it at our own `spec/qualify.spec` instead, so
//!   "which of this crate's modules are algebras by structure" is a ratified, diffable fact
//!   here too — the census is cheap, and its drift gate is exactly the kind of review surface
//!   the discipline is for. We do rename the bless variable (`BLESS_FIXTURE_QUALIFY`): the
//!   default `BLESS_QUALIFY` is also the parent repo's, and a workspace-wide
//!   `BLESS_QUALIFY=1 cargo build` must not silently re-bless two crates' censuses at once.
//!
//! Regenerate the census with `BLESS_FIXTURE_QUALIFY=1 cargo build -p downstream-fixture` and
//! ratify the diff. Everything else — what each tier enforces, the known limitations — is
//! documented on `boundary_enforce` itself.

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// The RATIFIED kernel of THIS crate — the only files allowed to declare `Tier: KERNEL`.
const KERNEL_ALLOWLIST: &[&str] = &["src/lib.rs"];

fn main() {
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut config = Config::new(&manifest);
    config.kernel_allowlist = KERNEL_ALLOWLIST.iter().map(|s| s.to_string()).collect();
    config.qualify_spec = Some(manifest.join("spec/qualify.spec"));
    config.bless_env = "BLESS_FIXTURE_QUALIFY".to_string();
    Enforcement::enforce_or_panic(&config);
}
