#!/usr/bin/env bash
# substrate — git itself is a lock: read the LIVE repository's tags and history back
# and hold them to the declared substrate (spec/substrate.spec). READ-ONLY — the reads
# are git plumbing against the checkout's own origin; no third-party API, no extra
# credential. Runs on the weekly clock as the `substrate (git drift)` world gate; it
# never feeds the countersign.
set -euo pipefail

# CI checks out shallow and tagless; deepen honestly (a failed fetch leaves empty
# reads, which extract to the absent state — and absence refuses by name downstream).
if [ "$(git rev-parse --is-shallow-repository)" = "true" ]; then
  git fetch -q --unshallow --tags origin main || true
else
  git fetch -q --tags origin main || true
fi
line="origin/main"
git rev-parse -q --verify "$line" >/dev/null || line="HEAD"

dir=$(mktemp -d)
git tag --list >"${dir}/tags.txt"
# the linearity epoch comes from the DECLARATION — the script never restates it.
epoch=$(cargo run -q --example substrate -- epoch)
git rev-list --min-parents=2 --count "${epoch}..${line}" >"${dir}/merges.txt" || : >"${dir}/merges.txt"
while IFS= read -r tag; do
  if git merge-base --is-ancestor "refs/tags/${tag}" "$line"; then
    echo "${tag} on-main"
  else
    echo "${tag} stray"
  fi
done <"${dir}/tags.txt" >"${dir}/ancestry.txt"

cargo run -q --example substrate -- judge "${dir}/tags.txt" "${dir}/ancestry.txt" "${dir}/merges.txt"
