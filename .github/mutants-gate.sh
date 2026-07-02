#!/usr/bin/env bash
# The mutation gate, as the method defines it: "0 MISSED mutants".
#
# `cargo mutants` exits non-zero on TIMEOUTS as well as on missed mutants, but a timeout is a
# DETECTION here, not a survivor — a non-termination mutant (a greedy loop whose termination
# guard was relaxed, or a `Pos::index` pinned to a constant that hangs the lexer) makes the
# mutant hang while the original terminates. A real test suite with a per-test timeout would
# fail on it exactly the same way. (Honest caveat: a behaviour-changing mutant that ALSO runs
# past the timeout would be recorded as a timeout and passed — the timeout list is printed on
# every non-clean pass precisely so that residue stays reviewable, not silent.)
#
# The gate keys on the survivors recorded in `outcomes.json`, cross-checked against
# `missed.txt` — NOT on the raw exit code, and not on one file's absence:
#
#   - exit 0                            -> clean, pass.
#   - non-zero, zero recorded survivors -> only timeouts/unviable, all detections, pass.
#   - non-zero, any recorded survivor   -> real MISSED mutants, FAIL (and print them).
#   - non-zero, no outcomes at all      -> the run itself failed (baseline/build/usage), propagate.
#
# Args are forwarded verbatim to `cargo mutants` (e.g. `--in-diff pr.diff`). The gate assumes
# the default output dir; do not pass `--output`.
set -uo pipefail

cargo mutants "$@"
code=$?

if [ "$code" -eq 0 ]; then
    exit 0
fi

# A non-zero exit with no outcomes file means cargo-mutants failed before testing any mutant
# (baseline failure, build error, bad usage). That is a real failure — propagate it.
if [ ! -f mutants.out/outcomes.json ]; then
    echo "::error::cargo-mutants exited $code before producing outcomes (baseline/build/usage failure)"
    exit "$code"
fi

# Count survivors from outcomes.json itself (the source of truth), so a missing or renamed
# missed.txt can never read as a pass.
missed_count=$(grep -c '"MissedMutant"' mutants.out/outcomes.json || true)

if [ "${missed_count:-0}" -gt 0 ] || [ -s mutants.out/missed.txt ]; then
    echo "::error::mutation gate: surviving (MISSED) mutants — the probe suite has a hole:"
    cat mutants.out/missed.txt 2>/dev/null || grep -o '"MissedMutant"[^}]*' mutants.out/outcomes.json
    exit 1
fi

echo "Mutation gate PASSED: 0 missed mutants. cargo-mutants exited $code on timeouts only"
echo "(non-termination mutants are detections, not survivors — review the list):"
cat mutants.out/timeout.txt 2>/dev/null || true
exit 0
