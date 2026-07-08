//!
//! scaffold — turn a cohesion SUGGESTION into the SPLIT: emit one `theory!` sub-module per latent
//! component, plus the seam obligation that keeps the split honest.
//!
//! This is the action half of the cohesion loop (the report is the signal). Given a module whose
//! discovered algebra decomposes, it generates the skeleton of each sub-module — operators grouped
//! by component, with their names, fixities, and sorts faithfully carried, and the `eval` functions
//! left as `move-here` placeholders (the interior is what moves; it cannot be regenerated). The
//! value/observe/inhabitant plumbing the engine cannot see at runtime is left to fill in (a human or
//! an agent, naming as it goes).
//!
//! The SAFETY of the split is structural. Because the components are defined by law-connectivity,
//! every discovered law lives entirely inside one component — so the split loses no law. The only
//! obligation is the seam: a TRANSPORT seam shares a sort that is the SAME type on both sides, so the
//! algebra is preserved by construction (no check needed); a TRANSFORM seam carries a conversion
//! that must be a HOMOMORPHISM, and the scaffold emits that obligation so a bad cut is a failing
//! check, not a silent bug.

use super::cohesion::{CohesionReport, SeamKind};
use super::engine::{Engine, Theory};

/// A generated sub-module: its placeholder name, the operators it owns, and its `theory!` skeleton.
pub struct ModulePlan {
    pub name: String,
    pub operators: Vec<&'static str>,
    pub source: String,
}

/// What the split must preserve at a seam.
pub struct SeamObligation {
    pub kind: SeamKind,
    pub note: String,
}

/// The full scaffold for a decomposable module.
pub struct Scaffold {
    pub theory: &'static str,
    pub modules: Vec<ModulePlan>,
    pub seams: Vec<SeamObligation>,
}

#[crate::mutate]
impl Scaffold {
    /// Generate the split for a theory's discovered cohesion components. Returns `None` when the
    /// module is cohesive (a single algebra — nothing to split). The generation is an associated
    /// function of its SCAFFOLD — the public surface is the value object, not a loose function
    /// (the no-rats-nest rule: every public callable hangs off a typestate).
    pub fn of<T: Theory>() -> Option<Self> {
        scaffold::<T>()
    }

    /// Generate the split for a theory's PLACEMENT components — the placer's action
    /// half. Returns `None` when the placement is settled (one component — the declared
    /// boundary is the derived boundary). Unlike the cohesion split, a placement split
    /// carries NO seam obligations: the components share no sorts, so nothing crosses
    /// the cut. It is lossless for the same structural reason as the cohesion split,
    /// one step stronger — a law's operators are net-connected through its terms'
    /// sorts, so every discovered law lives entirely inside one component.
    pub fn placement<T: Theory>() -> Option<Self> {
        let placement = super::shape::Placement::of::<T>();
        if placement.is_settled() {
            return None;
        }
        let decls = Engine::<T>::new().declarations();
        let modules = placement
            .components
            .iter()
            .enumerate()
            .map(|(idx, component)| {
                let owned: Vec<_> = decls
                    .iter()
                    .filter(|d| component.ops.contains(&d.1))
                    .collect();
                ModulePlan {
                    name: format!("Module{idx}"),
                    operators: component.ops.clone(),
                    source: render_module::<T>(idx, &owned),
                }
            })
            .collect();
        Some(Scaffold {
            theory: T::name(),
            modules,
            seams: Vec::new(),
        })
    }

    /// Render the whole scaffold as a readable, paste-able skeleton.
    pub fn render<T: Theory>() -> String {
        match Self::of::<T>() {
            None => format!("module `{}` is cohesive — nothing to split.\n", T::name()),
            Some(s) => {
                let mut out = format!(
                    "// scaffold for `{}` — {} sub-modules. Move the operator `eval` functions into the\n\
                     // matching block, name the modules, and fill the value-level plumbing.\n\n",
                    s.theory,
                    s.modules.len()
                );
                for m in &s.modules {
                    out.push_str(&format!("// {} owns: {}\n", m.name, m.operators.join(", ")));
                    out.push_str(&m.source);
                    out.push('\n');
                }
                for seam in &s.seams {
                    out.push_str(&format!("// SEAM — {}\n", seam.note));
                }
                out
            }
        }
    }
}

/// Generate the split for a theory's discovered cohesion components. Returns `None` when the module
/// is cohesive (a single algebra — nothing to split). (Private — reached as `Scaffold::of`.)
#[crate::mutate]
fn scaffold<T: Theory>() -> Option<Scaffold> {
    let report = CohesionReport::of::<T>();
    if report.is_cohesive() {
        return None;
    }
    let decls = Engine::<T>::new().declarations();

    let modules = report
        .components
        .iter()
        .enumerate()
        .map(|(idx, ops)| {
            let owned: Vec<_> = decls.iter().filter(|d| ops.contains(&d.1)).collect();
            ModulePlan {
                name: format!("Module{idx}"),
                operators: ops.clone(),
                source: render_module::<T>(idx, &owned),
            }
        })
        .collect();

    let seams = report
        .seams
        .iter()
        .map(|s| {
            let note = match s.kind {
                SeamKind::Transport => format!(
                    "transport seam on {}: the shared sort is the SAME type on both sides, so the \
                     algebra is preserved by construction — no check needed.",
                    s.shared.join(", ")
                ),
                SeamKind::Transform => format!(
                    "transform seam on {}: the conversion across it must be a HOMOMORPHISM — emit \
                     `h(a op b) == h(a) op' h(b)` as a probe so a bad cut fails a check.",
                    s.shared.join(", ")
                ),
            };
            SeamObligation { kind: s.kind, note }
        })
        .collect();

    Some(Scaffold {
        theory: T::name(),
        modules,
        seams,
    })
}

/// Render one sub-module as a `theory!` skeleton — operators faithful, the value-level plumbing the
/// engine cannot see at runtime left as fill-in.
#[crate::mutate]
fn render_module<T: Theory>(idx: usize, ops: &[&super::engine::OpDeclaration<T::Sort>]) -> String {
    // the sorts this sub-module touches, with their variable names (those the engine DOES know).
    let mut sorts: Vec<T::Sort> = Vec::new();
    for d in ops {
        for s in d.3.iter().chain(std::iter::once(&d.4)) {
            if !sorts.contains(s) {
                sorts.push(*s);
            }
        }
    }

    let mut out = format!("crate::theory! {{\n    Module{idx} : \"module{idx} (rename me)\",\n");
    out.push_str(
        "    Value = /* the original Value type */, Obs = /* original Obs */, Sort = Sort,\n",
    );
    out.push_str("    sort_of = /* carried from the original */,\n");
    out.push_str("    observe = /* carried from the original */,\n");
    out.push_str("    vars {\n");
    for s in &sorts {
        let names: Vec<String> = T::sort_vars(*s).iter().map(|v| format!("{v:?}")).collect();
        out.push_str(&format!(
            "        Sort::{s:?} => &[{}],\n",
            names.join(", ")
        ));
    }
    out.push_str("    }\n    inhabit {\n");
    for s in &sorts {
        out.push_str(&format!("        Sort::{s:?} => /* carried */,\n"));
    }
    out.push_str("    }\n    ops {\n");
    for d in ops {
        let (name, sym, fixity, inputs, output) = (d.0, d.1, d.2, &d.3, &d.4);
        let ins: Vec<String> = inputs.iter().map(|s| format!("Sort::{s:?}")).collect();
        out.push_str(&format!(
            "        {fixity:?} {name:?} {sym:?} ({}) -> Sort::{output:?} = /* move `{sym}` here */;\n",
            ins.join(", ")
        ));
    }
    out.push_str("    }\n}\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::arithmetic::Arithmetic;
    use crate::discover::date::Calendar;
    use crate::discover::router::Router;

    /// A cohesive module scaffolds to nothing — there is no split to make.
    #[test]
    fn a_cohesive_module_does_not_scaffold() {
        assert!(Scaffold::of::<Router>().is_none());
        assert!(Scaffold::render::<Router>().contains("cohesive"));
    }

    /// The date calculus scaffolds into TWO sub-modules across a TRANSFORM seam (the layer line).
    /// The `since`/`at` module owns exactly the conversion; the seam obligation is the homomorphism.
    #[test]
    fn the_date_calculus_scaffolds_into_two_modules() {
        let s = Scaffold::of::<Calendar>().expect("date decomposes");
        assert_eq!(s.modules.len(), 2);
        let conversion = s
            .modules
            .iter()
            .find(|m| m.operators.contains(&"since"))
            .expect("a conversion module");
        assert!(conversion.operators.contains(&"at"));
        assert_eq!(conversion.operators.len(), 2);
        assert_eq!(s.seams.len(), 1);
        assert_eq!(s.seams[0].kind, SeamKind::Transform);
        assert!(s.seams[0].note.contains("HOMOMORPHISM"));
    }

    /// The generated source is a faithful `theory!` skeleton: it carries each owned operator's name,
    /// symbol, fixity, and sorts, and marks its `eval` as move-here. Pins the renderer.
    #[test]
    fn the_generated_source_is_a_faithful_theory_block() {
        let s = Scaffold::of::<Arithmetic>().expect("arithmetic decomposes");
        let arith = s
            .modules
            .iter()
            .find(|m| m.operators.contains(&"+"))
            .expect("arithmetic module");
        // the `theory!` skeleton, the operator declaration, and the move-here marker are all present.
        assert!(arith.source.contains("crate::theory! {"));
        assert!(arith
            .source
            .contains("Infix \"Addition\" \"+\" (Sort::Int, Sort::Int) -> Sort::Int"));
        assert!(arith.source.contains("move `+` here"));
        // the sorts the module touches are carried, with their variable names.
        assert!(arith.source.contains("Sort::Int => &[\"x\", \"y\", \"z\"]"));
        // the comparison operators are NOT in the arithmetic module.
        assert!(!arith.source.contains("\"<\""));
    }

    /// The split is LOSSLESS: because components are defined by law-connectivity, every discovered
    /// law's operators live entirely inside one sub-module — so no law is orphaned by the cut.
    #[test]
    fn every_discovered_law_lands_in_one_sub_module() {
        use crate::discover::engine::Term;
        fn ops_in(t: &Term, out: &mut Vec<usize>) {
            if let Term::App(op, args) = t {
                out.push(*op);
                for a in args {
                    ops_in(a, out);
                }
            }
        }
        let engine = Engine::<Calendar>::new();
        let sigs = engine.signatures();
        let s = Scaffold::of::<Calendar>().expect("date decomposes");
        for law in engine.discover().laws {
            let mut idxs = Vec::new();
            ops_in(&law.lhs, &mut idxs);
            ops_in(&law.rhs, &mut idxs);
            let symbols: Vec<&str> = idxs.iter().map(|&i| sigs[i].0).collect();
            let owners = s
                .modules
                .iter()
                .filter(|m| symbols.iter().any(|sym| m.operators.contains(sym)))
                .count();
            assert_eq!(owners, 1, "law `{}` spans {owners} modules", law.equation);
        }
    }
}
