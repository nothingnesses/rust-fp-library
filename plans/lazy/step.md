# Step Type Analysis

## Overview

`Step<A, B>` is a two-variant enum used as the control type for tail-recursive monadic computations via `MonadRec`. It lives at `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/step.rs` (2841 lines including tests). The type is structurally isomorphic to `Result<B, A>` and `core::ops::ControlFlow<B, A>`, but carries domain-specific naming that makes `MonadRec` usage sites self-documenting.

```rust
pub enum Step<A, B> {
    Loop(A),  // continue with new state
    Done(B),  // computation finished
}
```

## 1. Type Design

### Verdict: Well-designed, appropriate for its role.

The two-variant design is correct and matches the established pattern from PureScript/Haskell (`Step` in purescript-tailrec). The generic parameter ordering `Step<A, B>` where `A` is the loop/continuation type and `B` is the done/result type is consistent with `Either<A, B>` conventions where the "success" or "primary" channel is the second parameter.

The `derive` list is appropriate:

- `Clone, Copy` for value semantics (Step contains no heap data by itself).
- `Debug, PartialEq, Eq, Hash` for diagnostics and testing.
- `serde::Serialize, serde::Deserialize` gated behind the `serde` feature.

One design observation: because `Step` derives `Copy`, it is zero-cost to pass around and match on, which is important since `MonadRec` implementations pattern-match on Step in a tight loop.

## 2. HKT Support

### Three brands, well-structured.

Step has thorough HKT support through three brand types defined in `brands.rs`:

| Brand | Kind | Purpose |
|-------|------|---------|
| `StepBrand` | `Step<A, B>` (bifunctor) | Fully polymorphic over both type parameters. |
| `StepLoopAppliedBrand<A>` | `Step<A, _>` (functor over Done) | Loop type fixed; polymorphic over Done. This is the "primary" functor, analogous to `Either`'s right-biased functor. |
| `StepDoneAppliedBrand<B>` | `Step<_, B>` (functor over Loop) | Done type fixed; polymorphic over Loop. The "secondary" functor. |

The bifunctor brand `StepBrand` has two `impl_kind!` declarations:

```rust
impl_kind! { for StepBrand { type Of<A, B> = Step<A, B>; } }
impl_kind! { for StepBrand { type Of<'a, A: 'a, B: 'a>: 'a = Step<A, B>; } }
```

This dual registration handles both lifetime-free and lifetime-bounded contexts.

### Observation: `'static` constraint on applied brands.

Both `StepLoopAppliedBrand<A>` and `StepDoneAppliedBrand<B>` require their applied type parameter to be `'static`:

```rust
impl_kind! {
    impl<LoopType: 'static> for StepLoopAppliedBrand<LoopType> { ... }
}
```

This is a necessary constraint given that brand type parameters in this library's HKT encoding flow through `impl` blocks that typically require `'static`. Since Step is primarily used as a control type within `tail_rec_m` (where the loop state `A` and result `B` are often concrete, stack-allocated types), this is rarely limiting in practice.

## 3. Type Class Implementations

### Full class hierarchy for both applied brands.

Each applied brand implements the complete Monad + MonadRec tower:

**StepLoopAppliedBrand (functor over Done):**
- `Functor` (maps over Done via `map_done`)
- `Pointed` (wraps in `Done`)
- `Lift` (lifts binary functions; both-Done produces Done, first-Loop short-circuits)
- `Semiapplicative` (applies wrapped functions; short-circuits on Loop)
- `ApplyFirst`, `ApplySecond` (marker traits, blanket defaults)
- `Applicative` (blanket from Pointed + Semiapplicative + ApplyFirst + ApplySecond)
- `Semimonad` (binds through Done, short-circuits Loop)
- `Monad` (blanket from Applicative + Semimonad)
- `Foldable` (folds Done value, returns initial for Loop)
- `Traversable` (traverses Done, wraps Loop in pure)
- `MonadRec` (loop-based `tail_rec_m` that handles nested `Step<Step<A, B>>`)

**StepDoneAppliedBrand (functor over Loop):**
- Same hierarchy as above, but mirrored: maps/binds/folds over Loop; short-circuits on Done.
- `Pointed` wraps in `Loop` (not Done).

**StepBrand (bifunctor):**
- `Bifunctor`
- `Bifoldable`
- `Bitraversable`

### Correctness observations.

1. **MonadRec for Step itself is unusual but correct.** The `tail_rec_m` implementation for `StepLoopAppliedBrand` works on `Step<LoopType, Step<A, B>>`. The outer Step is the monadic layer; the inner Step controls the recursion. If the outer layer is `Loop(l)`, it short-circuits. If it's `Done(Step::Loop(next))`, the iteration continues. If it's `Done(Step::Done(b))`, it finishes. This is the correct encoding for a MonadRec on an Either-like type.

2. **Monad laws are verified** via QuickCheck property tests (left identity, right identity, associativity) for both applied brands. This is thorough.

3. **No `Monad` or `Applicative` impls are written explicitly.** They come from blanket impls in the class modules, which is the correct pattern for this codebase.

### Missing type class: `Eq` as a type class.

Step derives `Eq`, but it does not implement an `Eq` type class (if one exists in the library). This is a minor point; Rust's `Eq` derive is sufficient for testing.

### Missing type class: `Alt` / `Plus`.

Step does not implement `Alt` or `Plus`. For `StepLoopAppliedBrand`, an `Alt` instance could try the first value and fall back to the second on `Loop`. This would mirror `Either`'s `Alt` instance. However, this is not essential for Step's primary use case (MonadRec control flow), so its absence is reasonable.

## 4. Usage Patterns

### Primary: MonadRec step function return type.

Step's main purpose is as the return type within `tail_rec_m` step functions. Every `MonadRec` implementation pattern-matches on Step:

```rust
// In ThunkBrand's MonadRec:
fn tail_rec_m<'a, A: 'a, B: 'a>(func: impl Fn(A) -> Thunk<'a, Step<A, B>> + 'a, initial: A) -> Thunk<'a, B> {
    Thunk::new(move || {
        let mut current = initial;
        loop {
            match func(current).evaluate() {
                Step::Loop(next) => current = next,
                Step::Done(res) => break res,
            }
        }
    })
}
```

This pattern is consistent across all 16+ `MonadRec` implementors in the codebase.

### Secondary: Free monad interpretation.

`Free::fold_free` uses Step to drive the interpretation loop:

```rust
G::tail_rec_m(
    move |free: Free<F, A>| match free.resume() {
        Ok(a)  => G::pure(Step::Done(a)),
        Err(fa) => G::map(|inner_free| Step::Loop(inner_free), nt.transform(fa)),
    },
    self,
)
```

### Tertiary: SendThunk and TrySendThunk.

These types have their own `tail_rec_m` inherent methods (not via the `MonadRec` trait, since they lack full HKT support) that use Step in the same pattern.

### Usage ergonomics at call sites.

Typical usage:

```rust
tail_rec_m::<ThunkBrand, _, _>(
    |(n, acc)| {
        if n == 0 { Thunk::pure(Step::Done(acc)) }
        else      { Thunk::pure(Step::Loop((n - 1, n * acc))) }
    },
    (n, 1),
)
```

The `Step::Loop` / `Step::Done` constructors are clear and self-documenting. No complaints here.

## 5. Ergonomics

### Strengths.

1. **Rich inherent API.** Step provides `is_loop`, `is_done`, `map_loop`, `map_done`, `bimap`, `bi_fold_right`, `bi_fold_left`, `bi_fold_map`, `fold_right`, `fold_left`, `fold_map`, `bind`, `bind_loop`, `done`, `loop_val`, `swap`. These cover the common operations without requiring the HKT machinery.

2. **Conversion traits.** Bidirectional `From` implementations for `Result` and `ControlFlow` with round-trip property tests. This makes interop with standard Rust idioms seamless.

3. **Variant naming.** `Loop` and `Done` are clear and domain-specific. They immediately communicate intent in `MonadRec` contexts, unlike generic names like `Left`/`Right` or `Continue`/`Break`.

### Potential improvements.

1. **`unwrap_done` / `unwrap_loop` methods.** Step has `done() -> Option<B>` and `loop_val() -> Option<A>`, but no panicking extractors. These could be useful for testing, similar to `Result::unwrap()`. However, their absence encourages safer code patterns, so this is a stylistic choice.

2. **`map` as an alias for `map_done`.** Since `StepLoopAppliedBrand` treats Done as the "primary" channel (matching `Either`'s right-bias), a `map` inherent method that maps Done would make the type feel more natural for users who think of Step as "right-biased Either". However, this could cause confusion since Step also supports mapping the Loop side.

3. **The `loop_val` name.** The name `loop_val` breaks from the pattern of `done`. A more symmetric pair would be `loop_val` / `done_val`, or `into_loop` / `into_done`, or just `loop_` / `done` (where `loop_` avoids the keyword). The current asymmetry (`done` vs. `loop_val`) is mildly inconsistent but not a significant issue.

## 6. Documentation Quality

### Verdict: Excellent.

1. **Module-level docs** with a working example showing `sum_to_zero`.
2. **Type-level docs** explain HKT representation with all three brands, serde support, variant descriptions.
3. **Every method** has `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` with working code examples.
4. **Type class implementations** each have full documentation following the same template.
5. **Conversion implementations** document the mapping between variants clearly.

The doc comments are consistent with the library's documentation standards. No gaps detected.

## 7. Issues and Limitations

### 7.1. `'static` requirement on applied brand type parameters.

As noted in section 2, `StepLoopAppliedBrand<LoopType: 'static>` and `StepDoneAppliedBrand<DoneType: 'static>` require `'static`. This means you cannot use Step with the HKT machinery for types containing borrowed data. For Step's primary use case (MonadRec control types with owned data), this is not a problem. But it prevents generic library code from using `StepLoopAppliedBrand` with borrowed loop states.

### 7.2. `Clone` requirement on applied brand parameters for Monad/Applicative.

`Semiapplicative`, `Semimonad`, `Lift`, and `MonadRec` impls require `LoopType: Clone + 'static` (or `DoneType: Clone + 'static`). This is because the "short-circuit" variant must be moved into the result, and multiple branches might need it. Since Step derives `Clone`, this propagates. This is inherent to the semantics and not a design flaw.

### 7.3. No `Extend` / `Comonad` instances.

Step cannot have a lawful `Comonad` instance because `extract` would need to choose between Loop and Done without knowing which variant is present. This is correct; these are intentionally absent.

### 7.4. No `Display` implementation.

Step derives `Debug` but does not implement `Display`. This is a minor gap; a `Display` impl could produce `"Loop(x)"` or `"Done(x)"` for types where `A: Display, B: Display`. Not essential but could improve error messages.

### 7.5. Inherent methods duplicate type class methods.

`Step::bimap`, `Step::fold_right`, `Step::bi_fold_right`, etc. duplicate the functionality provided by the `Bifunctor`, `Foldable`, `Bifoldable` type class implementations. The type class impls delegate to the inherent methods, so there is no code duplication in logic. The inherent methods serve as a more ergonomic API for direct use without HKT machinery, which is the correct design choice.

## 8. Alternatives Analysis

### 8.1. Using `Result<B, A>` directly instead of Step.

**Pros:** No new type; reuses a well-known standard type.
**Cons:** `Err` for "continue looping" and `Ok` for "done" inverts the usual semantics where `Err` means failure. This would make MonadRec code confusing. Step's domain-specific naming (`Loop`/`Done`) is a significant readability win.

**Verdict:** Step is better than bare Result for this use case.

### 8.2. Using `core::ops::ControlFlow<B, A>` directly.

**Pros:** Standard library type with `Continue`/`Break` semantics that roughly align.
**Cons:**
- `ControlFlow` uses `Break` for termination and `Continue` for looping, which maps well conceptually. However, `ControlFlow` is designed for Rust's `?` operator and `Try` trait, not for HKT/MonadRec. Having a separate type allows implementing HKT brands and type classes.
- `ControlFlow` does not derive `Hash` or `serde` traits.
- The naming `Break`/`Continue` is tied to Rust's loop semantics, not to "monadic recursion" semantics.

**Verdict:** Step is better because it can carry HKT brands and has domain-appropriate naming. The `From` conversions provide interop when needed.

### 8.3. Using a trait instead of an enum.

**Pros:** Could theoretically allow open extension.
**Cons:** MonadRec inherently needs exactly two states (loop or done). A trait would add indirection and prevent exhaustive matching. The enum is zero-cost and statically known.

**Verdict:** Enum is clearly the right representation.

### 8.4. Newtype over `Result`.

```rust
pub struct Step<A, B>(Result<B, A>);
```

**Pros:** Reuses Result's machinery internally.
**Cons:** Adds indirection conceptually. Pattern matching requires destructuring the newtype. The enum representation is simpler and more direct.

**Verdict:** Direct enum is better.

## 9. Testing Coverage

The test module (lines 2083-2840) is comprehensive:

- **Unit tests** for all inherent methods (`is_loop`, `is_done`, `map_loop`, `map_done`, `bimap`, `done`, `loop_val`, `swap`).
- **Type class tests** for both applied brands: Functor, Bifunctor, Lift, Pointed, Semiapplicative, Semimonad, Foldable, Traversable.
- **Law verification** via QuickCheck: Functor identity/composition, Bifunctor identity/composition, Monad left identity/right identity/associativity, MonadRec identity.
- **Conversion round-trip** property tests for Step/Result and Step/ControlFlow.
- **MonadRec** tests including identity law, recursive sum, short-circuiting, and stack safety (200,000 iterations).
- **Marker trait** compile-time checks for Applicative, Monad, and MonadRec.
- **Custom `Arbitrary` impl** for QuickCheck that generates both variants with equal probability.

This is thorough testing. No significant gaps.

## 10. Summary

Step is a well-designed, well-documented, thoroughly tested control type that serves its purpose cleanly. The enum representation with `Loop`/`Done` variants is the right choice. The three-brand HKT support (`StepBrand`, `StepLoopAppliedBrand`, `StepDoneAppliedBrand`) is comprehensive and mirrors the pattern used by other bifunctor types like Pair and Tuple2 in the library. The full Monad + MonadRec tower for both applied brands, plus Bifunctor/Bifoldable/Bitraversable for the base brand, covers all realistic use cases.

The type has no significant design issues. The minor observations (no `Display`, slight naming asymmetry between `done` and `loop_val`, `'static` bounds on applied brands) are all either intentional trade-offs or low-priority improvements that do not affect the type's fitness for its primary role in the MonadRec machinery.

**No changes recommended.** Step is production-ready as-is.
