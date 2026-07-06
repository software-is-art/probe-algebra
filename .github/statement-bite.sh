#!/usr/bin/env bash
# statement-bite gate: definition mutants of the Lean corpus, judged by the Lean
# kernel, survivors held against lean/bites.register (see discover::bite).
#
# The corpus is core-only (no lake, no mathlib), so the toolchain is a single elan
# install. LEAN_TOOLCHAIN floats at `stable` for the bootstrap runs; pin it here once
# the first countersigned run reports its resolved version in the gate log.
set -euo pipefail

if ! command -v lean >/dev/null 2>&1 && [ ! -x "$HOME/.elan/bin/lean" ]; then
  curl -sSfL https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh \
    | sh -s -- -y --default-toolchain "${LEAN_TOOLCHAIN:-stable}"
fi
export PATH="$HOME/.elan/bin:$PATH"

exec cargo run --example statement_bite
