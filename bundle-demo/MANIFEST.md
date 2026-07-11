# bundle-demo — the birth record

This member's `src` was grown ENTIRELY through the continuation verbs — the CLI as the
interface to the code, no text ever written into an open file. The one hand-made part is
the shell (this file, `Cargo.toml`, `lib.rs`, the tests and freeze example): the birth
act, genesis's residue, minted once. Everything inside `src/tally*.rs` is verb output.

The commands, exactly as run (each one a judged transaction — refusals write nothing):

```sh
# grow the module, item by item — each add lands canonically placed:
cargo run --example bundle -- add bundle-demo/src/tally.rs <the module doc + Tally enum>
cargo run --example bundle -- add bundle-demo/src/tally.rs <fn merge>
cargo run --example bundle -- add bundle-demo/src/tally.rs <fn floor>
cargo run --example bundle -- add bundle-demo/src/tally.rs <fn bump>

# declare the contract and derive the zero-annotation lift in one act:
cargo run --example bundle -- lift bundle-demo/src/tally.rs tally \
    'commutative(merge)' 'associative(merge)' 'idempotent(merge)' 'identity(merge, floor)' \
    > bundle-demo/src/tally_lift.rs

# wire the derived lift into the module (an add like any other):
cargo run --example bundle -- add bundle-demo/src/tally.rs <include!("tally_lift.rs");>

# canonicality confirmed by the judge, not by eyeballing:
cargo run --example bundle -- check bundle-demo/src/tally.rs   # => canonical

# freeze what discovery found:
cargo run -p bundle-demo --example freeze
```

What the gates hold (tests/contract.rs): the declared contract is MET (`Distance` over the
lifted theory — the red/green gate; its RED arm is drilled in
`boundary-spec/src/discover/lift.rs`, where an overshooting declaration reads unmet BY
NAME); the committed lift is BYTE-FOR-BYTE the scan's output (derived, never transcribed);
the module is canonically placed (the round-trip pin); and the frozen locks are fresh and
sensitivity-swept.

Honest frame, per house rules: this proves the continuation loop at the MODULE level —
grown, declared, judged, locked, with the CLI as the only interface. The flagship story's
SYSTEM level (seams, transports, the two-lifecycle red commit) is not reproduced here, so
the genesis emitter's retirement criterion is only partly met; the remaining half is
recorded in docs/roadmap.md.
