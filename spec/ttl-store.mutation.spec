# algebra mutation: ttl store — 13 operator-table mutants, all killed — regenerate via `cargo run --example freeze_spec`; ratify the diff.
#
# Every mutant is a perturbed operator table (a VALUE, not a build), judged by
# re-running discovery: KILLED means the named-law set changed — the committed
# spec would go stale against an implementation with this bug. A SURVIVOR means
# the ratified law language cannot tell the mutant from the real thing on this
# grid — a named degree of freedom (the bias-blindness precedent), to be closed
# by a sharper shape or expectation, or ratified here as a free choice.

- killed    `<+` evaluates as `empty`
- killed    `tick` evaluates as `empty`
- killed    `+` evaluates as `zero`
- killed    `<+` returns its first argument unchanged
- killed    `<+` returns its second argument unchanged
- killed    `tick` returns its first argument unchanged
- killed    `+` returns its first argument unchanged
- killed    `+` returns its second argument unchanged
- killed    `empty` becomes undefined everywhere
- killed    `<+` becomes undefined everywhere
- killed    `tick` becomes undefined everywhere
- killed    `zero` becomes undefined everywhere
- killed    `+` becomes undefined everywhere
