//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! watch — the LIVE half of the placer: edit code, see the layout move.
//!
//! `Placement::over` works on raw string signatures, so deriving the layout needs no
//! compilation: this module parses the `ops { ... }` stanzas straight out of a source
//! file's text and re-places on every save. That is what makes "watch the architecture
//! form as you type" feasible where anything discovery-based would not be — placement is
//! microseconds from source text, discovery is a build away.
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
    let open = rest.find('(').ok_or_else(|| refuse("no input list"))?;
    let close = rest
        .find(')')
        .ok_or_else(|| refuse("unclosed input list"))?;
    let inputs: Vec<String> = rest[open + 1..close]
        .split(',')
        .map(net_name)
        .filter(|s| !s.is_empty())
        .collect();
    let after = &rest[close + 1..];
    let arrow = after.find("->").ok_or_else(|| refuse("no output sort"))?;
    let eq = after.find('=').ok_or_else(|| refuse("no `= fn` meaning"))?;
    if eq < arrow {
        return Err(refuse("no output sort"));
    }
    let output = net_name(&after[arrow + 2..eq]);
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

    /// Place the source as `name`, classify the change against the previous step, and
    /// remember the new shape. Returns the placement and the event (None on a no-op
    /// edit — same operators, same shape).
    pub fn step(
        &mut self,
        name: &'static str,
        source: &str,
    ) -> Result<(Placement, Option<ShapeEvent>), String> {
        let sigs = parse_ops(source)?;
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
        Ok((placement, event))
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
            let parsed = Placement::over("parsed", parse_ops(&source).expect("parses"));
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
        // an entry left open when the block closes is refused, not truncated.
        let err = parse_ops("    ops {\n        Infix \"Add\" \"+\" (S::A) -> S::A = add\n    }\n")
            .unwrap_err();
        assert!(err.contains("closed mid-entry"), "{err}");
        // and a file with no ops block parses to an empty, settled placement.
        assert!(parse_ops("fn main() {}").unwrap().is_empty());
    }
}
