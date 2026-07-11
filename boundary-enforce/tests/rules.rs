//! The rule inventory, as executable spec: one tiny source-tree fixture per rule, asserting the
//! rule FIRES on the offending shape and stays silent on the clean one. A consumer can read this
//! file top-to-bottom as the list of what `boundary-enforce` will reject.
//!
//! No fixture declares a tier — there is nothing to declare. Each earns its place the way a real
//! tree does: a `src/lib.rs` root and `pub mod` chains make a file reachable, a production edge
//! impl or a fronting reference makes it a BOUNDARY, unreachability makes it INTERIOR, and the
//! reachable remainder is ALGEBRA. KERNEL never appears without a `kernel_allowlist` entry.

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

/// A reachable BOUNDARY fixture: root and glue wire `src/domain/boundary.rs` into the pub tree,
/// and an appended probed production edge is the door evidence that derives the file BOUNDARY —
/// so each test's `body` is judged under the tier-1 grammar.
fn boundary_violations(name: &str, body: &str) -> Vec<String> {
    let contents = format!(
        "{body}pub struct DoorEdge;\nimpl Morphism for DoorEdge {{}}\nimpl Probed for DoorEdge {{}}\n"
    );
    violations(
        name,
        &[
            ("src/lib.rs", "pub mod domain;\n"),
            ("src/domain/mod.rs", "pub mod boundary;\n"),
            ("src/domain/boundary.rs", &contents),
        ],
    )
}

// ===== the partition is total by DERIVATION ================================

#[test]
fn an_unmarked_file_is_judged_by_derivation() {
    // no marker, no registration — the file exists, so it HAS a tier: unreachable from the
    // root, it derives INTERIOR, and the inward rule fires. Nothing can opt out by silence.
    let vs = violations(
        "derived-interior",
        &[
            ("src/lib.rs", "mod helper;\n"),
            (
                "src/helper.rs",
                "pub(crate) fn render() -> String { String::new() }\n",
            ),
        ],
    );
    assert_fires(&vs, "`render` returns a raw `String`");
}

#[test]
fn a_tier_marker_grants_nothing() {
    // the old `//! Tier:` markers are dead syntax: a self-asserted KERNEL is judged like any
    // other file (here: reachable, no edges, fronts nothing — ALGEBRA, where a loose pub fn
    // still fires).
    let vs = violations(
        "marker-is-inert",
        &[
            ("src/lib.rs", "pub mod tool;\n"),
            (
                "src/tool.rs",
                "//! Tier: KERNEL — self-asserted; the marker is dead syntax.\n\
                 pub fn is_set(f: i64) -> bool { f > 0 }\n",
            ),
        ],
    );
    assert_fires(&vs, "`pub fn is_set` is a LOOSE public function");
}

// ===== KERNEL: a register decision, never derived ==========================

#[test]
fn kernel_comes_only_from_the_ratified_register() {
    let mut config = fixture(
        "ratified-kernel",
        &[(
            "src/lib.rs",
            // would violate ALGEBRA rules (loose pub fn, undeclared effect) — but the
            // registered kernel is exempt from every structural rule.
            "pub fn raw() -> String { std::fs::read_to_string(\"x\").unwrap() }\n",
        )],
    );
    config.kernel_allowlist = vec!["src/lib.rs".to_string()];
    let e = Enforcement::run(&config);
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
}

#[test]
fn an_unregistered_file_gets_no_exemption() {
    // the same tree WITHOUT the register entry: the root derives ALGEBRA and both the
    // capability-honesty and no-rats-nest rules fire.
    let vs = violations(
        "unregistered-kernel",
        &[(
            "src/lib.rs",
            "pub fn raw() -> String { std::fs::read_to_string(\"x\").unwrap() }\n",
        )],
    );
    assert_fires(
        &vs,
        "touches the world (`std::fs`) but declares no capability",
    );
    assert_fires(&vs, "`pub fn raw` is a LOOSE public function");
}

#[test]
fn a_stale_kernel_registration_is_a_violation() {
    // a register line for a file that no longer exists is a stale ratification — refused,
    // never silently ignored.
    let mut config = fixture("stale-register", &[("src/lib.rs", "pub struct Foo;\n")]);
    config.kernel_allowlist = vec!["src/ghost.rs".to_string()];
    let vs = Enforcement::run(&config).violations;
    assert_fires(&vs, "registered as KERNEL but no such file exists");
}

// ===== BOUNDARY: the strict tier-1 grammar ================================

#[test]
fn boundary_bans_free_fns_statics_submodules_and_traits() {
    let vs = boundary_violations(
        "boundary-items",
        "pub struct Ok1(String);\n\
         pub fn loose() -> Ok1 { Ok1(String::new()) }\n\
         static COUNT: i64 = 0;\n\
         mod inner {}\n\
         pub trait Sneaky {}\n",
    );
    assert_fires(&vs, "free function `loose`");
    assert_fires(&vs, "`static COUNT` — a boundary may not hold global state");
    assert_fires(&vs, "submodule `inner` — a boundary is flat");
    assert_fires(&vs, "trait `Sneaky` — traits belong in the grammar");
}

#[test]
fn boundary_field_purity_public_and_raw_primitive_fields() {
    let vs = boundary_violations(
        "boundary-fields",
        "pub struct Leaky { pub name: Ident }\n\
         pub struct Downgraded { table: std::collections::BTreeMap<String, Ident> }\n\
         pub struct Ident(String); // the sanctioned newtype wrapper\n",
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
    let vs = boundary_violations(
        "boundary-use-evasion",
        "use std::fs;\n\
         pub struct Saver;\n\
         impl Saver { pub fn save(&self) { fs::write(\"out\", \"data\").unwrap(); } }\n",
    );
    assert_fires(
        &vs,
        "`std::fs (reached via `use`)` — a boundary performs no I/O",
    );
}

#[test]
fn boundary_effect_smuggled_in_macro_tokens_is_caught() {
    // a macro's tokens are unparsed — the path visitor never sees them; the token scan must.
    let vs = boundary_violations(
        "boundary-macro-tokens",
        "pub struct Sneaky;\n\
         impl Sneaky { pub fn go(&self) { some_macro!(std::fs::write(\"out\", \"data\")); } }\n",
    );
    assert_fires(&vs, "`some_macro!` carries `std::fs` in its tokens");
}

#[test]
fn boundary_bans_unsafe_in_all_three_keyword_positions() {
    let vs = boundary_violations(
        "boundary-unsafe",
        "pub struct Foo;\n\
         impl Foo {\n\
             pub unsafe fn dance(&self) {}\n\
             pub fn block(&self) { unsafe { std::ptr::null::<Foo>(); } }\n\
         }\n\
         unsafe impl Send for Foo {}\n",
    );
    assert_fires(&vs, "`unsafe fn dance` — boundaries are safe code");
    assert_fires(&vs, "`unsafe` block — boundaries are safe code");
    assert_fires(&vs, "`unsafe impl` — boundaries are safe code");
}

#[test]
fn boundary_bans_printing_macros_and_reexports() {
    let vs = boundary_violations(
        "boundary-print-reexport",
        "pub use crate::other::Thing;\n\
         pub struct Foo;\n\
         impl Foo { pub fn shout(&self) { println!(\"hi\"); } }\n",
    );
    assert_fires(&vs, "re-export (`pub use`)");
    assert_fires(&vs, "`println!` — a boundary performs no I/O");
}

#[test]
fn a_fronting_file_is_held_to_the_boundary_grammar() {
    // no edge impls anywhere: the file derives BOUNDARY purely by FRONTING an interior
    // sibling (the tier-2 relation read backwards) — and the tier-1 grammar then applies.
    let vs = violations(
        "fronting-door",
        &[
            ("src/lib.rs", "pub mod door;\nmod inner_work;\n"),
            ("src/inner_work.rs", "pub(crate) fn helper() {}\n"),
            (
                "src/door.rs",
                "pub struct Door;\n\
                 impl Door { pub fn go(&self) { crate::inner_work::helper(); println!(\"hi\"); } }\n",
            ),
        ],
    );
    assert_fires(&vs, "`println!` — a boundary performs no I/O");
}

// ===== INTERIOR: the inward rule + no rats-nest ===========================

#[test]
fn interior_inward_rule_no_raw_primitive_returns() {
    let vs = violations(
        "interior-inward",
        &[
            ("src/lib.rs", "mod internal;\n"),
            (
                "src/internal.rs",
                "pub(crate) fn render() -> String { String::new() }\n",
            ),
        ],
    );
    assert_fires(&vs, "`render` returns a raw `String`");
}

#[test]
fn interior_loose_pub_fn_is_a_rats_nest() {
    // `bool` return disqualifies the operator shape, and full `pub` makes it loose plumbing.
    let vs = violations(
        "interior-loose-pub",
        &[
            ("src/lib.rs", "mod internal;\n"),
            (
                "src/internal.rs",
                "pub struct Flag;\n\
                 pub fn is_set(f: Flag) -> bool { let _ = f; true }\n",
            ),
        ],
    );
    assert_fires(&vs, "`pub fn is_set` is a LOOSE public function");
}

#[test]
fn operator_shaped_pub_fn_is_attached_not_loose() {
    let e = Enforcement::run(&fixture(
        "interior-operator-ok",
        &[
            ("src/lib.rs", "mod internal;\n"),
            (
                "src/internal.rs",
                "pub struct Tri;\n\
                 pub fn join(a: Tri, b: Tri) -> Tri { let _ = b; a }\n",
            ),
        ],
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
        &[
            ("src/lib.rs", "pub mod report;\n"),
            (
                "src/report.rs",
                "fn slurp() -> Source { Source::from(std::fs::read_to_string(\"x\").unwrap()) }\n",
            ),
        ],
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
        &[
            ("src/lib.rs", "pub mod report;\n"),
            (
                "src/report.rs",
                "/// Capability: whatever\n\
                 fn slurp() -> Source { Source::from(std::fs::read_to_string(\"x\").unwrap()) }\n",
            ),
        ],
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
        &[
            ("src/lib.rs", "pub mod report;\n"),
            (
                "src/report.rs",
                "/// Capability: Effectful — reads the source tree.\n\
                 fn slurp() -> Source { Source::from(std::fs::read_to_string(\"x\").unwrap()) }\n",
            ),
        ],
    ));
    assert!(e.violations.is_empty(), "got: {:#?}", e.violations);
}

// ===== edge-probe completeness ============================================

#[test]
fn concrete_edge_without_probe_is_a_violation() {
    // probe completeness is checked over EVERY file, whatever its tier — here an
    // unreachable (INTERIOR-derived) one.
    let vs = violations(
        "edge-no-probe",
        &[(
            "src/domain/internal.rs",
            "pub struct Halve;\n\
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
            "pub struct Halve;\n\
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
        "pub struct Tri;\n\
         pub fn meet(a: Tri, b: Tri) -> Tri { let _ = b; a }\n",
    );
    let mut config = fixture("census-drift", &[algebra_file]);
    std::fs::create_dir_all(config.manifest_dir.join("spec")).unwrap();
    config.qualify_spec = Some(config.manifest_dir.join(spec_rel));
    // a per-test bless variable, so parallel tests never race on the real BLESS_QUALIFY:
    config.bless_env = "BLESS_QUALIFY_ENFORCE_TEST_CENSUS".to_string();
    // the CI arm first: no autofix — the refusal reports and touches nothing.
    config.autofix_stale = false;

    // 1. no committed spec yet — the census has drifted (from empty), the message names the
    //    manifest-relative spec path and the configured bless variable, and the tree is NOT
    //    mutated (a fresh checkout must never mutate itself):
    let drifted = Enforcement::run(&config);
    assert_fires(
        &drifted.violations,
        "spec/qualify.spec is stale — the algebra-qualification census drifted",
    );
    assert_fires(
        &drifted.violations,
        "`BLESS_QUALIFY_ENFORCE_TEST_CENSUS=1 cargo build`",
    );
    assert!(
        !config.manifest_dir.join(spec_rel).exists(),
        "the CI arm never writes"
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

    // 4. THE BLESS LOOP, DISSOLVED (the local arm): drift again — grow the module — with
    //    autofix on. The refusal RUNS its fixing derivation: the regenerated census lands
    //    in the tree, the run still fails once (the movement is never silent), and the
    //    message says the diff is the ratification instead of assigning homework:
    std::fs::write(
        config.manifest_dir.join("src/domain/internal.rs"),
        "pub struct Tri;\n\
         pub fn meet(a: Tri, b: Tri) -> Tri { let _ = b; a }\n\
         pub fn join(a: Tri, b: Tri) -> Tri { let _ = a; b }\n",
    )
    .unwrap();
    config.autofix_stale = true;
    let fixed = Enforcement::run(&config);
    assert_fires(
        &fixed.violations,
        "spec/qualify.spec was stale — the algebra-qualification census drifted, and the \
         regenerated text is now IN YOUR WORKING TREE",
    );
    assert_fires(&fixed.violations, "The diff is the ratification");
    let regenerated = std::fs::read_to_string(config.manifest_dir.join(spec_rel)).unwrap();
    assert!(
        regenerated.contains("operators [join, meet] over sorts {Tri}"),
        "the fixing derivation ran: {regenerated}"
    );

    // 5. the very next run holds clean — the failure fired exactly once, and what remains
    //    is the diff awaiting its signature:
    let settled = Enforcement::run(&config);
    assert!(
        settled.violations.is_empty(),
        "got: {:#?}",
        settled.violations
    );
}

// ===== the clean fixture, end to end ======================================

#[test]
fn a_disciplined_tree_passes_all_passes() {
    let mut config = fixture(
        "clean-tree",
        &[
            ("src/lib.rs", "pub mod domain;\n"),
            ("src/domain/mod.rs", "pub mod boundary;\nmod internal;\n"),
            (
                "src/domain/boundary.rs",
                "pub struct Ident(String);\n\
                 pub struct Halve;\n\
                 impl Morphism for Halve {}\n\
                 impl Probed for Halve {}\n",
            ),
            (
                "src/domain/internal.rs",
                "pub struct Ident2;\n\
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

// ===== the tier lock: the derived partition, frozen ========================

#[test]
fn the_tier_partition_derives_every_file() {
    // one file per kind of evidence: a registered kernel, glue, an edge-carrying door, a
    // fronting door, the plain reachable remainder, and an unreachable interior.
    let mut config = fixture(
        "tier-partition",
        &[
            (
                "src/lib.rs",
                "pub mod api;\npub mod report;\npub mod rogue;\npub mod hub;\npub mod front;\nmod internal;\n",
            ),
            (
                "src/hub.rs",
                "pub mod nothing_here;\npub use crate::api::Credit;\n",
            ),
            (
                "src/api.rs",
                "pub struct Credit;\nimpl Construction for Credit { }\n",
            ),
            ("src/report.rs", "pub fn render() -> i64 { 1 }\n"),
            ("src/internal.rs", "fn helper() {}\n"),
            (
                "src/front.rs",
                "pub struct Door;\nimpl Door { pub fn go(&self) { crate::internal::helper() } }\n",
            ),
            ("src/rogue.rs", "pub fn misc() -> i64 { 2 }\n"),
        ],
    );
    config.kernel_allowlist = vec!["src/lib.rs".to_string()];
    let census = Enforcement::run(&config).tiers_census;
    assert!(
        census.contains("# 7 files: 2 boundary, 1 interior, 3 algebra, 1 kernel."),
        "{census}"
    );
    assert!(
        census.contains("- src/api.rs: BOUNDARY (pub-reachable, carries production edges)"),
        "{census}"
    );
    assert!(
        census.contains("- src/front.rs: BOUNDARY (pub-reachable, fronts an interior sibling)"),
        "{census}"
    );
    assert!(
        census.contains(
            "- src/hub.rs: ALGEBRA (glue — module declarations and re-exports only; tier by reachability)"
        ),
        "{census}"
    );
    assert!(
        census.contains("- src/internal.rs: INTERIOR (not pub-reachable)"),
        "{census}"
    );
    assert!(
        census.contains(
            "- src/rogue.rs: ALGEBRA (pub-reachable, no production edges, fronts nothing)"
        ),
        "{census}"
    );
    assert!(
        census.contains("- src/lib.rs: KERNEL (registered — a decision, never derived)"),
        "{census}"
    );
}

#[test]
fn a_stale_tier_census_is_a_violation() {
    let mut config = fixture("tier-census-stale", &[("src/lib.rs", "pub struct Foo;\n")]);
    let spec = config.manifest_dir.join("spec/tiers.spec");
    std::fs::create_dir_all(spec.parent().unwrap()).unwrap();
    std::fs::write(&spec, "old\n").unwrap();
    config.tiers_spec = Some(spec);
    // a per-test bless variable, so a real BLESS_TIERS in the environment can't flip this
    // into a bless run:
    config.tiers_bless_env = "BLESS_TIERS_ENFORCE_TEST".to_string();
    let vs = Enforcement::run(&config).violations;
    assert_fires(&vs, "the tier census drifted");
}
