//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
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
///
/// Capability: Effectful — reads the source file from disk (a world-read).
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

/// Escape a string to its RFC 8259 wire form: `"` and `\` and the three whitespace controls take
/// their short escapes; every OTHER C0 control (U+0000–U+001F, which raw JSON forbids outright)
/// takes the generic `\u00XX` form. Deliberately a PER-CHARACTER map — each char's wire form is
/// independent of its neighbours — so `esc` is a monoid homomorphism over concatenation, exactly
/// the law the `EscapeCodec` theory discovers below.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            // ESCAPED, not dropped: `esc` must be INVERTIBLE or the engine refuses the codec
            // round-trip below (a dropping escaper loses `\r` and `unesc(esc(s)) != s`).
            '\r' => out.push_str("\\r"),
            // the REMAINING C0 controls (`\x08`, `\x0c`, ...): RFC 8259 forbids every one of
            // them raw, so each becomes `\u00XX` — still a fixed per-character map.
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push(c),
        }
    }
    out
}

/// The inverse of `esc` — decode the wire-escaping back to the original. With `esc` it forms a
/// CODEC: `unesc(esc(s)) = s`, the round-trip the `EscapeCodec` theory discovers below.
fn unesc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                // exactly what `esc` emits for the remaining C0 controls: `\u` then four hex
                // digits. A malformed tail (not `esc` output) is kept literally — `unesc` is
                // total, and on `esc`'s image this arm is the exact inverse of `\u00XX`.
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        Some(decoded) if hex.len() == 4 => out.push(decoded),
                        _ => {
                            out.push_str("\\u");
                            out.push_str(&hex);
                        }
                    }
                }
                Some(other) => out.push(other),
                None => out.push('\\'),
            }
        } else {
            out.push(c);
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

/// `apply` is the architect's one EFFECTFUL edge — everything else here (the analysis, the codec)
/// is `Pure`. Declared, not hidden: the top of the capability lattice (`effectful ⊃ stateful ⊃
/// lossy ⊃ pure`). The `std::fs::write` beneath it is the leaf where the world is finally touched.
pub const APPLY_CAPABILITY: crate::boundary::Capability = crate::boundary::Capability::Effectful;

/// True iff `rel` stays UNDER a root when joined — the confinement bound `apply` declares. An
/// absolute path or a `..` component would escape it, so neither is confined.
fn confined(rel: &Path) -> bool {
    !rel.is_absolute()
        && !rel
            .components()
            .any(|c| c == std::path::Component::ParentDir)
}

/// Apply a code action: write every created file under `root`. Returns the paths written. This is
/// the "auto-apply" — the scaffolded skeletons land on disk; the developer (or an agent) then moves
/// the operator functions in and names the modules.
///
/// The effect is BOUNDED to `root`: an edit whose path escapes it (absolute, or via `..`) is
/// rejected rather than written, so the declared `Effectful`-confined-to-`root` capability is real,
/// not a vacuous claim.
///
/// Capability: Effectful — writes files to disk (confined to `root`; see `APPLY_CAPABILITY`).
pub fn apply(action: &CodeAction, root: &Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut written = Vec::new();
    for e in &action.edits {
        let rel = Path::new(&e.path);
        if !confined(rel) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("edit path `{}` escapes the root", e.path),
            ));
        }
        let path = root.join(rel);
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

// ----- the LSP serialization as an in-format TRANSFORM SEAM -----------------
//
// The wire-escaping that carries the scaffolded source (full of quotes and newlines) safely inside
// the JSON payload is modelled as a string theory over `concat`, `esc`, and `unesc`. Running it, the
// engine DISCOVERS the serializer's algebra:
//   - HOMOMORPHISM — `esc(a ++ b) = esc(a) ++ esc(b)` — so the payload assembles and escapes
//     piecewise, identically (exactly how `render_lsp` builds it from fragments).
//   - ROUND-TRIP — `unesc(esc(s)) = s` — so the wire form is a FAITHFUL CODEC (and this is what
//     forced `\r` to be escaped rather than dropped: the round-trip would not hold otherwise).
//
// What the algebra does NOT determine is the ENCODING — which character maps to which escape. That
// is a representation CONVENTION (conformance to the RFC 8259 JSON wire standard), not a law: many
// invertible homomorphisms exist (the identity is one), so the specific arms `'\t' => "\\t"` … and
// the `\u00XX` form for the remaining C0 controls are a LEAF the laws cannot derive. The in-format
// model thus locates the irreducible bit precisely: the codec STRUCTURE is discovered; the encoding
// is an oracle pinned by `esc_conforms_to_the_json_encoding`.

/// The escape codec's single sort.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ESort {
    Str,
}

fn cat(v: &[String]) -> Option<String> {
    Some(format!("{}{}", v[0], v[1]))
}
fn esc_op(v: &[String]) -> Option<String> {
    Some(esc(&v[0]))
}
fn unesc_op(v: &[String]) -> Option<String> {
    Some(unesc(&v[0]))
}
/// One inhabitant per short-form escape arm (the generic `\u00XX` C0 arm is pinned by the
/// `esc_conforms_to_the_json_encoding` oracle instead — the laws hold either way, so the grid
/// stays minimal), plus plain text AND already-escaped SEQUENCES (`\n` as the two
/// chars backslash-n, a trailing backslash). The sequences matter: without them the grid is too weak
/// and discovery over-fits — `unesc` looks like an involution and a homomorphism (it is neither);
/// the sequences refute both, leaving only the TRUE laws (esc homomorphism, the codec round-trip,
/// concat associativity). A non-invertible mutation still fails the round-trip on its escape arm.
fn escape_inhabitants() -> Vec<String> {
    ["", "\"", "\\", "\n", "\t", "\r", "ab", "\\n", "x\\y"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// The LSP serializer's escaping AS a theory — so the tool's wire format is a discovered algebra
/// (an invertible homomorphism), not a hand-rolled transform.
pub struct EscapeCodec;
crate::theory! {
    EscapeCodec : "lsp escape codec", Value = String, Obs = String, Sort = ESort,
    sort_of = |_: &String| ESort::Str,
    observe = |s: &String| s.clone(),
    vars { ESort::Str => &["a", "b", "c"], }
    inhabit { ESort::Str => escape_inhabitants(), }
    ops {
        Infix  "concat" "++"    (ESort::Str, ESort::Str) -> ESort::Str = cat;
        Prefix "esc"    "esc"   (ESort::Str) -> ESort::Str = esc_op;
        Prefix "unesc"  "unesc" (ESort::Str) -> ESort::Str = unesc_op;
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
            date.diagnostic.line, 58,
            "should locate the theory! declaration"
        );
    }

    /// DOGFOOD: the LSP wire-escaping is a DISCOVERED algebra, and its WHOLE spec is exactly this —
    /// the serializer's structure found by running `esc`/`unesc`/`concat`: `concat` is associative,
    /// `unesc` undoes `esc` (a faithful codec), and `esc` is a homomorphism over concatenation
    /// (exactly how `render_lsp` assembles the payload). Pinning the EXACT law set + consequence
    /// count kills the under-constrained mutants a loose `any()` check misses: a constant `concat`
    /// would spuriously become commutative (a 4th law), and a collapsed inhabitant grid would change
    /// the consequence count. (Note: the engine finds NO false `unesc` law — the escape-sequence
    /// inhabitants refute the involution/homomorphism a weaker grid would over-fit.)
    #[test]
    fn the_escape_codec_spec_is_exact() {
        let d = Engine::<EscapeCodec>::new().discover();
        let got: Vec<(String, String)> = d
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "With concat, the grouping of three values doesn't matter.",
                "((a ++ b) ++ c) = (a ++ (b ++ c))",
            ),
            (
                "unesc undoes esc — the round trip is the identity.",
                "unesc(esc(a)) = a",
            ),
            (
                "esc turns concat into concat.",
                "esc((a ++ b)) = (esc(a) ++ esc(b))",
            ),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, e)| (p.to_string(), e.to_string()))
            .collect();
        assert_eq!(got, expected, "the escape codec's discovered spec changed");
        assert!(
            d.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            d.uncovered_ops
        );
        assert_eq!(d.consequences, 39, "consequence count changed");
    }

    /// The codec STRUCTURE is discovered (above); the ENCODING is not a law — many invertible
    /// homomorphisms exist, so which character maps to which escape is a representation CONVENTION,
    /// conformance to the RFC 8259 JSON wire standard. This is the irreducible leaf the algebra
    /// cannot derive, pinned here as an oracle (and pinning every `esc` arm against deletion) —
    /// including the standard's blanket rule that EVERY C0 control (U+0000–U+001F), not just the
    /// five with short forms, must be escaped.
    #[test]
    fn esc_conforms_to_the_json_encoding() {
        assert_eq!(esc("a\"b\\c\nd\te\rf"), "a\\\"b\\\\c\\nd\\te\\rf");
        // and it round-trips through `unesc` — invertibility, including the once-dropped `\r`.
        assert_eq!(unesc(&esc("a\"b\\c\nd\te\rf")), "a\"b\\c\nd\te\rf");
        // the controls WITHOUT short forms take the generic `\u00XX` escape (the once-raw leak).
        assert_eq!(
            esc("\u{0}a\u{8}b\u{c}c\u{1f}"),
            "\\u0000a\\u0008b\\u000cc\\u001f"
        );
        // and EVERY C0 control leaves no raw control byte on the wire and round-trips exactly.
        for c in '\u{0}'..='\u{1f}' {
            let s = c.to_string();
            let wire = esc(&s);
            assert!(
                wire.chars().all(|w| w as u32 >= 0x20),
                "raw control {c:?} leaked into the wire form {wire:?}"
            );
            assert_eq!(unesc(&wire), s, "control {c:?} must round-trip");
        }
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

    /// `apply` writes the scaffolded files to disk — the auto-apply — and its effect is CONFINED to
    /// `root`: every written path lands under it (the declared `Effectful`-bounded-to-`root`
    /// capability, made real). Uses the scratch dir.
    #[test]
    fn apply_writes_the_split_files_confined_to_root() {
        let date = analyze()
            .into_iter()
            .find(|f| f.diagnostic.message.contains("date calculus"))
            .expect("date finding");
        let dir = std::env::temp_dir().join(format!("architect-test-{}", std::process::id()));
        let written = apply(&date.action, &dir).expect("write");
        assert_eq!(written.len(), date.action.edits.len());
        assert_eq!(APPLY_CAPABILITY, crate::boundary::Capability::Effectful);
        // CONFINEMENT: every write stayed under the root — the effect is bounded as declared.
        assert!(
            written.iter().all(|p| p.starts_with(&dir)),
            "a write escaped the root: {written:?}"
        );
        let body = std::fs::read_to_string(&written[0]).expect("read back");
        assert!(body.contains("crate::theory! {"));
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An edit whose path ESCAPES the root (absolute, or via `..`) is rejected, not written — so the
    /// confinement bound is enforced, not decorative. Pins the `confined` guard.
    #[test]
    fn apply_rejects_paths_that_escape_the_root() {
        let dir = std::env::temp_dir().join(format!("architect-escape-{}", std::process::id()));
        for bad in ["../escape.rs", "/etc/passwd"] {
            let action = CodeAction {
                title: "x".to_string(),
                edits: vec![FileEdit {
                    path: bad.to_string(),
                    contents: "x".to_string(),
                }],
            };
            assert!(
                apply(&action, &dir).is_err(),
                "escaping path `{bad}` should be rejected"
            );
        }
        // a plain relative path is accepted (the guard is not just rejecting everything).
        assert!(confined(Path::new("src/discover/date/module0.rs")));
        std::fs::remove_dir_all(&dir).ok();
    }
}
