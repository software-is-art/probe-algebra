//! perimeter — the world gate's envelope: read the LIVE repository settings back and
//! hold them to the declared floor (`discover::perimeter`).
//!
//!     .github/perimeter.sh                    # CI: fetch the three payloads, judge
//!     cargo run --example perimeter -- judge <rules.json> <repo.json> <pvr.json>
//!     cargo run --example perimeter -- ruleset # print the apply-able artifact
//!
//! The EXTRACTION lives here (field reads over the GitHub API payloads — dev-dependency
//! JSON, never shipped in the library); the JUDGMENT lives in `Perimeter::judge`, where
//! the probes reach it. Exit 1 on drift, with every violation named.

use boundary_spec::discover::perimeter::{LivePerimeter, Perimeter};

/// Extract the judged facts from the three API payloads: the active branch rules
/// (`GET /repos/{o}/{r}/rules/branches/<default>`), the repository object (merge-method
/// flags, for rules that predate `allowed_merge_methods`, and the auto-merge flag), and
/// the private vulnerability reporting flag. Unreadable payloads extract to the ABSENT
/// state — which the judge refuses by name, never assumes.
fn extract(rules_json: &str, repo_json: &str, pvr_json: &str) -> LivePerimeter {
    let rules: Vec<serde_json::Value> = serde_json::from_str(rules_json).unwrap_or_default();
    let repo: serde_json::Value = serde_json::from_str(repo_json).unwrap_or_default();
    let pvr: serde_json::Value = serde_json::from_str(pvr_json).unwrap_or_default();

    let has = |t: &str| rules.iter().any(|r| r["type"] == t);
    let pull_request = rules.iter().find(|r| r["type"] == "pull_request");
    let required_approvals =
        pull_request.and_then(|r| r["parameters"]["required_approving_review_count"].as_u64());
    // merge methods: the pull-request rule's `allowed_merge_methods` where present;
    // otherwise the repository's allow_* flags (older rules / classic protection).
    let merge_methods: Vec<String> =
        match pull_request.and_then(|r| r["parameters"]["allowed_merge_methods"].as_array()) {
            Some(methods) => methods
                .iter()
                .filter_map(|m| m.as_str())
                .map(str::to_lowercase)
                .collect(),
            None => [
                ("allow_merge_commit", "merge"),
                ("allow_squash_merge", "squash"),
                ("allow_rebase_merge", "rebase"),
            ]
            .iter()
            .filter(|(flag, _)| repo[*flag].as_bool() == Some(true))
            .map(|(_, method)| method.to_string())
            .collect(),
        };
    let required_checks: Vec<String> = rules
        .iter()
        .filter(|r| r["type"] == "required_status_checks")
        .filter_map(|r| r["parameters"]["required_status_checks"].as_array())
        .flatten()
        .filter_map(|c| c["context"].as_str())
        .map(str::to_string)
        .collect();

    LivePerimeter {
        deletion_blocked: has("deletion"),
        force_push_blocked: has("non_fast_forward"),
        required_approvals,
        merge_methods,
        required_checks,
        private_vulnerability_reporting: pvr["enabled"].as_bool(),
        auto_merge: repo["allow_auto_merge"].as_bool(),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("ruleset") => print!("{}", Perimeter::declared().ruleset_json()),
        Some("judge") if args.len() == 4 => {
            let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
            let live = extract(&read(&args[1]), &read(&args[2]), &read(&args[3]));
            match Perimeter::declared().judge(&live) {
                Ok(held) => {
                    println!("perimeter holds — the declared floor is live:");
                    for fact in held {
                        println!("- {fact}");
                    }
                }
                Err(violations) => {
                    eprintln!("perimeter DRIFTED from the declared floor:");
                    for v in violations {
                        eprintln!("- {v}");
                    }
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "usage: perimeter ruleset | perimeter judge <rules.json> <repo.json> <pvr.json>"
            );
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// EXTRACTION over real payload shapes: a fully-applied perimeter round-trips into
    /// the judged facts, `allowed_merge_methods` wins over the repo flags when present,
    /// the flags carry rules that predate it, and garbage payloads extract to the
    /// absent state the judge refuses.
    #[test]
    fn the_api_payloads_extract_into_the_judged_facts() {
        let rules = r#"[
            {"type":"deletion"},
            {"type":"non_fast_forward"},
            {"type":"pull_request","parameters":{"required_approving_review_count":0,"allowed_merge_methods":["squash"]}},
            {"type":"required_status_checks","parameters":{"required_status_checks":[{"context":"fmt + clippy + test"},{"context":"dogfood (changed lines)"}]}}
        ]"#;
        let repo = r#"{"allow_merge_commit":true,"allow_squash_merge":true,"allow_rebase_merge":false,"allow_auto_merge":true}"#;
        let live = extract(rules, repo, r#"{"enabled":true}"#);
        assert!(live.deletion_blocked && live.force_push_blocked);
        assert_eq!(live.required_approvals, Some(0));
        // the rule's allowed_merge_methods wins — the repo's wider flags are ignored:
        assert_eq!(live.merge_methods, vec!["squash"]);
        assert_eq!(live.required_checks.len(), 2);
        assert_eq!(live.private_vulnerability_reporting, Some(true));
        assert_eq!(live.auto_merge, Some(true));
        assert!(Perimeter::declared().judge(&live).is_ok());

        // no allowed_merge_methods on the rule: the repo flags carry the answer.
        let old_rule =
            r#"[{"type":"pull_request","parameters":{"required_approving_review_count":0}}]"#;
        let live = extract(old_rule, repo, "{}");
        assert_eq!(live.merge_methods, vec!["merge", "squash"]);
        assert_eq!(live.private_vulnerability_reporting, None);
        assert_eq!(live.auto_merge, Some(true));

        // garbage extracts to absence — and absence refuses downstream, never passes.
        let live = extract("not json", "also not", "");
        assert!(!live.deletion_blocked);
        assert_eq!(live.required_approvals, None);
        assert_eq!(live.auto_merge, None);
        assert!(Perimeter::declared().judge(&live).is_err());
    }
}
