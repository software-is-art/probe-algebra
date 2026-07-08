# algebra mutation: interpreter arithmetic — 19 operator-table mutants, all killed — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `0` evaluates as `1`
- killed    `1` evaluates as `0`
- killed    `+` evaluates as `0`
- killed    `+` evaluates as `1`
- killed    `+` evaluates as `*`
- killed    `*` evaluates as `0`
- killed    `*` evaluates as `1`
- killed    `*` evaluates as `+`
- killed    `<` evaluates as `false`
- killed    `+` returns its first argument unchanged
- killed    `+` returns its second argument unchanged
- killed    `*` returns its first argument unchanged
- killed    `*` returns its second argument unchanged
- killed    `0` becomes undefined everywhere
- killed    `1` becomes undefined everywhere
- killed    `false` becomes undefined everywhere
- killed    `+` becomes undefined everywhere
- killed    `*` becomes undefined everywhere
- killed    `<` becomes undefined everywhere

# deafness floor: 18 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — all killed: every operator's output provably depends on its input.

# dent sweep: 80 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 18 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `+` dented at [(0, 1), (0, 2)]: (0, 3) -> (0, 0)
- SURVIVED  `+` dented at [(0, 1), (0, 2)]: (0, 3) -> (0, 1)
- SURVIVED  `+` dented at [(0, 1), (0, 7)]: (0, 8) -> (0, 0)
- SURVIVED  `+` dented at [(0, 1), (0, 7)]: (0, 8) -> (0, 1)
- SURVIVED  `<` dented at [(0, 0), (0, 1)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 0), (0, 2)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 0), (0, 3)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 0), (0, 4)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 0), (0, 5)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 0), (0, 6)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 0), (0, 7)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 1), (0, 0)]: (1, 0) -> (1, 1)
- SURVIVED  `<` dented at [(0, 1), (0, 2)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 1), (0, 3)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 1), (0, 4)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 1), (0, 5)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 1), (0, 6)]: (1, 1) -> (1, 0)
- SURVIVED  `<` dented at [(0, 1), (0, 7)]: (1, 1) -> (1, 0)
