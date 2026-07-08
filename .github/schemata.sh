#!/usr/bin/env bash
# schemata — the compiled-mutant sweep: ONE build carries every expression-flip mutant
# (`#[mutate]` sites, spec/schemata.spec), and each verdict is a test run with the
# flip selected via PROBE_MUTANT — never a rebuild (the selector is read at runtime,
# so the cached test binary serves every site). A run that stays green with a flip
# active is a SURVIVOR, judged against the ratified register (spec/schemata.register)
# with set-difference semantics. Pure: a deterministic function of the tree.
set -euo pipefail

# build the lib test binary and the runner once; every per-site run below reuses them.
cargo test -q -p boundary-spec --lib --no-run
dir=$(mktemp -d)
cargo run -q --example schemata -- list >"${dir}/sites.txt"

: >"${dir}/survivors.txt"
while IFS= read -r site; do
  if PROBE_MUTANT="${site}" cargo test -q -p boundary-spec --lib >/dev/null 2>&1; then
    echo "SURVIVED ${site}"
    echo "${site}" >>"${dir}/survivors.txt"
  else
    echo "killed   ${site}"
  fi
done <"${dir}/sites.txt"

cargo run -q --example schemata -- judge "${dir}/survivors.txt"
