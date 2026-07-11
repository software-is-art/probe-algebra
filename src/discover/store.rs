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
    /// is a NAMED verdict, not an error.
    ///
    /// Capability: Effectful — reads payload blobs and the tree it judges.
    pub fn differential(journal: &str, store: &PayloadStore) -> Result<Replay, String> {
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
            let current = match states.get(module) {
                Some(Ok(text)) => text.clone(),
                Some(Err(_)) => continue,
                None => {
                    order.push(module.to_string());
                    String::new()
                }
            };
            let (name, address) = match detail.rsplit_once(" @") {
                Some((head, addr))
                    if addr.len() == 16 && addr.chars().all(|c| c.is_ascii_hexdigit()) =>
                {
                    (head, Some(addr))
                }
                _ => (detail, None),
            };
            let next = match verb {
                "add" | "edit" => match address {
                    None => Err(format!(
                        "line {} (`{verb}` of `{name}`) predates the payload store — \
                         no address to replay",
                        n + 1
                    )),
                    Some(addr) => store.fetch(addr).and_then(|payload| {
                        if verb == "add" {
                            Bundle::add(&current, &payload)
                        } else {
                            Bundle::edit(&current, name, &payload)
                        }
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
                    Ok(text) => match std::fs::read_to_string(&path) {
                        Ok(committed) if &committed == text => {
                            "replays to the committed text — the journal is a second \
                             source for this file"
                                .to_string()
                        }
                        Ok(_) => "DIVERGES from the committed text (hand edits, fmt, \
                                  or a pre-journal birth the record never saw)"
                            .to_string(),
                        Err(e) => format!("the tree has no file to compare against ({e})"),
                    },
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
        let report = Replay::differential(&journal, &store).unwrap().render();
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
        let report = Replay::differential(&journal, &store).unwrap().render();
        assert!(report.contains("DIVERGES"), "{report}");
    }

    /// Entries that predate the store are NAMED, not errored — the report is a
    /// progress bar, and history starts wherever it starts.
    #[test]
    fn a_pre_store_entry_is_named_honestly() {
        let root = scratch("prestore");
        let store = PayloadStore::beside(&root);
        let report = Replay::differential("add src/never.rs — fn x\n", &store)
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
        let report = Replay::differential("collect src/m.rs — fn x\n", &store)
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
            let report = Replay::differential(stray, &store).unwrap().render();
            assert!(
                report.contains("predates the payload store"),
                "wrong-length or non-hex stays detail: {report}"
            );
        }
    }
}
