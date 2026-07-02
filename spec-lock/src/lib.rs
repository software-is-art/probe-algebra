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
}
