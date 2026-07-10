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
use std::path::Path;

use super::genesis::byte_offset;
use super::shape::Placement;
use super::watch::Ticker;
use super::Spec;
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

    /// THE EDIT VERB — the largest gap named at the second rule's adoption, closed: replace
    /// one named item's TEXT while its SIGNATURE holds. Change is mutation, not only
    /// addition — but a signature move is an interface change wearing an edit's clothes, so
    /// it is REFUSED here by name (retire the item and `add` its successor; a future
    /// `resign` verb may earn the combined form). What an edit may change is the body, the
    /// docs, and the attributes: the meaning and its prose — everything a caller cannot
    /// observe through the signature. Everything that names the item re-judges downstream
    /// exactly because the signature held: the laws re-run at the next judgment, the lift
    /// drift-gate demands regeneration, the censuses re-derive.
    ///
    /// Refusals, named: an unparseable replacement, a replacement that is not exactly ONE
    /// item, a name the module does not carry, a replacement whose name differs (an edit
    /// is not a rename), and a FUNCTION whose signature moved (compared token-for-token).
    /// Non-function items (types, consts) hold their name and kind instead — reshaping a
    /// type is interface change, refused the same way.
    pub fn edit(module: &str, item_name_text: &str, replacement: &str) -> Result<String, String> {
        let new = syn::parse_file(replacement)
            .map_err(|e| format!("bundle edit: replacement unparseable: {e}"))?;
        let [new_item] = new.items.as_slice() else {
            return Err(format!(
                "bundle edit: the replacement must be exactly one item, got {} — one edit, \
                 one judged transaction",
                new.items.len()
            ));
        };
        let Some(new_name) = item_name(new_item) else {
            return Err(
                "bundle edit: the replacement has no defining name — nothing to hold an \
                 edit to"
                    .to_string(),
            );
        };
        if new_name != item_name_text {
            return Err(format!(
                "bundle edit: the replacement names `{new_name}`, not `{item_name_text}` — \
                 an edit is not a rename"
            ));
        }

        let file =
            syn::parse_file(module).map_err(|e| format!("bundle edit: module unparseable: {e}"))?;
        let target = file
            .items
            .iter()
            .find(|item| {
                let name = item_name(item);
                name.as_deref() == Some(item_name_text)
            })
            .ok_or_else(|| {
                format!(
                    "bundle edit: no item named `{item_name_text}` in the module — `add` \
                     grows, `edit` changes; nothing here to change"
                )
            })?;

        // the signature hold: for functions, token-for-token equality of the signature; for
        // everything else, the kind must match (a struct stays a struct) — reshaping is
        // interface change either way, and interface change is not an edit.
        match (target, new_item) {
            (syn::Item::Fn(old), syn::Item::Fn(new)) => {
                use quote::ToTokens;
                let held = old.sig.to_token_stream().to_string();
                let offered = new.sig.to_token_stream().to_string();
                if held != offered {
                    return Err(format!(
                        "bundle edit: `{item_name_text}`'s signature moved (`{held}` -> \
                         `{offered}`) — an interface change is not an edit; retire the item \
                         and add its successor"
                    ));
                }
            }
            (old, new) if std::mem::discriminant(old) == std::mem::discriminant(new) => {}
            _ => {
                return Err(format!(
                    "bundle edit: `{item_name_text}` changed item kind — an interface \
                     change is not an edit"
                ));
            }
        }

        // the splice: the target's TEXT is replaced in place (its position and its gap are
        // the module's furniture and stay), then one parse∘render keeps the canonical
        // guarantee.
        let start = byte_offset(module, target.span().start());
        let end = byte_offset(module, target.span().end());
        let mut out = String::with_capacity(module.len());
        out.push_str(&module[..start]);
        out.push_str(replacement.trim_end());
        out.push_str(&module[end..]);
        Ok(Bundle::parse(&out)?.render())
    }

    /// THE CONTINUATION VERB, library form (rung 2): add a snippet to a module, purely
    /// additively, and return the module re-rendered in canonical placed order — the new
    /// operator lands WITH ITS COMPONENT (the placer's dealing, not an append), a new type
    /// or helper lands before the trailing `#[cfg(test)]` module (tests stay last), and
    /// every existing item's bytes survive verbatim (addition is monotone by union-find's
    /// own algebra — the snippet can join or bridge components, never reshuffle what it
    /// does not touch). The caller writes the result to the file; the verb has no I/O.
    ///
    /// Refusals are named, never guessed: an unparseable snippet, an empty snippet, and a
    /// NAME COLLISION with an existing top-level item — the anti-duplication guarantee at
    /// the mechanical level, the type-library voice's whisper made a hard stop.
    pub fn add(module: &str, snippet: &str) -> Result<String, String> {
        let new = syn::parse_file(snippet)
            .map_err(|e| format!("bundle add: snippet unparseable: {e}"))?;
        if new.items.is_empty() {
            return Err("bundle add: the snippet declares nothing".to_string());
        }
        let existing =
            syn::parse_file(module).map_err(|e| format!("bundle add: module unparseable: {e}"))?;
        let taken: BTreeSet<String> = existing.items.iter().filter_map(item_name).collect();
        for name in new.items.iter().filter_map(item_name) {
            if taken.contains(&name) {
                return Err(format!(
                    "bundle add: `{name}` already exists in the module — addition is \
                     additive; edit the existing item or pick a name"
                ));
            }
        }

        // splice the snippet before the trailing `#[cfg(test)]` module when there is one
        // (tests stay last), else at the end — then one parse∘render places it.
        let block = format!("{}\n", snippet.trim_end());
        let test_start = existing.items.iter().find_map(|item| match item {
            syn::Item::Mod(m) if is_cfg_test(&m.attrs) => {
                Some(byte_offset(module, item.span().start()))
            }
            _ => None,
        });
        let mut spliced = String::new();
        match test_start {
            Some(at) => {
                spliced.push_str(&module[..at]);
                pad_to_blank_line(&mut spliced);
                spliced.push_str(&block);
                spliced.push('\n');
                spliced.push_str(&module[at..]);
            }
            None => {
                spliced.push_str(module);
                pad_to_blank_line(&mut spliced);
                spliced.push_str(&block);
            }
        }
        Ok(Bundle::parse(&spliced)?.render())
    }

    /// THE DECLARATION ENTRY, first form (the disposition's end state, rung 1): declare an
    /// expectation into a module's `#[algebra(...)]` attribute, additively — the SHOULD half
    /// joining the bundle the operators live in. `declaration` is the `expects` grammar
    /// exactly as the macro takes it (`commutative(peak)`, `identity(grant, zero)`); the
    /// result is the module re-rendered canonically, differing only in that attribute.
    ///
    /// Parser-as-gate, all refusals named: a shape word outside the ratified catalog is
    /// refused TEACHING the vocabulary (`Expectation::canonical` — the same validator the
    /// engine panics with, in its non-panicking form); a duplicate of an already-declared
    /// expectation is refused; a module with no `#[algebra]` item is refused naming the fix
    /// (the zero-annotation declaration channel — expectations on a `Lifted` module — is a
    /// disclosed further rung); two `#[algebra]` items are refused as ambiguous rather than
    /// guessed between.
    pub fn declare(module: &str, declaration: &str) -> Result<String, String> {
        let (key, _args) = parse_declaration(declaration)?;
        let canonical_key = super::expect::Expectation::canonical(&key).ok_or_else(|| {
            format!(
                "bundle declare: `{key}` is not in the ratified catalog \
                 (spec/shapes.spec). Declarable shapes: {}",
                super::expect::Expectation::vocabulary_keys().join(", ")
            )
        })?;

        let file = syn::parse_file(module)
            .map_err(|e| format!("bundle declare: module unparseable: {e}"))?;
        let mut algebra_attrs: Vec<&syn::Attribute> = Vec::new();
        for item in &file.items {
            let attrs: &[syn::Attribute] = match item {
                syn::Item::Mod(m) => &m.attrs,
                _ => continue,
            };
            algebra_attrs.extend(attrs.iter().filter(|a| {
                a.path()
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "algebra")
            }));
        }
        let attr = match algebra_attrs.as_slice() {
            [] => {
                return Err(
                    "bundle declare: no `#[algebra]` module to declare into — attach \
                     `#[algebra(Marker, \"name\")]` to the module (the zero-annotation \
                     declaration channel is a further rung)"
                        .to_string(),
                )
            }
            [one] => *one,
            many => {
                return Err(format!(
                    "bundle declare: {} `#[algebra]` modules in one file — ambiguous, \
                     refused rather than guessed between",
                    many.len()
                ))
            }
        };

        let start = byte_offset(module, attr.span().start());
        let end = byte_offset(module, attr.span().end());
        let attr_text = &module[start..end];
        let entry = declaration.trim();

        // a duplicate of an already-declared expectation is refused, compared through the
        // canonical shape name so `idempotent(x)` and its catalog name cannot both land.
        for existing in expects_entries(attr_text) {
            let Ok((existing_key, existing_args)) = parse_declaration(&existing) else {
                continue;
            };
            let (_, new_args) = parse_declaration(entry)?;
            if super::expect::Expectation::canonical(&existing_key) == Some(canonical_key)
                && existing_args == new_args
            {
                return Err(format!(
                    "bundle declare: `{entry}` is already declared — a declaration is \
                     additive, never repeated"
                ));
            }
        }

        // the edit: append inside an existing `expects(...)` (the grammar keeps it last,
        // so the attribute ends `))]`), or open one (the attribute ends `)]`). Anything
        // else is refused, never guessed at.
        let new_attr = if let Some(head) = attr_text.strip_suffix("))]") {
            if attr_text.contains("expects(") {
                format!("{head}, {entry}))]")
            } else {
                return Err(format!(
                    "bundle declare: unrecognized `#[algebra]` shape `{attr_text}` — \
                     declare by hand"
                ));
            }
        } else if let Some(head) = attr_text.strip_suffix(")]") {
            format!("{head}, expects({entry}))]")
        } else {
            return Err(format!(
                "bundle declare: unrecognized `#[algebra]` shape `{attr_text}` — declare \
                 by hand"
            ));
        };

        let mut out = String::with_capacity(module.len() + new_attr.len());
        out.push_str(&module[..start]);
        out.push_str(&new_attr);
        out.push_str(&module[end..]);
        Ok(Bundle::parse(&out)?.render())
    }

    /// THE PERCEPTION VERB — everything that PINS a named operator, read from the
    /// committed record and rendered as one deterministic report: the blast radius an
    /// agent wants BEFORE touching anything, derived instead of grepped. Four senses, one
    /// page: the operator's placement COMPONENT (who it is wired to), the committed LAWS
    /// naming it (the contract — an edit that breaks one refuses at the next judgment),
    /// its ratified FREEDOMS (the mutation-lock survivors at it — where a change is
    /// invisible to the spec, the guarantee's own fine print), and the downstream
    /// RELIANCES naming it (who breaks, and the why they declared). Read-only: perception
    /// writes nothing and journals nothing.
    ///
    /// Empty sections render honestly ("none — …"): silence is a finding, not a blank.
    /// An operator the module does not declare is a refusal — perception does not guess.
    /// Disclosed: declared expectations are read from `#[algebra]` attributes in the
    /// module text; a lifted module's declarations live in its lift artifact and reach
    /// this report through their frozen laws instead.
    ///
    /// Capability: Effectful — reads the committed locks in `spec_dir` and the optional
    /// reliance register.
    pub fn constrains(
        module: &str,
        name: &str,
        spec_dir: &Path,
        reliances: Option<&Path>,
    ) -> Result<String, String> {
        let sigs = Ticker::parse_rust_sigs(module)?;
        let declared_here = sigs
            .iter()
            .any(|(op, _, _)| *op == name || op.ends_with(&format!("::{name}")));
        if !declared_here {
            return Err(format!(
                "bundle constrains: the module declares no operator `{name}` — \
                 perception does not guess"
            ));
        }

        let mut out = format!("constrains `{name}`:\n");

        // the COMPONENT: who the operator is wired to, from the one placer.
        let placement = Placement::over("bundle", sigs);
        for component in &placement.components {
            if component
                .ops
                .iter()
                .any(|op| *op == name || op.ends_with(&format!("::{name}")))
            {
                out.push_str(&format!(
                    "  component: {{ {} }} over nets {{ {} }}\n",
                    component.ops.join(", "),
                    component.nets.join(", ")
                ));
            }
        }

        // the LAWS: every committed behaviour lock in spec_dir whose equation or prose
        // names the operator.
        let mut laws: Vec<String> = Vec::new();
        let mut freedoms: Vec<String> = Vec::new();
        if let Ok(dir) = std::fs::read_dir(spec_dir) {
            let mut files: Vec<_> = dir
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            files.sort();
            for file in files {
                let Ok(text) = std::fs::read_to_string(spec_dir.join(&file)) else {
                    continue;
                };
                if file.ends_with(".mutation.spec") {
                    let stem = file.trim_end_matches(".mutation.spec");
                    for line in text.lines() {
                        if line.contains("SURVIVED") && line.contains(&format!("`{name}`")) {
                            freedoms.push(format!(
                                "    - {stem}: {}\n",
                                line.trim_start_matches(['-', ' '])
                            ));
                        }
                    }
                } else if text.starts_with("# discovered spec:") {
                    let stem = file.trim_end_matches(".spec");
                    for (prose, equation) in Spec::parse_lock(&text) {
                        if mentions(&equation, name) {
                            laws.push(format!("    - {stem}: {equation}  ({prose})\n"));
                        }
                    }
                }
            }
        }
        out.push_str("  laws naming it (the contract):\n");
        if laws.is_empty() {
            out.push_str("    none — the committed spec is silent about this operator\n");
        } else {
            laws.iter().for_each(|l| out.push_str(l));
        }

        // the DECLARED expectations: the SHOULD half, read from `#[algebra]` attributes.
        let declared: Vec<String> = expects_entries(module)
            .into_iter()
            .filter(|entry| mentions(entry, name))
            .collect();
        out.push_str("  declared expectations naming it:\n");
        if declared.is_empty() {
            out.push_str("    none — conduct only; no declared contract names it here\n");
        } else {
            for entry in declared {
                out.push_str(&format!("    - {entry}\n"));
            }
        }

        // the FREEDOMS: the ratified survivors at this operator — the fine print.
        out.push_str("  freedoms at it (ratified degrees of freedom):\n");
        if freedoms.is_empty() {
            out.push_str("    none — every judged mutant of this operator dies\n");
        } else {
            freedoms.iter().for_each(|f| out.push_str(f));
        }

        // the RELIANCES: who declared they stand on a law naming it.
        out.push_str("  downstream reliances naming it:\n");
        let mut relied = Vec::new();
        if let Some(path) = reliances {
            let register = spec_lock::Register {
                name: "downstream reliances".to_string(),
                path: path.to_path_buf(),
            };
            for (key, justification) in register.entries()? {
                if mentions(&key, name) {
                    relied.push(format!("    - {key} ({justification})\n"));
                }
            }
        }
        if relied.is_empty() {
            out.push_str("    none declared — semver's old blind spot, now an honest empty\n");
        } else {
            relied.iter().for_each(|r| out.push_str(r));
        }
        Ok(out)
    }

    /// One journal line — stage 2 of the zero-file-patching aim: THE VERBS RECORD
    /// THEMSELVES. The format is deliberately minimal and deterministic
    /// (`<verb> <module> — <detail>`, no timestamps: order is the journal's only clock),
    /// so the file is an append-only record a PR body can be derived from —
    /// `bundle-demo/MANIFEST.md`'s hand-written story, machined. Disclosed limit, recorded
    /// where the roadmap names stage 3: entries carry NAMES, not payloads, so the journal
    /// is the agenda's source and the reviewer's record but not yet REPLAYABLE —
    /// tree == replay(journal) needs the payload store.
    pub fn journal_entry(verb: &str, module: &str, detail: &str) -> String {
        format!("{verb} {module} — {detail}\n")
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

/// A top-level item's name, for the collision refusal — functions, types, modules,
/// constants; items without a single defining ident (impls, uses) contribute none (an
/// impl extends an existing name, which is precisely not a collision).
#[crate::mutate]
fn item_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Fn(f) => Some(f.sig.ident.to_string()),
        syn::Item::Struct(s) => Some(s.ident.to_string()),
        syn::Item::Enum(e) => Some(e.ident.to_string()),
        syn::Item::Union(u) => Some(u.ident.to_string()),
        syn::Item::Type(t) => Some(t.ident.to_string()),
        syn::Item::Mod(m) => Some(m.ident.to_string()),
        syn::Item::Const(c) => Some(c.ident.to_string()),
        syn::Item::Static(s) => Some(s.ident.to_string()),
        _ => None,
    }
}

/// A declaration's shallow parse: `key(arg, arg)` → the shape key and its trimmed args.
/// The DEEP validation stays where it lives — the catalog for the vocabulary
/// ([`super::expect::Expectation::canonical`]), the macro for the full grammar at compile
/// time; this parse only needs enough structure to validate and compare.
#[crate::mutate]
pub(crate) fn parse_declaration(text: &str) -> Result<(String, Vec<String>), String> {
    let text = text.trim();
    let (key, rest) = text.split_once('(').ok_or_else(|| {
        format!("bundle declare: `{text}` is not a declaration — the grammar is `shape(op, ...)`")
    })?;
    let args = rest.strip_suffix(')').ok_or_else(|| {
        format!("bundle declare: `{text}` is missing its closing paren — nothing to judge")
    })?;
    let args: Vec<String> = args
        .split(',')
        .map(|a| a.trim().to_string())
        .filter(|a| !a.is_empty())
        .collect();
    if args.is_empty() {
        return Err(format!(
            "bundle declare: `{text}` names no operators — a declaration binds a shape to ops"
        ));
    }
    Ok((key.trim().to_string(), args))
}

/// The entries of an attribute's `expects(...)` group, split at top-level commas (an
/// entry's own parens respected). Empty when the attribute carries no `expects`.
#[crate::mutate]
fn expects_entries(attr_text: &str) -> Vec<String> {
    let Some(after) = attr_text.split_once("expects(").map(|(_, rest)| rest) else {
        return Vec::new();
    };
    let mut entries = Vec::new();
    let mut depth = 0usize;
    let mut current = String::new();
    for c in after.chars() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' if depth == 0 => break, // expects' own close
            ')' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => {
                entries.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        entries.push(current.trim().to_string());
    }
    entries
}

/// Does `text` mention `name` as a whole word (ident-boundary on both sides)? The report
/// matcher — `merge` must not match `submerged`, and an operator named inside an equation,
/// a declaration, or a reliance key counts wherever it stands.
#[crate::mutate]
pub(crate) fn mentions(text: &str, name: &str) -> bool {
    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
    let bytes = text.as_bytes();
    let mut from = 0;
    while let Some(at) = text[from..].find(name) {
        let start = from + at;
        let end = start + name.len();
        let before_ok = start == 0 || !is_ident(text[..start].chars().next_back().unwrap());
        let after_ok = end == bytes.len() || !is_ident(text[end..].chars().next().unwrap());
        if before_ok && after_ok {
            return true;
        }
        from = end;
    }
    false
}

/// Does an attribute list carry `#[cfg(test)]`?
#[crate::mutate]
fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.parse_args::<syn::Path>()
                .is_ok_and(|p| p.is_ident("test"))
    })
}

/// Ensure `out` ends with a blank line (exactly one empty line before what comes next) —
/// the separator convention the splice writes between existing text and the added block.
#[crate::mutate]
fn pad_to_blank_line(out: &mut String) {
    while out.ends_with("\n\n\n") {
        out.pop();
    }
    if out.is_empty() {
        return;
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.ends_with("\n\n") {
        out.push('\n');
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

    /// THE CONTINUATION VERB, drilled: an added operator lands WITH ITS COMPONENT — not at
    /// the end — every existing item's bytes survive verbatim, a following addition that
    /// touches a second sort BRIDGES (monotone: nothing else moves), a non-operator
    /// addition lands before the trailing test module, and the result is a fixed point of
    /// parse∘render.
    #[test]
    fn add_places_the_snippet_with_its_component() {
        let module = "\
//! a working module.

pub struct Count;
pub struct Flag;

/// peak.
pub fn peak(a: Count, b: Count) -> Count {
    a
}

/// both.
pub fn both(a: Flag, b: Flag) -> Flag {
    b
}

#[cfg(test)]
mod tests {
    #[test]
    fn holds() {}
}
";
        // a Count operator: it must land in the Count component (with peak), not at the end.
        let grown = Bundle::add(
            module,
            "/// gather.\npub fn gather(a: Count) -> Count {\n    a\n}\n",
        )
        .expect("adds");
        let peak_at = grown.find("pub fn peak").expect("peak survives");
        let gather_at = grown.find("pub fn gather").expect("gather landed");
        let both_at = grown.find("pub fn both").expect("both survives");
        assert!(
            peak_at < gather_at && gather_at < both_at,
            "gather joins the Count component ahead of the Flag one:\n{grown}"
        );
        // additive: every original item's text survives byte-identical, tests still last.
        for piece in [
            "//! a working module.",
            "/// peak.\npub fn peak(a: Count, b: Count) -> Count {\n    a\n}",
            "/// both.\npub fn both(a: Flag, b: Flag) -> Flag {\n    b\n}",
            "#[cfg(test)]\nmod tests {",
        ] {
            assert!(grown.contains(piece), "`{piece}` survives:\n{grown}");
        }
        assert!(
            grown.find("mod tests").unwrap() > gather_at,
            "tests stay last"
        );
        // the result is canonical — a fixed point of parse∘render.
        let reparsed = Bundle::parse(&grown).expect("the grown module parses");
        assert!(reparsed.is_canonical());
        assert_eq!(reparsed.render(), grown);

        // a helper type (no operator) lands before the tests, after the working items.
        let with_type = Bundle::add(&grown, "pub struct Spin;\n").expect("adds a type");
        assert!(
            with_type.find("pub struct Spin").unwrap() < with_type.find("mod tests").unwrap(),
            "{with_type}"
        );
    }

    /// The verb's refusals: a snippet that does not parse, a snippet that declares
    /// nothing, and a NAME COLLISION with an existing item — the type-library voice's
    /// whisper made a hard stop, named.
    #[test]
    fn add_refuses_collisions_and_noise() {
        let module = "pub struct Count;\npub fn peak(a: Count) -> Count {\n    a\n}\n";
        let err = Bundle::add(module, "pub fn broken( -> {").unwrap_err();
        assert!(err.contains("unparseable"), "{err}");
        let err = Bundle::add(module, "// only a comment\n").unwrap_err();
        assert!(err.contains("declares nothing"), "{err}");
        let err = Bundle::add(module, "pub fn peak(a: Count) -> Count {\n    a\n}\n").unwrap_err();
        assert!(err.contains("`peak` already exists"), "{err}");
        let err = Bundle::add(module, "pub struct Count;\n").unwrap_err();
        assert!(err.contains("`Count` already exists"), "{err}");
        // an impl extends an existing name — precisely NOT a collision.
        let grown = Bundle::add(
            module,
            "impl Count {\n    pub fn zero(self) -> Count {\n        self\n    }\n}\n",
        )
        .expect("an impl extends, never collides");
        assert!(grown.contains("pub fn zero"));
    }

    /// THE DECLARATION ENTRY, drilled on the REAL module: declaring an expectation into the
    /// committed `modularize.rs` (whose `soup` carries `#[crate::algebra]` with no
    /// `expects`) moves ONLY the attribute — every other byte survives — and the result
    /// parses, stays canonical, and carries the declaration. The SHOULD half joins the
    /// bundle without disturbing the IS half.
    #[test]
    fn declare_moves_only_the_attribute_on_a_real_module() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/discover/modularize.rs");
        let source = std::fs::read_to_string(path).expect("committed module");
        let declared = Bundle::declare(&source, "commutative(peak)").expect("declares");
        assert!(
            declared.contains("expects(commutative(peak))"),
            "the declaration landed"
        );
        // only the attribute differs: removing the one changed line from each side
        // leaves identical text.
        let differing: Vec<(&str, &str)> = source
            .lines()
            .zip(declared.lines())
            .filter(|(a, b)| a != b)
            .collect();
        assert_eq!(differing.len(), 1, "exactly one line moved");
        assert!(differing[0]
            .0
            .contains("#[crate::algebra(Soup, \"flat soup\")]"));
        assert!(differing[0]
            .1
            .contains("#[crate::algebra(Soup, \"flat soup\", expects(commutative(peak)))]"));
        // a second declaration APPENDS inside the existing expects; repeating one refuses.
        let twice = Bundle::declare(&declared, "associative(peak)").expect("appends");
        assert!(
            twice.contains("expects(commutative(peak), associative(peak))"),
            "{twice}"
        );
        let err = Bundle::declare(&twice, "commutative(peak)").unwrap_err();
        assert!(err.contains("already declared"), "{err}");
    }

    /// The declaration gate teaches, never guesses: an unratified shape word is refused
    /// LISTING the vocabulary; a module with no `#[algebra]` names the fix; malformed
    /// declarations name their grammar.
    #[test]
    fn declare_refuses_by_teaching() {
        let module = "#[crate::algebra(M, \"m\")]\npub mod m {\n    pub struct A;\n    pub fn f(a: A) -> A {\n        a\n    }\n}\n";
        let err = Bundle::declare(module, "sparkly(f)").unwrap_err();
        assert!(err.contains("not in the ratified catalog"), "{err}");
        assert!(err.contains("Declarable shapes:"), "{err}");
        assert!(err.contains("commutative"), "the refusal teaches: {err}");

        let err = Bundle::declare("pub struct A;\n", "commutative(f)").unwrap_err();
        assert!(err.contains("no `#[algebra]` module"), "{err}");

        let err = Bundle::declare(module, "commutative").unwrap_err();
        assert!(err.contains("the grammar is"), "{err}");
        let err = Bundle::declare(module, "commutative()").unwrap_err();
        assert!(err.contains("names no operators"), "{err}");
    }

    /// THE EDIT VERB, drilled: a body edit lands with the signature held — position, gap,
    /// and every other item's bytes untouched — while a signature move, a rename, a kind
    /// change, a multi-item replacement, and an unknown name each refuse BY NAME. Change
    /// is mutation; interface change is not an edit.
    #[test]
    fn edit_replaces_the_body_and_holds_the_signature() {
        let module = "\
pub struct Count;

/// peak.
pub fn peak(a: Count, b: Count) -> Count {
    a
}

/// gather.
pub fn gather(a: Count) -> Count {
    a
}
";
        // the meaning changes; the docs may change; the signature holds.
        let edited = Bundle::edit(
            module,
            "peak",
            "/// peak — now honestly the max.\npub fn peak(a: Count, b: Count) -> Count {\n    b\n}\n",
        )
        .expect("a body edit lands");
        assert!(edited.contains("now honestly the max"));
        assert!(edited.contains("    b\n}"), "{edited}");
        assert!(
            edited.contains("/// gather.\npub fn gather(a: Count) -> Count {\n    a\n}"),
            "the neighbour survives byte-identical: {edited}"
        );
        // the result stays canonical.
        assert!(Bundle::parse(&edited).expect("parses").is_canonical());

        // the refusals, each named:
        let err = Bundle::edit(
            module,
            "peak",
            "pub fn peak(a: Count) -> Count {\n    a\n}\n",
        )
        .unwrap_err();
        assert!(err.contains("signature moved"), "{err}");
        let err = Bundle::edit(
            module,
            "peak",
            "pub fn summit(a: Count, b: Count) -> Count {\n    a\n}\n",
        )
        .unwrap_err();
        assert!(err.contains("not a rename"), "{err}");
        let err = Bundle::edit(
            module,
            "Count",
            "pub fn Count(a: Count) -> Count {\n    a\n}\n",
        )
        .unwrap_err();
        assert!(err.contains("changed item kind"), "{err}");
        let err = Bundle::edit(
            module,
            "ghost",
            "pub fn ghost(a: Count) -> Count {\n    a\n}\n",
        )
        .unwrap_err();
        assert!(err.contains("no item named `ghost`"), "{err}");
        let err = Bundle::edit(module, "peak", "pub struct A;\npub struct B;\n").unwrap_err();
        assert!(err.contains("exactly one item"), "{err}");
    }

    /// THE PERCEPTION VERB, drilled on the REAL bundle-born member: `constrains merge`
    /// reads the committed record and reports the component, the laws (the declared four
    /// plus the discovered homomorphism), the honest empties, and the freedoms — the blast
    /// radius derived, not grepped. An operator the module does not declare refuses.
    #[test]
    fn constrains_reads_the_committed_record() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("bundle-demo");
        let module = std::fs::read_to_string(root.join("src/tally.rs")).expect("demo module");
        let report =
            Bundle::constrains(&module, "merge", &root.join("spec"), None).expect("reports");
        assert!(report.contains("component: { merge, floor, bump } over nets { Tally }"));
        assert!(
            report.contains("(x merge y) = (y merge x)"),
            "the contract laws appear: {report}"
        );
        assert!(
            report.contains("bump((x merge y)) = (bump(x) merge bump(y))"),
            "the discovered surprise appears too: {report}"
        );
        assert!(
            report.contains("none declared — semver's old blind spot"),
            "empty reliances render honestly: {report}"
        );

        // an operator that does not exist refuses — perception does not guess.
        let err = Bundle::constrains(&module, "ghost", &root.join("spec"), None).unwrap_err();
        assert!(err.contains("no operator `ghost`"), "{err}");
    }

    /// The word-boundary matcher: `merge` does not match `submerged` or `merges`, and
    /// matches at line edges and inside equations.
    #[test]
    fn mentions_is_ident_bounded() {
        assert!(mentions("(x merge y) = (y merge x)", "merge"));
        assert!(mentions("merge", "merge"));
        assert!(!mentions("submerged", "merge"));
        assert!(!mentions("merges", "merge"));
        assert!(mentions("`merge` evaluates as `floor`", "merge"));
    }

    /// The journal line is deterministic and minimal — order is its only clock — so the
    /// record the verbs append is diffable, derivable-from, and never smuggles state.
    #[test]
    fn the_journal_line_is_deterministic() {
        assert_eq!(
            Bundle::journal_entry("add", "src/tally.rs", "fn merge"),
            "add src/tally.rs — fn merge\n"
        );
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
