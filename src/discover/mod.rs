//!
//! discover — the laws WRITE (and READ) themselves, generically, over ANY boundary's algebra.
//!
//! The discovery pipeline lives in `engine`: a domain implements `engine::Theory` (its sorts,
//! operators, inhabitants, and an OBSERVATION on values), and the engine ENUMERATES terms over the
//! signature, groups them by behaviour on a grid, instantiates the universal algebraic shapes over
//! the operators, and keeps the ones that run true — rendered as a non-mathy, ratifiable spec. The
//! arithmetic that was hardcoded through v6 is now just one `Theory` (`arithmetic::Arithmetic`),
//! backed by the real interpreter, so its discovered laws still probe the interpreter's interior.
//!
//! On top of the value algebra the interpreter adds one STRUCTURAL law over a synthetic universal
//! observer `U` — the faithful rendering: `eval` collapses structure (`2+3` and `5` are equal to
//! it), so a transform that computes the right value but mangles the tree is invisible to the
//! equations; the U law (`U` distinguishes every structural/semantic perturbation) closes that.
//!
//! Honest frame unchanged: discovery's oracle is the baseline, so its power is deviation-catching
//! (mutation) plus expectation-checking (ratification); enumeration is depth- and grid-bounded (a
//! resource limit, not a curated list).

pub mod agenda;
pub mod architect;
pub mod arithmetic;
pub mod bite;
pub mod bridge;
pub mod bundle;
pub mod coherence;
pub mod cohesion;
pub mod composition;
pub mod date;
pub mod depend;
pub mod derived;
pub mod engine;
pub mod expect;
pub mod fabric;
pub mod floor;
pub mod freeze;
pub mod gates;
pub mod genesis;
pub mod infra;
pub mod judgment;
pub mod layering;
pub mod lift;
pub mod modularize;
pub mod mutation;
pub mod perimeter;
pub mod probes;
pub mod protocol;
pub mod relation;
pub mod residue;
pub mod router;
pub mod scaffold;
pub mod schemata;
pub mod shape;
pub mod substrate;
pub mod system;
pub mod trace;
pub mod verbs;
pub mod watch;
pub mod world;

/// Generate a whole `engine::Theory` impl from a concise declaration — so a discovery domain is
/// "just module definition": the value-object types, the operator functions, and this block. The
/// operator table, `sort_of`, `observe`, `sort_vars`, and `inhabitants` plumbing are all generated;
/// only the actual content (the operator functions and the value objects) is authored.
///
/// ```ignore
/// theory! {
///     Router : "router", Value = Routes, Obs = Vec<Option<u8>>, Sort = Sort,
///     sort_of = |_: &Routes| Sort::Router,
///     observe = |v: &Routes| v.0.to_vec(),
///     vars { Sort::Router => &["a", "b", "c"], }
///     inhabit { Sort::Router => routers(), }
///     ops {
///         Nullary "Empty" "empty" () -> Sort::Router = empty;
///         Infix   "Or"    "or"    (Sort::Router, Sort::Router) -> Sort::Router = or;
///     }
/// }
/// ```
///
/// Every form takes an OPTIONAL trailing `expects { ... }` clause — the theory's DECLARED
/// laws, the top-down half of the loop (see `discover::expect`). Each line is a ratified
/// catalog shape applied to operator symbols (a bare identifier for an identifier-shaped
/// symbol, a string literal for a symbolic one like `"+"`):
///
/// ```ignore
///     expects {
///         commutative(or);
///         associative(or);
///         identity(or, empty);
///     }
/// ```
///
/// The clause generates the `expect::Expected` impl, so `expect::Distance::of::<Router>()`
/// reports the distance between what was declared and what discovery finds. A shape name
/// outside the catalog fails loudly, by name, the first time the expectations are read.
/// No clause, no impl — nothing else changes.
#[macro_export]
macro_rules! theory {
    (
        $thy:ty : $namestr:literal,
        Value = $Value:ty,
        Obs = $Obs:ty,
        Sort = $Sort:ty,
        sort_of = $sortof:expr,
        observe = $observe:expr,
        vars { $( $vpat:pat => $vlist:expr, )+ }
        inhabit { $( $ipat:pat => $ilist:expr, )+ }
        ops {
            $( $fix:ident $opname:literal $opsym:literal ( $($insort:path),* ) -> $outsort:path = $eval:expr; )+
        }
        $( expects {
            $( $eshape:ident ( $($eop:tt),* ); )+
        } )?
    ) => {
        impl $crate::discover::engine::Theory for $thy {
            type Sort = $Sort;
            type Value = $Value;
            type Obs = $Obs;
            fn name() -> &'static str {
                $namestr
            }
            fn operators() -> ::std::vec::Vec<$crate::discover::engine::Operator<Self>> {
                ::std::vec![ $(
                    $crate::discover::engine::Operator {
                        name: $opname,
                        symbol: $opsym,
                        fixity: $crate::discover::engine::Fixity::$fix,
                        inputs: ::std::vec![ $($insort),* ],
                        output: $outsort,
                        eval: $eval,
                    }
                ),+ ]
            }
            fn inhabitants(sort: Self::Sort) -> ::std::vec::Vec<Self::Value> {
                match sort { $( $ipat => $ilist, )+ }
            }
            fn sort_of(value: &Self::Value) -> Self::Sort {
                ($sortof)(value)
            }
            fn observe(value: &Self::Value) -> Self::Obs {
                ($observe)(value)
            }
            fn sort_vars(sort: Self::Sort) -> &'static [&'static str] {
                match sort { $( $vpat => $vlist, )+ }
            }
        }
        $( $crate::__theory_expects! { $thy; $( $eshape ( $($eop),* ); )+ } )?
    };

    // DERIVED-GRID form: no `vars`, no hand-written `inhabit` — the grid is GENERATED from the value
    // type's own structure (its `Shaped` surface — a shadow algebra of synthetic generators the agent
    // never writes and that never enter the spec), and the variable letters fall back to the trait
    // default. So a domain is JUST its value objects and operator functions: the grid the laws are
    // judged on writes itself, fattened by the type structure so a boundary whose operators cannot
    // generate values (a bare monoid, a router) is still judged over a representative set. Requires
    // `Value: Shaped` (any `#[derive(Shaped)]` value object).
    (
        $thy:ty : $namestr:literal,
        Value = $Value:ty,
        Obs = $Obs:ty,
        Sort = $Sort:ty,
        sort_of = $sortof:expr,
        observe = $observe:expr,
        ops {
            $( $fix:ident $opname:literal $opsym:literal ( $($insort:path),* ) -> $outsort:path = $eval:expr; )+
        }
        $( expects {
            $( $eshape:ident ( $($eop:tt),* ); )+
        } )?
    ) => {
        impl $crate::discover::engine::Theory for $thy {
            type Sort = $Sort;
            type Value = $Value;
            type Obs = $Obs;
            fn name() -> &'static str {
                $namestr
            }
            fn operators() -> ::std::vec::Vec<$crate::discover::engine::Operator<Self>> {
                ::std::vec![ $(
                    $crate::discover::engine::Operator {
                        name: $opname,
                        symbol: $opsym,
                        fixity: $crate::discover::engine::Fixity::$fix,
                        inputs: ::std::vec![ $($insort),* ],
                        output: $outsort,
                        eval: $eval,
                    }
                ),+ ]
            }
            fn inhabitants(sort: Self::Sort) -> ::std::vec::Vec<Self::Value> {
                let sort_of = $sortof;
                $crate::discover::engine::shadow_grid::<$Value>(24)
                    .into_iter()
                    .filter(|v| sort_of(v) == sort)
                    .collect()
            }
            fn sort_of(value: &Self::Value) -> Self::Sort {
                ($sortof)(value)
            }
            fn observe(value: &Self::Value) -> Self::Obs {
                ($observe)(value)
            }
        }
        $( $crate::__theory_expects! { $thy; $( $eshape ( $($eop),* ); )+ } )?
    };

    // MINIMAL form: the floor of a discovered domain. No `Obs`, no `observe`, no `vars`, no `inhabit`
    // — for a first-order value object the observation IS the value (`Obs = Value`, observed by
    // identity), the variable letters default, and the grid is shadow-derived from the type. So all
    // that is left to write is the irreducible MEANING: the value type, its sort, and the operators.
    // (Requires `Value: Shaped + Clone + Eq + Ord + Hash` — any first-order `#[derive(Shaped)]` value
    // object. A behavioural observation or a curated grid is a deliberate deviation, written out.)
    (
        $thy:ty : $namestr:literal,
        Value = $Value:ty,
        Sort = $Sort:ty,
        sort_of = $sortof:expr,
        ops {
            $( $fix:ident $opname:literal $opsym:literal ( $($insort:path),* ) -> $outsort:path = $eval:expr; )+
        }
        $( expects { $($etail:tt)+ } )?
    ) => {
        $crate::theory! {
            $thy : $namestr,
            Value = $Value,
            Obs = $Value,
            Sort = $Sort,
            sort_of = $sortof,
            observe = |v: &$Value| ::std::clone::Clone::clone(v),
            ops {
                $( $fix $opname $opsym ( $($insort),* ) -> $outsort = $eval; )+
            }
            $( expects { $($etail)+ } )?
        }
    };
}

/// Generate a whole TYPESTATE PROTOCOL as a theory: states become SORTS, transitions
/// become typed (possibly partial) unary operators, and the grid is the seeds CLOSED
/// under the transitions — so reachability is visible, an illegal transition is
/// UNREPRESENTABLE (no operator carries that signature; there is nothing to test
/// because there is nothing to say), and a rejected transition is DEFINEDNESS (the
/// engine's partial-operator convention judges laws only where the protocol admits
/// values). See `discover::protocol` for the demonstration and the bag-of-booleans
/// contrast this replaces.
///
/// ```ignore
/// protocol! {
///     DocFlow : "doc flow",
///     Sort = DocFlowState, Value = DocFlowValue,       // generated by the macro
///     Payload = Doc, Obs = (String, u8),
///     observe = |d: &Doc| (d.body.clone(), d.rev),
///     states { Draft, Review, Published }
///     seeds { Draft => draft_seeds(), }
///     transitions {
///         submit  : Draft  => Review    = submit;      // fn(&Doc) -> Option<Doc>
///         revise  : Review => Draft     = revise;
///         approve : Review => Published = approve;
///         edit    : Draft  => Draft     = edit;
///     }
///     expects { round_trip(submit, revise); }
/// }
/// ```
///
/// The payload functions never see states — the macro does the state bookkeeping, so a
/// transition's meaning stays a plain `fn(&Payload) -> Option<Payload>` (`None` is the
/// rejection). Inhabitants close over the seeds for as many steps as there are states,
/// so every reachable state is populated and an unreachable state shows up EMPTY — a
/// dead state is a visible fact, not a latent one.
#[macro_export]
macro_rules! protocol {
    (
        $thy:ident : $namestr:literal,
        Sort = $Sort:ident, Value = $Value:ident,
        Payload = $Payload:ty, Obs = $Obs:ty,
        observe = $observe:expr,
        states { $( $state:ident ),+ $(,)? }
        seeds { $( $seed_state:ident => $seed:expr, )+ }
        transitions {
            $( $op:ident : $from:ident => $to:ident = $fn:expr; )+
        }
        $( expects {
            $( $eshape:ident ( $($eop:tt),* ); )+
        } )?
    ) => {
        /// The protocol's states — the SORTS of the generated theory.
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum $Sort {
            $( $state, )+
        }

        /// A payload IN a state — the explicit version of the flag bag: the state is a
        /// variant, so "which state is this value in" has exactly one answer.
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub enum $Value {
            $( $state($Payload), )+
        }

        /// The protocol's marker.
        pub struct $thy;

        impl $thy {
            $(
                fn $op(v: &[$Value]) -> ::std::option::Option<$Value> {
                    match &v[0] {
                        $Value::$from(p) => ($fn)(p).map($Value::$to),
                        _ => ::std::unreachable!("sort-checked by the engine"),
                    }
                }
            )+
        }

        impl $crate::discover::engine::Theory for $thy {
            type Sort = $Sort;
            type Value = $Value;
            type Obs = (&'static str, $Obs);

            fn name() -> &'static str {
                $namestr
            }
            fn operators() -> ::std::vec::Vec<$crate::discover::engine::Operator<Self>> {
                ::std::vec![ $(
                    $crate::discover::engine::Operator {
                        name: ::std::stringify!($op),
                        symbol: ::std::stringify!($op),
                        fixity: $crate::discover::engine::Fixity::Prefix,
                        inputs: ::std::vec![$Sort::$from],
                        output: $Sort::$to,
                        eval: $thy::$op,
                    }
                ),+ ]
            }
            fn inhabitants(sort: $Sort) -> ::std::vec::Vec<$Value> {
                // the seeds, closed under every transition for as many steps as there
                // are states — every reachable state is populated, a dead state is
                // visibly EMPTY, and the bound keeps payload-growing transitions
                // (a revision counter) finite.
                let mut pool: ::std::vec::Vec<$Value> = ::std::vec::Vec::new();
                $( pool.extend($seed.into_iter().map($Value::$seed_state)); )+
                let steps = [$(::std::stringify!($state)),+].len();
                for _ in 0..steps {
                    let mut grown: ::std::vec::Vec<$Value> = ::std::vec::Vec::new();
                    for v in &pool {
                        $(
                            if let $Value::$from(p) = v {
                                if let ::std::option::Option::Some(q) = ($fn)(p) {
                                    let candidate = $Value::$to(q);
                                    if !pool.contains(&candidate) && !grown.contains(&candidate) {
                                        grown.push(candidate);
                                    }
                                }
                            }
                        )+
                    }
                    if grown.is_empty() {
                        break;
                    }
                    pool.extend(grown);
                }
                pool.into_iter()
                    .filter(|v| <Self as $crate::discover::engine::Theory>::sort_of(v) == sort)
                    .collect()
            }
            fn sort_of(value: &$Value) -> $Sort {
                match value {
                    $( $Value::$state(_) => $Sort::$state, )+
                }
            }
            fn observe(value: &$Value) -> (&'static str, $Obs) {
                match value {
                    $( $Value::$state(p) => (::std::stringify!($state), ($observe)(p)), )+
                }
            }
        }

        $( $crate::__theory_expects! { $thy; $( $eshape ( $($eop),* ); )+ } )?
    };
}

/// The `expects { ... }` clause's expansion — one `expect::Expected` impl, each line an
/// `Expectation` (shape key, operator symbols). Split out of `theory!` so all three forms share
/// one expansion. Hidden: only ever invoked by `theory!` itself.
#[doc(hidden)]
#[macro_export]
macro_rules! __theory_expects {
    ( $thy:ty; $( $shape:ident ( $($op:tt),* ); )+ ) => {
        impl $crate::discover::expect::Expected for $thy {
            fn expectations() -> ::std::vec::Vec<$crate::discover::expect::Expectation> {
                ::std::vec![ $(
                    $crate::discover::expect::Expectation::of(
                        ::std::stringify!($shape),
                        ::std::vec![ $( $crate::__expect_op!($op) ),* ],
                    )
                ),+ ]
            }
        }
    };
}

/// One operator symbol inside an `expects` line: a bare identifier stringifies (`grant` →
/// `"grant"`); a string literal passes through, for symbols no identifier can spell (`"+"`,
/// `"-."`). Hidden: only ever invoked by `__theory_expects!`'s expansion.
#[doc(hidden)]
#[macro_export]
macro_rules! __expect_op {
    ( $op:literal ) => {
        $op
    };
    ( $op:ident ) => {
        ::std::stringify!($op)
    };
}

use crate::boundary::sensitive_to_all;
use crate::discover::arithmetic::Arithmetic;
use crate::discover::engine::{Engine, Theory};
use crate::interp::boundary::{Expr, Ident, Op};

/// A discovered law in rendered form — a plain-language sentence and its symbolic equation. This is
/// the readable, ratifiable contract (and what the freeze records).
pub struct Law {
    pub prose: String,
    pub equation: String,
}

impl Law {
    pub fn prose(&self) -> &str {
        &self.prose
    }
    pub fn equation(&self) -> &str {
        &self.equation
    }
}

/// A theory's discovered spec: the named laws, the count of further (consequence) equalities, and
/// the operators that appear in no named law (where the spec is silent).
pub struct Spec {
    pub theory: &'static str,
    pub laws: Vec<Law>,
    pub consequences: usize,
    pub uncovered_ops: Vec<&'static str>,
    /// Candidates in the declared tolerance's UNDECIDED band — neither held nor refuted,
    /// disclosed in the lock. Empty for exact-equality theories.
    pub undecided: Vec<Law>,
    /// The theory's REGISTERED tolerance bars, as display text — rendered into the lock
    /// header so ε is ratified along with the laws. `None` = exact equality.
    pub tolerance: Option<&'static str>,
}

impl Spec {
    /// Discover the named value-algebra laws of any theory, rendered into a `Spec` — the
    /// public constructor, attached per the no-rats-nest rule. A downstream crate implements
    /// `engine::Theory` for its own boundary and calls `Spec::of::<MyTheory>()`; the result
    /// freezes into the consumer's own repo via [`Spec::lock_in`] (see `freeze`).
    pub fn of<T: Theory>() -> Spec {
        let discovered = Engine::<T>::new().discover();
        Spec {
            theory: T::name(),
            laws: discovered
                .laws
                .iter()
                .map(|l| Law {
                    prose: l.prose.clone(),
                    equation: l.equation.clone(),
                })
                .collect(),
            consequences: discovered.consequences,
            uncovered_ops: discovered.uncovered_ops,
            undecided: discovered
                .undecided
                .iter()
                .map(|(prose, equation)| Law {
                    prose: prose.clone(),
                    equation: equation.clone(),
                })
                .collect(),
            tolerance: T::tolerance(),
        }
    }
}

/// The interpreter's discovered laws (named value-algebra laws + the `U` law), for consumers that
/// only need the law list (the example, the freeze).
pub fn discover_laws() -> Vec<Law> {
    interpreter_spec().laws
}

/// The interpreter's discovered spec: the named value-algebra laws (from the generic engine over
/// the `Arithmetic` theory) plus the structural `U` law. The author supplied only the operators;
/// everything here was found by running them, not declared.
pub fn interpreter_spec() -> Spec {
    let mut spec = Spec::of::<Arithmetic>();
    if observer_is_sensitive(render()) {
        spec.laws.push(Law {
            prose: "No two distinct programs look the same — the faithful rendering distinguishes \
                    every structural and semantic difference."
                .to_string(),
            equation: "U(p) = U(q)  ⟹  p = q   (U = faithful render)".to_string(),
        });
    }
    spec
}

/// The router's discovered spec (a non-commutative monoid).
pub fn router_spec() -> Spec {
    Spec::of::<router::Router>()
}

/// The date calculus's discovered spec (a multi-sorted domain with a partial operator).
pub fn date_spec() -> Spec {
    Spec::of::<date::Calendar>()
}

/// This repo's own COMPILED `system!` declaration (see `discover::system`): the four
/// demonstration theories as modules, no seams (the domains share no value objects). The
/// graph IS the registry — [`all_specs`] reads it off `modules()`, so admitting a theory is
/// a reviewed diff HERE, like admitting a kernel file — and the graph itself is frozen in
/// `spec/boundary-spec.system.spec`, drift-gated like every module lock.
pub struct BoundarySpec;

crate::system! {
    BoundarySpec : "boundary-spec",
    modules {
        arithmetic::Arithmetic = interpreter_spec();
        router::Router;
        date::Calendar;
        crate::kvstore::theory::TtlStore;
        world::StoreProtocol;
        protocol::DocFlow;
        fabric::Fabric;
    }
}

/// The TTL store's discovered spec (the first STATEFUL domain: merge monoid, tick action).
pub fn kvstore_spec() -> Spec {
    Spec::of::<crate::kvstore::theory::TtlStore>()
}

/// Every theory's discovered spec — what the freeze records and the staleness gate checks.
/// No longer a hand-maintained list: it is the [`BoundarySpec`] system's module registry.
pub fn all_specs() -> Vec<Spec> {
    <BoundarySpec as system::System>::modules()
}

// ----- the universal observer U: structure + semantics sensitivity --------

fn var(name: &str) -> Expr {
    Expr::var(Ident::new(name).expect("valid identifier"))
}
fn lit(n: i64) -> Expr {
    Expr::int(n).expect("non-negative literal")
}

/// Does the observer `obs` distinguish every structural and semantic perturbation of every sampled
/// program? Parameterized over the observer so the probe is exercised both ways: the faithful
/// rendering passes (it is the synthetic universal observer `U`), a collapsing one (`node_count`)
/// fails — which is what makes the discovered U law load-bearing rather than vacuous.
fn observer_is_sensitive<Y: PartialEq>(obs: impl Fn(&Expr) -> Y + Copy) -> bool {
    sample_programs().iter().all(|e| sensitive_to_all(obs, e))
}

/// The universal observer `U` — the faithful rendering. Named for the call site that gates the law.
fn render() -> impl Fn(&Expr) -> String + Copy {
    |x: &Expr| x.render()
}

/// A deterministic spread of programs that exercises every `Expr` constructor and dimension.
fn sample_programs() -> Vec<Expr> {
    let x = || var("x");
    vec![
        lit(0),
        lit(7),
        Expr::boolean(true),
        x(),
        Expr::bin(Op::Add, lit(2), lit(3)),
        Expr::bin(Op::Mul, x(), lit(4)),
        Expr::bin(Op::Lt, lit(1), x()),
        Expr::cond(Expr::boolean(true), lit(1), lit(2)),
        Expr::bind(
            Ident::new("x").unwrap(),
            lit(5),
            Expr::bin(Op::Add, x(), lit(1)),
        ),
    ]
}

pub mod squash;

pub mod store;

#[cfg(test)]
mod tests {
    use super::*;

    /// The interpreter's WHOLE discovered spec is exactly this — found by the generic engine running
    /// the operators, deterministically. Pins the engine end to end (against the unmutated
    /// interpreter, since this test runs when the engine itself is mutated): a mutation to
    /// enumeration, template matching, rendering, or coverage changes a law and is killed.
    #[test]
    fn the_interpreter_spec_is_exact() {
        let spec = interpreter_spec();
        assert_eq!(spec.theory, "interpreter arithmetic");
        assert_eq!(discover_laws().len(), 11, "law count changed");
        let got: Vec<(String, String)> = spec
            .laws
            .iter()
            .map(|l| (l.prose().to_string(), l.equation().to_string()))
            .collect();
        let expected: Vec<(&str, &str)> = vec![
            (
                "Addition gives the same result in either order.",
                "(x + y) = (y + x)",
            ),
            (
                "With Addition, the grouping of three values doesn't matter.",
                "((x + y) + z) = (x + (y + z))",
            ),
            ("Addition with 0 leaves a value unchanged.", "(0 + x) = x"),
            (
                "Multiplication gives the same result in either order.",
                "(x * y) = (y * x)",
            ),
            (
                "With Multiplication, the grouping of three values doesn't matter.",
                "((x * y) * z) = (x * (y * z))",
            ),
            (
                "Multiplication with 1 leaves a value unchanged.",
                "(1 * x) = x",
            ),
            ("Multiplication by 0 always gives 0.", "(0 * x) = 0"),
            (
                "Multiplication distributes over Addition.",
                "(x * (y + z)) = ((x * y) + (x * z))",
            ),
            ("A value is never less than itself.", "(x < x) = false"),
            // the WITNESS law — the inequation that closes the never-true-relation
            // survivor: a `<` pinned to constant false now contradicts the spec.
            ("less than is not constantly false.", "(x < y) ≠ false"),
            (
                "No two distinct programs look the same — the faithful rendering distinguishes \
                 every structural and semantic difference.",
                "U(p) = U(q)  ⟹  p = q   (U = faithful render)",
            ),
        ];
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(p, e)| (p.to_string(), e.to_string()))
            .collect();
        assert_eq!(got, expected, "the interpreter's discovered spec changed");
        // every operator participates in a law (coverage is complete for arithmetic).
        assert!(
            spec.uncovered_ops.is_empty(),
            "uncovered: {:?}",
            spec.uncovered_ops
        );
        // the count of further (consequence) equalities — pins enumeration depth and dedup.
        assert_eq!(spec.consequences, 332, "consequence count changed");
    }

    /// Enumeration over the (richer) arithmetic theory emits no reflexive `t = t` equality — every
    /// emitted equality relates two DISTINCT terms. Pins the `*c != t` guard in `enumerate`.
    #[test]
    fn arithmetic_enumeration_emits_no_reflexive_equality() {
        let eqs = Engine::<Arithmetic>::new().emitted_equalities();
        assert!(
            eqs.iter().all(|(a, b)| a != b),
            "a reflexive t = t equality was emitted"
        );
    }

    /// The discovered laws actually hold when re-probed against the interpreter, and the U observer
    /// is load-bearing: the faithful render is universally sensitive, a collapsing one
    /// (`node_count`) is not.
    #[test]
    fn the_spec_replays_and_u_is_load_bearing() {
        let engine = Engine::<Arithmetic>::new();
        assert_eq!(engine.check(&engine.discover().laws), Ok(()));
        assert!(observer_is_sensitive(render()), "render should be faithful");
        assert!(
            !observer_is_sensitive(|x: &Expr| x.node_count()),
            "a collapsing observer must fail the sensitivity law"
        );
    }

    /// Every discovered law renders readably — non-empty prose ending in a sentence, and an equation.
    #[test]
    fn discovered_laws_render_readably() {
        for law in discover_laws() {
            assert!(law.prose().ends_with('.') || law.prose().ends_with("difference."));
            assert!(!law.equation().is_empty());
        }
    }
}
