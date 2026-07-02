# spec-lock

Freeze a deterministically derived spec artifact into a committed file, and gate
CI on drift — so the PR diff is the ratification. Zero dependencies.

Extracted from [`boundary-algebra`](https://crates.io/crates/boundary-algebra) so
any project can lift the "frozen spec + drift gate" discipline without the
discovery engine. Full documentation and the CI discipline it enables live in
the main repository: <https://github.com/software-is-art/probe-algebra> (see
`docs/ci-discipline.md`).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
