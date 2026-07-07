# The tour — this repo, for humans

The other docs in this repo are written densely, and agents metabolise them fine. This
page is the opposite: one idea at a time, one running example, and a marked exit after
every station. If you only read one section, read Station 1 — everything else is the
same trick applied somewhere new.

---

## Station 0 — the way you already do it

Say you write a tiny module with two functions, `and` and `or`. In a normal project,
that kicks off four more pieces of writing:

1. **Tests.** You re-state what the code does, by hand:
   `assert_eq!(and(true, false), false);` — and a dozen more like it.
2. **A CI config.** You re-state which checks run, by hand, in YAML.
3. **Docs.** You re-state the behaviour a third time, in prose.
4. **A code review.** A colleague reads your implementation and simulates it in their
   head to decide whether it does what it should.

That's four hand-made copies of one intent. They start in sync. They do not stay in
sync: the docs rot first, the CI config rots quietly, and the tests only cover the
cases someone thought to write.

This repo's premise is a single move applied to that whole list: **anything that
re-states the code can be *derived from* the code, saved to a file, and checked on
every build.** You stop writing the copies. You start *reviewing* them.

---

## Station 1 — the machine writes the spec; you review it like a PR

You write only the meaning — a small value type and its operators:

```rust,ignore
pub fn and(a: Bool, b: Bool) -> Bool { ... }
pub fn or(a: Bool, b: Bool) -> Bool { ... }
pub fn not(a: Bool) -> Bool { ... }
```

Then the discovery engine runs those functions against every combination of a small
set of inputs, tries every law shape in its catalog against the results, and writes
down **everything it could not break**. For the Boolean module in this repo, the
output starts like this — this is a real committed file,
[`spec/bridged-bool.spec`](../spec/bridged-bool.spec):

```
- not twice returns the original value.
      not(not(x)) = x
- not turns and into or.
      not((x and y)) = (not(x) or not(y))
- and gives the same result in either order.
      (x and y) = (y and x)
- and with true leaves a value unchanged.
      (true and x) = x
- and by false always gives false.
      (false and x) = false
```

Nobody typed those sentences. The machine ran `and`, `or`, `not` a few thousand times
and wrote down what held, in plain English with the equation underneath. That includes
things you might not have thought to test — De Morgan's laws showed up on their own.

Now the trick that makes it more than a party trick: **commit that file.** From then
on, every `cargo test` re-derives the spec from the live code and compares it to the
committed file, byte for byte.

- Nothing changed? Green.
- You changed behaviour, even subtly? Red — and the failure names the exact sentence
  that appeared or vanished. To make it green you rerun one command
  (`cargo run --example freeze_spec`), which rewrites the file, and **the diff lands
  in your PR in plain English**:

  ```diff
  - - and with true leaves a value unchanged.
  -       (true and x) = x
  ```

  Your reviewer doesn't simulate your implementation in their head. They read: "this
  change made `and` stop respecting `true` as an identity — did we mean that?" and
  approve or reject *that*. Around here this is called **ratifying the diff**: the
  committed diff is the review artifact.

The file is called a **lock** — like a lockfile for behaviour instead of dependency
versions. Same lifecycle you already know from `Cargo.lock` / `package-lock.json`:
generated, committed, and a surprise diff in it means something real changed.

> **Exit 1.** This station alone — derive a text file, commit it, fail the build when
> it drifts — is extractable into any project via the tiny zero-dependency
> [`spec-lock`](../spec-lock) crate. It works for anything you can print
> deterministically: an API surface, a schema dump, a config baseline, a list of
> known findings. If that's all you wanted, you're done; the rest of this page is the
> same move aimed at fancier targets.

---

## Station 2 — a to-do list instead of a wall of red

The lock keeps you where you are. Getting somewhere is the same idea, run in reverse:
declare what you *intend* the module to satisfy, before the code earns it:

```rust,ignore
expects {
    commutative(and);
    identity(and, true);
}
```

Then ask for the **distance report**:

```
bridged-bool: 1 of 2 declared laws hold; MISSING: identity(and, true)
```

It reads like a compiler error, but for behaviour: here's what you said, here's what
the code actually does, here's the gap. You (or an agent) fix the code, the report
goes green law by law, and the moment it's fully green you freeze the lock from
Station 1 and it keeps you there.

Two directions, one vocabulary: `expects` gets you TO the spec, the lock keeps you AT
it.

---

## Station 3 — who tests the tests?

A generated spec sounds great until you ask: what if it's *weak*? A spec that can't
tell good code from broken code is decoration.

The answer is mutation testing, and it's blunt: **plant a bug on purpose and see if
anything goes red.** Change `and` so it ignores its second argument. Re-derive the
spec. Did any sentence change? If yes, the bug is "killed" — the spec would have
caught it. If nothing changed, you've learned something important, stated exactly:

> there is a bug the spec cannot see.

That's called a **survivor**, and survivors are treated as findings, not noise. Each
one either gets a new law/test that kills it, or gets written into a small
hand-maintained file with a one-line justification for why it's acceptable — an
**exception register**. The build fails if a new survivor appears that isn't in the
register, and also if the register lists one that no longer exists (a stale exception
is a lie). "2 new findings, 1 resolved" is the whole review.

This repo runs mutation at two speeds — a slow, thorough one in CI (every line of
source, weekly) and an instant one inside every `cargo test` (perturb the operators
in-memory, re-run discovery, milliseconds). You don't need to remember the split;
just the principle: **the spec's power is measured, not assumed.**

---

## Station 4 — the CI config gets the same treatment (the concrete comparison)

This is where the discipline usually clicks, because everyone has lived the
hand-authored version.

**The way you know:** `.github/workflows/ci.yml` is a YAML file someone wrote two
years ago. It runs `cargo test` — but is it the same command the README tells
newcomers to run locally? Nobody's sure. There's a `--workspace` flag in one place
and not another. Someone once deleted a step in a rush and nothing noticed for a
month. The file is *configuration*: nothing checks it against anything.

**The way it works here:** the pipeline is declared once, as data, in Rust
([`src/discover/gates.rs`](../src/discover/gates.rs)) — each gate is a row with a
name, the exact command, when it runs, and a sentence saying what it promises:

```rust,ignore
Gate {
    name: "test",
    verifies: "every workspace member's suites: the enforcement passes and
               every drift gate holds",
    command: &["cargo", "test", "--workspace", "--all-targets"],
    cadence: Cadence::EveryChange,
    ...
}
```

`ci.yml` is then **rendered from that table**, committed, and drift-gated exactly
like the Boolean spec in Station 1 — the file's first line says so:

```yaml
# GENERATED from the gate registry (`discover::gates`) — THE PIPELINE IS A LOCK.
# Never edit by hand: regenerate with `cargo run --example freeze_gates` and ratify
# the diff.
```

What that buys, concretely:

| hand-authored CI | rendered-lock CI (here) |
|---|---|
| YAML edited directly; nothing cross-checks it | YAML generated from one declared table; a hand edit fails `cargo test` *inside the very workflow the YAML runs* |
| local checks and CI checks drift apart | `cargo run --example gate` runs the every-change gates from the same table CI renders from — green locally and green in CI are the same claim |
| "why does this step exist?" → archaeology | every gate carries its `verifies:` sentence; the promises are themselves a committed, reviewable file ([`spec/gates.spec`](../spec/gates.spec)) |
| deleting a step is silent | deleting a gate changes two committed files; the diff is the review |

Same trick as Station 1. The target just moved from "what the code does" to "what
the pipeline runs."

> **Exit 2.** Stations 1–4 are the whole working discipline. Day to day it feels like
> this: you change code → `cargo test` goes red naming a stale file and the one
> command that regenerates it → you rerun it → the diff in your PR *is* the review.
> The single rule: **never edit a generated file by hand.**

---

## Station 5 — the same move, further up the mountain (skim freely)

Everything past here is the identical derive-freeze-gate loop pointed at bigger
targets. One sentence each, links for the curious:

- **Systems.** Declare which modules exist and where they touch; the seam graph is
  checked and frozen, so "module A quietly grew a dependency on B's internals" is a
  red gate, not a surprise in review. ([discovery.md](discovery.md))
- **The outside world.** You can't lock a real database, but you can lock your
  *assumptions* about it — record its behaviour over a battery of commands, freeze
  the recording, replay it later; the diff names exactly where the world changed
  under you. ([discovery.md](discovery.md))
- **Generated code with a licence.** The `delta-render` crate generates incremental
  dataflow code, and each optimisation is only allowed if the discovered spec of the
  operator *proves it's licensed* (linear operators get the cheap rule, others get
  the safe slow one) — discovery output consumed as an input to code generation.
  ([delta-render](../delta-render))
- **A proof assistant in the loop.** A small Lean file proves theorems about the
  Boolean module; discovery cross-checks the proofs' exported tables (a disagreement
  is a defect *somewhere*, guaranteed), supplies conjectures worth proving next, and
  a mutation pass flips definitions to check that the theorems actually constrain
  them. Same loop, kernel-grade referee. ([roadmap.md](roadmap.md), items 10–12)

---

## The fine print, honestly

Discovery **refutes; it never proves.** "Discovered law" means "a bounded set of
inputs could not break it" — not a theorem. That's the same epistemic deal ordinary
tests offer, just with far better coverage per keystroke, and the docs here are
careful to keep the distinction: reports *suggest*, locks *gate*, and the one thing
the machinery can never do is the human act of reading a diff and deciding it's what
you meant.

## Where next

- Want to adopt the smallest piece today → [`spec-lock`](../spec-lock)'s README.
- Want the full method, precisely → [discovery.md](discovery.md).
- Want the story of how it grew, brick by brick → [roadmap.md](roadmap.md).
- Working on this repo itself → [CLAUDE.md](../CLAUDE.md), the one-page discipline.
