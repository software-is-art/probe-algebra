//! spec-lock — freeze a derived spec into a committed file, and gate CI on drift.
//!
//! The pattern this crate extracts, in miniature:
//!
//! 1. **Derive.** Some tool in your build derives a text artifact from the code — a discovered
//!    algebraic spec, an API census, a schema dump, a routing table. The one hard requirement is
//!    that the derivation be **deterministic**: same code, same text, byte for byte, on every
//!    machine. (Sort your collections; pin your formatting; keep timestamps, hostnames, and hash
//!    iteration order out of the output. If the derivation is not deterministic, the gate below is
//!    noise, and you should fix that before adopting this crate.)
//! 2. **Freeze.** Run the derivation once and write its output to a committed file ([`bless`]).
//!    From then on the file is not documentation *about* the code — it is a behaviour lock the
//!    repository carries.
//! 3. **Gate.** In CI (a plain `#[test]` is enough), re-derive the live text and compare it to the
//!    committed file ([`check`]). A match means the code still means what was ratified. A mismatch
//!    fails the build, and the fix is never to edit the lock file by hand: regenerate it and let
//!    the resulting diff go through review. **The committed diff IS the ratification** — the one
//!    human act the derivation cannot perform. An unintended behaviour change becomes a red build;
//!    an intended one becomes a readable diff someone approves.
//!
//! The whole mechanism is one struct and two functions:
//!
//! ```
//! use spec_lock::{bless, check, Lock};
//!
//! let dir = std::env::temp_dir().join("spec-lock-doctest");
//! let lock = Lock {
//!     name: "my spec".to_string(),
//!     path: dir.join("my.spec"),
//!     live: "the deterministically derived text\n".to_string(),
//! };
//!
//! assert!(check(std::slice::from_ref(&lock)).is_err()); // never frozen: stale
//! bless(std::slice::from_ref(&lock)).unwrap();          // freeze it (ratify the diff!)
//! assert!(check(std::slice::from_ref(&lock)).is_ok());  // CI's gate from now on
//! # std::fs::remove_dir_all(&dir).ok();
//! ```
//!
//! What it buys: the derived artifact can never rot silently, and behaviour review happens where
//! review already happens — in the diff. What it costs: the determinism obligation above, and some
//! churn when the *deriving engine* changes (every lock regenerates at once; that diff still wants
//! a human eye). This crate deliberately knows nothing about where `live` comes from — it only
//! owns the compare and the write.
//!
//! ## Cross-locks: anchoring to a FOREIGN ratified baseline
//!
//! A [`Lock`] compares a live derivation against its OWN committed file. Long chains of
//! artifacts sharing one frozen instrument need a second kind: a [`CrossLock`] holds a live
//! derivation against a DIFFERENT artifact's committed file — one blessed by a different
//! derivation, in a different unit of work — pinned by content hash. There is deliberately
//! no bless path for it: if the anchor and the dependent drift apart, the only correct
//! outcome is a red gate naming the broken link, never a regeneration. The pinned hash is
//! what splits the red into its two different review conversations: "my derivation drifted"
//! (the anchor still matches its pin) versus "my anchor was re-ratified upstream" (the pin
//! no longer matches the file) — see [`check_cross`]. [`anchor_graph`] renders a battery's
//! chain topology deterministically, so who-anchors-whom is itself a lockable census
//! instead of prose in code comments. (Field-note origin: a registered-numerics research
//! corpus whose cross-stone gates re-derive a census live and assert it byte-identical to
//! a predecessor stone's committed baseline, sha-pinned — twenty-plus artifacts sharing
//! one instrument.)

use std::fs;
use std::path::PathBuf;

/// One frozen artifact: a display name (for error messages), the committed file that locks it,
/// and the freshly derived live text to hold that file against.
///
/// The caller owns the derivation of `live`; it must be deterministic (see the crate docs).
pub struct Lock {
    /// How the artifact is named in reports ("router", "public API", ...).
    pub name: String,
    /// The committed lock file — the ratified text.
    pub path: PathBuf,
    /// The live text, re-derived from the code right now.
    pub live: String,
}

/// The drift gate: compare each lock's live text to its committed file.
///
/// Returns `Ok` with every name verified fresh, or `Err` with the names that drifted. A missing
/// or unreadable lock file is **stale**, never fresh — an artifact that was never frozen has
/// never been ratified. The fix for a stale lock is to regenerate ([`bless`]) and put the diff
/// through review, not to edit the file by hand.
pub fn check(locks: &[Lock]) -> Result<Vec<&str>, Vec<&str>> {
    let mut fresh = Vec::new();
    let mut stale = Vec::new();
    for lock in locks {
        match fs::read_to_string(&lock.path) {
            Ok(committed) if committed == lock.live => fresh.push(lock.name.as_str()),
            _ => stale.push(lock.name.as_str()),
        }
    }
    if stale.is_empty() {
        Ok(fresh)
    } else {
        Err(stale)
    }
}

/// The regeneration path: write each lock's live text to its committed file, creating parent
/// directories as needed. Run this when behaviour legitimately changes; the diff it produces in
/// the committed files is what review ratifies.
pub fn bless(locks: &[Lock]) -> std::io::Result<()> {
    for lock in locks {
        if let Some(parent) = lock.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&lock.path, &lock.live)?;
    }
    Ok(())
}

/// One anchored artifact: a live derivation held against a FOREIGN committed file — an
/// artifact ratified elsewhere that this unit of work must match but may never re-bless.
///
/// There is no bless path for a `CrossLock`, by type: the anchor belongs to another
/// derivation's ratification. A red cross-lock is a broken chain to repair at whichever
/// end actually moved, never a file to regenerate from here.
pub struct CrossLock {
    /// How the dependent artifact is named in reports.
    pub name: String,
    /// The foreign committed file this derivation anchors to.
    pub foreign_path: PathBuf,
    /// sha256 of the anchor's RATIFIED bytes, lowercase hex — minted once when the chain
    /// was reviewed (see [`sha256_hex`]). This is what distinguishes "my derivation
    /// drifted" from "my anchor was re-ratified upstream": two different conversations.
    pub pinned_sha: String,
    /// The live text, re-derived right now, that must equal the anchor byte for byte.
    pub live: String,
}

/// The cross-lock gate: verify each anchored derivation against its foreign baseline.
///
/// `Ok` carries the verified names. `Err` carries one DIAGNOSIS per broken link, in the
/// failure's own vocabulary — the three ways a chain breaks are three different repairs:
///
/// - **anchor missing** — the foreign file is gone or unreadable: the chain's instrument
///   was removed, or the path is wrong;
/// - **anchor re-ratified upstream** — the foreign file no longer matches `pinned_sha`:
///   someone re-blessed the baseline; every dependent must be re-reviewed against the
///   new ratification (and re-pinned), or the upstream change reverted;
/// - **live derivation drifted** — the anchor still matches its pin, so the movement is
///   HERE: this unit of work no longer reproduces the shared baseline.
pub fn check_cross(locks: &[CrossLock]) -> Result<Vec<&str>, Vec<String>> {
    let mut fresh = Vec::new();
    let mut broken = Vec::new();
    for lock in locks {
        match fs::read(&lock.foreign_path) {
            Err(_) => broken.push(format!(
                "`{}`: anchor missing — {} is gone or unreadable; a chain cannot hang \
                 from a file that is not there",
                lock.name,
                lock.foreign_path.display()
            )),
            Ok(bytes) => {
                let found = sha256_hex(&bytes);
                if found != lock.pinned_sha {
                    broken.push(format!(
                        "`{}`: anchor re-ratified upstream (or tampered) — pinned {}…, \
                         found {}…; re-review every dependent against the new baseline \
                         and re-pin, or revert the upstream change",
                        lock.name,
                        &lock.pinned_sha[..12.min(lock.pinned_sha.len())],
                        &found[..12]
                    ));
                } else if bytes != lock.live.as_bytes() {
                    broken.push(format!(
                        "`{}`: live derivation drifted from its anchor — the baseline \
                         still matches its pin, so the movement is here, not upstream",
                        lock.name
                    ));
                } else {
                    fresh.push(lock.name.as_str());
                }
            }
        }
    }
    if broken.is_empty() {
        Ok(fresh)
    } else {
        Err(broken)
    }
}

/// The chain topology as deterministic text — one line per cross-lock, sorted by name:
/// who anchors whom, at which pin. Freeze it with an ordinary [`Lock`] and the anchor
/// graph itself becomes a reviewed census instead of prose in code comments.
pub fn anchor_graph(locks: &[CrossLock]) -> String {
    let mut lines: Vec<String> = locks
        .iter()
        .map(|l| {
            format!(
                "{} -> {} @ {}\n",
                l.name,
                l.foreign_path.display(),
                &l.pinned_sha[..12.min(l.pinned_sha.len())]
            )
        })
        .collect();
    lines.sort();
    let mut out = String::from(
        "# anchor graph: which derivation is pinned to which foreign baseline — \
         cross-locks are check-only, so every edge below is a chain review already held.\n",
    );
    for line in lines {
        out.push_str(&line);
    }
    out
}

/// A ratified-exceptions REGISTER: a committed, HAND-AUTHORED baseline of findings a
/// human has accepted, each with its justification — the third artifact kind, next to
/// [`Lock`] (generated, blessed) and [`CrossLock`] (foreign, pinned).
///
/// A register is never generated: writing a key into it IS the ratification, and the
/// justification is the one thing no derivation can produce. The tooling here only
/// reads and diffs — [`Register::check`] compares the live finding set against the
/// baseline and renders the drift as SET DIFFERENCE (`2 new finding(s)`, `1 resolved`)
/// instead of a byte diff, because for keyed exception sets that is the review
/// conversation: each new key wants a justification or a fix, each resolved key wants
/// its line deleted (a stale exception is a lie the register tells forever).
///
/// File format, one exception per line (`#` comments and blank lines skipped):
///
/// ```text
/// <key>: <justification — why this finding is accepted rather than fixed>
/// ```
///
/// A key with no justification is a PARSE error, not an entry — the format enforces
/// the discipline. A missing file is the empty register: zero ratified exceptions
/// (registers are declarations, so absence honestly declares "no exceptions"; this is
/// the one place missing-is-not-stale, because there is nothing to regenerate).
pub struct Register {
    /// Display name for drift messages.
    pub name: String,
    /// The committed register file.
    pub path: PathBuf,
}

/// The set difference between live findings and a [`Register`]'s ratified baseline.
pub struct RegisterDrift {
    /// Live findings with no ratified entry — each wants a justification or a fix.
    pub new: Vec<String>,
    /// Ratified entries no longer found live — each wants its line deleted.
    pub resolved: Vec<String>,
}

impl RegisterDrift {
    /// The review conversation, rendered: what appeared, what dissolved.
    pub fn render(&self, register: &Register) -> String {
        let mut out = format!("register `{}` drifted:", register.name);
        if !self.new.is_empty() {
            out.push_str(&format!(
                " {} new finding(s) — ratify each with a justification in {}, or fix it: {}.",
                self.new.len(),
                register.path.display(),
                self.new.join(", ")
            ));
        }
        if !self.resolved.is_empty() {
            out.push_str(&format!(
                " {} resolved — delete the line(s) (a stale exception is a lie): {}.",
                self.resolved.len(),
                self.resolved.join(", ")
            ));
        }
        out
    }
}

impl Register {
    /// Read the ratified entries: `key -> justification`, in file order de-duplicated by
    /// key (a duplicate key is a parse error — one finding, one ratification).
    pub fn entries(&self) -> Result<Vec<(String, String)>, String> {
        let text = match fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(format!("register `{}` unreadable: {e}", self.name)),
        };
        let mut entries: Vec<(String, String)> = Vec::new();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, justification) = line.split_once(':').ok_or(format!(
                "register `{}` line {}: an entry is `<key>: <justification>` — a bare key \
                 is not a ratification",
                self.name,
                n + 1
            ))?;
            let (key, justification) = (key.trim().to_string(), justification.trim());
            if justification.is_empty() {
                return Err(format!(
                    "register `{}` line {}: `{key}` carries no justification — the \
                     justification IS the ratification",
                    self.name,
                    n + 1
                ));
            }
            if entries.iter().any(|(k, _)| *k == key) {
                return Err(format!(
                    "register `{}` line {}: `{key}` is ratified twice — one finding, one \
                     ratification",
                    self.name,
                    n + 1
                ));
            }
            entries.push((key, justification.to_string()));
        }
        Ok(entries)
    }

    /// Hold a live finding set against the baseline. `Ok(())` means byte-for-byte the
    /// same SET (order-independent); `Err` is either a parse refusal (`Err(Err(msg))`
    /// flattened to text) or the set difference, pre-rendered for the gate log.
    pub fn check<'a>(&self, live: impl IntoIterator<Item = &'a str>) -> Result<(), String> {
        let ratified: Vec<String> = self.entries()?.into_iter().map(|(k, _)| k).collect();
        let live: Vec<&str> = live.into_iter().collect();
        let drift = RegisterDrift {
            new: live
                .iter()
                .filter(|k| !ratified.iter().any(|r| r == *k))
                .map(|k| k.to_string())
                .collect(),
            resolved: ratified
                .iter()
                .filter(|r| !live.contains(&r.as_str()))
                .cloned()
                .collect(),
        };
        if drift.new.is_empty() && drift.resolved.is_empty() {
            return Ok(());
        }
        Err(drift.render(self))
    }
}

/// sha256 of `bytes`, lowercase hex — for minting a [`CrossLock`]'s pin at chain-review
/// time. Implemented here (FIPS 180-4, pinned against the NIST vectors in this crate's
/// tests) so the crate keeps its zero-dependency promise.
pub fn sha256_hex(bytes: &[u8]) -> String {
    sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// SHA-256 (FIPS 180-4), self-contained — the textbook implementation, kept boring on
/// purpose; the NIST test vectors in this crate's tests are its correctness pin.
mod sha256 {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    pub fn digest(data: &[u8]) -> [u8; 32] {
        let mut h: [u32; 8] = [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];
        // padding: 0x80, zeros to 56 mod 64, then the bit length as big-endian u64.
        let mut msg = data.to_vec();
        let bit_len = (data.len() as u64).wrapping_mul(8);
        msg.push(0x80);
        while msg.len() % 64 != 56 {
            msg.push(0);
        }
        msg.extend_from_slice(&bit_len.to_be_bytes());

        for block in msg.chunks_exact(64) {
            let mut w = [0u32; 64];
            for (i, word) in block.chunks_exact(4).enumerate() {
                w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
            }
            for i in 16..64 {
                let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
                let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
                w[i] = w[i - 16]
                    .wrapping_add(s0)
                    .wrapping_add(w[i - 7])
                    .wrapping_add(s1);
            }
            let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
            for i in 0..64 {
                let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
                let ch = (e & f) ^ (!e & g);
                let temp1 = hh
                    .wrapping_add(s1)
                    .wrapping_add(ch)
                    .wrapping_add(K[i])
                    .wrapping_add(w[i]);
                let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
                let maj = (a & b) ^ (a & c) ^ (b & c);
                let temp2 = s0.wrapping_add(maj);
                hh = g;
                g = f;
                f = e;
                e = d.wrapping_add(temp1);
                d = c;
                c = b;
                b = a;
                a = temp1.wrapping_add(temp2);
            }
            h[0] = h[0].wrapping_add(a);
            h[1] = h[1].wrapping_add(b);
            h[2] = h[2].wrapping_add(c);
            h[3] = h[3].wrapping_add(d);
            h[4] = h[4].wrapping_add(e);
            h[5] = h[5].wrapping_add(f);
            h[6] = h[6].wrapping_add(g);
            h[7] = h[7].wrapping_add(hh);
        }
        let mut out = [0u8; 32];
        for (i, word) in h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// A per-test scratch directory under the system temp dir (no dev-deps: pid + test name
    /// keep concurrent test runs apart).
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(test: &str) -> Self {
            let dir = std::env::temp_dir()
                .join("spec-lock-tests")
                .join(format!("{}-{test}", std::process::id()));
            fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).ok();
        }
    }

    fn lock(dir: &Path, name: &str, live: &str) -> Lock {
        Lock {
            name: name.to_string(),
            path: dir.join(format!("{name}.spec")),
            live: live.to_string(),
        }
    }

    #[test]
    fn a_committed_file_matching_the_live_text_is_fresh() {
        let scratch = Scratch::new("fresh");
        let a = lock(&scratch.0, "a", "text of a\n");
        let b = lock(&scratch.0, "b", "text of b\n");
        fs::write(&a.path, &a.live).unwrap();
        fs::write(&b.path, &b.live).unwrap();
        assert_eq!(check(&[a, b]), Ok(vec!["a", "b"]));
    }

    #[test]
    fn a_committed_file_that_differs_is_stale_and_named() {
        let scratch = Scratch::new("stale");
        let a = lock(&scratch.0, "a", "text of a\n");
        let b = lock(&scratch.0, "b", "text of b\n");
        fs::write(&a.path, &a.live).unwrap();
        fs::write(&b.path, "some older ratified text\n").unwrap();
        // only the drifted lock is reported; the fresh one is not blamed.
        assert_eq!(check(&[a, b]), Err(vec!["b"]));
    }

    #[test]
    fn a_missing_lock_file_is_stale_never_fresh() {
        let scratch = Scratch::new("missing");
        // even an EMPTY live text is stale against a missing file: never frozen, never ratified.
        let never_frozen = lock(&scratch.0, "never-frozen", "");
        assert_eq!(check(&[never_frozen]), Err(vec!["never-frozen"]));
    }

    #[test]
    fn bless_then_check_round_trips_and_creates_parent_dirs() {
        let scratch = Scratch::new("bless");
        let nested = Lock {
            name: "nested".to_string(),
            path: scratch.0.join("deep/inside/nested.spec"),
            live: "live text, frozen\n".to_string(),
        };
        assert_eq!(check(std::slice::from_ref(&nested)), Err(vec!["nested"]));
        bless(std::slice::from_ref(&nested)).unwrap();
        assert_eq!(check(std::slice::from_ref(&nested)), Ok(vec!["nested"]));
        // the file holds exactly the live text — bless writes, check re-reads, nothing rewrites.
        assert_eq!(fs::read_to_string(&nested.path).unwrap(), nested.live);
    }

    // ===== cross-locks =====================================================

    /// The hash is pinned against the NIST FIPS 180-4 vectors — the implementation's
    /// one external referent.
    #[test]
    fn sha256_matches_the_nist_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        // and a message long enough to cross one block boundary (the padding path).
        assert_eq!(
            sha256_hex(&[b'a'; 64]),
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb"
        );
    }

    fn anchored(dir: &Path, name: &str, baseline: &str) -> CrossLock {
        let foreign_path = dir.join("anchor.spec");
        fs::write(&foreign_path, baseline).unwrap();
        CrossLock {
            name: name.to_string(),
            foreign_path,
            pinned_sha: sha256_hex(baseline.as_bytes()),
            live: baseline.to_string(),
        }
    }

    /// The good chain: the anchor matches its pin and the live derivation reproduces
    /// it byte for byte.
    #[test]
    fn an_intact_chain_is_verified_by_name() {
        let scratch = Scratch::new("chain-fresh");
        let link = anchored(&scratch.0, "stone-0974", "the shared census\n");
        assert_eq!(
            check_cross(std::slice::from_ref(&link)),
            Ok(vec!["stone-0974"])
        );
    }

    /// The three breaks are three DIFFERENT diagnoses — the pinned sha is what tells
    /// "my derivation drifted" apart from "my anchor was re-ratified upstream", which
    /// are different review conversations.
    #[test]
    fn each_break_is_diagnosed_in_its_own_vocabulary() {
        let scratch = Scratch::new("chain-broken");

        // the live derivation drifted; the anchor still matches its pin.
        let mut drifted = anchored(&scratch.0, "drifted", "the shared census\n");
        drifted.live = "a different derivation\n".to_string();
        let err = check_cross(std::slice::from_ref(&drifted)).unwrap_err();
        assert!(err[0].contains("live derivation drifted"), "{}", err[0]);
        assert!(err[0].contains("here, not upstream"), "{}", err[0]);

        // the anchor was re-blessed upstream: the pin no longer matches the file.
        let reratified = anchored(&scratch.0, "re-ratified", "the shared census\n");
        fs::write(&reratified.foreign_path, "a re-blessed baseline\n").unwrap();
        let err = check_cross(std::slice::from_ref(&reratified)).unwrap_err();
        assert!(err[0].contains("anchor re-ratified upstream"), "{}", err[0]);
        assert!(err[0].contains("re-pin"), "{}", err[0]);

        // the anchor is gone entirely.
        let mut missing = anchored(&scratch.0, "orphaned", "the shared census\n");
        missing.foreign_path = scratch.0.join("never-existed.spec");
        let err = check_cross(std::slice::from_ref(&missing)).unwrap_err();
        assert!(err[0].contains("anchor missing"), "{}", err[0]);
    }

    /// A broken link never hides a healthy one: verified names and diagnoses are
    /// reported side by side, per lock.
    #[test]
    fn a_broken_link_does_not_blame_the_intact_ones() {
        let scratch = Scratch::new("chain-mixed");
        let good = anchored(&scratch.0, "good", "baseline A\n");
        let mut bad = CrossLock {
            name: "bad".to_string(),
            foreign_path: scratch.0.join("other-anchor.spec"),
            pinned_sha: sha256_hex(b"baseline B\n"),
            live: "baseline B\n".to_string(),
        };
        fs::write(&bad.foreign_path, "baseline B\n").unwrap();
        bad.live = "not baseline B\n".to_string();
        let err = check_cross(&[good, bad]).unwrap_err();
        assert_eq!(err.len(), 1, "only the broken link is diagnosed: {err:?}");
        assert!(err[0].starts_with("`bad`"));
    }

    /// The anchor graph is a deterministic census — sorted by name, one edge per
    /// cross-lock — so the chain topology itself can be frozen with an ordinary Lock.
    #[test]
    fn the_anchor_graph_renders_the_chain_topology_deterministically() {
        let scratch = Scratch::new("chain-graph");
        let b = anchored(&scratch.0, "stone-b", "baseline\n");
        let a = CrossLock {
            name: "stone-a".to_string(),
            foreign_path: scratch.0.join("anchor.spec"),
            pinned_sha: b.pinned_sha.clone(),
            live: b.live.clone(),
        };
        let sha12 = b.pinned_sha[..12].to_string();
        let rendered = anchor_graph(&[b, a]);
        // declaration order does not matter: the render sorts by name.
        let expected = format!(
            "# anchor graph: which derivation is pinned to which foreign baseline — \
             cross-locks are check-only, so every edge below is a chain review already held.\n\
             stone-a -> {p} @ {sha12}\nstone-b -> {p} @ {sha12}\n",
            p = scratch.0.join("anchor.spec").display(),
        );
        assert_eq!(rendered, expected);
    }

    fn register_at(scratch: &Scratch, text: Option<&str>) -> Register {
        let path = scratch.0.join("findings.register");
        if let Some(t) = text {
            fs::write(&path, t).expect("write the register fixture");
        }
        Register {
            name: "findings".to_string(),
            path,
        }
    }

    /// The register's whole contract in one walk: an exact set match is green
    /// (order-independent), a new finding and a resolved one each drift with their own
    /// verb, and both are named in one render — the set-diff review conversation,
    /// never a byte diff.
    #[test]
    fn a_register_diffs_as_a_set_and_names_both_drift_directions() {
        let scratch = Scratch::new("register-diff");
        let r = register_at(
            &scratch,
            Some("# accepted findings\nf-1: known false positive, tracked upstream.\nf-2: rot is cosmetic here.\n"),
        );
        assert_eq!(r.check(["f-1", "f-2"]), Ok(()));
        assert_eq!(r.check(["f-2", "f-1"]), Ok(()), "a register is a SET");
        let err = r.check(["f-1", "f-2", "f-3"]).unwrap_err();
        assert!(
            err.contains("1 new finding(s)") && err.contains("f-3"),
            "{err}"
        );
        let err = r.check(["f-1"]).unwrap_err();
        assert!(err.contains("1 resolved") && err.contains("f-2"), "{err}");
        let err = r.check(["f-3"]).unwrap_err();
        assert!(
            err.contains("f-3") && err.contains("resolved") && err.contains("f-1"),
            "both directions in one render: {err}"
        );
    }

    /// A missing register is the EMPTY register (a declaration honestly absent), so
    /// zero live findings are green and any live finding is new.
    #[test]
    fn a_missing_register_declares_no_exceptions() {
        let scratch = Scratch::new("register-missing");
        let r = register_at(&scratch, None);
        assert_eq!(r.check([]), Ok(()));
        assert!(r.check(["f-1"]).unwrap_err().contains("1 new finding(s)"));
    }

    /// The format enforces the discipline: a bare key, an empty justification, and a
    /// duplicate ratification are each refused by name — never silently counted.
    #[test]
    fn a_register_refuses_unjustified_and_duplicate_ratifications() {
        let scratch = Scratch::new("register-refusals");
        let bare = register_at(&scratch, Some("f-1\n"));
        assert!(bare.check(["f-1"]).unwrap_err().contains("bare key"));
        let empty = register_at(&scratch, Some("f-1:   \n"));
        assert!(empty
            .check(["f-1"])
            .unwrap_err()
            .contains("no justification"));
        let dup = register_at(&scratch, Some("f-1: reason.\nf-1: reason again.\n"));
        assert!(dup.check(["f-1"]).unwrap_err().contains("ratified twice"));
    }
}
