# SendThunk Analysis

**File:** `fp-library/src/types/send_thunk.rs`

## 1. Type Design

```rust
pub struct SendThunk<'a, A>(Box<dyn FnOnce() -> A + Send + 'a>);
```

The representation is correct and minimal. It mirrors `Thunk<'a, A>(Box<dyn FnOnce() -> A + 'a>)` with the addition of a `Send` bound on the inner trait object. This is the canonical Rust encoding for a thread-safe, non-memoized deferred computation.

The `Send` auto-trait derivation is sound: `Box<dyn FnOnce() -> A + Send + 'a>` is `Send` whenever the trait object itself is `Send`, which it is by definition. The comment on lines 102-104 correctly explains this.

## 2. HKT Support

### Current state

`SendThunkBrand` exists in `brands.rs` and has an `impl_kind!` at line 430-434:

```rust
impl_kind! {
    for SendThunkBrand {
        type Of<'a, A: 'a>: 'a = SendThunk<'a, A>;
    }
}
```

So `SendThunk` **does** have a brand and a `Kind` mapping. What it lacks is implementations of the HKT type-class traits (`Functor`, `Pointed`, `Semimonad`, `Semiapplicative`, `MonadRec`, `Evaluable`, `Lift`, `ApplyFirst`, `ApplySecond`, `FunctorWithIndex`).

### Why it cannot implement them

The HKT traits accept closure parameters as `impl Fn(A) -> B + 'a` without a `Send` bound. For example, `Functor::map` takes `impl Fn(A) -> B + 'a`. If SendThunkBrand implemented `Functor`, a caller could pass a non-`Send` closure, and the implementation would need to store it inside a `SendThunk`, violating the `Send` invariant.

This is a fundamental limitation of the trait signatures, not a design flaw in `SendThunk` itself.

### Should it have a brand?

Having the brand is still valuable even without Functor/Monad. The brand enables:
- `Foldable` for `SendThunkBrand` (implemented), since `fold_right`/`fold_left`/`fold_map` consume the closure eagerly and never store a non-`Send` function inside the result.
- `FoldableWithIndex` for `SendThunkBrand` (implemented).
- `WithIndex` for `SendThunkBrand` (implemented, `Index = ()`).
- Potential future traits that only read from the structure without composing closures into it.

### Trade-offs

The cost of having the brand is negligible (a zero-sized marker type). The benefit is participation in generic code over `Foldable` and `WithIndex`. Removing the brand would be a regression. The current approach is correct.

### Possible future direction

If the library introduced `SendFunctor`, `SendSemimonad`, etc. (traits whose closure parameters carry `Send` bounds), `SendThunkBrand` could implement them. However, this would double the trait hierarchy and is likely not worth the complexity.

## 3. Type Class Implementations

### What SendThunk implements (via brand, on `SendThunkBrand`)

| Trait | Notes |
|-------|-------|
| `Foldable` | `fold_right`, `fold_left`, `fold_map` all evaluate eagerly and return `B`/`M`, not a new `SendThunk`. |
| `WithIndex` | `Index = ()`. |
| `FoldableWithIndex` | `fold_map_with_index`. |

### What SendThunk implements (directly on the type)

| Trait/Impl | Notes |
|------------|-------|
| `Deferrable<'a>` | Eager implementation: calls `f()` immediately because `Deferrable::defer` does not require `Send` on the closure. |
| `SendDeferrable<'a>` | Truly lazy: delegates to `SendThunk::defer(f)` which wraps the closure without evaluating. |
| `Semigroup` | Requires `A: Semigroup + Send`. |
| `Monoid` | Requires `A: Monoid + Send`. |
| `Debug` | Always prints `"SendThunk(<unevaluated>)"`. |
| `From<Thunk<'a, A>>` (where `A: Send`) | Eager evaluation: evaluates the Thunk first because its closure is `!Send`. |

### What SendThunk has as inherent methods

- `new`, `pure`, `defer`, `bind`, `map`, `evaluate`.
- `tail_rec_m` (stack-safe loop, requires `Clone + Send` on the step function).
- `arc_tail_rec_m` (like `tail_rec_m` but wraps the step function in `Arc` to avoid `Clone` requirement).
- `into_arc_lazy` (zero-cost conversion to `ArcLazy`).
- `into_inner` (crate-internal helper for the `From<SendThunk> for Thunk` unsizing coercion).

### What Thunk implements that SendThunk does NOT

| Trait | Reason missing |
|-------|---------------|
| `Functor` (HKT) | Closure in `map` is not `Send`. |
| `Pointed` (HKT) | Could arguably be implemented (pure values are `Send`), but Pointed alone without Functor has limited utility. |
| `Semimonad` (HKT) | Closure in `bind` is not `Send`. |
| `Semiapplicative` (HKT) | Depends on `CloneableFn` which is not `Send`-bounded. |
| `MonadRec` (HKT) | Closure in `tail_rec_m` is not `Send`. Has inherent `tail_rec_m` instead. |
| `Evaluable` (HKT) | Requires `Functor` as a supertrait. |
| `Lift` (HKT) | Closure is not `Send`. |
| `ApplyFirst`, `ApplySecond` (HKT) | Depend on `Semiapplicative`. |
| `FunctorWithIndex` (HKT) | Requires `Functor` (closure is not `Send`). |
| `From<Trampoline<A>>` | Not implemented. Could be: `Trampoline` is `!Send`, so conversion would require eager evaluation (like `From<Thunk>`). Arguably useful but not critical. |
| `From<Lazy<'a, A, Config>>` | Not implemented. `RcLazy` is `!Send`; `ArcLazy` could work but would need eager clone. |

### Notable: `Deferrable` is eager

The `Deferrable<'a>` implementation for `SendThunk` (line 622-626) calls `f()` immediately, making it semantically equivalent to a strict evaluation. This is documented and explained: since `Deferrable::defer` does not require `Send` on its closure parameter, there is no way to store the closure in a `SendThunk`. The `SendDeferrable` trait exists precisely to solve this, and its implementation on `SendThunk` (line 652-656) is truly lazy.

This is a correct design decision but worth noting: code generic over `Deferrable` that expects laziness will get eager evaluation when instantiated with `SendThunk`.

## 4. Thread Safety

### Send bounds

- `SendThunk<'a, A>` is `Send` because `Box<dyn FnOnce() -> A + Send + 'a>` is `Send`. This is auto-derived by the compiler, no manual `unsafe impl Send` is needed.
- The `pure` method requires `A: Send + 'a`, which is correct: the value `a` is moved into a closure that must be `Send`.
- The `map` and `bind` methods require their closure arguments to be `Send + 'a`, which is correct.
- The `Semigroup`/`Monoid` impls require `A: Send`, which is correct for composing two `SendThunk` values.

### Sync

`SendThunk` does NOT implement `Sync`, which is correct. `Box<dyn FnOnce()>` cannot be shared between threads because `FnOnce` is consumed on call (it takes `self`). There is no `&self` interface.

### Not verified

The type does not explicitly test that `SendThunk<'a, A>` is `!Sync`, though this is enforced by the compiler automatically. A compile-fail test for `!Sync` would be a minor improvement for documentation purposes.

## 5. Relationship to Thunk

### Code duplication

The inherent methods `new`, `pure`, `defer`, `bind`, `map`, `evaluate` are nearly identical between `Thunk` and `SendThunk`, differing only in the `+ Send` bound on closures. This is approximately 150 lines of duplicated logic.

### Conversion pathways

```
Thunk  --> SendThunk  (eager: evaluates, wraps result)
SendThunk --> Thunk   (zero-cost: unsizing coercion via into_inner)
SendThunk --> ArcLazy (lazy: closure passed directly)
Thunk  --> ArcLazy    (eager: must evaluate first since closure is !Send)
```

The asymmetry is correct and well-documented: `SendThunk -> Thunk` is zero-cost (unsizing coercion drops the `Send` marker), while `Thunk -> SendThunk` must be eager because the inner closure is `!Send`.

### Should they be unified?

**Option A: Generic over a marker trait.** One could imagine:
```rust
pub struct GenericThunk<'a, A, S: ThreadSafety>(Box<dyn FnOnce() -> A + ... + 'a>);
```
However, Rust does not support conditional auto-trait bounds on trait objects based on a type parameter. You cannot write `Box<dyn FnOnce() -> A + MaybeSend<S> + 'a>`. The `+ Send` bound is part of the trait object type, not something that can be parameterized. This approach is not feasible in current Rust.

**Option B: Macro-generated implementations.** A declarative macro could generate both `Thunk` and `SendThunk` from a common template, reducing duplication. This would be a maintenance improvement but adds macro complexity. The current duplication is modest (approximately 150 lines of inherent methods, plus parallel trait impls) and manageable.

**Option C: Shared inner function bodies.** Extract shared logic into free functions that both types call. This is possible but would require exposing internals and would not reduce the public API surface.

**Verdict:** The current duplication is acceptable. The two types have genuinely different bounds, different trait implementations, and different conversion semantics. Unification is not worth the complexity.

## 6. Lifetime Handling

`SendThunk<'a, A>` is parameterized over `'a`, allowing it to borrow from the environment. This matches `Thunk<'a, A>` and contrasts with `Trampoline<A>` which requires `'static`.

All method signatures correctly propagate `'a`:
- `new(f: impl FnOnce() -> A + Send + 'a)` captures `'a`.
- `bind(self, f: impl FnOnce(A) -> SendThunk<'a, B> + Send + 'a)` returns `SendThunk<'a, B>`.
- `map(self, f: impl FnOnce(A) -> B + Send + 'a)` returns `SendThunk<'a, B>`.
- `pure(a: A) where A: Send + 'a` requires `A` to be bounded by `'a`.

The `tail_rec_m` method requires `S: Send + 'a` and `f: Fn(S) -> SendThunk<'a, Step<S, A>> + Clone + Send + 'a`, which is correct.

One subtlety: `pure` requires `A: Send + 'a`, while `Thunk::pure` only requires `A: 'a`. The `Send` bound is necessary because the value is captured in a `Send` closure. This is correct.

## 7. Documentation Quality

### Strengths

- Thorough module-level documentation (lines 1-9) explaining the relationship to `Thunk` and the HKT limitation.
- Comprehensive comparison table in the struct docs (lines 57-66) covering thread safety, HKT compatibility, stack safety, memoization, lifetime, and use case.
- Clear explanation of why HKT traits cannot be implemented (lines 68-78).
- Algebraic properties (monad laws) stated explicitly (lines 80-87).
- Stack safety caveats documented (lines 89-91).
- Every public method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` attributes.
- The `Deferrable` impl documents its eager semantics.
- The `From<Thunk>` impl documents the eager evaluation requirement.

### Weaknesses

- The doc comment on `SendThunk::pure` says "Returns a pure value (already computed)" which is slightly misleading; it wraps the value in a closure that returns it. Technically it is "already determined" but the closure is still called on `evaluate()`.
- No `# Safety` or `# Panics` sections, though none are needed (no unsafe code, no panics in public API).
- The comparison table uses emoji characters which may not render well in all documentation viewers.
- Missing a note about `Sync` behavior (i.e., that `SendThunk` is `Send` but not `Sync`).

## 8. Issues, Limitations, and Design Flaws

### Issue 1: `Deferrable::defer` is eager

As discussed in section 3, the `Deferrable` implementation calls `f()` immediately. This is correct but potentially surprising to users who expect `Deferrable::defer` to be lazy. The doc comment explains this, but it violates the "transparency" law stated in the `Deferrable` trait docs (`defer(|| x)` should be equivalent to `x` "when evaluated"). Technically the law holds because `f()` produces a `SendThunk` that is still deferred, but the outer thunk `f` itself is not deferred.

**Severity:** Low. Documented and unavoidable given the trait signature.

### Issue 2: No `Evaluable` implementation

`Evaluable` requires `Functor` as a supertrait, so `SendThunkBrand` cannot implement it. This means generic code over `Evaluable` cannot work with `SendThunk`. The inherent `evaluate()` method exists but is not accessible through a trait-generic interface.

**Severity:** Medium. Users who want to write generic "evaluate any lazy type" code cannot include `SendThunk`.

### Issue 3: No `FunctorWithIndex` implementation

`Thunk` implements `FunctorWithIndex` (with `Index = ()`), but `SendThunk` does not because `FunctorWithIndex` requires `Functor`. `FoldableWithIndex` is implemented, so there is an asymmetry: you can fold with index but not map with index via the HKT trait. The inherent `map` method is available.

**Severity:** Low. `FunctorWithIndex` with `Index = ()` is rarely useful in practice.

### Issue 4: Missing `From<Trampoline<A>>` conversion

`Thunk` has `From<Trampoline<A>>` and `From<Trampoline<A>> for Thunk<'static, A>`. `SendThunk` has neither direction. Since `Trampoline` is `!Send` (it is built on `Free<ThunkBrand, A>` which uses `Thunk` internally), `From<Trampoline<A>> for SendThunk` would need eager evaluation, similar to `From<Thunk> for SendThunk`.

**Severity:** Low. The conversion path `Trampoline -> Thunk -> SendThunk` exists via chaining.

### Issue 5: `tail_rec_m` requires `Clone` on the step function

The inherent `tail_rec_m` requires `f: impl Fn(S) -> SendThunk<'a, Step<S, A>> + Clone + Send + 'a`. The `Clone` bound is needed because the loop calls `f` multiple times, but since `f` is `Fn` (not `FnOnce`), the `Clone` bound is actually unnecessary; `Fn` closures can be called multiple times without cloning. The `Clone` is inherited from `Thunk`'s `MonadRec::tail_rec_m` pattern, but for the inherent method it could be relaxed to just `Fn + Send + 'a`.

Looking more carefully: the `arc_tail_rec_m` variant exists for non-`Clone` closures, wrapping in `Arc`. But the underlying `tail_rec_m` still takes `Clone`. In practice, because `tail_rec_m` creates a `SendThunk::new(move || { ... loop { f(state) } })`, the `f` is moved into the closure and called in a loop. Since `Fn` closures can be called by reference (`(&f)(state)`), `Clone` is indeed not necessary here. The `Clone` requirement appears to be overly strict.

**Severity:** Low. `arc_tail_rec_m` provides a workaround, and most small closures are `Clone` anyway.

### Issue 6: No `apply` or `lift2` inherent methods

`Thunk` has HKT-level `Semiapplicative::apply` and `Lift::lift2`. `SendThunk` has neither as inherent methods, even though `Send`-bounded versions would be straightforward:
```rust
pub fn lift2<B, C>(f: impl FnOnce(A, B) -> C + Send + 'a, ...) -> SendThunk<'a, C>
```

**Severity:** Low. Users can compose `bind` and `map` manually.

## 9. Alternatives to the Current Design

### A. Conditional `Send` via sealed trait

```rust
mod sealed {
    pub trait MaybeSend {}
    pub struct IsSend;
    pub struct NotSend;
    impl MaybeSend for IsSend {}
    impl MaybeSend for NotSend {}
}
```

This does not help because `Box<dyn FnOnce() -> A>` vs `Box<dyn FnOnce() -> A + Send>` are different types at the trait-object level. You cannot parameterize the `+ Send` part.

### B. Feature flag for `Send` variants

This would be worse than the current approach. Feature flags are additive; you cannot have a feature that removes `Send` bounds.

### C. Newtype wrapper

```rust
pub struct SendThunk<'a, A>(Thunk<'a, A>);
```

This does not work because `Thunk`'s inner closure is `!Send`. You would need `unsafe` to assert `Send`, and the assertion would be incorrect for closures capturing `!Send` data.

### D. Status quo

The current approach of two separate types with parallel inherent methods is the correct Rust solution. It is the same pattern used by `Rc`/`Arc`, `Cell`/`AtomicCell`, `RefCell`/`Mutex`, etc. The duplication is a known Rust ergonomics limitation.

## 10. Summary of Recommendations

1. **Keep the current design.** The duplication between `Thunk` and `SendThunk` is the idiomatic Rust approach for `Send`/`!Send` variants.
2. **Consider relaxing the `Clone` bound on `tail_rec_m`** since the step function is `Fn`, not `FnOnce`.
3. **Consider adding inherent `lift2`/`apply` methods** for parity with the operations available on `Thunk` via HKT.
4. **Consider adding `From<Trampoline<A>>` for `SendThunk<'static, A>`** (with eager evaluation) for completeness.
5. **Consider adding a compile-fail test** verifying that `SendThunk` is `!Sync`.
6. **Documentation:** Add a note about `Send`-but-not-`Sync` behavior in the struct-level docs.

## 11. Test Coverage

The test module (lines 746-1016) is comprehensive:

- Basic operations: `pure`, `new`, `map`, `bind`, `defer`, `evaluate`.
- `into_arc_lazy` with double-access to verify memoization.
- `Semigroup` and `Monoid`.
- `From<Thunk>` conversion.
- `Debug` formatting.
- `Send` marker assertion (`assert_send::<SendThunk<'static, i32>>()`).
- `Deferrable` and `SendDeferrable` trait usage.
- Cross-thread evaluation (3 tests: basic, with `map`, with `bind`).
- `tail_rec_m` basic and stack-safety (200,000 iterations).
- `arc_tail_rec_m` with `AtomicUsize` counter.
- `Foldable`: `fold_right`, `fold_left`, `fold_map`.
- `FoldableWithIndex`: `fold_map_with_index`, unit-index assertion.
- Consistency between `Foldable` and `FoldableWithIndex`.

Missing test coverage:
- `Sync`-negative test (compile-fail).
- Monad law tests (left identity, right identity, associativity) for the inherent methods.
- Functor law tests (identity, composition) for the inherent `map`.
- Property-based (QuickCheck) tests (Thunk has them, SendThunk does not).
