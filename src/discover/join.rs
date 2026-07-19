//! join — the journal-level merge (the missing verb the concurrent-session field
//! report named): two suited sessions' records fork at their common prefix, and
//! the sibling's segment replays onto this tree as one envelope — commuting rows
//! stage mechanically, rows whose item BOTH segments touched surface as named
//! decisions (never conflict markers), and `land` judges the composite all or
//! none. The field report's diagnosis holds here: the conflict was never in the
//! items, it was in the renders — so join never reads a render. It reads two
//! records and stages one.
//!
//! Honest bounds, disclosed: the fork is row-identity (order is the only clock,
//! so a shared history is a shared prefix — nothing subtler); a both-touched
//! item is detected by address equality after kind-stripping, so a block-grain
//! edit on one side and a method-grain edit inside it on the other compose
//! silently (the composite judgment still rules); and the frozen verb algebra's
//! conflict classes surface as stage-time refusals rather than being consulted
//! as data — reordering licenses are a later rung.

use std::collections::BTreeSet;

use crate::discover::envelope::split_address;

/// One planned disposition for a sibling row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Planned {
    /// Stage this row into the joining envelope: the verb to stage (a sibling
    /// `recast` stages as `edit` — the envelope re-licenses it), the module,
    /// the detail head, and the payload address where one exists.
    Stage {
        /// The verb to stage.
        verb: String,
        /// The row's module.
        module: String,
        /// The detail head — item name(s), or the declaration.
        head: String,
        /// The payload address, where the verb carries one.
        address: Option<String>,
    },
    /// Not staged; the reason is the report. A both-touched decision carries
    /// the sibling's payload address so taking theirs is one staged verb away.
    Skip {
        /// The sibling row, verbatim.
        row: String,
        /// Why it does not stage.
        why: String,
    },
}

/// The join: pure record arithmetic — splitting, forking, planning. All I/O
/// (reading journals, fetching payloads, staging rows) stays with the caller.
pub struct Join;

#[crate::mutate("join")]
impl Join {
    /// Split a git-conflicted journal into (ours, theirs) whole texts — the
    /// conflict block made two records again, no hand-weaving. `None` when no
    /// markers are present (the file is a clean sibling record).
    pub fn split_conflicted(text: &str) -> Option<(String, String)> {
        if !text.contains("<<<<<<<") {
            return None;
        }
        let mut ours = String::new();
        let mut theirs = String::new();
        let mut side = 0u8;
        for line in text.lines() {
            if line.starts_with("<<<<<<<") {
                side = 1;
                continue;
            }
            if line.starts_with("=======") && side == 1 {
                side = 2;
                continue;
            }
            if line.starts_with(">>>>>>>") {
                side = 0;
                continue;
            }
            if side != 2 {
                ours.push_str(line);
                ours.push('\n');
            }
            if side != 1 {
                theirs.push_str(line);
                theirs.push('\n');
            }
        }
        Some((ours, theirs))
    }

    /// The fork point: the longest common prefix of rows. Returns the shared
    /// row count and the two segments after it, ours then theirs.
    pub fn fork(ours: &str, theirs: &str) -> (usize, Vec<String>, Vec<String>) {
        let a: Vec<&str> = ours.lines().collect();
        let b: Vec<&str> = theirs.lines().collect();
        let shared = a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count();
        (
            shared,
            a[shared..].iter().map(|s| s.to_string()).collect(),
            b[shared..].iter().map(|s| s.to_string()).collect(),
        )
    }

    /// Plan the sibling segment against ours: an effect row stages unless our
    /// segment also touched the same item — a DECISION, reported with the
    /// sibling's payload address so taking theirs is one staged verb away.
    /// Seals and non-stageable verbs skip by name; a row the journal grammar
    /// cannot read refuses (the record is machine-written — a strange row
    /// means a hand).
    pub fn plan(
        ours_segment: &[String],
        theirs_segment: &[String],
    ) -> Result<Vec<Planned>, String> {
        let mine = touched(ours_segment);
        let mut planned = Vec::new();
        for row in theirs_segment {
            let Some((verb, module, detail)) = split_row(row) else {
                return Err(format!(
                    "bundle join: sibling row is not `<verb> <module> — <detail>`: `{row}`"
                ));
            };
            let (head, address) = split_address(detail);
            match verb {
                "seal" => planned.push(Planned::Skip {
                    row: row.clone(),
                    why: "a seal marks the sibling's boundary — the landing writes ours"
                        .to_string(),
                }),
                "add" | "edit" | "recast" | "declare" => {
                    let stage_verb = if verb == "recast" { "edit" } else { verb };
                    let collision = (verb != "declare")
                        .then(|| {
                            head.split(", ")
                                .map(name_of)
                                .find(|name| mine.contains(&(module.to_string(), name.clone())))
                        })
                        .flatten();
                    if let Some(name) = collision {
                        planned.push(Planned::Skip {
                            row: row.clone(),
                            why: format!(
                                "both segments touched `{name}` of {module} — ours stands; \
                                 take theirs with `bundle {stage_verb} {module} …`{}",
                                address
                                    .map(|a| format!(" from payload @{a}"))
                                    .unwrap_or_default()
                            ),
                        });
                    } else {
                        planned.push(Planned::Stage {
                            verb: stage_verb.to_string(),
                            module: module.to_string(),
                            head: head.to_string(),
                            address: address.map(str::to_string),
                        });
                    }
                }
                other => planned.push(Planned::Skip {
                    row: row.clone(),
                    why: format!("`{other}` does not stage — re-run it after the join lands"),
                }),
            }
        }
        Ok(planned)
    }
}

/// The (module, item-name) pairs a segment's effect rows touched — the join's
/// collision surface, kind-stripped so an `add`'s `fn one` meets an `edit`'s
/// `one`.
#[crate::mutate]
fn touched(rows: &[String]) -> BTreeSet<(String, String)> {
    let mut set = BTreeSet::new();
    for row in rows {
        if let Some((verb, module, detail)) = split_row(row) {
            if matches!(verb, "add" | "edit" | "recast") {
                let (head, _) = split_address(detail);
                for name in head.split(", ") {
                    set.insert((module.to_string(), name_of(name)));
                }
            }
        }
    }
    set
}

/// A journal row's three parts — the grammar every record speaks.
#[crate::mutate]
fn split_row(row: &str) -> Option<(&str, &str, &str)> {
    row.split_once(' ')
        .and_then(|(verb, rest)| rest.split_once(" — ").map(|(m, d)| (verb, m, d)))
}

/// A detail head's item name with its kind word stripped — `fn one` and `one`
/// are the same touch.
#[crate::mutate]
fn name_of(head: &str) -> String {
    match head.split_once(' ') {
        Some(("fn" | "struct" | "enum" | "trait" | "mod", rest)) => rest.to_string(),
        _ => head.to_string(),
    }
}

#[cfg(test)]
mod probes {
    use super::{Join, Planned};

    /// A git-conflicted journal splits back into the two records the conflict
    /// block interleaved — shared lines on both sides, each side's rows on its
    /// own — and a clean record splits to `None`.
    #[test]
    fn a_conflicted_journal_splits_into_the_two_records() {
        let text = "add src/m.rs — fn one @0123456789abcdef\n\
                    <<<<<<< HEAD\n\
                    add src/m.rs — fn two @aaaaaaaaaaaaaaaa\n\
                    =======\n\
                    add src/m.rs — fn three @bbbbbbbbbbbbbbbb\n\
                    >>>>>>> sibling\n";
        let (ours, theirs) = Join::split_conflicted(text).expect("markers present");
        assert_eq!(
            ours,
            "add src/m.rs — fn one @0123456789abcdef\n\
             add src/m.rs — fn two @aaaaaaaaaaaaaaaa\n"
        );
        assert_eq!(
            theirs,
            "add src/m.rs — fn one @0123456789abcdef\n\
             add src/m.rs — fn three @bbbbbbbbbbbbbbbb\n"
        );
        assert_eq!(
            Join::split_conflicted(&ours),
            None,
            "clean record, no split"
        );
    }

    /// The fork is the longest common prefix; each segment is what its record
    /// says after it.
    #[test]
    fn the_fork_is_the_longest_common_prefix() {
        let ours = "add src/m.rs — fn one @0123456789abcdef\n\
                    edit src/m.rs — one @aaaaaaaaaaaaaaaa\n";
        let theirs = "add src/m.rs — fn one @0123456789abcdef\n\
                      add src/m.rs — fn two @bbbbbbbbbbbbbbbb\n\
                      declare src/m.rs — commutative(join)\n";
        let (shared, ours_seg, theirs_seg) = Join::fork(ours, theirs);
        assert_eq!(shared, 1);
        assert_eq!(ours_seg, vec!["edit src/m.rs — one @aaaaaaaaaaaaaaaa"]);
        assert_eq!(theirs_seg.len(), 2);
    }

    /// The plan: commuting rows stage (a sibling `recast` stages as `edit` —
    /// the envelope re-licenses it), a both-touched item is a DECISION naming
    /// the sibling's payload, a seal skips as boundary, a non-stageable verb
    /// skips by name, and a strange row refuses.
    #[test]
    fn the_plan_stages_the_commuting_and_names_the_decisions() {
        let ours = vec!["edit src/m.rs — one @aaaaaaaaaaaaaaaa".to_string()];
        let theirs = vec![
            "add src/m.rs — fn two @bbbbbbbbbbbbbbbb".to_string(),
            "recast src/m.rs — impl Count::speak @cccccccccccccccc".to_string(),
            "edit src/m.rs — one @dddddddddddddddd".to_string(),
            "seal envelope — 1 row(s) across 1 file(s) @eeeeeeeeeeeeeeee".to_string(),
            "place src/m.rs — re-placed canonically".to_string(),
            "declare src/m.rs — commutative(join)".to_string(),
        ];
        let planned = Join::plan(&ours, &theirs).expect("plans");
        assert_eq!(
            planned[0],
            Planned::Stage {
                verb: "add".to_string(),
                module: "src/m.rs".to_string(),
                head: "fn two".to_string(),
                address: Some("bbbbbbbbbbbbbbbb".to_string()),
            }
        );
        assert_eq!(
            planned[1],
            Planned::Stage {
                verb: "edit".to_string(),
                module: "src/m.rs".to_string(),
                head: "impl Count::speak".to_string(),
                address: Some("cccccccccccccccc".to_string()),
            },
            "a sibling recast stages as edit — the envelope re-licenses it"
        );
        let Planned::Skip { why, .. } = &planned[2] else {
            panic!("a both-touched item is a decision: {:?}", planned[2]);
        };
        assert!(why.contains("both segments touched `one`"), "{why}");
        assert!(why.contains("@dddddddddddddddd"), "{why}");
        let Planned::Skip { why, .. } = &planned[3] else {
            panic!("a seal skips: {:?}", planned[3]);
        };
        assert!(why.contains("the landing writes ours"), "{why}");
        let Planned::Skip { why, .. } = &planned[4] else {
            panic!("place skips by name: {:?}", planned[4]);
        };
        assert!(why.contains("`place` does not stage"), "{why}");
        assert!(
            matches!(&planned[5], Planned::Stage { verb, .. } if verb == "declare"),
            "declare commutes with everything: {:?}",
            planned[5]
        );
        let refusal = Join::plan(&ours, &["not a row".to_string()]).unwrap_err();
        assert!(refusal.contains("sibling row is not"), "{refusal}");
    }
}
