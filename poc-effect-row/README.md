# POC: effect-row canonicalisation hybrid

Standalone POC for the workaround 1 + workaround 3 hybrid described in
[`docs/plans/effects/port-plan.md`](../docs/plans/effects/port-plan.md)
section 4.1, "Ordering mitigations". Findings are written up in
[`docs/plans/effects/poc-effect-row-canonicalisation.md`](../docs/plans/effects/poc-effect-row-canonicalisation.md).

## What this proves

Workaround 1 (proc-macro lexically sorts effect names so two orderings
of the same effect set produce the same canonical type) plus workaround
3 (frunk's `CoproductSubsetter` mediates hand-written non-canonical
rows into the canonical form) compose without conflict. Thirteen tests
in [`tests/feasibility.rs`](tests/feasibility.rs) cover canonicalisation
of two-, three-, five-, and seven-effect rows; generic and
lifetime-parameterised effects; the `CoproductSubsetter` fallback path;
and the empty / singleton edge cases.

## Layout

- [`macros/`](macros/): proc-macro crate exposing `effects!`. ~30 lines
  of implementation.
- [`src/lib.rs`](src/lib.rs): re-exports for tests.
- [`tests/feasibility.rs`](tests/feasibility.rs): the test suite.

## Running

This is a standalone Cargo workspace, separate from the parent
`rust-fp-lib` workspace. Run from inside this directory under the
project's nix devenv:

```sh
cd poc-effect-row
direnv allow && eval "$(direnv export bash)"
cargo test
```

Expected output: `13 passed; 0 failed`.

## Status

Feasibility verdict: hybrid is feasible on stable Rust 1.94.1; no
port-plan edits required. Detailed findings in the docs link above.

This POC code can be deleted once its tests migrate to the production
crate during the port implementation.
