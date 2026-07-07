#!/bin/bash
set -euo pipefail

# The ENCODED form of "use rtk" (Rust Token Killer, github.com/rtk-ai/rtk) — a rule as
# machinery, not as a CLAUDE.md instruction. The PreToolUse hook in .claude/settings.json
# routes agent Bash commands through rtk, which compresses noisy dev-command output
# (cargo test ~90%) before it reaches the model; this SessionStart hook only guarantees
# the binary exists in remote (web) containers. Container state is cached after the hook
# completes, so the one-time build amortizes across sessions.
#
# Pinned rev, house style: bumping rtk is a reviewed diff to this file, never ambient.

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

command -v rtk >/dev/null 2>&1 ||
  cargo install --git https://github.com/rtk-ai/rtk --rev d823aaf7 --locked
