# Evaluable Trait Analysis

**File:** `fp-library/src/classes/evaluable.rs`

## 1. Trait Design

### What Abstraction Does Evaluable Capture?

`Evaluable` models a functor that always contains exactly one extractable value, providing a natural transformation `F ~> Id`. In category-theoretic terms, this is the `extract` operation of a comonad, restricted to functors that yield an *owned* value (not a reference). The trait has a single method:

```rust
fn evaluate<'a, A: 'a>(
    fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
) -> A;
```

This takes `F<A>` by value and returns `A`. The trait extends `Functor`, so any `Evaluable` type must also support `map`.

### Is It the Right Abstraction?

The abstraction is well-scoped for its primary purpose: enabling `Free::evaluate` to peel off functor layers iteratively. In `Free`'s evaluate loop, the `FreeInner::Wrap(fa)` case calls `<F as Evaluable>::evaluate(fa)` to unwrap each suspended layer.

However, the trait occupies an unusual position in the type class landscape:

- **It is essentially `Comonad::extract` without `extend`.** In Haskell/PureScript, `extract :: w a -> a` is part of the `Comonad` class. This library does not have `Comonad`, so `Evaluable` fills that gap for the single use case that needs it.
- **It is also equivalent to a natural transformation `F ~> Identity`.** The library already has `NaturalTransformation`, and one could express `evaluate` as a `NaturalTransformation<F, IdentityBrand>` instance. The dedicated trait avoids the verbosity of the natural transformation encoding and makes the intent clear.

The naming "Evaluable" is fitting for the lazy evaluation context (thunks, free monads) where "evaluate" means "force the computation." It would be less intuitive if the trait were applied to non-lazy types like `Identity` or `Vec` (for the head element), though that is not the current intent.

## 2. Relationship to Other Traits

### Direct Supertraits

- **`Functor`** (required): `Evaluable` extends `Functor`. This is needed because `Free::evaluate` calls `F::map` in the `Wrap` case to rebuild the Free structure, and `Free::lift_f` uses `F::map(Free::pure, fa)`.

### Related Traits (Not Supertraits)

| Trait | Relationship |
|-------|-------------|
| `Deferrable<'a>` | Dual/inverse. `Deferrable` constructs a lazy value from a thunk (`() -> Self`); `Evaluable` forces a lazy value to produce its result (`F<A> -> A`). Together they form a round-trip: `evaluate(defer(\|\| x))` should equal `x`. |
| `Foldable` | `Foldable` subsumes the information-extraction capability of `Evaluable` since `fold_right(f, z, fa)` can extract the single value. But `Foldable` works with borrowed function brands and does not return ownership, so it cannot replace `Evaluable` for `Free`'s needs. |
| `RefFunctor` / `SendRefFunctor` | These are mapping traits for types where the inner value is accessed by reference. `Lazy` implements `RefFunctor` but not `Evaluable` precisely because `Lazy::evaluate()` returns `&A`, not `A`. |
| `NaturalTransformation` | As noted, `Evaluable` is equivalent to a natural transformation `F ~> Identity`. The library keeps them separate for ergonomics. |
| `Comonad` (absent) | The library does not define `Comonad`. If it did, `Evaluable` would be a strict subset: `Comonad = Functor + extract + extend`, while `Evaluable = Functor + extract`. |

### Interaction with `Free`

`Evaluable` is the fundamental requirement for `Free<F, A>`:

- `Free<F, A>` requires `F: Evaluable + 'static` on every impl block, including `Drop`.
- `Free::evaluate` uses `<F as Evaluable>::evaluate(fa)` to unwrap `Wrap` layers.
- `Free::resume` uses `F::map` (from the `Functor` supertrait) to rebuild continuations.
- `Free::hoist_free` requires the target functor `G: Evaluable`.
- `Free`'s `Drop` impl uses `Evaluable::evaluate` to iteratively dismantle nested `Wrap` layers and avoid stack overflow.

This tight coupling means `Evaluable` is essentially "the trait you must implement to be a base functor for `Free`."

## 3. Method Signatures

### `evaluate`

```rust
fn evaluate<'a, A: 'a>(
    fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
) -> A;
```

**Observations:**

- **Lifetime `'a` is generic.** The method is polymorphic over lifetimes, matching the standard HKT pattern used throughout the library. This is correct: `Thunk<'a, A>` carries a `'a` lifetime, and the trait method accommodates it.
- **Takes `fa` by value.** This is essential since forcing a thunk consumes it (the inner `FnOnce` can only be called once). By-reference would be incorrect.
- **Returns owned `A`.** This is the key distinction from `Lazy::evaluate() -> &A`, which is why `Lazy` cannot implement `Evaluable`.

**Potential issue:** Despite the lifetime polymorphism in the trait definition, `Free` constrains `F: Evaluable + 'static` and `A: 'static`, so the `'a` parameter is always `'static` in practice. The lifetime flexibility in the trait signature is never actually exercised by `Free`, though `Thunk<'a, A>` does use it in standalone `evaluate::<ThunkBrand, _>()` calls.

### Free Function

```rust
pub fn evaluate<'a, F, A>(fa: Apply!(...)) -> A
where
    F: Evaluable,
    A: 'a,
```

The free function is a straightforward delegation. Its type parameter ordering `<'a, F, A>` means the caller writes `evaluate::<ThunkBrand, _>(thunk)`, which is ergonomic since `'a` is inferred.

## 4. Implementors

### Current Implementors

| Brand | Type | Notes |
|-------|------|-------|
| `ThunkBrand` | `Thunk<'a, A>` | The only implementor. Delegates to `Thunk::evaluate(self)`, which calls `(self.0)()`. |

### Types That *Could* Implement Evaluable But Do Not

| Type | Why Not |
|------|---------|
| `SendThunk<'a, A>` | Has a brand (`SendThunkBrand`) and an inherent `evaluate(self) -> A`. Has `Foldable` but does **not** implement `Functor`, so it cannot implement `Evaluable`. The reason: `SendThunk::new` requires `Send` on the closure, but the `Functor::map` signature takes `impl Fn(A) -> B + 'a` without a `Send` bound, making it impossible to produce a new `SendThunk` from a non-`Send` closure. |
| `Identity<A>` | Has a brand (`IdentityBrand`) and implements `Functor`. Could trivially implement `Evaluable` as `fn evaluate(fa) -> A { fa.0 }`. Currently does not, likely because there is no use case (nobody builds `Free<IdentityBrand, A>`). |
| `Trampoline<A>` | No brand (it is `Free<ThunkBrand, A>` under the hood). Has an inherent `evaluate(self) -> A`, but since it lacks a brand, it cannot participate in HKT traits. |
| `Lazy<'a, A, Config>` | Has a brand (`LazyBrand<Config>`) but `evaluate(&self) -> &A` returns a reference, not an owned value. Cannot satisfy `Evaluable`'s signature. |
| `TryThunk<'a, A, E>`, `TrySendThunk<'a, A, E>` | These have inherent `evaluate(self) -> Result<A, E>`, but the "evaluate" concept for them involves fallibility, which does not fit `Evaluable`'s infallible signature. Their applied brands (e.g., `TryThunkAppliedBrand<E>`) implement `Functor`, so a `TryEvaluable` variant could theoretically exist. |

### Assessment

Having only one implementor is a red flag for any trait, but in this case it is somewhat justified:

1. `Evaluable` exists primarily for `Free`, and `Free<ThunkBrand, A>` (i.e., `Trampoline`) is the primary use case.
2. The design documents suggest this is intentional: the `Free` module docs explicitly state that the interpretation logic is "baked into the `Evaluable` trait implemented by the functor `F`."
3. `fold_free` with `NaturalTransformation` provides the more general interpretation mechanism for other functors.

Still, the trait's generality is underutilized. The `Free` type could have been designed with a hardcoded `ThunkBrand` constraint instead of the generic `F: Evaluable`, and the observable behavior would be identical.

## 5. Free Functions

The module exports a single free function:

```rust
pub fn evaluate<'a, F: Evaluable, A: 'a>(fa: ...) -> A
```

This follows the library's standard pattern of providing a free function alongside each trait method. It is re-exported via `fp-library/src/functions.rs` (per the architectural pattern).

The free function is well-designed and consistent with the rest of the library. There is no `evaluate_send` variant, which is consistent since there is no `SendEvaluable` (and currently no `Send` type implements `Evaluable`).

## 6. Documentation Quality

### Strengths

- **Clear conceptual framing.** The trait doc explains it as "a natural transformation `F ~> Id`" and correctly identifies that it requires functors containing "exactly one extractable value."
- **Explains why `Lazy` cannot implement it.** The documentation proactively addresses the most obvious question.
- **Explains why `Trampoline` cannot implement it.** Notes the lack of a brand.
- **Law documentation.** The map-extract law is clearly stated with a textual formula.
- **Property-based test.** The `prop_evaluable_map_extract` test validates the stated law.

### Weaknesses

- **Module-level doc example is minimal.** It only shows `Thunk::new(|| 42)` followed by `evaluate`. A more illustrative example showing the law in action, or showing use within `Free`, would be more informative.
- **No mention of relationship to Comonad.** Users coming from Haskell might search for `extract` or `Comonad` and not find this trait.
- **No mention of `Free` in the trait doc itself.** The module doc mentions `Free::evaluate`, but the trait doc says "used by `Free::evaluate`" only in passing. A more explicit "this trait exists primarily to support `Free`" would help readers understand its role.
- **"eval" vs "evaluate" naming inconsistency.** The doc parameter description for the `ThunkBrand` impl says "The eval to run" and "Runs the eval", using "eval" as a noun to refer to the thunk. The trait-level doc uses "evaluate" consistently. Minor, but slightly confusing.

## 7. Issues, Limitations, and Inconsistencies

### Single Implementor

As discussed, `ThunkBrand` is the only implementor. This means:
- The trait's genericity over `F` in `Free<F, A>` is never exercised.
- No downstream code is generic over `Evaluable` except `Free`.
- There is no compile-time evidence that the abstraction boundary is correct beyond `ThunkBrand`.

### No Relationship to `Deferrable`

`Evaluable` and `Deferrable` are conceptual duals, but the type system does not capture this. There is no combined trait or law connecting the two. A "round-trip law" (`evaluate(defer(|| pure(x))) == x`) is tested in `deferrable.rs`'s tests but is not documented as an `Evaluable` law.

### The `Functor` Supertrait May Be Overly Restrictive

`Evaluable: Functor` means that any type implementing `Evaluable` must also implement `Functor`. This is needed by `Free`, but not by the `evaluate` operation itself. If `Evaluable` were standalone (without requiring `Functor`), then `SendThunkBrand` could implement it. However, since `Free` needs both `Evaluable` and `Functor`, splitting them would just move the bound to `Free`'s where clause, so the current design is pragmatically correct.

### `Free`'s `Drop` Depends on `Evaluable`

The `Drop` impl for `Free<F, A>` calls `<F as Evaluable>::evaluate(fa)` to eagerly extract inner `Free` values from `Wrap` layers. This means dropping a `Free` can trigger side effects if the functor `F` has them (unlikely for `Thunk`, which just runs a closure, but notable for hypothetical future implementors).

### Missing Law: Interaction with `pure`/`Pointed`

The documentation states one law (map-extract). A stronger characterization would include:

- **Pure-extract:** If `F` is also `Pointed`, then `evaluate(pure(x)) == x`.
- **Map-extract** (already stated): `evaluate(map(f, fa)) == f(evaluate(fa))`.

These two together would make `Evaluable` a proper comonad-like extraction. The pure-extract law is implicitly relied upon by `Free::evaluate` (which starts from `FreeInner::Pure`), but is not documented as a law of `Evaluable`.

### `'static` Constraint in Practice

While the trait itself is lifetime-polymorphic, all uses through `Free` require `'static`. This means the `'a` parameter in `evaluate<'a, A: 'a>` is always instantiated at `'static` in the `Free` context. The flexibility is only meaningful for standalone `evaluate::<ThunkBrand, _>(thunk)` calls on non-`'static` thunks, which is a valid but uncommon use case.

## 8. Alternatives and Improvements

### Alternative: Remove the Trait, Hardcode ThunkBrand in Free

Since `ThunkBrand` is the only implementor, `Free` could directly require `ThunkBrand` instead of the generic `F: Evaluable`. This would simplify the codebase but reduce extensibility. The current design is more future-proof, allowing hypothetical new functor types to be used with `Free`.

**Verdict:** The current approach is reasonable. The trait serves as a documented abstraction boundary even if only one type crosses it today.

### Alternative: Rename to `Extract` or `Comonad`

If the library eventually adds `Comonad` (with `extend`), `Evaluable` would be subsumed. At that point, either:
1. Make `Evaluable` a supertrait of `Comonad` (like `Functor` is to `Applicative`).
2. Rename `Evaluable` to `Extract` and add `Comonad: Extract + Functor`.

**Verdict:** The current name is fine for the current scope. A rename should wait until `Comonad` is actually needed.

### Improvement: Add `IdentityBrand` as an Implementor

`Identity<A>` trivially satisfies `Evaluable` (just unwrap the newtype). Adding this implementation would:
- Validate the abstraction with a second implementor.
- Enable `Free<IdentityBrand, A>`, which is isomorphic to `A` and could serve as a useful degenerate case in generic code.
- Provide a test bed for the map-extract law on a non-lazy type.

### Improvement: Document the Pure-Extract Law

If `F` is also `Pointed`, the law `evaluate(pure(x)) == x` should be documented. This law is implicitly relied upon and would strengthen the specification.

### Improvement: Cross-reference with Deferrable

The trait docs should mention that `Deferrable` is the conceptual dual, and that together they form a round-trip for types like `Thunk` that implement both.

### Improvement: Add a Module-Level Example Involving Free

The module doc example only shows standalone evaluation. Adding an example showing `Free::evaluate` calling through `Evaluable` would better communicate the trait's primary purpose.

## Summary

`Evaluable` is a focused, well-designed trait that serves a specific purpose: enabling `Free::evaluate` to unwrap functor layers. It captures the `extract` operation from comonad theory, restricted to owned values. The trait is documented competently, has a clear law, and includes a property-based test.

The main weakness is its extreme narrowness: only `ThunkBrand` implements it, and only `Free` consumes it. This makes the trait feel like an implementation detail of the `Free` monad rather than a general-purpose abstraction. Adding `IdentityBrand` as a second implementor and strengthening the law documentation would address this without changing any existing code.
