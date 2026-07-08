//! substrate — the git world gate's envelope: read the LIVE repository's tags and
//! history back and hold them to the declared substrate (`discover::substrate`).
//!
//!     .github/substrate.sh              # CI: run the git + index reads, judge
//!     cargo run --example substrate -- judge <tags.txt> <ancestry.txt> <merges.txt> <index.txt>
//!     cargo run --example substrate -- epoch    # print the declared epoch sha
//!     cargo run --example substrate -- crate    # print the publish-marker crate
//!
//! The EXTRACTION lives here (line parsing over `git tag --list`, per-tag
//! `git merge-base --is-ancestor` verdicts, a `git rev-list --min-parents=2 --count`,
//! and the crate's sparse-index body from index.crates.io); the JUDGMENT lives in
//! `Substrate::judge`, where the probes reach it. The `epoch` and `crate`
//! subcommands exist so the shell script asks the DECLARATION for those facts
//! instead of restating them. Exit 1 on drift, every violation named.

use boundary_spec::discover::substrate::{LiveSubstrate, Substrate};

/// Extract the judged facts from the four reads. Empty or malformed payloads extract
/// to the ABSENT state — which the judge refuses by name, never assumes.
/// `ancestry.txt` is one `<tag> on-main|stray` line per tag; `index.txt` is the raw
/// sparse-index body (one JSON object per line, each carrying `vers`); any other
/// shape on a line poisons that whole read (absent, not guessed).
fn extract(tags: &str, ancestry: &str, merges: &str, index: &str) -> LiveSubstrate {
    let tag_list: Vec<String> = tags
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    let ancestry_list: Option<Vec<(String, bool)>> = ancestry
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|line| match line.rsplit_once(' ') {
            Some((tag, "on-main")) => Some((tag.to_string(), true)),
            Some((tag, "stray")) => Some((tag.to_string(), false)),
            _ => None,
        })
        .collect();
    let published: Option<Vec<String>> = index
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|v| v["vers"].as_str().map(str::to_string))
        })
        .collect();
    LiveSubstrate {
        tags: if tag_list.is_empty() {
            None
        } else {
            Some(tag_list)
        },
        ancestry: ancestry_list.filter(|l| !l.is_empty()),
        merges_after_epoch: merges.trim().parse().ok(),
        published_versions: published.filter(|p| !p.is_empty()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("epoch") => println!("{}", Substrate::declared().linear_after),
        Some("crate") => println!("{}", Substrate::declared().publish_marker_crate),
        Some("judge") if args.len() == 5 => {
            let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
            let live = extract(
                &read(&args[1]),
                &read(&args[2]),
                &read(&args[3]),
                &read(&args[4]),
            );
            match Substrate::declared().judge(&live) {
                Ok(held) => {
                    println!("substrate holds — the declared meanings are live:");
                    for fact in held {
                        println!("- {fact}");
                    }
                }
                Err(violations) => {
                    eprintln!("substrate DRIFTED from its declaration:");
                    for v in violations {
                        eprintln!("- {v}");
                    }
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!(
                "usage: substrate epoch | substrate crate | substrate judge \
                 <tags.txt> <ancestry.txt> <merges.txt> <index.txt>"
            );
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// EXTRACTION over the real read shapes: the healthy triple round-trips into held
    /// facts; each empty or malformed read extracts to the absent state the judge
    /// refuses (a half-parsed ancestry is absent, never guessed); and the verdict
    /// words parse one arm at a time.
    #[test]
    fn the_git_reads_extract_into_the_judged_facts() {
        let tags = "mutants-green\nv0.1.0\nv2026.07.07\n";
        let ancestry = "mutants-green on-main\nv0.1.0 on-main\nv2026.07.07 on-main\n";
        let index = r#"{"name":"boundary-spec","vers":"0.1.0","deps":[]}"#;
        let live = extract(tags, ancestry, "0\n", index);
        assert_eq!(live.tags.as_ref().map(Vec::len), Some(3));
        assert_eq!(live.merges_after_epoch, Some(0));
        assert_eq!(live.published_versions, Some(vec!["0.1.0".to_string()]));
        assert!(Substrate::declared().judge(&live).is_ok());

        // the two verdict words parse to their own booleans — a stray is false.
        let live = extract(
            tags,
            "mutants-green on-main\nv0.1.0 stray\nv2026.07.07 on-main\n",
            "0",
            index,
        );
        let ancestry_parsed = live.ancestry.expect("parsed");
        assert_eq!(
            ancestry_parsed
                .iter()
                .find(|(t, _)| t == "v0.1.0")
                .map(|(_, a)| *a),
            Some(false)
        );
        assert_eq!(
            ancestry_parsed
                .iter()
                .find(|(t, _)| t == "mutants-green")
                .map(|(_, a)| *a),
            Some(true)
        );

        // a tag name carrying spaces still parses: the verdict is the LAST word.
        let live = extract("odd tag\n", "odd tag on-main\n", "0", index);
        assert_eq!(live.ancestry, Some(vec![("odd tag".to_string(), true)]));

        // a multi-version index body extracts every published version, in order.
        let two = "{\"vers\":\"0.1.0\"}\n{\"vers\":\"0.2.0\"}\n";
        let live = extract(tags, ancestry, "0", two);
        assert_eq!(
            live.published_versions,
            Some(vec!["0.1.0".to_string(), "0.2.0".to_string()])
        );

        // empty and malformed reads are ABSENT, and absence refuses downstream.
        let live = extract("", "", "", "");
        assert!(live.tags.is_none());
        assert!(live.ancestry.is_none());
        assert_eq!(live.merges_after_epoch, None);
        assert_eq!(live.published_versions, None);
        assert!(Substrate::declared().judge(&live).is_err());
        let poisoned = extract(
            tags,
            "mutants-green maybe\n",
            "not a number",
            "{\"vers\":\"0.1.0\"}\nnot json\n",
        );
        assert!(poisoned.ancestry.is_none());
        assert_eq!(poisoned.merges_after_epoch, None);
        assert_eq!(
            poisoned.published_versions, None,
            "one bad line poisons the read"
        );
    }
}
