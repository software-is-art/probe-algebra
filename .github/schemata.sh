#!/usr/bin/env bash
# schemata — the compiled-mutant sweep: ONE build carries every expression-flip and
# deafness mutant (`#[mutate]` sites, spec/schemata.spec), and each verdict is a test
# run with the flip selected via PROBE_MUTANT — never a rebuild (the selector is read
# at runtime, so the cached test binary serves every site). A run that stays green
# with a flip active is a SURVIVOR, judged against the ratified register
# (spec/schemata.register) with set-difference semantics. Pure and cheap, so it rides
# EVERY change alongside fmt/clippy/test.
#
# Economics, all derived: nextest's fail-fast stops a killed mutant at its first
# failing probe, and the timeout comes from a TIMED baseline run (5x + buffer), never
# a hand-picked number — a mutant that exceeds it is a DETECTION (non-termination is
# a behavioural difference the suite provably exposed), same doctrine as the source
# sweeps.
set -euo pipefail

runner() { # one suite run, fail-fast where nextest exists
  if command -v cargo-nextest >/dev/null 2>&1; then
    cargo nextest run -p boundary-spec --lib --fail-fast >/dev/null 2>&1
  else
    cargo test -q -p boundary-spec --lib >/dev/null 2>&1
  fi
}
export -f runner

# build the test binary and the census runner once; every per-site run reuses them.
cargo test -q -p boundary-spec --lib --no-run
dir=$(mktemp -d)
cargo run -q --example schemata -- list >"${dir}/sites.txt"

# the baseline: unmutated, timed — it must be green (a red baseline would make every
# mutant verdict meaningless), and its duration prices the timeout.
start=$(date +%s)
runner || {
  echo "schemata: the UNMUTATED suite is red — fix the suite before judging mutants"
  exit 1
}
baseline=$(($(date +%s) - start))
limit=$((baseline * 5 + 10))
echo "baseline ${baseline}s green; per-mutant timeout ${limit}s (derived, 5x + 10)"

: >"${dir}/survivors.txt"
while IFS= read -r site; do
  if timeout "${limit}" env PROBE_MUTANT="${site}" bash -c runner; then
    echo "SURVIVED ${site}"
    echo "${site}" >>"${dir}/survivors.txt"
  else
    echo "killed   ${site}"
  fi
done <"${dir}/sites.txt"

cargo run -q --example schemata -- judge "${dir}/survivors.txt"
