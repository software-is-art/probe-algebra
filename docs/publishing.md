# Publishing to crates.io

Four crates publish; one never does. All versions move together and start at
`0.1.0`.

| crate | role | publishes |
|---|---|---|
| `spec-lock` | freeze/drift-gate mechanics (zero deps) | yes |
| `boundary-spec-macros` | derive macros (proc-macro) | yes |
| `boundary-enforce` | build.rs enforcement passes | yes |
| `boundary-spec` | the library (root crate) | yes |
| `downstream-fixture` | consumer existence proof | **never** — `publish = false`, version `0.0.0` |

## Publish order

`boundary-spec` depends on the other three (`boundary-spec-macros` and
`spec-lock` as dependencies, `boundary-enforce` as a build-dependency), so it
must go last. The first three are mutually independent — any order among them:

```
cargo publish -p spec-lock
cargo publish -p boundary-spec-macros
cargo publish -p boundary-enforce
# wait for the three to be visible in the index, then:
cargo publish -p boundary-spec
```

Every path dependency in the workspace also carries `version = "0.1.0"`, which
is what ends up in the published manifests (cargo strips the `path` keys at
package time). Do **not** use `cargo publish --workspace`-style tooling blindly;
if you do, `downstream-fixture`'s `publish = false` guarantees it can never
ship, but the dependency order above must still hold.

## Pre-publish checklist

All of CI's gates, run locally at the release commit:

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy --workspace --all-targets` clean
- [ ] `cargo build` and `cargo test --workspace` green (build.rs enforcement
      passes; unit, property, and compile-fail suites pass; the
      `downstream-fixture` consumer loop still builds and tests)
- [ ] `cargo mutants` green — no unclassified survivors (the classified
      equivalents live in `.cargo/mutants.toml` and are themselves drift-gated)
- [ ] Spec locks fresh: `spec/qualify.spec` matches the census
      (`BLESS_QUALIFY=1 cargo build` produces no diff) and the discovery locks
      (`spec/*.spec` via `examples/freeze_spec.rs`) produce no diff
- [ ] `spec/shapes.spec` ratified — the shape catalog matches what the engine
      enumerates; any diff there has semver consequences (see below)
- [ ] `cargo package --list -p <crate>` reviewed for each of the four crates.
      For `boundary-spec` in particular, `spec/` (all six `.spec` files —
      `qualify.spec` above all) **must** be in the list: build.rs re-runs the
      enforcement passes and the qualify drift gate over the packaged tree at
      the consumer's build time, and a missing `spec/qualify.spec` fails every
      downstream build. `.github/` and `.cargo/` are excluded via the manifest's
      `exclude` list; `src/`, `spec/`, `docs/`, `examples/`, `tests/` and both
      license texts ship.
- [ ] Unpacked-crate smoke test (see below) passes.

## Semver contract

The upgrade contract in [ci-discipline.md](ci-discipline.md#the-upgrade-contract-for-downstream-consumers)
is the versioning policy:

- **A new universal shape** (consumers' locks may *gain* laws — additive drift
  they re-freeze and ratify) is a **minor** release.
- **A change to an existing shape's prose or semantics, or a removed shape**
  (consumers' locks may *lose or reword* ratified laws) is a **major** release,
  called out in the release notes.
- Engine internals (sampling, enumeration) alone never drift a consumer's lock;
  if a release would, the law set changed and the rule above applies.
- **A grid-semantics change** (how `shadow_grid` composes a derived grid) is the
  subtle case of the rule above: it leaves `spec/shapes.spec` untouched but can
  still change which laws a CONSUMER discovers — a consumer whose value type is
  a derived enum under a binding cap gets a different grid, so a coincidental
  law can (correctly) vanish from their re-frozen lock. Losing laws is the
  major-release row; call the change out in the release notes so the drift a
  consumer ratifies has a named cause. The structure-first partition of
  `shadow_grid` (2026-07-03) is such a change; it lands before the first
  publish, so no shipped consumer drifts, but it sets the precedent: hand-`Shaped`
  leaf grids and hand-curated `inhabit` grids are provably unaffected by it, and
  any future release in this class must say which grid class moves.

A diff to `spec/shapes.spec` is therefore the semver signal for the LAW
LANGUAGE: additive lines → at least minor; changed or removed lines → major.
Grid-semantics changes carry the same consequences without touching that file,
so they are named in release notes by hand — the one semver signal a committed
artifact does not raise for you.

## The unpacked-crate smoke test

The root crate is unusual: its `build.rs` runs the `boundary-enforce` passes
over **the packaged tree itself** at the consumer's build time. This is cheap
and re-verifies our own source, but it means the package must be self-contained
(all of `src/`, `spec/qualify.spec`). Verify before publishing:

```sh
cargo package -p boundary-spec --no-verify --exclude-lockfile --allow-dirty
mkdir -p /tmp/crate-check && cd /tmp/crate-check
tar xzf <repo>/target/package/boundary-spec-0.1.0.crate
cd boundary-spec-0.1.0
# Before first publish the three dep crates aren't on crates.io yet, so point
# the registry names back at the local tree (delete this before judging the
# real resolution):
cat >> Cargo.toml <<'EOF'
[patch.crates-io]
boundary-spec-macros = { path = "<repo>/boundary-spec-macros" }
spec-lock = { path = "<repo>/spec-lock" }
boundary-enforce = { path = "<repo>/boundary-enforce" }
EOF
cargo check   # must finish cleanly — this exercises build.rs enforcement + qualify drift gate
```

(`--no-verify`/`--exclude-lockfile`/the patch section are only needed **before**
the dependency crates exist on crates.io. After the first publish of the three
dep crates, plain `cargo package -p boundary-spec` — with verification — is
the right test.)

This was run for 0.1.0 on 2026-07-02: 75 files packaged, `cargo check` of the
unpacked tree finished cleanly — every enforcement pass and the qualify census
drift gate passed against the packaged `spec/qualify.spec`.

## After the first publish

- **README**: the git-dependency snippet
  (`boundary-spec = { git = "https://github.com/software-is-art/probe-algebra" }`,
  near the end of `README.md`) becomes a version dependency:
  `boundary-spec = "0.1"`. Same for any `git = …` advice in `docs/` and in
  `downstream-fixture`'s manifest comments.
- **`rust-version` (MSRV)**: intentionally omitted for 0.1.0 (not verified
  against older toolchains). If you later pin one — e.g. via `cargo msrv` —
  add it to all four manifests in the same release.
- Consider tagging the release (`v0.1.0`) so the repository state behind the
  published package is addressable.
- `downstream-fixture` stays a path consumer of the workspace — it never moves
  to the registry version and never publishes.
