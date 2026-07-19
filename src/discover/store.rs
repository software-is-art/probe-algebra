//! store — stage 3 of the zero-file-patching aim: the journal grows a PAYLOAD STORE, so
//! the record stops being names-only and starts being a SOURCE. Every `add`/`edit`
//! stashes its payload content-addressed beside the journal, the entry carries the
//! address, and `Replay::differential` re-applies the effects to judge
//! `tree == replay(journal)` file by file — the log-and-anchor disposition's
//! second-source cross-check, measured instead of promised.
//!
//! Two honest edges, disclosed: the address is a fingerprint (collisions REFUSED at
//! stash, never silently overwritten), and replay re-applies EFFECTS only — the judged
//! verbs (`collect`) refuse replay by naming the effect/judgment split, because a verb
//! replayed under tomorrow's judges may judge differently while the ratification already
//! happened.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::discover::bundle::Bundle;

/// The content-addressed payload store: `bundle.payloads/` beside a journal, one blob
/// per distinct payload, addressed by fingerprint. Stashing is a projection (the same
/// payload always lands at the same address), so re-recording a verb never bloats the
/// store — the same replay-safety the verb algebra froze, holding at the storage layer.
///
/// The address is a FINGERPRINT (FNV-1a 64, 16 hex digits), not a cryptographic
/// commitment — disclosed, and made safe the honest way: a stash that finds a DIFFERENT
/// payload already at its address REFUSES rather than overwrites, so a collision is a
/// named refusal, never a silent corruption.
pub struct PayloadStore {
    dir: PathBuf,
}

#[crate::mutate("store")]
impl PayloadStore {
    /// The store beside a crate's journal: `<crate root>/bundle.payloads`.
    pub fn beside(crate_root: &Path) -> PayloadStore {
        PayloadStore {
            dir: crate_root.join("bundle.payloads"),
        }
    }

    /// Stash a payload and return its address — idempotent by content. Refusals are
    /// named: an address already carrying a DIFFERENT payload (the fingerprint
    /// collision), or an unwritable store.
    ///
    /// Capability: Effectful — writes one blob under the store directory.
    pub fn stash(&self, payload: &str) -> Result<String, String> {
        let address = Self::fingerprint(payload);
        let blob = self.dir.join(format!("{address}.payload"));
        if let Ok(existing) = std::fs::read_to_string(&blob) {
            if existing == payload {
                return Ok(address);
            }
            return Err(format!(
                "payload store: fingerprint collision at @{address} — a different \
                 payload already lives there; the store refuses rather than overwrite"
            ));
        }
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| format!("payload store: cannot create {} ({e})", self.dir.display()))?;
        std::fs::write(&blob, payload)
            .map_err(|e| format!("payload store: cannot write @{address} ({e})"))?;
        Ok(address)
    }

    /// Fetch the payload at an address; a missing blob refuses by name — a journal
    /// entry whose payload is gone cannot replay, and saying so IS the report.
    ///
    /// Capability: Effectful — reads one blob from the store directory.
    pub fn fetch(&self, address: &str) -> Result<String, String> {
        std::fs::read_to_string(self.dir.join(format!("{address}.payload")))
            .map_err(|e| format!("payload store: nothing at @{address} ({e})"))
    }

    /// FNV-1a 64 over the payload bytes, rendered as 16 hex digits.
    fn fingerprint(payload: &str) -> String {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in payload.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{hash:016x}")
    }
}

/// The REPLAY DIFFERENTIAL — stage 3's judged claim, measured file by file: reconstruct
/// each journaled file by re-applying its entries' EFFECTS (payloads from the store,
/// never re-judging — a verb replayed under tomorrow's judges may judge differently;
/// the ratification already happened), then compare against the tree. "Replays" means
/// the journal is a second source for that file; divergence is a FINDING (hand edits,
/// fmt, a pre-journal birth), not a failure — the report is the zero-file-patching
/// aim's progress bar.
pub struct Replay {
    /// One verdict per journaled file, in first-appearance order.
    pub verdicts: Vec<(String, String)>,
}

#[crate::mutate("store")]
impl Replay {
    /// Replay a journal against the tree it describes. Paths resolve exactly as the
    /// verbs recorded them (relative to the invoking directory). The one hard refusal
    /// is a line the journal grammar cannot read; everything else that cannot replay
    /// is a NAMED verdict, not an error. `normalize` is the driver's fmt-normal twin:
    /// applied to each reconstructed FINAL before the committed-text comparison —
    /// never to intermediates, whose only consumers are the verbs' own parsers — so
    /// the symmetry `replay(journal) == tree` survives a landed render going
    /// fmt-normal. `None` is the raw comparison the probes pin.
    ///
    /// Capability: Effectful — reads payload blobs and the tree it judges.
    pub fn differential(
        journal: &str,
        store: &PayloadStore,
        normalize: Normalize<'_>,
    ) -> Result<Replay, String> {
        let mut states: BTreeMap<String, Result<String, String>> = BTreeMap::new();
        let mut order: Vec<String> = Vec::new();
        for (n, line) in journal.lines().enumerate() {
            let parsed = line
                .split_once(' ')
                .and_then(|(verb, rest)| rest.split_once(" — ").map(|(m, d)| (verb, m, d)));
            let Some((verb, module, detail)) = parsed else {
                return Err(format!(
                    "bundle replay: line {} is not `<verb> <module> — <detail>`: `{line}`",
                    n + 1
                ));
            };
            // a seal row is the envelope's journal boundary — a grouping mark, not
            // an effect: it claims nothing about any file, so replay reads through
            // it (skip, never refuse — the boundary is grammar, not a gap).
            if verb == "seal" {
                continue;
            }
            let current = match states.get(module) {
                Some(Ok(text)) => text.clone(),
                Some(Err(_)) => continue,
                None => {
                    order.push(module.to_string());
                    String::new()
                }
            };
            let (name, address) = crate::discover::envelope::split_address(detail);
            let next = match verb {
                "add" | "edit" | "recast" => match address {
                    None => Err(format!(
                        "line {} (`{verb}` of `{name}`) predates the payload store — \
                         no address to replay",
                        n + 1
                    )),
                    Some(addr) => store.fetch(addr).and_then(|payload| match verb {
                        "add" => Bundle::add(&current, &payload),
                        "edit" => Bundle::edit(&current, name, &payload),
                        // a recast landed under an envelope's license: the interface
                        // moved by ratified decision, so replay re-applies the effect
                        // half alone — the same splice it landed with.
                        _ => crate::discover::bundle::splice("recast", &current, name, &payload),
                    }),
                },
                "declare" => Bundle::declare(&current, name),
                "place" => Bundle::parse(&current).map(|b| b.render()),
                other => Err(format!(
                    "line {} — replaying `{other}` needs the effect/judgment split \
                     (a disclosed gap: the journal records the judged decision, and \
                     replay must re-apply the effect without re-judging)",
                    n + 1
                )),
            };
            states.insert(
                module.to_string(),
                next.map_err(|reason| format!("unreplayable — {reason}")),
            );
        }
        let verdicts = order
            .into_iter()
            .map(|path| {
                let verdict = match &states[&path] {
                    Err(reason) => reason.clone(),
                    Ok(text) => {
                        // a normalizer error is part of the verdict, never a refusal —
                        // replay reports, it does not gate.
                        let final_text = match normalize {
                            None => Ok(text.clone()),
                            Some(normalize) => normalize(text),
                        };
                        match final_text {
                            Err(reason) => {
                                format!("the normalizer refused the reconstruction ({reason})")
                            }
                            Ok(text) => match std::fs::read_to_string(&path) {
                                Ok(committed) if committed == text => {
                                    "replays to the committed text — the journal is a second \
                                     source for this file"
                                        .to_string()
                                }
                                Ok(_) => "DIVERGES from the committed text (hand edits, fmt, \
                                          or a pre-journal birth the record never saw)"
                                    .to_string(),
                                Err(e) => format!("the tree has no file to compare against ({e})"),
                            },
                        }
                    }
                };
                (path, verdict)
            })
            .collect();
        Ok(Replay { verdicts })
    }

    /// Render the differential report, one file per line.
    pub fn render(&self) -> String {
        self.verdicts
            .iter()
            .map(|(path, verdict)| format!("{path}: {verdict}\n"))
            .collect()
    }
}

#[crate::mutate("store")]
impl Replay {
    /// JOURNAL TIME (the reverse edge pointed at the record): which judged
    /// transaction each speaker of `name` ARRIVED or LEFT in — the item relation
    /// differentiated over the journal instead of integrated over the tree, so
    /// "who holds this fact" gains "and since when". Speakers are read exactly as
    /// `uses` reads them (item rows, the one ident-bounded matcher) — one
    /// vocabulary. A reading of what replay RECONSTRUCTS (effects, from the
    /// payload store): entries beyond that horizon — pre-store, judged verbs, a
    /// module gone dark — are COUNTED, never guessed at, and git holds the past
    /// the journal never saw. A name the journal never heard refuses by name,
    /// horizon disclosed.
    pub fn spoke(journal: &str, store: &PayloadStore, name: &str) -> Result<String, String> {
        use crate::discover::items::ItemRelation;
        let speakers = |module: &str, text: &str| -> Vec<String> {
            ItemRelation::of_module(module, text)
                .map(|relation| {
                    relation
                        .entries()
                        .into_iter()
                        .filter(|(item, _)| crate::discover::bundle::mentions(&item.body, name))
                        .map(|(item, _)| item.name)
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut states: BTreeMap<String, Result<String, String>> = BTreeMap::new();
        let mut moves: Vec<String> = Vec::new();
        let mut total = 0usize;
        let mut beyond = 0usize;
        for (n, line) in journal.lines().enumerate() {
            let parsed = line
                .split_once(' ')
                .and_then(|(verb, rest)| rest.split_once(" — ").map(|(m, d)| (verb, m, d)));
            let Some((verb, module, detail)) = parsed else {
                return Err(format!(
                    "bundle spoke: line {} is not `<verb> <module> — <detail>`: `{line}`",
                    n + 1
                ));
            };
            // a seal row is the envelope's journal boundary — a grouping mark, not
            // an effect entry: spoke reads straight through it, and it counts
            // nowhere (neither total nor horizon — a boundary is not blindness).
            if verb == "seal" {
                continue;
            }
            total += 1;
            let current = match states.get(module) {
                Some(Ok(text)) => text.clone(),
                Some(Err(_)) => {
                    beyond += 1;
                    continue;
                }
                None => String::new(),
            };
            let (head, address) = match detail.rsplit_once(" @") {
                Some((head, addr))
                    if addr.len() == 16 && addr.chars().all(|c| c.is_ascii_hexdigit()) =>
                {
                    (head, Some(addr))
                }
                _ => (detail, None),
            };
            let next = match verb {
                "add" | "edit" | "recast" => match address {
                    None => Err(()),
                    Some(addr) => store
                        .fetch(addr)
                        .and_then(|payload| match verb {
                            "add" => Bundle::add(&current, &payload),
                            "edit" => Bundle::edit(&current, head, &payload),
                            // a recast landed under the envelope's license replays
                            // through the effect half alone, exactly as the
                            // differential does — journal time must not go dark on
                            // a licensed interface move.
                            _ => {
                                crate::discover::bundle::splice("recast", &current, head, &payload)
                            }
                        })
                        .map_err(|_| ()),
                },
                "declare" => Bundle::declare(&current, head).map_err(|_| ()),
                "place" => Bundle::parse(&current).map(|b| b.render()).map_err(|_| ()),
                _ => Err(()),
            };
            match next {
                Err(()) => {
                    beyond += 1;
                    states.insert(
                        module.to_string(),
                        Err(format!("dark from entry {}", n + 1)),
                    );
                }
                Ok(next_text) => {
                    let before = speakers(module, &current);
                    let after = speakers(module, &next_text);
                    let arrived = after.iter().filter(|s| !before.contains(s));
                    let left = before.iter().filter(|s| !after.contains(s));
                    let delta: Vec<String> = arrived
                        .map(|s| format!("+ {s}"))
                        .chain(left.map(|s| format!("- {s}")))
                        .collect();
                    if !delta.is_empty() {
                        moves.push(format!(
                            "  entry {} ({verb} {module} — {head}): {}\n",
                            n + 1,
                            delta.join(", ")
                        ));
                    }
                    states.insert(module.to_string(), Ok(next_text));
                }
            }
        }
        let horizon =
            format!("{beyond} of {total} entries beyond the horizon (pre-store, judged, or dark)");
        if moves.is_empty() {
            return Err(format!(
                "bundle spoke: the journal never heard `{name}` within its horizon \
                 ({horizon}) — git holds the earlier past"
            ));
        }
        Ok(format!(
            "spoke `{name}` — journal time, effects only (what replay reconstructs; \
             git holds the earlier past):\n{}  {horizon}\n",
            moves.join("")
        ))
    }
}

/// The fmt-normal hook the replay differential applies to reconstructed FINALS —
/// the driver's write-path normalization, mirrored on the read-back side so
/// `replay(journal) == tree` survives the landed render going fmt-normal. `None`
/// is the raw comparison.
pub type Normalize<'a> = Option<&'a dyn Fn(&str) -> Result<String, String>>;

#[cfg(test)]
mod probes {
    use super::{PayloadStore, Replay};
    use crate::discover::bundle::Bundle;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("probe-store-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// THE HEADLINE: content addressing round-trips, and stash is a projection — the
    /// same payload always lands at the same address, so re-recording never bloats the
    /// store. The verb algebra's replay-safety law, holding at the storage layer.
    #[test]
    fn the_store_is_content_addressed_and_stash_is_a_projection() {
        let root = scratch("address");
        let store = PayloadStore::beside(&root);
        let first = store.stash("pub fn one() {}\n").unwrap();
        let again = store.stash("pub fn one() {}\n").unwrap();
        let other = store.stash("pub fn two() {}\n").unwrap();
        assert_eq!(first, again, "stash is a projection");
        assert_ne!(first, other, "distinct payloads, distinct addresses");
        assert_eq!(store.fetch(&first).unwrap(), "pub fn one() {}\n");
        assert!(
            store.fetch("0000000000000000").is_err(),
            "a missing address refuses by name"
        );
    }

    /// The collision seam, exercised: a blob planted at an address the payload would
    /// take makes stash REFUSE rather than overwrite — the fingerprint's honesty rule.
    #[test]
    fn a_collision_refuses_rather_than_overwrites() {
        let root = scratch("collision");
        let store = PayloadStore::beside(&root);
        let address = store.stash("the real payload").unwrap();
        let blob = root
            .join("bundle.payloads")
            .join(format!("{address}.payload"));
        std::fs::write(&blob, "an impostor").unwrap();
        let refusal = store.stash("the real payload").unwrap_err();
        assert!(refusal.contains("collision"), "{refusal}");
    }

    /// STAGE 3'S CLAIM, judged on a real (miniature) history: a verb-only file, its
    /// entries carrying addresses, replays byte-exact from journal plus store — the
    /// differential says the journal is a second source for the file.
    #[test]
    fn a_verb_only_history_replays_byte_exact() {
        let root = scratch("replay");
        let store = PayloadStore::beside(&root);
        let path = root.join("m.rs").to_string_lossy().into_owned();

        // the WHOLE history goes through the journal — birth included (a birth the
        // record never saw is exactly what divergence reports).
        let birth = "pub struct Tick;\n\n/// One.\npub fn one() -> Tick {\n    Tick\n}\n";
        let revision = "/// One, revised.\npub fn one() -> Tick {\n    Tick\n}\n";
        let born = Bundle::add("", birth).unwrap();
        let edited = Bundle::edit(&born, "one", revision).unwrap();
        std::fs::write(&path, &edited).unwrap();

        let journal = format!(
            "add {path} — struct Tick, fn one @{}\nedit {path} — one @{}\n",
            store.stash(birth).unwrap(),
            store.stash(revision).unwrap()
        );
        let report = Replay::differential(&journal, &store, None)
            .unwrap()
            .render();
        assert!(report.contains("replays to the committed text"), "{report}");
    }

    /// A hand touch reads as DIVERGENCE — the differential is exactly the detector the
    /// log-and-anchor disposition promised (agreement is a judged fact).
    #[test]
    fn a_hand_touch_reads_as_divergence() {
        let root = scratch("diverge");
        let store = PayloadStore::beside(&root);
        let path = root.join("m.rs").to_string_lossy().into_owned();

        let birth = "pub struct Tick;\n\npub fn one() -> Tick {\n    Tick\n}\n";
        let born = Bundle::add("", birth).unwrap();
        std::fs::write(&path, format!("{born}// touched by hand\n")).unwrap();

        let journal = format!("add {path} — fn one @{}\n", store.stash(birth).unwrap());
        let report = Replay::differential(&journal, &store, None)
            .unwrap()
            .render();
        assert!(report.contains("DIVERGES"), "{report}");
    }

    /// Entries that predate the store are NAMED, not errored — the report is a
    /// progress bar, and history starts wherever it starts.
    #[test]
    fn a_pre_store_entry_is_named_honestly() {
        let root = scratch("prestore");
        let store = PayloadStore::beside(&root);
        let report = Replay::differential("add src/never.rs — fn x\n", &store, None)
            .unwrap()
            .render();
        assert!(report.contains("predates the payload store"), "{report}");
    }

    /// A judged verb (`collect`) refuses replay by NAMING the effect/judgment split —
    /// the disclosed gap, pinned so its closure is visible.
    #[test]
    fn a_judged_verb_names_the_effect_judgment_split() {
        let root = scratch("judged");
        let store = PayloadStore::beside(&root);
        let report = Replay::differential("collect src/m.rs — fn x\n", &store, None)
            .unwrap()
            .render();
        assert!(report.contains("effect/judgment split"), "{report}");
    }

    /// The address guard is a CONJUNCTION: a trailing `@` token that is not exactly
    /// 16 hex digits is DETAIL, not an address — either half alone would send a
    /// stray token to the store and turn an honest pre-store verdict into a
    /// missing-blob error.
    #[test]
    fn a_stray_at_token_is_not_an_address() {
        let root = scratch("stray");
        let store = PayloadStore::beside(&root);
        for stray in [
            "add src/m.rs — fn x @abcdef12345\n",
            "add src/m.rs — fn x @zzzzzzzzzzzzzzzz\n",
        ] {
            let report = Replay::differential(stray, &store, None).unwrap().render();
            assert!(
                report.contains("predates the payload store"),
                "wrong-length or non-hex stays detail: {report}"
            );
        }
    }

    /// A `recast` row — an interface move landed under an envelope's license —
    /// replays through the effect half alone: the same splice it landed with, no
    /// signature re-judgment, byte-exact against the committed tree.
    #[test]
    fn a_recast_row_replays_the_interface_move() {
        let root = scratch("recast");
        let store = PayloadStore::beside(&root);
        let path = root.join("m.rs").to_string_lossy().into_owned();

        let birth = "/// One.\npub fn one() -> u32 {\n    1\n}\n";
        let widened = "/// One, widened.\npub fn one() -> u64 {\n    1\n}\n";
        let born = Bundle::add("", birth).unwrap();
        assert!(
            Bundle::edit(&born, "one", widened)
                .unwrap_err()
                .contains("signature moved"),
            "the single verb still refuses the move"
        );
        let landed = crate::discover::bundle::splice("recast", &born, "one", widened).unwrap();
        std::fs::write(&path, &landed).unwrap();

        let journal = format!(
            "add {path} — fn one @{}\nrecast {path} — one @{}\n",
            store.stash(birth).unwrap(),
            store.stash(widened).unwrap()
        );
        let report = Replay::differential(&journal, &store, None)
            .unwrap()
            .render();
        assert!(report.contains("replays to the committed text"), "{report}");
    }

    /// JOURNAL TIME, the headline: a miniature history where a speaker of
    /// `double` arrives at birth and stops speaking in a later edit — the report
    /// names the exact transactions, byte-pinned, and speakers are read with the
    /// same matcher `uses` trusts (one vocabulary).
    #[test]
    fn spoke_names_the_transaction_each_speaker_arrived_and_left_in() {
        let root = scratch("spoke");
        let store = PayloadStore::beside(&root);
        let birth = "/// Doubles.\npub fn double(x: i64) -> i64 {\n    x * 2\n}\n\npub fn quadruple(x: i64) -> i64 {\n    double(double(x))\n}\n";
        let revision = "pub fn quadruple(x: i64) -> i64 {\n    x * 4\n}\n";
        let journal = format!(
            "add m.rs — fn double, fn quadruple @{}\nedit m.rs — quadruple @{}\n",
            store.stash(birth).unwrap(),
            store.stash(revision).unwrap()
        );
        let report = Replay::spoke(&journal, &store, "double").expect("reports");
        assert_eq!(
            report,
            "spoke `double` — journal time, effects only (what replay reconstructs; \
             git holds the earlier past):\n\
             \x20 entry 1 (add m.rs — fn double, fn quadruple): + double, + quadruple\n\
             \x20 entry 2 (edit m.rs — quadruple): - quadruple\n\
             \x20 0 of 2 entries beyond the horizon (pre-store, judged, or dark)\n"
        );
    }

    /// A name the journal never heard refuses BY NAME with the horizon disclosed —
    /// the record's silence is not evidence of absence, and the refusal says whose
    /// past (git's) holds the rest.
    #[test]
    fn spoke_refuses_the_unheard_name_disclosing_the_horizon() {
        let root = scratch("spoke-unheard");
        let store = PayloadStore::beside(&root);
        let journal = format!(
            "add m.rs — fn one @{}\n",
            store.stash("pub fn one() -> i64 {\n    1\n}\n").unwrap()
        );
        let refusal = Replay::spoke(&journal, &store, "unspoken").expect_err("refuses");
        assert!(refusal.contains("never heard `unspoken`"), "{refusal}");
        assert!(
            refusal.contains("0 of 1 entries beyond the horizon"),
            "{refusal}"
        );
        assert!(refusal.contains("git holds the earlier past"), "{refusal}");
    }

    /// Entries replay cannot reconstruct — pre-store, judged verbs — are COUNTED
    /// beyond the horizon, and a module gone dark stays dark: no guessed deltas,
    /// only disclosed blindness.
    #[test]
    fn spoke_counts_what_it_cannot_reconstruct() {
        let root = scratch("spoke-horizon");
        let store = PayloadStore::beside(&root);
        let journal = format!(
            "add old.rs — fn relic\ncollect old.rs — relic\nadd m.rs — fn one @{}\n",
            store.stash("pub fn one() -> i64 {\n    1\n}\n").unwrap()
        );
        let report = Replay::spoke(&journal, &store, "one").expect("reports");
        assert!(
            report.contains("entry 3 (add m.rs — fn one): + one"),
            "{report}"
        );
        assert!(
            report.contains("2 of 3 entries beyond the horizon"),
            "{report}"
        );
    }

    /// JOURNAL TIME does not go dark on a licensed interface move: a `recast` row
    /// replays through the effect half (the same splice the differential trusts), so
    /// `spoke` reads speakers straight through it — the horizon counts nothing and
    /// the module stays lit.
    #[test]
    fn spoke_reads_through_a_recast_row() {
        let root = scratch("spoke-recast");
        let store = PayloadStore::beside(&root);
        let birth =
            "/// One.\npub fn one() -> u32 {\n    1\n}\n\npub fn caller() -> u32 {\n    one()\n}\n";
        let widened = "/// One, widened.\npub fn one() -> u64 {\n    1\n}\n";
        let journal = format!(
            "add m.rs — fn one, fn caller @{}\nrecast m.rs — one @{}\n",
            store.stash(birth).unwrap(),
            store.stash(widened).unwrap()
        );
        let report = Replay::spoke(&journal, &store, "one").expect("reports");
        assert!(
            report.contains("0 of 2 entries beyond the horizon"),
            "the recast must not count beyond the horizon: {report}"
        );
        assert!(
            report.contains("entry 1 (add m.rs — fn one, fn caller): + caller, + one"),
            "{report}"
        );
    }

    /// The address recognizer demands BOTH facts — sixteen chars AND hex — before a
    /// tail is a payload address: a short hex-looking tail (` @cafe`) is PROSE, part
    /// of the declaration, so the entry refuses and counts beyond the horizon rather
    /// than silently shedding its tail. (The compiled site this pins: `spoke:1:
    /// && -> ||` — under the flip the tail is stripped, the declare succeeds, and
    /// the horizon count lies.)
    #[test]
    fn spoke_reads_a_short_hex_tail_as_prose_not_address() {
        let root = scratch("spoke-tail");
        let store = PayloadStore::beside(&root);
        let birth = "#[crate::algebra(Peak, \"peak\")]\npub mod peak {\n    pub fn peak(a: i64, b: i64) -> i64 {\n        a.max(b)\n    }\n}\n";
        let journal = format!(
            "add m.rs — mod peak @{}\ndeclare m.rs — commutative(peak) @cafe\n",
            store.stash(birth).unwrap()
        );
        let report = Replay::spoke(&journal, &store, "peak").expect("reports");
        assert!(
            report.contains("entry 1 (add m.rs — mod peak): + peak"),
            "{report}"
        );
        assert!(
            report.contains("1 of 2 entries beyond the horizon"),
            "{report}"
        );
    }
    /// A SEAL row — the envelope's journal boundary — is grammar, not an effect:
    /// the differential reads through it (no `envelope` verdict line, the file
    /// still replays byte-exact) and `spoke` counts it nowhere — a boundary is
    /// not blindness.
    #[test]
    fn a_seal_row_is_a_boundary_not_an_effect() {
        let root = scratch("seal");
        let store = PayloadStore::beside(&root);
        let path = root.join("m.rs").to_string_lossy().into_owned();
        let birth = "pub fn one() -> u32 {\n    1\n}\n";
        let born = Bundle::add("", birth).unwrap();
        std::fs::write(&path, &born).unwrap();
        let journal = format!(
            "add {path} — fn one @{}\nseal envelope — 1 row(s) across 1 file(s) @{}\n",
            store.stash(birth).unwrap(),
            store.stash("the bill\n").unwrap()
        );
        let report = Replay::differential(&journal, &store, None)
            .unwrap()
            .render();
        assert!(report.contains("replays to the committed text"), "{report}");
        assert!(
            !report.contains("envelope"),
            "no verdict line for the boundary: {report}"
        );
        let spoke = Replay::spoke(&journal, &store, "one").unwrap();
        assert!(
            spoke.contains("0 of 1 entries beyond the horizon"),
            "the seal counts nowhere: {spoke}"
        );
    }
    /// THE SYMMETRY PIN for the fmt-normal seam: a reconstruction that diverges RAW
    /// but agrees under the normalizer replays — the differential judges the same
    /// bytes the landing wrote — and a normalizer error is a VERDICT, never a
    /// refusal (replay reports, it does not gate). Pure fake normalizers: no
    /// process spawning in probes.
    #[test]
    fn the_normalizer_is_symmetric_and_its_error_is_a_verdict() {
        let root = scratch("normal");
        let store = PayloadStore::beside(&root);
        let path = root.join("m.rs").to_string_lossy().into_owned();
        let birth = "pub fn one() -> u32 {\n    1\n}\n";
        let born = Bundle::add("", birth).unwrap();
        // the committed tree carries the NORMALIZED render (a trailing banner the
        // raw reconstruction lacks — a stand-in for rustfmt's whitespace moves).
        std::fs::write(&path, format!("{born}// normal\n")).unwrap();
        let journal = format!("add {path} — fn one @{}\n", store.stash(birth).unwrap());

        let raw = Replay::differential(&journal, &store, None)
            .unwrap()
            .render();
        assert!(raw.contains("DIVERGES"), "raw must diverge: {raw}");

        let normalize = |text: &str| -> Result<String, String> { Ok(format!("{text}// normal\n")) };
        let normalized = Replay::differential(&journal, &store, Some(&normalize))
            .unwrap()
            .render();
        assert!(
            normalized.contains("replays to the committed text"),
            "normalizer-equal must replay: {normalized}"
        );

        let refusing = |_: &str| -> Result<String, String> { Err("no rustfmt here".to_string()) };
        let verdict = Replay::differential(&journal, &store, Some(&refusing))
            .unwrap()
            .render();
        assert!(
            verdict.contains("the normalizer refused the reconstruction (no rustfmt here)"),
            "{verdict}"
        );
    }
}
