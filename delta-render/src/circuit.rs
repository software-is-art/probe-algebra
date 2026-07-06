//! circuit — the operator DAG, the derivation rules, and the RENDER step.
//!
//! A [`Circuit`] is a value object: nodes reference operators BY THE NAME that keys
//! `spec/licenses.spec`, edges are indices into a single space (sources first, then
//! node outputs), and validity is the smart constructor — acyclic by construction
//! (a node may only reference what precedes it), arities respected, and every
//! operator LICENSED (present in the registry): an unlicensed operator is
//! unconstructible, not a runtime error.
//!
//! The DERIVATION rules, each carrying its citation into the rendered output — the
//! rule table's trust root is the DBSP paper (Budiu et al., VLDB 2023):
//!
//! - LINEAR ⇒ `Q^Δ = Q` — a linear operator commutes with differentiation and
//!   integration, so it is its own incremental form;
//! - BILINEAR ⇒ the three-term delta:
//!   `Δ(a ⋈ b) = Δa ⋈ delay(I(Δb)) plus delay(I(Δa)) ⋈ Δb plus Δa ⋈ Δb`;
//! - composition ⇒ the chain rule (compose the derived forms, node by node);
//! - NEITHER ⇒ the generic fallback `Q^Δ = D ∘ Q ∘ I` — integrate, recompute,
//!   differentiate. Correct always, cheap never. This rule existing is what makes the
//!   render TOTAL; licenses only ever upgrade nodes.
//!
//! Time-invariance and causality are BY CONSTRUCTION, not by check: every rule lifts
//! a SET-level operator tickwise (plus `delay`/`I`, which only look backward), so a
//! licensed operator cannot peek across ticks. An operator authored directly at the
//! stream level would be outside this license system entirely — the licenses read
//! Z-set specs and mean nothing about stream-level behaviour they never judged.
//!
//! The render emits two locks from ONE derivation (never restated): the generated
//! Rust (`gen/<circuit>_incremental.rs` — compiled and tested like any source; a hand
//! edit is caught by the drift gate) and the plain-language ratification artifact
//! (`spec/<circuit>.circuit.spec` — node by node, the rule applied and the license
//! cited). [`Circuit::batch`] is the reference interpreter the end gate compares
//! against, and [`Circuit::incremental_with`] is the interpreter twin of the generated
//! code — parameterizable by registry, which is exactly the handle the fire drill
//! needs to FORGE a license and watch the end gate fire.

use std::path::Path;

use crate::license::{Classification, License, Registry};
use crate::ops;
use crate::stream::Stream;
use crate::zset::ZSet;

// ===== the operator surface the circuits range over ========================

/// The set-level evaluator and generated-code identifier for every circuit-usable
/// operator — the one table `batch`, the interpreter, and the render all read.
/// `None` = the name is not an operator here (distinct from unlicensed).
fn op_info(name: &str) -> Option<(usize, &'static str)> {
    match name {
        "filter" => Some((1, "filter_even")),
        "map" => Some((1, "project_halved")),
        "sum" => Some((1, "total")),
        "join" => Some((2, "join")),
        "scale" => Some((2, "scale")),
        "distinct" => Some((1, "distinct")),
        "min" => Some((1, "least")),
        _ => None,
    }
}

/// Apply an operator by name at the Z-set level (arity validated at construction).
fn apply(name: &str, args: &[&ZSet]) -> ZSet {
    match (name, args) {
        ("filter", [x]) => ops::filter_even(x),
        ("map", [x]) => ops::project_halved(x),
        ("sum", [x]) => ops::total(x),
        ("join", [a, b]) => ops::join(a, b),
        ("scale", [a, b]) => ops::scale(a, b),
        ("distinct", [x]) => ops::distinct(x),
        ("min", [x]) => ops::least(x),
        _ => unreachable!("arity and name validated at construction"),
    }
}

// ===== the stream-level rule bodies (what the generated code calls) ========

/// Apply a unary Z-set operator tickwise.
pub fn lift1(s: &Stream, f: fn(&ZSet) -> ZSet) -> Stream {
    Stream::of(&s.ticks().iter().map(f).collect::<Vec<_>>())
}

/// Apply a binary Z-set operator tickwise.
pub fn lift2(a: &Stream, b: &Stream, f: fn(&ZSet, &ZSet) -> ZSet) -> Stream {
    Stream::of(
        &a.ticks()
            .iter()
            .zip(b.ticks().iter())
            .map(|(x, y)| f(x, y))
            .collect::<Vec<_>>(),
    )
}

/// LINEAR rule: the operator is its own incremental form — deltas in, deltas out.
pub fn linear1(delta: &Stream, f: fn(&ZSet) -> ZSet) -> Stream {
    lift1(delta, f)
}

/// BILINEAR rule, the three-term delta: with `a = I(Δa)`, `b = I(Δb)` the accumulated
/// states, `Δ(a ⋈ b) = Δa ⋈ delay(b) plus delay(a) ⋈ Δb plus Δa ⋈ Δb`.
pub fn bilinear(da: &Stream, db: &Stream, f: fn(&ZSet, &ZSet) -> ZSet) -> Stream {
    let a_prev = da.integrate().delay();
    let b_prev = db.integrate().delay();
    lift2(da, &b_prev, f)
        .plus(&lift2(&a_prev, db, f))
        .plus(&lift2(da, db, f))
}

/// GENERIC fallback for a unary operator: `Q^Δ = D ∘ Q ∘ I` — integrate the delta
/// history into states, recompute on every tick, differentiate back to deltas.
pub fn fallback1(delta: &Stream, f: fn(&ZSet) -> ZSet) -> Stream {
    lift1(&delta.integrate(), f).differentiate()
}

/// GENERIC fallback for a binary operator.
pub fn fallback2(da: &Stream, db: &Stream, f: fn(&ZSet, &ZSet) -> ZSet) -> Stream {
    lift2(&da.integrate(), &db.integrate(), f).differentiate()
}

// ===== the circuit value object ============================================

/// One node: an operator name (the license registry's key) applied to earlier values.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Node {
    /// The operator, by registry name.
    pub operator: String,
    /// Argument indices into the single value space: `0..sources` are the circuit's
    /// input streams, `sources + k` is node `k`'s output. Validity: strictly less than
    /// this node's own index — acyclic by construction.
    pub inputs: Vec<usize>,
}

/// A validated operator DAG. The last node is the circuit's output.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Circuit {
    /// The circuit's name — keys `gen/<name>_incremental.rs` and
    /// `spec/<name>.circuit.spec`.
    pub name: String,
    /// How many input streams the circuit takes.
    pub sources: usize,
    nodes: Vec<Node>,
}

impl Circuit {
    /// Parse-don't-validate: `Err` names the first fault — an unknown operator, an
    /// UNLICENSED operator (absent from the registry — the registry is the circuit's
    /// vocabulary), a wrong arity, or a forward/self reference (the DAG property).
    pub fn new(
        name: &str,
        sources: usize,
        nodes: Vec<Node>,
        registry: &Registry,
    ) -> Result<Circuit, String> {
        if nodes.is_empty() {
            return Err(format!("circuit `{name}` has no nodes — nothing to derive"));
        }
        for (k, node) in nodes.iter().enumerate() {
            let Some((arity, _)) = op_info(&node.operator) else {
                return Err(format!(
                    "node {k} names `{}` — not an operator this crate lifts",
                    node.operator
                ));
            };
            if registry.get(&node.operator).is_none() {
                return Err(format!(
                    "node {k} names `{}` — unlicensed (no row in the license registry); \
                     an unlicensed operator is unconstructible, not a runtime surprise",
                    node.operator
                ));
            }
            if node.inputs.len() != arity {
                return Err(format!(
                    "node {k} applies `{}` to {} argument(s) — its arity is {arity}",
                    node.operator,
                    node.inputs.len()
                ));
            }
            for &input in &node.inputs {
                if input >= sources + k {
                    return Err(format!(
                        "node {k} references value {input}, which is not before it — \
                         circuits are acyclic by construction"
                    ));
                }
            }
        }
        Ok(Circuit {
            name: name.to_string(),
            sources,
            nodes,
        })
    }

    /// The BATCH reference: evaluate the DAG tickwise — slot `t` of the output is the
    /// plain set-level query applied to slot `t` of every source. This is the `Q` the
    /// end gate holds every incremental form against.
    pub fn batch(&self, sources: &[Stream]) -> Stream {
        assert_eq!(sources.len(), self.sources, "one stream per source");
        let ticks: Vec<ZSet> = (0..crate::stream::DEPTH)
            .map(|t| {
                let mut values: Vec<ZSet> = sources.iter().map(|s| s.at(t)).collect();
                for node in &self.nodes {
                    let args: Vec<&ZSet> = node.inputs.iter().map(|&i| &values[i]).collect();
                    values.push(apply(&node.operator, &args));
                }
                values.last().expect("nodes is non-empty").clone()
            })
            .collect();
        Stream::of(&ticks)
    }

    /// The incremental interpreter — the same derivation the render emits, executed
    /// against an EXPLICIT registry. This parameter is the point: the fire drill hands
    /// it a forged registry (`distinct → linear`) and the end gate must fire. The
    /// honest path (`Registry::derive()`) and the generated code agree with `batch` by
    /// the end law.
    pub fn incremental_with(&self, registry: &Registry, deltas: &[Stream]) -> Stream {
        assert_eq!(deltas.len(), self.sources, "one delta stream per source");
        let mut values: Vec<Stream> = deltas.to_vec();
        for node in &self.nodes {
            let rule = registry
                .get(&node.operator)
                .expect("validated at construction")
                .classification;
            let unary = |f: fn(&ZSet) -> ZSet, values: &[Stream]| -> Stream {
                let x = &values[node.inputs[0]];
                match rule {
                    Classification::Linear => linear1(x, f),
                    // a bilinear license on a unary operator cannot arise from the
                    // catalog (distributivity ranges over binaries); fall back safely.
                    Classification::Bilinear | Classification::Neither => fallback1(x, f),
                }
            };
            let binary = |f: fn(&ZSet, &ZSet) -> ZSet, values: &[Stream]| -> Stream {
                let (a, b) = (&values[node.inputs[0]], &values[node.inputs[1]]);
                match rule {
                    Classification::Linear => lift2(a, b, f),
                    Classification::Bilinear => bilinear(a, b, f),
                    Classification::Neither => fallback2(a, b, f),
                }
            };
            let out = match node.operator.as_str() {
                "filter" => unary(ops::filter_even, &values),
                "map" => unary(ops::project_halved, &values),
                "sum" => unary(ops::total, &values),
                "join" => binary(ops::join, &values),
                "scale" => binary(ops::scale, &values),
                "distinct" => unary(ops::distinct, &values),
                "min" => unary(ops::least, &values),
                _ => unreachable!("validated at construction"),
            };
            values.push(out);
        }
        values.last().expect("nodes is non-empty").clone()
    }
}

// ===== the derivation, once — both renders read it =========================

/// One node's derived rule: what the render emits for it, and why.
struct NodeRule<'a> {
    index: usize,
    node: &'a Node,
    license: &'a License,
    /// The rule's one-line story, in the registry's vocabulary.
    story: &'static str,
    /// The `circuit.rs` helper the generated code calls.
    helper: &'static str,
}

/// Derive every node's rule from the registry — the ONE derivation both renders and
/// the interpreter agree on.
fn derive<'a>(circuit: &'a Circuit, registry: &'a Registry) -> Vec<NodeRule<'a>> {
    circuit
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let license = registry.get(&node.operator).expect("validated");
            let binary = node.inputs.len() == 2;
            let (story, helper) = match license.classification {
                Classification::Linear => (
                    "LINEAR — the operator is its own incremental form (Q^Δ = Q)",
                    if binary { "lift2" } else { "linear1" },
                ),
                Classification::Bilinear => (
                    "BILINEAR — the three-term delta: Δ(a ⋈ b) = Δa ⋈ delay(I(Δb)) \
                     plus delay(I(Δa)) ⋈ Δb plus Δa ⋈ Δb",
                    "bilinear",
                ),
                Classification::Neither => (
                    "NEITHER — generic fallback (Q^Δ = D ∘ Q ∘ I): integrate, recompute, \
                     differentiate; correct always, cheap never",
                    if binary { "fallback2" } else { "fallback1" },
                ),
            };
            NodeRule {
                index,
                node,
                license,
                story,
                helper,
            }
        })
        .collect()
}

/// A value index rendered for prose (`s0` / `n3`).
fn value_name(circuit: &Circuit, index: usize) -> String {
    if index < circuit.sources {
        format!("s{index}")
    } else {
        format!("n{}", index - circuit.sources)
    }
}

/// A value index rendered as a Rust argument expression in the generated fn (source
/// params are `&Stream`, node locals are owned).
fn value_expr(circuit: &Circuit, index: usize) -> String {
    if index < circuit.sources {
        format!("s{index}")
    } else {
        format!("&n{}", index - circuit.sources)
    }
}

/// Render the generated Rust — `gen/<name>_incremental.rs`'s whole content.
pub fn render_incremental(circuit: &Circuit, registry: &Registry) -> String {
    let rules = derive(circuit, registry);
    let params: Vec<String> = (0..circuit.sources)
        .map(|i| format!("s{i}: &Stream"))
        .collect();
    let mut out = format!(
        "//! GENERATED by delta-render's render step from the `{name}` circuit declaration\n\
         //! and the license registry (`spec/licenses.spec`). NEVER hand-edit: the drift gate\n\
         //! re-renders this file inside every `cargo test` and fails on any difference —\n\
         //! regenerate with `cargo run -p delta-render --example freeze` and ratify the diff.\n\
         //!\n\
         //! Every node cites the license that granted its rule; the rule table's trust root\n\
         //! is the DBSP paper (Budiu et al.): linear operators commute with D and I, bilinear\n\
         //! operators admit the three-term delta, and D ∘ Q ∘ I is total.\n\n\
         // not every circuit uses every rule; the imports are the rule table.\n\
         #![allow(unused_imports, clippy::let_and_return)]\n\n\
         use crate::circuit::{{bilinear, fallback1, fallback2, lift2, linear1}};\n\
         use crate::ops::{{distinct, filter_even, join, least, project_halved, scale, total}};\n\
         use crate::stream::Stream;\n\n\
         /// The incremental form of circuit `{name}`: deltas in, deltas out —\n\
         /// `I ∘ {name}_incremental ∘ D = batch` is the end gate's law.\n\
         pub fn {name}_incremental({params}) -> Stream {{\n",
        name = circuit.name,
        params = params.join(", "),
    );
    for rule in &rules {
        let args: Vec<String> = rule
            .node
            .inputs
            .iter()
            .map(|&i| value_expr(circuit, i))
            .collect();
        let arg_names: Vec<String> = rule
            .node
            .inputs
            .iter()
            .map(|&i| value_name(circuit, i))
            .collect();
        out.push_str(&format!(
            "    // node {idx} `{op}({args})` — {story}.\n",
            idx = rule.index,
            op = rule.node.operator,
            args = arg_names.join(", "),
            story = rule.story,
        ));
        if rule.license.citations.is_empty() {
            out.push_str(&format!(
                "    //   no license in {} — every delta recomputes.\n",
                rule.license.spec_file
            ));
        } else {
            for c in &rule.license.citations {
                out.push_str(&format!(
                    "    //   licensed by {}: \"{}\"\n",
                    rule.license.spec_file, c
                ));
            }
        }
        let (_, fn_ident) = op_info(&rule.node.operator).expect("validated");
        out.push_str(&format!(
            "    let n{idx} = {helper}({args}, {fn_ident});\n",
            idx = rule.index,
            helper = rule.helper,
            args = args.join(", "),
        ));
    }
    out.push_str(&format!(
        "    n{}\n}}\n",
        circuit.nodes.len().saturating_sub(1)
    ));
    out
}

/// Render the plain-language ratification artifact — `spec/<name>.circuit.spec`'s
/// whole content: node by node, the rule applied and the license cited.
pub fn render_circuit_spec(circuit: &Circuit, registry: &Registry) -> String {
    let rules = derive(circuit, registry);
    let sources: Vec<String> = (0..circuit.sources).map(|i| format!("s{i}")).collect();
    let mut out = format!(
        "# incremental circuit: {name} — the derivation, node by node, each rule cited to \
         the license that granted it — regenerate via this repo's freeze path and ratify \
         the diff.\n#\n\
         # The licenses come from spec/licenses.spec (themselves derived from the frozen\n\
         # law specs); the generated code is gen/{name}_incremental.rs; the end gate\n\
         # (I ∘ Q^Δ ∘ D = Q over the stream grid) holds regardless of what any license\n\
         # says. A demoted license shows up here as a rule change, named.\n#\n\
         # sources: {sources}\n\
         # output:  {output}\n",
        name = circuit.name,
        sources = sources.join(", "),
        output = value_name(circuit, circuit.sources + circuit.nodes.len() - 1),
    );
    for rule in &rules {
        let arg_names: Vec<String> = rule
            .node
            .inputs
            .iter()
            .map(|&i| value_name(circuit, i))
            .collect();
        out.push_str(&format!(
            "\n- node {idx} `{op}({args})`: {story}.\n",
            idx = rule.index,
            op = rule.node.operator,
            args = arg_names.join(", "),
            story = rule.story,
        ));
        if rule.license.citations.is_empty() {
            out.push_str(&format!(
                "      no license in {} — every delta recomputes\n",
                rule.license.spec_file
            ));
        } else {
            for c in &rule.license.citations {
                out.push_str(&format!(
                    "      licensed by {}: \"{}\"\n",
                    rule.license.spec_file, c
                ));
            }
        }
    }
    out
}

/// The two rendered locks for a circuit: the generated Rust and the ratification
/// artifact — one derivation, two renders, both drift-gated.
pub fn circuit_locks(
    circuit: &Circuit,
    registry: &Registry,
    crate_root: &Path,
) -> [spec_lock::Lock; 2] {
    [
        spec_lock::Lock {
            name: format!("{} incremental (generated Rust)", circuit.name),
            path: crate_root.join(format!("gen/{}_incremental.rs", circuit.name)),
            live: render_incremental(circuit, registry),
        },
        spec_lock::Lock {
            name: format!("{} circuit derivation", circuit.name),
            path: crate_root.join(format!("spec/{}.circuit.spec", circuit.name)),
            live: render_circuit_spec(circuit, registry),
        },
    ]
}

// ===== the demo circuit ====================================================

/// The demo circuit — one of each rule kind on the way through:
/// `sum(distinct(join(map(filter(s0)), s0)))` — two linear nodes, the bilinear join,
/// the generic-fallback distinct, and a linear aggregate on top.
pub fn demo_circuit(registry: &Registry) -> Circuit {
    Circuit::new(
        "demo",
        1,
        vec![
            Node {
                operator: "filter".into(),
                inputs: vec![0],
            },
            Node {
                operator: "map".into(),
                inputs: vec![1],
            },
            Node {
                operator: "join".into(),
                inputs: vec![2, 0],
            },
            Node {
                operator: "distinct".into(),
                inputs: vec![3],
            },
            Node {
                operator: "sum".into(),
                inputs: vec![4],
            },
        ],
        registry,
    )
    .expect("the demo circuit is valid by design")
}

/// The audit circuit — the DAG shapes the demo cannot show: TWO sources, a FAN-OUT
/// (node 0 feeds both node 1 and node 2), and the non-commutative bilinear `scale`
/// (whose license is the distributivity PAIR, both laws discovered):
/// `sum(scale(filter(join(s0, s1)), join(s0, s1)))`.
pub fn audit_circuit(registry: &Registry) -> Circuit {
    Circuit::new(
        "audit",
        2,
        vec![
            Node {
                operator: "join".into(),
                inputs: vec![0, 1],
            },
            Node {
                operator: "filter".into(),
                inputs: vec![2],
            },
            Node {
                operator: "scale".into(),
                inputs: vec![3, 2],
            },
            Node {
                operator: "sum".into(),
                inputs: vec![4],
            },
        ],
        registry,
    )
    .expect("the audit circuit is valid by design")
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::zset::Row;

    fn registry() -> Registry {
        Registry::derive()
    }

    fn z(pairs: &[(u8, i64)]) -> ZSet {
        ZSet::of(
            &pairs
                .iter()
                .map(|(r, w)| (Row::new(*r), *w))
                .collect::<Vec<_>>(),
        )
    }

    /// The validity rule refuses each fault BY NAME: unknown operator, wrong arity,
    /// forward reference, empty circuit — and an operator missing from the registry is
    /// UNCONSTRUCTIBLE (the license registry is the circuit's vocabulary).
    #[test]
    fn invalid_circuits_are_unconstructible_by_name() {
        let r = registry();
        let node = |op: &str, inputs: Vec<usize>| Node {
            operator: op.into(),
            inputs,
        };
        let err = Circuit::new("bad", 1, vec![node("median", vec![0])], &r).unwrap_err();
        assert!(err.contains("not an operator"), "{err}");
        let err = Circuit::new("bad", 1, vec![node("join", vec![0])], &r).unwrap_err();
        assert!(err.contains("arity is 2"), "{err}");
        let err = Circuit::new("bad", 1, vec![node("filter", vec![1])], &r).unwrap_err();
        assert!(err.contains("acyclic"), "{err}");
        let err = Circuit::new("bad", 1, vec![], &r).unwrap_err();
        assert!(err.contains("no nodes"), "{err}");
        // a registry without `min` makes a min-circuit unconstructible.
        let mut gutted = r.clone();
        gutted.licenses.retain(|l| l.operator != "min");
        let err = Circuit::new("bad", 1, vec![node("min", vec![0])], &gutted).unwrap_err();
        assert!(err.contains("unlicensed"), "{err}");
    }

    /// The batch reference computes the plain tickwise query — pinned against a
    /// HAND-computed table (the one absolute referent in the crate: these numbers were
    /// derived on paper, not by running the code). The end law is otherwise RELATIVE —
    /// incremental vs batch share the operator implementations — so this pin plus the
    /// per-operator probes are what anchor it; the declared SQL-emulator slot is the
    /// eventual independent oracle.
    #[test]
    fn batch_is_the_plain_tickwise_query_pinned_by_hand() {
        let r = registry();
        let c = demo_circuit(&r);
        // one tick: rows 0 (even, kept) and 1 (odd, dropped); map folds 0→0; join with
        // the full source multiplies weights; distinct re-weights to 1; sum weighs it.
        // by hand, tick 0: {0:2, 1:3} → filter {0:2} → map {0:2} → join·src {0:4}
        //   → distinct {0:1} → sum 1·(0+1) = {0:1}.
        // by hand, tick 1: {1:5} → filter {} → everything downstream {} → sum {}.
        // by hand, tick 2: {0:1} → filter {0:1} → map {0:1} → join·src {0:1}
        //   → distinct {0:1} → sum {0:1}.
        let history = [z(&[(0, 2), (1, 3)]), z(&[(1, 5)]), z(&[(0, 1)])];
        let out = c.batch(&[Stream::of(&history)]);
        assert_eq!(out.at(0), z(&[(0, 1)]));
        assert_eq!(out.at(1), ZSet::empty());
        assert_eq!(out.at(2), z(&[(0, 1)]));
        // and the audit circuit, one hand-worked tick: s0 {0:2,1:1}, s1 {0:3} →
        // join {0:6} → filter {0:6} → scale by total-weight(join)=6 → {0:36} → sum
        // 36·(0+1) = {0:36}.
        let a = audit_circuit(&r);
        let out = a.batch(&[
            Stream::of(&[z(&[(0, 2), (1, 1)])]),
            Stream::of(&[z(&[(0, 3)])]),
        ]);
        assert_eq!(out.at(0), z(&[(0, 36)]));
    }

    /// The interpreter twin obeys the end law on the honest registry — the same claim
    /// the generated code carries, checked here at the interpreter level so the fire
    /// drill's forged-registry counterpart has a green twin.
    #[test]
    fn the_interpreter_meets_the_end_law_on_the_honest_registry() {
        let r = registry();
        let c = demo_circuit(&r);
        for s in crate::stream::grid() {
            let batch = c.batch(std::slice::from_ref(&s));
            let incremental = c.incremental_with(&r, &[s.differentiate()]).integrate();
            assert_eq!(
                incremental, batch,
                "I(Q^Δ(D(s))) = Q(s) failed on the honest registry (demo)"
            );
        }
        // and the audit circuit — two sources, fan-out, the non-commutative
        // bilinear — over every grid PAIR.
        let a = audit_circuit(&r);
        for s in crate::stream::grid() {
            for t in crate::stream::grid() {
                let batch = a.batch(&[s.clone(), t.clone()]);
                let incremental = a
                    .incremental_with(&r, &[s.differentiate(), t.differentiate()])
                    .integrate();
                assert_eq!(
                    incremental, batch,
                    "I(Q^Δ(D(s))) = Q(s) failed on the honest registry (audit)"
                );
            }
        }
    }

    /// EVERY inventoried operator is drivable through a one-node circuit, and meets the
    /// end law under its honest license — so no dispatch arm (batch's or the
    /// interpreter's) is dead code, `min` and `distinct` included: the operators outside
    /// any committed demo circuit still carry executable proof their fallback works.
    #[test]
    fn every_operator_drives_a_one_node_circuit_through_the_end_law() {
        let r = registry();
        for license in &r.licenses {
            let arity = if matches!(license.operator.as_str(), "join" | "scale") {
                2
            } else {
                1
            };
            let c = Circuit::new(
                &format!("one-{}", license.operator),
                1,
                vec![Node {
                    operator: license.operator.clone(),
                    // a binary gets the one source twice — indices, not copies.
                    inputs: vec![0; arity],
                }],
                &r,
            )
            .expect("a one-node circuit of an inventoried operator is valid");
            for s in crate::stream::grid() {
                let batch = c.batch(std::slice::from_ref(&s));
                let incremental = c.incremental_with(&r, &[s.differentiate()]).integrate();
                assert_eq!(
                    incremental, batch,
                    "the end law failed for one-node `{}`",
                    license.operator
                );
            }
        }
    }

    /// The RENDER drift gates, lib-side: both circuits' committed artifacts re-render
    /// byte for byte. This duplicates the integration gate ON PURPOSE — the mutation
    /// sweeps judge library mutants against LIB tests only, so without this twin a
    /// mutant in the render string-building would survive every sweep while the
    /// integration gate slept.
    #[test]
    fn the_committed_renders_are_fresh_from_the_library_side() {
        let crate_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let r = registry();
        let [a, b] = circuit_locks(&demo_circuit(&r), &r, &crate_root);
        let [c, d] = circuit_locks(&audit_circuit(&r), &r, &crate_root);
        let locks = [a, b, c, d];
        if let Err(stale) = spec_lock::check(&locks) {
            panic!(
                "a rendered circuit artifact drifted: {}. Never hand-edit — run \
                 `cargo run -p delta-render --example freeze` and ratify the diff.",
                stale.join(", ")
            );
        }
    }
}
