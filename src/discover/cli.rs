/// The argument-type vocabulary: seven sorts cover all seventeen of bundle's verbs. A
/// sort is what the harness knows about a slot before any judgment runs — how the
/// token renders in usage (its shell-quoting discipline included) and, when dispatch
/// migrates here, how its value is fetched. The sort is the type; the label on each
/// slot is the face (`add`'s payload is a snippet, `edit`'s a replacement — same sort).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    /// `<module.rs>` — a module path, the subject of most verbs. It may not exist
    /// yet: `add` founds files, so fetching tolerates absence.
    Module,
    /// `<item-name>` — an address inside a module: an item or operator name.
    Item,
    /// `<snippet.rs | ->` — content by file path, or `-` for stdin.
    Payload,
    /// `"<shape(op, ...)>"` — a law-language stanza; shell-quoted when it binds one
    /// value (a stanza carries spaces), bare when variadic (one stanza per word).
    Declaration,
    /// `<bundle.journal>` — the record itself as the verb's subject.
    Journal,
    /// `<theory>` — a compiled theory's name from the binary's roster.
    Theory,
    /// `'<term>'` — a ground term over a theory's operators; single-quoted.
    Term,
}

/// How many values a slot binds: exactly one, at most one, or the rest of the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Exactly one value; its absence is a refusal.
    Required,
    /// Zero or one — the verb has two judgment branches (`show` the table of
    /// contents, `show` one item).
    Optional,
    /// Every remaining value, possibly none.
    Variadic,
}

/// One argument slot: a sort (the type), a label (the usage face), a mode (the
/// arity). The declaration is a list of these per verb; the harness derives usage
/// text and argv binding from nothing else.
#[derive(Debug, Clone)]
pub struct Slot {
    /// The slot's sort — see [`Sort`].
    pub sort: Sort,
    /// The placeholder printed in usage, without brackets or quoting.
    pub label: &'static str,
    /// The slot's arity — see [`Mode`].
    pub mode: Mode,
}

#[crate::mutate]
impl Slot {
    /// A slot binding exactly one value.
    pub fn required(sort: Sort, label: &'static str) -> Slot {
        Slot {
            sort,
            label,
            mode: Mode::Required,
        }
    }

    /// A slot binding zero or one value.
    pub fn optional(sort: Sort, label: &'static str) -> Slot {
        Slot {
            sort,
            label,
            mode: Mode::Optional,
        }
    }

    /// A slot binding every remaining value.
    pub fn variadic(sort: Sort, label: &'static str) -> Slot {
        Slot {
            sort,
            label,
            mode: Mode::Variadic,
        }
    }

    /// The slot's usage token. Mode owns the brackets (`<x>` required, `[x]`
    /// optional, `[x ...]` variadic); the sort owns its decorations — a payload
    /// admits stdin (`<snippet.rs | ->`), a single declaration or term carries its
    /// shell quoting (`"<shape(op, ...)>"`, `'<term>'`) because its value holds
    /// spaces the shell would otherwise split.
    fn render(&self) -> String {
        let core = match self.mode {
            Mode::Required => {
                if self.sort == Sort::Payload {
                    format!("<{} | ->", self.label)
                } else {
                    format!("<{}>", self.label)
                }
            }
            Mode::Optional => format!("[{}]", self.label),
            Mode::Variadic => format!("[{} ...]", self.label),
        };
        match (self.sort, self.mode) {
            (Sort::Declaration, Mode::Required) => format!("\"{core}\""),
            (Sort::Term, Mode::Required) => format!("'{core}'"),
            _ => core,
        }
    }
}

/// One verb: a name and its slots. `sibling` is a declared pairing — the verb
/// renders on the previous verb's usage row (`bundle gates | bundle owes`), a
/// judgment about presentation (owes is gates' perception half), not a derived fact.
#[derive(Debug, Clone)]
pub struct VerbSpec {
    /// The verb's name, argv's first word.
    pub name: &'static str,
    /// The argument slots, in binding order.
    pub slots: Vec<Slot>,
    /// Whether this verb shares the previous verb's usage row.
    pub sibling: bool,
}

#[crate::mutate]
impl VerbSpec {
    /// A verb on its own usage row.
    pub fn verb(name: &'static str, slots: Vec<Slot>) -> VerbSpec {
        VerbSpec {
            name,
            slots,
            sibling: false,
        }
    }

    /// A verb sharing the previous verb's usage row.
    pub fn sibling(name: &'static str, slots: Vec<Slot>) -> VerbSpec {
        VerbSpec {
            name,
            slots,
            sibling: true,
        }
    }
}

/// A declared command-line language: a binary name and its verbs. The harness —
/// usage text, argv matching — is derived from this and only this; a CLI in the
/// class declares its verb list and inherits the rest.
#[derive(Debug, Clone)]
pub struct CliSpec {
    /// The binary's name, the prefix of every usage row.
    pub name: &'static str,
    /// The verbs, in usage order.
    pub verbs: Vec<VerbSpec>,
}

#[crate::mutate]
impl CliSpec {
    /// bundle's own seventeen verbs, declared — the hand-built witness of the class
    /// becomes its first instance. Seven sorts cover every slot; sixteen usage rows
    /// (gates and owes share one, by declaration) render byte-identical to the text
    /// the hand-written harness always printed, which is the first lock.
    pub fn bundle() -> CliSpec {
        use Sort::*;
        CliSpec {
            name: "bundle",
            verbs: vec![
                VerbSpec::verb(
                    "add",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::required(Payload, "snippet.rs"),
                    ],
                ),
                VerbSpec::verb(
                    "edit",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::required(Item, "item-name"),
                        Slot::required(Payload, "replacement.rs"),
                    ],
                ),
                VerbSpec::verb(
                    "declare",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::required(Declaration, "shape(op, ...)"),
                    ],
                ),
                VerbSpec::verb("place", vec![Slot::required(Module, "module.rs")]),
                VerbSpec::verb("check", vec![Slot::required(Module, "module.rs")]),
                VerbSpec::verb(
                    "show",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::optional(Item, "item-name"),
                    ],
                ),
                VerbSpec::verb(
                    "collect",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::optional(Item, "item-to-sweep"),
                    ],
                ),
                VerbSpec::verb("squash", vec![Slot::required(Journal, "bundle.journal")]),
                VerbSpec::verb("replay", vec![Slot::required(Journal, "bundle.journal")]),
                VerbSpec::verb(
                    "constrains",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::required(Item, "operator"),
                    ],
                ),
                VerbSpec::verb("uses", vec![Slot::required(Item, "item-name")]),
                VerbSpec::verb("spoke", vec![Slot::required(Item, "item-name")]),
                VerbSpec::verb(
                    "trace",
                    vec![
                        Slot::required(Theory, "theory"),
                        Slot::required(Term, "term"),
                    ],
                ),
                VerbSpec::verb(
                    "lift",
                    vec![
                        Slot::required(Module, "module.rs"),
                        Slot::required(Theory, "theory-name"),
                        Slot::variadic(Declaration, "declaration"),
                    ],
                ),
                VerbSpec::verb("gates", vec![]),
                VerbSpec::sibling("owes", vec![]),
                VerbSpec::verb("pin", vec![]),
            ],
        }
    }

    /// The usage text, derived: one row per verb (siblings joined with ` | `),
    /// first row prefixed `usage: `, the rest aligned under it, no trailing
    /// newline. This is the text the first lock pins byte-for-byte against what
    /// `bundle` has always printed.
    pub fn usage(&self) -> String {
        let mut rows: Vec<String> = Vec::new();
        for verb in &self.verbs {
            let mut cell = format!("{} {}", self.name, verb.name);
            for slot in &verb.slots {
                cell.push(' ');
                cell.push_str(&slot.render());
            }
            match rows.last_mut() {
                Some(last) if verb.sibling => {
                    last.push_str(" | ");
                    last.push_str(&cell);
                }
                _ => rows.push(cell),
            }
        }
        let head = "usage: ";
        rows.iter()
            .enumerate()
            .map(|(i, row)| {
                if i == 0 {
                    format!("{head}{row}")
                } else {
                    format!("{}{row}", " ".repeat(head.len()))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Match argv against the declaration: find the verb by its first word, bind
    /// the rest to its slots left to right (required takes one or refuses, optional
    /// takes one if present, variadic takes the remainder), and refuse a stray. The
    /// refusal is the usage text — the same teaching the hand-written harness gives —
    /// and a refusal binds nothing.
    pub fn parse<'a>(&'a self, args: &[String]) -> Result<Invocation<'a>, String> {
        let Some((name, rest)) = args.split_first() else {
            return Err(self.usage());
        };
        let Some(verb) = self.verbs.iter().find(|v| v.name == name) else {
            return Err(self.usage());
        };
        let mut values = Vec::new();
        let mut at = 0;
        for slot in &verb.slots {
            match slot.mode {
                Mode::Required => {
                    if at < rest.len() {
                        values.push(vec![rest[at].clone()]);
                        at += 1;
                    } else {
                        return Err(self.usage());
                    }
                }
                Mode::Optional => {
                    if at < rest.len() {
                        values.push(vec![rest[at].clone()]);
                        at += 1;
                    } else {
                        values.push(Vec::new());
                    }
                }
                Mode::Variadic => {
                    values.push(rest[at..].to_vec());
                    at = rest.len();
                }
            }
        }
        if at != rest.len() {
            return Err(self.usage());
        }
        Ok(Invocation { verb, values })
    }
}

/// A matched invocation: the verb and its bound values, `values[i]` parallel to
/// `verb.slots[i]` (a required slot holds exactly one, an optional zero or one, a
/// variadic any number). Judgments consume this; the harness owns everything before
/// it.
#[derive(Debug)]
pub struct Invocation<'a> {
    /// The matched verb.
    pub verb: &'a VerbSpec,
    /// The bound values, parallel to the verb's slots.
    pub values: Vec<Vec<String>>,
}

#[cfg(test)]
mod probes {
    use super::*;

    fn argv(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    /// THE FIRST LOCK of the language constructor: the usage text is an IMAGE of
    /// the declaration, byte-identical to the text the hand-written harness has
    /// always printed. The pin is the ratification — a declaration change that moves
    /// these bytes is a diff to sign, and the hand-written second author is gone
    /// (examples/bundle.rs renders its usage from this declaration).
    #[test]
    fn the_generated_usage_is_bundles_usage_byte_for_byte() {
        assert_eq!(
            CliSpec::bundle().usage(),
            "usage: bundle add <module.rs> <snippet.rs | ->\n\
             \x20      bundle edit <module.rs> <item-name> <replacement.rs | ->\n\
             \x20      bundle declare <module.rs> \"<shape(op, ...)>\"\n\
             \x20      bundle place <module.rs>\n\
             \x20      bundle check <module.rs>\n\
             \x20      bundle show <module.rs> [item-name]\n\
             \x20      bundle collect <module.rs> [item-to-sweep]\n\
             \x20      bundle squash <bundle.journal>\n\
             \x20      bundle replay <bundle.journal>\n\
             \x20      bundle constrains <module.rs> <operator>\n\
             \x20      bundle uses <item-name>\n\
             \x20      bundle spoke <item-name>\n\
             \x20      bundle trace <theory> '<term>'\n\
             \x20      bundle lift <module.rs> <theory-name> [declaration ...]\n\
             \x20      bundle gates | bundle owes\n\
             \x20      bundle pin"
        );
    }

    /// The matcher binds every arity shape the verbs use and refuses everything
    /// else with the usage text: required missing, stray trailing, unknown verb,
    /// empty argv. A refusal binds nothing — there is no partially-bound invocation
    /// to observe.
    #[test]
    fn every_argv_shape_binds_or_refuses_with_usage() {
        let cli = CliSpec::bundle();
        let usage = cli.usage();

        let inv = cli
            .parse(&argv(&["edit", "m.rs", "x", "-"]))
            .expect("binds");
        assert_eq!(inv.verb.name, "edit");
        assert_eq!(inv.values, vec![vec!["m.rs"], vec!["x"], vec!["-"]]);

        let bare = cli.parse(&argv(&["show", "m.rs"])).expect("binds");
        assert_eq!(bare.values, vec![vec!["m.rs".to_string()], vec![]]);
        let one = cli.parse(&argv(&["show", "m.rs", "item"])).expect("binds");
        assert_eq!(one.values[1], vec!["item"]);

        let many = cli
            .parse(&argv(&["lift", "m.rs", "theory", "a", "b", "c"]))
            .expect("binds");
        assert_eq!(many.values[2], vec!["a", "b", "c"]);
        let none = cli
            .parse(&argv(&["lift", "m.rs", "theory"]))
            .expect("binds");
        assert_eq!(none.values[2], Vec::<String>::new());

        assert_eq!(cli.parse(&argv(&["pin"])).expect("binds").values.len(), 0);

        assert_eq!(cli.parse(&argv(&["add", "m.rs"])).unwrap_err(), usage);
        assert_eq!(cli.parse(&argv(&["pin", "stray"])).unwrap_err(), usage);
        assert_eq!(
            cli.parse(&argv(&["show", "m.rs", "item", "stray"]))
                .unwrap_err(),
            usage
        );
        assert_eq!(cli.parse(&argv(&["summon", "m.rs"])).unwrap_err(), usage);
        assert_eq!(cli.parse(&[]).unwrap_err(), usage);
    }

    /// The declaration's census, pinned: seventeen verbs, and the slot vocabulary is
    /// exactly the seven sorts — every sort earns its place by appearing, and an
    /// eighth would be a diff to this probe, i.e. a decision to sign.
    #[test]
    fn seventeen_verbs_speak_exactly_the_seven_sorts() {
        let cli = CliSpec::bundle();
        assert_eq!(cli.verbs.len(), 17);
        let spoken: Vec<Sort> = cli
            .verbs
            .iter()
            .flat_map(|v| v.slots.iter().map(|s| s.sort))
            .collect();
        for sort in [
            Sort::Module,
            Sort::Item,
            Sort::Payload,
            Sort::Declaration,
            Sort::Journal,
            Sort::Theory,
            Sort::Term,
        ] {
            assert!(spoken.contains(&sort), "{sort:?} is never spoken");
        }
    }
}
