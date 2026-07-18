# the repository perimeter, DECLARED — the settings floor the weekly world gate
# (`perimeter (settings drift)`) reads back from the live API and refuses by name.
# Settings drift silently and nobody re-audits a settings page; this lock does.
# The WRITE stays human (a privilege is ratified, never self-served): apply
# spec/perimeter.ruleset.json once —
#   gh api repos/<owner>/<repo>/rulesets -X POST --input spec/perimeter.ruleset.json
# — and enable private vulnerability reporting in the repository's security
# settings. Extra live protections beyond this floor are NOT drift (stricter is
# never a lie); required approvals are the one exact match, because a count above
# zero deadlocks a solo maintainer. Regenerate with `cargo run --example freeze_gates`.

- pull requests required before merging; required approvals: 0 (a solo
  maintainer cannot approve their own PR — the gates are the reviewer)
- required status checks: fmt + clippy + test
- merge methods: squash only
- force pushes to the default branch: blocked
- deletion of the default branch: blocked
- private vulnerability reporting: enabled
- auto-merge: enabled (green is the merge decision — the gates are the
  reviewer; a merge waiting on attention after green is the queue)
