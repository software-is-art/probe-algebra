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

# FREEDOM, same discipline as topography: read from the COMMITTED mutation locks, so
# the line is byte-stable between ratified survivor changes. This is the agent's sense
# for open degrees of freedom — every SURVIVED line is a mutant the committed spec
# cannot refute (deafness constants, unpinned dent coordinates, table confusions),
# each one an address where a sharper shape or expectation would go.
if ls spec/*.mutation.spec >/dev/null 2>&1; then
  survivors=$(grep -h -c "^- SURVIVED" spec/*.mutation.spec 2>/dev/null | paste -sd+ - | bc)
  locks=$(ls spec/*.mutation.spec | wc -l | tr -d ' ')
  echo
  echo "## freedom (derived — the committed mutation locks)"
  echo "${survivors:-0} ratified survivor(s) across ${locks} mutation lock(s) — each a named degree of freedom (grep SURVIVED spec/*.mutation.spec)"
fi

# SITUATION, the volatile layer: small, git-derived, entropy-free (no timestamps —
# deterministic given repo state). Startup injection is once-per-session, so this
# never churns the within-session cache prefix; it exists so a session's first minutes
# are not spent re-deriving where it woke up.
if git rev-parse --git-dir >/dev/null 2>&1; then
  echo
  echo "## situation (volatile — this session's starting point)"
  branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "?")
  ahead=$(git rev-list --count origin/main..HEAD 2>/dev/null || echo "?")
  dirty=$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')
  echo "branch: ${branch} — ${ahead} commit(s) ahead of origin/main, ${dirty} uncommitted file(s)"
  echo "recent work:"
  git log --format='- %s' -3 2>/dev/null || true
  release=$(git tag --list 'v2*' --sort=-creatordate 2>/dev/null | head -n 1)
  if [ -n "${release}" ]; then
    echo "last release: ${release}, $(git rev-list --count "${release}..HEAD" 2>/dev/null || echo "?") commit(s) ago"
  fi
  if git rev-parse -q --verify refs/tags/mutants-green >/dev/null 2>&1; then
    echo "certified tree (mutants-green): $(git rev-list --count mutants-green..HEAD 2>/dev/null || echo "?") commit(s) behind HEAD"
  fi
fi

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

command -v rtk >/dev/null 2>&1 ||
  cargo install --git https://github.com/rtk-ai/rtk --rev d823aaf7 --locked
# the edit-time envelope is the SHIPPED binary now (probe-hook), not a bash wrapper:
# put it on PATH so the PostToolUse `command: "probe-hook"` resolves. Fail-open.
command -v probe-hook >/dev/null 2>&1 ||
  cargo install --path probe-hook --quiet 2>/dev/null || true
# nextest is the schemata sweep's fail-fast runner: without it `schemata verify`
# degrades to whole-suite evidence (carries nothing, judges everything — hours);
# with it the incremental tier works as designed (minutes). Paid once per container
# cache, like rtk. Fail-open: the sweep still runs without it, just honestly slower.
command -v cargo-nextest >/dev/null 2>&1 ||
  cargo install cargo-nextest --locked --quiet >/dev/null 2>&1 || true
# the PINNED SUIT, from session one: without this the first verb of every fresh
# container pays a full workspace build mid-conversation. Quiet and fail-open;
# stdout stays byte-stable (the injection shares the prompt-cache prefix).
[ -x .suit/bundle ] ||
  cargo run --quiet --example bundle -- pin >/dev/null 2>&1 || true
