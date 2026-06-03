# W2 Writer Baseline Inventory

This inventory was captured before migrating the first-order Writer
helper surface into descriptor-backed generation. It deliberately
separates first-order Writer operations from scoped Writer operations:
`tell`, `fold_writer`, and `run_writer` are W2 targets; `listen`,
`censor`, and their scoped carriers remain deferred to W8.

## Captured Baselines

Commands used:

```bash
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::writer > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-writer.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::run::smart_constructors > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-run-smart_constructors.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::rc_run::smart_constructors > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-rc_run-smart_constructors.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::arc_run::smart_constructors > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-arc_run-smart_constructors.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::run_explicit::smart_constructors > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-run_explicit-smart_constructors.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::rc_run_explicit::smart_constructors > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-rc_run_explicit-smart_constructors.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::arc_run_explicit::smart_constructors > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-arc_run_explicit-smart_constructors.rs
XDG_RUNTIME_DIR=/tmp just cargo expand -p fp-library --features effects --lib types::effects::named_helpers::writer > /tmp/rust-fp-lib-w2-writer-baselines/types-effects-named_helpers-writer.rs
```

Baseline outputs:

| Module                                                 | Lines | SHA-256                                                            |
| ------------------------------------------------------ | ----: | ------------------------------------------------------------------ |
| `types::effects::writer`                               |  2447 | `874f578e628a8621ed47a42bd2fce7e5d0eb3ba653be9b4dcb9a4797dd171097` |
| `types::effects::run::smart_constructors`              |  2041 | `1c8a585f0e41931c9c9c0a6a28a6a517bef01dd64c736a605a1768bd7b1fd64d` |
| `types::effects::rc_run::smart_constructors`           |  2458 | `68debfe03c8f5f4783c2fdf639f28ba6e19bf54923e59d58be4d03c6077516b5` |
| `types::effects::arc_run::smart_constructors`          |  2768 | `debb3e6df6562611f801125fbb9dbd986019210604c594b01ef1c02c38567836` |
| `types::effects::run_explicit::smart_constructors`     |  1962 | `c2201763868d3ae46da9e313e70227aea6ea78b766e6f740713037af0d0606f2` |
| `types::effects::rc_run_explicit::smart_constructors`  |  2216 | `40831e83b9640c979594579280a85188fdcf04693cfc8d4122524cf085dca909` |
| `types::effects::arc_run_explicit::smart_constructors` |  2662 | `a071c9850da431e2edb84d872e501d630056f3d39cd3e4d5392a26fe31ce45e3` |
| `types::effects::named_helpers::writer`                |  1380 | `20e3a2f2165eeb401b723f7e9081b9c774415f06d946aca7f0b5493bd1b05f13` |

## First-Order Writer Surface

W2 migration targets:

- `tell` constructors in all six smart-constructor modules.
- `fold_writer` named helpers in all six wrapper families.
- `run_writer` named helpers in all six wrapper families.

The current first-order Writer cell in `types::effects::writer` is
captured as a baseline but is not the first migration target. That
module also owns scoped Writer carriers and pointer-brand siblings for
`listen` and `censor`, so replacing the whole module with
`define_effect! { effect Writer; }` would cross the W8 boundary.

## Deferred Scoped Surface

Deferred to W8:

- `censor` smart constructors in all six smart-constructor modules.
- `listen` smart constructors in all six smart-constructor modules.
- `WriterCensor`, `WriterListen`, `BoxWriterCensor`,
  `BoxWriterListen`, `SendWriterCensor`, and `SendWriterListen` cells
  and brands.

The W2 migration must preserve the existing scoped surface unchanged.
