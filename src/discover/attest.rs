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
/// reached, the covering tests it was judged BY — carried so a countersign can
/// re-run exactly the judgment being audited — and the EVIDENCE key: a fingerprint
/// of the site's enclosing source item plus its covering tests' enclosing items,
/// the content this verdict is evidence ABOUT. An incremental re-judgment may carry
/// a verdict forward exactly when the evidence key still matches; an empty key
/// carries nothing. The lie direction that matters is a false `killed` (a hidden
/// degree of freedom), and it cannot be fabricated from non-covering tests: tests
/// that never reach the flipped guard pass under the mutant, the sampled
/// re-judgment observes a survivor, and the disagreement fires.
#[derive(Debug)]
pub struct SiteVerdict {
    pub site: String,
    pub verdict: String,
    pub tests: Vec<String>,
    pub evidence: String,
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
                "{}\t{}\t{}\t{}\n",
                entry.verdict,
                entry.site,
                entry.tests.join(","),
                entry.evidence
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
                // the fourth field arrived with incremental re-judgment; a transcript
                // from before it carries no evidence keys, so nothing is reusable and
                // the one honest transition is a full sweep.
                let evidence = fields.next().unwrap_or("").to_string();
                sites.push(SiteVerdict {
                    site: site.to_string(),
                    verdict: verdict.to_string(),
                    tests: tests
                        .split(',')
                        .filter(|t| !t.is_empty())
                        .map(str::to_string)
                        .collect(),
                    evidence,
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

/// The evidence resolver for incremental re-judgment: a census site or a covering
/// test resolves to the module FILE it lives in, and a site's evidence key folds
/// those bytes — its own module plus every covering test's module, names mixed in —
/// into a fingerprint of the content this verdict is evidence ABOUT. Module grain,
/// deliberately: item grain is the named sharpening, but it must wait for total
/// impl addressing (a missed generic impl would reuse a verdict whose code moved —
/// the one failure direction this design refuses). The standing discipline is
/// fail-open-into-judgment: anything unresolvable yields the empty key, and an
/// empty key is never carried — reuse is earned, judgment is the default.
pub struct Evidence {
    root: PathBuf,
    files: std::collections::BTreeMap<PathBuf, Option<String>>,
}

#[crate::mutate]
impl Evidence {
    /// A resolver rooted at a crate: names resolve under `<root>/src`, and each
    /// file's text is read once per resolver — one run, one reading of the tree.
    pub fn under(root: &Path) -> Evidence {
        Evidence {
            root: root.to_path_buf(),
            files: std::collections::BTreeMap::new(),
        }
    }

    /// The site's evidence key: FNV-64 (the verdict store's fold) over the module
    /// file the site resolves to and the module file of every covering test, names
    /// mixed in, tests in sorted order. Empty when anything fails to resolve.
    pub fn of(&mut self, site: &str, tests: &[String]) -> String {
        let Some(label) = Evidence::label(site) else {
            return String::new();
        };
        let Some(own) = self.module_text(&label, true) else {
            return String::new();
        };
        let mut parts = vec![(label, own)];
        let mut sorted: Vec<&String> = tests.iter().collect();
        sorted.sort();
        for test in sorted {
            let Some(text) = self.module_text(test, false) else {
                return String::new();
            };
            parts.push((test.clone(), text));
        }
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        };
        for (name, text) in &parts {
            eat(name.as_bytes());
            eat(text.as_bytes());
        }
        format!("{hash:016x}")
    }

    /// Carry a prior verdict forward — exactly when the prior transcript names this
    /// site with the SAME non-empty evidence key and the verdict is not a timeout.
    /// A timeout is load-sensitive: a slow machine's detection is re-judged, never
    /// inherited, so no verdict ever depends on a past machine's mood.
    pub fn carry<'t>(prior: &'t Transcript, site: &str, evidence: &str) -> Option<&'t str> {
        if evidence.is_empty() {
            return None;
        }
        prior
            .sites
            .iter()
            .find(|s| s.site == site && s.evidence == evidence && s.verdict != "timeout")
            .map(|s| s.verdict.as_str())
    }

    /// The census name with its mutant suffix stripped — `a::b::f:0: == -> !=` and
    /// `a::b::g:deaf -> None` both yield the label path the site lives at. A name
    /// with no recognisable suffix is not a site and resolves to nothing.
    fn label(site: &str) -> Option<String> {
        // the full form `:deaf -> ` — a bare `:deaf` would match inside a path
        // whose item is itself named `deaf_…` (observed: `mutation::deaf_battery`
        // truncated to a label that resolves nowhere).
        if let Some(i) = site.find(":deaf -> ") {
            return Some(site[..i].to_string());
        }
        let mut from = 0;
        while let Some(j) = site[from..].find(':').map(|j| from + j) {
            let rest = &site[j + 1..];
            let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
            if digits > 0 && rest.as_bytes().get(digits) == Some(&b':') {
                return Some(site[..j].to_string());
            }
            from = j + 1;
        }
        None
    }

    /// Resolve a `::`-path to the module file it lives in: segments walk
    /// directories under `src/` until one names a `.rs` file — or until a segment
    /// that is neither a file nor a directory, which names an item (an inline mod
    /// included) in the CURRENT module's own file: `mod.rs` here, `lib.rs` at the
    /// crate root. At least one segment must remain beyond the file (the item
    /// chain). Sites are crate-rooted (the leading crate name is dropped);
    /// nextest's test names are not. A name that resolves to nothing yields None.
    fn module_text(&mut self, name: &str, crate_rooted: bool) -> Option<String> {
        let mut segs: Vec<&str> = name.split("::").collect();
        if crate_rooted && !segs.is_empty() {
            segs.remove(0);
        }
        let src = self.root.join("src");
        let mut dir = src.clone();
        let mut idx = 0;
        let mut own_item: Option<&str> = None;
        let file = loop {
            let seg = *segs.get(idx)?;
            let as_file = dir.join(format!("{seg}.rs"));
            if as_file.is_file() {
                idx += 1;
                break as_file;
            }
            let as_dir = dir.join(seg);
            if as_dir.is_dir() {
                dir = as_dir;
                idx += 1;
                continue;
            }
            let own = if dir == src {
                dir.join("lib.rs")
            } else {
                dir.join("mod.rs")
            };
            if own.is_file() {
                // the segment must actually appear in the claimed file — without
                // this, every unresolvable name would silently key itself to a
                // parent file it does not live in.
                own_item = Some(seg);
                break own;
            }
            return None;
        };
        segs.get(idx)?;
        if !self.files.contains_key(&file) {
            let text = std::fs::read_to_string(&file).ok();
            self.files.insert(file.clone(), text);
        }
        let text = self.files.get(&file).and_then(|t| t.clone())?;
        match own_item {
            Some(seg) if !text.contains(seg) => None,
            _ => Some(text),
        }
    }
}

#[cfg(test)]
mod probes {
    use super::{Evidence, SiteVerdict, Transcript};

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
                    evidence: "1111aaaa2222bbbb".to_string(),
                },
                SiteVerdict {
                    site: "a::b::g:deaf -> None".to_string(),
                    verdict: "SURVIVED".to_string(),
                    tests: vec![],
                    evidence: String::new(),
                },
                SiteVerdict {
                    site: "a::b::h:1: && -> ||".to_string(),
                    verdict: "killed".to_string(),
                    tests: vec!["a::probes::three".to_string()],
                    evidence: "3333cccc4444dddd".to_string(),
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
            assert_eq!(a.evidence, b.evidence);
        }
        assert_eq!(parsed.render(), t.render(), "render is a fixed point");
        // a transcript from before evidence keys still parses — its sites carry the
        // empty key, so nothing is reusable and the one honest transition is a sweep.
        let elder =
            Transcript::parse("tree x\ntoolchain y\nbaseline 3\nkilled\ts:0: == -> !=\ta::t\n")
                .expect("the elder format parses");
        assert_eq!(elder.sites[0].evidence, "");
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

    /// The evidence discipline, pinned: the key derives from the site's module and
    /// its covering tests' modules — moving any of those bytes moves the key, the
    /// suffix and the test order are not evidence, an unresolvable name yields the
    /// empty key — and carry honours exactly a matching non-empty key, never a
    /// timeout (load-sensitive detections are re-judged, not inherited).
    #[test]
    fn evidence_keys_move_with_their_modules_and_timeouts_never_carry() {
        let root = std::env::temp_dir().join(format!("attest-evidence-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).expect("temp src");
        std::fs::write(
            root.join("src/foo.rs"),
            "fn bar() {}\nmod probes { fn t1() {} }\n",
        )
        .expect("foo.rs");
        let covering = vec!["foo::probes::t1".to_string()];
        let one = Evidence::under(&root).of("crate_x::foo::bar:0: == -> !=", &covering);
        assert_eq!(one.len(), 16, "a key is minted: `{one}`");
        assert_eq!(
            Evidence::under(&root).of("crate_x::foo::bar:0: == -> !=", &covering),
            one,
            "same bytes, same key"
        );
        assert_eq!(
            Evidence::under(&root).of("crate_x::foo::bar:deaf -> None", &covering),
            one,
            "the mutant suffix is not evidence"
        );
        // an item whose own name starts with `deaf` must not be mistaken for the
        // suffix: the label cut is the full `:deaf -> ` form.
        std::fs::write(
            root.join("src/deafen.rs"),
            "fn deaf_battery() {}\nmod probes { fn t9() {} }\n",
        )
        .expect("deafen.rs");
        let named = Evidence::under(&root).of("crate_x::deafen::deaf_battery:0: == -> !=", &[]);
        assert_eq!(named.len(), 16, "deaf_-named items resolve: `{named}`");
        std::fs::write(
            root.join("src/foo.rs"),
            "fn bar() { let _ = 1; }\nmod probes { fn t1() {} }\n",
        )
        .expect("foo.rs moves");
        let mut moved = Evidence::under(&root);
        let two = moved.of("crate_x::foo::bar:0: == -> !=", &covering);
        assert_ne!(two, one, "the module moved, the key moves");
        assert_ne!(
            moved.of("crate_x::foo::bar:0: == -> !=", &[]),
            two,
            "the covering set is evidence too"
        );
        // an inline mod in the parent's own file resolves to that file: mod.rs
        // inside a directory, lib.rs at the crate root.
        std::fs::create_dir_all(root.join("src/deep")).expect("deep dir");
        std::fs::write(
            root.join("src/deep/mod.rs"),
            "mod tests { fn t2() {} }
",
        )
        .expect("mod.rs");
        std::fs::write(
            root.join("src/lib.rs"),
            "mod tests { fn t3() {} }
",
        )
        .expect("lib.rs");
        let dir_key = moved.of(
            "crate_x::foo::bar:0: == -> !=",
            &["deep::tests::t2".to_string()],
        );
        assert_eq!(dir_key.len(), 16, "mod.rs resolves: `{dir_key}`");
        let root_key = moved.of("crate_x::foo::bar:0: == -> !=", &["tests::t3".to_string()]);
        assert_eq!(root_key.len(), 16, "lib.rs resolves: `{root_key}`");
        assert_ne!(dir_key, root_key, "different coverers, different keys");
        assert_eq!(moved.of("crate_x::nowhere::bar:0: == -> !=", &[]), "");
        assert_eq!(moved.of("crate_x::foo::bar", &[]), "", "no suffix, no site");
        assert_eq!(
            moved.of("crate_x::foo:0: == -> !=", &[]),
            "",
            "a file with no item chain is not a site"
        );
        let prior = Transcript {
            tree: "t".to_string(),
            toolchain: "tc".to_string(),
            baseline_secs: 1,
            sites: vec![
                SiteVerdict {
                    site: "s1".to_string(),
                    verdict: "killed".to_string(),
                    tests: vec![],
                    evidence: "aaaaaaaaaaaaaaaa".to_string(),
                },
                SiteVerdict {
                    site: "s2".to_string(),
                    verdict: "timeout".to_string(),
                    tests: vec![],
                    evidence: "bbbbbbbbbbbbbbbb".to_string(),
                },
                SiteVerdict {
                    site: "s3".to_string(),
                    verdict: "SURVIVED".to_string(),
                    tests: vec![],
                    evidence: String::new(),
                },
            ],
        };
        assert_eq!(
            Evidence::carry(&prior, "s1", "aaaaaaaaaaaaaaaa"),
            Some("killed")
        );
        assert_eq!(
            Evidence::carry(&prior, "s1", "cccccccccccccccc"),
            None,
            "a moved key is judged, not carried"
        );
        assert_eq!(
            Evidence::carry(&prior, "s2", "bbbbbbbbbbbbbbbb"),
            None,
            "a timeout is load-sensitive: re-judge"
        );
        assert_eq!(
            Evidence::carry(&prior, "s3", ""),
            None,
            "an empty key earns nothing"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
