//!
//! trace — the PERCEPTION verb for "how does this behave on THIS input?": a ground term
//! over a theory's operators, evaluated bottom-up with every step narrated, so the
//! debugging read stops requiring the editor (the editor disposition's first enumerated
//! reach, dissolved). The engine always held both halves — the operator tables and
//! `eval` — this module only gives them a mouth: parse `merge(bump(floor), floor)`
//! against the theory's own operator names, evaluate innermost-first, and render each
//! reduction with the observed value.
//!
//! GROUND terms only, deliberately: a variable has no value to trace (that is what the
//! laws quantify over — discovery's business, not trace's), so a bare name must be a
//! nullary operator and anything else refuses naming the vocabulary. Partiality is a
//! named refusal too: an operator declining its inputs is a fact worth seeing, not an
//! error to bury.

use super::engine::Theory;

/// One traced evaluation: the reduction steps (innermost first, each `subterm ⇒ observed
/// value`) and the final observation.
#[derive(Debug)]
pub struct Trace {
    /// Each reduction, rendered: `add_a(empty) ⇒ BundleState { .. }`.
    pub steps: Vec<String>,
    /// The whole term's observed value, rendered.
    pub result: String,
}

/// A parsed ground term, keeping the source text of every node for the narration.
struct Ground {
    op: usize,
    text: String,
    args: Vec<Ground>,
}

#[crate::mutate("trace")]
impl Trace {
    /// Trace a ground term over `T`'s operators. Refusals are named, never guessed: an
    /// unknown operator teaches the theory's vocabulary, an arity mismatch shows both
    /// counts, a non-ground leaf points at the laws (variables are discovery's), and a
    /// partial operator's refusal names the operator and its inputs.
    pub fn of<T: Theory>(term_text: &str) -> Result<Trace, String>
    where
        T::Obs: std::fmt::Debug,
    {
        let ops = T::operators();
        let names: Vec<&str> = ops.iter().map(|o| o.name).collect();
        let (ground, rest) = parse_ground(term_text.trim(), &names)?;
        if !rest.trim_start().is_empty() {
            return Err(format!(
                "trace: trailing text `{}` after the term — one term, one trace",
                rest.trim()
            ));
        }
        // arity check the whole tree before evaluating anything.
        check_arity(
            &ground,
            &names,
            &ops.iter().map(|o| o.inputs.len()).collect::<Vec<_>>(),
        )?;

        let mut steps = Vec::new();
        let value = eval_ground::<T>(&ground, &ops, &mut steps)?;
        Ok(Trace {
            steps,
            result: format!("{:?}", T::observe(&value)),
        })
    }

    /// The trace as one readable block — the CLI's render.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for step in &self.steps {
            out.push_str("  ");
            out.push_str(step);
            out.push('\n');
        }
        out.push_str(&format!("result: {}", self.result));
        out
    }
}

/// Parse one ground term from the front of `text`: `name` or `name(args, ...)`, names
/// resolved against the theory's operator table — an unknown name refuses TEACHING the
/// vocabulary. Returns the node and the unconsumed remainder.
#[crate::mutate]
fn parse_ground<'t>(text: &'t str, names: &[&str]) -> Result<(Ground, &'t str), String> {
    let text = text.trim_start();
    let split = text
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    let name = &text[..split];
    if name.is_empty() {
        return Err(format!(
            "trace: expected an operator name at `{}`",
            &text[..text.len().min(20)]
        ));
    }
    let Some(op) = names.iter().position(|n| *n == name) else {
        return Err(format!(
            "trace: `{name}` is not an operator of this theory. Operators: {}",
            names.join(", ")
        ));
    };
    let mut rest = &text[split..];
    let mut args = Vec::new();
    let mut rendered = name.to_string();
    if let Some(inner) = rest.trim_start().strip_prefix('(') {
        rest = inner;
        rendered.push('(');
        loop {
            let (arg, after) = parse_ground(rest, names)?;
            if !args.is_empty() {
                rendered.push_str(", ");
            }
            rendered.push_str(&arg.text);
            args.push(arg);
            let after = after.trim_start();
            if let Some(after_comma) = after.strip_prefix(',') {
                rest = after_comma;
            } else if let Some(after_close) = after.strip_prefix(')') {
                rest = after_close;
                rendered.push(')');
                break;
            } else {
                return Err(format!(
                    "trace: expected `,` or `)` in the arguments of `{name}`"
                ));
            }
        }
    }
    Ok((
        Ground {
            op,
            text: rendered,
            args,
        },
        rest,
    ))
}

/// Arity, checked over the whole tree before evaluation — a malformed term refuses whole,
/// never half-evaluates.
#[crate::mutate]
fn check_arity(node: &Ground, names: &[&str], arities: &[usize]) -> Result<(), String> {
    if node.args.len() != arities[node.op] {
        return Err(format!(
            "trace: `{}` takes {} argument(s), got {}",
            names[node.op],
            arities[node.op],
            node.args.len()
        ));
    }
    for arg in &node.args {
        check_arity(arg, names, arities)?;
    }
    Ok(())
}

/// Evaluate innermost-first, narrating each reduction with the theory's own observation.
#[crate::mutate]
fn eval_ground<T: Theory>(
    node: &Ground,
    ops: &[super::engine::Operator<T>],
    steps: &mut Vec<String>,
) -> Result<T::Value, String>
where
    T::Obs: std::fmt::Debug,
{
    let mut values = Vec::new();
    for arg in &node.args {
        values.push(eval_ground::<T>(arg, ops, steps)?);
    }
    let value = (ops[node.op].eval)(&values).ok_or_else(|| {
        format!(
            "trace: `{}` declined its inputs at `{}` — a partial operator's refusal is a \
             fact, shown rather than buried",
            ops[node.op].name, node.text
        )
    })?;
    steps.push(format!("{} ⇒ {:?}", node.text, T::observe(&value)));
    Ok(value)
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::verbs::state::VerbAlgebra;

    /// THE CONFLICT, MADE VISIBLE: the two orderings of the verb algebra's add/edit
    /// conflict traced side by side — same verbs, different histories, different states —
    /// the exact fact the frozen lock states as an ABSENT law, now watchable on one
    /// input without opening any file.
    #[test]
    fn the_trace_makes_the_conflict_visible() {
        let one = Trace::of::<VerbAlgebra>("edit_a(add_a(empty))").expect("traces");
        let other = Trace::of::<VerbAlgebra>("add_a(edit_a(empty))").expect("traces");
        assert_ne!(
            one.result, other.result,
            "the add/edit conflict is a visible divergence"
        );
        assert!(one.result.contains("V1"), "{}", one.result);
        assert!(other.result.contains("V0"), "{}", other.result);
        // every reduction is narrated, innermost first.
        assert_eq!(one.steps.len(), 3);
        assert!(one.steps[0].starts_with("empty ⇒ "), "{:?}", one.steps);
        assert!(
            one.steps[1].starts_with("add_a(empty) ⇒ "),
            "{:?}",
            one.steps
        );
        assert!(one.render().ends_with(&format!("result: {}", one.result)));
    }

    /// The refusals teach: an unknown operator lists the vocabulary, an arity mismatch
    /// shows both counts, trailing text refuses, and a bare unknown leaf is caught the
    /// same way — perception never guesses.
    #[test]
    fn trace_refusals_teach_the_vocabulary() {
        let err = Trace::of::<VerbAlgebra>("summon(empty)").unwrap_err();
        assert!(err.contains("not an operator"), "{err}");
        assert!(err.contains("add_a"), "the vocabulary is taught: {err}");
        let err = Trace::of::<VerbAlgebra>("add_a(empty, empty)").unwrap_err();
        assert!(err.contains("takes 1 argument(s), got 2"), "{err}");
        let err = Trace::of::<VerbAlgebra>("empty empty").unwrap_err();
        assert!(err.contains("trailing text"), "{err}");
    }
}
