//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! bridge — THEORY FROM EXPORTED DATA: a prover's finite operator tables become a live
//! `Theory`, and discovery runs as CONJECTURE SUPPLY and cross-check — never certification.
//!
//! An external prover (Agda, Lean, anything with a kernel) can emit its decidable
//! fragments as finite operator tables: a table IS an eval function. [`Export::parse`]
//! reads that emission (a small line format, every malformed line a NAMED refusal),
//! [`Export::install`] mounts it in one of a few compile-time slots, and
//! [`Bridged<SLOT>`] is a full `Theory` over it — the whole discovery apparatus (the
//! catalog, the spec lock, the algebra-mutation verdict) runs on the exported tables
//! with zero new engine machinery.
//!
//! The prover's own certificates ride along as `proved:` lines — the same declaration
//! vocabulary `expects { ... }` uses — and [`Triage`] reads the resulting distance as
//! three verdicts with three different weights:
//!
//! - AGREEMENT (proved upstream, grid could not refute): proves nothing new — the kernel's
//!   certificate already outranks a grid — but records that the export/bridge pipeline
//!   round-tripped;
//! - CONJECTURE (discovered here, no upstream certificate): a proof obligation, grid
//!   evidence only — the genesis meaning-hole move pointed at a prover: prove it or
//!   refute it upstream, then ratify it into `proved:`. Grid refutation kills a false
//!   conjecture before anyone burns a day proving it;
//! - DISAGREEMENT (proved upstream, grid refutes it): a defect SOMEWHERE in the
//!   export/bridge pipeline, with certainty — the one thing differential testing yields
//!   unconditionally, same polarity as delta-render's end gate. [`Triage::certify`]
//!   fails on it; it never renders as a row to ratify.
//!
//! Certainty's fine print: the bridged grid is the WHOLE exported carrier and judgment is
//! exhaustive (`grid_size` covers every assignment; v1 caps the export at 8 elements and
//! 8 operators to keep that true), so "discovery did not find a proved equation" means a
//! concrete refuting assignment exists over the carrier — not a sampling artifact. What
//! stays bounded is term depth and the catalog itself: absence of a CONJECTURE is never
//! evidence, and agreement never certifies.

use std::path::Path;
use std::sync::OnceLock;

use crate::discover::engine::{Fixity, Operator, Theory};
use crate::discover::expect::{Distance, Expectation, Expected};

/// How many exports can be mounted at once (compile-time slots — the price of keeping
/// `Operator.eval` a plain fn pointer, which is what the whole engine runs on).
pub const SLOTS: usize = 4;

/// The operator cap per export — eval fn pointers are minted per (slot, index) pair.
pub const MAX_OPS: usize = 8;

/// The element cap per export — keeps `grid_size` exhaustive over the whole carrier
/// (8³ = 512 assignments), so a bridged refutation is a fact, not a sample.
pub const MAX_ELEMENTS: usize = 8;

/// One exported operator: its token (name and symbol at once), arity, and the flattened
/// result table (row-major for arity 2: `table[i * n + j] = op(elem_i, elem_j)`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExportedOp {
    pub token: &'static str,
    pub arity: usize,
    pub table: Vec<u8>,
}

/// A prover's emission, parsed: the carrier, the operator tables, and the upstream
/// certificates (`proved:` lines, in the declaration vocabulary).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Export {
    pub name: &'static str,
    pub elements: Vec<String>,
    pub ops: Vec<ExportedOp>,
    pub proved: Vec<Expectation>,
}

impl Export {
    /// Parse an exported-table text. Line format (`#` comments and blank lines skipped):
    ///
    /// ```text
    /// theory: bridged-bool
    /// elements: false true
    /// op and/2: false false | true false
    /// op not/1: true false
    /// op false/0: false
    /// proved: commutative and
    /// ```
    ///
    /// Every malformed line is a NAMED refusal — the parser is a gate, and each
    /// diagnosis below is fire-drilled.
    pub fn parse(text: &str) -> Result<Export, String> {
        let mut name: Option<&'static str> = None;
        let mut elements: Vec<String> = Vec::new();
        let mut ops: Vec<ExportedOp> = Vec::new();
        let mut proved: Vec<Expectation> = Vec::new();

        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            let at = |msg: String| format!("line {}: {msg}", n + 1);
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix("theory:") {
                let t = rest.trim();
                if t.is_empty() {
                    return Err(at("`theory:` names nothing".into()));
                }
                name = Some(Box::leak(t.to_string().into_boxed_str()));
            } else if let Some(rest) = line.strip_prefix("elements:") {
                elements = rest.split_whitespace().map(str::to_string).collect();
                if elements.is_empty() {
                    return Err(at("`elements:` declares no carrier".into()));
                }
                if elements.len() > MAX_ELEMENTS {
                    return Err(at(format!(
                        "{} elements — v1 judges at most {MAX_ELEMENTS} exhaustively",
                        elements.len()
                    )));
                }
            } else if let Some(rest) = line.strip_prefix("op ") {
                let (head, body) = rest
                    .split_once(':')
                    .ok_or_else(|| at("an `op` line needs `<token>/<arity>: <table>`".into()))?;
                let (token, arity) = head
                    .trim()
                    .split_once('/')
                    .ok_or_else(|| at("an op head needs `<token>/<arity>`".into()))?;
                let token = token.trim();
                let arity: usize = arity
                    .trim()
                    .parse()
                    .map_err(|_| at(format!("arity {:?} is not a number", arity.trim())))?;
                if arity > 2 {
                    return Err(at(format!(
                        "`{token}` has arity {arity} — the export format carries 0, 1, and 2"
                    )));
                }
                if ops.iter().any(|o| o.token == token) {
                    return Err(at(format!("operator `{token}` exported twice")));
                }
                if ops.len() == MAX_OPS {
                    return Err(at(format!(
                        "more than {MAX_OPS} operators — v1's slot table stops there"
                    )));
                }
                if elements.is_empty() {
                    return Err(at(
                        "`op` before `elements:` — the carrier must come first".into()
                    ));
                }
                let index = |word: &str| -> Result<u8, String> {
                    elements
                        .iter()
                        .position(|e| e == word)
                        .map(|i| i as u8)
                        .ok_or_else(|| {
                            at(format!(
                                "`{word}` is not an exported element (op `{token}`)"
                            ))
                        })
                };
                let n_el = elements.len();
                let rows: Vec<&str> = body.split('|').collect();
                let expected_rows = if arity == 2 { n_el } else { 1 };
                if rows.len() != expected_rows {
                    return Err(at(format!(
                        "`{token}` has {} table row(s); arity {arity} over {n_el} elements \
                         needs {expected_rows}",
                        rows.len()
                    )));
                }
                let per_row = match arity {
                    0 => 1,
                    _ => n_el,
                };
                let mut table = Vec::new();
                for row in rows {
                    let cells: Vec<&str> = row.split_whitespace().collect();
                    if cells.len() != per_row {
                        return Err(at(format!(
                            "`{token}` has a row of {} result(s); arity {arity} over {n_el} \
                             elements needs {per_row} per row",
                            cells.len()
                        )));
                    }
                    for cell in cells {
                        table.push(index(cell)?);
                    }
                }
                ops.push(ExportedOp {
                    token: Box::leak(token.to_string().into_boxed_str()),
                    arity,
                    table,
                });
            } else if let Some(rest) = line.strip_prefix("proved:") {
                let mut words = rest.split_whitespace();
                let shape = words
                    .next()
                    .ok_or_else(|| at("`proved:` claims nothing".into()))?;
                let canonical = Expectation::canonical(shape).ok_or_else(|| {
                    at(format!(
                        "`{shape}` is not a ratified catalog shape. Declarable shapes: {}",
                        Expectation::vocabulary_keys().join(", ")
                    ))
                })?;
                let claim_ops: Vec<String> = words.map(str::to_string).collect();
                for op in &claim_ops {
                    if !ops.iter().any(|o| o.token == op) {
                        return Err(at(format!(
                            "`proved: {shape}` ranges over `{op}`, which is not an exported \
                             operator"
                        )));
                    }
                }
                proved.push(Expectation {
                    shape: canonical,
                    ops: claim_ops,
                });
            } else {
                return Err(at(format!("unreadable line {line:?}")));
            }
        }

        let name = name.ok_or("the export names no theory (`theory:` line missing)")?;
        if elements.is_empty() {
            return Err("the export declares no carrier (`elements:` line missing)".into());
        }
        if ops.is_empty() {
            return Err("the export carries no operator tables".into());
        }
        Ok(Export {
            name,
            elements,
            ops,
            proved,
        })
    }

    /// Mount this export in slot `SLOT`, making [`Bridged<SLOT>`] a live theory over it.
    /// Idempotent for an identical export (parallel tests re-install freely); a DIFFERENT
    /// export in an occupied slot is refused by name — a slot never silently retargets.
    pub fn install<const SLOT: usize>(self) -> Result<(), String> {
        let slot = &slots()[SLOT];
        if let Err(rejected) = slot.set(self) {
            let held = slot.get().expect("set failed, so the slot is occupied");
            if *held != rejected {
                return Err(format!(
                    "slot {SLOT} already holds `{}` — it cannot retarget to `{}`",
                    held.name, rejected.name
                ));
            }
        }
        Ok(())
    }
}

fn slots() -> &'static [OnceLock<Export>; SLOTS] {
    static SLOTS_STORE: [OnceLock<Export>; SLOTS] = [const { OnceLock::new() }; SLOTS];
    &SLOTS_STORE
}

fn export<const SLOT: usize>() -> &'static Export {
    slots()[SLOT]
        .get()
        .expect("no export installed in this slot — call Export::install first")
}

/// A bridged value: an index into the export's carrier.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct El(pub u8);

/// The one sort (v1 bridges single-sorted fragments; a multi-sorted export is future
/// format, not a silent extension).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct One;

fn eval<const SLOT: usize, const OP: usize>(v: &[El]) -> Option<El> {
    let e = export::<SLOT>();
    let op = &e.ops[OP];
    let n = e.elements.len();
    let i = match op.arity {
        0 => 0,
        1 => v[0].0 as usize,
        _ => v[0].0 as usize * n + v[1].0 as usize,
    };
    Some(El(op.table[i]))
}

/// The bridged theory over the export mounted in slot `SLOT` — a full citizen of the
/// discovery apparatus: `Spec::of`, `MutationReport::of`, `Distance::of` all run on it.
pub struct Bridged<const SLOT: usize>;

impl<const SLOT: usize> Theory for Bridged<SLOT> {
    type Sort = One;
    type Value = El;
    type Obs = u8;

    fn name() -> &'static str {
        export::<SLOT>().name
    }
    fn operators() -> Vec<Operator<Self>> {
        export::<SLOT>()
            .ops
            .iter()
            .enumerate()
            .map(|(i, op)| Operator {
                name: op.token,
                symbol: op.token,
                fixity: match op.arity {
                    0 => Fixity::Nullary,
                    1 => Fixity::Prefix,
                    _ => Fixity::Infix,
                },
                inputs: vec![One; op.arity],
                output: One,
                eval: match i {
                    0 => eval::<SLOT, 0>,
                    1 => eval::<SLOT, 1>,
                    2 => eval::<SLOT, 2>,
                    3 => eval::<SLOT, 3>,
                    4 => eval::<SLOT, 4>,
                    5 => eval::<SLOT, 5>,
                    6 => eval::<SLOT, 6>,
                    _ => eval::<SLOT, 7>,
                },
            })
            .collect()
    }
    fn inhabitants(_: One) -> Vec<El> {
        (0..export::<SLOT>().elements.len() as u8).map(El).collect()
    }
    fn sort_of(_: &El) -> One {
        One
    }
    fn observe(v: &El) -> u8 {
        v.0
    }
    fn sort_vars(_: One) -> &'static [&'static str] {
        &["x", "y", "z"]
    }
    fn grid_size() -> usize {
        512 // = MAX_ELEMENTS³: the whole carrier, judged exhaustively.
    }
}

impl<const SLOT: usize> Expected for Bridged<SLOT> {
    fn expectations() -> Vec<Expectation> {
        export::<SLOT>().proved.clone()
    }
}

/// The bridge's verdict over one export: the distance report re-read with the prover's
/// epistemics — agreements cross-check, conjectures obligate, disagreements CONVICT.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Triage {
    pub theory: &'static str,
    /// Proved upstream AND discovered here — the round-trip cross-check.
    pub agreements: Vec<String>,
    /// Proved upstream, refuted over the exhaustive carrier — a defect in the
    /// export/bridge pipeline, with certainty. Non-empty fails [`Triage::certify`].
    pub disagreements: Vec<String>,
    /// Discovered here, no upstream certificate — proof obligations, grid evidence only.
    pub conjectures: Vec<String>,
}

impl Triage {
    /// Triage a bridged theory: run the same declared-vs-discovered distance every
    /// theory gets, then read it in the prover's terms.
    pub fn of<T: Theory + Expected>() -> Triage {
        let d = Distance::of::<T>();
        let declared = T::expectations();
        Triage {
            theory: d.theory,
            agreements: declared
                .iter()
                .filter(|e| !d.missing.contains(e))
                .map(Expectation::render)
                .collect(),
            disagreements: d.missing.iter().map(Expectation::render).collect(),
            conjectures: d.surprises.iter().map(Expectation::render).collect(),
        }
    }

    /// The disagreement detector: `Err` names every proved-but-refuted law. Agreement
    /// proves nothing new; disagreement is a defect somewhere with certainty — so it is
    /// a gate, never a row to ratify.
    pub fn certify(&self) -> Result<(), String> {
        if self.disagreements.is_empty() {
            return Ok(());
        }
        Err(format!(
            "{}: the exhaustive grid REFUTES upstream-proved law(s): {} — a defect in the \
             export/bridge pipeline (wrong table, wrong element order, wrong statement), \
             with certainty. Fix the export; never ratify a disagreement.",
            self.theory,
            self.disagreements.join(", ")
        ))
    }

    /// The triage as deterministic text — `spec/<theory>.obligations.spec`'s content.
    /// Disagreements never render: they fail [`Triage::certify`] instead.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# theory-bridge triage: `{}` — operator tables exported by an external prover, \
             judged\n\
             # by discovery as CONJECTURE SUPPLY and cross-check, never certification. \
             Regenerate via\n\
             # `cargo run --example freeze_spec` and ratify the diff.\n\
             #\n\
             # - an AGREEMENT proves nothing new (the kernel's certificate outranks a grid); \
             it records\n\
             #   that the export/bridge pipeline round-tripped.\n\
             # - a CONJECTURE is a discovered law with no upstream certificate: a proof \
             obligation,\n\
             #   grid evidence only — prove or refute it upstream, then move it to `proved:`.\n\
             # - a DISAGREEMENT (a proved law the exhaustive carrier refutes) never renders \
             here: it\n\
             #   fails the gate — a defect in the export/bridge pipeline, with certainty.\n\n",
            self.theory
        );
        out.push_str("agreements (proved upstream; the grid could not refute them):\n");
        if self.agreements.is_empty() {
            out.push_str("- none claimed.\n");
        }
        for a in &self.agreements {
            out.push_str(&format!("- {a}\n"));
        }
        out.push_str("\nconjectures (discovered here; unproved upstream — proof obligations):\n");
        if self.conjectures.is_empty() {
            out.push_str("- none: every discovered law carries an upstream certificate.\n");
        }
        for c in &self.conjectures {
            out.push_str(&format!("- {c}\n"));
        }
        out
    }

    /// The triage as a lock at `spec_dir/<theory>.obligations.spec`.
    pub fn lock_in(&self, spec_dir: &Path) -> spec_lock::Lock {
        spec_lock::Lock {
            name: format!("{} obligations", self.theory),
            path: spec_dir.join(format!("{}.obligations.spec", self.theory)),
            live: self.render(),
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    fn bool_export() -> &'static str {
        // a private slot-2 twin of the committed fixture, so lib probes never race the
        // integration tests' slot-0 install of the file on disk. SEVEN operators on
        // purpose: the eval fn-pointer table has eight arms, and this fixture walks
        // indices 0..=6, so a deleted arm routes an operator through the fallback and
        // dies indexing past the export's table — no arm can rot unexercised.
        "theory: probe-bool\n\
         elements: false true\n\
         op false/0: false\n\
         op true/0: true\n\
         op not/1: true false\n\
         op and/2: false false | false true\n\
         op or/2: false true | true true\n\
         op xor/2: false true | true false\n\
         op nand/2: true true | true false\n\
         proved: commutative and\n\
         proved: identity and true\n"
    }

    /// The parser reads the format back exactly: carrier order, table layout (row-major,
    /// first argument fixes the row), arities, and the proved lines in the declaration
    /// vocabulary.
    #[test]
    fn the_parser_reads_the_format_back_exactly() {
        let e = Export::parse(bool_export()).expect("the fixture parses");
        assert_eq!(e.name, "probe-bool");
        assert_eq!(e.elements, vec!["false", "true"]);
        let and = e.ops.iter().find(|o| o.token == "and").expect("and");
        assert_eq!((and.arity, and.table.clone()), (2, vec![0, 0, 0, 1]));
        let not = e.ops.iter().find(|o| o.token == "not").expect("not");
        assert_eq!((not.arity, not.table.clone()), (1, vec![1, 0]));
        assert_eq!(
            e.proved.iter().map(Expectation::render).collect::<Vec<_>>(),
            vec!["commutative(and)", "identity(and, true)"]
        );
    }

    /// A mounted export is a FULL theory: discovery runs, the proved laws agree, and the
    /// conjecture list is non-empty (the whole point — the grid supplies what the prover
    /// has not certified yet).
    #[test]
    fn a_mounted_export_discovers_and_triages() {
        Export::parse(bool_export())
            .expect("parses")
            .install::<2>()
            .expect("slot 2 is the probes'");
        let t = Triage::of::<Bridged<2>>();
        assert_eq!(t.certify(), Ok(()), "no disagreement in an honest export");
        assert_eq!(
            t.agreements,
            vec!["commutative(and)", "identity(and, true)"],
            "both certificates must round-trip"
        );
        assert!(
            !t.conjectures.is_empty(),
            "the grid must supply conjectures beyond the two certificates"
        );
        // and re-installing the identical export is idempotent, while a different one
        // is refused by name.
        Export::parse(bool_export())
            .expect("parses")
            .install::<2>()
            .expect("identical re-install is idempotent");
        let other = Export::parse("theory: other\nelements: a\nop id/1: a\n").expect("parses");
        let refusal = other.install::<2>().expect_err("a slot never retargets");
        assert!(refusal.contains("cannot retarget"), "{refusal}");
    }

    /// THE DISAGREEMENT DETECTOR FIRES: an export claiming `commutative implies` — a
    /// certificate the tables refute — is convicted by name, with the certainty prose.
    #[test]
    fn a_false_certificate_is_a_conviction_not_a_row() {
        Export::parse(
            "theory: probe-implies\n\
             elements: false true\n\
             op implies/2: true true | false true\n\
             proved: commutative implies\n",
        )
        .expect("parses")
        .install::<3>()
        .expect("slot 3 is the drill's");
        let t = Triage::of::<Bridged<3>>();
        assert_eq!(t.disagreements, vec!["commutative(implies)"]);
        let verdict = t.certify().expect_err("the detector must fire");
        assert!(
            verdict.contains("REFUTES upstream-proved law(s): commutative(implies)")
                && verdict.contains("with certainty"),
            "the conviction must name the law and its weight: {verdict}"
        );
        // and the render never launders a disagreement into a ratifiable row.
        assert!(!t.render().contains("commutative(implies)"));
    }

    /// Every parse diagnosis is NAMED — the parser is a gate, drilled refusal by
    /// refusal (missing theory, empty carrier, oversize carrier, bad arity, bad row
    /// shape, unknown element, duplicate op, unknown shape word, unexported op,
    /// unreadable line).
    #[test]
    fn every_parse_refusal_arrives_by_name() {
        let refused = |text: &str, needle: &str| {
            let err = Export::parse(text).expect_err(needle);
            assert!(err.contains(needle), "wanted {needle:?} in {err:?}");
        };
        refused("elements: a\nop id/1: a\n", "names no theory");
        refused("theory: t\nop id/1: a\n", "the carrier must come first");
        refused("theory: t\nelements: a\n", "no operator tables");
        refused(
            "theory: t\nelements: a b c d e f g h i\n",
            "at most 8 exhaustively",
        );
        refused("theory: t\nelements: a\nop f/3: a\n", "arity 3");
        refused("theory: t\nelements: a b\nop f/1: a\n", "needs 2 per row");
        refused("theory: t\nelements: a b\nop f/2: a b\n", "needs 2");
        refused(
            "theory: t\nelements: a\nop f/1: q\n",
            "not an exported element",
        );
        refused(
            "theory: t\nelements: a\nop f/1: a\nop f/1: a\n",
            "exported twice",
        );
        refused(
            "theory: t\nelements: a\nop f/1: a\nproved: zigzag f\n",
            "not a ratified catalog shape",
        );
        refused(
            "theory: t\nelements: a\nop f/1: a\nproved: involution g\n",
            "not an exported operator",
        );
        refused(
            "theory: t\nelements: a\nop f/1: a\nwat\n",
            "unreadable line",
        );
    }

    /// The DERIVED bridge locks are fresh FROM THE LIBRARY SIDE — the delta-render
    /// lesson applied to the root crate: the mutation sweeps judge mutants against the
    /// lib probes (plus two enlisted drill suites), so every lock a mutant could
    /// silently invalidate must be re-derivable from here. A perturbed bridged theory
    /// (its name, operator table, variables, or sampling budget) discovers a different
    /// law set or renders a different triage, and these gates catch that as spec drift.
    /// (`tests/bridge.rs` keeps the human-facing twins.)
    #[test]
    fn the_committed_bridge_locks_are_fresh_from_the_library_side() {
        let spec_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec");
        let text = std::fs::read_to_string(spec_dir.join("bridged-bool.export"))
            .expect("the committed export fixture is part of the tree");
        Export::parse(&text)
            .expect("the committed export parses")
            .install::<0>()
            .expect("slot 0 holds the committed fixture");
        let locks = [
            crate::discover::Spec::of::<Bridged<0>>().lock_in(&spec_dir),
            crate::discover::mutation::MutationReport::of::<Bridged<0>>().lock_in(&spec_dir),
            Triage::of::<Bridged<0>>().lock_in(&spec_dir),
        ];
        if let Err(stale) = spec_lock::check(&locks) {
            panic!(
                "bridge lock drifted for: {}. \
                 Run `cargo run --example freeze_spec` and ratify the diff.",
                stale.join(", ")
            );
        }
    }

    /// The element cap is a CEILING, not a wall one short of it: an exactly-8-element
    /// carrier parses (the exhaustive-judgment guarantee covers it), and only the 9th
    /// is refused — pinned from both sides so the boundary cannot creep.
    #[test]
    fn the_element_cap_admits_exactly_the_ceiling() {
        let at_cap = "theory: t\nelements: a b c d e f g h\nop id/1: a b c d e f g h\n";
        assert_eq!(
            Export::parse(at_cap)
                .expect("eight elements parse")
                .elements
                .len(),
            MAX_ELEMENTS
        );
    }

    /// THE EXHAUSTIVENESS LOCKSTEP: "a bridged refutation is a fact, not a sample"
    /// holds only while the sampling budget covers the parse ceiling's worst carrier —
    /// two independently-declared constants that must move together. (The engine floors
    /// its budget at 64, so a small carrier hides a broken budget: this pin is the only
    /// probe that can see it.)
    #[test]
    fn the_sampling_budget_covers_the_whole_admissible_carrier() {
        assert!(
            <Bridged<0> as Theory>::grid_size() >= MAX_ELEMENTS * MAX_ELEMENTS * MAX_ELEMENTS,
            "grid_size must keep every admissible carrier exhaustive: raise it in \
             lockstep with MAX_ELEMENTS"
        );
    }
}
