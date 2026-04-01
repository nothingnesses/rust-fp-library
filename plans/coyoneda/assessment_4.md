# Coyoneda Implementations: Independent Code Review

## 1. Overview

This review covers the four Coyoneda implementations in the `fp-library` crate:

- `coyoneda.rs` -- `Coyoneda<'a, F, A>`: The primary free functor using boxed trait objects for existential quantification.
- `coyoneda_explicit.rs` -- `CoyonedaExplicit<'a, F, B, A, Func>`: Zero-cost variant exposing the intermediate type for compile-time function composition.
- `rc_coyoneda.rs` -- `RcCoyoneda<'a, F, A>`: `Rc`-based variant enabling `Clone`.
- `arc_coyoneda.rs` -- `ArcCoyoneda<'a, F, A>`: `Arc`-based variant enabling `Clone + Send + Sync`.

Related files examined: `brands.rs`, `kinds.rs`, `classes/functor.rs`, `classes/foldable.rs`, `classes/semimonad.rs`, `classes/traversable.rs`, `benches/benchmarks/coyoneda.rs`.

---

## 2. Identified Issues

### 2.1. `Fn` bound where `FnOnce` would suffice in `Coyoneda`

**Files:** `coyoneda.rs` lines 244, 306, 403, 525; `rc_coyoneda.rs` line 143; `arc_coyoneda.rs` line 161

`Coyoneda::map` accepts `impl Fn(A) -> B + 'a`, storing the function inside the layer. Since `Coyoneda::lower` consumes `self` (it takes `self: Box<Self>`), each stored function is used exactly once. The `Fn` bound is unnecessarily restrictive; `FnOnce` would be the natural choice, permitting closures that move out of captured values.

The same applies to `Coyoneda::new` (line 403) and `CoyonedaNewLayer` (line 306).

For `RcCoyoneda` and `ArcCoyoneda`, `Fn` is correct because `lower_ref` can be called multiple times, so the function must be callable multiple times. However, `Coyoneda` has no such constraint.

**Impact:** Users cannot pass `FnOnce` closures to `Coyoneda::map`. This forces unnecessary `Clone` on captured state or awkward workarounds. It prevents using move-out patterns that are natural in Rust.

**Root cause:** `F::map` itself is defined with `impl Fn(A) -> B + 'a` (functor.rs line 122). The `Fn` bound propagates from the Functor trait signature to `Coyoneda`. This is a library-wide design choice, not specific to Coyoneda, but Coyoneda's layered architecture makes it especially visible.

**Approaches:**

A. **Change the Functor trait to use `FnOnce`.** This would fix the root cause. Trade-off: `FnOnce` closures cannot be used with multi-element containers (a `Vec::map` with `FnOnce` can only process one element). The Functor trait uses `Fn` because functors like `Vec` call the function multiple times. This approach is a non-starter for the general case.

B. **Store the function in an `Option` or `Cell` and take it out in `lower`.** The layer could store `Option<Func>` and `unwrap` in `lower`. This allows accepting `FnOnce` at the `Coyoneda::map` level without changing the Functor trait. At `lower` time, the function is extracted from the option and passed to `F::map` (which still needs `Fn`), so this only works if the layer wraps the `FnOnce` in a `move || ...` cell that panics on second call. Trade-off: runtime panic risk if invariants are violated; increased complexity.

C. **Accept the `Fn` bound as a library-wide constraint.** The Functor trait's `Fn` requirement is well-motivated (multi-element containers), and `Coyoneda` must match because lowering delegates to `F::map`. The `Fn` bound is fundamental to how the library works.

**Recommendation:** Option C. The `Fn` bound is inherited from the Functor trait and cannot be relaxed without unsoundness or runtime panics. This is a known limitation of the library's design and should remain documented as such.

---

### 2.2. No map fusion in `Coyoneda` (k calls to `F::map` at lower time)

**File:** `coyoneda.rs` lines 12-18, 57-64

Each `map` call on `Coyoneda` adds a new `CoyonedaMapLayer`, and `lower` calls `F::map` once per layer. For k chained maps on a `Vec` of n elements, lowering costs O(k\*n). PureScript's implementation achieves O(n) by composing functions eagerly.

This is already documented extensively in the module docs (lines 12-18, 57-64). The documentation correctly identifies the root cause: Rust trait objects cannot have generic methods, so composing functions across the existential boundary is impossible.

**Impact:** For eager containers like `Vec`, `Coyoneda` provides no performance benefit over direct chaining. It is strictly worse because of the boxing overhead per layer. For lazy containers (like iterators), the overhead is negligible.

**Approaches:**

A. **Use `CoyonedaExplicit` instead.** Already exists. Trade-off: no HKT integration; function type leaks into the type signature; requires `.boxed()` for uniform types.

B. **Add a `fuse` method that composes the top two layers.** This would walk the layer chain and merge adjacent function layers. Trade-off: impossible without generic methods on dyn trait, same fundamental limitation.

C. **Hybrid approach: `Coyoneda` stores a `CoyonedaExplicit` internally.** The outer `Coyoneda` could hold a boxed `CoyonedaExplicit<F, B, A, Box<dyn Fn(B) -> A>>` where `B` is existentially hidden. Each `map` would compose with the boxed function rather than add a layer. This would fuse all maps into a single function at `map` time, achieving O(n) at `lower`. Trade-off: each `map` must box the composed function (one allocation per map, same as now), but lowering is single-pass. The intermediate type `B` is still hidden by the trait object boundary. This is actually feasible because the function composition doesn't need to "open" the existential; it just pre-composes on top.

**Recommendation:** Option C is worth exploring. The current `CoyonedaNewLayer` already stores `(fb, func)` like `CoyonedaExplicit`. If `Coyoneda::map` could compose with the stored function via dynamic dispatch (by boxing the composed function), fusion would be achieved. This would be a significant performance improvement. However, the boxing cost per `map` remains the same, and `lower` becomes single-pass.

Implementation sketch: Instead of the current `CoyonedaInner` trait with a `lower` method, store `Box<dyn CoyonedaInner<'a, F, B>>` (just the functor value wrapper) plus `Box<dyn Fn(B) -> A>` (the composed function). On `map(g)`, replace the function with `Box::new(move |b| g(f(b)))` where `f` is the old function. On `lower`, call `F::map(func, inner.get_fb())`. The existential type `B` remains hidden throughout.

Wait: the problem is that `B` is existentially quantified. When `map<C>(g: impl Fn(A) -> C)` is called, we want to compose `g` with the existing `Box<dyn Fn(B) -> A>` to get `Box<dyn Fn(B) -> C>`. This composition is: `move |b: B| g(f(b))`. But `B` is hidden. The new function type is `dyn Fn(B) -> C`, but `B` is not in scope at the call site. We can only do this inside the trait object that knows `B`, which means we need a method like `map_func<C>(g: impl Fn(A) -> C) -> Box<dyn CoyonedaInner<'a, F, C>>`. This method is generic over `C`, making it not dyn-compatible. So the fundamental limitation reasserts itself.

**Revised recommendation:** The documentation already identifies this limitation correctly. The `CoyonedaExplicit` type is the right workaround. No further action needed beyond what exists.

---

### 2.3. No stack safety for `RcCoyoneda` and `ArcCoyoneda`

**Files:** `rc_coyoneda.rs` lines 172-178; `arc_coyoneda.rs` lines 212-218

The `lower_ref` methods call `self.inner.lower_ref()` recursively through the layer chain. For deep chains (thousands of maps), this will overflow the stack, just like `Coyoneda::lower`. Unlike `Coyoneda`, these variants have no mitigation:

- No `stacker` feature support (no `#[cfg(feature = "stacker")]` blocks).
- No `collapse` method.
- No documentation warning about stack overflow risk.

**Impact:** Deep chains on `RcCoyoneda` and `ArcCoyoneda` will silently overflow the stack. Users may not realize this until production.

**Approaches:**

A. **Add `stacker` support to `lower_ref`.** Wrap the recursive call in `stacker::maybe_grow(...)` under `#[cfg(feature = "stacker")]`, matching the pattern in `Coyoneda`. Trade-off: minor complexity; near-zero overhead when stack is sufficient.

B. **Add a `collapse` method.** For `RcCoyoneda`, `collapse` would call `lower_ref()` (which clones the base), then re-lift the result into a fresh single-layer `RcCoyoneda`. Same for `ArcCoyoneda`. Trade-off: requires `F: Functor`, forces an intermediate allocation, but resets the layer depth.

C. **Document the limitation.** Add a stack safety section to the module docs and recommend periodic `collapse` or bounded chain depth. Trade-off: no code change, users must manage it themselves.

**Recommendation:** All three approaches should be adopted. Add `stacker` support (A), add `collapse` (B), and document the limitation (C). This matches the approach taken for `Coyoneda`.

---

### 2.4. `Foldable` for `RcCoyonedaBrand`/`ArcCoyonedaBrand` consumes the value unnecessarily

**Files:** `rc_coyoneda.rs` lines 413-421; `arc_coyoneda.rs` lines 417-424

The `Foldable::fold_map` signature takes `fa` by value. For `RcCoyonedaBrand` and `ArcCoyonedaBrand`, the implementation calls `fa.lower_ref()`, which takes `&self`. The value `fa` is consumed by the trait signature even though the implementation only borrows it. Since `RcCoyoneda` and `ArcCoyoneda` are `Clone`, the caller can clone before calling, but this is wasteful and unintuitive.

**Impact:** Minor ergonomic issue. Users who want to fold and then continue using the value must explicitly clone.

**Approaches:**

A. **Accept this as a trait constraint.** The `Foldable` trait signature takes `fa` by value; there is nothing `RcCoyonedaBrand` can do about it without changing the trait. Trade-off: none, this is the status quo.

B. **Add inherent `fold_map` method on `RcCoyoneda`/`ArcCoyoneda` that takes `&self`.** This provides a borrow-based fold alongside the trait-based one. Trade-off: API surface grows; naming collision risk with the free function.

**Recommendation:** Option B. An inherent `fold_map` method on `RcCoyoneda` and `ArcCoyoneda` that borrows `&self` would be a natural complement to `lower_ref`. The trait-based `Foldable` impl remains for generic contexts.

---

### 2.5. `Semimonad::bind` for `CoyonedaBrand` performs double traversal

**File:** `coyoneda.rs` line 858

```rust
fn bind<'a, A: 'a, B: 'a>(
    ma: ...,
    func: impl Fn(A) -> ... + 'a,
) -> ... {
    Coyoneda::lift(F::bind(ma.lower(), move |a| func(a).lower()))
}
```

The `func(a).lower()` call inside the bind callback creates a new `Coyoneda`, potentially with accumulated maps, and immediately lowers it. If the user writes `coyoneda.map(f).bind(g)`, the `map(f)` layer is lowered in `ma.lower()`, and then `g(a)` might return a `Coyoneda` with its own maps that are also lowered. This is correct but inefficient: the outer `lower()` applies all accumulated maps (traversing the inner functor), and then `bind` traverses it again.

**Impact:** For chained `map + bind` patterns, the functor is traversed once per `bind` call plus once per accumulated `map` chain. This is an inherent consequence of the layered architecture and the inability to fuse maps.

**Approaches:**

A. **Accept as inherent limitation.** The bind implementation is correct. The extra traversal is unavoidable without map fusion. Trade-off: none.

B. **Optimize by composing accumulated maps before binding.** If `Coyoneda` could extract the composed function and base functor value (like `CoyonedaExplicit` does), bind could compose the callback with the accumulated function before binding. This requires opening the existential, which is the same unsolvable problem.

**Recommendation:** Option A. This is inherent and already documented.

---

### 2.6. `ArcCoyonedaBrand` does not implement `Functor` (HKT gap)

**File:** `arc_coyoneda.rs` lines 375-379

```rust
// Note: ArcCoyonedaBrand does NOT implement Functor. The HKT trait signatures
// lack Send + Sync bounds on closure parameters, so there is no way to guarantee
// that closures passed to map are safe to store inside an Arc-wrapped layer.
```

This is documented but represents a significant gap. `ArcCoyoneda` cannot participate in generic `Functor`-polymorphic code. The same limitation applies to `SendThunkBrand` per the comment.

**Impact:** Users who need thread-safe Coyoneda with HKT integration have no solution. `RcCoyonedaBrand` implements `Functor` but is `!Send`.

**Approaches:**

A. **Add `SendFunctor` trait.** A parallel Functor trait with `Send + Sync` bounds on closures. Trade-off: doubles the trait hierarchy; pervasive code duplication; unclear where to draw the line (would also need `SendSemimonad`, `SendApplicative`, etc.).

B. **Accept as a known limitation.** The HKT system's `Functor::map` signature does not carry `Send + Sync` bounds, and changing it would break all non-`Send` uses. Trade-off: `ArcCoyoneda` remains usable only via inherent methods.

C. **Parameterize the Functor trait over thread-safety.** Use a marker parameter or associated type to toggle `Send + Sync` bounds on closures. Trade-off: extremely complex type-level machinery; probably not worth the effort.

**Recommendation:** Option B. This is a fundamental tension between Rust's ownership/thread-safety model and HKT polymorphism. The documentation correctly explains the limitation. Users who need thread-safe operations can use `ArcCoyoneda` directly via inherent methods.

---

### 2.7. `ArcCoyonedaMapLayer` has manually implemented `Send`/`Sync` with potentially insufficient verification

**File:** `arc_coyoneda.rs` lines 164-183

```rust
// SAFETY: Both fields are Arc<dyn ... + Send + Sync>, which are Send + Sync.
unsafe impl<'a, F, B: 'a, A: 'a> Send for ArcCoyonedaMapLayer<'a, F, B, A> where
    F: Kind_cdc7cd43dac7585f + 'a
{
}
unsafe impl<'a, F, B: 'a, A: 'a> Sync for ArcCoyonedaMapLayer<'a, F, B, A> where
    F: Kind_cdc7cd43dac7585f + 'a
{
}
```

The fields are:

- `inner: Arc<dyn ArcCoyonedaLowerRef<'a, F, B> + 'a>` -- `ArcCoyonedaLowerRef` requires `Send + Sync`, so this is `Arc<dyn Send + Sync + ...>`, which is indeed `Send + Sync`.
- `func: Arc<dyn Fn(B) -> A + Send + Sync + 'a>` -- explicitly `Send + Sync`.

The safety argument is correct: both fields are `Arc<dyn ... + Send + Sync>`, which are `Send + Sync` regardless of `F`. The bound on `F` is only `Kind_cdc7cd43dac7585f + 'a`, which is correct because `F` as a brand type is a zero-sized marker that doesn't carry data.

However, the `ArcCoyonedaBase` unsafe impls (lines 106-118) have a subtlety:

```rust
unsafe impl<'a, F, A: 'a> Send for ArcCoyonedaBase<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send,
{
}
```

This is also correct: `Send` is implemented only when the stored `fa` is `Send`. The bound `<F as Kind>::Of<'a, A>: Send` ensures this.

**Impact:** The unsafe code appears sound, but the safety comments could be more precise for future maintainability.

**Approaches:**

A. **Add static assertions.** Use `static_assertions` crate or `const` assertions to verify `Send + Sync` at compile time. Trade-off: adds a dev dependency.

B. **Add compile-fail tests.** Write `trybuild` tests proving that `ArcCoyoneda` with a `!Send` payload fails to compile. Trade-off: some setup effort; good regression coverage.

C. **Use `#[derive]` or auto-traits where possible.** If the compiler can derive `Send`/`Sync` automatically, remove the manual `unsafe` impls. For `ArcCoyonedaMapLayer`, this may not work because the compiler doesn't know that `ArcCoyonedaLowerRef: Send + Sync` implies the `Arc<dyn ArcCoyonedaLowerRef>` is `Send + Sync`.

**Recommendation:** Option B. Compile-fail tests would verify the soundness claims and catch regressions. The manual `unsafe` impls appear correct but benefit from automated verification.

---

### 2.8. `RcCoyoneda::lower_ref` and `ArcCoyoneda::lower_ref` have no memoization

**Files:** `rc_coyoneda.rs` lines 290-294; `arc_coyoneda.rs` lines 326-329

Each call to `lower_ref` re-traverses the entire layer chain and re-applies all mapping functions. For `RcCoyoneda` and `ArcCoyoneda`, which support multiple calls to `lower_ref`, this means repeated computation.

**Impact:** If a user calls `lower_ref` multiple times (e.g., logging, debugging, or using the value in multiple places), the cost is proportional to chain depth times functor size, paid each time. For a `Vec` of 1000 elements with 100 map layers, each `lower_ref` call does 100,000 function applications.

**Approaches:**

A. **Add lazy memoization.** Use `OnceCell`/`LazyCell` (for `RcCoyoneda`) or `OnceLock`/`LazyLock` (for `ArcCoyoneda`) to cache the result of the first `lower_ref` call. Trade-off: additional memory for the cached value; requires `F::Of<'a, A>: Clone` bound anyway (already required for lift), so storing the result is not a new constraint. Complicates the type (the memoization cell must be generic over `F::Of<'a, A>`).

B. **Accept repeated computation.** Users who need memoization can call `lower_ref` once and store the result. Trade-off: simple; no additional complexity.

C. **Provide a `memoize` method.** A method `memoize(self) -> RcCoyoneda<'a, F, A>` that calls `lower_ref`, wraps the result in a fresh base layer, and returns it. Similar to `collapse` but for the Rc/Arc variants. Trade-off: explicit opt-in; clear semantics.

**Recommendation:** Option C. A `memoize` or `collapse` method provides explicit control over when computation is cached. Option A adds hidden complexity (interior mutability inside shared references), which conflicts with the library's preference for explicit control. Option B is acceptable but leaves a performance foot-gun.

---

### 2.9. Missing type class instances on `RcCoyonedaBrand` and `ArcCoyonedaBrand`

**Files:** `rc_coyoneda.rs`, `arc_coyoneda.rs`

Both `RcCoyonedaBrand` and `ArcCoyonedaBrand` implement only `Functor` (Rc only) and `Foldable`. They lack:

- `Pointed` (could delegate to `F::pure` + `lift`, like `CoyonedaBrand`)
- `Semiapplicative` (could delegate to `F::apply` after `lower_ref`, like `CoyonedaBrand`)
- `Lift` (could delegate to `F::lift2` after `lower_ref`)
- `Semimonad` (could delegate to `F::bind` after `lower_ref`)
- `ApplyFirst`/`ApplySecond` (blanket impls; need `Lift`)

`CoyonedaBrand` implements all of these. The Rc/Arc variants should have feature parity where their constraints allow.

Note: `ArcCoyonedaBrand` lacks `Functor`, so it also cannot get the instances that depend on `Functor`. For `RcCoyonedaBrand`, which does have `Functor`, the missing instances are all implementable.

**Impact:** Users of `RcCoyonedaBrand` in generic contexts cannot use `pure`, `bind`, `apply`, or `lift2`. They must drop down to inherent methods or convert to `CoyonedaBrand`.

**Approaches:**

A. **Add the missing instances for `RcCoyonedaBrand`.** These are straightforward: lower via `lower_ref`, delegate to `F`, re-lift. Trade-off: more code, but follows the established pattern.

B. **Add inherent methods on `RcCoyoneda`/`ArcCoyoneda`.** Like `CoyonedaExplicit` does with `apply`, `bind`, `pure`, `fold_map`, etc. Trade-off: usable without brands; no HKT polymorphism.

**Recommendation:** Both A and B. Add trait instances for `RcCoyonedaBrand` where the bounds permit, and add inherent methods on `ArcCoyoneda` for operations that cannot go through the HKT trait system.

---

### 2.10. `CoyonedaExplicitBrand<F, B>` requires `B: 'static`, limiting lifetime flexibility

**File:** `coyoneda_explicit.rs` line 703

```rust
impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B> {
        type Of<'a, A: 'a>: 'a = BoxedCoyonedaExplicit<'a, F, B, A>;
    }
}
```

The `B: 'static` bound is required by the brand system (brands must be `'static` because `impl_kind!` generates implementations where the brand outlives all lifetimes). This means that `CoyonedaExplicitBrand` cannot be used when the base functor contains borrowed data.

The documentation notes this (line 39: `B: 'static required for brand: Yes`), but the practical consequence is that `CoyonedaExplicit` in HKT contexts is limited to owned data at the base layer.

**Impact:** Users cannot create `CoyonedaExplicitBrand<VecBrand, &str>` because `&str` is not `'static`. They must use `String` or other owned types. This is a limitation of the brand system, not specific to `CoyonedaExplicit`.

**Approaches:**

A. **Accept as a brand system limitation.** All brands are `'static` marker types. This is fundamental to the HKT encoding. Trade-off: documented, no action needed.

B. **Use `CoyonedaExplicit` without its brand in non-HKT contexts.** The inherent methods work fine with borrowed data. Trade-off: loses HKT polymorphism.

**Recommendation:** Option A. Document more prominently that this limitation applies only when using `CoyonedaExplicitBrand`, not when using `CoyonedaExplicit` directly.

---

### 2.11. `CoyonedaExplicit::boxed()` in loops causes cascading boxing overhead

**File:** `coyoneda_explicit.rs` lines 547-553; `benches/benchmarks/coyoneda.rs` lines 56-68

The benchmark pattern is:

```rust
let mut coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(v).boxed();
for _ in 0 .. k {
    coyo = coyo.map(|x: i32| x + 1).boxed();
}
coyo.lower()
```

Each iteration:

1. `map` composes the new function with the existing boxed function (creating a closure that calls the boxed function, then applies the new one).
2. `boxed()` re-boxes the composed function.

After k iterations, the function is a chain of k closures, each capturing the previous boxed function. When `lower` calls the function on each element, it traverses k closure indirections. This is O(k) per element, same as `Coyoneda`. The "single F::map call" guarantee still holds, but the composed function itself has O(k) overhead per invocation.

This means `CoyonedaExplicit` with `.boxed()` in a loop does not actually achieve O(n) lowering; it achieves O(k\*n), same as `Coyoneda`. The single `F::map` call is a micro-optimization (one iterator creation instead of k), but the dominant cost (k function applications per element) is identical.

Without `.boxed()` (inline composition), the compiler can potentially inline and optimize the composed function chain, achieving true O(n) lowering. But this requires static type knowledge, which loops destroy.

**Impact:** The `CoyonedaExplicit` + `.boxed()` pattern in loops does not provide the performance advantage it appears to promise. The documentation could be misleading.

**Approaches:**

A. **Clarify documentation.** Explain that `.boxed()` in a loop creates a chain of boxed closures with O(k) per-element overhead, matching `Coyoneda`'s cost profile. The advantage of `CoyonedaExplicit` is realized only with static composition (no `.boxed()` in loops). Trade-off: no code change.

B. **Provide a `compose_boxed` method.** This would eagerly evaluate the composed function at a concrete type level, avoiding closure chaining. Not possible for generic functions.

**Recommendation:** Option A. The documentation should clearly state that `.boxed()` in loops negates the fusion benefit. The real advantage of `CoyonedaExplicit` is static pipelines where the compiler can inline.

---

### 2.12. No conversion paths between `RcCoyoneda`/`ArcCoyoneda` and `Coyoneda`/`CoyonedaExplicit`

**Files:** All four Coyoneda files

`Coyoneda` and `CoyonedaExplicit` have `From` impls for each other (in both directions). Neither `RcCoyoneda` nor `ArcCoyoneda` has any conversion to or from the other variants.

- `Coyoneda` -> `RcCoyoneda`: Cannot be implemented because `Coyoneda` is not `Clone`, and `RcCoyoneda` requires cloning the base layer at `lower_ref` time.
- `RcCoyoneda` -> `Coyoneda`: Could lower via `lower_ref` (cloning the base), then lift into `Coyoneda`. This is lossy (all maps are eagerly applied).
- `CoyonedaExplicit` -> `RcCoyoneda`: Same situation; would require lowering first.
- `RcCoyoneda` -> `ArcCoyoneda`: Not possible without re-wrapping internal `Rc` pointers as `Arc`, which is fundamentally different memory layout.

**Impact:** Users who start with one variant and need another must manually lower and re-lift, which is verbose.

**Approaches:**

A. **Add `From<RcCoyoneda<'a, F, A>> for Coyoneda<'a, F, A> where F: Functor`.** Implemented as `Coyoneda::lift(rc_coyo.lower_ref())`. Trade-off: requires `F: Functor` and clones the base; lossy (maps are applied). Similar for `ArcCoyoneda`.

B. **Add inherent conversion methods.** E.g., `RcCoyoneda::into_coyoneda(self) -> Coyoneda<'a, F, A>` for clearer intent. Trade-off: more explicit than `From`; users see the cost in the method name.

**Recommendation:** Option A for `RcCoyoneda -> Coyoneda` and `ArcCoyoneda -> Coyoneda`. These are natural "downgrade" conversions. The reverse direction (upgrading to shared ownership) genuinely requires the base value to be `Clone`, which the user should provide explicitly.

---

### 2.13. `document_module(no_validation)` on `ArcCoyoneda` suppresses documentation checks

**File:** `arc_coyoneda.rs` line 42

```rust
#[fp_macros::document_module(no_validation)]
```

Both `coyoneda.rs` and `rc_coyoneda.rs` use `#[fp_macros::document_module]` (with validation). `arc_coyoneda.rs` uses `no_validation`. This means documentation attribute macros (`#[document_signature]`, `#[document_type_parameters]`, etc.) are not validated for correctness in this module.

**Impact:** Documentation bugs in `arc_coyoneda.rs` (mismatched parameter counts, wrong descriptions) will not be caught at compile time. This is a quality control gap.

**Approaches:**

A. **Enable validation.** Remove `no_validation` and fix any documentation issues that surface. Trade-off: may require fixing currently-passing but incorrect doc annotations.

B. **Document why validation is disabled.** If there is a technical reason (e.g., the `unsafe impl` blocks confuse the validation macro), add a comment. Trade-off: explains the gap but does not fix it.

**Recommendation:** Option A. Validation should be enabled for consistency. If there are macro limitations, they should be fixed in the macro crate rather than worked around by disabling validation.

---

### 2.14. No property-based tests for any Coyoneda variant

**Files:** All test modules

The test suites for all four variants consist entirely of unit tests with specific values. There are no QuickCheck or proptest-based tests verifying:

- Functor identity law: `map(id, fa) = fa` for arbitrary inputs.
- Functor composition law: `map(f . g, fa) = map(f, map(g, fa))` for arbitrary `f`, `g`, `fa`.
- Foldable consistency: `fold_map` agrees with direct fold on the lowered value.
- Roundtrip laws: `lift(lower(x)) = x` (up to Functor).

The CLAUDE.md explicitly calls for property-based tests as part of the testing strategy.

**Impact:** Edge cases (empty containers, single-element containers, very large values, identity mappings) may not be covered. Law violations would go undetected.

**Approaches:**

A. **Add QuickCheck tests.** Implement `Arbitrary` for `Coyoneda<VecBrand, i32>` (or test the laws directly). Trade-off: requires implementing or using test infrastructure for Coyoneda.

B. **Add proptest tests.** Use proptest strategies to generate Vec/Option values and arbitrary functions. Trade-off: proptest is heavier than QuickCheck but more flexible.

**Recommendation:** Option A. Add QuickCheck tests for the Functor laws on `CoyonedaBrand<VecBrand>` and `CoyonedaBrand<OptionBrand>`, plus the same for `RcCoyonedaBrand` and `CoyonedaExplicitBrand`. These would catch law violations and regressions.

---

### 2.15. `RcCoyoneda::map` and `ArcCoyoneda::map` consume `self`, preventing fluent reuse after clone

**Files:** `rc_coyoneda.rs` line 318-326; `arc_coyoneda.rs` line 356-364

```rust
pub fn map<B: 'a>(
    self,
    f: impl Fn(A) -> B + 'a,
) -> RcCoyoneda<'a, F, B> {
```

The `map` method takes `self` by value. Although `RcCoyoneda` is `Clone`, calling `map` consumes the original. If a user wants to branch (apply different maps to the same base), they must clone explicitly before each `map`.

This is a deliberate design choice (the `Coyoneda` type also consumes `self` on `map`), but for shared-ownership types like `RcCoyoneda`, taking `&self` would be more natural since the inner data is already shared via `Rc`.

**Impact:** Minor ergonomic friction. Users must write `coyo.clone().map(f)` instead of `coyo.map(f)` to preserve the original.

**Approaches:**

A. **Change `map` to take `&self`.** Since the inner `Rc` is cheap to clone, `map` could clone internally. Trade-off: implicit clone on every `map` call; hides allocation cost.

B. **Keep `self` by value.** This makes the clone explicit, which aligns with Rust's ownership philosophy. Trade-off: verbose but transparent.

C. **Add a `map_ref` method taking `&self`.** Users choose between consuming `map` and borrowing `map_ref`. Trade-off: API surface grows.

**Recommendation:** Option B. Consuming `self` is the Rust idiom. The `Clone` impl is O(1), so `coyo.clone().map(f)` is not expensive and makes the sharing explicit.

---

### 2.16. Benchmarks do not cover `RcCoyoneda` or `ArcCoyoneda`

**File:** `benches/benchmarks/coyoneda.rs`

The benchmark only compares Direct vs. `Coyoneda` vs. `CoyonedaExplicit`. The `RcCoyoneda` and `ArcCoyoneda` variants are not benchmarked, despite having different allocation profiles (2 reference-counted allocations per `map` vs. 1 box).

**Impact:** Performance claims about Rc/Arc variants are unverified. The overhead of `Rc::clone` in `lower_ref` and the double allocation per `map` are not quantified.

**Recommendation:** Add benchmark cases for `RcCoyoneda` and `ArcCoyoneda`, both for map-chaining and for multiple `lower_ref` calls on the same value (testing the no-memoization cost).

---

### 2.17. `RcCoyonedaMapLayer::lower_ref` clones the `Rc<dyn Fn>` unnecessarily

**File:** `rc_coyoneda.rs` lines 175-178

```rust
fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
    F: Functor, {
    let lowered = self.inner.lower_ref();
    let func = self.func.clone();
    F::map(move |b| (*func)(b), lowered)
}
```

The `self.func.clone()` clones the `Rc`, which is O(1) (refcount bump). Then the closure captures the cloned `Rc` and dereferences it for each call. This pattern is correct, but the clone is only needed because `F::map` takes the closure by move. An alternative would be to borrow `self.func` inside the closure:

```rust
let func = &self.func;
F::map(move |b| (*func)(b), lowered)
```

Wait, this would not work because `F::map` requires the closure to be `'a`, and borrowing `&self.func` ties the closure's lifetime to `&self`, which may not be `'a`. The `Rc::clone` approach is correct.

The same pattern appears in `ArcCoyoneda` (line 216-217).

**Impact:** Negligible. The `Rc::clone` is O(1). But the code would be clearer with a comment explaining why the clone is necessary.

**Recommendation:** Add a brief comment: `// Clone Rc to move into the closure (Rc::clone is O(1)).`

---

## 3. Summary of Recommendations

| Issue                                         | Priority | Action                                                                       |
| --------------------------------------------- | -------- | ---------------------------------------------------------------------------- |
| 2.3 No stack safety for Rc/Arc variants       | High     | Add `stacker` support, `collapse` method, and documentation.                 |
| 2.7 Unsafe Send/Sync needs compile-fail tests | High     | Add `trybuild` tests verifying soundness of `ArcCoyoneda`.                   |
| 2.9 Missing type class instances              | Medium   | Add `Pointed`, `Semiapplicative`, `Semimonad`, `Lift` for `RcCoyonedaBrand`. |
| 2.13 `no_validation` on ArcCoyoneda docs      | Medium   | Enable validation, fix any issues.                                           |
| 2.14 No property-based tests                  | Medium   | Add QuickCheck tests for Functor and Foldable laws.                          |
| 2.16 Missing benchmarks                       | Medium   | Benchmark Rc/Arc variants.                                                   |
| 2.4 Foldable consumes Rc/Arc value            | Low      | Add inherent `fold_map` taking `&self`.                                      |
| 2.8 No memoization on lower_ref               | Low      | Add `collapse` or `memoize` method.                                          |
| 2.11 Misleading boxed-in-loop performance     | Low      | Clarify documentation.                                                       |
| 2.12 No conversion paths                      | Low      | Add `From` impls for Rc/Arc to Coyoneda.                                     |
| 2.1 Fn vs FnOnce                              | Info     | Inherent in the Functor trait; no action needed.                             |
| 2.2 No map fusion                             | Info     | Inherent limitation; documentation is accurate.                              |
| 2.5 Double traversal in bind                  | Info     | Inherent; documentation is accurate.                                         |
| 2.6 ArcCoyonedaBrand lacks Functor            | Info     | Inherent HKT limitation; documentation is accurate.                          |
| 2.10 `B: 'static` for brand                   | Info     | Brand system limitation; documentation is accurate.                          |
| 2.15 map consumes self on Clone types         | Info     | Correct Rust idiom; no change needed.                                        |
| 2.17 Rc clone in lower_ref                    | Info     | Add explanatory comment.                                                     |
