# algebra mutation: router — 5 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `or` evaluates as `empty`
- killed    `or` returns its first argument unchanged
- killed    `or` returns its second argument unchanged
- killed    `empty` becomes undefined everywhere
- killed    `or` becomes undefined everywhere

# deafness floor: 6 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — all killed: every operator's output provably depends on its input.

# dent sweep: 32 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 3 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `or` dented at [[Some(1), None, None, None], [None, Some(2), None, None]]: [Some(1), Some(2), None, None] -> [None, None, None, None]
- SURVIVED  `or` dented at [[Some(1), None, None, None], [None, Some(2), None, None]]: [Some(1), Some(2), None, None] -> [Some(1), None, None, None]
- SURVIVED  `or` dented at [[None, Some(2), None, None], [Some(1), None, None, None]]: [Some(1), Some(2), None, None] -> [Some(1), None, None, None]
