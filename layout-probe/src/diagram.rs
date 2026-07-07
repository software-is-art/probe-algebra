//! diagram — the SOURCE side of the layout domain: what an agent edits.
//!
//! A [`Diagram`] is a declaration list of named nodes plus directed edges plus one
//! style bit (the theme). The operators are exactly the edits an agent loop makes
//! between iterations — reorder the declarations, toggle the theme, rename via a swap —
//! and the whole question of this member is which of those edits a layout engine's
//! OUTPUT is allowed to see. Canonical by construction: the mint dedups node names
//! (first declaration wins) and keeps only edges between declared nodes, so equality
//! on sources is honest before any engine runs.
//!
//! Renames are SWAPS on purpose: a swap is total and collision-free on any name set
//! (renaming `a → b` when `b` exists would merge nodes — an edit with different
//! semantics, out of scope for v1), and the swap-closed grid keeps every rename's
//! image on the grid.

/// A diagram source: declared nodes (order is meaningful — it is exactly what the
/// `eager` engine leaks), directed edges, and the theme bit.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Diagram {
    nodes: Vec<String>,
    edges: Vec<(String, String)>,
    dark: bool,
}

impl Diagram {
    /// The one mint: duplicate node declarations collapse (first wins), edges touching
    /// undeclared nodes are dropped — a malformed diagram is unconstructible.
    pub fn of(nodes: &[&str], edges: &[(&str, &str)]) -> Diagram {
        let mut seen: Vec<String> = Vec::new();
        for n in nodes {
            if !seen.iter().any(|s| s == n) {
                seen.push((*n).to_string());
            }
        }
        let edges = edges
            .iter()
            .filter(|(a, b)| seen.iter().any(|s| s == a) && seen.iter().any(|s| s == b))
            .map(|(a, b)| ((*a).to_string(), (*b).to_string()))
            .collect();
        Diagram {
            nodes: seen,
            edges,
            dark: false,
        }
    }

    /// The declared node names, in declaration order — the exit hatch the engines read.
    pub fn nodes(&self) -> &[String] {
        &self.nodes
    }

    /// The edges, as declared.
    pub fn edges(&self) -> &[(String, String)] {
        &self.edges
    }

    /// ROTATE the declaration list by one — the canonical "an agent re-emitted the
    /// same diagram in a different order" edit. Pure permutation: nothing else moves.
    pub fn reorder(&self) -> Diagram {
        let mut nodes = self.nodes.clone();
        if !nodes.is_empty() {
            nodes.rotate_left(1);
        }
        Diagram {
            nodes,
            edges: self.edges.clone(),
            dark: self.dark,
        }
    }

    /// Toggle the theme bit — a STYLE edit; geometry has no business seeing it.
    pub fn theme(&self) -> Diagram {
        Diagram {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            dark: !self.dark,
        }
    }

    /// Apply a rename (a swap) to every node and edge endpoint, in place in the
    /// declaration order — names change, ORDER does not.
    pub fn rename(&self, swap: &Swap) -> Diagram {
        Diagram {
            nodes: self.nodes.iter().map(|n| swap.apply(n)).collect(),
            edges: self
                .edges
                .iter()
                .map(|(a, b)| (swap.apply(a), swap.apply(b)))
                .collect(),
            dark: self.dark,
        }
    }
}

/// A rename as a SWAP of two names — the action parameter sort.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Swap(pub String, pub String);

impl Swap {
    /// Mint a swap.
    pub fn of(a: &str, b: &str) -> Swap {
        Swap(a.to_string(), b.to_string())
    }

    /// One name through the swap.
    pub fn apply(&self, name: &str) -> String {
        if name == self.0 {
            self.1.clone()
        } else if name == self.1 {
            self.0.clone()
        } else {
            name.to_string()
        }
    }

    /// The display form the observation carries.
    pub fn render(&self) -> String {
        format!("{}<->{}", self.0, self.1)
    }
}

#[cfg(test)]
mod probes {
    use super::*;

    /// The mint's canonical form, from every door: duplicate declarations collapse
    /// (first wins), dangling edges drop, and the exit hatches read back exactly what
    /// survived.
    #[test]
    fn the_mint_dedups_and_drops_dangling_edges() {
        let d = Diagram::of(&["a", "b", "a"], &[("a", "b"), ("a", "ghost")]);
        assert_eq!(d.nodes(), ["a".to_string(), "b".to_string()]);
        assert_eq!(d.edges(), [("a".to_string(), "b".to_string())]);
    }

    /// The three edits do exactly their one thing: reorder rotates declarations and
    /// nothing else, theme flips only the bit, rename swaps names everywhere while
    /// leaving declaration ORDER alone (the fact the eager engine's equivariance
    /// leans on).
    #[test]
    fn each_edit_moves_exactly_its_own_dimension() {
        let d = Diagram::of(&["b", "a", "c"], &[("b", "a")]);
        let r = d.reorder();
        assert_eq!(
            r.nodes(),
            ["a".to_string(), "c".to_string(), "b".to_string()]
        );
        assert_eq!(r.edges(), d.edges(), "reorder must not touch edges");
        assert_eq!(
            d.reorder().reorder().reorder(),
            d,
            "rotation has order 3 here"
        );
        assert_eq!(d.theme().theme(), d, "theme is an involution");
        assert_ne!(d.theme(), d, "theme actually flips the bit");
        let s = Swap::of("a", "b");
        let renamed = d.rename(&s);
        assert_eq!(
            renamed.nodes(),
            ["a".to_string(), "b".to_string(), "c".to_string()],
            "names swap IN PLACE: order untouched"
        );
        assert_eq!(renamed.edges(), [("a".to_string(), "b".to_string())]);
        assert_eq!(d.rename(&s).rename(&s), d, "a swap undoes itself");
        assert_eq!(s.apply("c"), "c", "a swap leaves bystanders alone");
        assert_eq!(s.render(), "a<->b");
    }
}
