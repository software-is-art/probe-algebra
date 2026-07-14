# weave-knee — the knee, derived (regenerate: `cargo run -p weave-knee --example knee -- freeze`)
# relation floor: 0.9 — a fanout is WOVEN while mean relation coverage holds this line

## weaver: claude-sonnet-5 — judge: claude-sonnet-5
fanout  trials  mention  claims  relations  foil-rej  quality
     2       2    1.000   1.000      1.000     1.000      4.0
     3       2    1.000   1.000      1.000     1.000      3.5
     4       2    1.000   1.000      1.000     1.000      4.0
     5       2    1.000   1.000      1.000     1.000      4.0
     6       2    1.000   1.000      1.000     1.000      3.5
     8       2    1.000   1.000      1.000     1.000      3.0
    10       2    1.000   1.000      1.000     1.000      3.0
    12       2    1.000   1.000      1.000     1.000      3.0
knee: >= 12 — the floor held through the whole sweep; a lower bound, keep sweeping

# honest frame: a sweep is a sample and the judge is a model — the knee is evidence, never proof.
