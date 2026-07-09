# algebra mutation: doc flow — 5 operator-table mutants, 2 SURVIVED — regenerate via this repo's freeze path; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `edit` returns its input unchanged
- killed    `submit` becomes undefined everywhere
- killed    `revise` becomes undefined everywhere
- SURVIVED  `approve` becomes undefined everywhere
- SURVIVED  `edit` becomes undefined everywhere

# deafness floor: 33 constant-return mutants (every operator × every distinct
# output), judged by re-checking the discovered laws — 15 SURVIVED.
- SURVIVED  `approve` goes deaf: always ("Published", ("spec", 1))
- SURVIVED  `approve` goes deaf: always ("Published", ("hello", 1))
- SURVIVED  `approve` goes deaf: always ("Published", ("spec", 2))
- SURVIVED  `edit` goes deaf: always ("Draft", ("", 0))
- SURVIVED  `edit` goes deaf: always ("Draft", ("hello", 0))
- SURVIVED  `edit` goes deaf: always ("Draft", ("spec", 1))
- SURVIVED  `edit` goes deaf: always ("Draft", ("", 1))
- SURVIVED  `edit` goes deaf: always ("Draft", ("hello", 1))
- SURVIVED  `edit` goes deaf: always ("Draft", ("spec", 2))
- SURVIVED  `edit` goes deaf: always ("Draft", ("", 2))
- SURVIVED  `edit` goes deaf: always ("Draft", ("hello", 2))
- SURVIVED  `edit` goes deaf: always ("Draft", ("spec", 3))
- SURVIVED  `edit` goes deaf: always ("Draft", ("", 3))
- SURVIVED  `edit` goes deaf: always ("Draft", ("hello", 3))
- SURVIVED  `edit` goes deaf: always ("Draft", ("spec", 4))

# dent sweep: 62 one-point mutants (first 16 grid points per operator,
# 2 wrong outputs per point — a resource bound, not a curated list), judged by
# re-checking the discovered laws — 38 SURVIVED — each an UNPINNED COORDINATE, the exact input a missing probe would constrain.
- SURVIVED  `submit` dented at [("Draft", ("hello", 3))]: ("Review", ("hello", 3)) -> ("Review", ("hello", 0))
- SURVIVED  `submit` dented at [("Draft", ("hello", 3))]: ("Review", ("hello", 3)) -> ("Review", ("spec", 1))
- SURVIVED  `submit` dented at [("Draft", ("spec", 4))]: ("Review", ("spec", 4)) -> ("Review", ("hello", 0))
- SURVIVED  `submit` dented at [("Draft", ("spec", 4))]: ("Review", ("spec", 4)) -> ("Review", ("spec", 1))
- SURVIVED  `approve` dented at [("Review", ("spec", 1))]: ("Published", ("spec", 1)) -> ("Published", ("hello", 1))
- SURVIVED  `approve` dented at [("Review", ("spec", 1))]: ("Published", ("spec", 1)) -> ("Published", ("spec", 2))
- SURVIVED  `approve` dented at [("Review", ("hello", 1))]: ("Published", ("hello", 1)) -> ("Published", ("spec", 1))
- SURVIVED  `approve` dented at [("Review", ("hello", 1))]: ("Published", ("hello", 1)) -> ("Published", ("spec", 2))
- SURVIVED  `approve` dented at [("Review", ("spec", 2))]: ("Published", ("spec", 2)) -> ("Published", ("spec", 1))
- SURVIVED  `approve` dented at [("Review", ("spec", 2))]: ("Published", ("spec", 2)) -> ("Published", ("hello", 1))
- SURVIVED  `approve` dented at [("Review", ("hello", 2))]: ("Published", ("hello", 2)) -> ("Published", ("spec", 1))
- SURVIVED  `approve` dented at [("Review", ("hello", 2))]: ("Published", ("hello", 2)) -> ("Published", ("hello", 1))
- SURVIVED  `approve` dented at [("Review", ("spec", 3))]: ("Published", ("spec", 3)) -> ("Published", ("spec", 1))
- SURVIVED  `approve` dented at [("Review", ("spec", 3))]: ("Published", ("spec", 3)) -> ("Published", ("hello", 1))
- SURVIVED  `edit` dented at [("Draft", ("", 0))]: ("Draft", ("", 1)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 0))]: ("Draft", ("", 1)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 0))]: ("Draft", ("hello", 1)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 0))]: ("Draft", ("hello", 1)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 1))]: ("Draft", ("spec", 2)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 1))]: ("Draft", ("spec", 2)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 1))]: ("Draft", ("", 2)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 1))]: ("Draft", ("", 2)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 1))]: ("Draft", ("hello", 2)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 1))]: ("Draft", ("hello", 2)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 2))]: ("Draft", ("spec", 3)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 2))]: ("Draft", ("spec", 3)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 2))]: ("Draft", ("", 3)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 2))]: ("Draft", ("", 3)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 2))]: ("Draft", ("hello", 3)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 2))]: ("Draft", ("hello", 3)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 3))]: ("Draft", ("spec", 4)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 3))]: ("Draft", ("spec", 4)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 3))]: ("Draft", ("", 4)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("", 3))]: ("Draft", ("", 4)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 3))]: ("Draft", ("hello", 4)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("hello", 3))]: ("Draft", ("hello", 4)) -> ("Draft", ("hello", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 4))]: ("Draft", ("spec", 5)) -> ("Draft", ("", 0))
- SURVIVED  `edit` dented at [("Draft", ("spec", 4))]: ("Draft", ("spec", 5)) -> ("Draft", ("hello", 0))
