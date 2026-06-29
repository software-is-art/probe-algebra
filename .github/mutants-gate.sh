#!/usr/bin/env bash
# The mutation gate, as the method defines it: "0 MISSED mutants".
#
# `cargo mutants` exits non-zero on TIMEOUTS as well as on missed mutants, but a timeout is a
# DETECTION here, not a survivor — a non-termination mutant (a greedy loop whose termination
# guard was relaxed, or a `Pos::index` pinned to a constant that hangs the lexer) makes the
# mutant hang while the original terminates. A real test suite with a per-test timeout would
# fail on it exactly the same way. So the gate keys on `missed.txt`, the survivors, not on the
# raw exit code:
#
#   - exit 0                       -> clean, pass.
#   - non-zero, `missed.txt` empty -> only timeouts/unviable, all detections, pass.
#   - non-zero, `missed.txt` full  -> real survivors, FAIL (and print them).
#   - non-zero, no outcomes at all -> the run itself failed (baseline/build/usage), propagate.
#
# Args are forwarded verbatim to `cargo mutants` (e.g. `--in-diff pr.diff`).
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

# Real survivors are the only thing that fails the gate.
if [ -s mutants.out/missed.txt ]; then
    echo "::error::mutation gate: surviving (MISSED) mutants — the probe suite has a hole:"
    cat mutants.out/missed.txt
    exit 1
fi

echo "Mutation gate PASSED: 0 missed mutants. cargo-mutants exited $code on timeouts only"
echo "(non-termination mutants are detections, not survivors):"
cat mutants.out/timeout.txt 2>/dev/null || true
exit 0
