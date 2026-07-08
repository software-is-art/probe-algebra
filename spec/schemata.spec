# the schemata census, DERIVED — every expression-flip mutant this build
# carries (`#[mutate]` sites, registered at link time), one per line. The
# every-change `mutation (schemata)` gate builds once and runs the lib suite
# once per site with the flip active; survivors are ratified by key in
# spec/schemata.register or killed with a probe. Instrumenting a function
# (or moving its sites) is a ratified diff to this file. Regenerate with
# `cargo run --example freeze_gates`.
#
# 70 sites.

- agenda::of:deaf -> Err(String::new())
- agenda::of_with:0: ! -> (deleted)
- agenda::of_with:deaf -> Err(String::new())
- agenda::render:0: ! -> (deleted)
- agenda::render:1: ! -> (deleted)
- agenda::render:deaf -> String::new()
- agenda_render::edit_guard:0: ! -> (deleted)
- agenda_render::edit_guard:1: ! -> (deleted)
- agenda_render::edit_guard:2: && -> ||
- agenda_render::edit_guard:3: && -> ||
- agenda_render::edit_guard:4: ! -> (deleted)
- agenda_render::edit_guard:5: && -> ||
- agenda_render::edit_guard:deaf -> None
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
- classify:deaf -> Err(String::new())
- dependence::judge:0: == -> !=
- dependence::judge:1: == -> !=
- dependence::judge:2: == -> !=
- dependence::judge:3: == -> !=
- dependence::judge:deaf -> Err(String::new())
- dependence_report::is_intact:deaf -> false
- dependence_report::is_intact:deaf -> true
- dependence_report::render:deaf -> String::new()
- infra::coherent:0: == -> !=
- infra::coherent:1: == -> !=
- infra::coherent:deaf -> Err(vec![])
- infra::coherent:deaf -> Ok(vec![])
- infra::judge:0: == -> !=
- infra::judge:1: == -> !=
- infra::judge:2: == -> !=
- infra::judge:3: == -> !=
- infra::judge:deaf -> Err(vec![])
- infra::judge:deaf -> Ok(vec![])
- judge_register:0: || -> &&
- judge_register:1: || -> &&
- judge_register:2: == -> !=
- judge_register:deaf -> Err(String::new())
- judge_register:deaf -> Ok(vec![])
- perimeter::judge:0: == -> !=
- perimeter::judge:1: == -> !=
- perimeter::judge:deaf -> Err(vec![])
- perimeter::judge:deaf -> Ok(vec![])
- ratification::question:deaf -> String::new()
- shape_event::render:deaf -> String::new()
- substrate::judge:0: == -> !=
- substrate::judge:1: ! -> (deleted)
- substrate::judge:2: == -> !=
- substrate::judge:deaf -> Err(vec![])
- substrate::judge:deaf -> Ok(vec![])
- tag_law::matches:0: == -> !=
- tag_law::matches:deaf -> false
- tag_law::matches:deaf -> true
- ticker::parse_ops:deaf -> Err(String::new())
- ticker::parse_ops:deaf -> Ok(vec![])
- ticker_speak::hook_line:0: > -> <=
- ticker_speak::hook_line:deaf -> None
- ticker_speak::resume:0: ! -> (deleted)
- ticker_speak::step:deaf -> Err(String::new())
- ticker_speak::store:deaf -> Err(String::new())
- ticker_speak::store:deaf -> Ok(String::new())
- ticker_speak::store_signatures:deaf -> String::new()
