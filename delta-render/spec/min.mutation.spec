# algebra mutation: min — 15 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `plus` evaluates as `zero`
- killed    `plus` evaluates as `neg`
- killed    `plus` evaluates as `min`
- killed    `neg` evaluates as `zero`
- killed    `neg` evaluates as `min`
- killed    `min` evaluates as `zero`
- killed    `min` evaluates as `neg`
- killed    `plus` returns its first argument unchanged
- killed    `plus` returns its second argument unchanged
- killed    `neg` returns its input unchanged
- killed    `min` returns its input unchanged
- killed    `zero` becomes undefined everywhere
- killed    `plus` becomes undefined everywhere
- killed    `neg` becomes undefined everywhere
- killed    `min` becomes undefined everywhere

# deafness floor: 24 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — 1 SURVIVED.
- SURVIVED  `min` goes deaf: always []

# dent sweep: 64 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 7 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `min` dented at [[(Row(0), -1)]]: [] -> [(Row(0), 1)]
- SURVIVED  `min` dented at [[(Row(0), -1)]]: [] -> [(Row(0), -1)]
- SURVIVED  `min` dented at [[(Row(0), 3)]]: [(Row(0), 1)] -> []
- SURVIVED  `min` dented at [[(Row(0), 2)]]: [(Row(0), 1)] -> []
- SURVIVED  `min` dented at [[(Row(0), -2)]]: [] -> [(Row(0), 1)]
- SURVIVED  `min` dented at [[(Row(0), 1), (Row(1), 1)]]: [(Row(0), 1)] -> []
- SURVIVED  `min` dented at [[(Row(0), 1), (Row(1), -2)]]: [(Row(0), 1)] -> []
