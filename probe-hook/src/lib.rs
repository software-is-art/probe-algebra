//! probe-hook — the edit-time ENVELOPE as a SHIPPED binary (the eleventh ask).
//!
//! The whole edit-time sense organ, every voice priced in silence — this hook is FOR the
//! agent, so it surfaces everything a path-and-text edit can derive:
//!
//! * the GUARD (`Agenda::edit_guard`) — refusals that already exist downstream, pre-fired;
//! * the SHAPE TICKER (`discover::watch::Ticker`) — a coupling move on ANY Rust edit: a theory
//!   nets on its sorts, a plain module on its own declared types, and the ticker speaks only
//!   when the edit bridges two clusters or opens a new one — a sixth sense, not a firehose;
//! * the QUALIFY voice (`spec/qualify.spec`) — the edit-time LOCK DELTA as a DRIFT LEDGER: the whole
//!   current drift of the surface census, re-derived from the tree via
//!   `boundary_enforce::qualify_census_lines` and diffed against the committed lock. Surfaced on ANY
//!   edit (un-scoped) but DEDUPED against the last-shown ledger, so it speaks only when the drift
//!   moves — the behavioural mirror at the edit, for the one lock a text edit can re-derive (no
//!   how-to-bless recipe: the named lock's own header carries it);
//! * the TYPE-LIBRARY voice (`library_voice`) — the anti-duplication sense (rung 0 of the
//!   bundle candidate): when the edited file touches sorts that already carry operators in
//!   OTHER files' census lines, the existing families are named — per-file, deduped, the
//!   census intersection doubling as the noise filter;
//! * the TIER voice (`spec/tiers.spec`) — on first edit of a file, its derived tier and the
//!   rules it carries (the reader-service the deleted `//! Tier:` markers gave);
//! * the FREEZE-DELTA courier (`freeze_delta_voice`) — the recommendation movement the last
//!   build derived via `spec_lock::Lock::delta` (a placement re-settling, a seam candidate
//!   appearing), inserted once into the window and then cleared.
//!
//! The first five voices the hook DERIVES from a single text edit. The sixth it does not
//! compute at all: distance, cohesion, and placement need the compiled theory (running `eval`),
//! which a text edit cannot afford — so the movement of those recommendations is derived where
//! it is cheap, at the build that emits the locks (`spec_lock::Lock::delta` holds both sides at
//! that instant), and the hook only COURIERS the result into the next context window. The qualify
//! voice is the boundary case that DOES fit a text edit: qualification is a structural property
//! (operator-shaped functions), so its lock delta is computed live here, while the behavioural
//! locks stay couriered. Nothing is watched or reconstructed: the emitter narrates its own delta,
//! the hook carries it (or, for qualify, derives it). (The
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

    // SIX voices, each priced in silence — the whole edit-time envelope, so the shipped
    // binary fully replaces the bash wrapper it descends from (the last, the freeze-delta
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

    // the TYPE-LIBRARY voice — the anti-duplication sense: when the edited file touches
    // sorts that already carry operators ELSEWHERE in the committed qualify census, name
    // those files and their operator families before a twin gets written (see
    // [`library_voice`]).
    if let Some(library) = library_voice(project_dir, &rel, &source) {
        blocks.push(library);
    }

    // the QUALIFY voice — the edit-time lock delta as a DRIFT LEDGER: the whole current drift of
    // `spec/qualify.spec`, re-derived from the tree on disk, surfaced on ANY edit (not scoped to
    // certain files) but DEDUPED so it speaks only when the drift actually moves — see
    // [`qualify_voice`]. Narrated BEFORE the build that would otherwise be first to notice.
    if let Some(qualify) = qualify_voice(project_dir) {
        blocks.push(qualify);
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

/// The QUALIFY voice — the edit-time LOCK DELTA as a DRIFT LEDGER, un-scoped and deduped. The
/// qualify census (`spec/qualify.spec`) is text-derivable — a module qualifies by the SHAPE of its
/// functions (operator-shaped, no I/O), no `eval` — so the whole live census is re-derived from the
/// tree on disk and diffed against the committed lock, and the delta IS the drift the next
/// `BLESS_QUALIFY=1` build would ratify. It shows the FULL current drift, every stale line together,
/// accumulating as files move and empty the moment a re-bless reconciles the tree. The behavioural
/// half of the mirror — distance, discovered laws — still needs the compiled theory and stays at the
/// build (the freeze-delta courier carries it); qualification is the one lock a text edit re-derives.
///
/// Two design choices, both deliberate:
/// * UN-SCOPED — the ledger is a property of the whole tree, surfaced on ANY edit, not narrowed to
///   `src/*.rs` or any file class. Qualify drift is a repo fact; which file you happened to touch
///   should not gate whether you see it.
/// * DEDUPED, not naggy — because the standing drift renders identically on every edit until it
///   moves, the voice persists the last-shown ledger under `target/probe-hook` and speaks ONLY when
///   the render differs: an accumulated line appears once, on whatever edit first surfaces it, then
///   stays quiet until the drift moves again; a re-bless empties it and the next real drift
///   re-announces. So breadth of triggering does not become breadth of repetition.
///
/// A file that does not parse contributes nothing to the live census (`qualify_census_lines` skips
/// it), so a half-written save never invents a movement. NO recipe: the delta names
/// `spec/qualify.spec`, whose own header carries the regenerate command
/// (`# … Regenerate with \`BLESS_QUALIFY=1 cargo build\``) — how-to-bless is self-documenting at the
/// named lock and stable orientation (also CLAUDE.md's one rule), not news to reprint each firing.
/// The movement renders through `spec_lock::LockDelta`, the renderer the freeze-delta courier uses.
///
/// A repo with no census pays nothing: the missing lock returns before any tree scan.
///
/// Capability: Effectful — reads `spec/qualify.spec`, rescans the `src/` tree, and reads/writes the
/// dedup state under `project_dir`.
/// The TYPE-LIBRARY voice — the anti-duplication sense (rung 0 of the bundle candidate,
/// docs/roadmap.md): when the edited file touches SORTS that already carry operators in
/// OTHER files, whisper which files and which operator families, so the existing vocabulary
/// is in the window before a twin gets written. The library is the committed
/// `spec/qualify.spec` — derived, ratified, cheap to read (the tier voice's move) — and the
/// edited file's side is `Ticker::type_vocabulary` (every type ident its signatures
/// mention, plus its own declared types). The census intersection IS the noise filter:
/// ubiquitous types (`String`, `Vec`, `Result`) never appear as census sorts, so nothing
/// wires to everything.
///
/// Priced in silence, the standing rules: only Rust edits; the edited file's own census
/// line never speaks (its operators are not news to itself); no intersection is silence;
/// and the render is DEDUPED per file (`target/probe-hook/<slug>.library` holds the last
/// shown text), so the library speaks on first contact and again only when the overlap
/// CHANGES — a grown family, a new sharing file, a dropped sort.
///
/// Capability: Effectful — reads `spec/qualify.spec` and the per-file dedup state.
fn library_voice(project_dir: &Path, rel: &str, source: &str) -> Option<String> {
    if !rel.ends_with(".rs") {
        return None;
    }
    let census = std::fs::read_to_string(project_dir.join("spec/qualify.spec")).ok()?;
    let vocabulary = Ticker::type_vocabulary(source).ok()?;

    let mut lines: Vec<String> = Vec::new();
    for line in census.lines() {
        // the committed format: `<path>: QUALIFIES — operators [..] over sorts {..}`;
        // header comments and blank lines simply do not match.
        let Some((path, rest)) = line.split_once(": QUALIFIES — operators [") else {
            continue;
        };
        let Some((operators, sorts)) = rest.split_once("] over sorts {") else {
            continue;
        };
        let Some(sorts) = sorts.strip_suffix('}') else {
            continue;
        };
        if path == rel {
            continue;
        }
        let shared: Vec<&str> = sorts
            .split(", ")
            .filter(|s| vocabulary.contains(*s))
            .collect();
        if shared.is_empty() {
            continue;
        }
        lines.push(format!(
            "  {path}: shares {{{}}} — operators [{operators}]",
            shared.join(", ")
        ));
    }
    if lines.is_empty() {
        return None;
    }
    let rendered = format!(
        "type library — sorts this file touches already carry operators elsewhere:\n{}",
        lines.join("\n")
    );

    let slug: String = rel
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let state = project_dir
        .join("target/probe-hook")
        .join(format!("{slug}.library"));
    if std::fs::read_to_string(&state).is_ok_and(|s| s == rendered) {
        return None;
    }
    std::fs::create_dir_all(project_dir.join("target/probe-hook")).ok()?;
    std::fs::write(&state, &rendered).ok()?;
    Some(rendered)
}

fn qualify_voice(project_dir: &Path) -> Option<String> {
    // a repo without the census returns here, before any scan — the feature exists only where the
    // lock does, which is not the same as scoping which EDITS may surface it.
    let committed_text = std::fs::read_to_string(project_dir.join("spec/qualify.spec")).ok()?;
    let committed: String = committed_text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    // the LIVE census body, rescanned from the tree on disk (the edit is already written) — the
    // WHOLE current drift, so every stale line sits together.
    let live =
        boundary_enforce::qualify_census_lines(&project_dir.join("src"), project_dir).join("\n");

    let state = project_dir.join("target/probe-hook/qualify-ledger");
    let delta = spec_lock::LockDelta::between(&committed, &live);
    if delta.is_empty() {
        // clean (nothing stale, or a re-bless just reconciled it): forget any shown ledger so the
        // NEXT real drift re-announces, and say nothing — an empty ledger is not worth a block.
        if std::fs::read_to_string(&state).is_ok_and(|s| !s.is_empty()) {
            let _ = std::fs::write(&state, "");
        }
        return None;
    }
    // DEDUP: the standing drift renders the same until it moves, so speak only when it differs from
    // what was last shown — breadth of triggering must not become breadth of repetition.
    let rendered = delta.render("qualify census (spec/qualify.spec)");
    if std::fs::read_to_string(&state).is_ok_and(|s| s == rendered) {
        return None;
    }
    std::fs::create_dir_all(project_dir.join("target/probe-hook")).ok()?;
    std::fs::write(&state, &rendered).ok()?;
    Some(rendered)
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

    /// THE TYPE-LIBRARY VOICE, drilled — the anti-duplication sense. An edit touching a sort
    /// that already carries operators ELSEWHERE names the file and its family, once; the same
    /// overlap again is silence (deduped per file); a GROWN overlap re-announces; a file whose
    /// types intersect nothing is silence; and the edited file's own census line never speaks.
    #[test]
    fn the_library_voice_names_existing_operator_families_once() {
        // the tree is CONSISTENT with its committed census (the qualify ledger stays
        // silent), so every block below is the library voice's alone. The edited file
        // does not qualify (borrowed arg) — but its signature MENTIONS Credits, an
        // imported domain type: exactly the twin-about-to-be-written moment.
        let meter = "pub struct Credits;\npub struct Order;\n\
             impl Credits {\n\
                 pub fn grant(self, o: Order) -> Credits { let _ = o; self }\n\
                 pub fn spend(self, c: Credits) -> Credits { c }\n\
             }\n";
        let gauge =
            "pub struct Level;\nimpl Level { pub fn fuse(self, l: Level) -> Level { l } }\n";
        let root = tree(
            "library",
            &[
                (
                    "spec/qualify.spec",
                    "# census\n\
                     src/gauge.rs: QUALIFIES — operators [Level::fuse] over sorts {Level}\n\
                     src/meter.rs: QUALIFIES — operators [Credits::grant, Credits::spend] over sorts {Credits, Order}\n",
                ),
                ("src/meter.rs", meter),
                ("src/gauge.rs", gauge),
                (
                    "src/new_work.rs",
                    "use crate::meter::Credits;\npub fn top_up(c: &Credits) -> Credits { c.spend(Credits) }\n",
                ),
                ("src/stranger.rs", "pub struct Unrelated;\n"),
            ],
        );
        let ev = event(&root.join("src/new_work.rs"));
        let voice = respond(&ev, &root).expect("the overlap speaks");
        assert!(
            voice.contains("type library — sorts this file touches already carry operators"),
            "{voice}"
        );
        assert!(
            voice.contains(
                "src/meter.rs: shares {Credits} — operators [Credits::grant, Credits::spend]"
            ),
            "{voice}"
        );
        assert!(!voice.contains("gauge"), "no Level overlap: {voice}");
        // deduped: the same overlap on the next edit is silence.
        assert_eq!(respond(&ev, &root), None);
        // a GROWN overlap re-announces: the edit now touches Order too.
        std::fs::write(
            root.join("src/new_work.rs"),
            "use crate::meter::{Credits, Order};\n\
             pub fn top_up(c: &Credits) -> Credits { c.spend(Credits) }\n\
             pub fn settle(o: &Order) -> Order { let _ = o; Order }\n",
        )
        .unwrap();
        let voice = respond(&ev, &root).expect("the grown overlap re-announces");
        assert!(voice.contains("shares {Credits, Order}"), "{voice}");
        // a file intersecting nothing is silence; the census file's OWN line never
        // speaks (its operators are not news to itself).
        assert_eq!(respond(&event(&root.join("src/stranger.rs")), &root), None);
        assert_eq!(
            respond(&event(&root.join("src/meter.rs")), &root),
            None,
            "a file's own operators are not news to itself"
        );
    }

    /// THE QUALIFY VOICE, drilled — the edit-time LOCK DELTA as an UN-SCOPED, DEDUPED drift ledger.
    /// A fresh tree is silence; a reshaping edit shows the census movement once with NO how-to-bless
    /// recipe; the SAME drift on a later edit is silence (deduped, not naggy); a grown ledger
    /// surfaces on ANY edit — including a non-`.rs` file — because it is un-scoped; and re-blessing
    /// empties it, so the next real drift re-announces.
    #[test]
    fn the_qualify_voice_is_an_unscoped_deduped_drift_ledger() {
        let root = tree(
            "qualify",
            &[
                (
                    "spec/qualify.spec",
                    "# census\nsrc/a.rs: QUALIFIES — operators [f] over sorts {A}\n",
                ),
                (
                    "src/a.rs",
                    "pub struct A;\npub fn f(x: A) -> A { todo!() }\n",
                ),
            ],
        );
        let eva = event(&root.join("src/a.rs"));
        // fresh: the live census reproduces the committed lock — silence.
        assert_eq!(respond(&eva, &root), None);

        // reshape a.rs (a second operator): the ledger moved (empty -> one line) → it speaks once.
        std::fs::write(
            root.join("src/a.rs"),
            "pub struct A;\npub fn f(x: A) -> A { todo!() }\npub fn g(x: A) -> A { todo!() }\n",
        )
        .unwrap();
        let v = respond(&eva, &root).expect("the census drift shows on change");
        assert!(v.contains("qualify census (spec/qualify.spec)"), "{v}");
        assert!(
            v.contains("+ src/a.rs: QUALIFIES — operators [f, g] over sorts {A}"),
            "{v}"
        );
        assert!(
            !v.contains("BLESS_QUALIFY") && !v.to_lowercase().contains("re-bless"),
            "the how-to-bless recipe is gone: {v}"
        );
        // DEDUP: the same drift on the next edit is silence — breadth of triggering is not breadth
        // of repetition.
        assert_eq!(respond(&eva, &root), None);

        // a SECOND file starts qualifying: the GROWN ledger surfaces on a NON-`.rs` edit — proof it
        // is un-scoped (which file you touch does not gate seeing the drift), and both lines sit.
        std::fs::write(
            root.join("src/b.rs"),
            "pub struct B;\npub fn h(x: B) -> B { todo!() }\n",
        )
        .unwrap();
        let doc = root.join("README.md");
        std::fs::write(&doc, "notes\n").unwrap();
        let v = respond(&event(&doc), &root).expect("a non-.rs edit surfaces the CHANGED ledger");
        assert!(
            v.contains("+ src/a.rs: QUALIFIES — operators [f, g] over sorts {A}"),
            "{v}"
        );
        assert!(
            v.contains("+ src/b.rs: QUALIFIES — operators [h] over sorts {B}"),
            "{v}"
        );
        // and the same grown drift on the next edit is silence again (deduped).
        assert_eq!(respond(&event(&doc), &root), None);

        // re-bless (the committed lock catches up to the tree): the ledger empties — silence.
        std::fs::write(
            root.join("spec/qualify.spec"),
            "# census\nsrc/a.rs: QUALIFIES — operators [f, g] over sorts {A}\n\
             src/b.rs: QUALIFIES — operators [h] over sorts {B}\n",
        )
        .unwrap();
        assert_eq!(respond(&eva, &root), None);
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
