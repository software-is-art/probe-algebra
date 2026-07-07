#!/usr/bin/env bash
# perimeter — the settings are a lock: read the LIVE repository perimeter back and hold
# it to the declared floor (spec/perimeter.spec). READ-ONLY by design — the write stays
# human (a privilege is ratified, never self-served); this gate's whole job is to make
# settings drift LOUD, including the never-applied state. Runs on the weekly clock as
# the `perimeter (settings drift)` world gate; it never feeds the countersign.
set -euo pipefail

repo="${GITHUB_REPOSITORY:-software-is-art/probe-algebra}"
dir=$(mktemp -d)

# unreadable endpoints degrade to EMPTY payloads, which extract to the absent state —
# and absence refuses by name in the judge, never passes silently.
gh api "repos/${repo}/rules/branches/main" >"${dir}/rules.json" || echo '[]' >"${dir}/rules.json"
gh api "repos/${repo}" >"${dir}/repo.json" || echo '{}' >"${dir}/repo.json"
gh api "repos/${repo}/private-vulnerability-reporting" >"${dir}/pvr.json" || echo '{}' >"${dir}/pvr.json"

cargo run -q --example perimeter -- judge "${dir}/rules.json" "${dir}/repo.json" "${dir}/pvr.json"
