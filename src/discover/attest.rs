//! attest — the sampled countersign's subject: the sweep writes a COMMITTED TRANSCRIPT
//! (tree hash, toolchain, every site's verdict and the covering tests it was judged
//! by), and `schemata verify` audits it by re-judging a random sample — derivation
//! local, countersign sampled, the full sweep as the fallback on any mismatch and the
//! weekly shards as the from-scratch backstop. The trust analysis this implements: a
//! working session must not countersign itself (local green lied twice the day this was
//! designed), but it CAN carry the derivation, because the auditor's sample is drawn
//! from entropy after the transcript is fixed — a false `killed` cannot be placed where
//! the audit won't look.
//!
//! Disclosed edges: timeouts are machine-relative, so a slow-box detection can read as
//! a fast-box survivor — a FALSE disagreement, which costs a redundant full sweep and
//! never a false green; and the transcript lives in `attest/`, excluded from the tree
//! hash, because the attestation describes the tree and cannot be part of it.

use std::path::{Path, PathBuf};

/// One judged site in a sweep transcript: its census name, the verdict the sweep
/// reached, and the covering tests it was judged BY — carried so a countersign can
/// re-run exactly the judgment being audited. The lie direction that matters is a
/// false `killed` (a hidden degree of freedom), and it cannot be fabricated from
/// non-covering tests: tests that never reach the flipped guard pass under the mutant,
/// the sampled re-judgment observes a survivor, and the disagreement fires.
#[derive(Debug)]
pub struct SiteVerdict {
    pub site: String,
    pub verdict: String,
    pub tests: Vec<String>,
}

/// The committed sweep TRANSCRIPT — the sampled countersign's subject: tree hash,
/// toolchain, baseline seconds (the timeout's seed), and every site's verdict. It lives
/// in `attest/`, which the tree hash EXCLUDES: the attestation describes the tree, so
/// it cannot be part of the tree it describes (the verdict-store scope rule, promoted
/// to committed form). Deterministic render, tab-separated fields — census sites carry
/// spaces and colons, never tabs.
#[derive(Debug)]
pub struct Transcript {
    pub tree: String,
    pub toolchain: String,
    pub baseline_secs: u64,
    pub sites: Vec<SiteVerdict>,
}

#[crate::mutate("attest")]
impl Transcript {
    /// Where the committed transcript lives.
    pub fn location(crate_root: &Path) -> PathBuf {
        crate_root.join("attest/sweep.transcript")
    }

    /// The deterministic text form — what gets committed and what the countersign
    /// parses back.
    pub fn render(&self) -> String {
        let mut out = format!(
            "# sweep transcript — the sampled countersign's subject; regenerate by \
             running the sweep.\ntree {}\ntoolchain {}\nbaseline {}\n",
            self.tree, self.toolchain, self.baseline_secs
        );
        for entry in &self.sites {
            out.push_str(&format!(
                "{}\t{}\t{}\n",
                entry.verdict,
                entry.site,
                entry.tests.join(",")
            ));
        }
        out
    }

    /// Parse a committed transcript; refusals are named — a malformed line means a
    /// hand touched the record, and a hand-touched attestation must not be audited,
    /// it must be re-derived.
    pub fn parse(text: &str) -> Result<Transcript, String> {
        let mut tree = None;
        let mut toolchain = None;
        let mut baseline = None;
        let mut sites = Vec::new();
        for (n, line) in text.lines().enumerate() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            if let Some(rest) = line.strip_prefix("tree ") {
                tree = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("toolchain ") {
                toolchain = Some(rest.trim().to_string());
            } else if let Some(rest) = line.strip_prefix("baseline ") {
                baseline = rest.trim().parse::<u64>().ok();
            } else {
                let mut fields = line.split('\t');
                let (verdict, site, tests) = (fields.next(), fields.next(), fields.next());
                let (Some(verdict), Some(site), Some(tests)) = (verdict, site, tests) else {
                    return Err(format!(
                        "attest: line {} is not `<verdict>\\t<site>\\t<tests>` — a \
                         hand-touched transcript is re-derived, never audited: `{line}`",
                        n + 1
                    ));
                };
                sites.push(SiteVerdict {
                    site: site.to_string(),
                    verdict: verdict.to_string(),
                    tests: tests
                        .split(',')
                        .filter(|t| !t.is_empty())
                        .map(str::to_string)
                        .collect(),
                });
            }
        }
        match (tree, toolchain, baseline) {
            (Some(tree), Some(toolchain), Some(baseline_secs)) => Ok(Transcript {
                tree,
                toolchain,
                baseline_secs,
                sites,
            }),
            _ => Err(
                "attest: the transcript header must carry `tree`, `toolchain`, and \
                 `baseline` lines — an incomplete claim audits nothing"
                    .to_string(),
            ),
        }
    }

    /// A deterministic K-sample of site indices for a given seed — the countersign
    /// draws its seed from entropy (unpredictability is the audit's teeth: which
    /// sites get re-judged cannot be known when the transcript is written), but the
    /// walk from seed to sample is a pure function, so it pins here.
    pub fn sample(&self, k: usize, seed: u64) -> Vec<&SiteVerdict> {
        let len = self.sites.len();
        if len == 0 {
            return Vec::new();
        }
        let want = k.min(len);
        let mut picked = std::collections::BTreeSet::new();
        let mut x = seed | 1;
        while picked.len() < want {
            x = x
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            picked.insert((x % len as u64) as usize);
        }
        picked.into_iter().map(|i| &self.sites[i]).collect()
    }
}

#[cfg(test)]
mod probes {
    use super::{SiteVerdict, Transcript};

    fn transcript() -> Transcript {
        Transcript {
            tree: "00aa11bb22cc33dd".to_string(),
            toolchain: "1.94.1".to_string(),
            baseline_secs: 7,
            sites: vec![
                SiteVerdict {
                    site: "a::b::f:0: == -> !=".to_string(),
                    verdict: "killed".to_string(),
                    tests: vec!["a::probes::one".to_string(), "a::probes::two".to_string()],
                },
                SiteVerdict {
                    site: "a::b::g:deaf -> None".to_string(),
                    verdict: "SURVIVED".to_string(),
                    tests: vec![],
                },
                SiteVerdict {
                    site: "a::b::h:1: && -> ||".to_string(),
                    verdict: "killed".to_string(),
                    tests: vec!["a::probes::three".to_string()],
                },
            ],
        }
    }

    /// THE HEADLINE: the transcript round-trips byte-losslessly — what the sweep
    /// attests is exactly what the countersign audits, spaces-and-colons site names,
    /// empty covering sets, and all.
    #[test]
    fn the_transcript_round_trips() {
        let t = transcript();
        let parsed = Transcript::parse(&t.render()).expect("parses");
        assert_eq!(parsed.tree, t.tree);
        assert_eq!(parsed.toolchain, t.toolchain);
        assert_eq!(parsed.baseline_secs, t.baseline_secs);
        assert_eq!(parsed.sites.len(), t.sites.len());
        for (a, b) in parsed.sites.iter().zip(t.sites.iter()) {
            assert_eq!(a.site, b.site);
            assert_eq!(a.verdict, b.verdict);
            assert_eq!(a.tests, b.tests);
        }
        assert_eq!(parsed.render(), t.render(), "render is a fixed point");
    }

    /// Refusals are named: a hand-touched line and a headless transcript both refuse —
    /// an attestation that cannot be read is re-derived, never audited.
    #[test]
    fn a_hand_touched_transcript_refuses() {
        let refusal =
            Transcript::parse("tree x\ntoolchain y\nbaseline 3\nnot a line\n").unwrap_err();
        assert!(refusal.contains("line 4"), "{refusal}");
        let headless = Transcript::parse("killed\tsite\ttests\n").unwrap_err();
        assert!(headless.contains("header"), "{headless}");
    }

    /// The seed-to-sample walk is a pure function: same seed, same sample; distinct
    /// entries; bounded by the roster; and the whole roster arrives when K exceeds it.
    #[test]
    fn the_sample_is_deterministic_distinct_and_bounded() {
        let t = transcript();
        let one: Vec<&str> = t.sample(2, 42).iter().map(|s| s.site.as_str()).collect();
        let two: Vec<&str> = t.sample(2, 42).iter().map(|s| s.site.as_str()).collect();
        assert_eq!(one, two, "same seed, same sample");
        assert_eq!(one.len(), 2);
        let all = t.sample(50, 7);
        assert_eq!(all.len(), 3, "K clamps to the roster");
        let different: Vec<&str> = t.sample(2, 43).iter().map(|s| s.site.as_str()).collect();
        let _ = different; // seeds may collide on a 3-site roster; determinism is the pin
    }
}
