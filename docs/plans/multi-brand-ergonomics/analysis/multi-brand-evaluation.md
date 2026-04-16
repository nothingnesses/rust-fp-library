# Multi-Brand Inference: Correctness and Design Evaluation

## Context

The brand inference system (see `fp-library/docs/brand-inference.md`) adds a
reverse mapping `concrete type -> brand` via the `InferableBrand_{hash}` trait,
so that free functions like `map(f, fa)` can resolve the brand without a
turbofish.

For some concrete types, multiple arity-1 brands are available. The current
design refuses to generate `InferableBrand` impls in that situation, forcing
callers to use `explicit::map::<SomeBrand, _, _, _, _>(...)`. The UI test
`fp-library/tests/ui/result_no_inferable_brand.rs` locks in this behavior with
a `trybuild` compile-fail assertion.

This document evaluates the correctness of that design and discusses
alternatives.

## Affected types

The following types carry `#[no_inferable_brand]` on the relevant
`impl_kind!` invocations:

| Concrete type      | Competing arity-1 brands                                                |
| ------------------ | ----------------------------------------------------------------------- |
| `Result<A, E>`     | `ResultErrAppliedBrand<E>`, `ResultOkAppliedBrand<A>`                   |
| `(A, B)`           | `Tuple2FirstAppliedBrand<A>`, `Tuple2SecondAppliedBrand<B>`             |
| `Pair<A, B>`       | `PairFirstAppliedBrand<A>`, `PairSecondAppliedBrand<B>`                 |
| `ControlFlow<B,C>` | `ControlFlowContinueAppliedBrand<C>`, `ControlFlowBreakAppliedBrand<B>` |
| `TryThunk<A, E>`   | `TryThunkErrAppliedBrand<E>` and the A-fixed variant                    |

`Result` is additionally reachable through `BifunctorFirstAppliedBrand<ResultBrand, A>`
and `BifunctorSecondAppliedBrand<ResultBrand, B>`, but these are auto-skipped
by the projection rule in `impl_kind.rs` because their `Of` contains `Apply!`.

Multi-brand ambiguity is arity-specific. `ResultBrand` at arity 2 is unique,
so `bimap` with inference still works for `Result`.

## Correctness

The implementation is:

- **Sound.** It never picks a wrong brand. If a type has multiple arity-1
  brands, inference fails with a compile error rather than silently selecting
  one.
- **Complete** with respect to its stated scope. Every concrete multi-brand
  type in the library marks the relevant `impl_kind!` sites with
  `#[no_inferable_brand]`, and the projection auto-skip rule handles the
  bifunctor-derived brands.
- **Internally consistent.** Arity-2 inference still works for multi-brand
  types (e.g. `bimap` on `Result`), because the ambiguity is arity-specific.
- **Well-diagnosed.** The error uses `#[diagnostic::on_unimplemented]` to
  emit a targeted message rather than a generic "trait not implemented"
  error, and names the escape hatch (`explicit::` with turbofish).

## Issues

None of these are bugs; they are points of friction or API smell worth naming.

1. **Ergonomic asymmetry with Haskell intuition.** Users familiar with
   Haskell expect `fmap f (Right 5)` to just work, mapping over the `Ok` side.
   In this library they get a compile error instead. The error is clear, but
   it's friction that Haskell users will not anticipate.

2. **Four arity-1 brands for `Result`.** `ResultErrAppliedBrand<E>` and
   `BifunctorSecondAppliedBrand<ResultBrand, E>` both functor over the `Ok`
   side; `ResultOkAppliedBrand<A>` and `BifunctorFirstAppliedBrand<ResultBrand, A>`
   both functor over the `Err` side. The bifunctor-derived ones go through
   `Bifunctor::bimap(identity, f, _)`, while the Applied variants have direct
   `Functor` impls. Semantically redundant, though the projection auto-skip
   keeps them out of inference.

3. **The projection-skip heuristic is syntactic.** `impl_kind.rs:197-204`
   skips `InferableBrand` generation when the `Of` target contains `Apply!` or
   `::`. This works today because projection cases happen to use that syntax,
   but the rule is coupled to the surface form of the macro input, not to
   what `Of` actually resolves to. A contrived type alias resolving to
   another brand's `Of` could slip past the rule. No current code does this,
   but it is a trap for future contributors extending the macro.

4. **The diagnostic does not list candidate brands.** The error tells the
   user to "use the `explicit::` variant with a turbofish" but does not name
   which brands would satisfy the call. `on_unimplemented` is static, so
   making this dynamic would require a different mechanism.

## Alternative approaches

### 1. Canonical "default" brand per type

Designate one brand per concrete type as the primary for inference
purposes (for example, `ResultErrAppliedBrand<E>` for `Result`, matching
Haskell's `Functor (Either e)`). Generate `InferableBrand` only for that
brand; require `explicit::` for the flipped direction. Could be driven by
a `#[primary_brand]` attribute so the choice is visible at the declaration
site.

- Pros: Matches Haskell ergonomics for the common case; removes friction
  for the default usage pattern; the choice is explicit and reviewable.
- Cons: The choice of "primary" is somewhat arbitrary; silently picks a
  direction for users who may not know there is another option;
  partially undermines the "both brands equally first-class" story.

### 2. Newtype disambiguation

Wrap one side so that each concrete type maps to exactly one brand
(`Of<'a, A> = FlipResult<Result<_, A>>` or similar).

- Pros: Restores the uniqueness invariant; conceptually classical and
  well-understood.
- Cons: Users must wrap and unwrap their values at the boundary; fights
  the library's "pass your normal types in, get normal types out" design.
  Strictly worse for this codebase's stated goals.

### 3. Concrete named helpers

Provide `map_ok`, `map_err`, `map_fst`, `map_snd`, and equivalents, which
skip the brand machinery entirely.

- Pros: Maximally clear at the call site; no brand picking required;
  additive, so it can coexist with the current design.
- Cons: Bypasses the HKT abstraction for these cases. Would duplicate the
  surface area that brand-generic functions already cover.

### 4. Richer diagnostics listing candidates

Keep the current refusal semantics but improve the error to list the
candidate brands that would accept the call.

- Pros: Minimum disruption; directly helps users choose the right
  `explicit::` turbofish; preserves the "no silent choice" invariant.
- Cons: `#[diagnostic::on_unimplemented]` is static, so enumerating
  candidates would require something else. A procedural macro could
  generate a per-type sealed helper trait whose name or impls encode the
  candidates and whose unimplemented message lists them. Mechanism is
  more complex than the current attribute.

### 5. Closure-directed inference

Disambiguation comes from the **closure's input type**. Given
`map(|x: i32| ..., Ok::<i32, String>(5))`, the closure input pins the
polymorphic slot, which in turn selects the brand whose `Of<'a, A>` lines
up with the container's concrete type:

- `ResultErrAppliedBrand<E>::Of<'a, A> = Result<A, E>`. With `A = i32`
  this is `Result<i32, ?>`, which unifies with `Result<i32, String>` when
  `E = String`. Match.
- `ResultOkAppliedBrand<T>::Of<'a, A> = Result<T, A>`. With `A = i32`
  this is `Result<?, i32>`, which does not unify with `Result<i32, String>`.
  No match.

Sketch (mirrors the existing Val/Ref dispatch pattern in
`fp-library/src/dispatch/`):

```rust
trait Slot<Brand, A> {}
impl<A, E> Slot<ResultErrAppliedBrand<E>, A> for Result<A, E> {}
impl<A, T> Slot<ResultOkAppliedBrand<T>, A> for Result<T, A> {}

fn map<FA, A, B, FB, Brand>(f: impl Fn(A) -> B, fa: FA) -> FB
where
    FA: Slot<Brand, A>,
    FB: Slot<Brand, B>,
    Brand: Functor,
```

The two `Slot` impls are not overlapping for coherence purposes because
their `Brand` parameter differs, so this pattern is plausible on stable
rustc 1.93.1.

**Diagonal failure case.** When both container type parameters are equal
and the closure input matches, both impls apply and inference is again
ambiguous:

- `Ok::<i32, i32>(5)` with `|x: i32| ...`: both `ResultErrAppliedBrand<i32>`
  (from impl 1 with `E = i32`) and `ResultOkAppliedBrand<i32>` (from impl 2
  with `T = i32`) satisfy `Slot<Brand, i32>` on `Result<i32, i32>`.
  Rust's trait selection reports "type annotations needed" or "multiple
  impls apply."

Analogous diagonal failures: `(T, T)`, `Pair<T, T>`, `ControlFlow<T, T>`,
`TryThunk<T, T>` (all with a closure consuming `T`). In these cases there
is a genuine semantic question the compiler cannot resolve from types
alone.

- Pros: Works without user intervention for the non-diagonal cases
  (probably the common case in practice). Strictly smaller failure set
  than today's blanket refusal. No unstable features required.
- Cons: Closure input must be nameable from context (`|x: i32| ...`
  works; `|x| ...` does not, because Rust has no basis to pick `A`).
  Diagonal `T = T` cases still require `explicit::`. Out-of-the-box error
  message for the diagonal case is worse than today's hand-tuned
  `on_unimplemented` message; recoverable, but requires additional
  machinery (e.g., a marker trait carrying an `on_unimplemented` hint for
  the overlap case).

### 6. Type-only priority without closure help

Pick a brand looking only at the container type, without using the
closure. For `Result<i32, String>` both `ResultErrAppliedBrand<String>`
(with `A = i32`) and `ResultOkAppliedBrand<i32>` (with `A = String`) are
valid, so a tiebreak rule is required.

To teach Rust to prefer one impl over another without a closure-based
signal, the options are:

- `#![feature(specialization)]` or `#![feature(min_specialization)]`.
  Still unstable.
- `#![feature(negative_impls)]`. Still unstable.
- Pick the primary brand at definition time and don't try to do this at
  use time at all. This is just alternative 1 in different clothing.

On stable rustc 1.93.1, alternative 6 as a mechanism distinct from
alternative 1 is not achievable. It either requires unstable features or
collapses into alternative 1.

## Stability summary (rustc 1.93.1)

| Alternative                           | Stable? | Notes                                                 |
| ------------------------------------- | ------- | ----------------------------------------------------- |
| 1. Canonical primary brand            | Yes     | Pure macro/attribute change.                          |
| 2. Newtype disambiguation             | Yes     | Doable but against design goals.                      |
| 3. Concrete named helpers             | Yes     | Additive, no trait-system trickery.                   |
| 4. Richer diagnostics                 | Yes     | Requires a helper marker trait; no unstable features. |
| 5. Closure-directed inference         | Yes     | Works except on diagonal `T = T` cases.               |
| 6. Type-only priority without closure | No      | Needs specialization or negative impls.               |

## Industry precedent

PureScript and Haskell face the same structural question (how does a
user map over the non-canonical side of `Either` / `Result`?) and both
converged on named functions: `map` / `<$>` maps over `Right` via the
canonical `Functor` instance, and `lmap` / `first` (from `Bifunctor`)
maps over `Left`. Neither language attempts closure-directed inference
or multiple competing `Functor` instances for the same type
constructor. For example, `Data.Bifunctor` in `purescript-bifunctors`
defines:

```purescript
lmap :: forall f a b c. Bifunctor f => (a -> b) -> f a c -> f b c
lmap f = bimap f identity
```

This convergence is strong evidence that named helpers (alternative 3)
are the industry-standard answer. The library's existing brand
machinery can combine this with inference for the primary direction
(alternative 1) to match the PureScript `map` / `lmap` ergonomics
directly.

## Recommendation

A feasibility POC
([closure_directed_inference_poc.rs](../../../../fp-library/tests/closure_directed_inference_poc.rs))
confirmed that Rust's stable trait selection can perform closure-directed
brand inference (alternative 5), but comparison with PureScript and
Haskell and examination of the diagonal failure case led to the
conclusion that named helpers (alternative 3) are the stronger design.

The recommended combined approach, in implementation order:

1. **Alternative 3 (concrete helpers).** `map_ok`, `map_err`, `map_fst`,
   `map_snd`, `map_break`, `map_continue` cover both directions of every
   multi-brand type. Additive, no HKT machinery touched, matches stdlib
   (`Result::map_err`) and PureScript (`map` / `lmap`) conventions.
2. **Alternative 1 (opt-in primary brand).** For types with a canonical
   primary direction (`Result`, `Pair`, `Tuple2`, `TryThunk`), designate
   the primary brand and generate `InferableBrand` for it. Treat
   `#[primary_brand]` as optional and documentary rather than mandatory:
   types without a canonical direction (e.g., `ControlFlow`) keep all
   brands marked `#[no_inferable_brand]`. Two strategies coexist:
   - Strategy A: one brand has `#[primary_brand]` (or default), siblings
     have `#[no_inferable_brand]`. `map(f, value)` works.
   - Strategy B: all brands have `#[no_inferable_brand]`. `map` refuses;
     users must use named helpers from layer 1 or `explicit::`.
3. **Alternative 4 revised (targeted diagnostics).** For Strategy B
   types, extend the `#[diagnostic::on_unimplemented]` message on
   `InferableBrand` to name the relevant helpers (e.g., "use
   `map_break` or `map_continue`") rather than only pointing at
   `explicit::`.

Alternative 2 (newtype wrappers) conflicts with the library's design
principles and is not recommended.

Alternative 5 (closure-directed inference) is feasible on stable but
not worth pursuing as the primary multi-brand solution. The POC
demonstrates that the `Slot<Brand, A>` pattern works for non-diagonal
cases, but the diagonal `T = T` case remains a permanent failure, and
the approach requires explicit closure parameter types that named
helpers do not. Kept as a documented feasibility artifact for potential
future use.

Alternative 6 (type-only priority without closure help) is not
achievable on stable without collapsing into alternative 1; defer
unless specialization stabilizes.
