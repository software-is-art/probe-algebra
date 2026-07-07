#!/bin/bash
# The edit-time feedback loop, as machinery (PostToolUse on Edit|Write). Two voices,
# both priced in silence: the GUARD pre-fires refusals that already exist downstream
# (hand-editing a generated lock; a loose `pub fn` the shim will refuse), and the SHAPE
# ticker speaks when a theory edit moves the layout (a bridge, a new net-disjoint
# component). Logic lives in mutation-tested Rust (discover::agenda::edit_guard,
# discover::watch::Ticker); this script is the envelope. Binaries are prebuilt by the
# SessionStart hook so an edit pays a process spawn, not a compile — with a build
# fallback if they are missing. Fail-open everywhere: a broken hook degrades to no
# feedback, never to a broken edit loop.
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
[ -n "${path}" ] || exit 0

cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0

run_example() { # <name> <args...>: prebuilt binary if present, else quiet build
  local name="$1"
  shift
  local bin="target/debug/examples/${name}"
  [ -x "${bin}" ] || cargo build -q --example "${name}" 2>/dev/null || return 0
  "${bin}" "$@" 2>/dev/null
}

run_example review_agenda --guard "${path}"

case "${path}" in
*.rs) ;;
*) exit 0 ;;
esac
[ -f "${path}" ] || exit 0
grep -q "ops {" "${path}" 2>/dev/null || exit 0
run_example place_watch --event "${path}" target/shape-watch
exit 0
