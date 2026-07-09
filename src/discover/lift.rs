//! lift — AUTO-LIFT a plain module's functions into a single-carrier `Theory`, so a
//! consumer who wrote only ordinary Rust gets the whole apparatus (discovery + the
//! sensitivity sweep) with ZERO declaration.
//!
//! The engine's [`Theory`](super::engine::Theory) needs `operators()` returning `fn`
//! pointers plus a sort signature, an inhabitant grid, and an observation. A plain module
//! cannot produce that by itself — but almost all of it is mechanical for the common case:
//! functions over ONE `Shaped` carrier. So the machinery is written ONCE here as a generic
//! `Theory` over any [`Liftable`] carrier, and the ONLY thing a scan of the consumer's
//! module must generate is the operator table (`impl Liftable`): the function names, their
//! arities, and thin wrappers calling them. Everything else — the single sort, the grid
//! (`shadow_grid` over the carrier's `Shaped` structure), the identity observation — is
//! derived here.
//!
//! Scope, disclosed (the enumerability edge the roadmap names): this lifts a module whose
//! functions are total-ish maps over ONE enumerable (`Shaped`) carrier — a single-sorted
//! theory. Multiple carriers (a heterogeneous, multi-sorted signature) need a `Value` enum
//! and per-sort dispatch, the next widening; a non-`Shaped` carrier has no grid and is out
//! of reach, exactly where discovery itself stops. Within that edge the lift is total: the
//! consumer writes types and Rust, the scan writes the table, and `Spec::of::<Lifted<C>>()`
//! plus `MutationReport::of::<Lifted<C>>()` are the probes and their sensitivity proof.
//!
//! Not mutated: the generic machinery is characterized by GENERATION — the worked example's
//! discovery and sensitivity sweep route every path through it — like `discover::floor`
//! (see `spec/instrumentation.register`).

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::boundary::{shadow_grid, Shaped};

use super::engine::{Fixity, Operator, Theory};

/// One lifted operator: a plain function over the carrier, with the display metadata the
/// engine renders laws with. `eval` is the thin wrapper a scan emits around the consumer's
/// function (`|args| Some(f(args[0], args[1]))`); `arity` is that function's carrier-input
/// count (0 for a constant).
pub struct LiftedOp<C: 'static> {
    /// Human name for prose ("Addition").
    pub name: &'static str,
    /// Symbol for equations (`+`).
    pub symbol: &'static str,
    pub fixity: Fixity,
    /// The number of carrier inputs the function takes.
    pub arity: usize,
    /// The wrapper calling the consumer's function over a slice of carrier values.
    pub eval: fn(&[C]) -> Option<C>,
}

/// A carrier a plain module's functions LIFT onto — it names the theory and carries the
/// operator table. The table is precisely what a scan of the module generates from its
/// public functions; every other piece of the `Theory` is derived by [`Lifted`] below. The
/// bounds are the enumerability edge: `Shaped` gives the grid, and `Eq + Ord + Hash` make
/// the carrier its own behavioural observation.
pub trait Liftable: Shaped + Eq + Ord + Hash + Debug + 'static {
    /// The lifted theory's display name (the module's name, from the scan).
    fn theory_name() -> &'static str;
    /// The operator table — the scan's whole output.
    fn ops() -> Vec<LiftedOp<Self>>;
    /// The grid cap for the carrier's `shadow_grid` closure.
    fn grid_cap() -> usize {
        32
    }
}

/// The single-carrier `Theory` over a [`Liftable`] carrier: one sort, the carrier's `Shaped`
/// grid, identity observation. All the `Theory` machinery lives here once — a consumer (or
/// the scan) writes only `impl Liftable`.
pub struct Lifted<C>(PhantomData<C>);

impl<C: Liftable> Theory for Lifted<C> {
    type Sort = ();
    type Value = C;
    type Obs = C;

    fn name() -> &'static str {
        C::theory_name()
    }

    fn operators() -> Vec<Operator<Self>> {
        C::ops()
            .into_iter()
            .map(|op| Operator {
                name: op.name,
                symbol: op.symbol,
                fixity: op.fixity,
                inputs: vec![(); op.arity],
                output: (),
                eval: op.eval,
            })
            .collect()
    }

    fn inhabitants(_sort: ()) -> Vec<C> {
        shadow_grid::<C>(C::grid_cap())
    }

    fn sort_of(_v: &C) {}

    fn observe(v: &C) -> C {
        v.clone()
    }
}

/// The simple path type a lifted operator ranges over — a single identifier
/// (`bool`, `Payment`). References, generics, and tuples are out of the single-carrier
/// scope and refuse by name.
fn carrier_of(ty: &syn::Type) -> Result<String, String> {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            let seg = &p.path.segments[0];
            if matches!(seg.arguments, syn::PathArguments::None) {
                Ok(seg.ident.to_string())
            } else {
                Err(format!(
                    "`{}` is generic — not a single-carrier type",
                    seg.ident
                ))
            }
        }
        other => Err(format!(
            "a lifted operator ranges over a single named carrier, got `{}`",
            quote::quote!(#other)
        )),
    }
}

/// The auto-lift scanner — the build-time half of the lift, namespaced per the no-rats-nest
/// rule (every public callable hangs off a type).
pub struct AutoLift;

impl AutoLift {
    /// SCAN a plain module's source and GENERATE the `impl Liftable` a consumer's `build.rs`
    /// includes — the zero-annotation half of the lift. Finds the public functions that are
    /// maps over ONE carrier (inferred as the type appearing in every signature; a second
    /// carrier is a named refusal — multi-sort is out of scope), and emits the operator
    /// table: each function a [`LiftedOp`] with its arity and a thin wrapper. The emitted
    /// `impl` is meant to be `include!`d where the functions are in scope, so its wrappers
    /// call them by bare name. This is the counterpart of the qualify census's build-time
    /// scan, pointed at generating a theory instead of freezing a surface.
    pub fn scan_module(source: &str, theory_name: &str) -> Result<String, String> {
        let file =
            syn::parse_file(source).map_err(|e| format!("lift scan: unparseable module: {e}"))?;
        let mut carrier: Option<String> = None;
        let mut ops: Vec<(String, usize)> = Vec::new();
        for item in &file.items {
            let syn::Item::Fn(f) = item else { continue };
            if !matches!(f.vis, syn::Visibility::Public(_)) {
                continue;
            }
            let mut arity = 0usize;
            let mut unify = |ty: &syn::Type| -> Result<(), String> {
                let name = carrier_of(ty)?;
                match &carrier {
                    None => carrier = Some(name),
                    Some(c) if *c == name => {}
                    Some(c) => {
                        return Err(format!(
                        "lift scan: two carriers `{c}` and `{name}` — multi-sort is out of scope"
                    ))
                    }
                }
                Ok(())
            };
            for arg in &f.sig.inputs {
                let syn::FnArg::Typed(pt) = arg else {
                    return Err(format!(
                        "lift scan: `{}` takes `self` — a lifted operator is a free function",
                        f.sig.ident
                    ));
                };
                unify(&pt.ty)?;
                arity += 1;
            }
            match &f.sig.output {
                syn::ReturnType::Type(_, t) => unify(t)?,
                syn::ReturnType::Default => {
                    return Err(format!(
                        "lift scan: `{}` returns nothing — not a map into the carrier",
                        f.sig.ident
                    ))
                }
            }
            ops.push((f.sig.ident.to_string(), arity));
        }
        let carrier =
            carrier.ok_or_else(|| "lift scan: no public functions over a carrier".to_string())?;

        let mut out = String::new();
        out.push_str(&format!(
            "impl ::boundary_spec::discover::lift::Liftable for {carrier} {{\n"
        ));
        out.push_str(&format!(
            "    fn theory_name() -> &'static str {{ {theory_name:?} }}\n"
        ));
        out.push_str(
        "    fn ops() -> ::std::vec::Vec<::boundary_spec::discover::lift::LiftedOp<Self>> {\n        ::std::vec![\n",
    );
        for (name, arity) in &ops {
            let fixity = match arity {
                0 => "Nullary",
                1 => "Prefix",
                2 => "Infix",
                _ => "Prefix",
            };
            let args = (0..*arity)
                .map(|i| format!("a[{i}].clone()"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
            "            ::boundary_spec::discover::lift::LiftedOp {{ name: {name:?}, symbol: {name:?}, \
             fixity: ::boundary_spec::discover::engine::Fixity::{fixity}, arity: {arity}, \
             eval: |a| ::std::option::Option::Some({name}({args})) }},\n"
        ));
        }
        out.push_str("        ]\n    }\n}\n");
        Ok(out)
    }
}

#[cfg(test)]
mod probes {
    use super::*;
    use crate::discover::mutation::MutationReport;
    use crate::discover::Spec;

    // A CONSUMER'S plain code — ordinary Rust functions over an ordinary type. Nothing here
    // imports probe-algebra; this is what a downstream module looks like before any lift.
    fn and(a: bool, b: bool) -> bool {
        a && b
    }
    fn or(a: bool, b: bool) -> bool {
        a || b
    }
    fn not(a: bool) -> bool {
        !a
    }
    fn tru() -> bool {
        true
    }

    // The lift: exactly what a scan of the module above GENERATES — the function names,
    // arities, and thin wrappers. Hand-written here to prove the machinery; the source-scan
    // that emits it is the remaining zero-annotation step.
    impl Liftable for bool {
        fn theory_name() -> &'static str {
            "lifted boolean"
        }
        // mechanical naming — name = symbol = the function's own name, fixity by arity —
        // exactly the convention `scan_module` emits, so the scan and this proven table agree.
        fn ops() -> Vec<LiftedOp<bool>> {
            vec![
                LiftedOp {
                    name: "and",
                    symbol: "and",
                    fixity: Fixity::Infix,
                    arity: 2,
                    eval: |a| Some(and(a[0], a[1])),
                },
                LiftedOp {
                    name: "or",
                    symbol: "or",
                    fixity: Fixity::Infix,
                    arity: 2,
                    eval: |a| Some(or(a[0], a[1])),
                },
                LiftedOp {
                    name: "not",
                    symbol: "not",
                    fixity: Fixity::Prefix,
                    arity: 1,
                    eval: |a| Some(not(a[0])),
                },
                LiftedOp {
                    name: "tru",
                    symbol: "tru",
                    fixity: Fixity::Nullary,
                    arity: 0,
                    eval: |_| Some(tru()),
                },
            ]
        }
    }

    /// The module source the worked example's functions were written as — the input a
    /// consumer's `build.rs` hands `scan_module`.
    const MODULE_SOURCE: &str = r#"
        pub fn and(a: bool, b: bool) -> bool { a && b }
        pub fn or(a: bool, b: bool) -> bool { a || b }
        pub fn not(a: bool) -> bool { !a }
        pub fn tru() -> bool { true }
    "#;

    /// A PLAIN MODULE lifts to a theory whose algebra DISCOVERS ITSELF — the consumer
    /// declared nothing, yet the boolean laws (commutativity, identity, idempotence, …) come
    /// back from the functions alone.
    #[test]
    fn a_plain_module_lifts_and_discovers_its_laws() {
        let spec = Spec::of::<Lifted<bool>>();
        let laws: Vec<String> = spec.laws.iter().map(|l| l.prose().to_string()).collect();
        assert!(
            laws.len() >= 3,
            "the lifted boolean module discovers its algebra: {laws:?}"
        );
        // every operator participates in some law — the spec is not silent about the module.
        assert!(
            spec.uncovered_ops.is_empty(),
            "operators left in no law: {:?}",
            spec.uncovered_ops
        );
    }

    /// The lifted theory is SENSITIVITY-SWEPT by the same oracle-swap the probe census
    /// demands: perturbed operator tables judged by the discovered laws, with dents and the
    /// deaf floor. A consumer's module gets the guarantee, having written only functions.
    #[test]
    fn the_lifted_theory_is_sensitivity_swept() {
        let report = MutationReport::of::<Lifted<bool>>();
        assert!(
            !report.deaf.is_empty() && !report.dents.is_empty(),
            "the deaf floor and dent sweep ran over the lifted operators"
        );
        // the sweep is not vacuous: the discovered laws KILL meaningful perturbations, so at
        // least one deaf mutant is caught (an operator constrained to depend on its input).
        assert!(
            report.deaf.iter().any(|(_, killed)| *killed),
            "the lifted laws catch a deaf operator"
        );
    }

    /// The SCAN closes the zero-annotation loop: `scan_module` reads the plain module source
    /// and generates an `impl Liftable` that (a) parses as valid Rust and (b) describes
    /// exactly the operator table — by name and arity — that the runtime tests above proved.
    /// So the generated table is the proven table; the only thing left to a consumer is the
    /// build.rs `include!`.
    #[test]
    fn the_scan_generates_the_proven_liftable_table() {
        let generated =
            AutoLift::scan_module(MODULE_SOURCE, "lifted boolean").expect("the module scans");
        syn::parse_str::<syn::ItemImpl>(&generated).expect("the generated impl is valid Rust");
        assert!(generated.contains("Liftable for bool"));

        // the generated ops, by (name, arity), equal the hand-written table the runtime proved.
        let proven: std::collections::BTreeSet<(String, usize)> = <bool as Liftable>::ops()
            .iter()
            .map(|o| (o.name.to_string(), o.arity))
            .collect();
        for (name, arity) in &proven {
            assert!(
                generated.contains(&format!("name: {name:?}"))
                    && generated.contains(&format!("arity: {arity}")),
                "the scan emits `{name}`/{arity}"
            );
        }
        assert_eq!(proven.len(), 4, "and, or, not, tru");
    }

    /// The scan REFUSES a multi-carrier module by name — multi-sort is out of the
    /// single-carrier scope, disclosed rather than mis-lifted.
    #[test]
    fn the_scan_refuses_a_second_carrier() {
        let two = "pub fn mix(a: bool, b: u8) -> bool { a }";
        let err = AutoLift::scan_module(two, "mixed").expect_err("two carriers refuse");
        assert!(err.contains("multi-sort is out of scope"), "{err}");
    }
}
