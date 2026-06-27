//! money::boundary — a spike: value objects as TAGGED primitives (see
//! `crate::boundary::Qty`).
//!
//! Most domain concepts over a primitive do NOT partition it — they have the full
//! domain but are a distinct NAMED concept. Here such a concept costs a zero-size
//! kind tag plus two trait impls and gets ALL arithmetic for free (`Balance`,
//! `Points`); a partitioning concept adds only a validity rule (`Cents`). No
//! hand-written operator sets — contrast `Cents`/`Balance`/`Quantity`/`Register`
//! in the other modules, which spell out `add`/`sub`/`zero`/`negate` by hand.
//!
//! This still passes the tier-1 boundary grammar: the tags are unit structs, the
//! rest are impls and type aliases, and no raw primitive ever appears here — it
//! lives in `Qty`, in the grammar.

use crate::boundary::{Kind, Qty, Total};

/// A running balance: a distinct concept over the FULL `i64` domain, so its
/// arithmetic is total. Zero hand-written operators.
pub struct BalanceKind;
impl Kind for BalanceKind {
    fn admits(_: i64) -> bool {
        true
    }
}
impl Total for BalanceKind {}
pub type Balance = Qty<BalanceKind>;

/// Loyalty points: the same shape, a DIFFERENT concept — it will not unify with
/// `Balance` even though both are full-domain `i64`.
pub struct PointsKind;
impl Kind for PointsKind {
    fn admits(_: i64) -> bool {
        true
    }
}
impl Total for PointsKind {}
pub type Points = Qty<PointsKind>;

/// Cents: a PARTITIONING concept (bounded range). It gets CHECKED arithmetic from
/// the same generic code; the ONLY per-type code is the validity rule.
pub struct CentsKind;
impl Kind for CentsKind {
    fn admits(n: i64) -> bool {
        n.abs() <= 100_000_000
    }
}
pub type Cents = Qty<CentsKind>;
