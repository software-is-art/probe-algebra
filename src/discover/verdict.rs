//! verdict — the gates become locks: GREEN AS A FACT ABOUT A TREE HASH, never a memory.
//! Every other judgment in this repo is already staleness-proof (a spec lock drifts
//! detectably; a census refuses); the gates were the one judgment living as terminal
//! output and working memory — which is how a green survived two tree moves in one
//! session and lied both times. This store closes the class by construction: a verdict
//! is keyed by the content fingerprint of what the gates judge, a missing entry fails
//! closed as UNJUDGED, and `bundle owes` derives the to-do list a change owes instead of
//! the agent remembering it.
//!
//! Disclosed edges: the fingerprint is FNV-64 (a fingerprint, not a commitment — these
//! verdicts are the suit's local memory; the committed, countersigned form is the
//! sampled-countersign candidate), and the skip set (.git, target, .suit, attest) is a
//! scope claim: what no gate reads cannot stale a verdict, and the attestation cannot
//! be part of the tree it describes.

use std::path::{Path, PathBuf};

use crate::discover::gates::{Cadence, GateRegistry};

/// The judgment store: green as a FACT ABOUT A TREE HASH, never a memory. A verdict is
/// keyed by the content fingerprint of everything the gates judge, so the failure class
/// that burned the session twice — a green held after the tree moved — is
/// unrepresentable: a moved tree has no key match, a missing entry fails CLOSED as
/// unjudged, and only green is ever recorded (a red gate demands a tree change, and a
/// changed tree is a new key). Verdicts live in `.suit/verdicts` — the suit's own local
/// memory, outside the scope they judge and outside git; the committed, countersigned
/// form is the sampled-countersign candidate's business.
pub struct VerdictStore {
    dir: PathBuf,
}

#[crate::mutate("verdict")]
impl VerdictStore {
    /// The store beside a crate: `<crate root>/.suit/verdicts`.
    pub fn beside(crate_root: &Path) -> VerdictStore {
        VerdictStore {
            dir: crate_root.join(".suit/verdicts"),
        }
    }

    /// The SCOPE KEY: a fingerprint of every file the gates judge — relative paths and
    /// bytes in sorted walk order, skipping only what no gate reads (`.git`, `target`,
    /// the suit's own artifacts) plus `attest/` (the transcript DESCRIBES the tree, so
    /// it cannot be part of the tree it describes). Relative paths make the key a claim
    /// about CONTENT, portable across checkouts — the countersign compares these.
    ///
    /// Capability: Effectful — reads the tree it fingerprints.
    pub fn tree_hash(crate_root: &Path) -> Result<String, String> {
        let mut files = Vec::new();
        Self::walk(crate_root, crate_root, &mut files)?;
        files.sort();
        Self::fingerprint(crate_root, &files)
    }

    /// A SUPPORT KEY: the same walk, filtered by the projection a gate DECLARES it
    /// reads — an edit outside the support cannot move this key, so it cannot owe the
    /// gate. The docs-only friction, closed: the delta's image under a gate whose
    /// support excludes it is zero by declaration, never by memory.
    ///
    /// Capability: Effectful — reads the tree it fingerprints.
    pub fn support_hash(
        crate_root: &Path,
        support: &crate::discover::gates::Support,
    ) -> Result<String, String> {
        let mut files = Vec::new();
        Self::walk(crate_root, crate_root, &mut files)?;
        files.sort();
        files.retain(|relative| support.admits(relative));
        Self::fingerprint(crate_root, &files)
    }

    /// FNV-64 over relative paths and bytes, in the caller's order — the one fold both
    /// keys share, so the scope key and every support key are the same claim about
    /// different projections.
    fn fingerprint(crate_root: &Path, files: &[String]) -> Result<String, String> {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for byte in bytes {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            hash ^= 0xff;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        };
        for relative in files {
            eat(relative.as_bytes());
            let bytes = std::fs::read(crate_root.join(relative))
                .map_err(|e| format!("verdict store: cannot read {relative} ({e})"))?;
            eat(&bytes);
        }
        Ok(format!("{hash:016x}"))
    }

    /// Is a green verdict held for this gate at this key? Fails CLOSED: no store, no
    /// entry, or a different key all read as UNJUDGED.
    pub fn held(&self, gate: &str, key: &str) -> bool {
        self.dir.join(Self::entry(gate, key)).exists()
    }

    /// Record a green verdict — the only kind recorded: red demands a tree change, and
    /// a changed tree is a new key, so "known red" and "unjudged" honestly coincide.
    ///
    /// Capability: Effectful — writes one entry under the store directory.
    pub fn record(&self, gate: &str, key: &str) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|e| format!("verdict store: cannot create {} ({e})", self.dir.display()))?;
        std::fs::write(
            self.dir.join(Self::entry(gate, key)),
            format!("green — toolchain {}\n", crate::discover::gates::TOOLCHAIN),
        )
        .map_err(|e| format!("verdict store: cannot record `{gate}` ({e})"))
    }

    /// The every-change roster with each gate's standing AT ITS OWN SUPPORT KEY — the
    /// derived to-do list: what this tree still OWES is exactly the gates without
    /// verdicts at the key of what they each read.
    pub fn owed(&self, crate_root: &Path) -> Result<Vec<(&'static str, bool)>, String> {
        GateRegistry::declared()
            .iter()
            .filter(|gate| matches!(gate.cadence, Cadence::EveryChange))
            .map(|gate| {
                let key = Self::support_hash(crate_root, &gate.support)?;
                Ok((gate.name, self.held(gate.name, &key)))
            })
            .collect()
    }

    /// A gate name as a filename: alphanumerics kept, everything else a dash.
    fn entry(gate: &str, key: &str) -> String {
        let slug: String = gate
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        format!("{slug}@{key}")
    }

    /// Sorted-order tree walk, relative paths out, the skip set disclosed in the doc.
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| format!("verdict store: cannot walk {} ({e})", dir.display()))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), ".git" | "target" | ".suit" | "attest") {
                continue;
            }
            if path.is_dir() {
                Self::walk(root, &path, out)?;
            } else {
                out.push(
                    path.strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod probes {
    use super::VerdictStore;
    use crate::discover::gates::{Cadence, GateRegistry, Support};

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("probe-verdict-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// THE CLASS, CLOSED: green is a fact about a tree hash. An unjudged tree owes
    /// (fails closed), a recorded verdict holds, ONE MOVED BYTE re-opens the debt —
    /// the stale green that burned the session twice is unrepresentable — and the
    /// store's own entries sit OUTSIDE the scope they judge, so recording a verdict
    /// does not move the tree it describes.
    #[test]
    fn a_moved_tree_owes_again() {
        let root = scratch("moved");
        std::fs::write(root.join("a.rs"), "pub fn a() {}\n").unwrap();
        let store = VerdictStore::beside(&root);
        let key = VerdictStore::tree_hash(&root).unwrap();
        assert!(!store.held("fmt", &key), "unjudged fails closed");
        store.record("fmt", &key).unwrap();
        assert!(store.held("fmt", &key), "a recorded verdict holds");

        std::fs::write(root.join("a.rs"), "pub fn a() { }\n").unwrap();
        let moved = VerdictStore::tree_hash(&root).unwrap();
        assert_ne!(key, moved, "one byte moves the key");
        assert!(!store.held("fmt", &moved), "a moved tree has no green");
        assert_eq!(
            VerdictStore::tree_hash(&root).unwrap(),
            moved,
            "verdicts sit outside their own scope"
        );
    }

    /// The key is CONTENT, not location or time: two checkouts with identical relative
    /// trees share a key — the portability the sampled countersign will stand on.
    #[test]
    fn the_key_is_content_not_place() {
        let here = scratch("content-here");
        let there = scratch("content-there");
        for root in [&here, &there] {
            std::fs::create_dir_all(root.join("src")).unwrap();
            std::fs::write(root.join("src/lib.rs"), "pub struct T;\n").unwrap();
        }
        assert_eq!(
            VerdictStore::tree_hash(&here).unwrap(),
            VerdictStore::tree_hash(&there).unwrap()
        );
        // a path is part of the content: the same bytes under another name is
        // another tree.
        std::fs::rename(there.join("src/lib.rs"), there.join("src/lob.rs")).unwrap();
        assert_ne!(
            VerdictStore::tree_hash(&here).unwrap(),
            VerdictStore::tree_hash(&there).unwrap()
        );
    }

    /// THE SUPPORT PROJECTION: an edit OUTSIDE a gate's support cannot move its key —
    /// docs are inert to every support, spec/ is inert to the rust surface — while an
    /// admitted edit still re-opens the debt. The docs-only friction, closed and held
    /// closed.
    #[test]
    fn an_inert_edit_cannot_owe() {
        let root = scratch("support");
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::create_dir_all(root.join("spec")).unwrap();
        std::fs::write(root.join("a.rs"), "pub fn a() {}\n").unwrap();
        std::fs::write(root.join("docs/roadmap.md"), "prose\n").unwrap();
        std::fs::write(root.join("README.md"), "front door\n").unwrap();
        std::fs::write(root.join("spec/x.spec"), "lock\n").unwrap();
        let judged = VerdictStore::support_hash(&root, &Support::Judged).unwrap();
        let rust = VerdictStore::support_hash(&root, &Support::RustSurface).unwrap();

        std::fs::write(root.join("docs/roadmap.md"), "more prose\n").unwrap();
        std::fs::write(root.join("README.md"), "wider front door\n").unwrap();
        assert_eq!(
            judged,
            VerdictStore::support_hash(&root, &Support::Judged).unwrap(),
            "prose is inert to the judged tree"
        );
        assert_eq!(
            rust,
            VerdictStore::support_hash(&root, &Support::RustSurface).unwrap(),
            "prose is inert to the rust surface"
        );

        std::fs::write(root.join("spec/x.spec"), "moved\n").unwrap();
        assert_ne!(
            judged,
            VerdictStore::support_hash(&root, &Support::Judged).unwrap(),
            "a spec edit moves the judged key"
        );
        assert_eq!(
            rust,
            VerdictStore::support_hash(&root, &Support::RustSurface).unwrap(),
            "a spec edit sits outside the rust surface"
        );

        std::fs::write(root.join("a.rs"), "pub fn a() { }\n").unwrap();
        assert_ne!(
            rust,
            VerdictStore::support_hash(&root, &Support::RustSurface).unwrap(),
            "an admitted edit re-opens the debt"
        );
    }

    /// The owed roster IS the registry's every-change set — derived, never restated:
    /// an empty store owes every one of them, and only them.
    #[test]
    fn the_owed_roster_is_the_registry() {
        let root = scratch("roster");
        let store = VerdictStore::beside(&root);
        let owed = store.owed(&root).unwrap();
        let declared = GateRegistry::declared()
            .iter()
            .filter(|g| matches!(g.cadence, Cadence::EveryChange))
            .count();
        assert_eq!(owed.len(), declared, "every-change gates, exactly");
        assert!(!owed.is_empty(), "the roster is never empty");
        assert!(
            owed.iter().all(|(_, held)| !held),
            "an empty store owes everything"
        );
    }
}
