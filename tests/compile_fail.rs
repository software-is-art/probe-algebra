//! Compile-fail suite for the interpreter boundary — the NEGATIVE specification.
//!
//! The runtime tests in `tests` and `laws` show the legal path works. These
//! fixtures pin the other half: the illegal uses must NOT compile. They are the
//! typestate analog of a perturbation probe — each fixture perturbs a boundary
//! invariant (here: feeding `Eval` a `WellTyped` witness minted for a DIFFERENT
//! program) and asserts the type system rejects it. A green run means the witness
//! genuinely ties `Check` to `Eval` at compile time, not merely at runtime.
//!
//! Fixtures live in `tests/compile_fail/` (a subdirectory, so cargo does not try
//! to build them as ordinary integration tests); trybuild compiles each one and
//! checks it fails with the saved `.stderr`.

#[test]
fn illegal_boundary_uses_do_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
