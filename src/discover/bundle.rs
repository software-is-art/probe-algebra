//!
//! bundle — the module as a DATABASE: rung 1 of the continuation candidate ("the bundle is
//! the source", docs/roadmap.md). An existing module parses into a BUNDLE — each top-level
//! item a verbatim text segment, the operator functions identified by the sixth sense's net
//! model — and renders back in CANONICAL PLACED ORDER. The rung's whole claim is fidelity:
//! a module whose operators are already component-grouped round-trips BYTE FOR BYTE (pinned
//! on a real committed module below), and a scrambled one renders back canonically — the
//! reorder is the render's only opinion. No generation until this holds.
//!
//! One vocabulary, three reuses, nothing restated: the net model is
//! [`Ticker::parse_rust_sigs`] (own types are the nets, ubiquitous types never are), the
//! grouping is [`Placement::over`] (components in first-appearance order, members in
//! declaration order — monotone under addition, which is what makes a purely-additive
//! author safe), and the span→text cut is genesis's own `byte_offset`.
//!
//! Disclosed limits, each a later rung's business, none silent: only TOP-LEVEL operator
//! functions are re-dealt — an impl-attached operator rides its impl block whole, and an
//! inline module rides as one item (grouping inside either is not this rung's). Separator
//! trivia (blank lines, a floating `//` comment between items) belongs to the POSITION, not
//! the item: a reorder dresses the dealt items in the original positions' spacing, while doc
//! comments — attributes inside the item's span — travel with their item. And the operator
//! identity is the function NAME (the watch module's own disclosed limitation), so a
//! top-level fn shadowed by an inner-module namesake would confuse the dealing — refused at
//! parse, never guessed.

use std::collections::BTreeSet;

use super::genesis::byte_offset;
use super::shape::Placement;
use super::watch::Ticker;
use syn::spanned::Spanned;

/// One parsed module as a bundle: the preamble (module docs — everything before the first
/// item), each top-level item split into its verbatim TEXT (docs through last token) and the
/// separator GAP after it (whitespace and floating comments up to the next item), and the
/// canonical operator order the placer derives. `render` re-deals the operator texts into
/// the operator positions in canonical order, keeping each position's gap where it stood;
/// every other byte is carried verbatim.
#[derive(Debug)]
pub struct Bundle {
    preamble: String,
    items: Vec<BundleItem>,
    /// Placement components in first-appearance order, members in declaration order,
    /// filtered to the top-level operator functions — the canonical dealing order.
    canonical: Vec<String>,
}

/// One top-level item: its operator name when the net model reads it as a top-level
/// operator function, its verbatim text, and the separator trivia after it (which belongs
/// to the POSITION — see the module doc).
#[derive(Debug)]
struct BundleItem {
    operator: Option<String>,
    text: String,
    gap: String,
}

#[crate::mutate("bundle")]
impl Bundle {
    /// Parse a module's source into its bundle. Refusals are named, never guessed: an
    /// unparseable module (the half-written-edit case), or an operator name that is not
    /// unique across the file's functions (the dealing would be ambiguous).
    pub fn parse(source: &str) -> Result<Bundle, String> {
        let file = syn::parse_file(source).map_err(|e| format!("bundle: unparseable: {e}"))?;
        let sigs = Ticker::parse_rust_sigs(source)?;

        // the top-level operator names: functions at item level whose signature nets on an
        // own type. The net model recurses into impls and inline modules too — those
        // operators exist but ride their enclosing item; only top-level fns are re-dealt.
        let top_level: Vec<String> = file
            .items
            .iter()
            .filter_map(|it| match it {
                syn::Item::Fn(f) => Some(f.sig.ident.to_string()),
                _ => None,
            })
            .collect();
        let netted: BTreeSet<&str> = sigs.iter().map(|(name, _, _)| *name).collect();
        let operators: BTreeSet<&String> = top_level
            .iter()
            .filter(|n| netted.contains(n.as_str()))
            .collect();
        for op in &operators {
            if sigs
                .iter()
                .filter(|(name, _, _)| *name == op.as_str())
                .count()
                > 1
            {
                return Err(format!(
                    "bundle: operator name `{op}` is not unique across the module's \
                     functions — the dealing would be ambiguous, refused"
                ));
            }
        }

        // segmentation: an item's TEXT runs from its span start (doc comments are
        // attributes, so they open it) to its span end; the GAP after it — up to the next
        // item's start — is the position's separator trivia and stays where it stood.
        let starts: Vec<usize> = file
            .items
            .iter()
            .map(|it| byte_offset(source, it.span().start()))
            .collect();
        let preamble = match starts.first() {
            Some(&s) => source[..s].to_string(),
            None => source.to_string(),
        };
        let items: Vec<BundleItem> = file
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let text_end = byte_offset(source, item.span().end());
                let gap_end = starts.get(i + 1).copied().unwrap_or(source.len());
                let operator = match item {
                    syn::Item::Fn(f) => {
                        let n = f.sig.ident.to_string();
                        operators.contains(&n).then_some(n)
                    }
                    _ => None,
                };
                BundleItem {
                    operator,
                    text: source[starts[i]..text_end].to_string(),
                    gap: source[text_end..gap_end].to_string(),
                }
            })
            .collect();

        // the canonical dealing order, from the one placer.
        let canonical: Vec<String> = Placement::over("bundle", sigs)
            .components
            .iter()
            .flat_map(|c| c.ops.iter())
            .filter(|n| operators.contains(&n.to_string()))
            .map(|n| n.to_string())
            .collect();

        Ok(Bundle {
            preamble,
            items,
            canonical,
        })
    }

    /// The module, rendered in canonical placed order: every non-operator item keeps its
    /// position and its bytes; the operator positions are re-dealt with the operator TEXTS
    /// in placement order, each position keeping its own separator gap. A module already in
    /// canonical order renders to its own source exactly — the round-trip pin below holds
    /// this on a real committed module.
    pub fn render(&self) -> String {
        let mut out = self.preamble.clone();
        let mut dealing = self.canonical.iter();
        for item in &self.items {
            if item.operator.is_some() {
                let next = dealing
                    .next()
                    .expect("one canonical name per operator slot");
                let dealt = self
                    .items
                    .iter()
                    .find(|i| i.operator.as_deref() == Some(next.as_str()))
                    .map(|i| i.text.as_str())
                    .expect("every canonical operator names an item");
                out.push_str(dealt);
            } else {
                out.push_str(&item.text);
            }
            out.push_str(&item.gap);
        }
        out
    }

    /// Is the module already in canonical placed order? (Parse-only judgment: true exactly
    /// when `render` would reproduce the input.)
    pub fn is_canonical(&self) -> bool {
        let declared: Vec<&str> = self
            .items
            .iter()
            .filter_map(|i| i.operator.as_deref())
            .collect();
        declared
            == self
                .canonical
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use std::path::Path;

    /// RUNG 1's PIN, on a real module: `modularize.rs` — the committed file whose whole
    /// subject is proposing modularity — parses into a bundle and renders back BYTE FOR
    /// BYTE. This is the faithfulness proof the roadmap demands before any generation:
    /// the representation loses nothing (docs, trivia, the attributed inline module, the
    /// cfg(test) suite all survive verbatim), and the committed module is ALREADY in
    /// canonical placed order — checked, not assumed.
    #[test]
    fn the_committed_module_round_trips_byte_for_byte() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/discover/modularize.rs");
        let source = std::fs::read_to_string(path).expect("committed module");
        let bundle = Bundle::parse(&source).expect("parses");
        assert!(
            bundle.is_canonical(),
            "the committed module is canonically placed"
        );
        assert_eq!(bundle.render(), source, "the round-trip is byte-for-byte");
    }

    /// A SCRAMBLED module — operators of two components interleaved — renders back
    /// component-grouped: peak and gather (both on Count) reunite ahead of both (on Flag),
    /// each carrying its own doc comment, every non-operator item (the types, the constant
    /// between the functions) holding its position and bytes. The reorder is the render's
    /// only opinion, and this is the drill proving it ACTS — a renderer that echoed its
    /// input would fail here, so the byte-for-byte pin above cannot be passed vacuously.
    #[test]
    fn a_scrambled_module_renders_back_canonically() {
        let scrambled = "\
//! a scrambled bag.

pub struct Count;
pub struct Flag;

/// peak.
pub fn peak(a: Count, b: Count) -> Count {
    a
}

const SEAM: usize = 1;

/// both.
pub fn both(a: Flag, b: Flag) -> Flag {
    b
}

/// gather.
pub fn gather(a: Count) -> Count {
    a
}
";
        let canonical = "\
//! a scrambled bag.

pub struct Count;
pub struct Flag;

/// peak.
pub fn peak(a: Count, b: Count) -> Count {
    a
}

const SEAM: usize = 1;

/// gather.
pub fn gather(a: Count) -> Count {
    a
}

/// both.
pub fn both(a: Flag, b: Flag) -> Flag {
    b
}
";
        let bundle = Bundle::parse(scrambled).expect("parses");
        assert!(!bundle.is_canonical(), "the scramble is detected");
        let rendered = bundle.render();
        assert_eq!(
            rendered, canonical,
            "components regroup, everything else holds"
        );
        // idempotence: the canonical render is a fixed point of parse∘render.
        let again = Bundle::parse(&rendered).expect("the render parses");
        assert!(again.is_canonical());
        assert_eq!(again.render(), rendered);
    }

    /// The refusal paths are NAMED: a half-written module refuses to parse (no bundle is
    /// guessed from broken text), and a module whose operator name is not unique refuses
    /// the dealing rather than picking a segment.
    #[test]
    fn refusals_are_named_never_guessed() {
        let err = Bundle::parse("pub fn broken( -> {").unwrap_err();
        assert!(err.contains("unparseable"), "{err}");

        let ambiguous = "\
pub struct Count;
pub fn peak(a: Count) -> Count {
    a
}
pub mod inner {
    use super::Count;
    pub fn peak(a: Count) -> Count {
        a
    }
}
";
        let err = Bundle::parse(ambiguous).unwrap_err();
        assert!(err.contains("not unique"), "{err}");
        assert!(err.contains("peak"), "{err}");
    }

    /// A module with NO operators (or no items at all) is its own render — the dealing has
    /// nothing to say, and fidelity still holds to the byte.
    #[test]
    fn a_module_without_operators_is_its_own_render() {
        let plain = "//! just types.\n\npub struct A;\n\npub struct B;\n";
        let bundle = Bundle::parse(plain).expect("parses");
        assert!(bundle.is_canonical());
        assert_eq!(bundle.render(), plain);
        assert_eq!(Bundle::parse("").expect("empty parses").render(), "");
    }
}
