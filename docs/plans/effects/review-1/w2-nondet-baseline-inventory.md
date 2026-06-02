# W2 NonDet Baseline Inventory

This document records the pre-migration `Empty` / `Choose` / `NonDet`
expansion baselines and current first-order API surface for the W2
descriptor-backed `NonDet` migration. The generated expansion files were
captured under `/tmp/rust-fp-lib-w2-nondet-baselines` from commit
`a317ee22`; they are not checked in because they are generated artifacts.
The line counts and hashes below provide a durable record of the captured
baseline.

Command shape used for each module:

```bash
just cargo expand -p fp-library --features effects --lib <module>
```

The local capture used `XDG_RUNTIME_DIR=/tmp` with the same command
shape so `just` could create temporary files in the sandbox. This does
not affect the expanded Rust output.

## Expansion Baselines

| Module                                                 | Temporary file                                         | Lines | SHA-256                                                            |
| ------------------------------------------------------ | ------------------------------------------------------ | ----: | ------------------------------------------------------------------ |
| `types::effects::empty`                                | `types-effects-empty.rs`                               |   301 | `22f5be128265a08b75aef449ee44582ea80d51a927746f10b375c23e37502ef9` |
| `types::effects::choose`                               | `types-effects-choose.rs`                              |   492 | `9c51dae5724fdee8ec1133dbf2e457b9520a3d0d548ab8480c348d25d90fddfa` |
| `types::effects::run::smart_constructors`              | `types-effects-run-smart_constructors.rs`              |  2041 | `1c8a585f0e41931c9c9c0a6a28a6a517bef01dd64c736a605a1768bd7b1fd64d` |
| `types::effects::rc_run::smart_constructors`           | `types-effects-rc_run-smart_constructors.rs`           |  2458 | `68debfe03c8f5f4783c2fdf639f28ba6e19bf54923e59d58be4d03c6077516b5` |
| `types::effects::arc_run::smart_constructors`          | `types-effects-arc_run-smart_constructors.rs`          |  2768 | `debb3e6df6562611f801125fbb9dbd986019210604c594b01ef1c02c38567836` |
| `types::effects::run_explicit::smart_constructors`     | `types-effects-run_explicit-smart_constructors.rs`     |  1962 | `c2201763868d3ae46da9e313e70227aea6ea78b766e6f740713037af0d0606f2` |
| `types::effects::rc_run_explicit::smart_constructors`  | `types-effects-rc_run_explicit-smart_constructors.rs`  |  2216 | `40831e83b9640c979594579280a85188fdcf04693cfc8d4122524cf085dca909` |
| `types::effects::arc_run_explicit::smart_constructors` | `types-effects-arc_run_explicit-smart_constructors.rs` |  2662 | `a071c9850da431e2edb84d872e501d630056f3d39cd3e4d5392a26fe31ce45e3` |
| `types::effects::named_helpers::nondet`                | `types-effects-named_helpers-nondet.rs`                |  2619 | `286c53148fd7e95ac8d2c2137076af3baced19bd4bb50685cd89201d8d7e726e` |

Total captured expansion size: 17,519 lines.

## Effect Cell Surface

`Empty<'a, A>` is a first-order abort effect with no successful branch.
The current effect module provides:

- `impl_kind!` for `EmptyBrand`.
- `Clone`, `Copy`, and `Default` for `Empty<'a, A>`.
- `Functor for EmptyBrand`.
- `SendFunctor for EmptyBrand`.

`Choose<'a, P, A>` is the local multi-shot choice cell. It stores two
continuations:

```rust
Left(P::Pointer<dyn Fn(A) -> A + 'a>),
Right(P::Pointer<dyn Fn(A) -> A + 'a>),
```

The current effect module provides:

- `impl_kind!` for `ChooseBrand<P>`.
- `Clone for Choose<'a, P, A>` when the pointer type for
  `dyn Fn(A) -> A + 'a` is `Clone`.
- `Functor for ChooseBrand<P>`.

`SendChoose<'a, P, A>` is the thread-safe multi-shot choice cell. It
stores two `Send + Sync` continuations:

```rust
Left(P::Pointer<dyn Fn(A) -> A + Send + Sync + 'a>),
Right(P::Pointer<dyn Fn(A) -> A + Send + Sync + 'a>),
```

The current effect module provides:

- `impl_kind!` for `SendChooseBrand<P>`.
- `Clone for SendChoose<'a, P, A>` when
  `P::Pointer<dyn Fn(A) -> A + Send + Sync + 'a>: Clone`.
- `SendFunctor for SendChooseBrand<P>`.

`BoxChoose<'a, P, A>` is the single-shot boxed choice cell:

```rust
Left(P::Pointer<dyn FnOnce(A) -> A + 'a>),
Right(P::Pointer<dyn FnOnce(A) -> A + 'a>),
```

The current effect module provides:

- `impl_kind!` for `BoxChooseBrand<P>`.
- `Functor for BoxChooseBrand<BoxBrand>`.

`BoxChoose` is not cloneable and is not currently exposed through the
public `choose` helper surface. The descriptor migration should preserve
that distinction unless W8's scoped-dispatch work makes a broader
single-shot `Choose` story explicit.

## Smart Constructor Surface

Each wrapper has an inherent `empty` constructor. All construct
`Empty::Empty(PhantomData)` and lift it through the wrapper-specific row
member type.

| Wrapper          | `empty` | `choose` |
| ---------------- | ------- | -------- |
| `Run`            | Yes     | No       |
| `RcRun`          | Yes     | Yes      |
| `ArcRun`         | Yes     | Yes      |
| `RunExplicit`    | Yes     | No       |
| `RcRunExplicit`  | Yes     | Yes      |
| `ArcRunExplicit` | Yes     | Yes      |

The `choose` constructor is intentionally multi-shot-only today. It is
available for `RcRun`, `ArcRun`, `RcRunExplicit`, and
`ArcRunExplicit`. The descriptor model should encode that capability rule
directly instead of hard-coding wrapper-name exceptions inside generated
method bodies.

## Named Helper Surface

`named_helpers::nondet` provides `run_empty` for all six wrappers:

| Wrapper          | Helper      | Source file                         |
| ---------------- | ----------- | ----------------------------------- |
| `Run`            | `run_empty` | `named_helpers/nondet.rs` line 159  |
| `RcRun`          | `run_empty` | `named_helpers/nondet.rs` line 214  |
| `ArcRun`         | `run_empty` | `named_helpers/nondet.rs` line 617  |
| `RunExplicit`    | `run_empty` | `named_helpers/nondet.rs` line 1081 |
| `RcRunExplicit`  | `run_empty` | `named_helpers/nondet.rs` line 1143 |
| `ArcRunExplicit` | `run_empty` | `named_helpers/nondet.rs` line 1577 |

The multi-shot helpers are available only for `RcRun`, `ArcRun`,
`RcRunExplicit`, and `ArcRunExplicit`:

| Wrapper          | `run_choose` | `run_nondet` | `run_first_success` |
| ---------------- | ------------ | ------------ | ------------------- |
| `Run`            | No           | No           | No                  |
| `RcRun`          | Yes          | Yes          | Yes                 |
| `ArcRun`         | Yes          | Yes          | Yes                 |
| `RunExplicit`    | No           | No           | No                  |
| `RcRunExplicit`  | Yes          | Yes          | Yes                 |
| `ArcRunExplicit` | Yes          | Yes          | Yes                 |

Their current source locations are:

| Wrapper          | `run_choose` line | `run_nondet` line | `run_first_success` line |
| ---------------- | ----------------: | ----------------: | -----------------------: |
| `RcRun`          |               273 |               347 |                      483 |
| `ArcRun`         |               681 |               771 |                      926 |
| `RcRunExplicit`  |              1209 |              1294 |                     1436 |
| `ArcRunExplicit` |              1685 |              1821 |                     2006 |

`run_empty` removes `EmptyBrand` from the first-order row and maps the
unreachable `Empty::Empty(_)` branch through an empty match.
`run_choose` removes `ChooseBrand` or `SendChooseBrand` from the
first-order row and handles the selected branch by invoking the stored
continuation. `run_nondet` composes the current `Empty` and `Choose`
interpreters. `run_first_success` implements the existing first-success
policy in terms of the same multi-shot choice capability.

## Acceptance Criteria

- The descriptor-backed effect cells and helper methods must either
  match the captured expansion hashes exactly or document intentional
  rustfmt-only or ordering-only differences before the migration is
  broadened.
- The descriptor model must encode the multi-shot-only `Choose`
  capability as a typed wrapper capability, not as method-body
  `if wrapper == ...` special cases.
- `BoxChoose` remains an internal single-shot cell until a later design
  step deliberately exposes or removes it.
