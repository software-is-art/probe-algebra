/// A tiny deterministic PRNG (xorshift64*): a trial must be a pure function of
/// (fanout, seed), because the drift gate regenerates the corpus at check time to
/// score the committed trials — a corpus that cannot regenerate is not evidence.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // an odd multiplier is invertible mod 2^64, so only seed 0 can land on
        // xorshift's fixed point; nudge it off.
        Rng(seed.wrapping_mul(0x2545_F491_4F6C_DD1D).max(1))
    }

    pub fn draw(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A bounded pick: uniform enough for corpus assembly, never for statistics.
    pub fn pick(&mut self, bound: usize) -> usize {
        (self.draw() % bound as u64) as usize
    }
}

/// An atomic claim and its FOIL — the same claim with one load-bearing parameter
/// flipped. The narrative must entail the claim and must not entail the foil; an
/// entailed foil convicts either the weaver of inventing or the judge of nodding.
/// `kind` is the template index and `param` its drawn number (0 when parameterless)
/// — recorded so emergent composites can be derived without re-parsing prose.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub foil: String,
    pub kind: usize,
    pub param: u64,
}

/// One child exterior: an invented identity plus its atomic claims. The lexeme is
/// invented so no world knowledge can stand in for reading the exterior.
#[derive(Debug, Clone, PartialEq)]
pub struct Child {
    pub lexeme: String,
    pub role: &'static str,
    pub claims: Vec<Claim>,
}

/// A planted cross-child relation, carried by both endpoints' exteriors. Relations
/// are the synthesis load: per-child claims survive fanout (retrieval), relations
/// are what a thin weave drops (synthesis).
#[derive(Debug, Clone, PartialEq)]
pub struct Relation {
    pub id: String,
    pub from: usize,
    pub to: usize,
    pub text: String,
    pub foil: String,
}

/// One generated trial: the fixed corpus a weaver is asked to fold into a single
/// bounded narrative at a given fanout.
#[derive(Debug, Clone, PartialEq)]
pub struct Trial {
    pub fanout: usize,
    pub seed: u64,
    pub system: String,
    pub children: Vec<Child>,
    pub relations: Vec<Relation>,
}

/// The word budget scales WITH fanout, generously, so token pressure never binds —
/// the sweep isolates the concept budget, which is the constant under measurement.
pub fn word_budget(fanout: usize) -> usize {
    80 + 60 * fanout
}

/// Invent a fresh two-syllable lexeme, capitalised, distinct within the trial.
fn invent(rng: &mut Rng, taken: &mut Vec<String>) -> String {
    loop {
        let a = SYLLABLES[rng.pick(SYLLABLES.len())];
        let b = SYLLABLES[rng.pick(SYLLABLES.len())];
        let mut name = String::new();
        let mut head = a.chars();
        if let Some(first) = head.next() {
            name.extend(first.to_uppercase());
        }
        name.push_str(head.as_str());
        name.push_str(b);
        if !taken.contains(&name) {
            taken.push(name.clone());
            return name;
        }
    }
}

const SYLLABLES: &[&str] = &[
    "var", "nex", "tol", "mir", "sax", "quen", "dor", "fel", "gri", "hul", "pam", "ost", "ruv",
    "kel", "zin", "bra",
];

const ROLES: &[&str] = &[
    "ledger",
    "relay",
    "cache",
    "scheduler",
    "registry",
    "sampler",
    "vault",
    "broker",
    "monitor",
    "courier",
    "index",
    "gate",
    "throttle",
    "archive",
    "beacon",
    "warden",
];

const HANDOFFS: &[&str] = &[
    "hands its sealed ledger to",
    "streams its evictions to",
    "signals backpressure to",
    "publishes its sweep verdicts to",
    "delegates overflow to",
    "reports missed heartbeats to",
];

fn shuffled_roles(rng: &mut Rng) -> Vec<&'static str> {
    let mut roles: Vec<&'static str> = ROLES.to_vec();
    for i in (1..roles.len()).rev() {
        roles.swap(i, rng.pick(i + 1));
    }
    roles
}

/// Three distinct claim templates per child, parameters drawn per claim.
fn claims_for(lexeme: &str, child: usize, rng: &mut Rng) -> Vec<Claim> {
    let mut templates: Vec<usize> = (0..6).collect();
    for i in (1..templates.len()).rev() {
        templates.swap(i, rng.pick(i + 1));
    }
    templates
        .into_iter()
        .take(3)
        .enumerate()
        .map(|(j, t)| claim(format!("c{child}.{j}"), t, lexeme, rng))
        .collect()
}

fn claim(id: String, template: usize, lexeme: &str, rng: &mut Rng) -> Claim {
    match template {
        0 => {
            let n = 1 + rng.pick(4);
            Claim {
                id,
                text: format!(
                    "{lexeme} retries a failed handoff at most {n} times before parking it."
                ),
                foil: format!(
                    "{lexeme} retries a failed handoff at most {} times before parking it.",
                    n + 2
                ),
                kind: 0,
                param: n as u64,
            }
        }
        1 => Claim {
            id,
            text: format!("{lexeme} keeps digests of what it carries, never the bodies."),
            foil: format!("{lexeme} keeps the full bodies of what it carries, never digests."),
            kind: 1,
            param: 0,
        },
        2 => {
            let m = 5 + rng.pick(8);
            Claim {
                id,
                text: format!("{lexeme} evicts anything left unclaimed after {m} ticks."),
                foil: format!(
                    "{lexeme} evicts anything left unclaimed after {} ticks.",
                    2 * m + 3
                ),
                kind: 2,
                param: m as u64,
            }
        }
        3 => Claim {
            id,
            text: format!("{lexeme} refuses new writes while its sweep is running."),
            foil: format!("{lexeme} accepts new writes while its sweep is running."),
            kind: 3,
            param: 0,
        },
        4 => Claim {
            id,
            text: format!("{lexeme} answers reads from its newest snapshot only."),
            foil: format!("{lexeme} answers reads from its oldest snapshot only."),
            kind: 4,
            param: 0,
        },
        _ => {
            let q = 2 + rng.pick(4);
            Claim {
                id,
                text: format!("{lexeme} acts only once {q} of its peers countersign."),
                foil: format!(
                    "{lexeme} acts only once {} of its peers countersign.",
                    q + 1
                ),
                kind: 5,
                param: q as u64,
            }
        }
    }
}

/// The ring: child i relates to child i+1, verbs drawn without immediate repeats so
/// no relation's FOIL (its direction reversed) restates another relation's text.
fn ring_relations(children: &[Child], rng: &mut Rng) -> Vec<Relation> {
    let mut verbs: Vec<&'static str> = HANDOFFS.to_vec();
    for i in (1..verbs.len()).rev() {
        verbs.swap(i, rng.pick(i + 1));
    }
    (0..children.len())
        .map(|i| {
            let j = (i + 1) % children.len();
            let verb = verbs[i % verbs.len()];
            let a = &children[i].lexeme;
            let b = &children[j].lexeme;
            Relation {
                id: format!("r{i}"),
                from: i,
                to: j,
                text: format!("{a} {verb} {b}."),
                foil: format!("{b} {verb} {a}."),
            }
        })
        .collect()
}

/// The numeric parameter of a child's claim of `kind`, if it drew one.
fn numeric(child: &Child, kind: usize) -> Option<u64> {
    child
        .claims
        .iter()
        .find(|c| c.kind == kind)
        .map(|c| c.param)
}

/// A planted EMERGENT FACT: true only of two children jointly (or of the ring as
/// a whole), one semantic step past anything a single claim states. A weave shows
/// POSITIVE GAIN by stating these; the foil — the same fact one step wrong —
/// polices invention. Concatenation states none of them: the zero-gain baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct Emergent {
    pub id: String,
    pub text: String,
    pub foil: String,
}

/// Generate the trial for (fanout, seed) — pure, total for fanout >= 2.
pub fn trial(fanout: usize, seed: u64) -> Trial {
    assert!(fanout >= 2, "a weave needs at least two threads");
    let mut rng = Rng::new(seed ^ (fanout as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut taken = Vec::new();
    let system = invent(&mut rng, &mut taken);
    let roles = shuffled_roles(&mut rng);
    let children: Vec<Child> = (0..fanout)
        .map(|i| {
            let lexeme = invent(&mut rng, &mut taken);
            let claims = claims_for(&lexeme, i, &mut rng);
            Child {
                lexeme,
                role: roles[i % roles.len()],
                claims,
            }
        })
        .collect();
    let relations = ring_relations(&children, &mut rng);
    Trial {
        fanout,
        seed,
        system,
        children,
        relations,
    }
}

/// One symmetric composite for an edge whose endpoints share a numeric claim kind:
/// a sum or a minimum — a number written nowhere in any child. Endpoints arrive
/// canonically ordered so the same pair derives the same sentence from either edge.
fn composite(rid: &str, kind: usize, a: &str, pa: u64, b: &str, pb: u64) -> Emergent {
    match kind {
        0 => Emergent {
            id: format!("e{rid}.retry"),
            text: format!(
                "Between them, {a} and {b} give a failed handoff at most {} retries \
                 before both have parked it.",
                pa + pb
            ),
            foil: format!(
                "Between them, {a} and {b} give a failed handoff at most {} retries \
                 before both have parked it.",
                pa.max(pb)
            ),
        },
        2 => Emergent {
            id: format!("e{rid}.dwell"),
            text: format!(
                "No item sits unclaimed in {a} and {b} for more than {} ticks combined.",
                pa + pb
            ),
            foil: format!(
                "No item sits unclaimed in {a} and {b} for more than {} ticks combined.",
                pa.max(pb)
            ),
        },
        _ => Emergent {
            id: format!("e{rid}.quorum"),
            text: format!(
                "Neither {a} nor {b} ever acts with fewer than {} countersignatures.",
                pa.min(pb)
            ),
            foil: format!(
                "Neither {a} nor {b} ever acts with fewer than {} countersignatures.",
                pa.min(pb) + 1
            ),
        },
    }
}

/// Derive the trial's emergent facts: the ring-closure fact (always present), plus
/// composites for every edge whose endpoints share a numeric claim kind. Pure in
/// the trial; duplicates from the fanout-2 double edge collapse by text.
pub fn emergents(t: &Trial) -> Vec<Emergent> {
    let mut out = vec![Emergent {
        id: "e.ring".to_string(),
        text: format!(
            "Following the handoffs of {} from any module returns to that module \
             after exactly {} steps: the {} modules close into a single ring.",
            t.system, t.fanout, t.fanout
        ),
        foil: format!(
            "The handoffs of {} form one open chain: followed from end to end they \
             never return to their starting module.",
            t.system
        ),
    }];
    for r in &t.relations {
        let (a, b) = (&t.children[r.from], &t.children[r.to]);
        let (a, b) = if a.lexeme <= b.lexeme { (a, b) } else { (b, a) };
        for kind in [0usize, 2, 5] {
            let (Some(pa), Some(pb)) = (numeric(a, kind), numeric(b, kind)) else {
                continue;
            };
            let e = composite(&r.id, kind, &a.lexeme, pa, &b.lexeme, pb);
            if out.iter().all(|prior| prior.text != e.text) {
                out.push(e);
            }
        }
    }
    out
}

#[cfg(test)]
mod probes {
    use super::*;

    #[test]
    fn a_trial_is_a_pure_function_of_fanout_and_seed() {
        assert_eq!(trial(4, 7), trial(4, 7));
        assert_ne!(trial(4, 7), trial(4, 8));
        assert_ne!(trial(4, 7), trial(5, 7));
    }

    #[test]
    fn lexemes_never_collide_inside_a_trial() {
        let t = trial(12, 3);
        let mut names: Vec<&str> = t.children.iter().map(|c| c.lexeme.as_str()).collect();
        names.push(t.system.as_str());
        let mut dedup = names.clone();
        dedup.sort_unstable();
        dedup.dedup();
        assert_eq!(dedup.len(), names.len());
    }

    #[test]
    fn every_foil_differs_from_its_claim() {
        let t = trial(6, 11);
        for c in &t.children {
            for cl in &c.claims {
                assert_ne!(cl.text, cl.foil);
            }
        }
    }

    #[test]
    fn no_relation_foil_restates_another_relation_at_any_swept_fanout() {
        for fanout in 2..=12 {
            for seed in 1..=5 {
                let t = trial(fanout, seed);
                for r in &t.relations {
                    assert!(
                        t.relations.iter().all(|other| other.text != r.foil),
                        "fanout {fanout} seed {seed}: foil of {} restates a relation",
                        r.id
                    );
                }
            }
        }
    }

    #[test]
    fn the_ring_covers_every_child() {
        let t = trial(5, 2);
        assert_eq!(t.relations.len(), 5);
        for (i, r) in t.relations.iter().enumerate() {
            assert_eq!(r.from, i);
            assert_eq!(r.to, (i + 1) % 5);
        }
    }

    #[test]
    fn emergents_are_pure_present_and_foiled_at_any_swept_fanout() {
        for fanout in 2..=12 {
            for seed in 1..=5 {
                let t = trial(fanout, seed);
                let es = emergents(&t);
                assert_eq!(es, emergents(&t), "fanout {fanout} seed {seed}: impure");
                assert!(!es.is_empty(), "fanout {fanout} seed {seed}: no emergent");
                for e in &es {
                    assert_ne!(e.text, e.foil, "fanout {fanout} seed {seed}: {}", e.id);
                }
                let mut ids: Vec<&str> = es.iter().map(|e| e.id.as_str()).collect();
                ids.sort_unstable();
                ids.dedup();
                assert_eq!(ids.len(), es.len(), "fanout {fanout} seed {seed}: dup id");
            }
        }
    }

    #[test]
    fn no_emergent_restates_a_claim_or_relation() {
        for fanout in 2..=12 {
            for seed in 1..=5 {
                let t = trial(fanout, seed);
                for e in emergents(&t) {
                    let stated_by_child = t.children.iter().any(|c| {
                        c.claims
                            .iter()
                            .any(|cl| cl.text == e.text || cl.foil == e.text)
                    });
                    let stated_by_relation = t
                        .relations
                        .iter()
                        .any(|r| r.text == e.text || r.foil == e.text);
                    assert!(
                        !stated_by_child && !stated_by_relation,
                        "fanout {fanout} seed {seed}: {} is not a semantic step",
                        e.id
                    );
                }
            }
        }
    }
}
