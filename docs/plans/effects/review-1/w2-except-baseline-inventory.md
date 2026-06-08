# W2 Except Baseline Inventory

This document records the pre-migration `Except` expansion baselines and
current first-order API surface for the W2 descriptor-backed `Except`
migration. The generated expansion files were captured under
`/tmp/rust-fp-lib-w2-except-baselines` from commit `a44e5752`; they are
not checked in because they are generated artifacts. The line counts and
hashes below provide a durable record of the captured baseline.

Command shape used for each module:

```bash
just cargo expand -p fp-library --features effects --lib <module>
```

If `just` cannot create a temporary directory under the default runtime
directory in the local sandbox, set `XDG_RUNTIME_DIR=/tmp` for the same
command. This does not affect the expanded Rust output.

## Expansion Baselines

| Module                                                 | Temporary file                                         | Lines | SHA-256                                                            |
| ------------------------------------------------------ | ------------------------------------------------------ | ----: | ------------------------------------------------------------------ |
| `types::effects::except`                               | `types-effects-except.rs`                              |   237 | `db2688aef4664ffd4018bfbdddd550c911798cde0394809014eaffdf97315407` |
| `types::effects::run::smart_constructors`              | `types-effects-run-smart_constructors.rs`              |  2041 | `1c8a585f0e41931c9c9c0a6a28a6a517bef01dd64c736a605a1768bd7b1fd64d` |
| `types::effects::rc_run::smart_constructors`           | `types-effects-rc_run-smart_constructors.rs`           |  2458 | `68debfe03c8f5f4783c2fdf639f28ba6e19bf54923e59d58be4d03c6077516b5` |
| `types::effects::arc_run::smart_constructors`          | `types-effects-arc_run-smart_constructors.rs`          |  2768 | `debb3e6df6562611f801125fbb9dbd986019210604c594b01ef1c02c38567836` |
| `types::effects::run_explicit::smart_constructors`     | `types-effects-run_explicit-smart_constructors.rs`     |  1962 | `c2201763868d3ae46da9e313e70227aea6ea78b766e6f740713037af0d0606f2` |
| `types::effects::rc_run_explicit::smart_constructors`  | `types-effects-rc_run_explicit-smart_constructors.rs`  |  2216 | `40831e83b9640c979594579280a85188fdcf04693cfc8d4122524cf085dca909` |
| `types::effects::arc_run_explicit::smart_constructors` | `types-effects-arc_run_explicit-smart_constructors.rs` |  2662 | `a071c9850da431e2edb84d872e501d630056f3d39cd3e4d5392a26fe31ce45e3` |
| `types::effects::named_helpers::except`                | `types-effects-named_helpers-except.rs`                |  1950 | `fde33e15dd651c127e580ccf5ad06776dec1233884eab3438e807da79fc98ff5` |

Total captured expansion size: 16,294 lines.

## Effect Cell Surface

`Except<'a, E, A>` is a first-order typed abort effect. It has one
variant:

```rust
Throw(E, PhantomData<&'a A>)
```

The current effect module provides:

- `impl_kind!` for `ExceptBrand<E>` with `E: 'static`.
- `Clone for Except<'a, E, A>` when `E: Clone`.
- `Functor for ExceptBrand<E>` when `E: 'static`.
- `SendFunctor for ExceptBrand<E>` when `E: Send + Sync + 'static`.

There is no pointer-brand sibling for `Except`; the same
`ExceptBrand<E>` serves all six wrappers because the effect carries no
continuation.

## Smart Constructor Surface

Each wrapper has an inherent `throw` constructor. All construct
`Except::Throw(e, PhantomData)` and lift it through the wrapper-specific
row member type.

| Wrapper          | Source file                              | Result lifetime | Error bound                                | Additional notable bounds                                                                                                            |
| ---------------- | ---------------------------------------- | --------------- | ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ |
| `Run`            | `run/smart_constructors.rs`              | `'static`       | `ErrorType: 'static`                       | `R` and `ScopedRow` use `WrapDrop + Functor + 'static`.                                                                              |
| `RcRun`          | `rc_run/smart_constructors.rs`           | `'static`       | `ErrorType: Clone + 'static`               | `A: Clone + 'static`; lifted node projection must be `Clone`.                                                                        |
| `ArcRun`         | `arc_run/smart_constructors.rs`          | `'static`       | `ErrorType: Clone + Send + Sync + 'static` | `A: Send + Sync`; `NodeBrand<R, ScopedRow>: SendFunctor`; lifted node projection must be `Clone`.                                    |
| `RunExplicit`    | `run_explicit/smart_constructors.rs`     | `'a`            | `ErrorType: 'static`                       | `A: 'static`; row membership uses `Coyoneda<'a, ExceptBrand<ErrorType>, A>`.                                                         |
| `RcRunExplicit`  | `rc_run_explicit/smart_constructors.rs`  | `'a`            | `ErrorType: Clone + 'static`               | `A: Clone + 'static`; row membership uses `RcCoyoneda<'a, ExceptBrand<ErrorType>, A>`.                                               |
| `ArcRunExplicit` | `arc_run_explicit/smart_constructors.rs` | `'a`            | `ErrorType: Clone + Send + Sync + 'static` | `A: Clone + Send + Sync + 'static`; row, scoped-row, and node projections must satisfy the current `Send + Sync` and `Clone` bounds. |

## Named Helper Surface

`named_helpers::except` provides convenience constructors and runners for
all six wrappers:

- `throw_unit<Idx>() -> Self`
- `rethrow<ErrorType, Idx>(result: Result<A, ErrorType>) -> Self`
- `note<ErrorType, Idx>(error: ErrorType, value: Option<A>) -> Self`
- `from_option<Idx>(value: Option<A>) -> Self`
- `run_except<ErrorType, Idx, RMinusExcept>(self) -> Wrapper<RMinusExcept, CNilBrand, Result<A, ErrorType>>`

The convenience helpers delegate through `throw` and `pure` shapes today
and should continue to do so after generation. `run_except` maps the
program result through `Ok`, handles `ExceptBrand<ErrorType>`, and turns
`Except::Throw(error, _)` into `pure(Err(error))` for the same wrapper
family.

The current named-helper wrapper order in the source is:

1. `Run`
2. `RunExplicit`
3. `RcRunExplicit`
4. `ArcRunExplicit`
5. `ArcRun`
6. `RcRun`

The generator does not need to preserve source order for semantic
correctness, but expansion comparison should document any rustfmt-only
or ordering-only movement before broadening the migration.

## Scoped Exclusion

Scoped `Catch` remains outside this W2 migration. `Catch` is a scoped
effect with handler-boundary semantics, not part of the first-order
typed-abort `Except` descriptor slice.
