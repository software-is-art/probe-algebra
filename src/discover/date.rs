//! A date/duration calculus, as a `Theory` — a MULTI-SORTED domain with a PARTIAL operator and a
//! round-trip pair, to show the engine discovers algebra across sorts and through partiality.
//!
//! Two sorts: `Date` (days since an epoch) and `Duration` (a non-negative span of days). `plus`
//! combines durations (a commutative monoid with `zero`); `add` shifts a date by a duration; `diff`
//! is PARTIAL (a later date minus an earlier one — `None` when it would go negative); `since`/`at`
//! convert between a date and its days-from-epoch, a round-trip. The engine discovers the duration
//! monoid, the round trip, AND the duration ACTION on dates (`add(d, zero) = d`, and repeated `add`
//! combining its parameters with `plus`) — so every operator participates in a law.

use super::engine::{Fixity, Operator, Theory};

/// A value in the calculus: a date or a duration (days).
#[derive(Clone)]
pub enum Time {
    Date(u32),
    Dur(u32),
}

/// The date/duration theory.
pub struct Calendar;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sort {
    Date,
    Duration,
}

fn date(v: &Time) -> u32 {
    match v {
        Time::Date(d) => *d,
        Time::Dur(d) => *d,
    }
}

fn zero(_: &[Time]) -> Option<Time> {
    Some(Time::Dur(0))
}
fn plus(v: &[Time]) -> Option<Time> {
    Some(Time::Dur(date(&v[0]) + date(&v[1])))
}
fn add(v: &[Time]) -> Option<Time> {
    Some(Time::Date(date(&v[0]) + date(&v[1])))
}
fn diff(v: &[Time]) -> Option<Time> {
    let (a, b) = (date(&v[0]), date(&v[1]));
    (a >= b).then(|| Time::Dur(a - b)) // PARTIAL: no negative durations
}
fn since(v: &[Time]) -> Option<Time> {
    Some(Time::Dur(date(&v[0])))
}
fn at(v: &[Time]) -> Option<Time> {
    Some(Time::Date(date(&v[0])))
}

impl Theory for Calendar {
    type Sort = Sort;
    type Value = Time;
    type Obs = (u8, u32);

    fn name() -> &'static str {
        "date calculus"
    }

    fn operators() -> Vec<Operator<Self>> {
        use Fixity::{Infix, Nullary, Prefix};
        use Sort::{Date, Duration};
        vec![
            Operator {
                name: "Zero",
                symbol: "zero",
                fixity: Nullary,
                inputs: vec![],
                output: Duration,
                eval: zero,
            },
            Operator {
                name: "Plus",
                symbol: "+",
                fixity: Infix,
                inputs: vec![Duration, Duration],
                output: Duration,
                eval: plus,
            },
            Operator {
                name: "Add",
                symbol: "add",
                fixity: Prefix,
                inputs: vec![Date, Duration],
                output: Date,
                eval: add,
            },
            Operator {
                name: "Diff",
                symbol: "diff",
                fixity: Prefix,
                inputs: vec![Date, Date],
                output: Duration,
                eval: diff,
            },
            Operator {
                name: "since",
                symbol: "since",
                fixity: Prefix,
                inputs: vec![Date],
                output: Duration,
                eval: since,
            },
            Operator {
                name: "at",
                symbol: "at",
                fixity: Prefix,
                inputs: vec![Duration],
                output: Date,
                eval: at,
            },
        ]
    }

    fn inhabitants(sort: Self::Sort) -> Vec<Self::Value> {
        match sort {
            Sort::Date => [0u32, 1, 2, 3, 5, 8].into_iter().map(Time::Date).collect(),
            Sort::Duration => [0u32, 1, 2, 4].into_iter().map(Time::Dur).collect(),
        }
    }

    fn sort_of(value: &Self::Value) -> Self::Sort {
        match value {
            Time::Date(_) => Sort::Date,
            Time::Dur(_) => Sort::Duration,
        }
    }

    fn observe(value: &Self::Value) -> Self::Obs {
        match value {
            Time::Date(d) => (0, *d),
            Time::Dur(d) => (1, *d),
        }
    }

    fn sort_vars(sort: Self::Sort) -> &'static [&'static str] {
        match sort {
            Sort::Date => &["s", "t", "u"],
            Sort::Duration => &["p", "q", "r"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::Engine;

    /// The engine discovers, across two sorts and through a PARTIAL operator, the duration monoid
    /// (commutativity, associativity, identity), the duration ACTION on dates (`add(d, zero) = d`,
    /// and repeated `add` combining its parameters with `plus`), the self-difference law, and BOTH
    /// round trips — so every operator participates in a law.
    #[test]
    fn the_calendar_algebra_is_discovered() {
        assert_eq!(Calendar::name(), "date calculus");
        let e = Engine::<Calendar>::new();
        let d = e.discover();
        let got: Vec<(String, String)> = d
            .laws
            .iter()
            .map(|l| (l.prose.clone(), l.equation.clone()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "Plus gives the same result in either order.",
                "(p + q) = (q + p)",
            ),
            (
                "With Plus, the grouping of three values doesn't matter.",
                "((p + q) + r) = (p + (q + r))",
            ),
            ("Plus with zero leaves a value unchanged.", "(zero + p) = p"),
            // the duration ACTION on dates — now discovered by the action templates.
            (
                "Add with zero leaves a value unchanged.",
                "add(s, zero) = s",
            ),
            (
                "Repeated Add combines its parameters with Plus.",
                "add(add(s, p), q) = add(s, (p + q))",
            ),
            (
                "Diff of a value with itself gives zero.",
                "diff(s, s) = zero",
            ),
            (
                "at undoes since — the round trip is the identity.",
                "at(since(s)) = s",
            ),
            (
                "since undoes at — the round trip is the identity.",
                "since(at(p)) = p",
            ),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, q)| (p.to_string(), q.to_string()))
            .collect();
        assert_eq!(got, expected, "the discovered calendar algebra changed");
        assert_eq!(d.consequences, 248);
        // every operator now participates in a law (the action templates cover `add`).
        assert!(
            d.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            d.uncovered_ops
        );
        assert_eq!(e.check(&d.laws), Ok(()));
    }
}
