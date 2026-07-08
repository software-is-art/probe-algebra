#!/usr/bin/env bash
# schemata — the compiled-mutant sweep: ONE build carries every expression-flip and
# deafness mutant (`#[mutate]` sites, spec/schemata.spec), and each verdict is a test
# run with the flip selected via PROBE_MUTANT — never a rebuild (the selector is read
# at runtime, so the cached test binary serves every site). A run that stays green
# with a flip active is a SURVIVOR, judged against the ratified register
# (spec/schemata.register) with set-difference semantics. Pure and cheap, so it rides
# EVERY change alongside fmt/clippy/test.
#
# Economics, all derived:
#   - the BASELINE run is also the COVERAGE run (SCHEMATA_RECORD): nextest runs one
#     test per process, so each process's touch-file is the site→test edge list. A
#     mutant can only change behaviour where its guard executes, so running just the
#     covering tests is EXACT, not an approximation — and a site NO test reaches is a
#     survivor before any run (unexecuted is unkillable, disclosed, never assumed).
#   - fail-fast stops a killed mutant at its first failing probe;
#   - the timeout prices itself from the timed baseline (never hand-picked); a mutant
#     that exceeds it is a DETECTION, same doctrine as the source sweeps.
# Without nextest (no per-test processes, no coverage granularity) the sweep falls
# back to the full suite per mutant — slower, same verdicts.
set -euo pipefail

# build the test binary and the census runner once; every per-site run reuses them.
cargo test -q -p boundary-spec --lib --no-run
dir=$(mktemp -d)
cargo run -q --example schemata -- list >"${dir}/sites.txt"

have_nextest=0
command -v cargo-nextest >/dev/null 2>&1 && have_nextest=1

# the baseline: unmutated, timed, RECORDED — it must be green (a red baseline would
# make every mutant verdict meaningless), and its duration prices the timeout.
mkdir -p "${dir}/record"
start=$(date +%s)
if [ "${have_nextest}" = "1" ]; then
  SCHEMATA_RECORD="${dir}/record" cargo nextest run -p boundary-spec --lib >/dev/null 2>&1 || {
    echo "schemata: the UNMUTATED suite is red — fix the suite before judging mutants"
    exit 1
  }
else
  cargo test -q -p boundary-spec --lib >/dev/null 2>&1 || {
    echo "schemata: the UNMUTATED suite is red — fix the suite before judging mutants"
    exit 1
  }
fi
baseline=$(($(date +%s) - start))
limit=$((baseline * 5 + 10))
echo "baseline ${baseline}s green; per-mutant timeout ${limit}s (derived, 5x + 10)"

# the plan: one line per site — `<site>\t<covering tests>` (tab-separated; empty
# test list means uncovered). Assembled from the touch-files in plain python.
python3 - "${dir}" <<'PYEOF'
import glob, os, sys

dir = sys.argv[1]
coverage = {}
for path in glob.glob(os.path.join(dir, "record", "*.touch")):
    tests, sites = [], []
    for line in open(path):
        if line.startswith("T "):
            tests = line[2:].split()
        elif line.startswith("S "):
            sites.append(line[2:].strip())
    for site in sites:
        coverage.setdefault(site, set()).update(tests)
with open(os.path.join(dir, "plan.txt"), "w") as out:
    for site in open(os.path.join(dir, "sites.txt")):
        site = site.strip()
        if site:
            out.write(site + "\t" + " ".join(sorted(coverage.get(site, []))) + "\n")
PYEOF

: >"${dir}/survivors.txt"
covered_runs=0
while IFS=$'\t' read -r site tests; do
  if [ "${have_nextest}" = "1" ]; then
    if [ -z "${tests}" ]; then
      # no test executes this site's guard: unkillable, disclosed immediately.
      echo "SURVIVED ${site} (uncovered — no test reaches it)"
      echo "${site}" >>"${dir}/survivors.txt"
      continue
    fi
    expr=""
    for t in ${tests}; do
      [ -n "${expr}" ] && expr="${expr} or "
      expr="${expr}test(=${t})"
    done
    covered_runs=$((covered_runs + 1))
    if timeout "${limit}" env PROBE_MUTANT="${site}" \
      cargo nextest run -p boundary-spec --lib --fail-fast -E "${expr}" >/dev/null 2>&1; then
      echo "SURVIVED ${site}"
      echo "${site}" >>"${dir}/survivors.txt"
    else
      echo "killed   ${site}"
    fi
  else
    if timeout "${limit}" env PROBE_MUTANT="${site}" \
      cargo test -q -p boundary-spec --lib >/dev/null 2>&1; then
      echo "SURVIVED ${site}"
      echo "${site}" >>"${dir}/survivors.txt"
    else
      echo "killed   ${site}"
    fi
  fi
done <"${dir}/plan.txt"

cargo run -q --example schemata -- judge "${dir}/survivors.txt"
