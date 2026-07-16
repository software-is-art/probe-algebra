use super::zset::ZSet;

/// A node in the operator DAG. Every input names an EARLIER node, so the graph is
/// acyclic by construction and one forward fold is a complete schedule — order in
/// the executor comes only from real dependencies, never from a program counter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Node {
    /// A stream entering from outside; its deltas are fed at `tick`, never computed.
    Source,
    /// Group addition — linear, so the delta passes straight through.
    Add(usize, usize),
    /// Group inverse — linear.
    Neg(usize),
    /// z⁻¹ — linear: the delta stream itself is delayed one tick.
    Delay(usize),
    /// The bilinear operator: its delta obeys the product rule
    /// Δ(a⋈b) = Δa⋈b + a⋈Δb + Δa⋈Δb, paid for by integrating both inputs.
    Join(usize, usize),
    /// The nonlinear operator: D∘distinct∘I — recompute over the integrated
    /// input, memoized in the warehouse by that input's CONTENT.
    Distinct(usize),
    /// The UNINTERPRETED operator: D∘f∘I with the recompute handed to an admitted
    /// tenant the executor never looks inside — `distinct`'s rule generalized,
    /// verdicts warehoused by the integrated input's CONTENT. The second field
    /// names the tenant `admit` returned.
    Opaque(usize, usize),
}

/// A node's standing state: its output-delta history (append-only), how many ticks
/// it has judged, the integrated inputs its rule needs, and its integrated output
/// (which is both `latest`'s answer and the nonlinear rule's previous value).
#[derive(Clone)]
struct Cell<K: Ord> {
    deltas: Vec<ZSet<K>>,
    done: usize,
    left: ZSet<K>,
    right: ZSet<K>,
    out: ZSet<K>,
}

#[crate::mutate]
impl<K: Ord + Clone> Cell<K> {
    /// A cell before any tick: empty history, everything zero.
    fn fresh() -> Cell<K> {
        Cell {
            deltas: Vec::new(),
            done: 0,
            left: ZSet::zero(),
            right: ZSet::zero(),
            out: ZSet::zero(),
        }
    }
}

/// The nonlinear rule's memo: values keyed by the CONTENT of the integrated input
/// they were computed from, never by when or where they were computed. The
/// owes/gates verdict warehouse is this same table in the degenerate case — rustc
/// and the test suite as uninterpreted operators, the tree as the integrated input.
struct Warehouse<K: Ord> {
    memo: std::collections::BTreeMap<Evidence<K>, ZSet<K>>,
    hits: u64,
    misses: u64,
}

#[crate::mutate]
impl<K: Ord + Clone> Warehouse<K> {
    /// An empty warehouse: no standing evidence.
    fn empty() -> Warehouse<K> {
        Warehouse {
            memo: std::collections::BTreeMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// The one move: carry at standing evidence, judge only what has no key.
    fn recall(
        &mut self,
        op: &'static str,
        key: Vec<(K, i64)>,
        judge: impl FnOnce() -> ZSet<K>,
    ) -> ZSet<K> {
        match self.memo.entry((op, key)) {
            std::collections::btree_map::Entry::Occupied(found) => {
                self.hits += 1;
                found.get().clone()
            }
            std::collections::btree_map::Entry::Vacant(missing) => {
                self.misses += 1;
                missing.insert(judge()).clone()
            }
        }
    }
}

/// The executor: an operator DAG over Z-set streams, evaluated by EDUCTION.
/// `tick` only records what arrived; `latest` demands a node, and only the
/// demanded ancestor cone is judged, each node catching up through the recorded
/// ticks by its operator's delta rule. Naive recompute over the same DAG is the
/// oracle this engine is gated against (see `probes`), never the engine itself.
pub struct Circuit<K: Ord> {
    nodes: Vec<Node>,
    cells: Vec<Cell<K>>,
    tenants: Vec<Tenant<K>>,
    warehouse: Warehouse<K>,
    ticks: usize,
    judged: u64,
}

#[crate::mutate]
impl<K: Ord + Clone> Circuit<K> {
    /// An empty circuit: no nodes, no tenants, no ticks, an empty warehouse.
    pub fn new() -> Circuit<K> {
        Circuit {
            nodes: Vec::new(),
            cells: Vec::new(),
            tenants: Vec::new(),
            warehouse: Warehouse::empty(),
            ticks: 0,
            judged: 0,
        }
    }

    /// Declare a node; a refusal wires nothing. Refused: an input that names no
    /// existing node, an opaque node naming no admitted tenant, and any wiring
    /// after the stream has started — the DAG is declared whole, then the stream
    /// runs, so no node ever has a gap in its history that judgment would have to
    /// invent.
    pub fn wire(&mut self, node: Node) -> Option<usize> {
        let reaches = |input: usize| input < self.nodes.len();
        let sound = match node {
            Node::Source => true,
            Node::Neg(a) | Node::Delay(a) | Node::Distinct(a) => reaches(a),
            Node::Add(a, b) | Node::Join(a, b) => reaches(a) && reaches(b),
            Node::Opaque(a, who) => reaches(a) && who < self.tenants.len(),
        };
        if !sound || self.ticks > 0 {
            return None;
        }
        self.nodes.push(node);
        self.cells.push(Cell::fresh());
        Some(self.nodes.len() - 1)
    }

    /// Record one tick: each source's arriving delta (absent sources receive
    /// zero, duplicate mentions sum). Nothing is computed — computation belongs
    /// to eduction. A feed naming a non-source refuses the whole tick.
    pub fn tick(&mut self, feed: &[(usize, ZSet<K>)]) -> Option<usize> {
        if feed
            .iter()
            .any(|(id, _)| self.nodes.get(*id) != Some(&Node::Source))
        {
            return None;
        }
        for (id, node) in self.nodes.iter().enumerate() {
            if *node != Node::Source {
                continue;
            }
            let mut arrived = ZSet::zero();
            for (fed, z) in feed {
                if *fed == id {
                    arrived = arrived.add(z);
                }
            }
            self.cells[id].deltas.push(arrived);
        }
        self.ticks += 1;
        Some(self.ticks - 1)
    }

    /// Eduction's read: demand the node, judging its cone through every recorded
    /// tick, and return its integrated value NOW. A node that was never wired
    /// refuses.
    pub fn latest(&mut self, node: usize) -> Option<ZSet<K>> {
        if node >= self.nodes.len() {
            return None;
        }
        self.pull(node);
        Some(self.cells[node].out.clone())
    }

    /// How many node-ticks judgment has actually run — the eduction probe's
    /// evidence that demand computes the cone and only the cone.
    pub fn judged(&self) -> u64 {
        self.judged
    }

    /// The warehouse's ledger, (hits, misses): content-keyed reuse, observable.
    pub fn recalled(&self) -> (u64, u64) {
        (self.warehouse.hits, self.warehouse.misses)
    }

    /// Mark the demanded node's ancestor cone (one reverse fold over the declared
    /// order — acyclicity makes this complete), then judge the marked nodes in
    /// declaration order, each catching up through the recorded ticks.
    fn pull(&mut self, node: usize) {
        let mut wanted = vec![false; self.nodes.len()];
        wanted[node] = true;
        for id in (0..=node).rev() {
            if !wanted[id] {
                continue;
            }
            match self.nodes[id] {
                Node::Source => {}
                Node::Neg(a) | Node::Delay(a) | Node::Distinct(a) | Node::Opaque(a, _) => {
                    wanted[a] = true
                }
                Node::Add(a, b) | Node::Join(a, b) => {
                    wanted[a] = true;
                    wanted[b] = true;
                }
            }
        }
        for (id, marked) in wanted.iter().enumerate().take(node + 1) {
            if !*marked {
                continue;
            }
            for t in self.cells[id].done..self.ticks {
                self.judge(id, t);
            }
        }
    }

    /// One node, one tick: compute the output delta by the operator's rule,
    /// append it to the history, fold it into the integrated output.
    fn judge(&mut self, id: usize, t: usize) {
        let dout = match self.nodes[id] {
            Node::Source => self.cells[id].deltas[t].clone(),
            Node::Add(a, b) => self.cells[a].deltas[t].add(&self.cells[b].deltas[t]),
            Node::Neg(a) => self.cells[a].deltas[t].neg(),
            Node::Delay(a) => {
                if t == 0 {
                    ZSet::zero()
                } else {
                    self.cells[a].deltas[t - 1].clone()
                }
            }
            Node::Join(a, b) => {
                let da = self.cells[a].deltas[t].clone();
                let db = self.cells[b].deltas[t].clone();
                let right = self.cells[id].right.add(&db);
                let dout = da.join(&right).add(&self.cells[id].left.join(&db));
                self.cells[id].left = self.cells[id].left.add(&da);
                self.cells[id].right = right;
                dout
            }
            Node::Distinct(a) => {
                let held = self.cells[id].left.add(&self.cells[a].deltas[t]);
                let fresh = self
                    .warehouse
                    .recall("distinct", held.entries(), || held.distinct());
                let dout = fresh.add(&self.cells[id].out.neg());
                self.cells[id].left = held;
                dout
            }
            Node::Opaque(a, who) => {
                let held = self.cells[id].left.add(&self.cells[a].deltas[t]);
                let tenant = &mut self.tenants[who];
                let fresh = self
                    .warehouse
                    .recall(tenant.name, held.entries(), || (tenant.judge)(&held));
                let dout = fresh.add(&self.cells[id].out.neg());
                self.cells[id].left = held;
                dout
            }
        };
        if self.nodes[id] != Node::Source {
            self.cells[id].deltas.push(dout.clone());
        }
        self.cells[id].out = self.cells[id].out.add(&dout);
        self.cells[id].done = t + 1;
        self.judged += 1;
    }
}

#[crate::mutate]
impl<K: Ord + Clone> Default for Circuit<K> {
    fn default() -> Circuit<K> {
        Circuit::new()
    }
}

/// A verdict's address in the warehouse: the operator that judged, and the
/// CONTENT it judged — never the node, never the tick.
type Evidence<K> = (&'static str, Vec<(K, i64)>);

/// An admitted opaque operator: the name its verdicts are warehoused under, and
/// the judge the executor calls on a warehouse miss. The name IS the operator's
/// identity — same name, same content, same verdict — which is why `admit`
/// refuses a name already spoken. The judge may be effectful (rustc and the test
/// suite are the intended tenants); the warehouse guarantees it runs at most once
/// per novel content.
struct Tenant<K: Ord> {
    name: &'static str,
    judge: Judge<K>,
}

#[crate::mutate]
impl<K: Ord + Clone> Circuit<K> {
    /// Admit an opaque tenant; a refusal admits nothing. The name is the
    /// operator's identity in the warehouse — same name, same content, same
    /// verdict — so a name already spoken is refused (the interpreted operator
    /// speaks `distinct`), and admission after the stream has started is refused
    /// exactly as late wiring is: the circuit is declared whole, then run.
    pub fn admit(&mut self, name: &'static str, judge: Judge<K>) -> Option<usize> {
        let spoken = name == "distinct" || self.tenants.iter().any(|t| t.name == name);
        if spoken || self.ticks > 0 {
            return None;
        }
        self.tenants.push(Tenant { name, judge });
        Some(self.tenants.len() - 1)
    }

    /// Standing evidence enters before the stream runs: a verdict carried from an
    /// earlier life of the same operator over the same content — the transcript's
    /// carry, spoken in the executor's vocabulary. Refused, carrying nothing: an
    /// operator this circuit does not run, evidence already standing (carried or
    /// judged, the first word holds), and any carry after the stream has started.
    pub fn carry(&mut self, op: &'static str, key: Vec<(K, i64)>, verdict: ZSet<K>) -> Option<()> {
        let runs = op == "distinct" || self.tenants.iter().any(|t| t.name == op);
        if !runs || self.ticks > 0 || self.warehouse.memo.contains_key(&(op, key.clone())) {
            return None;
        }
        self.warehouse.memo.insert((op, key), verdict);
        Some(())
    }
}

/// The judge an opaque tenant runs on a warehouse miss: an uninterpreted, possibly
/// effectful function of the integrated input — rustc and the test suite are the
/// intended occupants. Boxed, because the executor stores what it must not read.
pub type Judge<K> = Box<dyn FnMut(&ZSet<K>) -> ZSet<K>>;

#[cfg(test)]
mod probes {
    use super::*;

    /// A grid tenant: a name and a PURE judge, as a plain function pointer — so
    /// the naive oracle can run the very same judges the circuit admits.
    type GridTenant = (&'static str, fn(&ZSet<u8>) -> ZSet<u8>);

    /// The pure tenants the grid admits: `support` is distinct's own function
    /// under a name of its own (the recognition probe's subject), and `square`
    /// is a genuinely foreign nonlinear operator (self-join).
    const TENANTS: &[GridTenant] = &[("support", support), ("square", square)];

    /// Distinct's function, admitted as an uninterpreted tenant.
    fn support(z: &ZSet<u8>) -> ZSet<u8> {
        z.distinct()
    }

    /// Self-join — nonlinear, and nothing the interpreted operators speak.
    fn square(z: &ZSet<u8>) -> ZSet<u8> {
        z.join(z)
    }

    /// The oracle: naive recompute. Every node's full value at every tick, from
    /// scratch, by the plain kernel operators over integrated inputs — opaque
    /// nodes by running the tenant's own judge — no deltas, no memo, no laziness.
    /// The executor is gated against this; it never runs it.
    fn naive(plan: &[Node], fed: &[Vec<ZSet<u8>>]) -> Vec<Vec<ZSet<u8>>> {
        let mut values: Vec<Vec<ZSet<u8>>> = Vec::new();
        for (t, arrivals) in fed.iter().enumerate() {
            let mut now: Vec<ZSet<u8>> = Vec::new();
            for (id, node) in plan.iter().enumerate() {
                let value = match *node {
                    Node::Source => {
                        let before = if t == 0 {
                            ZSet::zero()
                        } else {
                            values[t - 1][id].clone()
                        };
                        before.add(&arrivals[id])
                    }
                    Node::Add(a, b) => now[a].add(&now[b]),
                    Node::Neg(a) => now[a].neg(),
                    Node::Delay(a) => {
                        if t == 0 {
                            ZSet::zero()
                        } else {
                            values[t - 1][a].clone()
                        }
                    }
                    Node::Join(a, b) => now[a].join(&now[b]),
                    Node::Distinct(a) => now[a].distinct(),
                    Node::Opaque(a, who) => (TENANTS[who].1)(&now[a]),
                };
                now.push(value);
            }
            values.push(now);
        }
        values
    }

    /// A circuit from a declared plan — the grid tenants admitted first, then
    /// every wire lands because the plans list nodes in dependency order.
    fn build(plan: &[Node]) -> Circuit<u8> {
        let mut circuit: Circuit<u8> = Circuit::new();
        for (name, judge) in TENANTS {
            circuit
                .admit(name, Box::new(*judge))
                .expect("the grid tenants' names are fresh");
        }
        for node in plan {
            circuit.wire(*node).expect("declared in dependency order");
        }
        circuit
    }

    /// The delta spread the schedules draw from: zero, insertions, a retraction,
    /// a mixed-sign set — so the grid exercises weights crossing zero, distinct
    /// flipping, and the product rule's cross term.
    fn spread() -> Vec<ZSet<u8>> {
        vec![
            ZSet::zero(),
            ZSet::from_pairs(&[(0, 1)]),
            ZSet::from_pairs(&[(0, -1)]),
            ZSet::from_pairs(&[(1, 2)]),
            ZSet::from_pairs(&[(0, 1), (1, -2)]),
            ZSet::from_pairs(&[(2, 3)]),
        ]
    }

    /// A deterministic feed schedule, [tick][node]: each source walks the spread
    /// on its own stride, non-sources receive zero.
    fn schedule(plan: &[Node], ticks: usize, seed: usize) -> Vec<Vec<ZSet<u8>>> {
        let pool = spread();
        (0..ticks)
            .map(|t| {
                plan.iter()
                    .enumerate()
                    .map(|(id, node)| {
                        if *node == Node::Source {
                            pool[(seed + 2 * t + 3 * id) % pool.len()].clone()
                        } else {
                            ZSet::zero()
                        }
                    })
                    .collect()
            })
            .collect()
    }

    /// One tick's arrivals as a feed: every source's row entry, named.
    fn feed_of(plan: &[Node], arrivals: &[ZSet<u8>]) -> Vec<(usize, ZSet<u8>)> {
        plan.iter()
            .enumerate()
            .filter(|(_, node)| **node == Node::Source)
            .map(|(id, _)| (id, arrivals[id].clone()))
            .collect()
    }

    /// THE ORACLE GATE: over a grid of circuits (every operator — opaque tenants
    /// included — shared fan-out, distinct-of-join, join-of-distincts, delay in
    /// and out of cones, a tenant wired at two nodes) and seeded feed schedules,
    /// the incremental executor equals naive recompute at every node after every
    /// tick — demanded eagerly, and again demanded only once at the end, so
    /// catch-up is judged against the same oracle as step-by-step. Sampled
    /// equivalence, like every drill here: evidence, not proof.
    #[test]
    fn the_incremental_executor_matches_naive_recompute() {
        let plans: Vec<Vec<Node>> = vec![
            vec![
                Node::Source,
                Node::Distinct(0),
                Node::Delay(1),
                Node::Join(1, 2),
                Node::Neg(0),
                Node::Add(3, 4),
            ],
            vec![
                Node::Source,
                Node::Source,
                Node::Join(0, 1),
                Node::Distinct(2),
                Node::Add(3, 0),
                Node::Delay(4),
                Node::Neg(5),
                Node::Add(6, 3),
            ],
            vec![
                Node::Source,
                Node::Source,
                Node::Neg(1),
                Node::Add(0, 2),
                Node::Distinct(3),
                Node::Distinct(0),
                Node::Join(4, 5),
                Node::Delay(6),
                Node::Add(6, 7),
            ],
            vec![
                Node::Source,
                Node::Opaque(0, 0),
                Node::Source,
                Node::Join(0, 2),
                Node::Opaque(3, 1),
                Node::Add(1, 4),
                Node::Opaque(5, 0),
                Node::Delay(6),
                Node::Add(6, 7),
            ],
        ];
        for (which, plan) in plans.iter().enumerate() {
            for seed in 0..spread().len() {
                let fed = schedule(plan, 5, seed);
                let values = naive(plan, &fed);
                let mut eager = build(plan);
                for (t, arrivals) in fed.iter().enumerate() {
                    eager
                        .tick(&feed_of(plan, arrivals))
                        .expect("every feed names a source");
                    for (id, expected) in values[t].iter().enumerate() {
                        assert_eq!(
                            eager.latest(id),
                            Some(expected.clone()),
                            "eager divergence: plan {which}, seed {seed}, tick {t}, node {id}"
                        );
                    }
                }
                let mut lazy = build(plan);
                for arrivals in &fed {
                    lazy.tick(&feed_of(plan, arrivals))
                        .expect("every feed names a source");
                }
                let last = values.last().expect("the schedule has ticks");
                for (id, expected) in last.iter().enumerate() {
                    assert_eq!(
                        lazy.latest(id),
                        Some(expected.clone()),
                        "catch-up divergence: plan {which}, seed {seed}, node {id}"
                    );
                }
            }
        }
    }

    /// Eduction's economy, pinned in node-ticks: a tick judges nothing, a demand
    /// judges exactly its ancestor cone through the recorded ticks, a repeated
    /// demand judges nothing new, and a later demand pays only what is still owed
    /// — after which the caught-up value still matches the oracle.
    #[test]
    fn eduction_judges_the_demanded_cone_and_only_it() {
        let plan = vec![
            Node::Source,
            Node::Distinct(0),
            Node::Source,
            Node::Neg(2),
            Node::Add(1, 3),
        ];
        let mut circuit = build(&plan);
        let a = ZSet::from_pairs(&[(0u8, 1)]);
        let b = ZSet::from_pairs(&[(1u8, 2)]);
        for _ in 0..4 {
            circuit
                .tick(&[(0, a.clone()), (2, b.clone())])
                .expect("both feeds name sources");
        }
        assert_eq!(circuit.judged(), 0, "tick records; it never judges");
        let _ = circuit.latest(1);
        assert_eq!(
            circuit.judged(),
            8,
            "the distinct branch's cone: two nodes, four ticks"
        );
        let _ = circuit.latest(1);
        assert_eq!(
            circuit.judged(),
            8,
            "a second demand with nothing new judges nothing"
        );
        let fed: Vec<Vec<ZSet<u8>>> = (0..4)
            .map(|_| {
                vec![
                    a.clone(),
                    ZSet::zero(),
                    b.clone(),
                    ZSet::zero(),
                    ZSet::zero(),
                ]
            })
            .collect();
        assert_eq!(
            circuit.latest(4),
            Some(naive(&plan, &fed)[3][4].clone()),
            "the sink caught up to the oracle"
        );
        assert_eq!(
            circuit.judged(),
            20,
            "the sink's demand judged only the three nodes still owing"
        );
    }

    /// The warehouse is the nonlinear rule's memo, and content is its only key:
    /// two distinct nodes walking the same integrated content share verdicts node
    /// to node, and a retraction that returns the content to a value already
    /// judged returns BOTH to standing evidence — not the node, not the tick,
    /// the content. The owes/gates verdict table is this same discipline with
    /// the tree as the integrated input.
    #[test]
    fn the_warehouse_is_the_nonlinear_rules_memo() {
        let plan = vec![
            Node::Source,
            Node::Distinct(0),
            Node::Source,
            Node::Distinct(2),
        ];
        let mut circuit = build(&plan);
        let a = ZSet::from_pairs(&[(0u8, 1), (1, -2)]);
        let b = ZSet::from_pairs(&[(2u8, 3)]);
        for delta in [a.clone(), b.clone(), b.neg()] {
            circuit
                .tick(&[(0, delta.clone()), (2, delta)])
                .expect("both feeds name sources");
            let one = circuit.latest(1).expect("wired");
            let two = circuit.latest(3).expect("wired");
            assert_eq!(one, two, "same content, same verdict");
        }
        assert_eq!(
            circuit.recalled(),
            (4, 2),
            "two contents judged once each; every other demand carried"
        );
        assert_eq!(
            circuit.latest(1),
            Some(a.distinct()),
            "the carried verdicts are the computed ones"
        );
    }

    /// The declaration verbs refuse what they cannot wire, and a refusal writes
    /// nothing: inputs must exist, opaque nodes must name an admitted tenant, the
    /// DAG is declared whole before the stream starts, feeds must name sources
    /// (duplicate mentions sum), and a node never wired has no value — while a
    /// wired, unfed source honestly reads zero.
    #[test]
    fn the_circuit_refuses_what_it_cannot_wire() {
        let mut circuit: Circuit<u8> = Circuit::new();
        assert_eq!(
            circuit.wire(Node::Neg(0)),
            None,
            "an input must name an existing node"
        );
        let s = circuit.wire(Node::Source).expect("a source needs nothing");
        assert_eq!(
            circuit.wire(Node::Add(s, 7)),
            None,
            "both inputs must exist"
        );
        assert_eq!(
            circuit.wire(Node::Opaque(s, 0)),
            None,
            "an opaque node must name an admitted tenant"
        );
        assert_eq!(
            circuit.tick(&[(9, ZSet::zero())]),
            None,
            "a feed naming no source refuses the whole tick"
        );
        assert_eq!(
            circuit.tick(&[]),
            Some(0),
            "an empty tick still advances the stream"
        );
        assert_eq!(
            circuit.wire(Node::Neg(s)),
            None,
            "wiring after the stream starts is refused — the DAG is declared whole"
        );
        assert_eq!(circuit.latest(9), None, "a node never wired has no value");
        assert_eq!(
            circuit.latest(s),
            Some(ZSet::zero()),
            "a source that received nothing reads zero"
        );
        let d = ZSet::from_pairs(&[(3u8, 2)]);
        circuit
            .tick(&[(s, d.clone()), (s, d.clone())])
            .expect("a source, twice");
        assert_eq!(
            circuit.latest(s),
            Some(d.add(&d)),
            "duplicate mentions of a source sum"
        );
    }

    /// The recognition, run in both directions: an opaque node whose tenant is
    /// distinct's own function agrees with the interpreted `Distinct` at every
    /// tick and through catch-up — `Distinct` is the Opaque rule with the tenant
    /// interpreted, so interpreting an operator changes what the executor can
    /// SAY about it, never what it computes.
    #[test]
    fn the_opaque_node_is_distinct_uninterpreted() {
        let plan = vec![Node::Source, Node::Distinct(0), Node::Opaque(0, 0)];
        let mut circuit = build(&plan);
        let pool = spread();
        for delta in &pool {
            circuit
                .tick(&[(0, delta.clone())])
                .expect("the feed names the source");
            assert_eq!(
                circuit.latest(1),
                circuit.latest(2),
                "interpreted and uninterpreted distinct diverged"
            );
        }
        let mut lazy = build(&plan);
        for delta in &pool {
            lazy.tick(&[(0, delta.clone())])
                .expect("the feed names the source");
        }
        assert_eq!(
            lazy.latest(2),
            lazy.latest(1),
            "catch-up preserved the recognition"
        );
    }

    /// The tenant's economy, pinned in invocations: an admitted judge runs at
    /// most once per novel content — a second node walking the same content
    /// carries, a divergence pays exactly once, and a retraction that returns
    /// the content to a judged value invokes nothing. This is the property the
    /// sweep's build and baseline tenants buy: rustc runs when the cone's
    /// content is novel, never because a tick happened.
    #[test]
    fn the_tenant_runs_at_most_once_per_novel_content() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0u64));
        let seen = calls.clone();
        let mut circuit: Circuit<u8> = Circuit::new();
        let counted = circuit
            .admit(
                "counted",
                Box::new(move |z| {
                    seen.set(seen.get() + 1);
                    z.distinct()
                }),
            )
            .expect("a fresh name");
        let s1 = circuit.wire(Node::Source).expect("wired");
        let s2 = circuit.wire(Node::Source).expect("wired");
        let o1 = circuit.wire(Node::Opaque(s1, counted)).expect("wired");
        let o2 = circuit.wire(Node::Opaque(s2, counted)).expect("wired");
        let a = ZSet::from_pairs(&[(0u8, 1), (1, -2)]);
        let b = ZSet::from_pairs(&[(2u8, 3)]);
        circuit
            .tick(&[(s1, a.clone()), (s2, a.clone())])
            .expect("both feeds name sources");
        let _ = circuit.latest(o1);
        let _ = circuit.latest(o2);
        assert_eq!(calls.get(), 1, "same content: the second node carried");
        circuit
            .tick(&[(s1, b.clone())])
            .expect("the feed names a source");
        let _ = circuit.latest(o1);
        let _ = circuit.latest(o2);
        assert_eq!(calls.get(), 2, "one novel content: one invocation");
        circuit
            .tick(&[(s1, b.neg())])
            .expect("the feed names a source");
        let _ = circuit.latest(o1);
        let _ = circuit.latest(o2);
        assert_eq!(
            calls.get(),
            2,
            "a retraction returned the content to standing evidence"
        );
        assert_eq!(circuit.recalled(), (4, 2), "the ledger agrees");
    }

    /// Admission and carry are judged transactions: a name already spoken is
    /// refused (the interpreted operator speaks `distinct`), evidence for an
    /// operator the circuit does not run is refused, evidence already standing
    /// is refused, and both doors close when the stream starts. What a carry
    /// admits is TRUSTED like any standing evidence: the demanded value is the
    /// carried verdict, and the tenant is never invoked — which is exactly the
    /// transcript's carry, spoken in the executor's vocabulary.
    #[test]
    fn standing_evidence_enters_before_the_stream() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0u64));
        let seen = calls.clone();
        let mut circuit: Circuit<u8> = Circuit::new();
        assert_eq!(
            circuit.admit("distinct", Box::new(|z| z.distinct())),
            None,
            "the interpreted operator already speaks `distinct`"
        );
        let counted = circuit
            .admit(
                "counted",
                Box::new(move |z| {
                    seen.set(seen.get() + 1);
                    z.distinct()
                }),
            )
            .expect("a fresh name");
        assert_eq!(
            circuit.admit("counted", Box::new(|z| z.clone())),
            None,
            "a spoken name is refused — the name is the operator's identity"
        );
        let s = circuit.wire(Node::Source).expect("wired");
        let o = circuit.wire(Node::Opaque(s, counted)).expect("wired");
        let a = ZSet::from_pairs(&[(0u8, 1)]);
        let fabricated = ZSet::from_pairs(&[(7u8, 1)]);
        assert_eq!(
            circuit.carry("unspoken", a.entries(), fabricated.clone()),
            None,
            "evidence for an operator this circuit does not run is refused"
        );
        circuit
            .carry("counted", a.entries(), fabricated.clone())
            .expect("standing evidence enters before the stream");
        assert_eq!(
            circuit.carry("counted", a.entries(), a.clone()),
            None,
            "evidence already standing is refused — the first word holds"
        );
        circuit
            .tick(&[(s, a.clone())])
            .expect("the feed names the source");
        assert_eq!(
            circuit.admit("late", Box::new(|z| z.clone())),
            None,
            "admission after the stream starts is refused"
        );
        assert_eq!(
            circuit.carry("counted", ZSet::<u8>::zero().entries(), a.clone()),
            None,
            "carry after the stream starts is refused"
        );
        assert_eq!(
            circuit.latest(o),
            Some(fabricated),
            "the carried verdict IS the answer"
        );
        assert_eq!(calls.get(), 0, "the tenant was never invoked");
    }

    /// The deferred tenant gets its integrated input: an opaque node consuming
    /// the ITEM RELATION, fed the way the verbs feed it — a whole module at
    /// weight 1, then an edit as a two-row delta. The tenant's verdicts are
    /// keyed by the tree's CONTENT: an edit that preserves the derived view
    /// still pays (the content is novel — keying is by evidence, not by
    /// answer), and the retraction that returns the tree to a judged state
    /// invokes nothing. Build and baseline-run are this probe at repo scale.
    #[test]
    fn the_deferred_tenant_gets_the_tree() {
        use super::super::items::{Item, ItemRelation};

        fn census(z: &ZSet<Item>) -> ZSet<Item> {
            let mut counts: std::collections::BTreeMap<String, i64> =
                std::collections::BTreeMap::new();
            for (item, weight) in z.entries() {
                *counts.entry(item.module.clone()).or_insert(0) += weight;
            }
            let rows: Vec<(Item, i64)> = counts
                .into_iter()
                .map(|(module, n)| {
                    (
                        Item {
                            module,
                            name: ":count:".to_string(),
                            body: n.to_string(),
                        },
                        1,
                    )
                })
                .collect();
            ZSet::from_pairs(&rows)
        }

        let calls = std::rc::Rc::new(std::cell::Cell::new(0u64));
        let seen = calls.clone();
        let mut circuit: Circuit<Item> = Circuit::new();
        let tenant = circuit
            .admit(
                "census",
                Box::new(move |z| {
                    seen.set(seen.get() + 1);
                    census(z)
                }),
            )
            .expect("a fresh name");
        let tree = circuit.wire(Node::Source).expect("wired");
        let view = circuit.wire(Node::Opaque(tree, tenant)).expect("wired");

        let before = "/// Doubles.\npub fn double(x: i64) -> i64 {\n    x * 2\n}\n";
        let after = "/// Doubles.\npub fn double(x: i64) -> i64 {\n    2 * x\n}\n";
        let a1 = ItemRelation::of_module("a.rs", before).expect("derives");
        let a2 = ItemRelation::of_module("a.rs", after).expect("derives");
        let b = ItemRelation::of_module("b.rs", "struct Inner;\n").expect("derives");

        circuit
            .tick(&[(tree, a1.add(&b))])
            .expect("the feed names the tree");
        let first = circuit.latest(view).expect("wired");
        assert_eq!(
            first,
            census(&a1.add(&b)),
            "the view is the tenant's answer"
        );
        assert_eq!(calls.get(), 1, "novel tree: judged once");

        let edit = a2.add(&a1.neg());
        assert_eq!(edit.entries().len(), 2, "an edit is a two-row delta");
        circuit
            .tick(&[(tree, edit)])
            .expect("the verbs feed deltas");
        assert_eq!(
            circuit.latest(view),
            Some(first.clone()),
            "the derived view is unmoved by this edit"
        );
        assert_eq!(
            calls.get(),
            2,
            "but the content is novel, so the tenant paid — keys are evidence, not answers"
        );

        circuit
            .tick(&[(tree, a1.add(&a2.neg()))])
            .expect("the retraction is a delta like any other");
        assert_eq!(circuit.latest(view), Some(first), "back to the judged tree");
        assert_eq!(
            calls.get(),
            2,
            "a tree already judged is standing evidence — nothing runs"
        );
    }
}
