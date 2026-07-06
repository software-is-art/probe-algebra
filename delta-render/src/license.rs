//! license — the PIVOT artifact: operator classifications derived by READING the specs.
//!
//! An operator's classification is the presence of laws in its frozen spec, never a
//! boolean anyone types:
//!
//! - LINEAR ⇔ the spec contains the additive homomorphism (`{op} turns plus into
//!   plus.`) AND the zero fixed point (`{op} leaves zero fixed.`);
//! - BILINEAR ⇔ additive in each argument slot: the distributivity pair (`{op}
//!   distributes over plus.` + `{op} distributes over plus from the right.`), or one
//!   distributivity law plus commutativity (the catalog skips the right-slot law for a
//!   commutative operator because the left-slot law already says it);
//! - NEITHER ⇔ no license found. The generic fallback (`Q^Δ = D ∘ Q ∘ I`) still
//!   applies — correct always, cheap never — so a missing license only ever costs
//!   performance, never correctness.
//!
//! [`Registry::derive`] re-renders each operator's spec through the same path the lock
//! freezes and parses the TEXT for those law lines — so the registry's input is the spec
//! artifact itself, and `spec/licenses.spec` is a lock over a derivation whose input is
//! other locks: discovery output consumed as generation input. Each classification
//! carries its citations (the exact law lines, and the spec file they are ratified in)
//! into the render, so the registry reads as a claim with evidence, never a table of
//! assertions.
//!
//! Honest frame: a license is DISCOVERED — bounded refutation over the grid — so it is
//! evidence, not proof. The end gate (`I ∘ Q^Δ ∘ D = Q`) holds regardless of what this
//! file says; a forged license surfaces there.

use std::path::Path;

use boundary_spec::discover::engine::Theory;
use boundary_spec::discover::Spec;

use crate::ops::{DistinctOp, FilterOp, JoinOp, MapOp, MinOp, SumOp};

/// What an operator's spec licenses — the derivation rule its circuit nodes may use.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Classification {
    /// The operator is its own incremental form (`Q^Δ = Q`) — DBSP: linear operators
    /// commute with differentiation and integration.
    Linear,
    /// The three-term delta applies: `Δ(a ⋈ b) = Δa ⋈ b_prev + a_prev ⋈ Δb + Δa ⋈ Δb`.
    Bilinear,
    /// No license — the generic fallback (`Q^Δ = D ∘ Q ∘ I`): integrate, recompute,
    /// differentiate. Correct always, cheap never.
    Neither,
}

impl Classification {
    /// The registry's display token.
    pub fn render(&self) -> &'static str {
        match self {
            Classification::Linear => "LINEAR",
            Classification::Bilinear => "BILINEAR",
            Classification::Neither => "NEITHER",
        }
    }
}

/// One operator's license: its classification plus the law lines that grant it, cited
/// against the spec file they are ratified in.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct License {
    /// The operator, by the name that keys its theory and its spec file.
    pub operator: String,
    /// What the spec's laws license.
    pub classification: Classification,
    /// The exact law lines that granted the classification (empty for NEITHER).
    pub citations: Vec<String>,
    /// The spec file the citations are ratified in (`spec/<op>.spec`).
    pub spec_file: String,
}

impl License {
    /// Read one operator's license off its spec TEXT (the same rendered form the lock
    /// freezes). Pure text: the input is the spec artifact, not the code.
    pub fn read(operator: &str, spec_file: &str, spec_text: &str) -> License {
        let has = |line: &str| spec_text.lines().any(|l| l.trim() == line);
        let hom = format!("- {operator} turns plus into plus.");
        let fixed = format!("- {operator} leaves zero fixed.");
        let dist = format!("- {operator} distributes over plus.");
        let dist_right = format!("- {operator} distributes over plus from the right.");
        let comm = format!("- {operator} gives the same result in either order.");

        let cite = |lines: &[&String]| -> Vec<String> {
            lines
                .iter()
                .map(|l| l.trim_start_matches("- ").to_string())
                .collect()
        };

        if has(&dist) && (has(&dist_right) || has(&comm)) {
            let second = if has(&dist_right) { &dist_right } else { &comm };
            License {
                operator: operator.to_string(),
                classification: Classification::Bilinear,
                citations: cite(&[&dist, second]),
                spec_file: spec_file.to_string(),
            }
        } else if has(&hom) && has(&fixed) {
            License {
                operator: operator.to_string(),
                classification: Classification::Linear,
                citations: cite(&[&hom, &fixed]),
                spec_file: spec_file.to_string(),
            }
        } else {
            License {
                operator: operator.to_string(),
                classification: Classification::Neither,
                citations: vec![],
                spec_file: spec_file.to_string(),
            }
        }
    }
}

/// The license registry: every lifted operator's classification, derived, in inventory
/// order — the single table the circuit renderer reads.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Registry {
    pub licenses: Vec<License>,
}

impl Registry {
    /// Derive the whole registry from the live specs, rendered through the same path
    /// the locks freeze (so this text is byte-identical to the committed spec files
    /// whenever those are fresh — the registry reads the specs, not the code).
    pub fn derive() -> Registry {
        fn one<T: Theory>(op: &str) -> License {
            // the lock's path is irrelevant here — only its rendered text is read.
            let lock = Spec::of::<T>().lock_in(Path::new("spec"));
            License::read(op, &format!("spec/{}.spec", T::name()), &lock.live)
        }
        Registry {
            licenses: vec![
                one::<FilterOp>("filter"),
                one::<MapOp>("map"),
                one::<SumOp>("sum"),
                one::<JoinOp>("join"),
                one::<DistinctOp>("distinct"),
                one::<MinOp>("min"),
            ],
        }
    }

    /// A license by operator name — the renderer's lookup; `None` is "unlicensed", which
    /// the circuit validity rule turns into unconstructible, not a runtime error.
    pub fn get(&self, operator: &str) -> Option<&License> {
        self.licenses.iter().find(|l| l.operator == operator)
    }

    /// The registry as deterministic text — `spec/licenses.spec`'s whole content.
    pub fn render(&self) -> String {
        let mut out = String::from(
            "# license registry: operator → classification, DERIVED by parsing the frozen law \
             specs — regenerate via this repo's freeze path and ratify the diff.\n#\n\
             # The pivot artifact: discovery output consumed as generation input. A\n\
             # classification is the PRESENCE of laws in the operator's spec, never a\n\
             # declared boolean:\n\
             #   linear   ⇔ additive homomorphism over the Z-set group AND zero preserved\n\
             #   bilinear ⇔ additive in each argument slot (the distributivity pair, or one\n\
             #              distributivity law plus commutativity)\n\
             #   neither  ⇔ no license — the generic fallback (Q^Δ = D ∘ Q ∘ I) applies\n",
        );
        for l in &self.licenses {
            out.push_str(&format!(
                "\n- {}: {}\n",
                l.operator,
                l.classification.render()
            ));
            if l.citations.is_empty() {
                out.push_str(&format!(
                    "      no additivity law in {} — every delta recomputes (D ∘ Q ∘ I)\n",
                    l.spec_file
                ));
            } else {
                for c in &l.citations {
                    out.push_str(&format!("      {}: \"{}\"\n", l.spec_file, c));
                }
            }
        }
        out
    }

    /// The registry as a lock in this crate's `spec/` directory.
    pub fn lock_in(&self, spec_dir: &Path) -> spec_lock::Lock {
        spec_lock::Lock {
            name: "licenses".into(),
            path: spec_dir.join("licenses.spec"),
            live: self.render(),
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The classifier reads TEXT, so it is pinned on synthetic spec text — each rule and
    /// each near-miss: hom+fixed is linear; either bilinear route wins over linear; a
    /// fixed point alone (distinct's real shape) is nothing; one distributivity without
    /// the other slot is nothing; and the law must belong to THIS operator.
    #[test]
    fn the_classifier_reads_exactly_the_defining_laws() {
        let linear = "- f turns plus into plus.\n- f leaves zero fixed.\n";
        assert_eq!(
            License::read("f", "spec/f.spec", linear).classification,
            Classification::Linear
        );
        let bilinear_comm =
            "- j distributes over plus.\n- j gives the same result in either order.\n";
        assert_eq!(
            License::read("j", "spec/j.spec", bilinear_comm).classification,
            Classification::Bilinear
        );
        let bilinear_pair =
            "- j distributes over plus.\n- j distributes over plus from the right.\n";
        assert_eq!(
            License::read("j", "spec/j.spec", bilinear_pair).classification,
            Classification::Bilinear
        );
        // fixed point alone licenses nothing (distinct's real spec shape).
        let fixed_only = "- d leaves zero fixed.\n";
        assert_eq!(
            License::read("d", "spec/d.spec", fixed_only).classification,
            Classification::Neither
        );
        // homomorphism alone licenses nothing (zero preservation is load-bearing).
        let hom_only = "- f turns plus into plus.\n";
        assert_eq!(
            License::read("f", "spec/f.spec", hom_only).classification,
            Classification::Neither
        );
        // one slot's additivity alone licenses nothing.
        let one_slot = "- j distributes over plus.\n";
        assert_eq!(
            License::read("j", "spec/j.spec", one_slot).classification,
            Classification::Neither
        );
        // another operator's laws are not this operator's license.
        let someone_elses = "- g turns plus into plus.\n- g leaves zero fixed.\n";
        assert_eq!(
            License::read("f", "spec/f.spec", someone_elses).classification,
            Classification::Neither
        );
    }

    /// The citations carry the exact granted laws, and NEITHER carries none.
    #[test]
    fn a_license_cites_the_laws_that_granted_it() {
        let linear = "- f turns plus into plus.\n- f leaves zero fixed.\n";
        let l = License::read("f", "spec/f.spec", linear);
        assert_eq!(
            l.citations,
            vec![
                "f turns plus into plus.".to_string(),
                "f leaves zero fixed.".to_string()
            ]
        );
        assert!(License::read("d", "spec/d.spec", "").citations.is_empty());
    }

    /// The registry lookup answers by name and refuses the unknown — the circuit's
    /// unconstructibility hangs off this `None`.
    #[test]
    fn the_registry_lookup_is_by_name_and_total_in_none() {
        let r = Registry::derive();
        assert_eq!(r.get("join").expect("join is inventoried").operator, "join");
        assert!(r.get("median").is_none());
    }
}
