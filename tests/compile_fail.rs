//! Compile-fail suite for the lifecycle protocol — the NEGATIVE specification.
//!
//! The runtime tests in `lifecycle::tests` show the legal path works. These
//! fixtures pin the other half: the illegal sequences must NOT compile. They are
//! the typestate analog of a perturbation probe — each fixture perturbs the
//! protocol along the CONTROL axis (wrong order, skipped step, mismatched proof)
//! and asserts the type system rejects it. A green run means every illegal
//! transition is still a compile error, not a runtime slip.
//!
//! Fixtures live in `tests/compile_fail/` (a subdirectory, so cargo does not try
//! to build them as ordinary integration tests); trybuild compiles each one and
//! checks it fails with the saved `.stderr`.

#[test]
fn lifecycle_illegal_sequences_do_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
