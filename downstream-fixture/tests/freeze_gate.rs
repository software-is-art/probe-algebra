//! freeze_gate — the consumer's DRIFT GATE, as a plain integration test.
//!
//! Re-derive the live discovered spec and hold it against the committed lock
//! (`spec/credit-meter.spec`). A mismatch fails CI; the fix is to regenerate
//! (`cargo run -p downstream-fixture --example freeze`) and put the resulting diff through
//! review — never to edit the lock by hand. A missing lock file is stale, never fresh.
//!
//! This file lives in `tests/` deliberately: it consumes only the fixture's public surface
//! (`ops::meter_ops::CreditMeter`) plus the library's public `Spec` and `spec_lock`'s `check`,
//! so it is exactly the test a consumer copies. Per the library's upgrade contract
//! (`docs/ci-discipline.md`), the lock holds only DOMAIN facts — laws and coverage — so an
//! engine-internal library upgrade cannot drift it; if this gate goes red, the LAWS changed,
//! which is precisely what wants ratification.

use std::path::PathBuf;

use boundary_algebra::discover::Spec;
use downstream_fixture::ops::meter_ops::CreditMeter;

/// This crate's own spec directory — the lock must live where OUR CI can diff and ratify it,
/// never inside the library checkout.
fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec")
}

/// The committed spec lock is FRESH: the live discovered algebra still matches what was
/// ratified. This is move 3 of the CI discipline — the drift gate.
#[test]
fn the_committed_spec_is_fresh() {
    let lock = Spec::of::<CreditMeter>().lock_in(&spec_dir());
    if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
        panic!(
            "discovered spec drifted from the committed lock for: {}. \
             Run `cargo run -p downstream-fixture --example freeze` and ratify the diff.",
            stale.join(", ")
        );
    }
}

/// The GOLDEN law list — the discovered algebra, pinned exactly, with the refusals spelled
/// out. The lock file already gates the rendered text; this test additionally documents (and
/// defends, one layer below the rendering) WHAT was discovered and what was refused:
///
///   - `grant` is a commutative semigroup (saturating addition at the cap);
///   - `renew` is a non-commutative band, and its BIAS is a stated law — the sandwich shape
///     names the later operand as the winner;
///   - `spend` satisfies NO universal shape: it appears in `uncovered_ops`, the spec's
///     explicitly-named silence;
///   - there is no `zero` constant in the signature (see `ops`' header for why), so no
///     identity law can appear.
#[test]
fn the_discovered_laws_are_exactly_the_ratified_ones() {
    let spec = Spec::of::<CreditMeter>();
    assert_eq!(spec.theory, "credit meter");
    let got: Vec<(String, String)> = spec
        .laws
        .iter()
        .map(|l| (l.prose().to_string(), l.equation().to_string()))
        .collect();
    let expected: Vec<(&str, &str)> = vec![
        (
            "grant gives the same result in either order.",
            "(x grant y) = (y grant x)",
        ),
        (
            "With grant, the grouping of three values doesn't matter.",
            "((x grant y) grant z) = (x grant (y grant z))",
        ),
        (
            "With renew, the grouping of three values doesn't matter.",
            "((x renew y) renew z) = (x renew (y renew z))",
        ),
        (
            "renew of a value with itself gives that value.",
            "(x renew x) = x",
        ),
        (
            "With renew, the later operand wins where the two disagree — re-applying an \
             earlier one cannot overwrite it.",
            "((x renew y) renew x) = (y renew x)",
        ),
    ];
    let expected: Vec<(String, String)> = expected
        .into_iter()
        .map(|(p, e)| (p.to_string(), e.to_string()))
        .collect();
    assert_eq!(got, expected, "the discovered credit-meter algebra changed");
    // the refusal is load-bearing: `spend` is the named silence, and nothing else is.
    assert_eq!(
        spec.uncovered_ops,
        vec!["spend"],
        "the operators the spec is silent about changed"
    );
}
