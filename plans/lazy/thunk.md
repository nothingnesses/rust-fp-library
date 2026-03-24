# Analysis: `Thunk<'a, A>` (`fp-library/src/types/thunk.rs`)

## Summary

`Thunk<'a, A>` is a deferred, non-memoized computation wrapping `Box<dyn FnOnce() -> A + 'a>`. It serves as the lightweight HKT-compatible computation type in the lazy evaluation hierarchy, sitting between raw closures and the stack-safe `Trampoline`. The implementation is solid overall, with thorough documentation and correct HKT trait implementations, though there are several design tensions and potential improvements worth examining.

## 1. Overall Design Assessment

The design is sound and well-motivated. `Thunk` fills a clear niche: it is the only computation type that is both HKT-compatible (via `ThunkBrand`) and supports non-`'static` lifetimes. This makes it essential as the bridge between borrowed data and the type class hierarchy.

The core trade-off (no stack safety, no memoization, but full HKT support and lifetime polymorphism) is appropriate and clearly documented.

## 2. Translation from PureScript's `Lazy`

PureScript's `Data.Lazy` is a memoized lazy type (computed at most once). `Thunk` deliberately does NOT correspond to `Lazy`; it corresponds more closely to the raw `Unit -> a` thunk that PureScript's `defer` accepts. The Rust library's `Lazy` type (backed by `Rc<LazyCell<...>>` / `Arc<LazyLock<...>>`) is the actual counterpart to PureScript's `Lazy`.

### Type class coverage comparison

| PureScript `Lazy` | Rust `Thunk` | Notes |
|---|---|---|
| `Functor` | `Functor` | Equivalent. |
| `Apply` | `Semiapplicative` | Equivalent (different naming convention). |
| `Applicative` | `Pointed` | Equivalent. |
| `Bind` | `Semimonad` | Equivalent. |
| `Monad` | (implied by `Pointed` + `Semimonad`) | Correct. |
| `Foldable` | `Foldable` | Equivalent. |
| `FoldableWithIndex Unit` | `FoldableWithIndex` (index = `()`) | Equivalent. |
| `FunctorWithIndex Unit` | `FunctorWithIndex` (index = `()`) | Equivalent. |
| `Traversable` | Not implemented | Documented as impossible; see section 5. |
| `TraversableWithIndex Unit` | Not implemented | Same reason. |
| `Foldable1` | Not implemented | Library does not appear to have `Foldable1`. Not a gap specific to `Thunk`. |
| `Traversable1` | Not implemented | Same as `Traversable`. |
| `Extend` | Not implemented | Library does not have `Extend`/`Comonad`. |
| `Comonad` | Not implemented | Library does not have `Comonad`. |
| `Semigroup a => Semigroup (Lazy a)` | `Semigroup a => Semigroup Thunk` | Equivalent. |
| `Monoid a => Monoid (Lazy a)` | `Monoid a => Monoid Thunk` | Equivalent. |
| `Eq`, `Ord`, `Show`, `Semiring`, etc. | Not implemented | See section 3. |
| `Lazy` (from `Control.Lazy`) | `Deferrable` | Equivalent. |

The translation is appropriate given that `Thunk` is not memoized. The missing `Comonad`/`Extend` is a library-level gap, not specific to this file.

## 3. Missing Standard Trait Implementations

### `Eq` and `Ord`

PureScript implements `Eq` and `Ord` for `Lazy` by forcing evaluation. For `Thunk`, this is impossible without consuming `self` (since `evaluate` takes `self` by value), and Rust's `Eq`/`PartialEq` require `&self`. This is a fundamental Rust limitation with `FnOnce`-based types. No action needed.

### `Display` / `Show`

PureScript's `Show` forces evaluation to show the value. For `Thunk`, this would require consuming `self`, which conflicts with `Display` requiring `&self`. The current `Debug` implementation that prints `Thunk(<unevaluated>)` is the right choice. No action needed.

### Numeric type classes (`Semiring`, `Ring`, etc.)

PureScript implements these for `Lazy a` by deferring the operation. `Thunk` could theoretically implement Rust's `Add`, `Mul`, `Sub`, `Neg`, etc. via the same pattern:

```rust
impl<'a, A: Add<Output = A> + 'a> Add for Thunk<'a, A> {
    type Output = Thunk<'a, A>;
    fn add(self, rhs: Self) -> Self::Output {
        Thunk::new(move || self.evaluate() + rhs.evaluate())
    }
}
```

This could be useful but is low priority. The `Semigroup`/`Monoid` implementations already cover the algebraic structure pattern.

## 4. HKT Trait Implementations

### Correctness

All HKT trait implementations are correct:

- **`Functor`**: Delegates to inherent `map`, which wraps the closure composition. Correct.
- **`Pointed`**: Delegates to inherent `pure`. Correct.
- **`Semimonad`**: Delegates to inherent `bind`. There is a subtle difference: the HKT `bind` requires `impl Fn(A) -> ...` while the inherent method accepts `impl FnOnce(A) -> ...`. The HKT version delegates to the inherent method, which works because `Fn` is a subtype of `FnOnce`. This is correct and well-documented.
- **`Semiapplicative`**: Uses `bind` + `map` (the standard monad-based `apply` derivation). Correct.
- **`Lift`**: Uses `bind` + `map`. Correct but has unnecessary `Clone` bounds (see section 6).
- **`Foldable`**: Evaluates the thunk and applies the fold function. Correct for a single-element container.
- **`FunctorWithIndex` / `FoldableWithIndex`**: Use `()` as index, delegate to `map`/`evaluate`. Correct.
- **`MonadRec`**: Uses an iterative loop. Stack-safe as long as the step function returns shallow thunks. Correctly documented caveat about deep bind chains within the step function.
- **`Evaluable`**: Simple delegation to `evaluate`. Correct.

### `impl_kind!` macro

```rust
impl_kind! {
    for ThunkBrand {
        type Of<'a, A: 'a>: 'a = Thunk<'a, A>;
    }
}
```

This is correct: `Thunk<'a, A>` is `'a`-bounded, and the brand captures the full lifetime-polymorphic type constructor.

## 5. Traversable Limitation

The documentation states `Thunk` cannot implement `Traversable` because `FnOnce` cannot be cloned. This explanation is somewhat misleading. The real issue is that `traverse` for a single-element container requires:

```
traverse :: Applicative f => (a -> f b) -> Thunk a -> f (Thunk b)
```

This requires evaluating the thunk (consuming it) to get the `a`, applying `f`, then wrapping the result back in a `Thunk`. The issue is NOT about cloning `FnOnce`; it is that the result `f (Thunk b)` requires constructing a `Thunk<B>` inside the applicative context. Since `Thunk` is consumed on evaluation, you need the original thunk's value to construct the new one.

Actually, `traverse` for a single-element container should be straightforward:

```rust
// Pseudocode for what traverse would look like:
fn traverse<F, A, B>(f: impl Fn(A) -> F::Of<B>, ta: Thunk<A>) -> F::Of<Thunk<B>> {
    map::<F, _, _>(|b| Thunk::pure(b), f(ta.evaluate()))
}
```

This does not require `Clone` at all. The stated reason in the documentation ("requires `Clone` bounds on the result type") may be an artifact of how the library's `Traversable` trait is defined rather than a fundamental limitation. If the library's `Traversable` trait signature imposes `Clone` bounds that are not needed for `Thunk`, this is a trait design issue worth investigating.

**Recommendation**: Verify whether the `Traversable` trait's signature actually prevents a `Thunk` implementation. If the `Clone` bound is on the trait itself rather than needed by `Thunk`, consider whether the trait can be relaxed, or document the limitation more precisely as a trait-level constraint rather than a fundamental one.

## 6. The `Lift` Implementation and Unnecessary `Clone` Bounds

The `Lift::lift2` implementation for `ThunkBrand`:

```rust
fn lift2<'a, A, B, C>(
    func: impl Fn(A, B) -> C + 'a,
    fa: Thunk<'a, A>,
    fb: Thunk<'a, B>,
) -> Thunk<'a, C>
where
    A: Clone + 'a,
    B: Clone + 'a,
    C: 'a,
{
    fa.bind(move |a| fb.map(move |b| func(a, b)))
}
```

The `Clone` bounds on `A` and `B` come from the `Lift` trait definition, not from `Thunk`'s needs. For `Thunk` specifically, since both `fa` and `fb` are consumed exactly once, neither `A` nor `B` needs to be `Clone`. This is a trait-level over-constraint. The trait presumably requires `Clone` because types like `Vec` need to clone values during lifting.

This is not a bug, but it does limit `lift2` for `Thunk` unnecessarily. No fix is needed at the `Thunk` level; this is a trade-off of the unified trait hierarchy.

## 7. `Deferrable` Implementation

The implementation is sound:

```rust
impl<'a, A: 'a> Deferrable<'a> for Thunk<'a, A> {
    fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
        Thunk::defer(f)
    }
}
```

This flattens `FnOnce() -> Thunk<'a, A>` into `Thunk<'a, A>` by evaluating the outer closure when the resulting thunk is forced. This satisfies the transparency law: `defer(|| x)` is observationally equivalent to `x`.

The `Deferrable` trait documentation correctly explains why `fix` is not provided at this level (requires shared ownership for self-reference).

## 8. Conversion Methods and Interoperability

### `From<Lazy<'a, A, Config>>` for `Thunk<'a, A>`

Requires `A: Clone`, which is necessary because `Lazy::evaluate` returns `&A`. Correct.

### `From<Trampoline<A>>` for `Thunk<'static, A>`

Requires `A: 'static`, which is correct since `Trampoline` requires `'static`. The conversion wraps `trampoline.evaluate()` in a thunk. Correct.

### `memoize` and `memoize_arc`

- `memoize` returns `Lazy<'a, A, RcLazyConfig>`. Correct; wraps in `Rc<LazyCell<...>>`.
- `memoize_arc` requires `A: Send + Sync` and evaluates eagerly because `Thunk` is `!Send`. The documentation and test correctly verify this eager evaluation. This is a well-thought-out design decision.

### Missing conversions

There is no `From<Thunk<'static, A>>` for `Trampoline<A>`. This would be useful for converting lightweight thunks into stack-safe computations:

```rust
impl<A: 'static> From<Thunk<'static, A>> for Trampoline<A> {
    fn from(thunk: Thunk<'static, A>) -> Self {
        Trampoline::new(move || thunk.evaluate())
    }
}
```

There is also no conversion from `Thunk` to `SendThunk`. This is correct; `Thunk` is `!Send`, so the conversion would require eager evaluation (similar to `memoize_arc`). If needed, a user can do `SendThunk::new(move || thunk.evaluate())` after evaluating.

## 9. Performance Considerations

### Boxing overhead

Every `Thunk::new` allocates a `Box`. Chaining operations like `map` and `bind` creates nested boxes. For example, `thunk.map(f).map(g).map(h)` creates 4 allocations total (the original + one per map). This is inherent to the design and documented.

### `bind` in HKT context

The HKT `Semimonad::bind` requires `impl Fn` (not `FnOnce`), which means the binding function could theoretically be called multiple times. For `Thunk`, the function is only called once, so this is fine. However, users who want `FnOnce` semantics should use the inherent `bind` method. This is well-documented.

### `Semiapplicative::apply` closure

```rust
ff.bind(move |f| {
    fa.map(
        #[allow(clippy::redundant_closure)]
        move |a| f(a),
    )
})
```

The `#[allow(clippy::redundant_closure)]` is correctly annotated. The closure is needed for move semantics, not a clippy false positive.

## 10. Edge Cases

### Empty/panic thunks

A `Thunk::new(|| panic!("boom"))` will panic when evaluated. This is expected and correct behavior.

### Zero-sized types

`Thunk::pure(())` works correctly. The `Box<dyn FnOnce() -> ()>` still allocates, but this is unavoidable with the current design.

### Deeply nested bind chains

As documented, these will overflow the stack. The `MonadRec` implementation provides an escape hatch via `tail_rec_m`, which is the standard FP approach.

## 11. Documentation Quality

Documentation is thorough and accurate:

- The struct-level doc comment has a clear comparison table with `Trampoline`.
- Algebraic properties (monad laws) are stated.
- Stack safety caveats are prominently documented.
- The `Traversable` limitation is explained (though the explanation could be more precise; see section 5).
- Every method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]`.
- All doc examples include assertions and appear correct.

### Minor documentation issues

1. The `Traversable` limitation explanation (lines 89-93) attributes the issue to `FnOnce` not being cloneable. While true in general, for a single-element container like `Thunk`, traversal should be possible without cloning. The limitation likely comes from the `Traversable` trait's bounds, not from `Thunk`'s fundamental design. The explanation should be more precise.

2. The `map` inherent method accepts `FnOnce`, but the doc comment just says "transforms the result" without noting this difference from the HKT-level `Functor::map` which requires `Fn`.

## 12. Test Coverage

Tests are comprehensive:

- Basic operations: `new`, `pure`, `map`, `bind`, `defer`, `evaluate`.
- Borrowing (lifetime polymorphism).
- Conversions: `From<Lazy>`, `From<Trampoline>`.
- `Semigroup`/`Monoid` implementations.
- HKT-level traits: `Foldable`, `Lift`, `Semiapplicative`, `Evaluable`.
- Memoization: both `memoize` (Rc) and `memoize_arc` (Arc), including caching verification.
- QuickCheck property tests for functor laws, monad laws, semigroup/monoid laws.

### Missing test cases

- No test for `MonadRec::tail_rec_m` with a large iteration count to verify stack safety.
- No test for `FunctorWithIndex` or `FoldableWithIndex` via the HKT-level free functions (only direct trait invocation tests exist).
- No negative test verifying that deeply nested `bind` chains do overflow (this would be a `#[test] #[should_panic]` or similar, but is admittedly hard to write reliably).

## 13. Recommendations

### High priority

1. **Clarify `Traversable` limitation**: The current explanation is imprecise. Investigate whether the `Traversable` trait signature is the actual blocker, and update the documentation to reflect the real constraint.

### Medium priority

2. **Add `From<Thunk<'static, A>>` for `Trampoline<A>`**: Provides a natural upgrade path from `Thunk` to stack-safe computation.

3. **Add `tail_rec_m` stack safety test**: A property test with 100,000+ iterations would verify the `MonadRec` implementation does not overflow.

### Low priority

4. **Consider `std::ops` trait implementations**: `Add`, `Mul`, etc. for `Thunk<'a, A>` where `A` implements the corresponding trait. Low priority but would improve ergonomics for numeric computations.

5. **Document `Fn` vs `FnOnce` difference on `map`/`bind`**: The inherent methods accept `FnOnce` while HKT methods require `Fn`. This is mentioned for `bind` but not for `map`.

## 14. Interaction with `Trampoline` and `Free`

`Thunk` interacts with `Trampoline` and `Free` in two ways:

1. **As the effect functor for `Free`**: `Trampoline<A>` is defined as `Free<ThunkBrand, A>`. The `Free` monad uses `Evaluable` to execute `Thunk` effects during evaluation. This is why `ThunkBrand` implementing `Evaluable` is critical.

2. **As a lightweight alternative**: Users choose `Thunk` when they need HKT compatibility and lifetime polymorphism, and upgrade to `Trampoline` when they need stack safety. The `From<Trampoline>` for `Thunk` conversion supports downgrading.

The interaction is clean and well-designed. The `Evaluable` trait serves as the precise interface between `Thunk` and `Free`.

## 15. Conclusion

`Thunk` is a well-implemented, well-documented type that fills an important niche in the lazy evaluation hierarchy. The design makes appropriate trade-offs for its role. The main areas for improvement are: (1) the imprecise `Traversable` limitation explanation, (2) a missing `Thunk -> Trampoline` conversion, and (3) a missing stack-safety test for `MonadRec`. None of these are critical issues.
