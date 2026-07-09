//! probe-hook — the edit-time ENVELOPE as a SHIPPED binary (the eleventh ask).
//!
//! The whole edit-time sense organ, every voice priced in silence — this hook is FOR the
//! agent, so it surfaces everything a path-and-text edit can derive:
//!
//! * the GUARD (`Agenda::edit_guard`) — refusals that already exist downstream, pre-fired;
//! * the SHAPE TICKER (`discover::watch::Ticker`) — a coupling move on ANY Rust edit: a theory
//!   nets on its sorts, a plain module on its own declared types, and the ticker speaks only
//!   when the edit bridges two clusters or opens a new one — a sixth sense, not a firehose;
//! * the TIER voice (`spec/tiers.spec`) — on first edit of a file, its derived tier and the
//!   rules it carries (the reader-service the deleted `//! Tier:` markers gave);
//! * the FREEZE-DELTA courier (`freeze_delta_voice`) — the recommendation movement the last
//!   build derived via `spec_lock::Lock::delta` (a placement re-settling, a seam candidate
//!   appearing), inserted once into the window and then cleared.
//!
//! The first three voices the hook DERIVES from a single text edit. The fourth it does not
//! compute at all: distance, cohesion, and placement need the compiled theory (running `eval`),
//! which a text edit cannot afford — so the movement of those recommendations is derived where
//! it is cheap, at the build that emits the locks (`spec_lock::Lock::delta` holds both sides at
//! that instant), and the hook only COURIERS the result into the next context window. Nothing is
//! watched or reconstructed: the emitter narrates its own delta, the hook carries it. (The
//! freedom/survivor census stays at session start, read from the committed mutation locks.) All
//! the voices are mutation-tested and register-driven, and before this crate every
//! consumer wrapped them in the same four pieces of unjudged glue: a bash wrapper, inline
//! JSON-parsing Python, a build-on-demand fallback that could run a stale binary, and
//! hand-authored `settings.json` plumbing. This crate is that envelope, inside the boundary —
//! and this repo now dogfoods it in its own `.claude/settings.json`, the retired
//! `shape-watch.sh` being exactly the glue described:
//!
//! * **speaks the Claude Code hook protocol natively** — reads the PostToolUse JSON
//!   from stdin, extracts `tool_input.file_path`, honours `CLAUDE_PROJECT_DIR`;
//! * **discovers the repo's own declarations** — voices derived from the tree
//!   (`GuardVoices::for_edit`), classes taught from `spec/agenda.register`; the
//!   consumer writes zero code;
//! * **carries the fail-open contract inside the boundary** — every internal failure
//!   (malformed JSON, missing path, unreadable file, refused register) is silence,
//!   as a drilled property of [`respond`], not a `|| exit 0` convention;
//! * **installs its own wiring** — [`install`] writes or idempotently merges the
//!   `settings.json` entry, so the plumbing is derived output, never hand-authored.
//!
//! Honest frame — version skew: a globally installed `probe-hook` can be newer or
//! older than the probe-algebra a repo pins, and the two can disagree about register
//! grammar or voice derivation. The floor shipped here: every non-silent voice block
//! carries the binary's version on its last line, and the guard is advisory and
//! fail-open, so skew degrades to weaker advice, never a false refusal. (Re-execing a
//! repo-local build is the known nicer form; deliberately not built until skew is
//! observed hurting.)

use std::path::Path;

use boundary_spec::discover::agenda::{Agenda, GuardVoices};
use boundary_spec::discover::watch::Ticker;

/// The version tag every non-silent voice block carries — the skew floor.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The edited file's path, out of the hook protocol's stdin JSON
/// (`tool_input.file_path`). `None` for anything else — malformed JSON, a tool event
/// without a path — because the hook's silence must be total on input it does not
/// understand.
pub fn extract_path(hook_json: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(hook_json).ok()?;
    value
        .get("tool_input")?
        .get("file_path")?
        .as_str()
        .map(str::to_string)
}

/// The whole envelope, total: hook JSON in, whichever of the guard / shape-ticker / tier
/// voices fire out (joined, one version footer), or `None`. Every failure path is `None` — the fail-open
/// contract as a return type. The voices are derived from `project_dir` (the register, the
/// shim evidence), the classes come from `spec/agenda.register`, and the edited path is
/// normalized repo-relative so messages match what the repo's own gates would say.
///
/// Capability: Effectful — reads the edited file, the registers, and `build.rs`
/// under `project_dir`.
pub fn respond(hook_json: &str, project_dir: &Path) -> Option<String> {
    let path = extract_path(hook_json)?;
    // one file, one name: the Edit tool hands absolute paths — normalized here so
    // guard messages and class matching see the repo-relative form.
    let rel = project_dir
        .to_str()
        .and_then(|root| path.strip_prefix(&format!("{root}/")))
        .unwrap_or(path.as_str())
        .to_string();
    let source = std::fs::read_to_string(&path).unwrap_or_default();

    // FOUR voices, each priced in silence — the whole edit-time envelope, so the shipped
    // binary fully replaces the bash wrapper it descends from (the fourth, the freeze-delta
    // courier, the hook carries rather than computes):
    let mut blocks: Vec<String> = Vec::new();

    // the GUARD — refusals that already exist downstream (a hand-edited generated lock, a
    // loose `pub fn` the shim refuses), voices derived from the tree, classes taught from
    // spec/agenda.register (a refused register fails open to "no taught classes").
    let voices = GuardVoices::for_edit(project_dir, &rel);
    let classes = spec_lock::Register {
        name: "agenda".to_string(),
        path: project_dir.join("spec/agenda.register"),
    }
    .entries()
    .unwrap_or_default();
    if let Some(guard) = Agenda::edit_guard(&rel, &source, &voices, &classes) {
        blocks.push(guard);
    }

    // the SHAPE TICKER — speaks only when an edit moves the coupling (a bridge, a new
    // net-disjoint component) on ANY Rust file, stateful across invocations (see [`shape_voice`]).
    if let Some(shape) = shape_voice(project_dir, &rel, &source) {
        blocks.push(shape);
    }

    // the TIER voice — on FIRST edit of a file, its derived tier and the rules that tier
    // carries (the reader-service the deleted `//! Tier:` markers gave, moved to the hook).
    if let Some(tier) = tier_voice(project_dir, &rel) {
        blocks.push(tier);
    }

    // the FREEZE-DELTA courier — the recommendation movement the last build derived (via
    // `spec_lock::Lock::delta`) and left at `target/probe-hook/freeze-delta`. The hook does
    // not COMPUTE it — it inserts it once into this context window and clears the courier.
    if let Some(delta) = freeze_delta_voice(project_dir) {
        blocks.push(delta);
    }

    if blocks.is_empty() {
        return None;
    }
    Some(format!(
        "{}\n(probe-hook {VERSION} — advisory, fail-open)",
        blocks.join("\n")
    ))
}

/// The shape ticker's voice for one edit — the coupling sense, folded in from the retired
/// `place_watch --event` wrapper and now widened to ANY Rust file, not just theories. Two
/// fronts onto one placer core: a `.rs` carrying `ops {` stanzas nets on its theory sorts
/// (`parse_ops`); any other `.rs` nets on its OWN declared types (`parse_rust_sigs` — the
/// module's structs/enums are its sorts, ubiquitous types never couple). Either way the
/// placement is re-derived from TEXT (no compilation), diffed against the previous placement
/// kept in a per-file state slug under `<project>/target/probe-hook`, and a move is narrated in
/// the monotone vocabulary (a second net-disjoint component forming, or a BRIDGE coupling two).
/// The noise policy lives in the ticker: an edit within one cluster is silence, so this stays a
/// sixth sense, not a firehose. Fail-open throughout: an unparseable (half-written) file or any
/// unreadable/unwritable state is silence (`.ok()?`), never a broken edit loop.
///
/// Capability: Effectful — reads and writes the ticker state under `project_dir/target`.
fn shape_voice(project_dir: &Path, rel: &str, source: &str) -> Option<String> {
    if !rel.ends_with(".rs") {
        return None;
    }
    // pick the front by content: theory sorts if there is an `ops { }` stanza, else the
    // module's own types. A half-written file that neither parser accepts falls silent.
    let sigs = if source.contains("ops {") {
        Ticker::parse_ops(source)
    } else {
        Ticker::parse_rust_sigs(source)
    }
    .ok()?;

    let state_dir = project_dir.join("target/probe-hook");
    let slug: String = rel
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let state = state_dir.join(format!("{slug}.sigs"));
    // the ticker keys on a `&'static` name; the process is short-lived, so a leak per hook
    // invocation is the price of the borrow (the same trade the example made).
    let name: &'static str = Box::leak(rel.to_string().into_boxed_str());
    // the stored form is signature-level and parser-agnostic, so resume works for either front.
    let mut ticker = match std::fs::read_to_string(&state) {
        Ok(stored) => Ticker::resume(name, &stored),
        // first sight: capture the baseline; a multi-component file announces itself once.
        Err(_) => Ticker::new(),
    };
    let line = ticker.hook_line_signatures(name, sigs.clone());
    std::fs::create_dir_all(&state_dir).ok()?;
    std::fs::write(&state, Ticker::store_signatures(&sigs)).ok()?;
    line
}

/// The TIER voice — one line on the FIRST edit of a file: its DERIVED tier and what that
/// tier's membership means, BOTH read from the committed `spec/tiers.spec` (the single source
/// the build's rule dispatch also consumes). The tier comes from the file's `- <path>: <TIER>`
/// row; the meaning comes from the lock's `# rule <TIER>:` legend, which `boundary-enforce`
/// renders — so the hook recites what is ENFORCED, from disk, never a compiled-in copy that
/// could drift from a newer enforcer. (An older lock without the legend still names the tier
/// and points at regeneration; it never guesses the rules.) This is the reader-service the
/// deleted `//! Tier:` markers gave, moved to the edit hook. Fires once per file (a persisted
/// marker under `<project>/target/probe-hook`), so the orientation is paid once, not on every
/// save. Silent for a file the partition does not name, and fail-open on any read/write failure.
///
/// Capability: Effectful — reads `spec/tiers.spec`, reads and writes the seen-marker.
fn tier_voice(project_dir: &Path, rel: &str) -> Option<String> {
    let tiers = std::fs::read_to_string(project_dir.join("spec/tiers.spec")).ok()?;
    // the committed format is `- <path>: <TIER> (<reason>)`, one line per file.
    let prefix = format!("- {rel}: ");
    let rest = tiers
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(&prefix))?;
    let tier = rest.split_whitespace().next()?;

    let slug: String = rel
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let seen = project_dir
        .join("target/probe-hook")
        .join(format!("{slug}.tier"));
    if seen.exists() {
        return None;
    }

    // the rule prose is READ from the lock's legend, never held here — so what the hook recites
    // is exactly what the enforcer that wrote the lock forbids. A lock without the legend (an
    // older enforcer, a consumer yet to regenerate) gets the tier and a pointer, not a guess.
    let legend = format!("# rule {tier}: ");
    let line = match tiers
        .lines()
        .find_map(|l| l.trim_start().strip_prefix(&legend))
    {
        Some(rules) => format!("tier: {rel} is {tier} — {}", rules.trim()),
        None => format!("tier: {rel} is {tier} — regenerate spec/tiers.spec for this tier's rules"),
    };
    std::fs::create_dir_all(seen.parent()?).ok()?;
    std::fs::write(&seen, "").ok()?;
    Some(line)
}

/// The FREEZE-DELTA courier — the fourth voice, and the only one the hook does NOT compute.
/// It carries a recommendation movement (`interpreter arithmetic` re-placed as two, a seam
/// candidate on `Int` appeared) that `spec_lock::Lock::delta` already derived at the build
/// which produced it: `examples/freeze_spec` holds each lock's committed text against the live
/// text it just derived — the diff the drift gate collapses to a bool — and writes the rendered
/// movement to `target/probe-hook/freeze-delta`. The hook reads that courier, injects it ONCE,
/// and clears it, so the movement reaches the next context window after the build that caused it
/// and never lingers. Nothing here re-derives or watches: the mechanism is native to `delta()`,
/// run at freeze time; this is only the wire into the window.
///
/// Fail-open: a missing/empty/unreadable courier is silence, and if the clear-on-consume write
/// fails the voice stays silent rather than risk repeating the same movement every edit.
///
/// Capability: Effectful — reads and truncates the courier under `project_dir/target`.
fn freeze_delta_voice(project_dir: &Path) -> Option<String> {
    let courier = project_dir.join("target/probe-hook/freeze-delta");
    let narration = std::fs::read_to_string(&courier).ok()?;
    let narration = narration.trim();
    if narration.is_empty() {
        return None;
    }
    // consume before speaking: if the courier cannot be cleared, stay silent — better an
    // unseen movement than the same one re-injected on every subsequent edit.
    std::fs::write(&courier, "").ok()?;
    Some(format!(
        "your last freeze moved these recommendations (from spec_lock::Lock::delta):\n{narration}"
    ))
}

/// The settings entry this crate wires for itself.
const MATCHER: &str = "Edit|Write";

/// Write or merge the `.claude/settings.json` hook entry — the plumbing as DERIVED
/// output. Idempotent: an entry whose command already invokes `probe-hook` is left
/// alone; everything else in the file is preserved untouched. Returns a line saying
/// what happened; errs only on a `settings.json` that exists but does not parse
/// (never overwrite what cannot be read — that file is not ours).
///
/// Capability: Effectful — reads and writes `.claude/settings.json` under
/// `project_dir`.
pub fn install(project_dir: &Path) -> Result<String, String> {
    let path = project_dir.join(".claude/settings.json");
    let mut root: serde_json::Value = match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| {
            format!(
                "{} exists but does not parse ({e}) — refusing to touch a settings \
                 file that cannot be read back",
                path.display()
            )
        })?,
        Err(_) => serde_json::json!({}),
    };

    let post = root
        .as_object_mut()
        .ok_or("settings.json is not a JSON object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("settings.json `hooks` is not an object")?
        .entry("PostToolUse")
        .or_insert_with(|| serde_json::json!([]));
    let entries = post
        .as_array_mut()
        .ok_or("settings.json `hooks.PostToolUse` is not an array")?;

    let already = entries.iter().any(|entry| {
        entry["hooks"]
            .as_array()
            .is_some_and(|hooks| hooks.iter().any(|h| h["command"] == "probe-hook"))
    });
    if already {
        return Ok(format!(
            "probe-hook {VERSION}: already installed — no change"
        ));
    }

    entries.push(serde_json::json!({
        "matcher": MATCHER,
        "hooks": [{ "type": "command", "command": "probe-hook", "timeout": 10 }]
    }));
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("create {} ({e})", parent.display()))?;
    }
    let rendered = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    std::fs::write(&path, rendered + "\n")
        .map_err(|e| format!("write {} ({e})", path.display()))?;
    Ok(format!(
        "probe-hook {VERSION}: installed PostToolUse({MATCHER}) into {}",
        path.display()
    ))
}

#[cfg(test)]
mod drills {
    use super::*;

    fn tree(name: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("probe-hook-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        for (rel, contents) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }
        root
    }

    fn event(path: &std::path::Path) -> String {
        serde_json::json!({ "tool_input": { "file_path": path.to_str().unwrap() } }).to_string()
    }

    /// THE FAIL-OPEN CONTRACT, drilled — the property the per-consumer `|| exit 0`
    /// conventions never tested: malformed JSON, an event without a path, a path that
    /// does not exist, and a refused class register are all SILENCE, not errors.
    #[test]
    fn every_failure_path_is_silence() {
        let root = tree(
            "fail-open",
            &[("spec/agenda.register", "bare key no colon\n")],
        );
        assert_eq!(respond("not json at all {", &root), None);
        assert_eq!(respond("{\"tool_input\":{}}", &root), None);
        assert_eq!(respond("{\"tool_input\":{\"file_path\":123}}", &root), None);
        // the register above REFUSES to parse — the guard still answers (fail-open to
        // no taught classes), and an unremarkable file is silence:
        let doc = root.join("docs/note.md");
        std::fs::create_dir_all(doc.parent().unwrap()).unwrap();
        std::fs::write(&doc, "prose\n").unwrap();
        assert_eq!(respond(&event(&doc), &root), None);
    }

    /// The envelope end to end: a generated-lock edit warns (with the version-tagged
    /// skew floor), a taught consumer class warns as a lock instead of refusing as
    /// unknown, and the structural voice speaks only where the shim's refusal exists.
    #[test]
    fn the_envelope_speaks_with_derived_voices_and_taught_classes() {
        let root = tree(
            "voices",
            &[
                ("spec/router.spec", "# a lock\n"),
                (
                    "spec/agenda.register",
                    "surface.lock: the surface census moved — admit the new commands.\n",
                ),
                ("spec/custom.surface.lock", "# consumer lock\n"),
                (
                    "src/gates.rs",
                    "pub fn pipeline() -> Pipeline { todo!() }\n",
                ),
            ],
        );
        // a generated lock warns, and the voice block carries the version (skew floor):
        let voice = respond(&event(&root.join("spec/router.spec")), &root).expect("a lock warns");
        assert!(voice.contains("never hand-edit"), "{voice}");
        assert!(voice.contains(&format!("probe-hook {VERSION}")), "{voice}");
        // a TAUGHT class is a known lock (never "teach the router"):
        let voice =
            respond(&event(&root.join("spec/custom.surface.lock")), &root).expect("taught warns");
        assert!(voice.contains("never hand-edit"), "{voice}");
        // no shim in this tree: the loose pub fn is NOT a refusal here — silence.
        assert_eq!(respond(&event(&root.join("src/gates.rs")), &root), None);
        // the same tree WITH shim evidence: the structural voice exists and speaks.
        std::fs::write(
            root.join("build.rs"),
            "use boundary_enforce::Enforcement;\nfn main() {}\n",
        )
        .unwrap();
        let voice = respond(&event(&root.join("src/gates.rs")), &root).expect("loose fn warns");
        assert!(
            voice.contains("`pub fn pipeline` is a loose public function"),
            "{voice}"
        );
    }

    /// THE SECOND VOICE, drilled — a theory edit that splits into net-disjoint features
    /// makes the shape ticker speak (the half folded in from the retired `place_watch`
    /// wrapper), while an ordinary `.rs` with a single type cluster stays silent. The version
    /// footer rides both voices.
    #[test]
    fn a_theory_edit_speaks_the_shape_voice() {
        let root = tree("shape", &[]);
        let thy = root.join("src/workbench.rs");
        std::fs::create_dir_all(thy.parent().unwrap()).unwrap();
        // two net-disjoint features in one bundle: the ticker announces the split on sight.
        let ops = "    ops {\n        Nullary \"zero\" \"zero\" () -> S::A = zero;\n        \
                   Nullary \"off\" \"off\" () -> S::B = off;\n    }\n";
        std::fs::write(&thy, ops).unwrap();
        let voice =
            respond(&event(&thy), &root).expect("a multi-component theory announces its shape");
        assert!(voice.contains("net-disjoint"), "{voice}");
        assert!(voice.contains(&format!("probe-hook {VERSION}")), "{voice}");
        // a plain .rs with one type and no coupling structure is silence.
        let plain = root.join("src/plain.rs");
        std::fs::write(&plain, "pub struct X;\n").unwrap();
        assert_eq!(respond(&event(&plain), &root), None);
    }

    /// THE SIXTH SENSE, drilled — the coupling voice now fires on ANY Rust file, netting on the
    /// module's OWN types. A two-cluster module announces itself once; then the function that
    /// first spans both clusters BRIDGES them, as it is saved. The whole point of the widening:
    /// no `ops { }`, no theory, just plain Rust getting the live coupling sense.
    #[test]
    fn a_plain_rust_edit_speaks_the_coupling_voice() {
        let root = tree("coupling", &[]);
        let m = root.join("src/billing.rs");
        std::fs::create_dir_all(m.parent().unwrap()).unwrap();
        // Order|Invoice one cluster, Ledger another — two net-disjoint components on first sight.
        let two = "struct Order; struct Invoice; struct Ledger;\n\
                   fn bill(o: Order) -> Invoice { todo!() }\n\
                   fn post(l: Ledger) -> Ledger { todo!() }\n";
        std::fs::write(&m, two).unwrap();
        let voice = respond(&event(&m), &root).expect("a two-cluster module announces itself");
        assert!(voice.contains("net-disjoint components"), "{voice}");
        // an edit within one cluster: silence (the sixth sense, not a firehose).
        let local = format!("{two}fn refund(o: Order) -> Order {{ todo!() }}\n");
        std::fs::write(&m, &local).unwrap();
        assert_eq!(respond(&event(&m), &root), None);
        // the function that spans both clusters bridges them, at the moment it is written.
        let bridge = format!("{local}fn reconcile(i: Invoice) -> Ledger {{ todo!() }}\n");
        std::fs::write(&m, &bridge).unwrap();
        let voice = respond(&event(&m), &root).expect("the spanning fn bridges");
        assert!(
            voice.contains("BRIDGED") && voice.contains("intended?"),
            "{voice}"
        );
    }

    /// THE THIRD VOICE, drilled — a file's derived tier and rules on FIRST edit, paid once
    /// (a second edit is silent), and silent for a file the partition does not name. The rule
    /// prose is READ from the lock's `# rule <TIER>:` legend, never compiled in — so a lock
    /// WITHOUT the legend names the tier and points at regeneration rather than guessing.
    #[test]
    fn the_tier_voice_reads_its_rules_from_the_lock_legend() {
        let root = tree(
            "tier",
            &[
                (
                    "spec/tiers.spec",
                    "# the partition\n# rule BOUNDARY: tier 1 — a domain's surface; no loose `pub fn`\n\
                     - src/engine.rs: BOUNDARY (a door)\n",
                ),
                ("src/engine.rs", "pub struct X;\n"),
            ],
        );
        let ev = event(&root.join("src/engine.rs"));
        let first = respond(&ev, &root).expect("the first edit orients");
        assert!(first.contains("tier: src/engine.rs is BOUNDARY"), "{first}");
        // the rule text came from the lock's legend, not a copy in the binary:
        assert!(first.contains("no loose `pub fn`"), "{first}");
        // the orientation was paid — a second edit of the same file is silence.
        assert_eq!(respond(&ev, &root), None);
        // a file the partition does not name: silence.
        let other = root.join("src/unlisted.rs");
        std::fs::write(&other, "pub struct Y;\n").unwrap();
        assert_eq!(respond(&event(&other), &root), None);

        // a lock that names the tier but carries NO legend line: the tier still surfaces, and
        // the hook points at regeneration instead of reciting a rule it no longer holds.
        let bare = tree(
            "tier-no-legend",
            &[
                (
                    "spec/tiers.spec",
                    "# the partition\n- src/x.rs: ALGEBRA (remainder)\n",
                ),
                ("src/x.rs", "pub struct Z;\n"),
            ],
        );
        let voice =
            respond(&event(&bare.join("src/x.rs")), &bare).expect("tier still names itself");
        assert!(voice.contains("tier: src/x.rs is ALGEBRA"), "{voice}");
        assert!(voice.contains("regenerate spec/tiers.spec"), "{voice}");
    }

    /// THE FOURTH VOICE, drilled — the courier carries a movement `delta()` derived at freeze
    /// time into the window ONCE, then clears itself, and an empty/absent courier is silence.
    /// The hook computes nothing here; it only wires the emitter's narration into context.
    #[test]
    fn the_freeze_delta_courier_injects_once_then_clears() {
        let root = tree("courier", &[("src/plain.rs", "pub struct X;\n")]);
        // an empty courier (a build with no movement) is silence.
        let courier = root.join("target/probe-hook/freeze-delta");
        std::fs::create_dir_all(courier.parent().unwrap()).unwrap();
        std::fs::write(&courier, "").unwrap();
        assert_eq!(respond(&event(&root.join("src/plain.rs")), &root), None);
        // a real movement (as `spec_lock::LockDelta::render` would write it) is carried once.
        std::fs::write(
            &courier,
            "lock `boundary-spec` moved:\n  - verdict: 7 of 7 settled\n  + verdict: 6 of 7 settled\n",
        )
        .unwrap();
        let voice = respond(&event(&root.join("src/plain.rs")), &root).expect("the courier speaks");
        assert!(
            voice.contains("your last freeze moved these recommendations"),
            "{voice}"
        );
        assert!(voice.contains("6 of 7 settled"), "{voice}");
        // consumed: the very next edit is silent — the movement is injected once, not per save.
        assert_eq!(respond(&event(&root.join("src/plain.rs")), &root), None);
        assert_eq!(
            std::fs::read_to_string(&courier).unwrap(),
            "",
            "the courier is cleared on consume"
        );
    }

    /// `install` is derived plumbing: creates the file from nothing, is idempotent,
    /// and preserves everything it did not write — including an existing unrelated
    /// hook in the same PostToolUse list. A settings file that does not parse is
    /// REFUSED, never overwritten.
    #[test]
    fn install_wires_itself_and_touches_nothing_else() {
        let root = tree("install", &[]);
        let done = install(&root).expect("installs from nothing");
        assert!(done.contains("installed PostToolUse(Edit|Write)"), "{done}");
        let again = install(&root).expect("idempotent");
        assert!(again.contains("already installed"), "{again}");

        // an existing settings file with an unrelated hook survives the merge intact:
        let root = tree(
            "install-merge",
            &[(
                ".claude/settings.json",
                r#"{"model":"opus","hooks":{"PostToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"other-tool"}]}]}}"#,
            )],
        );
        install(&root).expect("merges");
        let text = std::fs::read_to_string(root.join(".claude/settings.json")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["model"], "opus", "unrelated settings preserved");
        let post = value["hooks"]["PostToolUse"].as_array().unwrap();
        assert_eq!(post.len(), 2, "the unrelated hook survives alongside ours");
        assert_eq!(post[0]["hooks"][0]["command"], "other-tool");
        assert_eq!(post[1]["hooks"][0]["command"], "probe-hook");

        // a corrupt settings file is refused by name, never clobbered:
        let root = tree(
            "install-corrupt",
            &[(".claude/settings.json", "{ not json")],
        );
        let err = install(&root).unwrap_err();
        assert!(err.contains("does not parse"), "{err}");
        assert_eq!(
            std::fs::read_to_string(root.join(".claude/settings.json")).unwrap(),
            "{ not json",
            "the unreadable file is untouched"
        );
    }
}
