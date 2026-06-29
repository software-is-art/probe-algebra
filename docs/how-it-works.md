# How it works

The end-to-end mechanism: what you write, what each level validates, and why an
under-specified probe is not just caught but unrepresentable. Read [concepts](concepts.md)
first if you want the terminology.

## 1. What you write — the one rigid place

A module's boundary is the only thing held rigid. For the interpreter it is, in full:

- **value objects** — each domain primitive plus its validity rule. `refined!` takes the
  rule and generates the newtype, the parse-don't-validate constructor, and registration:

  ```rust,ignore
  refined! {
      #[derive(Debug, Clone, Copy, PartialEq, Eq, Shaped)]
      pub struct Int(i64);
      fn new(n: i64) = (n >= 0).then_some(n);   // the predicate is the only content
  }
  ```

- **edges** — one of the four shapes, with the transformation body. The body calls into the
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

That is the rigidity, in one file (`interp/boundary.rs`). Nothing below the boundary is
constrained in style.

## 2. What compile time validates

The specification's claims that can be statically false become build errors. Each is pinned
by a `tests/compile_fail` fixture that must *fail to compile*:

- **"well-typed programs don't go wrong"** — `Eval` demands a `WellTyped<N>` for the *same*
  brand as the expression. An unchecked or ill-typed program, or a proof minted for a
  *different* program, will not type-check. (`eval_wrong_program`)
- **effect ceilings** — `run_pure` accepts only `AtMost<Pure>`; the `Lossy` constant-folder
  is refused. (`run_pure_rejects_lossy`)
- **cost budgets** — mapping an O(n) edge per element is O(n²); demanding it stay linear is a
  type error. (`cost_over_budget`)
- **probe completeness** — a hand-written probe that covers only some degrees of freedom
  cannot satisfy `require_complete`. (`incomplete_probe_rejected`)

## 3. What autotest time validates

Everything the type system cannot see — does the code compute the *right values*? — is
checked by probes, and the probes are checked by mutation.

- **generators, degrees of freedom, and the complete probe are derived** from each value
  object's structure by `#[derive(Shaped)]`: `inhabitant()` (a seed), `perturbation_classes()`
  (one neighbour-group per dimension), and `HasDofs` (the DOF set).
- the **fused universal probe** `sensitive_to_all(map, x)` runs a map against every derived
  perturbation class: a faithful map (`render`) responds to all; a map that collapses a
  dimension (`node_count`, which ignores the operator and literal values) is caught.
- the **law registry** (`src/laws.rs`) registers each edge once and gets its whole structural
  suite — the parse round-trip, the capability/residual law — generically.
- **value laws are declared, not plumbed.** For the value frontier you write only the
  *meaning* — `Law::Identity { op: Add, element: 0 }`, `Law::Commutative { op: Mul }` — and
  the harness mints a first-class `Relation` around it, over a derived generator. The
  applicability guard and the **non-vacuity check** are the harness's job: `relation_laws`
  counts how many generated inputs the probe actually fired on and fails if that count is
  zero. A probe whose guard never holds — the hand-written-guard antipattern — is rejected on
  an ordinary run, so the author cannot ship a vacuously-passing test.
- **`cargo mutants` judges all of it.** The interpreter's interior carries no tests; the
  sweep reports how many of its mutants the boundary kills.

## 4. Why a weak probe is impossible, not merely caught

This is the property the "full mechanical derivation" pass buys. The chain is:

1. `#[derive(Shaped)]` emits `HasDofs` with one `Field<T, I>` marker per variant/field — the
   degree-of-freedom set is **derived**, so there is no hand-written marker to forget.
2. `Complete<T>` covers *every* `Field<T, I>` by a blanket impl — the complete probe is
   **generated**, so `assert_complete::<T>()` holds by construction.
3. The fused probe perturbs along every derived class — so the *runtime* probe is likewise
   complete by construction.

The only thing left hand-writable is a *partial* `Covers` set, and `require_complete` rejects
it at compile time. An agent therefore cannot specify a probe that misses a dimension: the
complete one is derived, and the incomplete one does not compile.

## 5. The interior is free — and that freedom is audited

`interp::internal` is ordinary imperative Rust: `HashMap` environments, mutable cursors,
recursion, allocation. It has **zero tests**. Its correctness is entirely a consequence of
the boundary, *measured* by the interior mutation sweep (72 of 73 viable mutants killed; the
one survivor provably equivalent).

Freedom does not mean the interior can lie about its capability. The `capability` module
*audits a declaration against behaviour*: perturb a declared capability source and watch
whether the output moves. It catches both directions —

- **over-claiming** (declares `Stateful`, ignores the state) → slop;
- **under-claiming** (declares `Pure`, secretly reads the environment) → a hidden dependency
  the type system trusted, surfaced by the probe.

So the boundary is paid once, the interior stays free, and the claims in the specification
are validated at compile time *and* at autotest time — which was the whole question.
