//! Compile-fail suite for the fixture's witness discipline — the NEGATIVE specification,
//! run DOWNSTREAM.
//!
//! `tests/probes.rs` shows the legal path works: `CheckFunds` mints an `Affordable<N>`,
//! `Deduct` consumes it. These fixtures pin the other half — the whole point of the
//! Branch/Guarded pair is what a consumer CANNOT write, so each illegal use must fail to
//! compile in this consumer crate, through public API alone:
//!
//! * `deduct_forged_witness`       — the witness cannot be minted by hand (private field);
//! * `deduct_insufficient_witness` — the negative arm's witness discharges nothing;
//! * `deduct_wrong_order`          — a witness for order A cannot deduct order B.
//!
//! Fixtures live in `tests/compile_fail/` (a subdirectory, so cargo does not build them as
//! integration tests); trybuild compiles each and checks it fails with the saved `.stderr`,
//! generated on the CI-pinned toolchain (regenerate with `TRYBUILD=overwrite` in lockstep
//! with a pin bump — see the workspace's `.github/workflows/ci.yml`).

#[test]
fn illegal_witness_uses_do_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
