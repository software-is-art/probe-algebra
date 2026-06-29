# How it works

The end-to-end mechanism: what you write, what each level validates, and why an
under-specified probe is unrepresentable. Read [concepts](concepts.md) for the terminology.

## 1. What you write — the one rigid place

A module's boundary is the only rigid thing. For the interpreter it is, in full:

- **value objects** — each domain primitive plus its validity rule. `refined!` takes the rule
  and generates the newtype, the parse-don't-validate constructor, and registration:

  ```rust,ignore
  refined! {
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
      pub struct Int(i64);
      fn new(n: i64) = (n >= 0).then_some(n);   // the predicate is the only content
  }
  ```

- **edges** — one of the four shapes, with the transformation body. The body calls the
  interior; the *shape* is the contract:

  ```rust,ignore
  impl Guarded for Eval {
      type Capability = Pure;
      type Proof<N>  = WellTyped<N>;          // uncallable without Check's witness
      type Out<N>    = Named<N, Value>;
      fn guard<N>(&self, e: &Named<N, Expr>, _: &WellTyped<N>) -> Named<N, Value> {
          e.map(super::internal::eval)         // the interior, untyped-by-the-grammar
      }
  }
  ```

That is the rigidity, in one file. Nothing below the boundary is constrained in style.

## 2. What compile time validates

Claims that can be statically false become build errors, each pinned by a `tests/compile_fail`
fixture that must *fail to compile*:

- **"well-typed programs don't go wrong"** — `Eval` demands a `WellTyped<N>` for the *same*
  brand; an unchecked, ill-typed, or wrong-program proof won't type-check (`eval_wrong_program`);
- **effect ceilings** — `run_pure` accepts only `AtMost<Pure>`; the `Lossy` folder is refused
  (`run_pure_rejects_lossy`);
- **cost budgets** — mapping an O(n) edge per element is O(n²); demanding it stay linear is a
  type error (`cost_over_budget`);
- **probe completeness** — a probe covering only some degrees of freedom cannot satisfy
  `require_complete` (`incomplete_probe_rejected`).

The grading algebras additionally **prove their own laws** at compile time (see
[concepts](concepts.md#the-gradings)) — the part mutation can't reach.

## 3. What autotest time validates

Everything the type system can't see — does the code compute the *right values*? — is checked
by probes, and the probes are checked by mutation:

- **generators, DOFs, and the complete probe are derived** by `#[derive(Shaped)]`: `inhabitant`
  (a seed), `perturbation_classes` (a neighbour-group per dimension), `HasDofs` (the DOF set).
- the **fused probe** `sensitive_to_all(map, x)` runs a map against every derived perturbation
  class: a faithful map (`render`) responds to all; a collapsing map (`node_count`) is caught.
- the **autogen harness** registers each edge once and gets its whole suite generically — the
  parse round-trip, the capability/residual laws, `ConstFold`'s fold-preserves-value law,
  `Resolve`'s two-route law, and `eval_semantics_are_probed`, which pins every evaluator arm
  oracle-free against an independent computation — so evaluation needs *no hand-written examples*.
- **value laws are declared, not plumbed** — you write `Law::Identity { op: Add, element: 0 }`
  and the harness mints a `Relation` over a derived generator. The applicability guard and the
  **non-vacuity check** are the harness's job (`relation_laws` fails if the probe fired zero
  times), so a vacuously-passing test cannot ship.
- **`cargo mutants` judges all of it** — the interior carries no tests; the sweep reports how
  many of its mutants the boundary kills.

## 4. Why a weak probe is impossible, not merely caught

The chain the derivation buys:

1. `#[derive(Shaped)]` emits `HasDofs` with one `Field<T, I>` per variant/field — the DOF set is
   **derived**, no marker to forget;
2. `Complete<T>` covers every `Field<T, I>` by a blanket impl — the complete probe is
   **generated**, so `assert_complete::<T>()` holds by construction;
3. the fused probe perturbs along every derived class — so the *runtime* probe is complete too.

The only thing left hand-writable is a *partial* `Covers` set, which `require_complete` rejects
at compile time. The same shape applies to the **edge set**: `assert_all_probed::<Edges>()`
makes "every listed edge carries a probe" a compile-time bound, and `build.rs` closes the
open-world residue an in-language check can't — it **enumerates every concrete edge impl in the
source** and rejects any without an `impl Probed` (a build error). A counterexample fixture is
`#[cfg(test)]`, so it is skipped: it is not a spec edge. The type system can't enumerate impls;
the build step can, so edge-completeness is total.

## 5. The interior is free — and that freedom is audited

`interp::internal` is ordinary imperative Rust — `HashMap` environments, mutable cursors,
recursion — with **zero tests**. Its correctness is a consequence of the boundary, *measured* by
the mutation sweep: every viable mutant dies, the survivors only documented equivalents.

Freedom doesn't let the interior lie about its capability. The `capability` module *audits a
declaration against behaviour* — perturb a declared source and watch whether the output moves —
catching both **over-claiming** (declares `Stateful`, ignores state → slop) and **under-claiming**
(declares `Pure`, secretly reads state → a hidden dependency the type system trusted). This is
the one honesty check the type level can't yet do, because a declared capability is trusted, not
inferred from the body.

## 6. Self-hosting, and what it unlocks

The discipline runs on *every* runtime module. `select` (the kill-matrix selector) is a second
structural self-host; `gdp` and `capability` are self-hosted by *verification* — example tests
replaced with oracle-free property probes (`permute ∘ unpermute == id`, the liveness rule, the
audit's claim/behaviour reconciliation), kept in the sweep. And `gdp` is load-bearing: `select`
reads its matrix through gdp's `InBounds` proof, so an index proven for one matrix can't index
another and an out-of-range read is a type error, not a panic — the same proof-carrying that
makes `Eval` uncallable without `Check`'s witness, lifted to a *relation between two values*.

Once the abstraction certifies itself:

- **consumers stop paying for mutation** — it validates the abstraction once; a module built on
  the discipline inherits the guarantee and runs only the cheap derived probes per PR;
- **a new verified module is nearly free** — declaring value objects and laws is the whole
  authoring cost; the harness mints the probes;
- **the selector closes the loop on real data** (`cargo run --example suite_audit`): it reads
  `cargo-mutants`' own output into a real kill matrix and runs `select` over it — the minimal
  attributing suite, and survivors as *missing-relation* signals — the method optimizing its own
  tests;
- **the trusted base is small and explicit** — `rustc`, `cargo-mutants`, `typewit`, the grammar,
  and the hand-written rejection tests. Nothing else is taken on faith.

That was the whole question: the boundary is paid once, the interior stays free, the method
certifies its own runtime, and the spec's claims are validated at compile time *and* autotest
time.
