//!
//! substrate — GIT ITSELF IS A LOCK: the repository's tags and history carry meanings
//! this repo leans on everywhere and declared nowhere, until now.
//!
//! The perimeter brick covered GitHub-the-service; nothing covered git-the-data. Yet
//! the `mutants-green` tag is the incremental mutation gate's entire baseline, its
//! semantics living as prose and as behaviour inside `.github/mutants-gate.sh` — the
//! exact gate-defined-where-no-drift-gate-can-see-it shape the pipeline brick killed
//! for ci.yml. The CalVer release tags carry certification semantics `release.sh`
//! assumes. And "main is linear — every commit the squash of a gated PR" is a declared
//! perimeter rule about FUTURE merges but an unjudged observation about the history
//! that exists.
//!
//! Same pattern as the perimeter and the infra graph — declare, freeze, read the live
//! state back, refuse by name — with one delightful difference: judging git needs no
//! credentials and no third-party API. The repository is already on disk; the reads
//! are `git tag`, `git merge-base --is-ancestor`, `git rev-list` (a fetch of tags and
//! history in CI's shallow checkout, nothing more). This is the first world lock with
//! (almost) zero world-read cost.
//!
//!   - [`Substrate::declared`] names each tag or tag family and what it MEANS, plus
//!     the history law: after the declared epoch (the last pre-discipline merge
//!     commit), main is linear — squash-only, judged backward over the history that
//!     exists, not just promised forward by the perimeter.
//!   - [`Substrate::judge`] holds a [`LiveSubstrate`] — field reads
//!     `.github/substrate.sh` extracts via `examples/substrate.rs` — to those
//!     declarations: a required tag that does not exist, a meaning-carrying tag that
//!     is not on the certified line, a merge commit after the epoch, an unreadable
//!     read: each refuses by name.
//!   - `spec/substrate.spec` freezes the render (via `freeze_gates`, with the other
//!     repo-meta locks); the weekly `substrate (git drift)` world gate reads the live
//!     repository back. Like every world fact, it never feeds the countersign.
//!
//! One declaration is deliberately RED on arrival: the owed `v0.1.0` tag (the first
//! crates.io publish shipped from a branch-scoped session that could not push tags).
//! Declaring it required turns a roadmap memo into the reminder that never expires —
//! the perimeter's never-applied doctrine, applied to a tag.
//!
//! Honest frame: the judge sees names, ancestry, and parent counts — it cannot see
//! WHO advanced a tag or whether a run earned it (that is the workflow's countersign
//! discipline, pinned in `discover::gates`' probes). A meaning here is held evidence
//! that the shape is right, never proof the process was.

use std::path::PathBuf;

use spec_lock::Lock;

/// A tag (or trailing-`*` family) and the meaning the repo hangs on it.
pub struct TagLaw {
    /// An exact tag name, or a family as `<prefix>*` (trailing star only).
    pub pattern: &'static str,
    /// What the tag MEANS — the prose the tooling otherwise keeps in scripts.
    pub means: &'static str,
    /// Must at least one matching tag exist? A required-but-absent tag refuses —
    /// the reminder that never expires.
    pub required: bool,
}

impl TagLaw {
    /// Does a live tag name match this law? Exact match, or prefix for a family —
    /// `v2*` matches `v2026.07.07` and never `v0.1.0`.
    pub fn matches(&self, tag: &str) -> bool {
        match self.pattern.strip_suffix('*') {
            Some(prefix) => tag.starts_with(prefix),
            None => tag == self.pattern,
        }
    }
}

/// The declared git substrate — what `spec/substrate.spec` freezes.
pub struct Substrate {
    /// Tag meanings, each judged for existence (where required) and for sitting on
    /// the certified line.
    pub tags: Vec<TagLaw>,
    /// The last pre-discipline merge commit: after this, main is LINEAR (squash-only
    /// — the perimeter's forward rule, judged backward). Full sha, abbreviated in
    /// the render.
    pub linear_after: &'static str,
}

/// What the world reports — parsed from `git tag --list`, per-tag
/// `git merge-base --is-ancestor`, and `git rev-list --min-parents=2 --count` by
/// `examples/substrate.rs`. `None` = the read failed; refused by name, never assumed.
#[derive(Default)]
pub struct LiveSubstrate {
    /// Every tag name the repository carries.
    pub tags: Option<Vec<String>>,
    /// `(tag, is on the default-branch line)` for every tag.
    pub ancestry: Option<Vec<(String, bool)>>,
    /// Merge commits (parent count > 1) after the declared epoch.
    pub merges_after_epoch: Option<u64>,
}

impl Substrate {
    /// The declared substrate: the two tag families the machinery leans on, the one
    /// owed tag, and the linearity epoch.
    pub fn declared() -> Substrate {
        Substrate {
            tags: vec![
                TagLaw {
                    pattern: "mutants-green",
                    means: "the last default-branch tree whose full mutation sweep was \
                            clean — the incremental gate's diff baseline; advanced only \
                            by runs that earned it (per-merge incremental green, or the \
                            weekly countersign)",
                    required: true,
                },
                TagLaw {
                    pattern: "v2*",
                    means: "a certified release of the tree — CalVer, minted by \
                            release.sh on every countersign; the version claims a date, \
                            never a compatibility promise",
                    required: false,
                },
                TagLaw {
                    pattern: "v0.1.0",
                    means: "the first crates.io publish (2026-07-07) — OWED: the \
                            publishing session's push scope was branch-only, so the tag \
                            never left it; this refusal is the reminder that never \
                            expires",
                    required: true,
                },
            ],
            linear_after: "7ed05011728846db88030f2d0183fe35b0818cee",
        }
    }

    /// Hold the LIVE repository to the declared substrate. `Ok` carries the held
    /// facts; `Err` refuses each departure by name — the missing tag, the stray tag,
    /// the merge commit, the unreadable read.
    pub fn judge(&self, live: &LiveSubstrate) -> Result<Vec<String>, Vec<String>> {
        let mut held = Vec::new();
        let mut violations = Vec::new();

        match (&live.tags, &live.ancestry) {
            (Some(tags), Some(ancestry)) => {
                for law in &self.tags {
                    let matching: Vec<&String> = tags.iter().filter(|t| law.matches(t)).collect();
                    if matching.is_empty() {
                        if law.required {
                            violations.push(format!(
                                "tag `{}` does not exist — it means: {}",
                                law.pattern, law.means
                            ));
                        } else {
                            held.push(format!(
                                "tag family `{}`: nothing minted yet — nothing claimed",
                                law.pattern
                            ));
                        }
                        continue;
                    }
                    for tag in matching {
                        match ancestry.iter().find(|(t, _)| t == tag) {
                            None => violations.push(format!(
                                "the ancestry of tag `{tag}` could not be READ — \
                                 refused by name, never assumed on the certified line"
                            )),
                            Some((_, true)) => {
                                held.push(format!("tag `{tag}` is on the certified line"))
                            }
                            Some((_, false)) => violations.push(format!(
                                "tag `{tag}` is not on the default branch — its declared \
                                 meaning names a certified default-branch tree, and a \
                                 stray tag claims one it never earned"
                            )),
                        }
                    }
                }
            }
            _ => violations.push(
                "the tag list could not be READ — refused by name, never assumed".to_string(),
            ),
        }

        match live.merges_after_epoch {
            None => violations.push(format!(
                "the merge-commit count after {} could not be READ — refused by name, \
                 never assumed linear",
                &self.linear_after[..12]
            )),
            Some(0) => held.push(format!(
                "history after {} is linear — every commit a single-parent squash",
                &self.linear_after[..12]
            )),
            Some(n) => violations.push(format!(
                "{n} merge commit(s) landed on the default branch after {} — the \
                 declared history is squash-only (the perimeter enforces it forward; \
                 this judges it backward)",
                &self.linear_after[..12]
            )),
        }

        if violations.is_empty() {
            Ok(held)
        } else {
            Err(violations)
        }
    }

    /// The human-readable substrate — what `spec/substrate.spec` freezes.
    pub fn render(&self) -> String {
        let mut out = String::from(
            "# the git substrate, DECLARED — the tags and history laws the machinery leans\n\
             # on, previously living as prose and script behaviour. The weekly\n\
             # `substrate (git drift)` world gate reads the live repository back\n\
             # (`.github/substrate.sh` → `examples/substrate.rs`) and refuses each\n\
             # departure by name. A world fact never feeds the countersign. Regenerate\n\
             # with `cargo run --example freeze_gates`.\n\n",
        );
        for law in &self.tags {
            out.push_str(&format!(
                "tag {}{} — means: {}\n",
                law.pattern,
                if law.required { " (required)" } else { "" },
                law.means
            ));
        }
        out.push_str(&format!(
            "history: linear after {} (the last pre-discipline merge commit — \
             squash-only, judged backward)\n",
            self.linear_after
        ));
        out
    }

    /// `spec/substrate.spec` — the declaration, frozen.
    pub fn lock(&self) -> Lock {
        Lock {
            name: "substrate".to_string(),
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("spec")
                .join("substrate.spec"),
            live: self.render(),
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// A live state that satisfies every declaration.
    fn applied() -> LiveSubstrate {
        LiveSubstrate {
            tags: Some(vec![
                "mutants-green".to_string(),
                "v0.1.0".to_string(),
                "v2026.07.07".to_string(),
            ]),
            ancestry: Some(vec![
                ("mutants-green".to_string(), true),
                ("v0.1.0".to_string(), true),
                ("v2026.07.07".to_string(), true),
            ]),
            merges_after_epoch: Some(0),
        }
    }

    /// The pattern grammar, one arm at a time: an exact name matches only itself
    /// (never a prefix of something longer), a family matches by prefix (never a
    /// different family), and the declared laws use both forms.
    #[test]
    fn a_pattern_matches_exactly_or_by_declared_family() {
        let exact = TagLaw {
            pattern: "mutants-green",
            means: "",
            required: true,
        };
        assert!(exact.matches("mutants-green"));
        assert!(!exact.matches("mutants-green-2"));
        let family = TagLaw {
            pattern: "v2*",
            means: "",
            required: false,
        };
        assert!(family.matches("v2026.07.07"));
        assert!(!family.matches("v0.1.0"));
        let declared = Substrate::declared();
        assert_eq!(declared.tags.len(), 3);
        assert!(declared.tags.iter().any(|l| l.pattern == "mutants-green"));
        assert!(declared.linear_after.starts_with("7ed05011"));
    }

    /// The applied substrate holds, and the held facts name what is defended —
    /// including the vacuous family (nothing minted, nothing claimed) when no
    /// release tag exists yet.
    #[test]
    fn an_applied_substrate_holds() {
        let s = Substrate::declared();
        let held = s.judge(&applied()).expect("the applied substrate holds");
        assert!(held
            .iter()
            .any(|h| h == "tag `mutants-green` is on the certified line"));
        assert!(held.iter().any(|h| h.contains("history after 7ed05011")));

        let mut unminted = applied();
        unminted
            .tags
            .as_mut()
            .unwrap()
            .retain(|t| t != "v2026.07.07");
        unminted
            .ancestry
            .as_mut()
            .unwrap()
            .retain(|(t, _)| t != "v2026.07.07");
        let held = s.judge(&unminted).expect("an empty family claims nothing");
        assert!(held
            .iter()
            .any(|h| h == "tag family `v2*`: nothing minted yet — nothing claimed"));
    }

    /// Every departure refuses by name: the owed tag absent, a stray tag off the
    /// certified line (named alone — the on-line tags stay unaccused), a merge commit
    /// after the epoch, and each unreadable read separately.
    #[test]
    fn every_departure_refuses_by_name() {
        let s = Substrate::declared();

        // the owed v0.1.0: required-but-absent is a red fact, not a memo.
        let mut owed = applied();
        owed.tags.as_mut().unwrap().retain(|t| t != "v0.1.0");
        owed.ancestry
            .as_mut()
            .unwrap()
            .retain(|(t, _)| t != "v0.1.0");
        let violations = s.judge(&owed).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("tag `v0.1.0` does not exist")));
        assert!(
            !violations.iter().any(|v| v.contains("mutants-green")),
            "the present tag must not be accused: {violations:#?}"
        );

        // ONE stray release tag among on-line ones refuses by ITS name only.
        let mut stray = applied();
        stray.tags.as_mut().unwrap().push("v2026.07.08".to_string());
        stray
            .ancestry
            .as_mut()
            .unwrap()
            .push(("v2026.07.08".to_string(), false));
        let violations = s.judge(&stray).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("tag `v2026.07.08` is not on the default branch")));
        assert!(
            !violations.iter().any(|v| v.contains("`v2026.07.07`")),
            "the on-line tag must not be accused: {violations:#?}"
        );

        // a matching tag with no ancestry line is an unreadable read, not a pass.
        let mut unread = applied();
        unread
            .ancestry
            .as_mut()
            .unwrap()
            .retain(|(t, _)| t != "v0.1.0");
        let violations = s.judge(&unread).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("ancestry of tag `v0.1.0` could not be READ")));

        // a merge commit after the epoch names the count and the rule.
        let mut merged = applied();
        merged.merges_after_epoch = Some(2);
        let violations = s.judge(&merged).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("2 merge commit(s)") && v.contains("squash-only")));

        // nothing readable at all: the tag list and the merge count refuse separately.
        let violations = s.judge(&LiveSubstrate::default()).unwrap_err();
        assert!(violations
            .iter()
            .any(|v| v.contains("tag list could not be READ")));
        assert!(violations
            .iter()
            .any(|v| v.contains("merge-commit count") && v.contains("could not be READ")));
    }

    /// The committed substrate lock is FRESH — declaration and spec move together or
    /// the build refuses.
    #[test]
    fn the_committed_substrate_lock_is_fresh() {
        let s = Substrate::declared();
        if let Err(stale) = spec_lock::check(&[s.lock()]) {
            panic!(
                "the substrate lock drifted: {}. Regenerate with \
                 `cargo run --example freeze_gates` and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
