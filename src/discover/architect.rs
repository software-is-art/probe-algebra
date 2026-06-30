//! architect — the cohesion suggestion as an EDITOR DEV TOOL: LSP-shaped diagnostics and an
//! auto-applicable code action that scaffolds the split.
//!
//! This is the core a rust-analyzer / VS-Code extension calls. It runs the cohesion analysis over a
//! registry of the crate's theories, and for every decomposable module emits a `Diagnostic` (a
//! `Hint` at the `theory!` site — "this module is secretly N modules") and a `CodeAction` whose edit
//! CREATES the scaffolded sub-module files (`discover::scaffold`). `render_lsp` serialises both to
//! the JSON an editor consumes; `apply` writes the files. The full LSP server (the stdio protocol
//! loop) is a thin shim over this — the interesting part, the architecture-to-quick-fix pipeline,
//! lives here and is mutation-tested like everything else.

use std::path::Path;

use super::scaffold::{scaffold, Scaffold};
use crate::discover::arithmetic::Arithmetic;
use crate::discover::date::Calendar;
use crate::discover::engine::Theory;
use crate::discover::router::Router;

/// LSP diagnostic severity (the wire integers).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// A diagnostic at a source location (LSP `Diagnostic`).
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub severity: Severity,
    pub message: String,
}

/// One file the code action creates (an LSP `CreateFile` + insert).
pub struct FileEdit {
    pub path: String,
    pub contents: String,
}

/// An auto-applicable fix (LSP `CodeAction`, kind `refactor.extract`).
pub struct CodeAction {
    pub title: String,
    pub edits: Vec<FileEdit>,
}

/// A diagnostic together with its fix.
pub struct Finding {
    pub name: String,
    pub diagnostic: Diagnostic,
    pub action: CodeAction,
}

struct Entry {
    name: &'static str,
    /// The source file the theory is declared in (repo-relative).
    file: &'static str,
    /// The directory new sub-modules are written under (repo-relative).
    out_dir: &'static str,
    scaffold: fn() -> Option<Scaffold>,
}

/// The theories the architect analyses (the crate's real discovery domains).
fn registry() -> Vec<Entry> {
    fn s<T: Theory>() -> Option<Scaffold> {
        scaffold::<T>()
    }
    vec![
        Entry {
            name: Arithmetic::name(),
            file: "src/discover/arithmetic.rs",
            out_dir: "src/discover/arithmetic",
            scaffold: s::<Arithmetic>,
        },
        Entry {
            name: Router::name(),
            file: "src/discover/router.rs",
            out_dir: "src/discover/router",
            scaffold: s::<Router>,
        },
        Entry {
            name: Calendar::name(),
            file: "src/discover/date.rs",
            out_dir: "src/discover/date",
            scaffold: s::<Calendar>,
        },
    ]
}

/// The 1-based line of the `theory!` invocation in `file` (so the diagnostic points at the
/// declaration), or 1 if it can't be found.
fn theory_line(file: &str) -> usize {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
    std::fs::read_to_string(path)
        .ok()
        .and_then(|src| {
            src.lines()
                .position(|l| l.contains("theory!"))
                .map(|i| i + 1)
        })
        .unwrap_or(1)
}

/// Analyse the registry and produce a finding (diagnostic + scaffold code action) for every module
/// whose discovered algebra DECOMPOSES — the cohesive ones produce nothing, exactly as an editor
/// would surface a hint only where there is something to act on.
pub fn analyze() -> Vec<Finding> {
    let mut findings = Vec::new();
    for e in registry() {
        let Some(sc) = (e.scaffold)() else {
            continue;
        };
        let edits = sc
            .modules
            .iter()
            .enumerate()
            .map(|(i, m)| FileEdit {
                path: format!("{}/module{i}.rs", e.out_dir),
                contents: m.source.clone(),
            })
            .collect();
        let seams: Vec<&str> = sc
            .seams
            .iter()
            .map(|s| match s.kind {
                super::cohesion::SeamKind::Transport => "transport",
                super::cohesion::SeamKind::Transform => "transform",
            })
            .collect();
        findings.push(Finding {
            name: e.name.to_string(),
            diagnostic: Diagnostic {
                file: e.file.to_string(),
                line: theory_line(e.file),
                severity: Severity::Hint,
                message: format!(
                    "`{}` is secretly {} modules — its algebra decomposes (seam: {}). Consider splitting.",
                    e.name,
                    sc.modules.len(),
                    seams.join(", ")
                ),
            },
            action: CodeAction {
                title: format!("Split `{}` into {} modules", e.name, sc.modules.len()),
                edits,
            },
        });
    }
    findings
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

/// Serialise the findings to the LSP JSON an editor consumes: a `diagnostics` array and a
/// `codeActions` array (each action a `refactor.extract` whose `documentChanges` create the files).
pub fn render_lsp(findings: &[Finding]) -> String {
    let diags: Vec<String> = findings
        .iter()
        .map(|f| {
            let d = &f.diagnostic;
            let line = d.line.saturating_sub(1);
            format!(
                "{{\"uri\":\"{}\",\"range\":{{\"start\":{{\"line\":{line},\"character\":0}},\
                 \"end\":{{\"line\":{line},\"character\":0}}}},\"severity\":{},\
                 \"source\":\"architect\",\"message\":\"{}\"}}",
                esc(&d.file),
                d.severity as u8,
                esc(&d.message)
            )
        })
        .collect();
    let actions: Vec<String> = findings
        .iter()
        .map(|f| {
            let changes: Vec<String> = f
                .action
                .edits
                .iter()
                .map(|e| {
                    format!(
                        "{{\"kind\":\"create\",\"uri\":\"{}\",\"text\":\"{}\"}}",
                        esc(&e.path),
                        esc(&e.contents)
                    )
                })
                .collect();
            format!(
                "{{\"title\":\"{}\",\"kind\":\"refactor.extract\",\
                 \"edit\":{{\"documentChanges\":[{}]}}}}",
                esc(&f.action.title),
                changes.join(",")
            )
        })
        .collect();
    format!(
        "{{\"diagnostics\":[{}],\"codeActions\":[{}]}}",
        diags.join(","),
        actions.join(",")
    )
}

/// Apply a code action: write every created file under `root`. Returns the paths written. This is
/// the "auto-apply" — the scaffolded skeletons land on disk; the developer (or an agent) then moves
/// the operator functions in and names the modules.
pub fn apply(action: &CodeAction, root: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut written = Vec::new();
    for e in &action.edits {
        let path = root.join(&e.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &e.contents)?;
        written.push(path);
    }
    Ok(written)
}

// ----- the architect's OWN algebra: a report is a join-semilattice ----------
//
// Dogfood: the tool that surfaces each module's algebra is itself written in the algebra format. The
// architect's value object is a REPORT — the set of modules a run flagged for splitting. Reports
// compose by UNION (`merge`), with the empty report as identity: a commutative, idempotent monoid
// (a join-semilattice), discovered by running it like any other domain. Merging real runs (several
// crates, several passes) is exactly this operator, not a contrivance.

use std::collections::BTreeSet;

/// The architect's value object: the set of module names a run flagged.
#[derive(Clone)]
pub struct Report(BTreeSet<String>);

impl Report {
    /// The flagged module names, sorted.
    pub fn flagged(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}

/// The architect's output AS a value object — the set of flagged modules from one analysis run.
pub fn report() -> Report {
    Report(analyze().into_iter().map(|f| f.name).collect())
}

/// The report theory's single sort.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RSort {
    Report,
}

fn empty_report(_: &[Report]) -> Option<Report> {
    Some(Report(BTreeSet::new()))
}
fn merge_reports(v: &[Report]) -> Option<Report> {
    let mut merged = v[0].0.clone();
    merged.extend(v[1].0.iter().cloned());
    Some(Report(merged))
}
fn sample_reports() -> Vec<Report> {
    let r = |names: &[&str]| Report(names.iter().map(|s| s.to_string()).collect());
    vec![r(&[]), r(&["x"]), r(&["y"]), r(&["x", "y"]), r(&["z"])]
}

/// The architect's report monoid, as a `Theory` — so the tool's own algebra is discovered, frozen,
/// and ratified exactly like the modules it analyses.
pub struct Reports;
crate::theory! {
    Reports : "architect report", Value = Report, Obs = Vec<String>, Sort = RSort,
    sort_of = |_: &Report| RSort::Report,
    observe = |r: &Report| r.0.iter().cloned().collect(),
    vars { RSort::Report => &["a", "b", "c"], }
    inhabit { RSort::Report => sample_reports(), }
    ops {
        Nullary "Empty" "empty" () -> RSort::Report = empty_report;
        Infix   "Merge" "merge" (RSort::Report, RSort::Report) -> RSort::Report = merge_reports;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::cohesion::cohesion;
    use crate::discover::engine::Engine;

    /// The architect surfaces a finding for every DECOMPOSABLE module and none for the cohesive
    /// ones — exactly the actionable hints an editor would show. Date and arithmetic decompose;
    /// router is cohesive, so it is silent.
    #[test]
    fn it_surfaces_findings_only_for_decomposable_modules() {
        let f = analyze();
        let named: Vec<&str> = f.iter().map(|x| x.diagnostic.message.as_str()).collect();
        assert!(named
            .iter()
            .any(|m| m.contains("date calculus") && m.contains("transform")));
        assert!(named
            .iter()
            .any(|m| m.contains("interpreter arithmetic") && m.contains("transport")));
        assert!(
            !named.iter().any(|m| m.contains("router")),
            "router is cohesive: {named:?}"
        );
        // every finding is a Hint with a create-the-modules code action.
        for finding in &f {
            assert_eq!(finding.diagnostic.severity, Severity::Hint);
            assert!(finding.action.edits.len() >= 2, "splits into ≥2 files");
            assert!(finding.action.title.starts_with("Split `"));
        }
    }

    /// The diagnostic points at the `theory!` declaration line, not the top of the file.
    #[test]
    fn the_diagnostic_points_at_the_theory_declaration() {
        let date = analyze()
            .into_iter()
            .find(|f| f.diagnostic.message.contains("date calculus"))
            .expect("date finding");
        assert_eq!(date.diagnostic.file, "src/discover/date.rs");
        // the exact line of `crate::theory! {` in date.rs — pins the `i + 1` line math.
        assert_eq!(
            date.diagnostic.line, 56,
            "should locate the theory! declaration"
        );
    }

    /// `esc` escapes the JSON specials (so the scaffolded source — full of quotes and newlines —
    /// rides safely inside the LSP payload). Pins every escape arm.
    #[test]
    fn esc_escapes_json_specials() {
        assert_eq!(esc("a\"b\\c\nd\te"), "a\\\"b\\\\c\\nd\\te");
        assert_eq!(esc("x\ry"), "xy", "carriage returns are dropped");
    }

    /// `render_lsp` emits the LSP payloads an editor consumes — a Hint (severity 4) diagnostic and a
    /// `refactor.extract` code action whose documentChange CREATES the scaffolded module file.
    #[test]
    fn it_renders_lsp_json() {
        let json = render_lsp(&analyze());
        assert!(json.contains("\"diagnostics\":["));
        assert!(json.contains("\"severity\":4"));
        assert!(json.contains("\"source\":\"architect\""));
        assert!(json.contains("\"kind\":\"refactor.extract\""));
        assert!(json.contains("\"kind\":\"create\""));
        // the scaffolded source rides inside the edit (escaped).
        assert!(json.contains("crate::theory! {"));
    }

    /// DOGFOOD: the architect's own domain is a discovered algebra. Its report is a join-semilattice
    /// — `merge` is commutative, associative, and idempotent, with the empty report as identity — and
    /// the engine finds exactly those laws by running it. And `cohesion` on the tool's own algebra
    /// reports COHESIVE: the architect does not itself want splitting.
    #[test]
    fn the_architect_is_itself_a_discovered_algebra() {
        let proses: Vec<String> = Engine::<Reports>::new()
            .discover()
            .laws
            .iter()
            .map(|l| l.prose.clone())
            .collect();
        for shape in [
            "Merge gives the same result in either order.", // commutative
            "Merge of a value with itself gives that value.", // idempotent
            "Merge with empty leaves a value unchanged.",   // identity
        ] {
            assert!(
                proses.iter().any(|p| p == shape),
                "missing `{shape}` in {proses:?}"
            );
        }
        assert!(
            proses.iter().any(|p| p.contains("grouping")),
            "associativity"
        );
        assert!(
            cohesion::<Reports>().is_cohesive(),
            "the architect itself is one algebra"
        );
        // and `report()` yields the architect's output as that value object.
        assert!(report().flagged().iter().any(|n| n == "date calculus"));
    }

    /// `apply` writes the scaffolded files to disk — the auto-apply. Uses the scratch dir.
    #[test]
    fn apply_writes_the_split_files() {
        let date = analyze()
            .into_iter()
            .find(|f| f.diagnostic.message.contains("date calculus"))
            .expect("date finding");
        let dir = std::env::temp_dir().join(format!("architect-test-{}", std::process::id()));
        let written = apply(&date.action, &dir).expect("write");
        assert_eq!(written.len(), date.action.edits.len());
        let body = std::fs::read_to_string(&written[0]).expect("read back");
        assert!(body.contains("crate::theory! {"));
        std::fs::remove_dir_all(&dir).ok();
    }
}
