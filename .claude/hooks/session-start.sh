#!/bin/bash
set -euo pipefail

# Two jobs, both encoded rules rather than instructions:
#
# 1. TOPOGRAPHY, injected at session start: the derived shape, read from the COMMITTED
#    lock — never computed, never timestamped — so the injection is byte-stable between
#    ratified shape changes. Prefix-stability is the design constraint: startup-injected
#    content shares the prompt-cache prefix across sessions, so it must derive from
#    committed text only; a volatile line here would shorten every session's shared
#    prefix. When this output changes, the shape genuinely changed — exactly the news
#    worth a cache miss.
#
# 2. Remote-container provisioning: rtk (Rust Token Killer, pinned rev — bumping it is
#    a reviewed diff) and the prebuilt hook binaries, so an edit pays a process spawn,
#    not a compile. Container state is cached after the hook completes.

cd "${CLAUDE_PROJECT_DIR:-.}" 2>/dev/null || true

if [ -f "spec/boundary-spec.shape.spec" ]; then
  echo "## topography (derived — the committed shape lock; see docs/discovery.md)"
  grep -E "^- |^verdict:" spec/boundary-spec.shape.spec || true
fi

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

command -v rtk >/dev/null 2>&1 ||
  cargo install --git https://github.com/rtk-ai/rtk --rev d823aaf7 --locked
cargo build -q --example place_watch --example review_agenda 2>/dev/null || true
