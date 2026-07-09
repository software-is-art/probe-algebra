//! floor — ONE interpreter for every declared-floor judge.
//!
//! A judge checks observed world facts against a declared floor. Written by hand, each
//! judge is free-form Rust whose operators need mutation coverage — the perimeter, infra,
//! and substrate judges were exactly the code `#[mutate]` strained to reach. But the
//! operation is always the SAME: fold a catalog of declared requirements over an
//! observation and report which held. So there is one interpreter — [`judge`] — and a
//! judge becomes its FLOOR (data) fed here.
//!
//! Two consequences. First, correctness lives in this ONE place, so it is the one thing
//! we verify — characterized by differential GENERATION against each incumbent judge, and
//! as more judges route through it, by the diversity of their grids (a fold bug that hides
//! under one floor dies under another). There is no `#[mutate]` here: the interpreter is
//! not mutated, it is generated against. Second, a consumer needs no judging code and no
//! oracle of their own — they declare a floor and present observations, and this shipped
//! interpreter is the judge. Nothing on their side is instrumented, mutated, or restated.

use std::collections::BTreeMap;

/// A world observation, keyed by fact name. Every value carries its own readability — an
/// unread (`None`) reading is REFUSED, never assumed to hold.
#[derive(Clone, Debug)]
pub enum Observed {
    /// A protection flag (deletion blocked, force-push blocked).
    Flag(bool),
    /// A count — an approving-review floor, say; `None` = the endpoint could not be read.
    Count(Option<u64>),
    /// A set of names (merge methods, required checks, secret names); `None` = unread.
    Names(Option<Vec<String>>),
    /// A boolean setting that must be affirmatively on; `None` = unread.
    Toggle(Option<bool>),
    /// A free-text reading (a build command, say); `None` = unread.
    Text(Option<String>),
}

/// The check a requirement imposes on its fact.
pub enum Check {
    /// A flag must read `true`.
    True,
    /// A count must read exactly this — unread, or any other value, refuses.
    Exactly(u64),
    /// Every observed name must lie WITHIN this allow-set — extra-strict live sets are
    /// not drift; the constraint is a ceiling.
    Within(Vec<String>),
    /// The observed names must COVER this required set — extra live names are not drift;
    /// the constraint is a floor.
    Covers(Vec<String>),
    /// A toggle must read `Some(true)`.
    Enabled,
    /// A text reading must equal this exactly — unread, or any other value, refuses.
    Is(String),
}

/// One declared requirement: the fact it names and the check it imposes.
pub struct Requirement {
    pub key: String,
    pub check: Check,
}

impl Requirement {
    pub fn new(key: impl Into<String>, check: Check) -> Self {
        Requirement {
            key: key.into(),
            check,
        }
    }
}

/// A declared floor — the whole judge as DATA, a catalog of requirements. Judging is its
/// method: there is no free-form judge, only a `Floor` fed a world.
pub struct Floor {
    requirements: Vec<Requirement>,
}

impl Floor {
    pub fn of(requirements: Vec<Requirement>) -> Self {
        Floor { requirements }
    }

    /// THE interpreter. Fold the floor over the world; accept iff every requirement
    /// holds, returning the satisfied keys or the violated ones. This is the only
    /// judging code in the system — every declared-floor judge is a `Floor` judged here.
    pub fn judge(&self, world: &BTreeMap<String, Observed>) -> Result<Vec<String>, Vec<String>> {
        let mut held = Vec::new();
        let mut violations = Vec::new();
        for req in &self.requirements {
            if holds(&req.check, world.get(&req.key)) {
                held.push(req.key.clone());
            } else {
                violations.push(req.key.clone());
            }
        }
        if violations.is_empty() {
            Ok(held)
        } else {
            Err(violations)
        }
    }
}

/// Why a requirement (or one element of a set requirement) failed — the interpreter's
/// JUDGMENT, from which a judge renders its prose. The interpreter decides held-vs-why;
/// the message is the judge's presentation, pinned byte-for-byte by its own tests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fail {
    /// The fact could not be read (`None`) — refused, never assumed.
    Unread,
    /// A required element (Covers) is absent from the observed set.
    Missing,
    /// An observed element (Within) is not in the allow-set.
    Outside,
    /// A scalar read the wrong value (Exactly / Is) — the read carried here.
    Wrong(String),
    /// A flag read false, or a toggle read `Some(false)`.
    Off,
}

/// One judged line: the requirement key, the SUBJECT it concerns (the element for a set
/// check, the value for a scalar, empty for a bare flag), and the verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub key: String,
    pub subject: String,
    pub verdict: Result<(), Fail>,
}

impl Floor {
    /// The structured judgment — every requirement expanded to its per-element lines,
    /// held or failed with a reason. A judge renders these to prose; the DECISION is
    /// here, so no free-form judge code decides pass/fail. Set checks (Covers/Within)
    /// expand to one line per element, in element order.
    pub fn outcomes(&self, world: &BTreeMap<String, Observed>) -> Vec<Line> {
        let mut lines = Vec::new();
        for req in &self.requirements {
            let obs = world.get(&req.key);
            let line = |subject: &str, verdict: Result<(), Fail>| Line {
                key: req.key.clone(),
                subject: subject.to_string(),
                verdict,
            };
            match (&req.check, obs) {
                (Check::True, Some(Observed::Flag(b))) => {
                    lines.push(line("", if *b { Ok(()) } else { Err(Fail::Off) }));
                }
                (Check::Enabled, Some(Observed::Toggle(Some(b)))) => {
                    lines.push(line("", if *b { Ok(()) } else { Err(Fail::Off) }));
                }
                (Check::Exactly(n), Some(Observed::Count(Some(m)))) => {
                    if m == n {
                        lines.push(line(&m.to_string(), Ok(())));
                    } else {
                        lines.push(line("", Err(Fail::Wrong(m.to_string()))));
                    }
                }
                (Check::Is(want), Some(Observed::Text(Some(got)))) => {
                    if got == want {
                        lines.push(line(got, Ok(())));
                    } else {
                        lines.push(line("", Err(Fail::Wrong(got.clone()))));
                    }
                }
                (Check::Within(allow), Some(Observed::Names(Some(names)))) => {
                    for name in names {
                        let v = if allow.contains(name) {
                            Ok(())
                        } else {
                            Err(Fail::Outside)
                        };
                        lines.push(line(name, v));
                    }
                }
                (Check::Covers(required), Some(Observed::Names(Some(names)))) => {
                    for elem in required {
                        let v = if names.contains(elem) {
                            Ok(())
                        } else {
                            Err(Fail::Missing)
                        };
                        lines.push(line(elem, v));
                    }
                }
                // any unread fact, absent fact, or type mismatch is a single Unread line
                // for the key — refused by name, never assumed.
                _ => lines.push(line("", Err(Fail::Unread))),
            }
        }
        lines
    }
}

/// Does one check hold against its (possibly unread, possibly absent) fact? An unread
/// reading, a missing fact, or a type mismatch never holds — refused, never assumed.
fn holds(check: &Check, obs: Option<&Observed>) -> bool {
    match (check, obs) {
        (Check::True, Some(Observed::Flag(b))) => *b,
        (Check::Exactly(n), Some(Observed::Count(Some(m)))) => m == n,
        (Check::Within(allow), Some(Observed::Names(Some(names)))) => {
            names.iter().all(|n| allow.contains(n))
        }
        (Check::Covers(required), Some(Observed::Names(Some(names)))) => {
            required.iter().all(|r| names.contains(r))
        }
        (Check::Enabled, Some(Observed::Toggle(Some(b)))) => *b,
        (Check::Is(want), Some(Observed::Text(Some(got)))) => got == want,
        _ => false,
    }
}
