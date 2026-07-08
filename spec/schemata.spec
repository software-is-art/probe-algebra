# the schemata census, DERIVED — every expression-flip mutant this build
# carries (`#[mutate]` sites, registered at link time), one per line. The
# every-change `mutation (schemata)` gate builds once and runs the lib suite
# once per site with the flip active; survivors are ratified by key in
# spec/schemata.register or killed with a probe. Instrumenting a function
# (or moving its sites) is a ratified diff to this file. Regenerate with
# `cargo run --example freeze_gates`.
#
# 24 sites.

- classify:0: || -> &&
- classify:1: == -> !=
- classify:2: == -> !=
- classify:3: == -> !=
- classify:4: == -> !=
- classify:5: == -> !=
- classify:6: || -> &&
- classify:7: == -> !=
- classify:8: == -> !=
- classify:9: == -> !=
- infra::coherent:0: == -> !=
- infra::coherent:1: == -> !=
- infra::judge:0: == -> !=
- infra::judge:1: == -> !=
- infra::judge:2: == -> !=
- infra::judge:3: == -> !=
- judge_register:0: || -> &&
- judge_register:1: || -> &&
- judge_register:2: == -> !=
- perimeter::judge:0: == -> !=
- perimeter::judge:1: == -> !=
- substrate::judge:0: == -> !=
- substrate::judge:1: == -> !=
- tag_law::matches:0: == -> !=
