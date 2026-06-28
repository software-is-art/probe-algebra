//! COVERAGE violation: a probe that omits a real degree of freedom must not be
//! accepted as complete. `Transaction` declares `Totals` AND `Multiplicity`, but
//! `OutputOnlyCheck` only `Covers<Totals>` — so `require_complete` cannot prove
//! `CoversAll` and the incomplete probe is a compile error, not a runtime gap. This is
//! the static twin of `synth`'s runtime coverage check: the LSP push-back a coding
//! agent gets for an incomplete probe before any test runs.
#![allow(unused_variables, unused_imports, dead_code)]

use probe_algebra::boundary::require_complete;
use probe_algebra::ledger::boundary::Transaction;
use probe_algebra::synth::OutputOnlyCheck;

fn main() {
    // `OutputOnlyCheck` is blind to multiplicity, so it does not `CoversAll` the DOFs
    // `Transaction` declares.
    require_complete::<Transaction, _>(&OutputOnlyCheck);
}
