//!
//! modularize — SELECT the right shapes out of a flat bag of functions. The whole discovery stack,
//! turned around and pointed at the PATHOLOGICAL case: one file with everything crammed together,
//! functions that belong to several unrelated algebras all thrown in a heap. Cohesion, layering, and
//! the discovered laws were built to critique a module that already has a shape; here there is no
//! shape yet, and the job is to PROPOSE one.
//!
//! The algebra IS the selection criterion. Give it a flat `#[algebra]` bag and it:
//!
//!   - PARTITIONS the functions by law-connectivity — the same law-connected components `cohesion`
//!     reads off (and `layering` recomputes as it looks for hinges), each a candidate module;
//!   - SCORES each candidate by how many discovered laws live inside it — a component the laws bind
//!     tightly is a real algebraic shape; a lone function no law ever mentions is NOISE; and
//!   - checks each shape's internal tightness (`layering`) — atomic (one layer) or sprawling.
//!
//! The partition and the tightness both come from the SAME `layering` report — it already computes
//! the law-connectivity components (to find their hinges), so modularize reads the components and
//! their atomicity from one place rather than re-deriving and re-matching a partition.
//!
//! What falls out is a RANKING: the richest shapes first (a lattice with ten laws beats a bare
//! semilattice with three), and at the bottom the MISFITS — functions that cohere with nothing, which
//! the proposal refuses to dress up as a module. So a good decomposition is not designed, it is READ
//! OFF: propose the law-bound clusters as modules, name them, and leave the misfits flagged as the
//! functions that do not (yet) mean anything algebraically. It is the culmination of the stack —
//! cohesion said "this module wants to split", layering said "this component wants to layer", and
//! modularize says "here is the structure hiding in your unstructured bag, ranked by how real it is".

use std::collections::BTreeSet;

use super::engine::{Engine, Term, Theory};
use super::layering::LayeringReport;

/// One proposed module read off the bag: the functions that cluster together, how many discovered
/// laws bind them, and whether the cluster is internally atomic or sprawling.
pub struct ProposedModule {
    /// The operator symbols this candidate module owns.
    pub operators: Vec<&'static str>,
    /// How many discovered laws live entirely inside this cluster — the richness score.
    pub laws: usize,
    /// Atomic (one tight layer, no hinge) versus sprawling (holds together through a hinge).
    pub atomic: bool,
}

impl ProposedModule {
    /// A genuine algebraic SHAPE — the functions cohere into at least one law. A cluster with no law
    /// is not a module, it is a misfit.
    pub fn is_shape(&self) -> bool {
        self.laws > 0
    }
}

/// The proposed decomposition of a flat bag of functions: the candidate modules, richest first, with
/// the law-less misfits sunk to the bottom.
pub struct Proposal {
    pub theory: &'static str,
    pub modules: Vec<ProposedModule>,
}

impl Proposal {
    /// Propose a modular decomposition of a theory's (flat) algebra. The analysis is an
    /// associated function of its PROPOSAL — the public surface is the value object, not a
    /// loose function (the no-rats-nest rule: every public callable hangs off a typestate).
    pub fn of<T: Theory>() -> Self {
        modularize::<T>()
    }

    /// Render the proposal as a readable suggestion — the shapes ranked, the misfits named.
    pub fn render(&self) -> String {
        let mut out = format!("bag `{}` → proposed decomposition:\n", self.theory);
        for (i, m) in self.modules.iter().filter(|m| m.is_shape()).enumerate() {
            let layer = if m.atomic { "atomic" } else { "layered" };
            out.push_str(&format!(
                "  shape {i}: {{ {} }} — {} law(s), {layer}\n",
                m.operators.join(", "),
                m.laws
            ));
        }
        let misfits = self.misfits();
        if misfits.is_empty() {
            out.push_str("  no misfits — every function found a shape.\n");
        } else {
            out.push_str(&format!(
                "  misfits (bound by no law — left unstructured): {}\n",
                misfits.join(", ")
            ));
        }
        out
    }

    /// The proposed shapes — clusters that carry at least one law, best first.
    pub fn shapes(&self) -> Vec<&ProposedModule> {
        self.modules.iter().filter(|m| m.is_shape()).collect()
    }

    /// The MISFITS — functions in no law, which the proposal refuses to package as a module.
    pub fn misfits(&self) -> Vec<&'static str> {
        self.modules
            .iter()
            .filter(|m| !m.is_shape())
            .flat_map(|m| m.operators.iter().copied())
            .collect()
    }
}

/// Collect the operator indices a term mentions.
fn ops_in(t: &Term, out: &mut BTreeSet<usize>) {
    if let Term::App(op, args) = t {
        out.insert(*op);
        for a in args {
            ops_in(a, out);
        }
    }
}

/// Propose a modular decomposition of a theory's (flat) algebra. Partition by law-connectivity, score
/// each cluster by the laws it holds, mark its tightness, and rank the shapes ahead of the misfits.
/// (Private — reached as `Proposal::of`.)
fn modularize<T: Theory>() -> Proposal {
    let engine = Engine::<T>::new();
    let sigs = engine.signatures();
    let layers = LayeringReport::of::<T>();

    // score: how many discovered laws live entirely inside each component. Because the components are
    // the law-connectivity clusters, every law's operators fall wholly within exactly one — so a law
    // whose operators a component all contains is one the component fully owns.
    let mut law_counts = vec![0usize; layers.components.len()];
    for law in engine.discover().laws {
        let mut ops = BTreeSet::new();
        ops_in(&law.lhs, &mut ops);
        ops_in(&law.rhs, &mut ops);
        let symbols: Vec<&'static str> = ops.iter().map(|&i| sigs[i].0).collect();
        if let Some(idx) = layers
            .components
            .iter()
            .position(|c| symbols.iter().all(|s| c.operators.contains(s)))
        {
            law_counts[idx] += 1;
        }
    }

    // the partition and the tightness both come from the one layering report — no re-matching.
    let mut modules: Vec<ProposedModule> = layers
        .components
        .iter()
        .enumerate()
        .map(|(i, c)| ProposedModule {
            operators: c.operators.clone(),
            laws: law_counts[i],
            atomic: c.is_atomic(),
        })
        .collect();

    // rank richest-first; misfits (zero laws) sink to the bottom. A stable sort keeps clusters of
    // equal richness in their cohesion order, so the proposal is deterministic.
    modules.sort_by(|a, b| b.laws.cmp(&a.laws));

    Proposal {
        theory: T::name(),
        modules,
    }
}

/// A deliberately PATHOLOGICAL flat bag: four functions over three unrelated value types, all dumped
/// in one module with no structure — the input a real agent would hand us. `peak` is a `max`
/// semilattice on `Count`; `both`/`either` are the `and`/`or` of a lattice on `Flag`; `rotate` is a
/// three-cycle on `Spin` that satisfies no law at all. `modularize` is asked to find the structure:
/// it should propose the lattice and the semilattice as modules (the lattice richer) and flag
/// `rotate` as a misfit. `#[algebra]` synthesises the whole multi-sorted theory from just these
/// functions and their `#[derive(Shaped)]` value objects — nothing about the decomposition is
/// declared. (Names kept clear of the crate's `boundary` type vocabulary.)
#[crate::algebra(Soup, "flat soup")]
pub mod soup {
    use crate::Shaped;

    /// A three-level count — `max` makes it a semilattice.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
    pub enum Count {
        C0,
        C1,
        C2,
    }
    /// A boolean flag — `both`/`either` make it a lattice.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
    pub enum Flag {
        No,
        Yes,
    }
    /// A three-way rotation — `rotate` cycles it, satisfying no universal shape.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Shaped)]
    pub enum Spin {
        Spin0,
        Spin1,
        Spin2,
    }

    /// `max` on the count — a commutative, associative, idempotent semilattice.
    pub fn peak(a: Count, b: Count) -> Count {
        a.max(b)
    }
    /// `and` on the flag — the meet of the lattice.
    pub fn both(a: Flag, b: Flag) -> Flag {
        a.min(b)
    }
    /// `or` on the flag — the join of the lattice.
    pub fn either(a: Flag, b: Flag) -> Flag {
        a.max(b)
    }
    /// A three-cycle on the spin — `rotate(rotate(rotate(x))) = x`, but no BINARY/UNARY universal law
    /// (not involution: `rotate(rotate(x)) != x`), so it coheres with nothing.
    pub fn rotate(x: Spin) -> Spin {
        match x {
            Spin::Spin0 => Spin::Spin1,
            Spin::Spin1 => Spin::Spin2,
            Spin::Spin2 => Spin::Spin0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::soup::Soup;
    use super::*;

    /// The headline: from a flat bag of four functions over three unrelated types, `modularize`
    /// proposes exactly the two hidden algebras as modules and flags the structureless function as a
    /// misfit — nothing about this decomposition was written down, it was read off the discovered
    /// laws.
    #[test]
    fn it_selects_the_hidden_shapes_and_flags_the_misfit() {
        let p = Proposal::of::<Soup>();
        let shapes = p.shapes();
        assert_eq!(
            shapes.len(),
            2,
            "two real shapes: the lattice and the semilattice"
        );

        // richest first: the Flag lattice (both/either — ten lattice laws) outranks the Count
        // semilattice (peak — three laws).
        let lattice = shapes[0];
        assert!(
            lattice.operators.contains(&"both") && lattice.operators.contains(&"either"),
            "the top shape is the two-operator lattice: {:?}",
            lattice.operators
        );
        assert_eq!(lattice.laws, 10, "the distributive lattice's ten laws");

        let semilattice = shapes[1];
        assert_eq!(
            semilattice.operators,
            vec!["peak"],
            "the semilattice is peak alone"
        );
        assert_eq!(
            semilattice.laws, 3,
            "commutativity, associativity, idempotence"
        );

        // and `rotate` — bound by no law — is the misfit, not packaged as a module.
        assert_eq!(p.misfits(), vec!["rotate"], "rotate coheres with nothing");
    }

    /// The ranking is by richness: every proposed shape carries at least one law, they are in
    /// descending law-count order, and the misfits (zero laws) all sit AFTER the shapes. Pins the
    /// sort and the shape/misfit split against a flipped comparison.
    #[test]
    fn shapes_are_ranked_by_richness_above_the_misfits() {
        let p = Proposal::of::<Soup>();
        // shapes carry laws, misfit modules carry none.
        assert!(p
            .modules
            .iter()
            .filter(|m| m.is_shape())
            .all(|m| m.laws > 0));
        assert!(p
            .modules
            .iter()
            .filter(|m| !m.is_shape())
            .all(|m| m.laws == 0));
        // descending by law count across the whole proposal (so misfits necessarily land last).
        for w in p.modules.windows(2) {
            assert!(w[0].laws >= w[1].laws, "proposal is not richest-first");
        }
        // the last module is the misfit.
        assert!(!p.modules.last().unwrap().is_shape());
    }

    /// Each proposed shape is INTERNALLY tight — both the lattice and the semilattice are atomic (no
    /// load-bearing hinge), so the proposal does not also ask to layer them. Pins the atomicity read
    /// against the layering report.
    #[test]
    fn the_proposed_shapes_are_atomic() {
        let p = Proposal::of::<Soup>();
        assert!(
            p.shapes().iter().all(|m| m.atomic),
            "both hidden algebras are single tight layers"
        );
    }

    /// The report renders readably — the ranked shapes with their law counts, and the misfit named as
    /// left unstructured.
    #[test]
    fn the_report_renders_readably() {
        let text = Proposal::of::<Soup>().render();
        assert!(text.contains("bag `flat soup`"));
        assert!(text.contains("shape 0: { both, either } — 10 law(s), atomic"));
        assert!(text.contains("shape 1: { peak } — 3 law(s), atomic"));
        assert!(
            text.contains("misfits (bound by no law — left unstructured): rotate"),
            "the misfit is named: {text}"
        );
    }
}
