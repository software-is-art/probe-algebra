//!
//! watch — the LIVE half of the placer: edit code, see the layout move.
//!
//! `Placement::over` works on raw string signatures, so deriving the layout needs no
//! compilation: this module parses the `ops { ... }` stanzas straight out of a source
//! file's text and re-places on every save. That is what makes "watch the architecture
//! form as you type" feasible where anything discovery-based would not be — placement is
//! microseconds from source text, discovery is a build away.
//!
//! Text is ONE source. A consumer that models its theory through the library API (in
//! test code, say) has no stanza text to parse — so the ticker also steps straight from
//! a compiled theory ([`Ticker::step_theory`]): the engine's signature table feeds the
//! same source-agnostic core, and code-modeled theories get the identical layout sense.
//!
//! Placement is monotone (an operator can seed a component, join one, or bridge two —
//! never re-split what it does not touch), so three verbs are the COMPLETE event
//! vocabulary of an edit session. [`Ticker::step`] diffs consecutive placements into
//! those events; `examples/place_watch.rs` is the loop that watches a file and prints
//! them. The goal is the lint experience: an agent writes operators into its workbench
//! and never thinks about layout — the ticker narrates the shape forming, the architect
//! offers the extraction when a component matures, and the shape lock gates the result.
//!
//! Parsing is refusal-shaped, house style: a line that looks like an operator but does
//! not parse is a named error, never a silent skip — a watcher that silently drops an
//! operator would narrate a wrong shape with full confidence.

use super::engine::Theory;
use super::shape::{NetSignature, Placement};

impl Ticker {
    /// Extract the operator signatures from a source file's `ops { ... }` stanzas.
    ///
    /// Grammar (the `theory!` ops line): `Fixity "Name" "symbol" (Sort, ...) -> Sort = fn;`
    /// Entries may wrap across lines; each ends at `;`. Sorts are compared by their final
    /// path segment (`Sort::Int` and `Int` are the same net), matching how `Placement::of`
    /// renders sorts by their Debug name.
    pub fn parse_ops(source: &str) -> Result<Vec<NetSignature>, String> {
        parse_ops(source)
    }
}

/// The parser body (private — reached as `Ticker::parse_ops`).
fn parse_ops(source: &str) -> Result<Vec<NetSignature>, String> {
    let mut sigs = Vec::new();
    let mut depth = 0usize; // > 0 while inside an `ops {` block
    let mut entry = String::new();
    for (line_no, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if depth == 0 {
            if trimmed == "ops {" || trimmed.ends_with(" ops {") {
                depth = 1;
            }
            continue;
        }
        if trimmed == "}" {
            if !entry.trim().is_empty() {
                return Err(format!(
                    "ops block closed mid-entry at line {}: `{}`",
                    line_no + 1,
                    entry.trim()
                ));
            }
            depth = 0;
            continue;
        }
        entry.push(' ');
        entry.push_str(trimmed);
        if trimmed.ends_with(';') {
            sigs.push(parse_entry(entry.trim(), line_no + 1)?);
            entry.clear();
        }
    }
    Ok(sigs)
}

/// One accumulated ops entry (ending in `;`) to a raw signature, or a named refusal.
fn parse_entry(entry: &str, line: usize) -> Result<NetSignature, String> {
    let refuse = |what: &str| format!("line {line}: {what} in ops entry `{entry}`");
    let mut quoted = entry.split('"');
    let (Some(_), Some(_name), Some(_), Some(symbol)) =
        (quoted.next(), quoted.next(), quoted.next(), quoted.next())
    else {
        return Err(refuse("expected two quoted strings (name, symbol)"));
    };
    let rest = quoted
        .next()
        .ok_or_else(|| refuse("nothing after symbol"))?;
    // `split_once` throughout — no index arithmetic to hold a degree of freedom the
    // probes cannot see (the first targeted sweep found exactly those as survivors).
    let (_, rest) = rest
        .split_once('(')
        .ok_or_else(|| refuse("no input list"))?;
    let (input_list, after) = rest
        .split_once(')')
        .ok_or_else(|| refuse("unclosed input list"))?;
    let inputs: Vec<String> = input_list
        .split(',')
        .map(net_name)
        .filter(|s| !s.is_empty())
        .collect();
    // `=` first, then `->` within what precedes it: an entry whose arrow comes after
    // its meaning (`(A) = f -> B;`) has no output sort, structurally — no guard needed.
    let (before_eq, _) = after
        .split_once('=')
        .ok_or_else(|| refuse("no `= fn` meaning"))?;
    let (_, output_raw) = before_eq
        .split_once("->")
        .ok_or_else(|| refuse("no output sort"))?;
    let output = net_name(output_raw);
    if output.is_empty() {
        return Err(refuse("empty output sort"));
    }
    // the symbol is leaked: signatures are `&'static str`-keyed like the engine's, and a
    // watcher's working set is one file's operators — bounded, re-parsed in place.
    Ok((
        Box::leak(symbol.to_string().into_boxed_str()),
        inputs,
        output,
    ))
}

/// A sort's net name: the final path segment, trimmed (`Sort::Int` → `Int`).
fn net_name(raw: &str) -> String {
    raw.trim()
        .rsplit("::")
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

/// What one edit did to the layout — the complete vocabulary, by monotonicity.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ShapeEvent {
    /// New operators started a component of their own: a module is forming.
    Seeded { ops: Vec<String> },
    /// New operators landed inside existing components: local, boundary unchanged.
    Joined { ops: Vec<String> },
    /// The edit connected previously separate components: the boundary moved. The
    /// alarm case when unintended — a conversion just coupled two features.
    Bridged { from: usize, to: usize },
    /// Operators left (or a refactor landed): the shape was re-derived, not diffed.
    Rederived,
}

impl ShapeEvent {
    /// The event as a ticker line.
    pub fn render(&self) -> String {
        match self {
            ShapeEvent::Seeded { ops } => {
                format!("seeded  a new module is forming: {{ {} }}", ops.join(", "))
            }
            ShapeEvent::Joined { ops } => {
                format!("joined  {{ {} }} — boundary unchanged", ops.join(", "))
            }
            ShapeEvent::Bridged { from, to } => {
                format!("BRIDGED  {from} modules became {to} — the boundary moved (intended?)")
            }
            ShapeEvent::Rederived => "re-derived  operators changed non-additively".to_string(),
        }
    }
}

/// The watcher's state: the previous placement, diffed against each new parse.
#[derive(Default)]
pub struct Ticker {
    previous: Option<Placement>,
}

impl Ticker {
    /// A fresh ticker (the first step reports the whole shape as `Seeded`).
    pub fn new() -> Ticker {
        Ticker::default()
    }

    /// A ticker RESUMED from stored signatures — the hook path: each hook invocation is
    /// a fresh process, so the previous placement is rebuilt from the state the last
    /// invocation persisted (one signature per line, `symbol\tin,in\tout`).
    pub fn resume(name: &'static str, stored: &str) -> Ticker {
        let sigs: Vec<NetSignature> = stored
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('\t');
                let (symbol, ins, out) = (parts.next()?, parts.next()?, parts.next()?);
                Some((
                    Box::leak(symbol.to_string().into_boxed_str()) as &'static str,
                    ins.split(',')
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect(),
                    out.to_string(),
                ))
            })
            .collect();
        Ticker {
            previous: Some(Placement::over(name, sigs)),
        }
    }

    /// The current signatures in [`Ticker::resume`]'s stored form — what a hook persists
    /// between invocations.
    pub fn store(source: &str) -> Result<String, String> {
        Ok(Ticker::store_signatures(&parse_ops(source)?))
    }

    /// [`Ticker::store`]'s form for raw signatures — the type path's persistence, so a
    /// code-modeled theory survives a process boundary the same way stanza text does.
    pub fn store_signatures(sigs: &[NetSignature]) -> String {
        sigs.iter()
            .map(|(symbol, ins, out)| format!("{symbol}\t{}\t{out}\n", ins.join(",")))
            .collect()
    }

    /// The hook's one-line verdict for an edit, or `None` — the noise policy that makes
    /// the agent surface affordable: `Bridged` always speaks (a coupling is the alarm),
    /// a `Seeded` speaks only when it opens a SECOND component (a new disjoint feature —
    /// the extraction becomes available), and everything else is silence, because the
    /// common case must cost zero tokens.
    pub fn hook_line(&mut self, name: &'static str, source: &str) -> Option<String> {
        let (placement, event) = self.step(name, source).ok()?;
        match event? {
            ShapeEvent::Bridged { from, to } => Some(format!(
                "shape: BRIDGED {from}->{to} in {name} — the edit coupled previously \
                 separate features (intended?)"
            )),
            ShapeEvent::Seeded { ops } if placement.components.len() > 1 => Some(format!(
                "shape: {name} places as {} modules — {{ {} }} is net-disjoint; the \
                 extraction is available (Architect::place)",
                placement.components.len(),
                ops.join(", ")
            )),
            _ => None,
        }
    }

    /// Place the source as `name`, classify the change against the previous step, and
    /// remember the new shape. Returns the placement and the event (None on a no-op
    /// edit — same operators, same shape).
    pub fn step(
        &mut self,
        name: &'static str,
        source: &str,
    ) -> Result<(Placement, Option<ShapeEvent>), String> {
        Ok(self.step_signatures(name, parse_ops(source)?))
    }

    /// The SECOND SOURCE: step from a compiled theory instead of stanza text. A
    /// consumer that models its theory through the library API (in test code, say) has
    /// no `ops { ... }` text for the parser — the type path reads the same signatures
    /// off the engine, so code-modeled theories get the identical live layout sense.
    pub fn step_theory<T: Theory>(&mut self) -> (Placement, Option<ShapeEvent>) {
        self.step_signatures(T::name(), Placement::signatures_of::<T>())
    }

    /// Step from raw signatures — the source-agnostic core both fronts share (text
    /// parses into it, the type path reads the engine into it). Infallible: the
    /// signatures are already structured, so there is nothing left to refuse.
    pub fn step_signatures(
        &mut self,
        name: &'static str,
        sigs: Vec<NetSignature>,
    ) -> (Placement, Option<ShapeEvent>) {
        let placement = Placement::over(name, sigs.clone());
        let event = match &self.previous {
            None => Some(ShapeEvent::Seeded {
                ops: placement
                    .components
                    .iter()
                    .flat_map(|c| &c.ops)
                    .map(|s| s.to_string())
                    .collect(),
            }),
            Some(prev) => diff(prev, &placement),
        };
        self.previous = Some(Placement::over(name, sigs));
        (placement, event)
    }
}

/// Classify one step against the previous placement. Additive edits use the monotone
/// vocabulary; anything non-additive (an operator renamed or removed) is `Rederived`.
fn diff(prev: &Placement, next: &Placement) -> Option<ShapeEvent> {
    let ops_of = |p: &Placement| -> Vec<String> {
        p.components
            .iter()
            .flat_map(|c| &c.ops)
            .map(|s| s.to_string())
            .collect()
    };
    let (old_ops, new_ops) = (ops_of(prev), ops_of(next));
    let added: Vec<String> = new_ops
        .iter()
        .filter(|op| !old_ops.contains(op))
        .cloned()
        .collect();
    if old_ops.iter().any(|op| !new_ops.contains(op)) {
        return Some(ShapeEvent::Rederived);
    }
    if added.is_empty() {
        return None;
    }
    if next.components.len() < prev.components.len() {
        return Some(ShapeEvent::Bridged {
            from: prev.components.len(),
            to: next.components.len(),
        });
    }
    if next.components.len() > prev.components.len() {
        return Some(ShapeEvent::Seeded { ops: added });
    }
    Some(ShapeEvent::Joined { ops: added })
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::date::Calendar;
    use crate::discover::router::Router;
    use crate::discover::shape::Placement;

    /// THE PARSER AGAINST REALITY: placement computed from the SOURCE TEXT of the
    /// repo's own theories equals placement computed from the compiled types — the
    /// text path and the type path derive the same shape, so the live view never
    /// narrates a layout the build would disagree with.
    #[test]
    fn source_parsed_placement_matches_compiled_placement() {
        for (file, compiled) in [
            ("src/discover/arithmetic.rs", Placement::of::<Arithmetic>()),
            ("src/discover/router.rs", Placement::of::<Router>()),
            ("src/discover/date.rs", Placement::of::<Calendar>()),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
            let source = std::fs::read_to_string(path).expect("repo source");
            // through the PUBLIC surface, so the wrapper is load-bearing, not decoration.
            let parsed = Placement::over("parsed", Ticker::parse_ops(&source).expect("parses"));
            assert_eq!(
                parsed.components.len(),
                compiled.components.len(),
                "{file}: component counts diverge"
            );
            for (a, b) in parsed.components.iter().zip(&compiled.components) {
                assert_eq!(a.ops, b.ops, "{file}: component members diverge");
                assert_eq!(a.nets, b.nets, "{file}: nets diverge");
            }
        }
    }

    /// THE SECOND SOURCE: a theory modeled in CODE gets the same live layout sense —
    /// no stanza text anywhere. The type path's steps match the text path's on the
    /// repo's own theory (same placement, same first event), an unchanged theory is
    /// silence, and store/resume carries the type path across a process boundary
    /// exactly like the hook's text path.
    #[test]
    fn the_type_path_ticks_like_the_text_path() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/discover/date.rs");
        let source = std::fs::read_to_string(path).expect("repo source");

        let mut by_type = Ticker::new();
        let (p_type, e_type) = by_type.step_theory::<Calendar>();
        let mut by_text = Ticker::new();
        let (p_text, e_text) = by_text.step("date calculus", &source).expect("parses");
        assert_eq!(p_type.components.len(), p_text.components.len());
        for (a, b) in p_type.components.iter().zip(&p_text.components) {
            assert_eq!(a.ops, b.ops, "component members diverge across sources");
            assert_eq!(a.nets, b.nets, "nets diverge across sources");
        }
        assert_eq!(e_type, e_text, "the first event is the same seed");

        // an unchanged theory is silence, and a re-step after resume stays silent:
        assert_eq!(by_type.step_theory::<Calendar>().1, None);
        let stored = Ticker::store_signatures(&Placement::signatures_of::<Calendar>());
        let mut resumed = Ticker::resume("date calculus", &stored);
        assert_eq!(resumed.step_theory::<Calendar>().1, None);
    }

    /// An edit session, narrated: the first material seeds, same-net material joins,
    /// disjoint material seeds a second module, and a conversion BRIDGES them — the
    /// complete monotone vocabulary, in the order an agent would produce it.
    #[test]
    fn a_session_narrates_seed_join_seed_bridge() {
        let stage = |ops: &str| format!("    ops {{\n{ops}    }}\n");
        let counter = "        Nullary \"zero\" \"zero\" () -> S::A = zero;\n";
        let bump = "        Prefix \"bump\" \"bump\" (S::A) -> S::A = bump;\n";
        let flag = "        Nullary \"off\" \"off\" () -> S::B = off;\n";
        let bridge = "        Prefix \"read\" \"read\" (S::B) -> S::A = read;\n";

        let mut ticker = Ticker::new();
        let (p, e) = ticker.step("bench", &stage(counter)).unwrap();
        assert!(p.is_settled());
        assert_eq!(
            e,
            Some(ShapeEvent::Seeded {
                ops: vec!["zero".into()]
            })
        );

        let joined = format!("{counter}{bump}");
        let (p, e) = ticker.step("bench", &stage(&joined)).unwrap();
        assert!(p.is_settled());
        assert_eq!(
            e,
            Some(ShapeEvent::Joined {
                ops: vec!["bump".into()]
            })
        );
        assert!(e.unwrap().render().contains("boundary unchanged"));

        let two = format!("{counter}{bump}{flag}");
        let (p, e) = ticker.step("bench", &stage(&two)).unwrap();
        assert_eq!(p.components.len(), 2, "a second module is forming");
        assert_eq!(
            e,
            Some(ShapeEvent::Seeded {
                ops: vec!["off".into()]
            })
        );

        let coupled = format!("{counter}{bump}{flag}{bridge}");
        let (p, e) = ticker.step("bench", &stage(&coupled)).unwrap();
        assert!(p.is_settled(), "the conversion coupled the features");
        assert_eq!(e, Some(ShapeEvent::Bridged { from: 2, to: 1 }));
        assert!(e.unwrap().render().contains("intended?"));

        // a save with no operator change is silent; a removal re-derives.
        let (_, e) = ticker.step("bench", &stage(&coupled)).unwrap();
        assert_eq!(e, None);
        let (_, e) = ticker.step("bench", &stage(&two)).unwrap();
        assert_eq!(e, Some(ShapeEvent::Rederived));
    }

    /// THE AGENT SURFACE: `hook_line` prices feedback correctly — silence is free.
    /// Joined edits (the common case) say nothing; a seed that opens a second
    /// component announces the available extraction; a bridge always speaks, because
    /// coupling two features is the alarm the hook exists for. And the store/resume
    /// pair survives the hook's process boundary: each invocation rebuilds the
    /// previous placement from persisted signatures.
    #[test]
    fn the_hook_speaks_only_when_the_shape_moves() {
        let stage = |ops: &str| format!("    ops {{\n{ops}    }}\n");
        let counter = "        Nullary \"zero\" \"zero\" () -> S::A = zero;\n";
        let bump = "        Prefix \"bump\" \"bump\" (S::A) -> S::A = bump;\n";
        let flag = "        Nullary \"off\" \"off\" () -> S::B = off;\n";
        let bridge = "        Prefix \"read\" \"read\" (S::B) -> S::A = read;\n";

        // baseline capture, then a joined edit: both silent, through store/resume.
        let stored = Ticker::store(&stage(counter)).unwrap();
        let joined = format!("{counter}{bump}");
        let mut t = Ticker::resume("bench", &stored);
        assert_eq!(
            t.hook_line("bench", &stage(&joined)),
            None,
            "joined is free"
        );

        // a second component: one line, naming the extraction.
        let two = format!("{counter}{bump}{flag}");
        let mut t = Ticker::resume("bench", &Ticker::store(&stage(&joined)).unwrap());
        let line = t
            .hook_line("bench", &stage(&two))
            .expect("a seed that splits speaks");
        assert!(line.contains("places as 2 modules"), "{line}");
        assert!(line.contains("extraction is available"));

        // a bridge: always one line, always the question.
        let coupled = format!("{counter}{bump}{flag}{bridge}");
        let mut t = Ticker::resume("bench", &Ticker::store(&stage(&two)).unwrap());
        let line = t
            .hook_line("bench", &stage(&coupled))
            .expect("a bridge speaks");
        assert_eq!(
            line,
            "shape: BRIDGED 2->1 in bench — the edit coupled previously separate \
             features (intended?)"
        );

        // an unparseable edit is silence to the HOOK (the refusal surfaces in the
        // watcher and the tests, not mid-keystroke), and a no-op edit is silence.
        let mut t = Ticker::resume("bench", &Ticker::store(&stage(&coupled)).unwrap());
        assert_eq!(t.hook_line("bench", "ops { garbage"), None);
        assert_eq!(t.hook_line("bench", &stage(&coupled)), None);
    }

    /// A line that looks like an operator but does not parse is a NAMED refusal — a
    /// watcher that silently dropped it would narrate a wrong shape confidently.
    #[test]
    fn malformed_ops_entries_refuse_by_name() {
        let bad =
            |entry: &str| parse_ops(&format!("    ops {{\n        {entry}\n    }}\n")).unwrap_err();
        assert!(bad("Infix \"+\" (S::A) -> S::A = add;").contains("two quoted strings"));
        assert!(bad("Infix \"Add\" \"+\" S::A -> S::A = add;").contains("no input list"));
        assert!(bad("Infix \"Add\" \"+\" (S::A = add;").contains("unclosed input list"));
        assert!(bad("Infix \"Add\" \"+\" (S::A) = add;").contains("no output sort"));
        assert!(bad("Infix \"Add\" \"+\" (S::A) -> S::A;").contains("no `= fn` meaning"));
        // an arrow AFTER the meaning is still "no output sort" — structure, not a guard.
        assert!(bad("Infix \"Add\" \"+\" (S::A) = add -> S::A;").contains("no output sort"));
        // the refusal names the 1-based source line (entries sit on line 2 of `bad`).
        assert!(bad("Infix \"+\" (S::A) -> S::A = add;").contains("line 2:"));
        // bare (path-free) sorts are nets too — pins the output slice against off-by-one
        // drift, since `net_name` cannot repair an unpathed sort.
        let sigs = Ticker::parse_ops(
            "    ops {\n        Infix \"Add\" \"+\" (Int, Int) -> Int = add;\n    }\n",
        )
        .unwrap();
        assert_eq!(
            sigs,
            vec![("+", vec!["Int".into(), "Int".into()], "Int".into())]
        );
        // an entry left open when the block closes is refused, not truncated.
        let err = parse_ops("    ops {\n        Infix \"Add\" \"+\" (S::A) -> S::A = add\n    }\n")
            .unwrap_err();
        assert!(err.contains("closed mid-entry at line 3"), "{err}");
        // and a file with no ops block parses to an empty, settled placement.
        assert!(parse_ops("fn main() {}").unwrap().is_empty());
    }
}
