//! probe-hook — the edit-time ENVELOPE as a SHIPPED binary (the eleventh ask).
//!
//! Two voices, both priced in silence: the GUARD (`Agenda::edit_guard` — refusals that
//! already exist downstream) and the SHAPE TICKER (`discover::watch::Ticker` — a layout
//! move on a theory edit). Both are mutation-tested and register-driven — and before this
//! crate, every consumer wrapped them in the same four pieces of unjudged glue: a bash
//! wrapper, inline JSON-parsing Python, a build-on-demand fallback that could run a stale
//! binary, and hand-authored `settings.json` plumbing. All of it outside the mutation
//! boundary, each consumer with different bugs. This crate is that envelope, inside the
//! boundary — and this repo now dogfoods it in its own `.claude/settings.json`, the retired
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

/// The whole envelope, total: hook JSON in, the guard and/or shape-ticker voices out
/// (joined, one version footer), or `None`. Every failure path is `None` — the fail-open
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

    // TWO voices, both priced in silence — the whole edit-time envelope, so the shipped
    // binary fully replaces the bash wrapper it descends from:
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

    // the SHAPE TICKER — speaks only when a theory edit moves the layout (a bridge, a new
    // net-disjoint component), stateful across invocations (see [`shape_voice`]).
    if let Some(shape) = shape_voice(project_dir, &rel, &source) {
        blocks.push(shape);
    }

    if blocks.is_empty() {
        return None;
    }
    Some(format!(
        "{}\n(probe-hook {VERSION} — advisory, fail-open)",
        blocks.join("\n")
    ))
}

/// The shape ticker's voice for one edit — the second half of the envelope, folded in from
/// what the retired `place_watch --event` wrapper did. Silent for anything but a theory file
/// (a `.rs` carrying `ops {` stanzas); otherwise re-derives the placement from the file's
/// TEXT (no compilation), diffs it against the previous placement kept in a per-file state
/// slug under `<project>/target/probe-hook`, and narrates a move in the monotone vocabulary
/// (seeded / joined / BRIDGED). Fail-open throughout: any unreadable/unwritable state is
/// silence (`.ok()?`), never a broken edit loop.
///
/// Capability: Effectful — reads and writes the ticker state under `project_dir/target`.
fn shape_voice(project_dir: &Path, rel: &str, source: &str) -> Option<String> {
    if !rel.ends_with(".rs") || !source.contains("ops {") {
        return None;
    }
    let state_dir = project_dir.join("target/probe-hook");
    let slug: String = rel
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let state = state_dir.join(format!("{slug}.sigs"));
    // the ticker keys on a `&'static` name; the process is short-lived, so a leak per hook
    // invocation is the price of the borrow (the same trade the example made).
    let name: &'static str = Box::leak(rel.to_string().into_boxed_str());
    let line = match std::fs::read_to_string(&state) {
        Ok(stored) => Ticker::resume(name, &stored).hook_line(name, source),
        // first sight: capture the baseline; a multi-component file announces itself once.
        Err(_) => Ticker::new().hook_line(name, source),
    };
    std::fs::create_dir_all(&state_dir).ok()?;
    std::fs::write(&state, Ticker::store(source).ok()?).ok()?;
    line
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
    /// wrapper), while an ordinary `.rs` with no `ops {` stanza stays silent. The version
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
        assert!(
            voice.contains("net-disjoint") || voice.contains("modules"),
            "{voice}"
        );
        assert!(voice.contains(&format!("probe-hook {VERSION}")), "{voice}");
        // a plain .rs (no `ops {`) is silence — the ticker speaks only for theories.
        let plain = root.join("src/plain.rs");
        std::fs::write(&plain, "pub struct X;\n").unwrap();
        assert_eq!(respond(&event(&plain), &root), None);
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
