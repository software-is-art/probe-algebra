//!
//! bite — STATEMENT-BITE MUTATION for a proof corpus: mutate the DEFINITIONS, never the
//! proofs, and demand every theorem fail to re-check.
//!
//! A kernel certifies that proofs follow from statements; it cannot ask whether the
//! statements SAY anything. A definition mutant makes that question executable: flip one
//! result literal in a definition and re-run the checker — if every theorem still
//! checks, no theorem depended on that degree of freedom, which is the vacuous-statement
//! finding (a definition too weak to constrain, a quantifier scoped past the content)
//! the kernel cannot make about itself. Killed / survived / ratified follows the residue
//! policy verbatim; survivors are ratified BY KEY with a justification in a
//! [`spec_lock::Register`] — the exception-register pattern, first-class: drift renders
//! as set difference ("1 new finding, 2 resolved"), never a byte diff.
//!
//! The corpus under bite is our own: `lean/ProbeBool.lean` formalises the bridged
//! Boolean fragment, making it the REAL upstream prover behind
//! `spec/bridged-bool.export` — the bridge's `proved:` lines each cite a Lean theorem
//! (the `-- certifies:` annotations; the bijection is a probe below), so conjecture
//! supply, upstream proof, and certificate ratification are one gated loop.
//!
//! Two checkers, one honesty split:
//!
//! - the SUBSTRATE checker (`lean`, run by `examples/statement_bite.rs` and the weekly
//!   CI gate) is the authority — the same kernel that certified the proofs judges the
//!   mutants;
//! - the MIRROR checker (in this module's probes) re-evaluates every theorem over the
//!   parsed tables in Rust, so the expected survivor set is pinned inside every
//!   `cargo test` without a Lean toolchain. The mirror is the corpus's semantic twin,
//!   maintained next to the theorem list; it verifies the REGISTER, never the proofs.
//!
//! v1 scope, disclosed: the mutant family is result-literal flips in match arms (the
//! corpus's whole definition style — the same family `discover::mutation` plants in
//! operator tables, at the prover's level). Pattern flips are excluded on purpose: they
//! make definitions ill-formed, and a compile error is not a theorem failing to re-check.

use std::path::{Path, PathBuf};

use spec_lock::Register;

/// The proof corpus, split at its statement-bite marker: everything before the marker
/// is biteable DEFINITIONS; everything after is theorems the bites must break.
#[derive(Debug)]
pub struct Corpus {
    /// The whole file, verbatim.
    pub text: String,
    /// The definitions region (bites happen here).
    pub definitions: String,
    /// The theorems region (never mutated).
    pub theorems: String,
}

/// The marker line prefix that ends the definitions region.
pub const MARKER: &str = "-- ===== THEOREMS";

impl Corpus {
    /// This repo's committed corpus file.
    pub fn committed_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lean/ProbeBool.lean")
    }

    /// The ratified-survivors register for the committed corpus — hand-authored, one
    /// justification per surviving bite (see `spec_lock::Register` for the discipline).
    pub fn survivor_register() -> Register {
        Register {
            name: "lean statement-bite survivors".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lean/bites.register"),
        }
    }

    /// Split a corpus text at the marker. No marker, no corpus: a file that does not
    /// declare where its definitions end cannot be bitten honestly.
    pub fn parse(text: &str) -> Result<Corpus, String> {
        let Some(pos) = text
            .lines()
            .position(|l| l.trim_start().starts_with(MARKER))
        else {
            return Err(format!(
                "the corpus has no `{MARKER}` marker — the statement-bite region must be \
                 declared, not guessed"
            ));
        };
        let lines: Vec<&str> = text.lines().collect();
        Ok(Corpus {
            text: text.to_string(),
            definitions: lines[..pos].join("\n"),
            theorems: lines[pos..].join("\n"),
        })
    }

    /// Read and split the committed corpus file.
    pub fn read(path: &Path) -> Result<Corpus, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("corpus unreadable at {}: {e}", path.display()))?;
        Corpus::parse(&text)
    }
}

/// One statement bite: a keyed definition mutant (the key is what the register
/// ratifies) and the whole mutated file text.
pub struct Bite {
    /// `band | true, false => true` — the definition, the arm, and what the result
    /// literal was flipped TO. Colon-free on purpose: register keys end at the first
    /// colon.
    pub key: String,
    /// The corpus text with exactly this one literal flipped.
    pub mutated: String,
}

impl Corpus {
    /// Generate every bite: for each match arm in the DEFINITIONS region, flip each
    /// `true`/`false` word in the arm's result (right of `=>`). Patterns are never
    /// touched — a pattern flip is a syntax wound, not a statement bite.
    /// Deterministic: file order.
    pub fn bites(&self) -> Vec<Bite> {
        let corpus = self;
        let mut out = Vec::new();
        let mut current_def = String::new();
        let lines: Vec<&str> = corpus.text.lines().collect();
        let def_line_count = corpus.definitions.lines().count();
        for (i, line) in lines.iter().enumerate().take(def_line_count) {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("def ") {
                current_def = rest.split_whitespace().next().unwrap_or("").to_string();
            }
            if trimmed.starts_with("--") || !trimmed.starts_with('|') {
                continue;
            }
            let Some((pattern, rhs)) = line.split_once("=>") else {
                continue;
            };
            let pattern_key = pattern.trim().trim_start_matches('|').trim();
            let words: Vec<&str> = rhs.split_whitespace().collect();
            for (w, word) in words.iter().enumerate() {
                let flipped = match *word {
                    "true" => "false",
                    "false" => "true",
                    _ => continue,
                };
                let mut new_words: Vec<&str> = words.clone();
                new_words[w] = flipped;
                let new_line = format!("{}=> {}", pattern, new_words.join(" "));
                let mut mutated_lines = lines.clone();
                mutated_lines[i] = &new_line;
                out.push(Bite {
                    key: format!("{current_def} | {pattern_key} => {flipped}"),
                    mutated: format!("{}\n", mutated_lines.join("\n")),
                });
            }
        }
        out
    }
}

/// The verdict sheet: every bite judged by the checker.
#[derive(Debug)]
pub struct BiteVerdicts {
    /// Bites whose mutant broke at least one theorem — the statement constrained
    /// that degree of freedom.
    pub killed: Vec<String>,
    /// Bites the corpus re-checked UNDER — no theorem noticed; ratify or write the
    /// missing theorem.
    pub survived: Vec<String>,
}

impl Corpus {
    /// Judge every bite with an injected checker: `check(text)` returns `Ok(true)`
    /// when the whole corpus text re-checks, `Ok(false)` when some theorem fails. The
    /// baseline (unmutated) corpus MUST check first — a corpus that does not check
    /// judges nothing.
    ///
    /// The checker is injected so the substrate (`lean`) and the mirror (the probes'
    /// Rust re-evaluation) run through one harness.
    pub fn judge_with(
        &self,
        mut check: impl FnMut(&str) -> Result<bool, String>,
    ) -> Result<BiteVerdicts, String> {
        if !check(&self.text)? {
            return Err(
                "the UNMUTATED corpus does not check — fix the corpus before judging bites \
             against it"
                    .to_string(),
            );
        }
        let mut verdicts = BiteVerdicts {
            killed: Vec::new(),
            survived: Vec::new(),
        };
        for bite in self.bites() {
            if check(&bite.mutated)? {
                verdicts.survived.push(bite.key);
            } else {
                verdicts.killed.push(bite.key);
            }
        }
        Ok(verdicts)
    }
}

impl BiteVerdicts {
    /// The gate: survivors held against the ratified register. `Ok` carries the
    /// summary line for the gate log; `Err` is the register's set-difference render
    /// (new findings want a justification or a theorem; resolved ones want their
    /// line deleted).
    pub fn gate(&self, register: &Register) -> Result<String, String> {
        register.check(self.survived.iter().map(String::as_str))?;
        Ok(format!(
            "statement bites: {} planted, {} killed, {} survived (every survivor \
             ratified in {})",
            self.killed.len() + self.survived.len(),
            self.killed.len(),
            self.survived.len(),
            register.path.display()
        ))
    }
}

// ===== table parsing (shared by the mirror probe and the export cross-check) =======

impl Corpus {
    /// Every definition's result table, parsed from its match arms in the DEFINITIONS
    /// region: pattern words → result.
    pub fn tables(&self) -> std::collections::BTreeMap<String, Vec<(Vec<bool>, bool)>> {
        let mut out = std::collections::BTreeMap::new();
        let mut current = String::new();
        for line in self.definitions.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("def ") {
                current = rest.split_whitespace().next().unwrap_or("").to_string();
                out.entry(current.clone()).or_insert_with(Vec::new);
                continue;
            }
            if trimmed.starts_with("--") || !trimmed.starts_with('|') {
                continue;
            }
            let Some((pattern, rhs)) = trimmed.split_once("=>") else {
                continue;
            };
            let word = |w: &str| w.trim() == "true";
            let args: Vec<bool> = pattern
                .trim_start_matches('|')
                .split(',')
                .map(word)
                .collect();
            let result = word(rhs);
            out.entry(current.clone())
                .or_insert_with(Vec::new)
                .push((args, result));
        }
        out
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::bridge::Export;

    fn corpus() -> Corpus {
        Corpus::read(&Corpus::committed_path()).expect("the committed corpus splits at its marker")
    }

    /// THE MIRROR: every theorem in the corpus, re-evaluated over parsed tables. This
    /// is the corpus's semantic twin — one closure per Lean theorem, in theorem order —
    /// so bites are judged inside `cargo test` with no Lean toolchain. Lean (CI's
    /// weekly gate) stays the substrate authority; the mirror pins the REGISTER.
    fn mirror(text: &str) -> Result<bool, String> {
        let c = Corpus::parse(text)?;
        let t = c.tables();
        let lookup = |name: &str, args: &[bool]| -> bool {
            t.get(name)
                .and_then(|rows| rows.iter().find(|(a, _)| a == args))
                .map(|(_, r)| *r)
                .unwrap_or_else(|| panic!("no arm for {name} {args:?}"))
        };
        let bnot = |x: bool| lookup("bnot", &[x]);
        let band = |x: bool, y: bool| lookup("band", &[x, y]);
        let bor = |x: bool, y: bool| lookup("bor", &[x, y]);
        let bxor = |x: bool, y: bool| lookup("bxor", &[x, y]);
        let b = [false, true];
        let mut holds = true;
        for x in b {
            holds &= band(true, x) == x; // and_id
            holds &= bnot(bnot(x)) == x; // not_involution
            holds &= bor(false, x) == x; // or_id
            holds &= !bxor(x, x); // xor_self (`bxor x x = false`)
            for y in b {
                holds &= band(x, y) == band(y, x); // and_comm
                holds &= bor(x, y) == bor(y, x); // or_comm
                holds &= bnot(band(x, y)) == bor(bnot(x), bnot(y)); // demorgan_and
                holds &= bnot(bor(x, y)) == band(bnot(x), bnot(y)); // demorgan_or
                holds &= bxor(x, y) == bxor(y, x); // xor_comm
                for z in b {
                    holds &= band(band(x, y), z) == band(x, band(y, z)); // and_assoc
                }
            }
        }
        Ok(holds)
    }

    /// The generator flips exactly the result literals: one bite per arm (the corpus's
    /// arms carry one literal each), patterns untouched, keys colon-free and stable.
    #[test]
    fn bites_flip_results_and_never_patterns() {
        let c = corpus();
        let all = c.bites();
        // 4 defs x 4 arms + bnot's 2 arms = 18 single-literal arms.
        assert_eq!(all.len(), 18, "the corpus's arm census moved");
        for bite in &all {
            assert!(!bite.key.contains(':'), "register keys end at ':'");
            // exactly one line differs, and only right of `=>`.
            let orig: Vec<&str> = c.text.lines().collect();
            let mutated: Vec<&str> = bite.mutated.lines().collect();
            let diffs: Vec<usize> = (0..orig.len())
                .filter(|i| orig[*i] != mutated[*i])
                .collect();
            assert_eq!(diffs.len(), 1, "one bite, one line: {}", bite.key);
            let (o, m) = (orig[diffs[0]], mutated[diffs[0]]);
            assert_eq!(
                o.split_once("=>").map(|(p, _)| p.trim()),
                m.split_once("=>").map(|(p, _)| p.trim()),
                "the pattern side must never move: {}",
                bite.key
            );
        }
        assert!(
            all.iter().any(|b| b.key == "bnand | true, true => true"),
            "keys read as definition-arm-flip"
        );
    }

    /// THE REGISTER, PINNED SEMANTICALLY: judged by the mirror, the survivors are
    /// exactly bnand's four bites — the planted untheoremed definition — and nothing
    /// else. Every other definition is fully constrained by the ten theorems. This is
    /// what `lean/bites.register` ratifies; the weekly Lean gate re-judges the same
    /// set on the substrate.
    #[test]
    fn the_survivors_are_exactly_the_planted_untheoremed_definition() {
        let verdicts = corpus().judge_with(mirror).expect("the corpus checks");
        assert_eq!(
            verdicts.survived,
            vec![
                "bnand | true, true => true",
                "bnand | true, false => false",
                "bnand | false, true => false",
                "bnand | false, false => false",
            ],
            "a survivor moved: either a theorem lost its grip (new survivor) or a \
             ratified debt was paid (resolved) — re-ratify the register either way"
        );
        assert_eq!(verdicts.killed.len(), 14, "every constrained bite dies");
        // and the committed register matches, through the real gate.
        let summary = verdicts
            .gate(&Corpus::survivor_register())
            .expect("the register is current");
        assert_eq!(
            summary,
            format!(
                "statement bites: 18 planted, 14 killed, 4 survived (every survivor \
                 ratified in {})",
                Corpus::survivor_register().path.display()
            )
        );
    }

    /// The judge's polarity and the baseline rule, on a synthetic corpus: a checker
    /// that fails the baseline refuses to judge; kills and survivals land on the
    /// right side.
    #[test]
    fn the_judge_demands_a_green_baseline_and_sorts_verdicts() {
        let text = "def f : Bool → Bool\n  | true => true\n  | false => false\n\n\
                    -- ===== THEOREMS =====\ntheorem t : f true = true := rfl\n";
        let c = Corpus::parse(text).expect("splits");
        let err = c.judge_with(|_| Ok(false)).unwrap_err();
        assert!(err.contains("UNMUTATED corpus does not check"), "{err}");
        // a checker that only accepts the original: every bite is killed.
        let original = c.text.clone();
        let v = c.judge_with(|t| Ok(t == original)).expect("baseline green");
        assert_eq!((v.killed.len(), v.survived.len()), (2, 0));
        // a checker that accepts anything: every bite survives.
        let v = c.judge_with(|_| Ok(true)).expect("baseline green");
        assert_eq!((v.killed.len(), v.survived.len()), (0, 2));
        // and an unratified survivor is a NAMED register drift, set-diff rendered.
        let empty_register = Register {
            name: "empty".to_string(),
            path: std::env::temp_dir().join("bite-empty-register/none.register"),
        };
        let drift = v.gate(&empty_register).unwrap_err();
        assert!(
            drift.contains("2 new finding(s)") && drift.contains("f | true => false"),
            "{drift}"
        );
    }

    /// A corpus without the marker is refused — the biteable region is declared,
    /// never guessed.
    #[test]
    fn a_markerless_corpus_is_refused() {
        let err = Corpus::parse("def f : Bool := true\n").unwrap_err();
        assert!(err.contains("no `-- ===== THEOREMS` marker"), "{err}");
    }

    /// THE TABLES ARE THE EXPORT'S: every exported unary/binary operator has a Lean
    /// twin with cell-for-cell identical semantics (the nullary exports are Bool's own
    /// literals), and the planted `bnand` is exported NOWHERE — its debt lives in the
    /// register, not in the bridge.
    #[test]
    fn the_corpus_tables_and_the_export_tables_are_one_algebra() {
        let export_text = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec/bridged-bool.export"),
        )
        .expect("the committed export");
        let export = Export::parse(&export_text).expect("parses");
        let t = corpus().tables();
        let twins = [
            ("bnot", "not"),
            ("band", "and"),
            ("bor", "or"),
            ("bxor", "xor"),
        ];
        let idx = |b: bool| usize::from(b); // element order in the export: false=0, true=1
        for (lean_name, op_name) in twins {
            let rows = t.get(lean_name).expect("the corpus defines the twin");
            let op = export
                .ops
                .iter()
                .find(|o| o.token == op_name)
                .expect("the export carries the operator");
            assert_eq!(rows.len(), 1 << op.arity, "full table for {op_name}");
            for (args, result) in rows {
                let cell = match args.as_slice() {
                    [x] => op.table[idx(*x)],
                    [x, y] => op.table[idx(*x) * export.elements.len() + idx(*y)],
                    _ => panic!("arity beyond the export format"),
                };
                assert_eq!(
                    export.elements[cell as usize] == "true",
                    *result,
                    "{lean_name}{args:?} disagrees with export `{op_name}` — the corpus \
                     and the export must be ONE algebra"
                );
            }
        }
        // every exported non-nullary op is twinned; bnand is deliberately not exported.
        for op in export.ops.iter().filter(|o| o.arity > 0) {
            assert!(
                twins.iter().any(|(_, e)| *e == op.token),
                "exported `{}` has no Lean twin",
                op.token
            );
        }
        assert!(
            !export.ops.iter().any(|o| o.token == "nand"),
            "the untheoremed bnand must not be exported"
        );
    }

    /// THE CERTIFICATE BIJECTION: each `proved:` line in the export cites exactly one
    /// `-- certifies:` annotation in the corpus's theorem region, and vice versa — a
    /// theorem about an exported operator that exports no certificate is HIDING one,
    /// and a certificate with no theorem is a forgery.
    #[test]
    fn every_certificate_cites_a_theorem_and_every_theorem_exports_its_certificate() {
        let export_text = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec/bridged-bool.export"),
        )
        .expect("the committed export");
        let mut proved: Vec<&str> = export_text
            .lines()
            .filter_map(|l| l.trim().strip_prefix("proved:"))
            .map(str::trim)
            .collect();
        let c = corpus();
        let mut certified: Vec<&str> = c
            .theorems
            .lines()
            .filter_map(|l| l.trim().strip_prefix("-- certifies:"))
            .map(str::trim)
            .collect();
        proved.sort_unstable();
        certified.sort_unstable();
        assert_eq!(
            proved, certified,
            "proved: lines and certifies: annotations must be a bijection"
        );
        assert_eq!(proved.len(), 10, "the certificate census moved");
    }

    /// Only match ARMS are biteable and tabled: a definitions-region line that carries
    /// `=>` but does not start with `|` (prose, a lambda in a comment) is neither a
    /// bite site nor a table row — pinned on a corpus that plants exactly that trap,
    /// with exact counts so a loosened line filter cannot hide.
    #[test]
    fn only_match_arms_are_bitten_and_tabled() {
        let text = "def f : Bool → Bool\n\
                    \x20 | true => true\n\
                    \x20 | false => false\n\
                    note: this prose maps everything => true and is not an arm\n\
                    \n\
                    -- ===== THEOREMS =====\n\
                    theorem t : f true = true := rfl\n";
        let c = Corpus::parse(text).expect("splits");
        let keys: Vec<String> = c.bites().into_iter().map(|b| b.key).collect();
        assert_eq!(
            keys,
            vec!["f | true => false", "f | false => true"],
            "exactly the two arms bite — the prose line must not"
        );
        let t = c.tables();
        assert_eq!(
            t.get("f").map(Vec::len),
            Some(2),
            "exactly the two arms are table rows — the prose line must not"
        );
    }
}
