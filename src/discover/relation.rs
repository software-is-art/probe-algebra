//! relation — the relational primitive for a judge whose judgment is a JOIN, not a flat
//! floor. `discover::floor` checks independent facts against a declared floor; substrate
//! cannot be said that way. "For each tag matching a pattern, is it on the certified
//! line?" joins the tag list with the ancestry read — two dimensions the flat floor has
//! no shape for.
//!
//! Two pieces, both what substrate forces and the flat family never needed:
//!
//!   - [`Joined`] is a LEFT JOIN of named facts with a per-name annotation (substrate's
//!     tags ⋈ ancestry-on-the-certified-line). Assembled at OBSERVE time — joining two
//!     world reads is observation, not judgment — and queried by the judge. A name the
//!     annotating read has no row for reads `None`: refused by name, never assumed.
//!   - [`Requirements`] carries the DERIVE-THEN-JUDGE typestate. A floor whose members
//!     come from a live read (substrate's publish markers, from the crates.io index)
//!     cannot be judged before the read is consumed: `Requirements<Underived>` has no
//!     members and no judgment; [`Requirements::derive`] consumes the live versions to
//!     produce `Requirements<Derived>`, the only form the judge accepts. The types
//!     forbid judging a marker floor you never derived.
//!
//! Both are generation-tested, not mutated — like `discover::floor`, the correctness is
//! characterized by the judges that route through them, plus the probes below.

use std::marker::PhantomData;

/// A left join of named rows with a boolean annotation — substrate's tags joined with
/// their certified-line status. Each row is `(name, annotation)`, where the annotation
/// is `None` when the annotating read carried no row for that name.
pub struct Joined {
    rows: Vec<(String, Option<bool>)>,
}

impl Joined {
    /// Left-join `names` with `annotations` on the name. Every name appears; a name the
    /// annotations do not mention gets `None` (the annotating fact could not be read for
    /// it — refused by name, never assumed).
    pub fn left(names: &[String], annotations: &[(String, bool)]) -> Joined {
        let rows = names
            .iter()
            .map(|name| {
                let annotation = annotations
                    .iter()
                    .find(|(key, _)| key == name)
                    .map(|(_, value)| *value);
                (name.clone(), annotation)
            })
            .collect();
        Joined { rows }
    }

    /// The names (with annotations) matching `pred` — the filter half of a query.
    pub fn matching(&self, pred: impl Fn(&str) -> bool) -> Vec<(&str, Option<bool>)> {
        self.rows
            .iter()
            .filter(|(name, _)| pred(name))
            .map(|(name, annotation)| (name.as_str(), *annotation))
            .collect()
    }

    /// Is `name` present in the joined rows at all?
    pub fn has(&self, name: &str) -> bool {
        self.rows.iter().any(|(n, _)| n == name)
    }

    /// The annotation for `name`: `Some(Some(b))` present and annotated, `Some(None)`
    /// present but its annotation was unread, `None` absent entirely.
    pub fn annotation(&self, name: &str) -> Option<Option<bool>> {
        self.rows
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, annotation)| *annotation)
    }
}

/// The unjudged state of a live-derived floor — no members, no judgment.
pub struct Underived;
/// The judgeable state — its members were derived from the live read.
pub struct Derived;

/// A floor whose members are DERIVED from a live read, carrying the derive-then-judge
/// protocol in its type. Only [`Requirements<Derived>`] exposes its members, so the
/// judgment cannot consult a marker floor that was never derived.
pub struct Requirements<Phase> {
    members: Vec<String>,
    _phase: PhantomData<Phase>,
}

impl Requirements<Underived> {
    /// A floor awaiting its live read.
    pub fn awaiting() -> Requirements<Underived> {
        Requirements {
            members: Vec::new(),
            _phase: PhantomData,
        }
    }

    /// Consume the live published versions to derive one marker requirement each
    /// (`v<version>`) — the transition to the judgeable state.
    pub fn derive(self, published: &[String]) -> Requirements<Derived> {
        Requirements {
            members: published.iter().map(|v| format!("v{v}")).collect(),
            _phase: PhantomData,
        }
    }
}

impl Requirements<Derived> {
    /// The derived members — only reachable after `derive`.
    pub fn members(&self) -> &[String] {
        &self.members
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The left join is total and honest: every name appears, an annotated name carries
    /// its bool, and a name the annotations omit reads `None` — never assumed.
    #[test]
    fn the_left_join_annotates_every_name_and_refuses_the_absent() {
        let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let ann = vec![("a".to_string(), true), ("b".to_string(), false)];
        let j = Joined::left(&names, &ann);
        assert_eq!(j.annotation("a"), Some(Some(true)));
        assert_eq!(j.annotation("b"), Some(Some(false)));
        assert_eq!(
            j.annotation("c"),
            Some(None),
            "c present, annotation unread"
        );
        assert_eq!(j.annotation("d"), None, "d absent entirely");
        assert!(j.has("c") && !j.has("d"));
    }

    /// The filter selects exactly the matching rows, in order, carrying annotations.
    #[test]
    fn matching_selects_by_predicate() {
        let names = vec!["v1".to_string(), "v2".to_string(), "x".to_string()];
        let ann = vec![("v1".to_string(), true)];
        let j = Joined::left(&names, &ann);
        let vs: Vec<_> = j.matching(|n| n.starts_with('v'));
        assert_eq!(vs, vec![("v1", Some(true)), ("v2", None)]);
    }

    /// The typestate: an underived floor cannot be judged (no `members`); deriving from
    /// the live versions mints the `v<version>` markers and unlocks them.
    #[test]
    fn the_derive_typestate_gates_the_marker_floor() {
        let awaiting = Requirements::awaiting();
        let derived = awaiting.derive(&["0.1.0".to_string(), "0.2.0".to_string()]);
        assert_eq!(
            derived.members(),
            &["v0.1.0".to_string(), "v0.2.0".to_string()]
        );
    }
}
