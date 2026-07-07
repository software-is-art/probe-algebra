//! The rule inventory, as executable spec: one tiny source-tree fixture per rule, asserting the
//! rule FIRES on the offending shape and stays silent on the clean one. A consumer can read this
//! file top-to-bottom as the list of what `boundary-enforce` will reject.

use std::path::PathBuf;

use boundary_enforce::{Config, Enforcement};

/// Materialize a fixture tree under `std::env::temp_dir()` and return a `Config` rooted at it.
/// Each test names its own directory, so tests are independent and re-runnable.
fn fixture(name: &str, files: &[(&str, &str)]) -> Config {
    let root = std::env::temp_dir().join(format!(
        "boundary-enforce-fixture-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    for (rel, contents) in files {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    Config::new(root)
}

fn violations(name: &str, files: &[(&str, &str)]) -> Vec<String> {
    Enforcement::run(&fixture(name, files)).violations
}

/// Assert some violation contains `needle` (violation strings are long; we match the load-bearing
/// fragment).
fn assert_fires(violations: &[String], needle: &str) {
    assert!(
        violations.iter().any(|v| v.contains(needle)),
        "expected a violation containing {needle:?}, got: {violations:#?}"
    );
}

// ===== the total partition: every file names its tier ====================

#[test]
fn missing_tier_is_a_build_error() {
    let vs = violations("missing-tier", &[("src/lib.rs", "pub struct Foo;\n")]);
    assert_fires(&vs, "no `Tier:` declaration");
}

#[test]
fn tier_in_a_plain_comment_does_not_count() {
    // only `//!` module-doc lines can declare (or spoof) a tier — a `//` code comment cannot.
    let vs = violations(
        "plain-comment-tier",
        &[(
            "src/lib.rs",
            "// Tier: INTERIOR — not a module-doc line\npub struct Foo;\n",
        )],
    );
    assert_fires(&vs, "no `Tier:` declaration");
}

// ===== KERNEL: exemption must be ratified in the consumer's build.rs =====

#[test]
fn unratified_kernel_is_a_violation() {
    let vs = violations(
        "unratified-kernel",
        &[(
            "src/lib.rs",
            "//! Tier: KERNEL — self-asserted, not allowlisted.\n",
        )],
    );
    assert_fires(
        &vs,
        "declares `Tier: KERNEL` but is not in KERNEL_ALLOWLIST",
    );
}

#[test]
fn allowlisted_kernel_is_exempt_from_every_rule() {
    let mut config = fixture(
        "ratified-kernel",
        &[(
            "src/lib.rs",
            // would violate BOUNDARY/INTERIOR rules — but a ratified kernel is exempt.
            "//! Tier: KERNEL — the trusted floor.\npub fn raw() -> String { std::fs::read_to_string(\"x\").unwrap() }\n",
        )],
    );
    config.kernel_allowlist = vec!["src/lib.rs".to_string()];
    let e = Enforcement::run(&config);
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
}

// ===== BOUNDARY: the strict tier-1 grammar ================================

#[test]
fn boundary_bans_free_fns_statics_submodules_and_traits() {
    let vs = violations(
        "boundary-items",
        &[(
            "src/domain/boundary.rs",
            "//! Tier: BOUNDARY — the value-object surface.\n\
             pub struct Ok1(String);\n\
             pub fn loose() -> Ok1 { Ok1(String::new()) }\n\
             static COUNT: i64 = 0;\n\
             mod inner {}\n\
             pub trait Sneaky {}\n",
        )],
    );
    assert_fires(&vs, "free function `loose`");
    assert_fires(&vs, "`static COUNT` — a boundary may not hold global state");
    assert_fires(&vs, "submodule `inner` — a boundary is flat");
    assert_fires(&vs, "trait `Sneaky` — traits belong in the grammar");
}

#[test]
fn boundary_field_purity_public_and_raw_primitive_fields() {
    let vs = violations(
        "boundary-fields",
        &[(
            "src/domain/boundary.rs",
            "//! Tier: BOUNDARY — the value-object surface.\n\
             pub struct Leaky { pub name: Ident }\n\
             pub struct Downgraded { table: std::collections::BTreeMap<String, Ident> }\n\
             pub struct Ident(String); // the sanctioned newtype wrapper\n",
        )],
    );
    assert_fires(&vs, "`Leaky` has a public field");
    assert_fires(&vs, "`Downgraded` has a field containing raw `String`");
    // the lone-field newtype is the one sanctioned home for a primitive:
    assert!(
        !vs.iter().any(|v| v.contains("`Ident`")),
        "newtype wrapper wrongly flagged: {vs:#?}"
    );
}

#[test]
fn boundary_effect_via_use_import_is_not_an_evasion() {
    // `use std::fs;` then `fs::write(…)` — no two-segment `std::fs` window in any path, so a
    // naive path scan misses it; the import map must close the loophole.
    let vs = violations(
        "boundary-use-evasion",
        &[(
            "src/domain/boundary.rs",
            "//! Tier: BOUNDARY — the value-object surface.\n\
             use std::fs;\n\
             pub struct Saver;\n\
             impl Saver { pub fn save(&self) { fs::write(\"out\", \"data\").unwrap(); } }\n",
        )],
    );
    assert_fires(
        &vs,
        "`std::fs (reached via `use`)` — a boundary performs no I/O",
    );
}

#[test]
fn boundary_effect_smuggled_in_macro_tokens_is_caught() {
    // a macro's tokens are unparsed — the path visitor never sees them; the token scan must.
    let vs = violations(
        "boundary-macro-tokens",
        &[(
            "src/domain/boundary.rs",
            "//! Tier: BOUNDARY — the value-object surface.\n\
             pub struct Sneaky;\n\
             impl Sneaky { pub fn go(&self) { some_macro!(std::fs::write(\"out\", \"data\")); } }\n",
        )],
    );
    assert_fires(&vs, "`some_macro!` carries `std::fs` in its tokens");
}

#[test]
fn boundary_bans_unsafe_in_all_three_keyword_positions() {
    let vs = violations(
        "boundary-unsafe",
        &[(
            "src/domain/boundary.rs",
            "//! Tier: BOUNDARY — the value-object surface.\n\
             pub struct Foo;\n\
             impl Foo {\n\
                 pub unsafe fn dance(&self) {}\n\
                 pub fn block(&self) { unsafe { std::ptr::null::<Foo>(); } }\n\
             }\n\
             unsafe impl Send for Foo {}\n",
        )],
    );
    assert_fires(&vs, "`unsafe fn dance` — boundaries are safe code");
    assert_fires(&vs, "`unsafe` block — boundaries are safe code");
    assert_fires(&vs, "`unsafe impl` — boundaries are safe code");
}

#[test]
fn boundary_bans_printing_macros_and_reexports() {
    let vs = violations(
        "boundary-print-reexport",
        &[(
            "src/domain/boundary.rs",
            "//! Tier: BOUNDARY — the value-object surface.\n\
             pub use crate::other::Thing;\n\
             pub struct Foo;\n\
             impl Foo { pub fn shout(&self) { println!(\"hi\"); } }\n",
        )],
    );
    assert_fires(&vs, "re-export (`pub use`)");
    assert_fires(&vs, "`println!` — a boundary performs no I/O");
}

// ===== INTERIOR: the inward rule + no rats-nest ===========================

#[test]
fn interior_inward_rule_no_raw_primitive_returns() {
    let vs = violations(
        "interior-inward",
        &[(
            "src/domain/internal.rs",
            "//! Tier: INTERIOR — the workshop.\n\
             pub(crate) fn render() -> String { String::new() }\n",
        )],
    );
    assert_fires(&vs, "`render` returns a raw `String`");
}

#[test]
fn interior_loose_pub_fn_is_a_rats_nest() {
    // `bool` return disqualifies the operator shape, and full `pub` makes it loose plumbing.
    let vs = violations(
        "interior-loose-pub",
        &[(
            "src/domain/internal.rs",
            "//! Tier: INTERIOR — the workshop.\n\
             pub struct Flag;\n\
             pub fn is_set(f: Flag) -> bool { let _ = f; true }\n",
        )],
    );
    assert_fires(&vs, "`pub fn is_set` is a LOOSE public function");
}

#[test]
fn operator_shaped_pub_fn_is_attached_not_loose() {
    let e = Enforcement::run(&fixture(
        "interior-operator-ok",
        &[(
            "src/domain/internal.rs",
            "//! Tier: INTERIOR — the workshop.\n\
             pub struct Tri;\n\
             pub fn join(a: Tri, b: Tri) -> Tri { let _ = b; a }\n",
        )],
    ));
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
    // and the same shape is what the census counts:
    assert!(
        e.qualify_census
            .contains("QUALIFIES — operators [join] over sorts {Tri}"),
        "census was: {}",
        e.qualify_census
    );
}

// ===== ALGEBRA: capability honesty ========================================

#[test]
fn algebra_undeclared_world_touch_is_a_violation() {
    let vs = violations(
        "algebra-undeclared-effect",
        &[(
            "src/report.rs",
            "//! Tier: ALGEBRA — a report layer.\n\
             fn slurp() -> Source { Source::from(std::fs::read_to_string(\"x\").unwrap()) }\n",
        )],
    );
    assert_fires(
        &vs,
        "`slurp` touches the world (`std::fs`) but declares no capability",
    );
}

#[test]
fn algebra_malformed_capability_line_is_decoration_not_contract() {
    let vs = violations(
        "algebra-malformed-capability",
        &[(
            "src/report.rs",
            "//! Tier: ALGEBRA — a report layer.\n\
             /// Capability: whatever\n\
             fn slurp() -> Source { Source::from(std::fs::read_to_string(\"x\").unwrap()) }\n",
        )],
    );
    assert_fires(
        &vs,
        "a `Capability:` line that names no capability (`whatever`)",
    );
}

#[test]
fn algebra_declared_capability_makes_the_effect_an_honest_edge() {
    let e = Enforcement::run(&fixture(
        "algebra-declared-capability",
        &[(
            "src/report.rs",
            "//! Tier: ALGEBRA — a report layer.\n\
             /// Capability: Effectful — reads the source tree.\n\
             fn slurp() -> Source { Source::from(std::fs::read_to_string(\"x\").unwrap()) }\n",
        )],
    ));
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
}

// ===== edge-probe completeness ============================================

#[test]
fn concrete_edge_without_probe_is_a_violation() {
    let vs = violations(
        "edge-no-probe",
        &[(
            "src/domain/internal.rs",
            "//! Tier: INTERIOR — the workshop.\n\
             pub struct Halve;\n\
             impl Morphism for Halve {}\n",
        )],
    );
    assert_fires(&vs, "boundary edge `Halve` has no `impl Probed`");
}

#[test]
fn probed_edge_and_generic_combinator_pass() {
    let e = Enforcement::run(&fixture(
        "edge-probed",
        &[(
            "src/domain/internal.rs",
            "//! Tier: INTERIOR — the workshop.\n\
             pub struct Halve;\n\
             impl Morphism for Halve {}\n\
             impl Probed for Halve {}\n\
             pub struct Compose<F, G>(F, G);\n\
             impl<F, G> Morphism for Compose<F, G> {}\n\
             #[cfg(test)]\n\
             mod tests { pub struct Fixture; impl Morphism for Fixture {} }\n",
        )],
    ));
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
}

// ===== the qualify census: frozen and drift-gated =========================

#[test]
fn census_drifts_then_blesses_then_holds() {
    let spec_rel = "spec/qualify.spec";
    let algebra_file = (
        "src/domain/internal.rs",
        "//! Tier: INTERIOR — the workshop.\n\
         pub struct Tri;\n\
         pub fn meet(a: Tri, b: Tri) -> Tri { let _ = b; a }\n",
    );
    let mut config = fixture("census-drift", &[algebra_file]);
    std::fs::create_dir_all(config.manifest_dir.join("spec")).unwrap();
    config.qualify_spec = Some(config.manifest_dir.join(spec_rel));
    // a per-test bless variable, so parallel tests never race on the real BLESS_QUALIFY:
    config.bless_env = "BLESS_QUALIFY_ENFORCE_TEST_CENSUS".to_string();

    // 1. no committed spec yet — the census has drifted (from empty), and the message names the
    //    manifest-relative spec path and the configured bless variable:
    let drifted = Enforcement::run(&config);
    assert_fires(
        &drifted.violations,
        "spec/qualify.spec is stale — the algebra-qualification census drifted",
    );
    assert_fires(
        &drifted.violations,
        "`BLESS_QUALIFY_ENFORCE_TEST_CENSUS=1 cargo build`",
    );

    // 2. bless: the census is written where the spec lives, and the run is clean:
    std::env::set_var(&config.bless_env, "1");
    let blessed = Enforcement::run(&config);
    std::env::remove_var(&config.bless_env);
    assert!(
        blessed.violations.is_empty(),
        "got: {:#?}",
        blessed.violations
    );
    let committed = std::fs::read_to_string(config.manifest_dir.join(spec_rel)).unwrap();
    assert_eq!(committed, blessed.qualify_census);
    assert!(committed.contains("QUALIFIES — operators [meet] over sorts {Tri}"));

    // 3. and with the spec committed, the gate holds without the env var:
    let held = Enforcement::run(&config);
    assert!(held.violations.is_empty(), "got: {:#?}", held.violations);
}

// ===== the clean fixture, end to end ======================================

#[test]
fn a_disciplined_tree_passes_all_passes() {
    let mut config = fixture(
        "clean-tree",
        &[
            (
                "src/lib.rs",
                "//! Tier: KERNEL — the crate root.\npub mod domain;\n",
            ),
            (
                "src/domain/boundary.rs",
                "//! Tier: BOUNDARY — the value-object surface.\n\
                 pub struct Ident(String);\n\
                 pub struct Halve;\n\
                 impl Morphism for Halve {}\n\
                 impl Probed for Halve {}\n",
            ),
            (
                "src/domain/internal.rs",
                "//! Tier: INTERIOR — the workshop.\n\
                 pub struct Ident2;\n\
                 pub(crate) fn intern(raw: Ident2) -> Ident2 { raw }\n",
            ),
        ],
    );
    config.kernel_allowlist = vec!["src/lib.rs".to_string()];
    let e = Enforcement::run(&config);
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
    // the rerun set covers the src root (so new files re-trigger) and every walked entry:
    assert_eq!(e.rerun_paths.first(), Some(&config.src_root));
    assert!(e
        .rerun_paths
        .iter()
        .any(|p| p.ends_with(PathBuf::from("src/domain"))));
    assert!(e
        .rerun_paths
        .iter()
        .any(|p| p.ends_with(PathBuf::from("src/domain/boundary.rs"))));
}

// ===== the tier census: the declared partition vs derived evidence ========

#[test]
fn the_tier_census_derives_and_records_coherence() {
    let config = fixture(
        "tier-census",
        &[
            (
                "src/lib.rs",
                "//! Tier: KERNEL — floor\npub mod api;\npub mod report;\npub mod rogue;\nmod internal;\n",
            ),
            (
                "src/api.rs",
                "//! Tier: BOUNDARY — surface\npub struct Credit;\npub fn grant(a: Credit) -> Credit { a }\n",
            ),
            (
                "src/report.rs",
                "//! Tier: ALGEBRA — report\npub fn render() -> i64 { 1 }\n",
            ),
            (
                "src/internal.rs",
                "//! Tier: INTERIOR — workshop\nfn helper() {}\n",
            ),
            (
                "src/rogue.rs",
                "//! Tier: INTERIOR — misdeclared: pub-reachable cannot be interior\npub fn misc() -> i64 { 2 }\n",
            ),
        ],
    );
    let census = Enforcement::run(&config).tiers_census;
    assert!(
        census.contains("# 5 files: 3 agree, 1 disagree, 1 kernel decisions."),
        "{census}"
    );
    assert!(census.contains(
        "src/api.rs: declared BOUNDARY; derived BOUNDARY (pub-reachable, operator-shaped) — agree"
    ));
    assert!(census.contains(
        "src/internal.rs: declared INTERIOR; derived INTERIOR (not pub-reachable) — agree"
    ));
    assert!(census.contains(
        "src/rogue.rs: declared INTERIOR; derived ALGEBRA (pub-reachable, not operator-shaped) — DISAGREES"
    ));
    assert!(census.contains("src/lib.rs: declared KERNEL — a decision"));
}

#[test]
fn a_stale_tier_census_is_a_violation() {
    let mut config = fixture(
        "tier-census-stale",
        &[("src/lib.rs", "//! Tier: KERNEL — floor\n")],
    );
    let spec = config.manifest_dir.join("spec/tiers.spec");
    std::fs::create_dir_all(spec.parent().unwrap()).unwrap();
    std::fs::write(&spec, "old\n").unwrap();
    config.tiers_spec = Some(spec);
    let vs = Enforcement::run(&config).violations;
    assert_fires(&vs, "the tier census drifted");
}
