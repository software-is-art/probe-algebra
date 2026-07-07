#!/usr/bin/env bash
# statement-bite gate: definition mutants of the Lean corpus, judged by the Lean
# kernel, survivors held against lean/bites.register (see discover::bite).
#
# The corpus is core-only (no lake, no mathlib), so the toolchain is a single elan
# install. PINNED to the version the first kernel run resolved and agreed with the
# mirror on (run 84: 18 planted, 14 killed, 4 survived — the bnand set, exactly as
# the register ratifies); bumping the pin is a reviewed diff like any other.
set -euo pipefail

LEAN_TOOLCHAIN="${LEAN_TOOLCHAIN:-leanprover/lean4:v4.31.0}"

if ! command -v lean >/dev/null 2>&1 && [ ! -x "$HOME/.elan/bin/lean" ]; then
  curl -sSfL https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh \
    | sh -s -- -y --default-toolchain "$LEAN_TOOLCHAIN"
fi
export PATH="$HOME/.elan/bin:$PATH"

exec cargo run --example statement_bite
