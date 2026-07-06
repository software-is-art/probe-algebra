//! Tier: ALGEBRA — a discovered-law / report layer (exempt from the inward rule).
//!
//! system — the COMPILED `system!` graph layer: an application's spec as a graph of algebras.
//!
//! A large application's spec is not a big law list; it is a GRAPH: nodes are module specs
//! (each frozen in its own lock), edges are SEAM OBLIGATIONS. Exactly two edge kinds exist,
//! and both already have checkers:
//!
//!   - **transport** — two modules share a value and must AGREE on its laws. Where the two
//!     theories' signatures align, the obligation is discharged by re-running each side's
//!     discovered laws under the other's operators (`coherence::CoherenceReport::between`);
//!     where the shared value is ONE type used by both sides, it is discharged BY
//!     CONSTRUCTION, and the declaration carries a compile-time witness (`fn(V) -> V`) so the
//!     discharge is machinery, not a claim.
//!   - **transform** — a conversion crosses the seam and must be a HOMOMORPHISM. The
//!     obligation is discharged inside a SPANNING theory (one whose operator table contains
//!     both sides' operators and the conversion): discovery must find the homomorphism law,
//!     and composites across chained seams are checked by RUNNING (`composition::PipelineLaw`),
//!     never assumed — so verification scales with modules + seams, not their product.
//!
//! `genesis` already PARSES this grammar from tokens for the blank-slate path; this module
//! makes the same declaration COMPILE in a finished codebase. The [`crate::system!`] macro is
//! the compiled twin: a [`System`] marker whose `modules()` IS the spec registry (the graph
//! replaces any hand-maintained `all_specs()`-style list — ratified in the diff like the
//! kernel allowlist) and whose `seams()` wire straight to the checkers above.
//!
//! The graph itself is a committed, drift-gated artifact: [`SystemReport::lock_in`] renders
//! modules, seams, each seam's obligation and status to `spec/<system>.system.spec`, under the
//! same `spec-lock` discipline as every module lock. Hierarchical ratification falls out:
//! an interior change touches no lock, a module law change touches one module lock, and only
//! a re-drawn seam (or a seam whose verdict flips) touches the system lock — review attention
//! scales with blast radius.

use std::path::{Path, PathBuf};

use spec_lock::Lock;

use super::coherence::CoherenceReport;
use super::cohesion::CohesionReport;
use super::composition::PipelineLaw;
use super::engine::{Engine, Theory};
use super::Spec;

pub use super::cohesion::SeamKind;

/// A system: a declared graph of module specs joined by seam obligations. Implemented by the
/// [`crate::system!`] macro — the marker type is the whole application's spec surface.
pub trait System {
    /// The system's display name (also the stem of its lock file).
    fn name() -> &'static str;
    /// The module registry — each member theory's discovered spec, in declaration order.
    /// This IS the registry: what the freeze records and the staleness gate checks; adding a
    /// module is a reviewed diff to the declaration, like admitting a kernel file.
    fn modules() -> Vec<Spec>;
    /// The declared seams, each already put to its checker (the verdicts are computed, never
    /// transcribed).
    fn seams() -> Vec<SeamReport>;
    /// Each registry module's cohesion analysis — the LATENT modularity, for
    /// [`SystemDistance`] to hold against the declared graph. REQUIRED (no default): the
    /// `system!` macro is the implementing path, and it always generates this — a default
    /// body would be dead code wearing a trait's clothes (and an equivalent-mutant site).
    fn cohesions() -> Vec<CohesionReport>;
}

/// A seam obligation's verdict — what the checker actually returned, never a claim.
#[derive(Debug)]
pub enum SeamStatus {
    /// Transport, signatures aligned, checked both directions: every law either module
    /// discovers holds under the other's operators.
    Coherent,
    /// Transport, checked, and the modules DISAGREE about the shared algebra — connectable
    /// but incoherent (the bug class the type system cannot see). The rendered violations.
    Incoherent(Vec<String>),
    /// The seam's question is ILL-POSED and is reported, never judged: a transport whose
    /// operator tables do not align index-for-index (see `CoherenceReport::between`), or a
    /// transform naming a conversion the spanning theory does not declare.
    IllPosed(String),
    /// Transport declared `by construction`: the shared value is ONE type on both sides, and
    /// the declaration carries the compile-time witness (`fn(V) -> V`) that pins it.
    ByConstruction,
    /// Transform: the seam's named conversion crosses as a discovered homomorphism inside
    /// the spanning theory `via`; `composites` are the end-to-end pipeline laws THROUGH that
    /// conversion, checked by running (`PipelineLaw`) — possibly empty when nothing chains.
    Preserved {
        conversion: &'static str,
        via: &'static str,
        homomorphisms: Vec<String>,
        composites: Vec<String>,
    },
    /// Transform: discovery found NO homomorphism law for the seam's named conversion in the
    /// spanning theory `via` — the conversion does not (yet) preserve the algebra.
    Unearned {
        conversion: &'static str,
        via: &'static str,
    },
}

impl SeamStatus {
    /// Is the obligation discharged? `Incoherent`, `IllPosed`, and `Unearned` are open.
    pub fn is_met(&self) -> bool {
        matches!(
            self,
            SeamStatus::Coherent | SeamStatus::ByConstruction | SeamStatus::Preserved { .. }
        )
    }
}

/// One seam of the graph: the two modules it joins, the value it is `on`, its kind, and the
/// verdict its checker returned.
#[derive(Debug)]
pub struct SeamReport {
    /// The joined modules' theory names.
    pub left: &'static str,
    pub right: &'static str,
    /// The declared shared value's name (report vocabulary — the checkers work on the
    /// theories themselves).
    pub on: &'static str,
    pub kind: SeamKind,
    pub status: SeamStatus,
}

impl SeamReport {
    /// A TRANSPORT seam between two same-signature theories, discharged by the coherence
    /// check: re-run each side's discovered laws under the other's operators, both ways.
    /// The type bounds are the alignment precondition — a seam between differently-shaped
    /// theories does not compile under this form (declare it `by construction` when the
    /// shared value is literally one type, or restructure).
    pub fn transport<A, B>(on: &'static str) -> SeamReport
    where
        A: Theory,
        B: Theory<Sort = A::Sort, Value = A::Value, Obs = A::Obs>,
    {
        let status = match CoherenceReport::between::<A, B>() {
            Ok(report) if report.violations.is_empty() => SeamStatus::Coherent,
            Ok(report) => SeamStatus::Incoherent(report.violations),
            Err(why) => SeamStatus::IllPosed(why),
        };
        SeamReport {
            left: A::name(),
            right: B::name(),
            on,
            kind: SeamKind::Transport,
            status,
        }
    }

    /// A TRANSPORT seam whose shared value is ONE type used by both modules — the laws cross
    /// unchanged because there is nothing to translate. The `system!` macro emits the
    /// compile-time witness (`fn(V) -> V`) next to this call, so the by-construction claim
    /// stops compiling the day the sides diverge into two types.
    pub fn transport_by_construction<A: Theory, B: Theory>(on: &'static str) -> SeamReport {
        SeamReport {
            left: A::name(),
            right: B::name(),
            on,
            kind: SeamKind::Transport,
            status: SeamStatus::ByConstruction,
        }
    }

    /// A TRANSFORM seam: the named `conversion` crosses it, discharged inside the SPANNING
    /// theory `Via` (whose operator table carries both sides and the conversion). Discovery
    /// must find the homomorphism law for THAT conversion — `h(x ⊕ y) = h(x) ⊗ h(y)` — and
    /// every chained composite through it is verified by running (`PipelineLaw::discover`),
    /// never assumed. A conversion `Via` does not declare is ILL-POSED, not unearned — a
    /// misspelling must never read as unfinished work.
    pub fn transform<A: Theory, B: Theory, Via: Theory>(
        on: &'static str,
        conversion: &'static str,
    ) -> SeamReport {
        let seam = |status| SeamReport {
            left: A::name(),
            right: B::name(),
            on,
            kind: SeamKind::Transform,
            status,
        };
        let engine = Engine::<Via>::new();
        let symbols: Vec<&'static str> = engine
            .signatures()
            .into_iter()
            .map(|(symbol, _, _)| symbol)
            .collect();
        if !symbols.contains(&conversion) {
            return seam(SeamStatus::IllPosed(format!(
                "`{conversion}` is not an operator of the spanning theory ({})",
                Via::name()
            )));
        }
        let homomorphisms: Vec<String> = engine
            .discover()
            .laws
            .iter()
            .filter(|law| law.shape == "homomorphism" && law.ops(&symbols).contains(&conversion))
            .map(|law| law.prose.clone())
            .collect();
        let composites: Vec<String> = PipelineLaw::discover::<Via>()
            .into_iter()
            .filter(|law| law.via.contains(&conversion))
            .map(|law| law.equation)
            .collect();
        seam(if homomorphisms.is_empty() {
            SeamStatus::Unearned {
                conversion,
                via: Via::name(),
            }
        } else {
            SeamStatus::Preserved {
                conversion,
                via: Via::name(),
                homomorphisms,
                composites,
            }
        })
    }
}

/// The rendered graph of a [`System`] — the application-level spec as a value object: the
/// module registry and every seam's obligation and verdict. What the system lock freezes.
pub struct SystemReport {
    /// The system's display name.
    pub system: &'static str,
    /// The registered modules' theory names, in declaration order.
    pub modules: Vec<&'static str>,
    /// The seams, with computed verdicts.
    pub seams: Vec<SeamReport>,
}

impl SystemReport {
    /// Compute a system's report: read the registry off `modules()` and put every declared
    /// seam to its checker. The analysis is an associated function of its REPORT — the public
    /// surface is the value object, not a loose function (the no-rats-nest rule).
    pub fn of<S: System>() -> SystemReport {
        SystemReport {
            system: S::name(),
            modules: S::modules().iter().map(|spec| spec.theory).collect(),
            seams: S::seams(),
        }
    }

    /// Is the whole graph green — every seam obligation discharged? (Module-level distance
    /// and lock freshness are the modules' own gates; this is the system-level axis.)
    pub fn is_met(&self) -> bool {
        self.seams.iter().all(|seam| seam.status.is_met())
    }

    /// The canonical text of the graph — deterministic, human-readable, diffable: the module
    /// registry, then each seam with its obligation and status. Only DOMAIN facts are
    /// rendered (which modules exist, what each seam obliges, what the checkers returned) —
    /// no engine counters, per the lock principle in `discover::freeze`.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "# system spec: {} — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.\n\n",
            self.system
        ));
        out.push_str("modules (the ratified registry — one committed module lock each):\n");
        for module in &self.modules {
            out.push_str(&format!("- {module}\n"));
        }
        out.push('\n');
        if self.seams.is_empty() {
            out.push_str("seams: none — no module pair declares a shared-value obligation.\n");
            return out;
        }
        out.push_str("seams (each edge: its obligation, then the verdict its checker returned):\n");
        for seam in &self.seams {
            let kind = match seam.kind {
                SeamKind::Transport => "transport",
                SeamKind::Transform => "transform",
            };
            out.push_str(&format!(
                "- {} -- {} : {kind} on {}\n",
                seam.left, seam.right, seam.on
            ));
            let obligation = match seam.kind {
                SeamKind::Transport => "the modules share this value and must agree on its laws",
                SeamKind::Transform => "the conversion across the seam must be a homomorphism",
            };
            out.push_str(&format!("      obligation: {obligation}\n"));
            match &seam.status {
                SeamStatus::Coherent => out.push_str(
                    "      status: coherent — every law either module discovers holds under \
                     the other's operators\n",
                ),
                SeamStatus::Incoherent(violations) => {
                    out.push_str(
                        "      status: INCOHERENT — the modules disagree about the shared \
                         algebra:\n",
                    );
                    for violation in violations {
                        out.push_str(&format!("        * {violation}\n"));
                    }
                }
                SeamStatus::IllPosed(why) => {
                    out.push_str(&format!("      status: ILL-POSED — {why}\n"));
                }
                SeamStatus::ByConstruction => out.push_str(
                    "      status: discharged by construction — the shared value is one type \
                     on both sides (the declaration carries the compile-time witness)\n",
                ),
                SeamStatus::Preserved {
                    conversion,
                    via,
                    homomorphisms,
                    composites,
                } => {
                    out.push_str(&format!(
                        "      status: preserved — the conversion `{conversion}` is a \
                         discovered homomorphism (spanning theory: {via}):\n"
                    ));
                    for law in homomorphisms {
                        out.push_str(&format!("        * {law}\n"));
                    }
                    if !composites.is_empty() {
                        out.push_str("        composites (checked by running, never assumed):\n");
                        for law in composites {
                            out.push_str(&format!("        * {law}\n"));
                        }
                    }
                }
                SeamStatus::Unearned { conversion, via } => out.push_str(&format!(
                    "      status: UNEARNED — no homomorphism law for `{conversion}` in the \
                     spanning theory ({via}); the conversion does not yet preserve the \
                     algebra\n"
                )),
            }
        }
        out
    }

    /// This report as a `spec_lock::Lock` rooted in a caller-supplied spec directory — the
    /// CONSUMER-facing form, the exact sibling of `Spec::lock_in`. The lock file lives at
    /// `spec_dir/<slugified-system>.system.spec` and carries the canonical rendering.
    pub fn lock_in(&self, spec_dir: &Path) -> Lock {
        let slug: String = self
            .system
            .chars()
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();
        Lock {
            name: format!("{} system", self.system),
            path: spec_dir.join(format!("{slug}.system.spec")),
            live: self.render(),
        }
    }

    /// This report as a `spec_lock::Lock` in THIS repo's `spec/` directory — the same
    /// this-repo convenience as `Spec::lock` (a downstream crate uses [`SystemReport::lock_in`]
    /// with its own spec directory instead).
    pub fn lock(&self) -> Lock {
        self.lock_in(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("spec"))
    }
}

/// The SYSTEM-LEVEL DISTANCE: the declared modularity held against the LATENT modularity —
/// per registry module, does its discovered algebra decompose into sub-algebras no
/// declaration names? A declared module that is secretly several is the system-level twin
/// of an operator missing a declared law. Cohesion is a SUGGESTION, never a constraint
/// (see `discover::cohesion`), so this is a REPORT to read in review, not a red/green gate:
/// act on a latent split by re-drawing the declaration (a ratified system-lock diff), or
/// deliberately keep the module whole — both are decisions, and this names where one is due.
pub struct SystemDistance {
    /// The system's display name.
    pub system: &'static str,
    /// Each registry module's cohesion analysis, in declaration order.
    pub cohesions: Vec<CohesionReport>,
}

impl SystemDistance {
    /// Compute a system's distance report: every registry module through the cohesion
    /// analysis. Associated fn per the no-rats-nest rule.
    pub fn of<S: System>() -> SystemDistance {
        SystemDistance {
            system: S::name(),
            cohesions: S::cohesions(),
        }
    }

    /// The modules whose algebras DECOMPOSE — the latent splits no declaration names.
    pub fn latent(&self) -> Vec<&CohesionReport> {
        self.cohesions
            .iter()
            .filter(|report| !report.is_cohesive())
            .collect()
    }

    /// The report, in the distance voice: a one-line verdict, then each latent module's
    /// cohesion render (components and suggested seams) indented beneath it.
    pub fn render(&self) -> String {
        let cohesive = self.cohesions.len() - self.latent().len();
        let mut out = format!(
            "{}: {cohesive} of {} declared modules are cohesive",
            self.system,
            self.cohesions.len()
        );
        if self.latent().is_empty() {
            out.push_str("; no latent splits\n");
            return out;
        }
        out.push_str(
            "; LATENT SPLITS (suggestions, never constraints — re-draw the declaration or \
             deliberately keep the module whole):\n",
        );
        for report in self.latent() {
            for line in report.render().lines() {
                out.push_str(&format!("  {line}\n"));
            }
        }
        out
    }
}

/// Compile a `system!` declaration: the same grammar shape `genesis` parses for the
/// blank-slate path (name, modules, seams), as working code in a finished codebase. The
/// invoker declares the marker type; the macro implements [`System`] for it:
///
/// ```ignore
/// pub struct CreditApp;
/// boundary_spec::system! {
///     CreditApp : "credit-app",
///     modules {
///         Meter;                          // Spec::of::<Meter>()
///         Billing;                        // Spec::of::<Billing>()
///     }
///     seams {
///         Meter -- Billing : transport on Credits by construction;
///     }
/// }
/// ```
///
/// Module entries are theory types; `Type = expr;` overrides the spec derivation for a module
/// whose spec is richer than `Spec::of` (this repo's interpreter adds the structural `U` law).
/// Seam lines mirror the genesis grammar, `left -- right : kind on Value`, with the discharge
/// spelled where genesis leaves a hole:
///
///   - `A -- B : transport on V;` — same-signature theories; compiled to the coherence check
///     (does not compile when the signatures do not align — that misdeclaration is caught at
///     build time, not review time);
///   - `A -- B : transport on V by construction;` — `V` is ONE type both modules use; the
///     macro emits the compile-time witness `fn(V) -> V` (so `V` must be in scope);
///   - `A -- B : transform on V via h in Spanning;` — the conversion `h` crosses the seam,
///     discharged inside the spanning theory (its homomorphism law must be discovered;
///     composites through it are run, never assumed).
///
/// # The FULL-GRAMMAR form: one declaration, two lifecycle stages
///
/// The macro also accepts the ENTIRE genesis declaration — the same tokens
/// `examples/genesis_*.rs` carry — after a leading `Marker:`:
///
/// ```ignore
/// pub struct CreditApp;
/// boundary_spec::system! {
///     CreditApp:
///     name: "credit-app",
///     values { Credits = i64 where 0..=20 saturating; }
///     modules {
///         meter {
///             ops { zero() -> Credits; grant(Credits, Credits) -> Credits; }
///             expects { commutative(grant); identity(grant, zero); }
///         }
///     }
///     seams { /* the genesis seam grammar, verbatim */ }
/// }
/// ```
///
/// Genesis emits `src/system.rs` in exactly this shape — the ORIGINAL declaration tokens,
/// spliced verbatim — so the declaration is ONE artifact at every point in the crate's life,
/// and declaration↔code drift is a COMPILE error, not a stale document:
///
///   - each module name resolves to its genesis-conventional theory
///     (`meter` → `crate::ops::meter_ops::Meter`), so a module missing from code fails to
///     compile, and `modules()` — the registry — reads straight off the declaration;
///   - every declared operator signature becomes a WITNESS
///     (`const _: () = { let _: fn(Credits, Credits) -> Credits = crate::ops::…::grant; }`),
///     so renaming an operator, changing its arity, or moving a sort breaks the build with
///     the declaration as the named source of truth;
///   - `transport on V;` compiles to the by-construction discharge (genesis defines each
///     value once — the witness pins it); `transform on V via h;` compiles to the
///     homomorphism check in the genesis-named spanning theory
///     (`meter_billing_seam_ops::MeterBillingSeam`); a via-less transform is skipped here
///     (its hole lives in `tests/seams.rs`);
///   - `values { … }` rules and `expects { … }` clauses are accepted and carried (the rules
///     already generated their artifacts; the expectations are semantically gated by
///     `Distance` through the `#[algebra]` attribute) — v1 does not re-check them here.
///
/// The module-name → path derivation is why ops modules PUB-re-export their sorts (genesis
/// emits `pub use crate::meter::Credits;` inside `meter_ops`): the macro can only name a
/// value through a module the declaration mentions.
// `crate::` inside the expansion is DELIBERATE (and the whole point): the paths must
// resolve in the INVOKING crate's tree (`crate::ops::meter_ops::Meter`), which is exactly
// what bare `crate` does in a macro_rules expansion — `$crate` would wrongly point here.
#[allow(clippy::crate_in_macro_def)]
#[macro_export]
macro_rules! system {
    // ===== the full-grammar form: the genesis declaration verbatim, after `Marker:` =========
    (
        $sys:ident :
        name : $namestr:literal ,
        values { $($values:tt)* }
        modules {
            $( $m:ident {
                ops { $( $f:ident ( $($arg:ident),* ) -> $ret:ident ; )+ }
                $( expects { $($expects:tt)* } )?
            } )+
        }
        $( seams {
            $($seams:tt)*
        } )?
    ) => {
        impl $crate::discover::system::System for $sys {
            fn name() -> &'static str {
                $namestr
            }
            fn modules() -> ::std::vec::Vec<$crate::discover::Spec> {
                ::std::vec![ $(
                    $crate::__paste! {
                        $crate::discover::Spec::of::<crate::ops::[<$m _ops>]::[<$m:camel>]>()
                    }
                ),+ ]
            }
            fn seams() -> ::std::vec::Vec<$crate::discover::system::SeamReport> {
                $crate::__full_seams!( @parsed [] $( $($seams)* )? )
            }
            fn cohesions() -> ::std::vec::Vec<$crate::discover::cohesion::CohesionReport> {
                ::std::vec![ $(
                    $crate::__paste! {
                        $crate::discover::cohesion::CohesionReport::of::<
                            crate::ops::[<$m _ops>]::[<$m:camel>],
                        >()
                    }
                ),+ ]
            }
        }
        // DRIFT WITNESSES — every declared operator signature, held against the code at
        // compile time. The paths go through the declaration's own module names, so the
        // declaration is the source of truth the error message points back to.
        $( $crate::__paste! {
            const _: () = {
                $( let _: fn( $( crate::ops::[<$m _ops>]::$arg ),* )
                    -> crate::ops::[<$m _ops>]::$ret
                    = crate::ops::[<$m _ops>]::$f; )+
            };
        } )+
    };

    // ===== the compact form: theory types and explicit discharges ===========================
    (
        $sys:ident : $namestr:literal,
        modules {
            $( $module:ty $( = $spec:expr )? ; )+
        }
        $( seams {
            $($seams:tt)*
        } )?
    ) => {
        impl $crate::discover::system::System for $sys {
            fn name() -> &'static str {
                $namestr
            }
            fn modules() -> ::std::vec::Vec<$crate::discover::Spec> {
                ::std::vec![ $( $crate::__system_module!( $module $( = $spec )? ) ),+ ]
            }
            fn seams() -> ::std::vec::Vec<$crate::discover::system::SeamReport> {
                $crate::__system_seams!( @parsed [] $( $($seams)* )? )
            }
            fn cohesions() -> ::std::vec::Vec<$crate::discover::cohesion::CohesionReport> {
                ::std::vec![ $(
                    $crate::discover::cohesion::CohesionReport::of::<$module>()
                ),+ ]
            }
        }
    };
}

/// The seam lines of a FULL-GRAMMAR `system!` declaration — the genesis seam grammar,
/// munched one line at a time: `transport on V;` discharges by construction (with the
/// one-type witness through the left module's re-export), `transform on V via h;` checks the
/// homomorphism in the genesis-named spanning theory, and a via-less transform is dropped
/// (its hole lives in `tests/seams.rs`). Hidden: only ever invoked by `system!`'s expansion.
#[doc(hidden)]
#[allow(clippy::crate_in_macro_def)] // call-site paths on purpose — see `system!`
#[macro_export]
macro_rules! __full_seams {
    ( @parsed [ $($done:expr,)* ] ) => {
        ::std::vec![ $($done),* ]
    };
    ( @parsed [ $($done:expr,)* ]
      $l:ident -- $r:ident : transport on $v:ident ; $($rest:tt)* ) => {
        $crate::__full_seams!( @parsed [ $($done,)*
            $crate::__paste! {{
                // the compile-time discharge: `$v` names ONE type, reachable through the
                // left module's ops re-exports — two diverged types stop compiling here.
                let _witness: fn(crate::ops::[<$l _ops>]::$v) -> crate::ops::[<$l _ops>]::$v =
                    |value| value;
                $crate::discover::system::SeamReport::transport_by_construction::<
                    crate::ops::[<$l _ops>]::[<$l:camel>],
                    crate::ops::[<$r _ops>]::[<$r:camel>],
                >(::std::stringify!($v))
            }}, ]
            $($rest)* )
    };
    ( @parsed [ $($done:expr,)* ]
      $l:ident -- $r:ident : transform on $v:ident via $h:ident ; $($rest:tt)* ) => {
        $crate::__full_seams!( @parsed [ $($done,)*
            $crate::__paste! {
                $crate::discover::system::SeamReport::transform::<
                    crate::ops::[<$l _ops>]::[<$l:camel>],
                    crate::ops::[<$r _ops>]::[<$r:camel>],
                    crate::ops::[<$l _ $r _seam_ops>]::[<$l:camel $r:camel Seam>],
                >(::std::stringify!($v), ::std::stringify!($h))
            }, ]
            $($rest)* )
    };
    ( @parsed [ $($done:expr,)* ]
      $l:ident -- $r:ident : transform on $v:ident ; $($rest:tt)* ) => {
        $crate::__full_seams!( @parsed [ $($done,)* ] $($rest)* )
    };
}

/// One module entry of a `system!` declaration: a bare theory type derives its spec with
/// `Spec::of`; `Type = expr` supplies it. Hidden: only ever invoked by `system!`'s expansion.
#[doc(hidden)]
#[macro_export]
macro_rules! __system_module {
    ( $module:ty = $spec:expr ) => {
        $spec
    };
    ( $module:ty ) => {
        $crate::discover::Spec::of::<$module>()
    };
}

/// The seam lines of a `system!` declaration, munched one at a time (three forms — see
/// [`crate::system!`]). Hidden: only ever invoked by `system!`'s expansion.
#[doc(hidden)]
#[macro_export]
macro_rules! __system_seams {
    ( @parsed [ $($done:expr,)* ] ) => {
        ::std::vec![ $($done),* ]
    };
    ( @parsed [ $($done:expr,)* ]
      $l:ident -- $r:ident : transport on $v:ident ; $($rest:tt)* ) => {
        $crate::__system_seams!( @parsed [ $($done,)*
            $crate::discover::system::SeamReport::transport::<$l, $r>(::std::stringify!($v)), ]
            $($rest)* )
    };
    ( @parsed [ $($done:expr,)* ]
      $l:ident -- $r:ident : transport on $v:ident by construction ; $($rest:tt)* ) => {
        $crate::__system_seams!( @parsed [ $($done,)*
            {
                // the compile-time discharge: `$v` names ONE type, reachable from this
                // declaration — two diverged types stop compiling here.
                let _witness: fn($v) -> $v = |value| value;
                $crate::discover::system::SeamReport::transport_by_construction::<$l, $r>(
                    ::std::stringify!($v),
                )
            }, ]
            $($rest)* )
    };
    ( @parsed [ $($done:expr,)* ]
      $l:ident -- $r:ident : transform on $v:ident via $h:ident in $via:ty ; $($rest:tt)* ) => {
        $crate::__system_seams!( @parsed [ $($done,)*
            $crate::discover::system::SeamReport::transform::<$l, $r, $via>(
                ::std::stringify!($v),
                ::std::stringify!($h),
            ), ]
            $($rest)* )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::coherence::{FirstMerge, GcdMerge, MaxMerge};
    use crate::discover::{all_specs, BoundarySpec};

    /// THIS REPO'S graph is the registry: the compiled `BoundarySpec` declaration carries
    /// exactly the four theories `all_specs` used to hand-list (same names, same order), the
    /// module-override arm threads the interpreter's RICHER spec (the structural `U` law is
    /// present, so the override was not silently replaced by `Spec::of`), and with no seams
    /// declared the graph is trivially met.
    #[test]
    fn the_repo_graph_is_the_registry() {
        let report = SystemReport::of::<BoundarySpec>();
        assert_eq!(report.system, "boundary-spec");
        assert_eq!(
            report.modules,
            vec![
                "interpreter arithmetic",
                "router",
                "date calculus",
                "ttl store",
                "store protocol"
            ]
        );
        assert!(report.seams.is_empty());
        assert!(report.is_met());
        let interpreter = &all_specs()[0];
        assert!(
            interpreter
                .laws
                .iter()
                .any(|law| law.equation.contains("U(p) = U(q)")),
            "the override arm must carry interpreter_spec()'s U law into the registry"
        );
    }

    /// THE SYSTEM-LEVEL DISTANCE over this repo's own graph — and the report earns its
    /// keep twice over. Two of the five declared modules are SECRETLY SEVERAL,
    /// byte-pinned: the interpreter splits arithmetic from comparison, the calendar
    /// splits duration arithmetic from epoch conversion — real architectural
    /// observations, each with its suggested seam kind, held here as a deliberate
    /// keep-whole decision (this repo's modules are demonstration substrates; splitting
    /// them is the downstream lesson, not ours).
    ///
    /// The TTL store USED to be the third split (merge monoid vs clock action) — until
    /// the action-law stanzas landed and discovery found `tick(empty, p) = empty`: the
    /// clock action FIXES the merge monoid's identity, a law that bridges the two
    /// clusters. The split dissolved because the law language grew, not because any
    /// code moved — cohesion is a verdict about what the vocabulary can SEE, and this
    /// pin records the day that stopped being a hypothetical.
    #[test]
    fn the_repo_distance_names_the_latent_splits() {
        let distance = SystemDistance::of::<BoundarySpec>();
        assert_eq!(distance.latent().len(), 2);
        let expected = "\
boundary-spec: 3 of 5 declared modules are cohesive; LATENT SPLITS (suggestions, never constraints — re-draw the declaration or deliberately keep the module whole):
  module `interpreter arithmetic`: decomposes into 2 latent modules — consider splitting:
    module 0: { 0, 1, +, * }
    module 1: { false, < }
    seam 0↔1 on Int — transport (algebra stays — check coherence)
  module `date calculus`: decomposes into 2 latent modules — consider splitting:
    module 0: { zero, +, add, diff }
    module 1: { since, at }
    seam 0↔1 on Date, Duration — transform (algebra changes — check the homomorphism)
";
        assert_eq!(distance.render(), expected);
    }

    /// A system whose modules are all single algebras reports flat, in one line — and the
    /// macro generated `cohesions()` for every module (the count pins the plumbing).
    #[test]
    fn a_cohesive_system_reports_no_latent_splits() {
        let distance = SystemDistance::of::<MergeSuite>();
        assert_eq!(
            distance.cohesions.len(),
            3,
            "one analysis per registry module"
        );
        assert!(distance.latent().is_empty());
        assert_eq!(
            distance.render(),
            "merge suite: 3 of 3 declared modules are cohesive; no latent splits\n"
        );
    }

    /// The committed SYSTEM lock is FRESH: the live graph render matches what was ratified.
    /// A drift (a module added or renamed, a seam re-drawn or its verdict flipped) fails here —
    /// regenerate with `cargo run --example freeze_spec` and ratify the diff.
    #[test]
    fn the_committed_system_spec_is_fresh() {
        let lock = SystemReport::of::<BoundarySpec>().lock();
        if let Err(stale) = spec_lock::check(std::slice::from_ref(&lock)) {
            panic!(
                "the system graph drifted from the committed lock for: {}. \
                 Run `cargo run --example freeze_spec` and ratify the diff.",
                stale.join(", ")
            );
        }
    }

    // ===== a compiled system with all three transport verdicts ================================
    //
    // The merge theories from `coherence` (same signature, different semantics) joined into one
    // declared graph: max/gcd agree (coherent), max/first-match disagree (incoherent), and the
    // by-construction arm is exercised on the shared raw type. A deliberately mixed fixture:
    // one lock render shows every transport verdict at once.

    type Key = i64;

    struct MergeSuite;
    crate::system! {
        MergeSuite : "merge suite",
        modules {
            MaxMerge;
            GcdMerge;
            FirstMerge;
        }
        seams {
            MaxMerge -- GcdMerge : transport on Key;
            MaxMerge -- FirstMerge : transport on Key;
            GcdMerge -- FirstMerge : transport on Key by construction;
        }
    }

    /// The three transport verdicts, computed (never transcribed) by the compiled seams:
    /// max/gcd share every law (coherent); max/first-match disagree about commutativity
    /// (incoherent, with the violation carried); the by-construction seam is met by the
    /// macro's compile-time witness.
    #[test]
    fn transport_seams_are_put_to_the_coherence_checker() {
        let report = SystemReport::of::<MergeSuite>();
        assert_eq!(
            report.modules,
            vec!["max-merge", "gcd-merge", "first-merge"]
        );
        assert!(
            !report.is_met(),
            "the incoherent seam must leave the graph red"
        );

        let [agree, disagree, constructed] = report.seams.as_slice() else {
            panic!("three declared seams, got {}", report.seams.len());
        };
        assert_eq!(
            (agree.left, agree.right, agree.on),
            ("max-merge", "gcd-merge", "Key")
        );
        assert_eq!(agree.kind, SeamKind::Transport);
        assert!(matches!(agree.status, SeamStatus::Coherent));

        let SeamStatus::Incoherent(violations) = &disagree.status else {
            panic!("max/first-match must be incoherent: {:?}", disagree.status);
        };
        assert!(
            violations.iter().any(|v| v.contains("either order")),
            "the commutativity disagreement must be carried: {violations:?}"
        );
        assert!(!disagree.status.is_met());

        assert!(matches!(constructed.status, SeamStatus::ByConstruction));
        assert!(constructed.status.is_met());
    }

    /// The system lock renders the WHOLE graph — registry, seam lines, obligations, verdicts —
    /// pinned byte-for-byte: this text is what a reviewer ratifies, so its exact shape is the
    /// product. (Deterministic: discovery is a pure function of the theories.)
    #[test]
    fn the_graph_renders_exactly() {
        let expected = "\
# system spec: merge suite — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.

modules (the ratified registry — one committed module lock each):
- max-merge
- gcd-merge
- first-merge

seams (each edge: its obligation, then the verdict its checker returned):
- max-merge -- gcd-merge : transport on Key
      obligation: the modules share this value and must agree on its laws
      status: coherent — every law either module discovers holds under the other's operators
- max-merge -- first-merge : transport on Key
      obligation: the modules share this value and must agree on its laws
      status: INCOHERENT — the modules disagree about the shared algebra:
        * \"Merge gives the same result in either order.\" holds in max-merge but not first-merge
- gcd-merge -- first-merge : transport on Key
      obligation: the modules share this value and must agree on its laws
      status: discharged by construction — the shared value is one type on both sides (the declaration carries the compile-time witness)
";
        assert_eq!(SystemReport::of::<MergeSuite>().render(), expected);
    }

    /// The lock lands at `<spec_dir>/<slug>.system.spec` — spaces slugified, the `.system.`
    /// infix keeping it apart from the module locks in the same directory.
    #[test]
    fn the_system_lock_has_its_own_namespace() {
        let lock = SystemReport::of::<MergeSuite>().lock_in(Path::new("spec"));
        assert_eq!(lock.name, "merge suite system");
        assert_eq!(lock.path, Path::new("spec").join("merge-suite.system.spec"));
        assert_eq!(lock.live, SystemReport::of::<MergeSuite>().render());
    }

    /// A misaligned transport seam (operator tables in different orders) is ILL-POSED — the
    /// verdict names the misalignment instead of judging the wrong operators, and it does not
    /// count as met.
    #[test]
    fn a_misaligned_transport_seam_is_ill_posed_not_judged() {
        pub struct SwappedMerge;
        crate::theory! {
            SwappedMerge : "swapped-merge", Value = i64, Obs = i64, Sort = crate::discover::coherence::Sort,
            sort_of = |_: &i64| crate::discover::coherence::Sort::Key,
            observe = |v: &i64| *v,
            vars { crate::discover::coherence::Sort::Key => &["a", "b", "c"], }
            inhabit { crate::discover::coherence::Sort::Key => vec![0, 1, 2, 3, 4, 6, 12], }
            ops {
                Infix   "Merge" "merge" (crate::discover::coherence::Sort::Key, crate::discover::coherence::Sort::Key) -> crate::discover::coherence::Sort::Key = |v: &[i64]| Some(v[0].max(v[1]));
                Nullary "Empty" "empty" () -> crate::discover::coherence::Sort::Key = |_: &[i64]| Some(0);
            }
        }
        let seam = SeamReport::transport::<MaxMerge, SwappedMerge>("Key");
        let SeamStatus::IllPosed(why) = &seam.status else {
            panic!("misaligned tables must be ill-posed: {:?}", seam.status);
        };
        assert!(
            why.contains("misaligned"),
            "the verdict names the fault: {why}"
        );
        assert!(!seam.status.is_met());
        // and it renders as a loud, named verdict.
        let text = SystemReport {
            system: "swapped",
            modules: vec!["max-merge", "swapped-merge"],
            seams: vec![seam],
        }
        .render();
        assert!(text.contains("status: ILL-POSED — operator tables misaligned"));
    }

    // ===== a compiled system with a transform seam =============================================
    //
    // Two stage modules and a conversion crossing between them. The seam is discharged inside
    // the SPANNING theory `Span` (both stages' operators plus the conversions): discovery must
    // find the homomorphism laws, and the A→C composite is verified by running (PipelineLaw).
    // `BrokenSpan` negates the second conversion — not a homomorphism over max — so the same
    // declaration shape lands UNEARNED.

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    struct One;
    struct SourceStage;
    struct TargetStage;
    fn maxi(v: &[i64]) -> Option<i64> {
        Some(v[0].max(v[1]))
    }
    crate::theory! {
        SourceStage : "source stage", Value = i64, Obs = i64, Sort = One,
        sort_of = |_: &i64| One,
        observe = |v: &i64| *v,
        vars { One => &["a", "b", "c"], }
        inhabit { One => vec![0, 1, 2], }
        ops { Infix "max" "max" (One, One) -> One = maxi; }
    }
    crate::theory! {
        TargetStage : "target stage", Value = i64, Obs = i64, Sort = One,
        sort_of = |_: &i64| One,
        observe = |v: &i64| *v,
        vars { One => &["a", "b", "c"], }
        inhabit { One => vec![0, 1, 2], }
        ops { Infix "max" "max" (One, One) -> One = maxi; }
    }

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    enum S3 {
        A,
        B,
        C,
    }
    struct Span;
    struct BrokenSpan;
    fn maxa(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((0, v[0].1.max(v[1].1)))
    }
    fn maxb(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1.max(v[1].1)))
    }
    fn maxc(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, v[0].1.max(v[1].1)))
    }
    fn hab(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((1, v[0].1))
    }
    fn hbc(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, v[0].1))
    }
    fn hbc_broken(v: &[(u8, i64)]) -> Option<(u8, i64)> {
        Some((2, -v[0].1))
    }
    macro_rules! span_theory {
        ($thy:ty, $name:literal, $hbc:expr) => {
            crate::theory! {
                $thy : $name, Value = (u8, i64), Obs = (u8, i64), Sort = S3,
                sort_of = |v: &(u8, i64)| match v.0 { 0 => S3::A, 1 => S3::B, _ => S3::C },
                observe = |v: &(u8, i64)| *v,
                vars { S3::A => &["a"], S3::B => &["b"], S3::C => &["c"], }
                inhabit {
                    S3::A => vec![(0, 0), (0, 1), (0, 2)],
                    S3::B => vec![(1, 0), (1, 1), (1, 2)],
                    S3::C => vec![(2, 0), (2, 1), (2, 2)],
                }
                ops {
                    Infix  "maxA" "maxA" (S3::A, S3::A) -> S3::A = maxa;
                    Infix  "maxB" "maxB" (S3::B, S3::B) -> S3::B = maxb;
                    Infix  "maxC" "maxC" (S3::C, S3::C) -> S3::C = maxc;
                    Prefix "hAB"  "hAB"  (S3::A) -> S3::B = hab;
                    Prefix "hBC"  "hBC"  (S3::B) -> S3::C = $hbc;
                }
            }
        };
    }
    span_theory!(Span, "span", hbc);
    span_theory!(BrokenSpan, "broken span", hbc_broken);

    struct StagedApp;
    crate::system! {
        StagedApp : "staged app",
        modules {
            SourceStage;
            TargetStage;
        }
        seams {
            SourceStage -- TargetStage : transform on Magnitude via hAB in Span;
        }
    }

    /// The transform seam is discharged by DISCOVERY in the spanning theory: the named
    /// conversion is found to be a homomorphism, and the A→C composite THROUGH it is a
    /// pipeline law checked by running — all carried into the verdict, so the lock states
    /// them.
    #[test]
    fn a_transform_seam_is_discharged_inside_the_spanning_theory() {
        let report = SystemReport::of::<StagedApp>();
        assert!(report.is_met());
        let [seam] = report.seams.as_slice() else {
            panic!("one declared seam");
        };
        assert_eq!(seam.kind, SeamKind::Transform);
        let SeamStatus::Preserved {
            conversion,
            via,
            homomorphisms,
            composites,
        } = &seam.status
        else {
            panic!("hAB is a homomorphism: {:?}", seam.status);
        };
        assert_eq!((*conversion, *via), ("hAB", "span"));
        assert!(
            homomorphisms.iter().any(|l| l.contains("hAB")),
            "the named conversion's law must be found: {homomorphisms:?}"
        );
        assert!(
            composites.iter().any(|l| l.contains("hBC∘hAB")),
            "the composite through it must be checked by running: {composites:?}"
        );
        let text = report.render();
        assert!(text.contains("status: preserved — the conversion `hAB`"));
        assert!(text.contains("composites (checked by running, never assumed):"));
    }

    /// A broken conversion (negation does not commute with max) lands UNEARNED — even though
    /// ANOTHER conversion in the same spanning theory (`hAB`) is a perfectly good
    /// homomorphism: the verdict is tied to the seam's NAMED conversion, so a healthy
    /// neighbour cannot discharge a broken seam.
    #[test]
    fn a_broken_conversion_leaves_the_transform_seam_unearned() {
        let seam =
            SeamReport::transform::<SourceStage, TargetStage, BrokenSpan>("Magnitude", "hBC");
        let SeamStatus::Unearned { conversion, via } = seam.status else {
            panic!(
                "a non-homomorphic conversion must be unearned: {:?}",
                seam.status
            );
        };
        assert_eq!((conversion, via), ("hBC", "broken span"));
        assert!(!seam.status.is_met());
        let text = SystemReport {
            system: "broken staged app",
            modules: vec!["source stage", "target stage"],
            seams: vec![seam],
        }
        .render();
        assert!(text.contains("status: UNEARNED — no homomorphism law for `hBC`"));
        assert!(text.contains("(broken span)"));
    }

    /// A conversion the spanning theory does not declare is ILL-POSED, not unearned — a
    /// misspelling must never sit in the lock looking like unfinished work.
    #[test]
    fn a_conversion_the_span_lacks_is_ill_posed() {
        let seam = SeamReport::transform::<SourceStage, TargetStage, Span>("Magnitude", "hXY");
        let SeamStatus::IllPosed(why) = &seam.status else {
            panic!("an unknown conversion must be ill-posed: {:?}", seam.status);
        };
        assert!(
            why.contains("`hXY` is not an operator"),
            "the verdict names the fault: {why}"
        );
    }
}
