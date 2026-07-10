# algebra mutation: tally — 9 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `merge` evaluates as `floor`
- killed    `merge` evaluates as `bump`
- killed    `bump` evaluates as `floor`
- killed    `merge` returns its first argument unchanged
- killed    `merge` returns its second argument unchanged
- killed    `bump` returns its input unchanged
- killed    `merge` becomes undefined everywhere
- killed    `floor` becomes undefined everywhere
- killed    `bump` becomes undefined everywhere

# deafness floor: 6 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — 3 SURVIVED.
- SURVIVED  `bump` goes deaf: always T0
- SURVIVED  `bump` goes deaf: always T1
- SURVIVED  `bump` goes deaf: always T2

# dent sweep: 24 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 3 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `bump` dented at [T0]: T1 -> T0
- SURVIVED  `bump` dented at [T0]: T1 -> T2
- SURVIVED  `bump` dented at [T1]: T2 -> T1
