# system spec: relay-app — the seam graph (modules + seam obligations); regenerate via this repo's freeze path and ratify the diff.

modules (the ratified registry — one committed module lock each):
- mixer
- gauge

seams (each edge: its obligation, then the verdict its checker returned):
- mixer -- gauge : transform on Signal
      obligation: the conversion across the seam must be a homomorphism
      status: preserved — the conversion `cook` is a discovered homomorphism (spanning theory: mixer-gauge seam):
        * cook turns blend into fuse.
