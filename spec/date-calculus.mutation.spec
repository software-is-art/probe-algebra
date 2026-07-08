# algebra mutation: date calculus — 13 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `+` evaluates as `zero`
- killed    `diff` evaluates as `zero`
- killed    `diff` evaluates as `since`
- killed    `since` evaluates as `zero`
- killed    `+` returns its first argument unchanged
- killed    `+` returns its second argument unchanged
- killed    `add` returns its first argument unchanged
- killed    `zero` becomes undefined everywhere
- killed    `+` becomes undefined everywhere
- killed    `add` becomes undefined everywhere
- killed    `diff` becomes undefined everywhere
- killed    `since` becomes undefined everywhere
- killed    `at` becomes undefined everywhere

# deafness floor: 24 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — all killed: every operator's output provably depends on its input.

# dent sweep: 96 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 18 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `add` dented at [(0, 0), (1, 1)]: (0, 1) -> (0, 0)
- SURVIVED  `add` dented at [(0, 0), (1, 1)]: (0, 1) -> (0, 2)
- SURVIVED  `add` dented at [(0, 0), (1, 2)]: (0, 2) -> (0, 0)
- SURVIVED  `add` dented at [(0, 0), (1, 2)]: (0, 2) -> (0, 1)
- SURVIVED  `add` dented at [(0, 0), (1, 4)]: (0, 4) -> (0, 0)
- SURVIVED  `add` dented at [(0, 0), (1, 4)]: (0, 4) -> (0, 1)
- SURVIVED  `add` dented at [(0, 3), (1, 0)]: (0, 3) -> (0, 0)
- SURVIVED  `add` dented at [(0, 3), (1, 0)]: (0, 3) -> (0, 1)
- SURVIVED  `add` dented at [(0, 3), (1, 1)]: (0, 4) -> (0, 0)
- SURVIVED  `add` dented at [(0, 3), (1, 1)]: (0, 4) -> (0, 1)
- SURVIVED  `diff` dented at [(0, 1), (0, 0)]: (1, 1) -> (1, 0)
- SURVIVED  `diff` dented at [(0, 1), (0, 0)]: (1, 1) -> (1, 2)
- SURVIVED  `diff` dented at [(0, 2), (0, 0)]: (1, 2) -> (1, 0)
- SURVIVED  `diff` dented at [(0, 2), (0, 0)]: (1, 2) -> (1, 1)
- SURVIVED  `diff` dented at [(0, 2), (0, 1)]: (1, 1) -> (1, 0)
- SURVIVED  `diff` dented at [(0, 2), (0, 1)]: (1, 1) -> (1, 2)
- SURVIVED  `since` dented at [(0, 3)]: (1, 3) -> (1, 0)
- SURVIVED  `since` dented at [(0, 3)]: (1, 3) -> (1, 1)
