//! A date/duration calculus, as a `Theory` — a MULTI-SORTED domain with a PARTIAL operator and a
//! round-trip pair, to show the engine discovers algebra across sorts and through partiality.
//!
//! Two sorts: `Date` (days since an epoch) and `Duration` (a non-negative span of days). `plus`
//! combines durations (a commutative monoid with `zero`); `add` shifts a date by a duration; `diff`
//! is PARTIAL (a later date minus an earlier one — `None` when it would go negative); `since`/`at`
//! convert between a date and its days-from-epoch, a round-trip. The engine discovers the duration
//! monoid, the round trip, AND the duration ACTION on dates (`add(d, zero) = d`, and repeated `add`
//! combining its parameters with `plus`) — so every operator participates in a law.

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

// The whole multi-sorted `Theory` impl is generated from this block — only the value object
// (`Time`) and the operator functions above are authored.
crate::theory! {
    Calendar : "date calculus", Value = Time, Obs = (u8, u32), Sort = Sort,
    sort_of = |v: &Time| match v {
        Time::Date(_) => Sort::Date,
        Time::Dur(_) => Sort::Duration,
    },
    observe = |v: &Time| match v {
        Time::Date(d) => (0u8, *d),
        Time::Dur(d) => (1u8, *d),
    },
    vars {
        Sort::Date => &["s", "t", "u"],
        Sort::Duration => &["p", "q", "r"],
    }
    inhabit {
        Sort::Date => [0u32, 1, 2, 3, 5, 8].into_iter().map(Time::Date).collect(),
        Sort::Duration => [0u32, 1, 2, 4].into_iter().map(Time::Dur).collect(),
    }
    ops {
        Nullary "Zero"  "zero"  () -> Sort::Duration = zero;
        Infix   "Plus"  "+"     (Sort::Duration, Sort::Duration) -> Sort::Duration = plus;
        Prefix  "Add"   "add"   (Sort::Date, Sort::Duration) -> Sort::Date = add;
        Prefix  "Diff"  "diff"  (Sort::Date, Sort::Date) -> Sort::Duration = diff;
        Prefix  "since" "since" (Sort::Date) -> Sort::Duration = since;
        Prefix  "at"    "at"    (Sort::Duration) -> Sort::Date = at;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::engine::{Engine, Theory};

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
