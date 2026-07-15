//! bundle — the continuation verbs as a CLI: THE INTERFACE TO THE CODE (the peak the
//! operator named: agents should not write text into open files — and agents already
//! drive everything else through a CLI, so the CLI becomes the interaction mode).
//!
//! Every verb wraps the library form in `discover::bundle` / `discover::lift` — the CLI
//! adds only the file I/O and the exit code, so the judged transaction IS the interface:
//! a refused verb writes nothing and says why, a successful verb leaves the module
//! canonically placed. No open files, no partial states.
//!
//!     cargo run --example bundle -- add <module.rs> <snippet.rs | ->
//!     cargo run --example bundle -- edit <module.rs> <item-name> <replacement.rs | ->
//!     cargo run --example bundle -- declare <module.rs> "<shape(op, ...)>"
//!     cargo run --example bundle -- place <module.rs>
//!     cargo run --example bundle -- check <module.rs>
//!     cargo run --example bundle -- lift <module.rs> <theory-name> [declaration ...]
//!
//! `add` grows a module additively (a missing module file starts empty — birth is the
//! degenerate case of continuation); `edit` replaces one item's body while its signature
//! HOLDS (an interface change is not an edit — refused); `declare` adds an expectation to
//! its `#[algebra]`; `place` rewrites a module into canonical placed order (the one verb
//! that can move code, still never changing a byte inside any item); `check` judges
//! canonicality without writing; `lift` prints the generated zero-annotation
//! `impl Liftable` — with the declarations baked in — for the caller to commit and
//! drift-gate.
//!
//! THE JOURNAL (stage 2 of the zero-file-patching aim): every verb that changes a module
//! appends one line to `bundle.journal` beside the nearest `Cargo.toml` — the verbs
//! record themselves, so the change record is derived, never narrated. Entries carry
//! names, not payloads (order is the only clock); replayability is stage 3's business.

use std::io::Read;
use std::process::ExitCode;

use boundary_spec::discover::bundle::Bundle;
use boundary_spec::discover::lift::AutoLift;
use boundary_spec::discover::trace::Trace;

fn main() -> ExitCode {
    // a closed pipe (`bundle show m.rs | head`) is the reader's satisfaction, not a
    // failure — perception output tolerates it instead of panicking mid-print.
    use std::io::Write;
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(message) => {
            let _ = writeln!(std::io::stdout(), "{message}");
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            let _ = writeln!(std::io::stderr(), "{refusal}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<String, String> {
    // argv is bound by the DECLARATION (discover::cli): parse finds the verb and
    // binds its slots or refuses with the usage text — the same image the first lock
    // pins — so the hand-written arity match this function held until brick 2 is
    // gone. The judgments below consume the bound invocation only; the declaration
    // is the single author of what argv can say.
    let cli = boundary_spec::discover::cli::CliSpec::bundle();
    let invocation = cli.parse(args)?;
    let one = |slot: usize| invocation.values[slot][0].as_str();
    match invocation.verb.name {
        "owes" => {
            // perception: the derived to-do list — green is a fact about a SUPPORT
            // key (the fingerprint of what each gate declares it reads), so what this
            // tree still OWES is exactly the every-change gates without verdicts at
            // their own keys. An edit outside a gate's support cannot owe it.
            // Read-only; journals nothing.
            use boundary_spec::discover::gates::Support;
            use boundary_spec::discover::verdict::VerdictStore;
            let root = std::path::Path::new(".");
            let judged = VerdictStore::support_hash(root, &Support::Judged)?;
            let store = VerdictStore::beside(root);
            let standing: Vec<String> = store
                .owed(root)?
                .into_iter()
                .map(|(name, held)| {
                    if held {
                        format!("held: {name}")
                    } else {
                        format!("OWED: {name}")
                    }
                })
                .collect();
            Ok(format!("judged tree {judged}\n{}", standing.join("\n")))
        }
        "gates" => {
            // THE GATES BECOME LOCKS: run every owed every-change gate from the same
            // declaration CI executes, and record green as a verdict KEYED to the
            // SUPPORT it judged — the projection of the tree the gate declares it
            // reads — a held verdict is a content match (the same computation), never
            // a warm memory. Nothing records unless the projection the gates judged
            // is the one that exists afterwards (autofix ratifications move the key).
            use boundary_spec::discover::gates::{Cadence, GateRegistry, Support};
            use boundary_spec::discover::verdict::VerdictStore;
            let root = std::path::Path::new(".");
            let store = VerdictStore::beside(root);
            let before = VerdictStore::support_hash(root, &Support::Judged)?;
            let mut judged = Vec::new();
            for gate in GateRegistry::declared()
                .into_iter()
                .filter(|gate| matches!(gate.cadence, Cadence::EveryChange))
            {
                let key = VerdictStore::support_hash(root, &gate.support)?;
                if store.held(gate.name, &key) {
                    judged.push(format!("held: {} — verdict at its support", gate.name));
                    continue;
                }
                println!("judging: {} — {}", gate.name, gate.command.join(" "));
                let status = std::process::Command::new(gate.command[0])
                    .args(&gate.command[1..])
                    .status()
                    .map_err(|e| format!("bundle gates: `{}` cannot run ({e})", gate.name))?;
                if !status.success() {
                    return Err(format!(
                        "bundle gates: `{}` is RED — nothing recorded; a red tree owes \
                         every re-judgment",
                        gate.name
                    ));
                }
                judged.push(format!("green: {}", gate.name));
            }
            let after = VerdictStore::support_hash(root, &Support::Judged)?;
            if before != after {
                return Err(format!(
                    "bundle gates: the gates MOVED the tree (autofix ratifications \
                     landed) — the run judged a tree that no longer exists; re-run to \
                     judge the settled one\n{}",
                    judged.join("\n")
                ));
            }
            for gate in GateRegistry::declared()
                .into_iter()
                .filter(|gate| matches!(gate.cadence, Cadence::EveryChange))
            {
                let key = VerdictStore::support_hash(root, &gate.support)?;
                if !store.held(gate.name, &key) {
                    store.record(gate.name, &key)?;
                }
            }
            Ok(format!(
                "every-change gates green at judged tree {after}\n{}",
                judged.join("\n")
            ))
        }
        "pin" => {
            // THE PINNED SUIT (the frozen-arm field report's fix): install THIS
            // RUNNING BINARY at .suit/bundle, so the verbs stop rebuilding behind the
            // gate of the tree they operate on — a mid-transaction tree can no longer
            // block the verb that would heal it. Provenance rides beside the binary.
            let me = std::env::current_exe()
                .map_err(|e| format!("bundle pin: cannot find the running binary ({e})"))?;
            std::fs::create_dir_all(".suit")
                .map_err(|e| format!("bundle pin: cannot create .suit ({e})"))?;
            std::fs::copy(&me, ".suit/bundle")
                .map_err(|e| format!("bundle pin: cannot install ({e})"))?;
            std::fs::write(
                ".suit/provenance",
                format!(
                    "bundle {} — pinned from {} (toolchain {})\n",
                    env!("CARGO_PKG_VERSION"),
                    me.display(),
                    boundary_spec::discover::gates::TOOLCHAIN,
                ),
            )
            .map_err(|e| format!("bundle pin: provenance unwritten ({e})"))?;
            Ok(
                "pinned: .suit/bundle — the suit no longer rebuilds behind the tree's gate"
                    .to_string(),
            )
        }
        verb => {
            let subject = one(0);
            let module = std::fs::read_to_string(subject).unwrap_or_default();
            match verb {
                "add" => {
                    let snippet = read_payload("add", one(1))?;
                    let grown = Bundle::add(&module, &snippet)?;
                    let named: Vec<String> = syn::parse_file(&snippet)
                        .ok()
                        .map(|f| {
                            f.items
                                .iter()
                                .filter_map(|i| {
                                    use syn::Item::*;
                                    match i {
                                        Fn(f) => Some(format!("fn {}", f.sig.ident)),
                                        Struct(s) => Some(format!("struct {}", s.ident)),
                                        Enum(e) => Some(format!("enum {}", e.ident)),
                                        Trait(t) => Some(format!("trait {}", t.ident)),
                                        Mod(m) => Some(format!("mod {}", m.ident)),
                                        _ => None,
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    // stage 3: the payload rides the record — stash first, so a store
                    // refusal writes nothing at all.
                    use boundary_spec::discover::store::PayloadStore;
                    let store = PayloadStore::beside(&nearest_crate_root(subject));
                    let address = store.stash(&snippet)?;
                    let detail = format!("{} @{address}", named.join(", "));
                    commit(subject, &grown, "add", &detail)?;
                    Ok(format!("added into {subject} — canonically placed"))
                }
                "edit" => {
                    let item_name = one(1);
                    let replacement = read_payload("edit", one(2))?;
                    let edited = Bundle::edit(&module, item_name, &replacement)?;
                    use boundary_spec::discover::store::PayloadStore;
                    let store = PayloadStore::beside(&nearest_crate_root(subject));
                    let address = store.stash(&replacement)?;
                    commit(subject, &edited, "edit", &format!("{item_name} @{address}"))?;
                    Ok(format!(
                        "edited `{item_name}` in {subject} — signature held"
                    ))
                }
                "declare" => {
                    let declaration = one(1);
                    let declared = Bundle::declare(&module, declaration)?;
                    commit(subject, &declared, "declare", declaration)?;
                    Ok(format!("declared `{declaration}` in {subject}"))
                }
                "place" => {
                    let placed = Bundle::parse(&module)?.render();
                    if placed == module {
                        Ok(format!("{subject} is already canonically placed"))
                    } else {
                        commit(subject, &placed, "place", "re-placed canonically")?;
                        Ok(format!("{subject} re-placed canonically"))
                    }
                }
                "check" => {
                    if Bundle::parse(&module)?.is_canonical() {
                        Ok(format!("{subject}: canonical"))
                    } else {
                        Err(format!(
                            "{subject}: NOT canonical — `bundle place` would move it"
                        ))
                    }
                }
                "show" => match invocation.values[1].first() {
                    // perception: with no item named, the module's table of contents —
                    // addresses, kinds, and the exact signatures `edit` will hold; with
                    // one, the item's verbatim segment — the same bytes `edit` holds —
                    // so `bundle show m.rs x > payload` starts an edit cycle that never
                    // opens a file. Read-only; journals nothing.
                    None => Bundle::inventory(&module).map(|toc| toc.trim_end().to_string()),
                    Some(item_name) => Bundle::show(&module, item_name),
                },
                "collect" => {
                    let root = nearest_crate_root(subject);
                    let register = root.join("downstream/reliances.register");
                    let register = register.exists().then_some(register);
                    match invocation.values[1].first() {
                        None => {
                            // the MARK census: read-only, journals nothing — a derived
                            // fact list.
                            let marked = Bundle::collectable(
                                &module,
                                &root.join("spec"),
                                register.as_deref(),
                            )?;
                            if marked.is_empty() {
                                Ok(format!(
                                    "{subject}: nothing collectable — every item is reached by a root"
                                ))
                            } else {
                                Ok(marked
                                    .iter()
                                    .map(|(name, evidence)| {
                                        format!("collectable: {name} — {evidence}")
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n"))
                            }
                        }
                        Some(name) => {
                            // the SWEEP: one judged transaction, journaled; refuses the
                            // unmarked.
                            let swept = Bundle::collect(
                                &module,
                                name,
                                &root.join("spec"),
                                register.as_deref(),
                            )?;
                            commit(subject, &swept, "collect", name)?;
                            Ok(format!(
                                "collected `{name}` from {subject} — the journal remembers it"
                            ))
                        }
                    }
                }
                "squash" => {
                    // the first slot names the JOURNAL, not a module: compaction to
                    // law-normal form, licensed line by line by the verb algebra's
                    // frozen laws — a collapse the lock does not state never happens.
                    // The one verb that rewrites the record, warranted by the record's
                    // own algebra; it journals nothing (its record IS the journal's
                    // diff).
                    if module.is_empty() {
                        Err(format!(
                            "bundle squash: no journal at {subject} — nothing to compact"
                        ))
                    } else {
                        use boundary_spec::discover::squash::SquashRules;
                        let spec = boundary_spec::discover::Spec::of::<
                            boundary_spec::discover::verbs::state::VerbAlgebra,
                        >();
                        let compacted = SquashRules::from_spec(&spec).compact(&module)?;
                        if compacted == module {
                            Ok(format!(
                                "{subject}: already law-normal — no licensed squash remains"
                            ))
                        } else {
                            let (was, now) = (module.lines().count(), compacted.lines().count());
                            std::fs::write(subject, &compacted)
                                .map_err(|e| format!("bundle squash: cannot write: {e}"))?;
                            Ok(format!(
                                "{subject}: squashed {was} entries → {now} — every \
                                 collapse licensed by the frozen verb algebra"
                            ))
                        }
                    }
                }
                "replay" => {
                    // perception: the REPLAY DIFFERENTIAL — reconstruct each journaled
                    // file from the record plus the payload store and judge it against
                    // the tree. Read-only; journals nothing. Divergence is a FINDING,
                    // not a failure: the report is stage 3's progress bar
                    // (tree == replay(journal), measured file by file).
                    if module.is_empty() {
                        Err(format!(
                            "bundle replay: no journal at {subject} — nothing to replay"
                        ))
                    } else {
                        use boundary_spec::discover::store::{PayloadStore, Replay};
                        let store = PayloadStore::beside(&nearest_crate_root(subject));
                        Replay::differential(&module, &store)
                            .map(|replay| replay.render().trim_end().to_string())
                    }
                }
                "constrains" => {
                    // perception: read-only, journals nothing. The committed record is the
                    // nearest crate's spec directory and its downstream reliance register.
                    let root = nearest_crate_root(subject);
                    let register = root.join("downstream/reliances.register");
                    let register = register.exists().then_some(register);
                    Bundle::constrains(&module, one(1), &root.join("spec"), register.as_deref())
                        .map(|report| report.trim_end().to_string())
                }
                "trace" => {
                    // perception: the first argument slot names a THEORY, not a module —
                    // trace runs over compiled theories (the build/text boundary: eval
                    // needs the build). The roster is the theories this example links.
                    let term = one(1);
                    match subject {
                        "verbs" | "verb algebra" => {
                            Trace::of::<boundary_spec::discover::verbs::state::VerbAlgebra>(term)
                                .map(|t| t.render())
                        }
                        "router" => Trace::of::<boundary_spec::discover::router::Router>(term)
                            .map(|t| t.render()),
                        "fabric" => Trace::of::<boundary_spec::discover::fabric::Fabric>(term)
                            .map(|t| t.render()),
                        other => Err(format!(
                            "bundle trace: `{other}` is not in this binary's theory roster \
                             (verbs, router, fabric) — a consumer traces its own theories \
                             through discover::trace::Trace in its own suite"
                        )),
                    }
                }
                "lift" => {
                    let declarations: Vec<&str> =
                        invocation.values[2].iter().map(String::as_str).collect();
                    // the generated impl ends with its own newline; println adds the last
                    // one back, so a `> file` redirect captures the artifact byte-exact.
                    AutoLift::scan_module(&module, one(1), &declarations)
                        .map(|generated| generated.trim_end().to_string())
                }
                // a verb the declaration speaks but no judgment yet consumes —
                // declaration/dispatch drift, refused with the teaching text.
                _ => Err(cli.usage()),
            }
        }
    }
}

/// A verb payload: a file path, or `-` for stdin.
fn read_payload(verb: &str, source: &str) -> Result<String, String> {
    if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("bundle {verb}: stdin unreadable: {e}"))?;
        Ok(buf)
    } else {
        std::fs::read_to_string(source)
            .map_err(|e| format!("bundle {verb}: payload unreadable: {e}"))
    }
}

/// The write half of a judged transaction: the module lands AND the verb records itself —
/// one line appended to `bundle.journal` beside the nearest `Cargo.toml` (the crate is
/// the journal's scope) — AND the qualify census pair updates at the delta (the
/// maintained view; the build gate stays the oracle). The write happens first; a journal
/// or census failure is reported, never silently swallowed — a change the record missed
/// is exactly what the record exists to prevent.
fn commit(module_path: &str, content: &str, verb: &str, detail: &str) -> Result<(), String> {
    std::fs::write(module_path, content)
        .map_err(|e| format!("bundle {verb}: cannot write: {e}"))?;
    let root = nearest_crate_root(module_path);
    let journal = root.join("bundle.journal");
    let entry = Bundle::journal_entry(verb, module_path, detail);
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal)
        .and_then(|mut f| f.write_all(entry.as_bytes()))
        .map_err(|e| format!("bundle {verb}: wrote the module but NOT the journal ({e})"))?;
    maintain_census(&root, module_path, content, verb)
}

/// The nearest ancestor directory carrying a `Cargo.toml` — the journal's home. Falls
/// back to the module's own directory when no manifest is found (still a real record,
/// just unanchored).
fn nearest_crate_root(module_path: &str) -> std::path::PathBuf {
    let start = std::path::Path::new(module_path)
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let mut dir = start;
    loop {
        if dir.join("Cargo.toml").exists() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return start.to_path_buf(),
        }
    }
}

/// THE MAINTAINED VIEW at the verb (the item-relation program's brick 3): every
/// committing verb re-derives its own module's line in the crate's qualify census
/// pair — retract, re-judge, re-render through the census's own from-parts
/// renderers — so the drift gate that re-derives the pair wholesale on every build
/// (the ORACLE) cannot fire for a verb-carried change, and the `BLESS_*` ceremony
/// stops being owed at the edit. A crate without the pair owes nothing; a module
/// outside the census's `src/` walk moves nothing; the bless env names are read
/// from the committed headers, never configured.
fn maintain_census(
    root: &std::path::Path,
    module_path: &str,
    content: &str,
    verb: &str,
) -> Result<(), String> {
    let census_path = root.join("spec/qualify.spec");
    let reasons_path = root.join("spec/qualify-reasons.spec");
    if !census_path.exists() || !reasons_path.exists() {
        return Ok(());
    }
    let loc = std::path::Path::new(module_path)
        .strip_prefix(root)
        .unwrap_or(std::path::Path::new(module_path))
        .display()
        .to_string();
    if !loc.starts_with("src/") {
        return Ok(());
    }
    let env = |text: &str, fallback: &str| -> String {
        text.lines()
            .find_map(|l| {
                let (_, rest) = l.split_once("Regenerate with `")?;
                let (name, _) = rest.split_once("=1 cargo build")?;
                Some(name.to_string())
            })
            .unwrap_or_else(|| fallback.to_string())
    };
    let census = std::fs::read_to_string(&census_path)
        .map_err(|e| format!("bundle {verb}: census unreadable ({e})"))?;
    let reasons = std::fs::read_to_string(&reasons_path)
        .map_err(|e| format!("bundle {verb}: reasons census unreadable ({e})"))?;
    let (fresh_census, fresh_reasons) = boundary_enforce::maintain_qualify(
        &census,
        &reasons,
        &loc,
        content,
        &env(&census, "BLESS_QUALIFY"),
        &env(&reasons, "BLESS_REASONS"),
    )
    .map_err(|e| format!("bundle {verb}: wrote the module but the census did not follow ({e})"))?;
    if fresh_census != census {
        std::fs::write(&census_path, fresh_census)
            .map_err(|e| format!("bundle {verb}: census unwritable ({e})"))?;
    }
    if fresh_reasons != reasons {
        std::fs::write(&reasons_path, fresh_reasons)
            .map_err(|e| format!("bundle {verb}: reasons census unwritable ({e})"))?;
    }
    Ok(())
}
