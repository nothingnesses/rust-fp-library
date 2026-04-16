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

In principle, `map(|x: i32| ..., Ok::<i32, String>(5))` is unambiguous:
the closure input `i32` aligns only with `ResultErrAppliedBrand` (where
`Of<'a, i32> = Result<i32, E>`), not `ResultOkAppliedBrand` (where
`Of<'a, i32> = Result<A, i32>`). A more sophisticated encoding could
disambiguate based on the closure signature.

- Pros: Could resolve most call sites without user intervention.
- Cons: Depends on specialization or brittle trait-resolution tricks;
  only works when the closure input is already concretely typed; fragile
  under type inference. Probably not worth the complexity.

### 6. Arity-aware inference priority

Prefer the brand whose fixed parameters are already grounded in the
argument. Effectively alternative 5 expressed at the brand level rather
than the closure level.

- Pros: Same as 5.
- Cons: Same as 5.

## Recommendation

The current design is the conservative, correct choice and should remain
the default. The two highest-leverage improvements, in order:

1. **Alternative 3 (concrete helpers)** as an additive, zero-risk
   convenience layer. `map_ok`, `map_err`, `map_fst`, `map_snd` cover the
   overwhelming majority of real usage of multi-brand types and require
   no changes to the brand-inference machinery.
2. **Alternative 1 (primary brand per type)** if Haskell-style
   ergonomics are a design goal. The choice of primary should be
   explicit via a `#[primary_brand]` attribute so it is discoverable at
   the impl site.

Alternative 4 (richer diagnostics) is a reasonable complement to the
current design regardless of whether 1 or 3 is adopted; it simply makes
the existing error better.

Alternative 2 (newtype wrappers) conflicts with the library's design
principles and is not recommended.

Alternatives 5 and 6 (closure-directed or arity-aware inference) are
probably not worth the complexity they would introduce.
