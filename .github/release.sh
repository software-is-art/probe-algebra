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
prev=$(git tag --list 'v2*' --sort=-creatordate | head -n 1)

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
    git diff "${prev}..HEAD" -- spec/ '*/spec/' | head -n 400
    echo '```'
    echo
    echo "## Signed by change"
    echo
    echo "The instance that made this release, named by what it ratified — the"
    echo "fingerprint hashes the spec-lock diff only, so interior refactors do not"
    echo "change the name; only changes to meaning do."
    echo
    moved=$(git diff --name-only "${prev}..HEAD" -- spec/ '*/spec/' | sed 's|.*/||' | sort -u | tr '\n' ' ')
    print=$(git diff "${prev}..HEAD" -- spec/ '*/spec/' | sha256sum | cut -c1-12)
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
