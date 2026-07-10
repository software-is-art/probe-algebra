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
//! ONE carrier is [`Lifted`]; TWO carriers (a heterogeneous, multi-SORTED signature — a
//! cross-sort map, a round trip) is [`Lifted2`], the same shape one level up with a tagged
//! [`Either`] value and per-sort grids. [`AutoLift::scan_module`] emits BOTH from a plain
//! module — `impl Liftable` for one carrier, a marker `struct` + `impl Liftable2` for two —
//! so the zero-annotation loop is closed for one and two sorts. Scope, disclosed (the
//! enumerability edge the roadmap names): a non-`Shaped` carrier has no grid and is out of
//! reach, exactly where discovery itself stops; three or more DISTINCT carriers generalise the
//! tag to an N-way enum, which needs per-N codegen (no variadic generics) and is a named
//! refusal in the scan for now — note that N conceptual sorts modelled as ONE `Shaped` enum
//! are already single-carrier `Lifted`. Within that edge the lift is total: the consumer
//! writes types and Rust, the scan writes the table, and `Spec::of` / `MutationReport` over
//! `Lifted<C>` or `Lifted2<T>` are the probes and their sensitivity proof.
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
    /// The DECLARED laws — the zero-annotation world's SHOULD channel. Default empty: a
    /// plain lift starts with no contract, and every declaration is an explicit act
    /// (`AutoLift::scan_module`'s declarations parameter bakes them in, vocabulary-gated
    /// at scan time). With this, `Distance::of::<Lifted<C>>()` answers for a module whose
    /// author wrote only types and Rust — the red target lock reaches the lifted world.
    fn expectations() -> Vec<crate::discover::expect::Expectation> {
        Vec::new()
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

/// The lifted theory's declared laws are the carrier's ([`Liftable::expectations`]) — so
/// `Distance::of::<Lifted<C>>()` runs for a zero-annotation module the moment its author
/// declares anything, and reports "met" trivially before that (an empty contract is
/// honestly met, never a lie about coverage — the spec lock still carries what discovery
/// FOUND).
impl<C: Liftable> crate::discover::expect::Expected for Lifted<C> {
    fn expectations() -> Vec<crate::discover::expect::Expectation> {
        C::expectations()
    }
}

// ---- multi-sort: two carriers ------------------------------------------------------------
//
// The single-carrier lift covers a module over ONE type. A module whose functions range over
// TWO types (a cross-sort map like `is_on: Bit -> bool`, a round-trip `wrap`/`unwrap`) is a
// two-SORTED theory. The generic machinery is the same shape, one level up: a tagged value
// [`Either`] with a [`Duo`] sort, per-sort grids from each carrier's `shadow_grid`, and an
// operator table whose slots carry sorts. `Either` needs only `Clone`/`Eq`/`Ord`/`Hash` —
// NOT `Shaped` — because the grid is supplied per sort, not closed over the pair.

/// A two-sorted lifted value: the left carrier or the right, tagged. The Value AND Obs of a
/// [`Lifted2`] theory.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Either<A, B> {
    /// The left (first) carrier.
    L(A),
    /// The right (second) carrier.
    R(B),
}

/// The two sorts of a [`Lifted2`] theory — which carrier a slot ranges over.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Duo {
    /// The left carrier's sort.
    L,
    /// The right carrier's sort.
    R,
}

/// A two-sorted operator's evaluator — the wrapper over tagged values a scan emits.
pub type Eval2<A, B> = fn(&[Either<A, B>]) -> Option<Either<A, B>>;

/// One two-sorted lifted operator: like [`LiftedOp`] but its slots carry sorts, so a
/// cross-sort map (`Duo::L` in, `Duo::R` out) is expressible. `eval` unwraps each argument to
/// its carrier, calls the consumer's function, and re-tags the result.
pub struct LiftedOp2<A: 'static, B: 'static> {
    /// Human name for prose.
    pub name: &'static str,
    /// Symbol for equations.
    pub symbol: &'static str,
    pub fixity: Fixity,
    /// The sort of each input slot.
    pub inputs: Vec<Duo>,
    /// The sort of the output.
    pub output: Duo,
    /// The wrapper over tagged values.
    pub eval: Eval2<A, B>,
}

/// A module over TWO `Shaped` carriers — it names the theory, its two carrier types, and the
/// operator table. The counterpart of [`Liftable`] one sort up; the scan generates the table
/// and the `Either`-tagged wrappers.
pub trait Liftable2: Sized + 'static {
    /// The left carrier.
    type A: Shaped + Eq + Ord + Hash + Debug + 'static;
    /// The right carrier.
    type B: Shaped + Eq + Ord + Hash + Debug + 'static;
    /// The lifted theory's display name.
    fn theory_name() -> &'static str;
    /// The operator table.
    fn ops() -> Vec<LiftedOp2<Self::A, Self::B>>;
    /// The grid cap for each carrier's `shadow_grid` closure.
    fn grid_cap() -> usize {
        16
    }
}

/// The two-carrier `Theory` over a [`Liftable2`] module: sorts [`Duo`], value [`Either`], each
/// sort's grid the carrier's `shadow_grid`, identity observation. Written once, like
/// [`Lifted`].
pub struct Lifted2<T>(PhantomData<T>);

impl<T: Liftable2> Theory for Lifted2<T> {
    type Sort = Duo;
    type Value = Either<T::A, T::B>;
    type Obs = Either<T::A, T::B>;

    fn name() -> &'static str {
        T::theory_name()
    }

    fn operators() -> Vec<Operator<Self>> {
        T::ops()
            .into_iter()
            .map(|op| Operator {
                name: op.name,
                symbol: op.symbol,
                fixity: op.fixity,
                inputs: op.inputs,
                output: op.output,
                eval: op.eval,
            })
            .collect()
    }

    fn inhabitants(sort: Duo) -> Vec<Either<T::A, T::B>> {
        match sort {
            Duo::L => shadow_grid::<T::A>(T::grid_cap())
                .into_iter()
                .map(Either::L)
                .collect(),
            Duo::R => shadow_grid::<T::B>(T::grid_cap())
                .into_iter()
                .map(Either::R)
                .collect(),
        }
    }

    fn sort_of(v: &Either<T::A, T::B>) -> Duo {
        match v {
            Either::L(_) => Duo::L,
            Either::R(_) => Duo::R,
        }
    }

    fn observe(v: &Either<T::A, T::B>) -> Either<T::A, T::B> {
        v.clone()
    }
}

/// The named carrier type a lifted operator ranges over, canonicalised (whitespace removed,
/// so `Box < bool >` and `Box<bool>` are one carrier). Any path type is a carrier, including a
/// generic like `Box<bool>`; references, tuples, and `impl`/`dyn` are not owned `Shaped`
/// carriers and refuse by name.
fn carrier_of(ty: &syn::Type) -> Result<String, String> {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() => {
            Ok(quote::quote!(#ty).to_string().replace(' ', ""))
        }
        other => Err(format!(
            "a lifted operator ranges over named carriers, got `{}`",
            quote::quote!(#other)
        )),
    }
}

/// A valid type identifier derived from a theory name — the marker a two-sorted lift's
/// `impl Liftable2` hangs on (`"lifted bool/box"` -> `LiftedBoolBox`).
fn to_marker(name: &str) -> String {
    let mut s = String::new();
    for part in name.split(|c: char| !c.is_alphanumeric()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            s.extend(first.to_uppercase());
            s.push_str(chars.as_str());
        }
    }
    if s.is_empty() || s.starts_with(|c: char| c.is_ascii_digit()) {
        s.insert_str(0, "Lift");
    }
    s
}

/// The index of `name` in `carriers`, appending it in first-appearance order if new.
fn index_of(carriers: &mut Vec<String>, name: String) -> usize {
    match carriers.iter().position(|c| *c == name) {
        Some(i) => i,
        None => {
            carriers.push(name);
            carriers.len() - 1
        }
    }
}

/// The fixity a scanned operator renders with — by arity, the mechanical convention.
fn fixity_for(arity: usize) -> &'static str {
    match arity {
        0 => "Nullary",
        1 => "Prefix",
        2 => "Infix",
        _ => "Prefix",
    }
}

/// Render the single-carrier `impl Liftable` — the ops table over one carrier.
fn render_single(
    theory_name: &str,
    carrier: &str,
    ops: &[(String, Vec<usize>, usize)],
    declared: &[(String, Vec<String>)],
) -> String {
    // the eval closures clone grid values without knowing the carrier's Copy-ness —
    // the generated impl carries the allow so a Copy carrier lints clean.
    let mut out = format!(
        "#[allow(clippy::clone_on_copy)]\nimpl ::boundary_spec::discover::lift::Liftable for {carrier} {{\n"
    );
    out.push_str(&format!(
        "    fn theory_name() -> &'static str {{ {theory_name:?} }}\n"
    ));
    out.push_str(
        "    fn ops() -> ::std::vec::Vec<::boundary_spec::discover::lift::LiftedOp<Self>> {\n        ::std::vec![\n",
    );
    for (name, inputs, _output) in ops {
        let arity = inputs.len();
        let fixity = fixity_for(arity);
        let binder = if arity == 0 { "_" } else { "a" };
        let args = (0..arity)
            .map(|i| format!("a[{i}].clone()"))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "            ::boundary_spec::discover::lift::LiftedOp {{ name: {name:?}, symbol: {name:?}, \
             fixity: ::boundary_spec::discover::engine::Fixity::{fixity}, arity: {arity}, \
             eval: |{binder}| ::std::option::Option::Some({name}({args})) }},\n"
        ));
    }
    out.push_str("        ]\n    }\n");
    // the SHOULD channel, baked in: the declarations ride the generated impl (canonical
    // shape names — validated at scan time, so `Expectation::of`'s panic path is dead
    // code by construction).
    if !declared.is_empty() {
        out.push_str(
            "    fn expectations() -> ::std::vec::Vec<::boundary_spec::discover::expect::Expectation> {\n        ::std::vec![\n",
        );
        for (shape, args) in declared {
            let ops = args
                .iter()
                .map(|a| format!("{a:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "            ::boundary_spec::discover::expect::Expectation::of({shape:?}, ::std::vec![{ops}]),\n"
            ));
        }
        out.push_str("        ]\n    }\n");
    }
    out.push_str("}\n");
    out
}

/// Render the two-carrier `struct` + `impl Liftable2` — the ops table over two carriers,
/// each op's slots tagged with their `Duo` sort and its wrapper unwrapping `Either`, calling
/// the consumer function, and re-tagging. `carriers[0]` is the left sort, `carriers[1]` the
/// right.
fn render_two(
    theory_name: &str,
    carriers: &[String],
    ops: &[(String, Vec<usize>, usize)],
) -> String {
    let marker = to_marker(theory_name);
    let duo = |i: usize| if i == 0 { "L" } else { "R" };
    let mut out = format!("pub struct {marker};\n");
    out.push_str(&format!(
        "impl ::boundary_spec::discover::lift::Liftable2 for {marker} {{\n"
    ));
    out.push_str(&format!("    type A = {};\n", carriers[0]));
    out.push_str(&format!("    type B = {};\n", carriers[1]));
    out.push_str(&format!(
        "    fn theory_name() -> &'static str {{ {theory_name:?} }}\n"
    ));
    out.push_str(
        "    fn ops() -> ::std::vec::Vec<::boundary_spec::discover::lift::LiftedOp2<Self::A, Self::B>> {\n        ::std::vec![\n",
    );
    for (name, inputs, output) in ops {
        let arity = inputs.len();
        let fixity = fixity_for(arity);
        let input_sorts = inputs
            .iter()
            .map(|i| format!("::boundary_spec::discover::lift::Duo::{}", duo(*i)))
            .collect::<Vec<_>>()
            .join(", ");
        let out_sort = format!("::boundary_spec::discover::lift::Duo::{}", duo(*output));
        let eval = if arity == 0 {
            format!(
                "|_a| ::std::option::Option::Some(::boundary_spec::discover::lift::Either::{}({name}()))",
                duo(*output)
            )
        } else {
            let tuple = (0..arity)
                .map(|k| format!("&a[{k}]"))
                .collect::<Vec<_>>()
                .join(", ");
            let pat = inputs
                .iter()
                .enumerate()
                .map(|(k, i)| format!("::boundary_spec::discover::lift::Either::{}(x{k})", duo(*i)))
                .collect::<Vec<_>>()
                .join(", ");
            let call = (0..arity)
                .map(|k| format!("x{k}.clone()"))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "|a| match ({tuple}) {{ ({pat}) => ::std::option::Option::Some(::boundary_spec::discover::lift::Either::{}({name}({call}))), _ => ::std::option::Option::None }}",
                duo(*output)
            )
        };
        out.push_str(&format!(
            "            ::boundary_spec::discover::lift::LiftedOp2 {{ name: {name:?}, symbol: {name:?}, \
             fixity: ::boundary_spec::discover::engine::Fixity::{fixity}, inputs: ::std::vec![{input_sorts}], \
             output: {out_sort}, eval: {eval} }},\n"
        ));
    }
    out.push_str("        ]\n    }\n}\n");
    out
}

/// The auto-lift scanner — the build-time half of the lift, namespaced per the no-rats-nest
/// rule (every public callable hangs off a type).
pub struct AutoLift;

impl AutoLift {
    /// SCAN a plain module's source and GENERATE the lift a consumer's `build.rs` includes —
    /// the zero-annotation half. Finds the public functions and the carriers they range over
    /// (in first-appearance order). ONE carrier emits `impl Liftable` (each function a
    /// [`LiftedOp`] with a thin wrapper); TWO carriers emit a marker `struct` + `impl
    /// Liftable2` (each op's slots tagged with their [`Duo`] sort and an `Either`-unwrapping
    /// wrapper); THREE or more is a named refusal (the N-way tag is not built). The emitted
    /// code is `include!`d where the functions are in scope, so its wrappers call them by bare
    /// name. This is the counterpart of the qualify census's build-time scan, pointed at
    /// generating a theory instead of freezing a surface.
    ///
    /// `declarations` is the SHOULD channel (the zero-annotation world's `expects`): each
    /// entry in the declaration grammar (`commutative(and)`) is vocabulary-gated at scan
    /// time — an unratified shape word refuses TEACHING the catalog, exactly like
    /// `Bundle::declare` — and bakes into the generated impl's
    /// [`Liftable::expectations`], so `Distance::of::<Lifted<C>>()` judges the contract on
    /// every test run. Declarations on a TWO-carrier module are a named refusal (the
    /// two-sorted declaration channel is not built).
    pub fn scan_module(
        source: &str,
        theory_name: &str,
        declarations: &[&str],
    ) -> Result<String, String> {
        let mut declared: Vec<(String, Vec<String>)> = Vec::new();
        for text in declarations {
            let (key, args) = super::bundle::parse_declaration(text)
                .map_err(|e| e.replace("bundle declare", "lift scan"))?;
            let canonical = super::expect::Expectation::canonical(&key).ok_or_else(|| {
                format!(
                    "lift scan: `{key}` is not in the ratified catalog (spec/shapes.spec). \
                     Declarable shapes: {}",
                    super::expect::Expectation::vocabulary_keys().join(", ")
                )
            })?;
            declared.push((canonical.to_string(), args));
        }
        let file =
            syn::parse_file(source).map_err(|e| format!("lift scan: unparseable module: {e}"))?;
        let mut carriers: Vec<String> = Vec::new();
        // each op: (name, input carrier indices, output carrier index).
        let mut ops: Vec<(String, Vec<usize>, usize)> = Vec::new();
        for item in &file.items {
            let syn::Item::Fn(f) = item else { continue };
            if !matches!(f.vis, syn::Visibility::Public(_)) {
                continue;
            }
            let mut inputs = Vec::new();
            for arg in &f.sig.inputs {
                let syn::FnArg::Typed(pt) = arg else {
                    return Err(format!(
                        "lift scan: `{}` takes `self` — a lifted operator is a free function",
                        f.sig.ident
                    ));
                };
                let name = carrier_of(&pt.ty)?;
                inputs.push(index_of(&mut carriers, name));
            }
            let output = match &f.sig.output {
                syn::ReturnType::Type(_, t) => {
                    let name = carrier_of(t)?;
                    index_of(&mut carriers, name)
                }
                syn::ReturnType::Default => {
                    return Err(format!(
                        "lift scan: `{}` returns nothing — not a map into a carrier",
                        f.sig.ident
                    ))
                }
            };
            ops.push((f.sig.ident.to_string(), inputs, output));
        }
        match carriers.len() {
            0 => Err("lift scan: no public functions over a carrier".to_string()),
            1 => Ok(render_single(theory_name, &carriers[0], &ops, &declared)),
            2 if declared.is_empty() => Ok(render_two(theory_name, &carriers, &ops)),
            2 => Err(
                "lift scan: declarations on a two-carrier module — the two-sorted \
                 declaration channel is not built; declare on a single-carrier module"
                    .to_string(),
            ),
            n => Err(format!(
                "lift scan: {n} carriers ({}) — the scan lifts one carrier (`Lifted`) or two \
                 (`Lifted2`); three or more is the N-way tag, not built",
                carriers.join(", ")
            )),
        }
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
        // the SHOULD channel: what the module's author declares the algebra must hold —
        // exactly what `scan_module`'s declarations parameter bakes into a generated impl.
        fn expectations() -> Vec<crate::discover::expect::Expectation> {
            vec![
                crate::discover::expect::Expectation::of("commutative", vec!["and"]),
                crate::discover::expect::Expectation::of("identity", vec!["and", "tru"]),
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

    /// THE SHOULD CHANNEL reaches the lifted world (the zero-annotation declaration rung):
    /// the bool carrier declares `commutative(and)` and `identity(and, tru)`, and
    /// `Distance::of::<Lifted<bool>>()` judges the contract MET — the same engine, the
    /// declared laws found. An author who wrote only types and Rust now has a red/green
    /// contract gate.
    #[test]
    fn a_lifted_module_holds_its_declared_contract() {
        let d = crate::discover::expect::Distance::of::<Lifted<bool>>();
        assert_eq!(d.declared, 2);
        assert!(
            d.missing.is_empty(),
            "the declared boolean laws hold: {:?}",
            d.missing.iter().map(|e| e.render()).collect::<Vec<_>>()
        );
    }

    /// The RED half, drilled: a declaration the module does NOT satisfy reads UNMET by
    /// name — the red target lock for the zero-annotation world. `shift` is an involution
    /// (`shift(shift(x)) = x`), so declaring it IDEMPOTENT overshoots, and the distance
    /// names exactly that overshoot instead of a bare test failure.
    #[test]
    fn an_unmet_declaration_reads_red_by_name() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, crate::Shaped)]
        enum Gear {
            Lo,
            Hi,
        }
        fn shift(g: Gear) -> Gear {
            match g {
                Gear::Lo => Gear::Hi,
                Gear::Hi => Gear::Lo,
            }
        }
        impl Liftable for Gear {
            fn theory_name() -> &'static str {
                "gears"
            }
            fn ops() -> Vec<LiftedOp<Gear>> {
                vec![LiftedOp {
                    name: "shift",
                    symbol: "shift",
                    fixity: Fixity::Prefix,
                    arity: 1,
                    eval: |a| Some(shift(a[0])),
                }]
            }
            fn expectations() -> Vec<crate::discover::expect::Expectation> {
                vec![crate::discover::expect::Expectation::of(
                    "idempotent",
                    vec!["shift"],
                )]
            }
        }
        let d = crate::discover::expect::Distance::of::<Lifted<Gear>>();
        assert_eq!(d.missing.len(), 1, "the overshoot is unmet");
        assert!(
            d.missing[0].render().contains("idempotent(shift)"),
            "named, not a bare failure: {}",
            d.missing[0].render()
        );
    }

    /// The scan BAKES DECLARATIONS IN: the generated impl carries `expectations()` under
    /// the canonical shape names (vocabulary-gated at scan time, so the runtime panic path
    /// is dead by construction); an unratified word refuses TEACHING the catalog; and
    /// declarations on a two-carrier module refuse by name.
    #[test]
    fn the_scan_bakes_declarations_in() {
        let generated =
            AutoLift::scan_module(MODULE_SOURCE, "lifted boolean", &["commutative(and)"])
                .expect("scans with a declaration");
        syn::parse_str::<syn::ItemImpl>(&generated).expect("valid Rust");
        assert!(generated.contains("fn expectations()"), "{generated}");
        assert!(
            generated.contains("Expectation::of(\"commutativity\", ::std::vec![\"and\"])"),
            "canonical shape name baked in: {generated}"
        );
        let err = AutoLift::scan_module(MODULE_SOURCE, "x", &["sparkly(and)"]).unwrap_err();
        assert!(err.contains("not in the ratified catalog"), "{err}");
        assert!(err.contains("Declarable shapes:"), "{err}");
        let err = AutoLift::scan_module(
            "pub fn wrap(a: bool) -> Wrap { Wrap }",
            "two",
            &["commutative(wrap)"],
        )
        .unwrap_err();
        assert!(err.contains("two-carrier"), "{err}");
    }

    /// The SCAN closes the zero-annotation loop: `scan_module` reads the plain module source
    /// and generates an `impl Liftable` that (a) parses as valid Rust and (b) describes
    /// exactly the operator table — by name and arity — that the runtime tests above proved.
    /// So the generated table is the proven table; the only thing left to a consumer is the
    /// build.rs `include!`.
    #[test]
    fn the_scan_generates_the_proven_liftable_table() {
        let generated =
            AutoLift::scan_module(MODULE_SOURCE, "lifted boolean", &[]).expect("the module scans");
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

    /// The scan REFUSES a THREE-carrier module by name — one or two carriers lift (`Lifted`,
    /// `Lifted2`); three or more is the N-way tag, disclosed rather than mis-lifted.
    #[test]
    fn the_scan_refuses_three_carriers() {
        let three = "pub fn mix(a: bool, b: u8) -> char { (a as u8 + b) as char }";
        let err = AutoLift::scan_module(three, "mixed", &[]).expect_err("three carriers refuse");
        assert!(err.contains("N-way tag, not built"), "{err}");
    }

    /// The TWO-carrier scan emits the `Liftable2` a consumer includes — the marker struct, the
    /// two carrier types in first-appearance order, and each op's `Duo`-sorted slots with an
    /// `Either`-unwrapping wrapper. It parses as valid Rust and names exactly the module's ops.
    #[test]
    fn the_scan_emits_liftable2_for_two_carriers() {
        let source = r#"
            pub fn both(a: bool, b: bool) -> bool { a && b }
            pub fn wrap(x: bool) -> Box<bool> { Box::new(x) }
            pub fn unwrap(x: Box<bool>) -> bool { *x }
        "#;
        let generated =
            AutoLift::scan_module(source, "lifted bool/box", &[]).expect("two carriers scan");
        syn::parse_str::<syn::File>(&generated).expect("the generated module is valid Rust");
        assert!(generated.contains("Liftable2 for LiftedBoolBox"));
        assert!(generated.contains("type A = bool"));
        assert!(generated.contains("type B = Box<bool>"));
        // the ops, by (name, input sorts, output sort), match the runtime-proven BoolBox table.
        let proven: std::collections::BTreeSet<(String, Vec<Duo>, Duo)> =
            <BoolBox as Liftable2>::ops()
                .iter()
                .map(|o| (o.name.to_string(), o.inputs.clone(), o.output))
                .collect();
        assert_eq!(proven.len(), 3, "both, wrap, unwrap");
        for (name, _inputs, _output) in &proven {
            assert!(
                generated.contains(&format!("name: {name:?}")),
                "emits `{name}`"
            );
        }
        // the cross-sort ops carry the right sort tags: wrap L->R, unwrap R->L.
        assert!(generated.contains("output: ::boundary_spec::discover::lift::Duo::R"));
        assert!(generated.contains("inputs: ::std::vec![::boundary_spec::discover::lift::Duo::R]"));
    }

    // A CONSUMER'S plain code over TWO types — a cross-sort map and a round trip. Ordinary
    // Rust; nothing here knows about probe-algebra.
    fn both(a: bool, b: bool) -> bool {
        a && b
    }
    fn wrap(x: bool) -> Box<bool> {
        Box::new(x)
    }
    #[allow(clippy::boxed_local)] // the Box param is the point — the R carrier is Box<bool>.
    fn unwrap(x: Box<bool>) -> bool {
        *x
    }

    // The two-sorted lift: the table a multi-sort scan generates — each op's slot sorts and a
    // wrapper that unwraps `Either`, calls the consumer function, and re-tags.
    struct BoolBox;
    impl Liftable2 for BoolBox {
        type A = bool;
        type B = Box<bool>;
        fn theory_name() -> &'static str {
            "lifted bool/box"
        }
        fn ops() -> Vec<LiftedOp2<bool, Box<bool>>> {
            vec![
                LiftedOp2 {
                    name: "both",
                    symbol: "both",
                    fixity: Fixity::Infix,
                    inputs: vec![Duo::L, Duo::L],
                    output: Duo::L,
                    eval: |a| match (&a[0], &a[1]) {
                        (Either::L(x), Either::L(y)) => Some(Either::L(both(*x, *y))),
                        _ => None,
                    },
                },
                LiftedOp2 {
                    name: "wrap",
                    symbol: "wrap",
                    fixity: Fixity::Prefix,
                    inputs: vec![Duo::L],
                    output: Duo::R,
                    eval: |a| match &a[0] {
                        Either::L(x) => Some(Either::R(wrap(*x))),
                        _ => None,
                    },
                },
                LiftedOp2 {
                    name: "unwrap",
                    symbol: "unwrap",
                    fixity: Fixity::Prefix,
                    inputs: vec![Duo::R],
                    output: Duo::L,
                    eval: |a| match &a[0] {
                        Either::R(x) => Some(Either::L(unwrap(x.clone()))),
                        _ => None,
                    },
                },
            ]
        }
    }

    /// A TWO-CARRIER module lifts to a two-sorted theory that discovers its algebra —
    /// including the CROSS-SORT round trip `unwrap(wrap(x)) = x` no single-carrier lift can
    /// express — and is sensitivity-swept, all from the plain functions.
    #[test]
    fn a_two_carrier_module_lifts_and_discovers_cross_sort_laws() {
        let spec = Spec::of::<Lifted2<BoolBox>>();
        let laws: Vec<String> = spec.laws.iter().map(|l| l.prose().to_string()).collect();
        assert!(
            laws.len() >= 2,
            "the two-sorted module discovers its algebra: {laws:?}"
        );
        let report = MutationReport::of::<Lifted2<BoolBox>>();
        assert!(
            !report.deaf.is_empty() && report.deaf.iter().any(|(_, killed)| *killed),
            "the lifted two-sorted laws catch a deaf operator"
        );
    }
}
