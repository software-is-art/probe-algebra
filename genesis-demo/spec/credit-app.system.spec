# system spec: credit-app — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.

modules (the ratified registry — one committed module lock each):
- meter
- billing

seams (each edge: its obligation, then the verdict its checker returned):
- meter -- billing : transport on Credits
      obligation: the modules share this value and must agree on its laws
      status: discharged by construction — the shared value is one type on both sides (the declaration carries the compile-time witness)
