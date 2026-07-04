# The spec-lock discipline: freeze the spec, gate the drift, mutate the diff

This repo's CI is built around one extractable pattern, and none of it depends on the discovery
engine or on algebra. If your project can derive *any* text about itself deterministically — an
API census, a schema dump, a CLI's `--help`, a routing table, a discovered spec — you can lift
the whole discipline. It is four moves; each buys something specific and costs something
specific.

(The pattern's endpoint, applied here: the pipeline itself is move-1-through-3'd.
`.github/workflows/ci.yml` is rendered from a declared gate registry (`discover::gates`) and
drift-gated byte for byte, so CI validates its own definition inside the `cargo test` it runs;
the mutation economics — changed lines per PR, the diff since the `mutants-green` tag per
merge, a sharded full sweep weekly — are cadence DATA in that registry, not YAML accidents.
This document describes the extractable core; [discovery.md](discovery.md#the-pipeline) has
the pipeline-as-lock specifics.)

## Move 1 — derive a deterministic spec artifact

Have a tool derive a text artifact from the code: what the code *means*, computed rather than
asserted. Here it is the discovered algebraic spec (`cargo run --example freeze_spec` renders
each theory's laws to text via [`src/discover/freeze.rs`](../src/discover/freeze.rs)); in an
ordinary codebase it might be `cargo public-api`'s output, an OpenAPI dump, or a generated
grammar summary.

**What it buys:** a review surface. Behaviour becomes a document you can read, diff, and argue
about, instead of a property scattered across the implementation.

**The honest constraint:** the derivation must be **deterministic** — same code, same bytes, on
every machine. Sort collections, pin formatting, keep timestamps, hostnames, and hash-map
iteration order out of the output. If two runs on the same commit differ, every later move
degenerates into noise; fix that first.

## Move 2 — freeze it to a committed file

Run the derivation once and commit its output. That is the whole [`spec-lock`](../spec-lock)
crate: a `Lock` names one artifact (display name, committed path, live text), `bless` writes the
live texts, `check` compares them to the committed files. Here the locks live in
[`spec/`](../spec) — one `.spec` file per theory, plus the boundary-qualification census
([`spec/qualify.spec`](../spec/qualify.spec)).

**What it buys:** the expensive derivation runs once, and the file stops being documentation
*about* the code and becomes a behaviour lock the repository carries. History now records what
the code meant at every commit.

## Move 3 — drift-gate it in CI, so the PR diff is the ratification

A plain test re-derives the live text and fails if it differs from the committed file — here
`freeze::check_fresh`, exercised by the `the_committed_specs_are_fresh` test inside
[`src/discover/freeze.rs`](../src/discover/freeze.rs) and run by the `check` job of
[`.github/workflows/ci.yml`](../.github/workflows/ci.yml). A missing lock file is stale, never
fresh.

The rule that makes this a discipline rather than a snapshot test: **the fix for a red gate is
never to edit the lock file by hand.** You regenerate (`cargo run --example freeze_spec`) and
put the resulting diff through review. So the committed diff, read in the pull request, IS the
ratification — the one human act the derivation cannot perform. An *unintended* behaviour change
is a build error; an *intended* one is a readable diff someone approves. A law you expected and
don't see in that diff is a bug surfaced before merge.

**What it buys:** behaviour review happens where review already happens. Nobody has to remember
to re-run the tool; forgetting is a red build, not a stale document.

## Move 4 — scope mutation testing to changed lines, with a periodic full sweep

Moves 1–3 lock what the code means; mutation testing measures whether the test surface could
*tell* if it changed. A full sweep is expensive, so it runs where it pays
([`.github/workflows/ci.yml`](../.github/workflows/ci.yml)):

- **on every PR** (`mutants-diff`): `git diff base...HEAD > pr.diff`, then
  `cargo mutants --in-diff pr.diff` — only the lines the PR touches are mutated. New code no
  test can kill fails the gate; pre-existing accepted equivalents are not in the diff, so they
  never re-trip it.
- **on the default branch and weekly** (`mutants-full`): the whole-crate sweep, green gate
  "0 missed". This is the backstop for what diff-scoping cannot see — a PR that deletes a test,
  weakening coverage of *unchanged* lines, passes the diff gate but fails the next sweep.

Both run through [`.github/mutants-gate.sh`](../.github/mutants-gate.sh), which keys on the
recorded survivors in `outcomes.json` rather than on `cargo mutants`' exit code, because the
exit code conflates two different verdicts (see the residue policy below).

**What it buys:** per-PR mutation cost proportional to the PR, with the full guarantee
re-established on a schedule — you pay the sweep once per week, not once per change.

## The costs, paid openly

- **Determinism is a real engineering obligation** (move 1). Any nondeterminism in the
  derivation — parallel iteration order, float formatting, environment leakage — must be
  hunted down before the gate is trustworthy.
- **Engine changes can churn every lock at once.** When the *deriving tool* changes in a way
  that changes the law set (a new law shape, a renderer tweak), every committed lock
  regenerates in one PR. That diff is large and mostly mechanical, and it still deserves a
  human eye — the churn is the price of the ratification being real. Budget for it when
  touching the engine. (Engine facts that would churn with *no* behaviour change — the
  consequence-equality counts — are deliberately kept out of the locks; they live in this
  repo's golden tests instead. See the upgrade contract below.)
- **Timeout mutants leave a reviewable residue.** A non-termination mutant (a relaxed loop
  guard) makes the mutant hang while the original terminates — that is a *detection*, not a
  survivor, so the gate passes a timeouts-only run. The honest caveat, from
  [`.github/mutants-gate.sh`](../.github/mutants-gate.sh) itself: a behaviour-changing mutant
  that ALSO runs past the timeout is recorded as a timeout and passed. The gate therefore
  prints the timeout list on every non-clean pass, precisely so that residue stays reviewable
  rather than silent. Skim it.
- **Equivalent mutants need a policy, not a shrug.** Some survivors no test can kill because
  they don't change behaviour. Here each is classified (redundant guard → simplify it away;
  free choice → carve it out in `.cargo/mutants.toml`) and the classified list is itself
  drift-gated, so an exclusion cannot accumulate undocumented.

## The upgrade contract, for downstream consumers

A consumer of this library can run the same discipline over *its own* boundary: implement
`engine::Theory`, build the spec with `Spec::of::<MyTheory>()`, and freeze it into the
consumer's own repository with `Spec::lock_in(spec_dir)` (`Spec::lock` without a directory is
this repo's convenience — its path is baked in at this crate's compile time). What that lock
contains is a contract this library commits to:

- **What you freeze: laws + coverage, nothing else.** A lock records only facts about *your
  domain* — the header, the named laws discovery found true of your operators, and the
  coverage line (which operators no law speaks for). It never records facts about the engine.
  In particular the consequence-equality count — a property of our enumeration and sampling,
  not of your behaviour — is not in the lock.
- **What a library upgrade can do to your lock.** A new release may add universal shapes, so
  discovery may find *new* named laws over your unchanged operators — additive lock drift,
  which you re-freeze and ratify like any other diff (a pleasant one: the spec got more
  articulate about behaviour you already had). What an upgrade cannot do is drift your lock
  through engine internals alone: sampling and enumeration improvements change no lock unless
  the law set changes. If your lock drifts, the *laws* changed — which is exactly the drift
  the gate exists to put in front of you.
- **The semver policy.** A new universal shape (your lock may gain laws) is a **minor**
  release. A change to an existing shape's prose or semantics, or a removed shape (your lock
  may lose or reword laws you ratified), is a **major** release, called out in the release
  notes.

## How this repo instantiates each move

| move | generic piece | this repo's piece |
|---|---|---|
| derive | your generator | the discovery engine + `freeze::render` ([`src/discover/freeze.rs`](../src/discover/freeze.rs)) |
| freeze | [`spec-lock`](../spec-lock)'s `Lock` + `bless` | [`examples/freeze_spec.rs`](../examples/freeze_spec.rs) → [`spec/*.spec`](../spec) |
| gate | [`spec-lock`](../spec-lock)'s `check` in a test | `freeze::check_fresh`, run by `ci.yml`'s `check` job |
| mutate the diff | `cargo mutants --in-diff` + a survivor-counting gate | `mutants-diff` / `mutants-full` jobs + [`.github/mutants-gate.sh`](../.github/mutants-gate.sh) |

To lift it: depend on `spec-lock`, build a `Vec<Lock>` from whatever your project can derive
deterministically, wire `check` into a test and `bless` into a small regen binary, and copy the
two mutants jobs plus the gate script. The discipline is the four moves; the crate is just the
fifty lines you shouldn't have to rewrite.
