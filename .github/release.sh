#!/usr/bin/env bash
# release — publish the certified tree, automatically.
#
# Runs as the pipeline's one Effectful gate (see discover::gates), immediately after a
# countersign advances the mutants-green tag: every fully-certified default-branch tree
# becomes a release, no human deciding "when". The version is CalVer (a date claims
# nothing about compatibility, which is honest); the notes are DERIVED — the commit
# subjects plus the ratified spec-lock diff, which is the uncompressed truth a semver
# integer would lossy-compress into an unchecked claim.
set -euo pipefail

# self-sufficient on shallow checkouts: the weekly countersign job clones depth-1, and
# the notes need tags plus history back to the previous release.
git fetch --tags --force >/dev/null 2>&1 || true
git fetch --unshallow >/dev/null 2>&1 || true

sha=$(git rev-parse --short HEAD)
prev=$(git tag --list 'v2*' --sort=-creatordate | awk 'NR == 1')

tag="v$(date -u +%Y.%m.%d)"
n=1
while git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; do
  n=$((n + 1))
  tag="v$(date -u +%Y.%m.%d).${n}"
done

notes=$(mktemp)
{
  echo "Certified tree \`${sha}\` — every gate green, the sweeps countersigned."
  echo
  if [ -n "${prev}" ]; then
    echo "## Since ${prev}"
    echo
    git log --format='- %s' "${prev}..HEAD"
    echo
    echo "## The ratified spec diff (the changelog IS the lock diff)"
    echo
    echo '```diff'
    # awk, not head: under pipefail, head closing the pipe at line 400 kills git
    # with SIGPIPE (exit 141) — which killed the first release whose spec diff
    # exceeded 400 lines. awk reads its whole input; same truncation, no signal.
    git diff "${prev}..HEAD" -- spec/ '*/spec/' | awk 'NR <= 400'
    echo '```'
    echo
    echo "## Signed by change"
    echo
    echo "The instance that made this release, named by what it ratified — the"
    echo "fingerprint hashes the spec-lock diff only, so interior refactors do not"
    echo "change the name; only changes to meaning do."
    echo
    moved=$(git diff --name-only "${prev}..HEAD" -- spec/ '*/spec/' | sed 's|.*/||' | sort -u | tr '\n' ' ')
    # --raw: blob hashes only, so the name is machine-invariant (a diff-algorithm
    # config difference must never split one identity into two).
    print=$(git diff --raw "${prev}..HEAD" -- spec/ '*/spec/' | sort | sha256sum | cut -c1-12)
    echo "- moved: ${moved:-nothing — a machinery-only release}"
    echo "- fingerprint: \`${print}\`"
  else
    echo "First automatic release — from here on, every certified tree publishes itself."
  fi
} >"${notes}"

git tag "${tag}"
git push origin "${tag}"
gh release create "${tag}" --title "${tag} — certified tree ${sha}" --notes-file "${notes}"
echo "released ${tag} (${sha})"

# crates.io rides the same event: any publishable crate whose manifest version is not
# yet on the registry publishes now, in dependency order. IDEMPOTENT by construction —
# an already-published version is skipped, so every certification is safe to re-run and
# a release with no version bump publishes nothing. A version bump is therefore a
# ratified DECISION (a manifest diff, reviewed like anything else; the semver signal is
# spec/shapes.spec per docs/publishing.md); shipping it is machinery. A publish failure
# fails the job loudly — a certified bump that cannot ship is a real error — but a
# missing token skips quietly (forks and checkouts without the secret still release).
if [ -n "${CARGO_REGISTRY_TOKEN:-}" ]; then
  for crate in spec-lock boundary-spec-macros boundary-enforce boundary-spec probe-hook; do
    version=$(cargo metadata --no-deps --format-version 1 |
      python3 -c "import json,sys; print(next(p['version'] for p in json.load(sys.stdin)['packages'] if p['name']=='${crate}'))")
    prefix2=$(printf '%s' "${crate}" | cut -c1-2)
    prefix4=$(printf '%s' "${crate}" | cut -c3-4)
    if curl -sSf "https://index.crates.io/${prefix2}/${prefix4}/${crate}" 2>/dev/null |
      grep -q "\"vers\":\"${version}\""; then
      echo "crates.io: ${crate} ${version} already published — skipping"
    else
      echo "crates.io: publishing ${crate} ${version}"
      cargo publish -p "${crate}"
      # the publish-marker law (spec/substrate.spec): every published root-crate
      # version marks the certified tree it shipped from. Minted HERE, at publish
      # time, so the derived law self-satisfies — the substrate gate only ever
      # reddens on a marker some out-of-band publish never minted.
      if [ "${crate}" = "boundary-spec" ]; then
        marker="v${version}"
        if ! git rev-parse -q --verify "refs/tags/${marker}" >/dev/null; then
          git tag "${marker}"
          git push origin "${marker}"
          echo "publish marker: ${marker} -> ${sha}"
        fi
      fi
    fi
  done
else
  echo "crates.io: CARGO_REGISTRY_TOKEN absent — GitHub release only"
fi
