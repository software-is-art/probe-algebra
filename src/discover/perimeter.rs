//!
//! perimeter — THE SETTINGS ARE A LOCK: the repository's protection floor, declared,
//! rendered, and read back.
//!
//! The settings page was the last hand-clicked configuration in the loop — the exact
//! antipattern named in `docs/experience.md` (open text feeding the rules), one level
//! up: prose recipes translated into a UI, verified by nobody, drifting silently, and
//! never re-audited. This module makes the perimeter what everything else here already
//! is: a DECLARED artifact with a derived render and a drift gate.
//!
//!   - [`Perimeter::declared`] is the single source. Its required status checks are
//!     DERIVED from the gate registry ([`GateRegistry::pr_checks`]), so a gate renamed
//!     or re-cadenced moves this lock in the same diff — a rename can never silently
//!     unprotect the default branch.
//!   - `spec/perimeter.spec` locks the human-readable floor; `spec/perimeter.ruleset.json`
//!     is the APPLY-ABLE artifact — the branch ruleset a human posts once:
//!     `gh api repos/<owner>/<repo>/rulesets -X POST --input spec/perimeter.ruleset.json`.
//!   - The weekly `perimeter (settings drift)` world gate reads the LIVE rules back
//!     (`.github/perimeter.sh` → `examples/perimeter.rs`) and holds them to the floor —
//!     a perimeter never applied, or quietly relaxed, is a red gate naming the rule.
//!
//! The WRITE stays human, deliberately: repository administration is the platform's
//! kernel tier — the perimeter constrains the agents, so it must not be agent-writable
//! (a privilege is ratified, never inferred from conduct, never self-served). What the
//! machine owns is deriving what the settings SHOULD be and refusing to let reality rot
//! away from the declaration.
//!
//! FLOOR semantics, honestly framed: the declaration states minimums. Extra live
//! protections (more required checks, stricter rules) are NOT drift — stricter is never
//! a lie. Approvals are the one exact match: the floor declares zero because a solo
//! maintainer cannot approve their own pull request, so a live count above zero is not
//! "stricter", it is a deadlocked repository — that too refuses by name.

use std::path::PathBuf;

use spec_lock::Lock;

use super::gates::GateRegistry;

/// The declared perimeter — the repository-settings floor.
pub struct Perimeter {
    /// Status-check contexts a PR must pass — derived from the gate registry
    /// (rendered job names: the contexts GitHub actually reports).
    pub required_checks: Vec<String>,
    /// Exactly this many required approvals (see the module docs for why zero).
    pub required_approvals: u64,
    /// The only merge methods the ruleset allows.
    pub merge_methods: &'static [&'static str],
    /// Private vulnerability reporting must be enabled (SECURITY.md points at it).
    pub private_vulnerability_reporting: bool,
}

/// What the world reports, extracted from the GitHub API by `examples/perimeter.rs`
/// (the extraction is field reads; the JUDGMENT lives here, where the probes reach).
#[derive(Clone, Default)]
pub struct LivePerimeter {
    /// A `deletion` rule is active on the default branch.
    pub deletion_blocked: bool,
    /// A `non_fast_forward` rule is active (force pushes blocked).
    pub force_push_blocked: bool,
    /// The active `pull_request` rule's required approving review count, if any rule.
    pub required_approvals: Option<u64>,
    /// The merge methods actually allowed (from the pull-request rule, or the repo's
    /// `allow_*` flags where the rule predates `allowed_merge_methods`).
    pub merge_methods: Vec<String>,
    /// Required status-check contexts on the default branch.
    pub required_checks: Vec<String>,
    /// `None` = the endpoint could not be read; refused by name, never assumed.
    pub private_vulnerability_reporting: Option<bool>,
}

#[crate::mutate]
impl LivePerimeter {
    /// The judge's dent battery, derived from an APPLIED fixture (`self` must satisfy
    /// the floor exactly — no extra protections, or a widening dent stops being
    /// refusal-worthy). One dent per refusal-worthy single-fact perturbation; the
    /// destructure below is the completeness pin — a new live field refuses to
    /// compile until its dent is decided (see `discover::judgment`).
    pub fn dents(&self) -> Vec<super::judgment::LiveDent<LivePerimeter>> {
        let LivePerimeter {
            deletion_blocked: _,
            force_push_blocked: _,
            required_approvals: _,
            merge_methods: _,
            required_checks,
            private_vulnerability_reporting: _,
        } = self;
        let mut dents = Vec::new();
        let mut dent = |what: &str, must_name: &str, live: LivePerimeter| {
            dents.push(super::judgment::LiveDent {
                what: what.to_string(),
                live,
                must_name: must_name.to_string(),
            });
        };
        dent("deletion unblocked", "can be DELETED", {
            let mut l = self.clone();
            l.deletion_blocked = false;
            l
        });
        dent("force pushes unblocked", "FORCE PUSHES", {
            let mut l = self.clone();
            l.force_push_blocked = false;
            l
        });
        dent("the pull-request rule dropped", "no pull-request rule", {
            let mut l = self.clone();
            l.required_approvals = None;
            l
        });
        dent(
            "required approvals raised above the floor",
            "deadlocks a solo maintainer",
            {
                let mut l = self.clone();
                l.required_approvals = l.required_approvals.map(|n| n + 1);
                l
            },
        );
        dent("a widened merge method", "merge method `rebase`", {
            let mut l = self.clone();
            l.merge_methods.push("rebase".to_string());
            l
        });
        for check in required_checks {
            dent(
                &format!("required check `{check}` removed"),
                &format!("`{check}` is not required"),
                {
                    let mut l = self.clone();
                    l.required_checks.retain(|c| c != check);
                    l
                },
            );
        }
        dent("vulnerability reporting disabled", "DISABLED", {
            let mut l = self.clone();
            l.private_vulnerability_reporting = Some(false);
            l
        });
        dent("vulnerability reporting unreadable", "could not be READ", {
            let mut l = self.clone();
            l.private_vulnerability_reporting = None;
            l
        });
        dents
    }
}

#[crate::mutate]
impl Perimeter {
    /// The declared floor — required checks derived from the gate registry.
    pub fn declared() -> Perimeter {
        Perimeter {
            required_checks: GateRegistry::pr_checks(),
            required_approvals: 0,
            merge_methods: &["squash"],
            private_vulnerability_reporting: true,
        }
    }

    /// The human-readable floor — what `spec/perimeter.spec` freezes.
    pub fn render(&self) -> String {
        let mut out = String::from(
            "# the repository perimeter, DECLARED — the settings floor the weekly world gate\n\
             # (`perimeter (settings drift)`) reads back from the live API and refuses by name.\n\
             # Settings drift silently and nobody re-audits a settings page; this lock does.\n\
             # The WRITE stays human (a privilege is ratified, never self-served): apply\n\
             # spec/perimeter.ruleset.json once —\n\
             #   gh api repos/<owner>/<repo>/rulesets -X POST --input spec/perimeter.ruleset.json\n\
             # — and enable private vulnerability reporting in the repository's security\n\
             # settings. Extra live protections beyond this floor are NOT drift (stricter is\n\
             # never a lie); required approvals are the one exact match, because a count above\n\
             # zero deadlocks a solo maintainer. Regenerate with `cargo run --example freeze_gates`.\n\n",
        );
        out.push_str(&format!(
            "- pull requests required before merging; required approvals: {} (a solo\n\
             \x20 maintainer cannot approve their own PR — the gates are the reviewer)\n",
            self.required_approvals
        ));
        out.push_str(&format!(
            "- required status checks: {}\n",
            self.required_checks.join(", ")
        ));
        out.push_str(&format!(
            "- merge methods: {} only\n",
            self.merge_methods.join(", ")
        ));
        out.push_str("- force pushes to the default branch: blocked\n");
        out.push_str("- deletion of the default branch: blocked\n");
        out.push_str(&format!(
            "- private vulnerability reporting: {}\n",
            if self.private_vulnerability_reporting {
                "enabled"
            } else {
                "disabled"
            }
        ));
        out
    }

    /// The apply-able branch ruleset — what `spec/perimeter.ruleset.json` freezes. The
    /// one manual act left is posting this artifact; hand-authoring it in a UI is the
    /// antipattern this module exists to retire.
    pub fn ruleset_json(&self) -> String {
        let checks: Vec<String> = self
            .required_checks
            .iter()
            .map(|c| format!("        {{ \"context\": \"{c}\" }}"))
            .collect();
        let methods: Vec<String> = self
            .merge_methods
            .iter()
            .map(|m| format!("\"{m}\""))
            .collect();
        format!(
            "{{\n\
             \x20 \"name\": \"perimeter\",\n\
             \x20 \"target\": \"branch\",\n\
             \x20 \"enforcement\": \"active\",\n\
             \x20 \"conditions\": {{ \"ref_name\": {{ \"include\": [\"~DEFAULT_BRANCH\"], \"exclude\": [] }} }},\n\
             \x20 \"rules\": [\n\
             \x20   {{ \"type\": \"deletion\" }},\n\
             \x20   {{ \"type\": \"non_fast_forward\" }},\n\
             \x20   {{\n\
             \x20     \"type\": \"pull_request\",\n\
             \x20     \"parameters\": {{\n\
             \x20       \"required_approving_review_count\": {approvals},\n\
             \x20       \"dismiss_stale_reviews_on_push\": false,\n\
             \x20       \"require_code_owner_review\": false,\n\
             \x20       \"require_last_push_approval\": false,\n\
             \x20       \"required_review_thread_resolution\": false,\n\
             \x20       \"allowed_merge_methods\": [{methods}]\n\
             \x20     }}\n\
             \x20   }},\n\
             \x20   {{\n\
             \x20     \"type\": \"required_status_checks\",\n\
             \x20     \"parameters\": {{\n\
             \x20       \"strict_required_status_checks_policy\": false,\n\
             \x20       \"required_status_checks\": [\n\
             {checks}\n\
             \x20       ]\n\
             \x20     }}\n\
             \x20   }}\n\
             \x20 ]\n\
             }}\n",
            approvals = self.required_approvals,
            methods = methods.join(", "),
            checks = checks.join(",\n"),
        )
    }

    /// `spec/perimeter.spec` — the floor, frozen.
    pub fn lock(&self) -> Lock {
        Lock {
            name: "perimeter".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("perimeter.spec"),
            live: self.render(),
        }
    }

    /// `spec/perimeter.ruleset.json` — the apply-able artifact, frozen.
    pub fn ruleset_lock(&self) -> Lock {
        Lock {
            name: "perimeter ruleset".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("perimeter.ruleset.json"),
            live: self.ruleset_json(),
        }
    }

    /// Hold the LIVE perimeter to the declared floor. `Ok` carries the held facts (a
    /// green run still shows what is being defended); `Err` carries every violation,
    /// each naming the rule and what to do — including the never-applied state, which
    /// is how the one manual act stays a red gate instead of a forgotten checklist.
    pub fn judge(&self, live: &LivePerimeter) -> Result<Vec<String>, Vec<String>> {
        fn fact(
            held: &mut Vec<String>,
            violations: &mut Vec<String>,
            ok: bool,
            held_line: String,
            violation: String,
        ) {
            if ok {
                held.push(held_line);
            } else {
                violations.push(violation);
            }
        }
        let mut held = Vec::new();
        let mut violations = Vec::new();

        fact(
            &mut held,
            &mut violations,
            live.deletion_blocked,
            "deletion of the default branch: blocked".to_string(),
            "the default branch can be DELETED — the declared perimeter blocks deletion \
             (apply spec/perimeter.ruleset.json)"
                .to_string(),
        );
        fact(
            &mut held,
            &mut violations,
            live.force_push_blocked,
            "force pushes: blocked".to_string(),
            "the default branch accepts FORCE PUSHES — the declared perimeter blocks them \
             (apply spec/perimeter.ruleset.json)"
                .to_string(),
        );
        match live.required_approvals {
            None => violations.push(
                "no pull-request rule is active — direct pushes to the default branch are \
                 unguarded (apply spec/perimeter.ruleset.json)"
                    .to_string(),
            ),
            Some(n) if n == self.required_approvals => {
                held.push(format!("pull requests required; approvals: {n}"));
            }
            Some(n) => violations.push(format!(
                "required approvals is {n}, declared {} — above the floor this is not \
                 stricter, it deadlocks a solo maintainer (no one can approve their own PR)",
                self.required_approvals
            )),
        }
        for method in &live.merge_methods {
            fact(
                &mut held,
                &mut violations,
                self.merge_methods.contains(&method.as_str()),
                format!("merge method allowed: {method}"),
                format!(
                    "merge method `{method}` is allowed — the declared perimeter permits \
                     only: {}",
                    self.merge_methods.join(", ")
                ),
            );
        }
        for check in &self.required_checks {
            fact(
                &mut held,
                &mut violations,
                live.required_checks.iter().any(|c| c == check),
                format!("required check: {check}"),
                format!(
                    "status check `{check}` is not required on the default branch — a PR \
                     can merge without it (apply spec/perimeter.ruleset.json)"
                ),
            );
        }
        match live.private_vulnerability_reporting {
            Some(true) => held.push("private vulnerability reporting: enabled".to_string()),
            Some(false) => violations.push(
                "private vulnerability reporting is DISABLED — SECURITY.md instructs \
                 reporters to use it (enable in the repository's security settings)"
                    .to_string(),
            ),
            None => violations.push(
                "private vulnerability reporting could not be READ — refused by name, \
                 never assumed enabled; verify the endpoint and the token's read access"
                    .to_string(),
            ),
        }

        if violations.is_empty() {
            Ok(held)
        } else {
            Err(violations)
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    fn applied_floor() -> LivePerimeter {
        LivePerimeter {
            deletion_blocked: true,
            force_push_blocked: true,
            required_approvals: Some(0),
            merge_methods: vec!["squash".to_string()],
            required_checks: Perimeter::declared()
                .required_checks
                .iter()
                .map(|s| s.to_string())
                .collect(),
            private_vulnerability_reporting: Some(true),
        }
    }

    /// The declaration DERIVES from the gate registry: the required checks are the
    /// every-change job plus each per-diff gate, by rendered job name — so a renamed
    /// gate moves this lock in the same diff.
    #[test]
    fn the_required_checks_come_from_the_registry() {
        let p = Perimeter::declared();
        assert_eq!(
            p.required_checks,
            vec!["fmt + clippy + test", "dogfood (changed lines)"]
        );
        assert_eq!(p.merge_methods, &["squash"]);
        assert_eq!(p.required_approvals, 0);
        // both renders carry the derived checks verbatim:
        assert!(p
            .render()
            .contains("fmt + clippy + test, dogfood (changed lines)"));
        assert!(p
            .ruleset_json()
            .contains("\"context\": \"dogfood (changed lines)\""));
    }

    /// An applied floor holds — and the held facts name everything defended. Extra
    /// live protections are NOT drift: stricter is never a lie.
    #[test]
    fn an_applied_floor_holds_and_stricter_is_not_drift() {
        let p = Perimeter::declared();
        let held = p.judge(&applied_floor()).expect("the applied floor holds");
        assert!(held.iter().any(|h| h == "force pushes: blocked"));
        assert!(held.iter().any(|h| h.contains("private vulnerability")));

        let mut stricter = applied_floor();
        stricter
            .required_checks
            .push("an extra check someone added".to_string());
        assert!(p.judge(&stricter).is_ok(), "extra checks are not drift");
    }

    /// Every departure refuses BY NAME: the never-applied state, a relaxed rule, a
    /// deadlocking approval count, a widened merge method, an unreadable
    /// vulnerability-reporting flag. The red gate is the reminder that never expires.
    #[test]
    fn every_departure_refuses_by_name() {
        let p = Perimeter::declared();

        // never applied: everything absent — the manual act is a red gate, not a memo.
        let bare = LivePerimeter::default();
        let violations = p.judge(&bare).unwrap_err();
        assert!(violations.iter().any(|v| v.contains("can be DELETED")));
        assert!(violations.iter().any(|v| v.contains("FORCE PUSHES")));
        assert!(violations
            .iter()
            .any(|v| v.contains("no pull-request rule")));
        assert!(violations
            .iter()
            .any(|v| v.contains("`fmt + clippy + test` is not required")));
        assert!(violations.iter().any(|v| v.contains("could not be READ")));

        // a deadlocking approval count is named as such, not accepted as stricter:
        let mut deadlock = applied_floor();
        deadlock.required_approvals = Some(1);
        let violations = p.judge(&deadlock).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("deadlocks a solo maintainer")));

        // a widened merge method refuses by the method's name:
        let mut widened = applied_floor();
        widened.merge_methods.push("merge".to_string());
        let violations = p.judge(&widened).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("merge method `merge`")));

        // ONE declared check missing while another is present refuses by the missing
        // check's name — the membership test must match the check itself, not merely
        // notice that some other context differs (`==` vs `!=` in the `any`).
        let mut partial = applied_floor();
        partial.required_checks = vec!["fmt + clippy + test".to_string()];
        let violations = p.judge(&partial).unwrap_err();
        assert!(
            violations
                .iter()
                .any(|v| v.contains("`dogfood (changed lines)` is not required")),
            "{violations:#?}"
        );
        assert!(
            !violations
                .iter()
                .any(|v| v.contains("`fmt + clippy + test` is not required")),
            "the present check must not be reported missing: {violations:#?}"
        );

        // vulnerability reporting explicitly off is its own violation:
        let mut off = applied_floor();
        off.private_vulnerability_reporting = Some(false);
        let violations = p.judge(&off).unwrap_err();
        assert!(violations.iter().any(|v| v.contains("DISABLED")));
    }

    /// The judge is DEAF TO NOTHING: every single-fact perturbation of the applied
    /// floor moves the verdict and names its fact — the derived battery that closes,
    /// as a class, the survivor species the source sweeps kept finding here (a
    /// membership check that stops distinguishing one element among present ones).
    #[test]
    fn the_judge_is_deaf_to_nothing() {
        let applied = applied_floor();
        let held = crate::discover::judgment::LiveDent::drill(
            |l| Perimeter::declared().judge(l),
            &applied,
            applied.dents(),
        )
        .expect("the perimeter judge distinguishes every fact");
        assert_eq!(held.len(), 9, "{held:#?}");
        assert!(held
            .iter()
            .any(|h| h == "sensitive to required check `dogfood (changed lines)` removed"));
    }

    /// The committed perimeter locks are FRESH — declaration, render, and ruleset move
    /// together or the build refuses.
    #[test]
    fn the_committed_perimeter_locks_are_fresh() {
        let p = Perimeter::declared();
        if let Err(stale) = spec_lock::check(&[p.lock(), p.ruleset_lock()]) {
            panic!(
                "the perimeter locks drifted: {}. Regenerate with \
                 `cargo run --example freeze_gates` and ratify the diff.",
                stale.join(", ")
            );
        }
    }

    /// The ruleset artifact is real JSON — the apply step must never discover a syntax
    /// error at the API.
    #[test]
    fn the_ruleset_artifact_parses_as_json() {
        let value: serde_json::Value =
            serde_json::from_str(&Perimeter::declared().ruleset_json()).expect("valid JSON");
        assert_eq!(value["target"], "branch");
        assert_eq!(value["rules"].as_array().expect("rules").len(), 4);
    }
}
