# shape spec: boundary-spec — the DERIVED shape (operators placed by net connectivity); regenerate via this repo's freeze path and ratify the diff.

Two operators share a net when a sort appears in both signatures (the circuit-CAD
signal: nets, not laws). A settled module is one whose declared boundary the placer
re-derives; a module placing as several holds operators that share NOTHING — move
the code, never pin the report.

- interpreter arithmetic: settled — { 0, 1, false, +, *, < } over nets { Bool, Int }
- router: settled — { empty, or } over nets { Router }
- date calculus: settled — { zero, +, add, diff, since, at } over nets { Date, Duration }
- ttl store: settled — { empty, <+, tick, zero, + } over nets { Duration, Store }
- store protocol: settled — { empty, ++ } over nets { P }
- doc flow: settled — { submit, revise, approve, edit } over nets { Draft, Published, Review }

verdict: 6 of 6 modules settled — the declared shape is a fixed point of the placer.

seam candidates (cross-module net-NAME coincidences no declared seam covers — a
suggestion: declare the seam, or leave the shared name standing as coincidence):
- date calculus ↔ ttl store on Duration
