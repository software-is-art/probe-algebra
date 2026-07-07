#!/bin/bash
# The bridge alarm, as machinery (PostToolUse on Edit|Write): after an agent edits a
# theory file, re-derive the placement from source text and inject one line into the
# loop ONLY when the shape moved — a bridge (two features coupled) always speaks, a new
# net-disjoint component announces its extraction, everything else is free silence.
# See discover::watch (Ticker::hook_line) for the policy; the logic lives in
# mutation-tested Rust, this script is the envelope. Fail-open everywhere: a broken
# hook degrades to no feedback, never to a broken edit loop.
set -uo pipefail

input=$(cat) || exit 0
path=$(python3 -c '
import json, sys
try:
    data = json.load(sys.stdin)
    print(data.get("tool_input", {}).get("file_path", ""))
except Exception:
    pass
' <<<"${input}") || exit 0

case "${path}" in
*.rs) ;;
*) exit 0 ;;
esac
[ -f "${path}" ] || exit 0
# only theory files pay the cargo invocation.
grep -q "ops {" "${path}" 2>/dev/null || exit 0

cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
cargo run -q --example place_watch -- --event "${path}" target/shape-watch 2>/dev/null
exit 0
