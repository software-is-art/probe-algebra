//! The REFUSAL CENSUS: fire-drill pointed at genesis's validation gates.
//!
//! The first full mutation sweep found its largest survivor cluster in `genesis::validate` —
//! guard-weakening mutants (`||`→`&&`, `==`→`!=`) invisible because no known-bad declaration
//! exercised each refusal arm. This battery is the by-design kill: one bad declaration per
//! refusal, each of which genesis must refuse WITH ITS OWN MESSAGE — a weakened guard stops
//! refusing its fixture and the drill names it VACUOUS. Positive twins pin the over-eager
//! direction (a good declaration must still plan). Grow this file the census way: a new
//! refusal arm in `validate` is a new drill here.

use boundary_spec::discover::genesis::{Deps, Genesis};
use fire_drill::{Battery, Outcome};

/// A declaration must be REFUSED with a message containing `fragment`.
fn refused(declaration: &str, fragment: &str) -> Outcome {
    match Genesis::plan(declaration, &Deps::Version("0".into())) {
        Err(msg) if msg.contains(fragment) => Outcome::Fired,
        _ => Outcome::Passed,
    }
}

/// A declaration must PLAN — the over-eager twin (a guard tightened by mutation refuses it).
fn planned(declaration: &str) -> Outcome {
    match Genesis::plan(declaration, &Deps::Version("0".into())) {
        Ok(_) => Outcome::Fired,
        Err(_) => Outcome::Passed,
    }
}

/// A minimal valid declaration with holes to perturb.
fn good(name: &str, values: &str, modules: &str) -> String {
    format!("system! {{ name: \"{name}\", values {{ {values} }} modules {{ {modules} }} }}")
}

const CREDITS: &str = r#"Credits = i64 where "0..=20";"#;
const METER: &str = "meter { ops { grant(Credits, Credits) -> Credits; } }";

#[test]
fn every_refusal_arm_still_fires() {
    let battery = Battery::named("genesis validation")
        .requires(["crate name", "reserved module", "arity", "seam touch", "range rules"])
        // -- crate name: empty / not letter-first / bad character (the three ||-joined arms)
        .drill("crate name", "an empty crate name",
            refused(&good("", CREDITS, METER), "not a usable crate name"))
        .drill("crate name", "a digit-first crate name",
            refused(&good("1credit", CREDITS, METER), "not a usable crate name"))
        .drill("crate name", "a crate name with an illegal character",
            refused(&good("credit!app", CREDITS, METER), "not a usable crate name"))
        // -- reserved module names (the ||-joined reserved check)
        .drill("reserved module", "a module named `ops`",
            refused(&good("drill-app", CREDITS, "ops { ops { grant(Credits, Credits) -> Credits; } }"),
                "collides with a generated file"))
        .drill("reserved module", "a module named `lib`",
            refused(&good("drill-app", CREDITS, "lib { ops { grant(Credits, Credits) -> Credits; } }"),
                "collides with a generated file"))
        .drill("reserved module", "a module named `meter_internal`",
            refused(&good("drill-app", CREDITS, "meter_internal { ops { grant(Credits, Credits) -> Credits; } }"),
                "collides with a generated file"))
        // -- operator arity: 8 inputs refused, 6 accepted (kills > vs == and > vs >=)
        .drill("arity", "an operator with eight inputs",
            refused(&good("drill-app", CREDITS,
                "meter { ops { grant(Credits, Credits, Credits, Credits, Credits, Credits, Credits, Credits) -> Credits; } }"),
                "more than 6 inputs"))
        .drill("arity", "an operator with exactly six inputs (must PLAN — the over-eager twin)",
            planned(&good("drill-app", CREDITS,
                "meter { ops { grant(Credits, Credits, Credits, Credits, Credits, Credits) -> Credits; } }")))
        // -- transport seam on a value neither side touches (kills the touches ==→!= pair)
        .drill("seam touch", "a seam on a value neither side's operators touch",
            refused(&format!(
                "system! {{ name: \"drill-app\", values {{ {CREDITS} Ghost = i64 where \"any\"; }} \
                 modules {{ {METER} pay {{ ops {{ spend(Credits, Credits) -> Credits; }} }} \
                 ledger {{ ops {{ file(Ghost, Ghost) -> Ghost; }} }} }} \
                 seams {{ meter -- pay : transport on Ghost; }} }}"),
                "no operator of"))
        // -- range rules: unsigned-negative and empty-range refused; degenerate accepted
        .drill("range rules", "an unsigned value ranging from a negative bound",
            refused(&good("drill-app", "Credits = u8 where -3..=20 saturating;", METER),
                "is unsigned"))
        .drill("range rules", "an empty range (lo > hi)",
            refused(&good("drill-app", "Credits = i64 where 9..=3 saturating;", METER),
                "empty range"))
        .drill("range rules", "a range rule on a non-integer raw type",
            refused(&good("drill-app", "Credits = String where 0..=20 saturating;", METER),
                "is not a "))
        .drill("range rules", "a degenerate one-point range (must PLAN — the over-eager twin)",
            planned(&good("drill-app", "Credits = i64 where 5..=5 saturating;", METER)));

    if let Err(rot) = battery.verdict() {
        panic!(
            "a refusal arm went vacuous:\n{rot}\n\nregister:\n{}",
            battery.render()
        );
    }

    // the census half, honest about its current reach: `validate` carries 25 refusal arms
    // (`return err(` sites); this battery covers the arms the full sweep proved vacuous,
    // and the count below RATCHETS — extending validate without extending the battery is a
    // conscious edit here, never silence.
    let source = include_str!("../src/discover/genesis.rs");
    assert_eq!(
        source.matches("return err(").count(),
        25,
        "genesis::validate grew a refusal arm — add its known-bad drill above"
    );
}

/// The vocabulary refusal lists the whole declarable vocabulary (kills the irreflexive
/// filter's `!=`→`==`, which would collapse the listing to nothing useful).
#[test]
fn an_unknown_shape_refusal_teaches_the_vocabulary() {
    let err = Genesis::plan(
        &good(
            "drill-app",
            CREDITS,
            "meter { ops { grant(Credits, Credits) -> Credits; } expects { transitive(grant); } }",
        ),
        &Deps::Version("0".into()),
    );
    let err = match err {
        Err(msg) => msg,
        Ok(_) => panic!("an unratified shape must be refused"),
    };
    assert!(
        err.contains("commutative/1"),
        "the refusal must list the vocabulary: {err}"
    );
    assert!(
        !err.contains("irreflexive"),
        "irreflexive is not declarable: {err}"
    );
}

/// A stray non-`system!` macro before the declaration is skipped, not parsed as it (kills
/// the parse_declaration match-guard `→ true` mutant, which would grab the first macro).
#[test]
fn a_stray_macro_does_not_impersonate_the_declaration() {
    let decl = format!(
        "thread_local! {{ static X: u8 = 0; }}\n{}",
        good("drill-app", CREDITS, METER)
    );
    assert!(Genesis::plan(&decl, &Deps::Version("0".into())).is_ok());
}
