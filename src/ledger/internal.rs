//! internal — PRIVATE implementation detail of the ledger module.
//!
//! Nothing here is part of any boundary; other modules cannot name it. The
//! boundary delegates the raw, primitive-typed algorithm to this module and
//! wraps the result back up in value objects. This is where "messy" code is
//! allowed to live — behind the interface, not at it.

use std::collections::BTreeMap;

/// Raw fold: `(account, amount)` pairs -> (per-account totals, per-account
/// sorted breakdown of the amounts that summed to each total).
pub(super) fn fold(
    postings: &[(String, i64)],
) -> (BTreeMap<String, i64>, BTreeMap<String, Vec<i64>>) {
    let mut totals: BTreeMap<String, i64> = BTreeMap::new();
    let mut breakdown: BTreeMap<String, Vec<i64>> = BTreeMap::new();
    for (acct, amt) in postings {
        *totals.entry(acct.clone()).or_insert(0) += *amt;
        breakdown.entry(acct.clone()).or_default().push(*amt);
    }
    for v in breakdown.values_mut() {
        v.sort();
    }
    (totals, breakdown)
}
