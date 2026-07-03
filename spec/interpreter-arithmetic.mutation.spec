# algebra mutation: interpreter arithmetic — 19 operator-table mutants, 1 SURVIVED — regenerate via `cargo run --example freeze_spec`; ratify the diff.
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
- SURVIVED  `<` evaluates as `false`
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
