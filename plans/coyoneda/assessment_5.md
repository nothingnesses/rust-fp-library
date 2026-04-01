# Coyoneda Implementation Assessment

## 1. Overview

This review covers the four Coyoneda implementations in the `fp-library` crate:

- `Coyoneda` (`fp-library/src/types/coyoneda.rs`): The primary free functor using `Box<dyn CoyonedaInner>` for existential quantification. Provides `Functor`, `Pointed`, `Foldable`, `Lift`, `Semiapplicative`, and `Semimonad` via HKT brands. Not `Clone`, not `Send`.
- `CoyonedaExplicit` (`fp-library/src/types/coyoneda_explicit.rs`): Exposes the intermediate type `B` as a type parameter, enabling zero-cost map fusion via `compose`. Provides `Functor` and `Foldable` via its own brand (`CoyonedaExplicitBrand<F, B>`), plus inherent `traverse`, `apply`, `bind`, `fold_map_with_index`, and `hoist`. Not `Clone` (unless the underlying functor value and function are), but the compiler auto-derives `Send` when all components are `Send`.
- `RcCoyoneda` (`fp-library/src/types/rc_coyoneda.rs`): Reference-counted variant wrapping layers in `Rc` for cheap `Clone`. Provides `Functor` and `Foldable` via `RcCoyonedaBrand<F>`. Not `Send`.
- `ArcCoyoneda` (`fp-library/src/types/arc_coyoneda.rs`): Atomically reference-counted variant wrapping layers in `Arc` for `Clone + Send + Sync`. Provides only `Foldable` via `ArcCoyonedaBrand<F>` (no `Functor` due to missing `Send + Sync` bounds on HKT trait closure parameters).

Related files reviewed: `brands.rs` (brand definitions), `kinds.rs` (Kind traits), `classes/functor.rs` (Functor trait), `classes/foldable.rs` (Foldable trait), `classes/semimonad.rs` (Semimonad trait), `benches/benchmarks/coyoneda.rs` (benchmarks).

---

## 2. Identified Flaws, Issues, and Limitations

### 2.1 No Map Fusion in `Coyoneda` (Fundamental Design Limitation)

**File:** `coyoneda.rs`, lines 279-294 (the `lower` method on `CoyonedaMapLayer`)

The `lower` method on `CoyonedaMapLayer` recursively calls `self.inner.lower()` and then `F::map(self.func, lowered)`. For k chained maps, this produces k calls to `F::map`. For eager containers like `Vec`, this means k full traversals of the data.

The module documentation (lines 9-17) explains this clearly: "Each layer calls `F::map` independently, so k chained maps produce k calls to `F::map` at lower time, the same cost as chaining `F::map` directly."

This is a fundamental consequence of Rust's trait object limitations (no generic methods on `dyn` types), and the documentation acknowledges it well. However, the implication is that `Coyoneda` provides no performance benefit over direct `map` calls for eager functors. Its only advantage is deferred evaluation for functors where that matters, and providing a `Functor` instance for non-functor type constructors. This makes the name "free functor" somewhat misleading in terms of performance expectations versus PureScript's single-pass `Coyoneda`.

**Approaches:**

A. **Status quo.** The documentation already explains this. `CoyonedaExplicit` exists as the fusion-capable alternative. No change needed.

B. **Rename or add a prominent performance warning in the type-level docs.** Users coming from Haskell/PureScript may expect `Coyoneda` to provide fusion and be surprised when it does not.

C. **Provide a `lower_fused` method** that attempts to compose the functions at lower time by iterating through layers rather than recursing. This is difficult because each layer hides a different existential type `B`.

**Trade-offs:** Option A is the most pragmatic. Option B improves documentation without code changes. Option C is likely impossible without unsafe code or major redesign due to the existential type boundary.

**Recommendation:** Option A (status quo). The documentation is already thorough. Consider adding a one-line note to the struct doc saying "does not perform map fusion; use `CoyonedaExplicit` for fusion."

---

### 2.2 Missing Stack Safety in `RcCoyoneda` and `ArcCoyoneda`

**Files:** `rc_coyoneda.rs` lines 172-178, `arc_coyoneda.rs` lines 212-218

`Coyoneda` has two stack overflow mitigations: the `stacker` feature (lines 282-288) and the `collapse` method (lines 494-498). Neither `RcCoyoneda` nor `ArcCoyoneda` provides either.

The `lower_ref` method on `RcCoyonedaMapLayer` (line 175) recursively calls `self.inner.lower_ref()`, building up a call stack proportional to the number of chained maps. For deep chains (thousands of maps), this will overflow the stack.

```rust
fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
    F: Functor, {
    let lowered = self.inner.lower_ref();  // recursive call
    let func = self.func.clone();
    F::map(move |b| (*func)(b), lowered)
}
```

**Approaches:**

A. **Add `stacker` support.** Mirror `Coyoneda`'s conditional compilation with `stacker::maybe_grow`. This is straightforward since the recursion structure is identical.

B. **Add a `collapse` method.** For `RcCoyoneda`, `collapse` would call `lower_ref()` (cloning the base value), then re-`lift`. This is semantically correct since `lower_ref` borrows `&self`. For `ArcCoyoneda`, the same approach works.

C. **Both.** Add `stacker` support and `collapse`.

**Trade-offs:** Option A is transparent, but adds a feature dependency. Option B is zero-dependency but requires user awareness. Option C is the most robust. Note that `collapse` for `RcCoyoneda`/`ArcCoyoneda` is more expensive than for `Coyoneda` because `lower_ref` clones the base value, while `Coyoneda::lower` consumes it.

**Recommendation:** Option C. Both mitigations are cheap to implement and consistent with what `Coyoneda` already provides. The `collapse` method is especially important since `stacker` is an opt-in feature.

---

### 2.3 `RcCoyoneda` and `ArcCoyoneda` Lack a Consuming `lower` Method

**Files:** `rc_coyoneda.rs`, `arc_coyoneda.rs`

Both `RcCoyoneda` and `ArcCoyoneda` only provide `lower_ref(&self)`, which always clones the base value. When the `Rc`/`Arc` reference count is 1 (sole owner), a consuming `lower(self)` could avoid the clone by using `Rc::try_unwrap` / `Arc::try_unwrap`.

Currently, even if the user is done with the `RcCoyoneda` and will never use it again, they are forced to pay for a clone:

```rust
let coyo = RcCoyoneda::<VecBrand, _>::lift(vec![1, 2, 3]).map(|x| x + 1);
let result = coyo.lower_ref();  // clones vec![1, 2, 3] even though coyo is about to be dropped
```

**Approaches:**

A. **Add `lower(self)` that attempts `Rc::try_unwrap`/`Arc::try_unwrap`.** If the refcount is 1, unwrap and avoid cloning. If the refcount is > 1, fall back to `lower_ref`.

B. **Add `lower(self)` that calls `lower_ref` and drops self.** Simpler, but still clones. At least the API is consistent with `Coyoneda`.

C. **Do nothing.** The `lower_ref` API is intentional since the primary purpose of `Rc`/`ArcCoyoneda` is cheap sharing.

**Trade-offs:** Option A is the most performant but adds complexity. The `try_unwrap` approach requires restructuring the inner trait to support both consuming and borrowing lowering, which is non-trivial since the layers are `Rc<dyn RcCoyonedaLowerRef>` and the trait only has `lower_ref(&self)`. You would need a second trait method or a separate trait. Option B is trivial but offers no performance benefit. Option C is defensible but inconsistent with `Coyoneda`'s API.

**Recommendation:** Option B as a minimal improvement for API consistency. Option A is worth investigating if profiling shows the clone is a bottleneck, but the implementation complexity is significant.

---

### 2.4 `RcCoyoneda` and `ArcCoyoneda` Missing API Surface

**Files:** `rc_coyoneda.rs`, `arc_coyoneda.rs`

Compared to `Coyoneda`, the `Rc`/`Arc` variants are missing several methods and type class instances:

| Feature                   | `Coyoneda` | `RcCoyoneda` | `ArcCoyoneda` |
| ------------------------- | ---------- | ------------ | ------------- |
| `new(f, fb)`              | Yes        | No           | No            |
| `lift(fa)`                | Yes        | Yes          | Yes           |
| `lower` (consuming)       | Yes        | No           | No            |
| `lower_ref` (borrowing)   | No         | Yes          | Yes           |
| `collapse`                | Yes        | No           | No            |
| `map`                     | Yes        | Yes          | Yes           |
| `hoist`                   | Yes        | No           | No            |
| `Functor` (brand)         | Yes        | Yes          | No            |
| `Pointed` (brand)         | Yes        | No           | No            |
| `Foldable` (brand)        | Yes        | Yes          | Yes           |
| `FoldableWithIndex`       | No         | No           | No            |
| `Lift` (brand)            | Yes        | No           | No            |
| `Semiapplicative` (brand) | Yes        | No           | No            |
| `Semimonad` (brand)       | Yes        | No           | No            |

The `Rc` variants could implement `Pointed`, `Lift`, `Semiapplicative`, and `Semimonad` using the same "lower then delegate to F" pattern that `Coyoneda` uses. The `Arc` variant cannot implement `Functor` (and therefore most other instances) due to the `Send + Sync` bound limitation on HKT closure parameters, but could provide inherent methods for the equivalent operations.

The absence of `new(f, fb)` is notable. `Coyoneda::new` creates a single-layer value from a function and a functor value, avoiding the extra base layer that `lift` + `map` would produce.

**Approaches:**

A. **Add missing methods and instances incrementally.** Prioritize `new`, `collapse`, and `hoist` as inherent methods, then `Pointed` and `Semimonad` as brand instances for `RcCoyonedaBrand`.

B. **Add only the most commonly needed ones.** `new`, `collapse`, and `hoist` cover the core API. Skip brand instances to keep the code lean.

C. **Keep the Rc/Arc variants minimal.** They were designed for the specific use case of cloneable deferred mapping chains. Expanding them risks feature creep.

**Trade-offs:** Option A provides the most complete API but increases maintenance burden and code size. Option B is pragmatic. Option C is defensible but makes the variants less useful.

**Recommendation:** Option B. At minimum, add `new`, `collapse`, and `hoist` to `RcCoyoneda` and `ArcCoyoneda` as inherent methods. These are core Coyoneda operations and their absence is a gap, not a deliberate design choice.

---

### 2.5 `ArcCoyonedaBrand` Cannot Implement `Functor`

**File:** `arc_coyoneda.rs`, lines 375-379

The comment explains: "The HKT trait signatures lack Send + Sync bounds on closure parameters, so there is no way to guarantee that closures passed to map are safe to store inside an Arc-wrapped layer."

This is a fundamental limitation of the library's HKT design. The `Functor::map` signature is:

```rust
fn map<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> B + 'a,
    fa: ...,
) -> ...;
```

Note that `f` is `impl Fn(A) -> B + 'a`, with no `Send + Sync` bound. `ArcCoyoneda::map` requires `impl Fn(A) -> B + Send + Sync + 'a`. There is no way to bridge this gap without changing the `Functor` trait.

This means `ArcCoyoneda` cannot participate in generic `Functor`-polymorphic code via its brand. Users must use it directly through inherent methods.

**Approaches:**

A. **Status quo with documentation.** The comment at lines 375-379 explains the limitation. The module doc (lines 17-23) also documents it.

B. **Add a `SendFunctor` trait** with `Send + Sync` bounds on the closure. This would be a library-wide change affecting many modules.

C. **Provide a `UnsafeFunctor` blanket impl** that transmutes the closure. Definitely not recommended.

D. **Use a different approach for the `ArcCoyoneda` brand.** Instead of implementing the standard `Functor`, provide a separate `ArcFunctor` type class specific to `Arc`-based types.

**Trade-offs:** Option A is the simplest and most honest. Option B is a significant design change with far-reaching implications; it would need to be justified by more than just `ArcCoyoneda`. Option D adds complexity without solving the fundamental interop problem.

**Recommendation:** Option A. The limitation is inherent to the `Functor` trait design and affects all `Send`-requiring types (this is the same issue as `SendThunkBrand`). A `SendFunctor` trait might be worth exploring as a broader library improvement, but it should not be driven solely by `ArcCoyoneda`.

---

### 2.6 `RcCoyonedaMapLayer::lower_ref` and `ArcCoyonedaMapLayer::lower_ref` Clone the Function Pointer Unnecessarily per Element Application

**Files:** `rc_coyoneda.rs` line 176, `arc_coyoneda.rs` line 216

```rust
fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
    F: Functor, {
    let lowered = self.inner.lower_ref();
    let func = self.func.clone();  // Rc/Arc bump per lower_ref call
    F::map(move |b| (*func)(b), lowered)
}
```

Each `lower_ref` call clones the `Rc<dyn Fn(B) -> A>` (or `Arc` equivalent), incrementing the reference count. This is a minor overhead per `lower_ref` call, not per element. The clone is necessary because the closure passed to `F::map` must have lifetime `'a`, but `&self` has a shorter anonymous lifetime. The `Rc`/`Arc` clone ensures the function outlives the closure.

However, there is an additional indirection cost: the closure `move |b| (*func)(b)` dereferences the `Rc`/`Arc` and then dynamically dispatches through the `dyn Fn` vtable. This is two levels of indirection per element mapped (one `Rc`/`Arc` deref, one vtable dispatch). In contrast, `Coyoneda`'s `CoyonedaMapLayer` stores the function inline (monomorphized `Func`) and calls `F::map(self.func, lowered)` with zero indirection.

For `RcCoyoneda` and `ArcCoyoneda`, each map layer adds:

1. One `Rc`/`Arc` clone (refcount bump) at lower time.
2. One `Rc`/`Arc` deref + one vtable dispatch per element at lower time.

For k chained maps and n elements, this is k refcount bumps + k \* n double-indirections.

**Approaches:**

A. **Accept the overhead.** This is inherent to the `Rc`/`Arc` design. Users choosing `RcCoyoneda` are already paying for reference counting.

B. **Store the function inline using a generic parameter**, like `CoyonedaMapLayer` does. The problem is that `RcCoyonedaMapLayer` cannot have a generic `Func` parameter because it must be stored behind `Rc<dyn RcCoyonedaLowerRef>`, which erases the type. The `Rc`-wrapping is what enables `Clone`. So the double-indirection is the cost of cloneability.

C. **Use a hybrid approach** where the function is stored inline in the struct (monomorphized) but the trait object erases it. This is actually what happens: `RcCoyonedaMapLayer` is a concrete type with the function as `Rc<dyn Fn(B) -> A>`. The `Rc<dyn Fn>` is already the minimal cost for a cloneable dynamically typed function.

**Trade-offs:** The overhead is inherent to the design goals. There is no way to have both `Clone` and zero-cost function storage.

**Recommendation:** Option A. Document the performance characteristics more prominently. The module doc mentions "Each map allocates 2 Rc values" but does not mention the per-element double-indirection cost during lowering.

---

### 2.7 `CoyonedaExplicit` Compile-Time Explosion for Deep Chains

**File:** `coyoneda_explicit.rs`, lines 210-219

Each `.map()` call on `CoyonedaExplicit` nests the function type deeper:

```rust
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C + 'a> {
    CoyonedaExplicit {
        fb: self.fb,
        func: compose(f, self.func),
        _phantom: PhantomData,
    }
}
```

After k maps, the function type is `compose(fk, compose(fk-1, ... compose(f2, f1)))`, which is a deeply nested generic type. The documentation (lines 16-17) warns: "For chains deeper than ~20-30 maps, consider inserting `.boxed()` to bound compile-time type complexity."

This is well-documented. However, the `.boxed()` escape hatch reintroduces a `Box` allocation and dynamic dispatch, which partially negates the zero-cost fusion benefit. More importantly, `.boxed()` in a loop means each iteration allocates a new `Box`, so the benchmark (`benches/benchmarks/coyoneda.rs` lines 60-63) shows:

```rust
let mut coyo = CoyonedaExplicit::<VecBrand, _, _, _>::lift(v).boxed();
for _ in 0 .. k {
    coyo = coyo.map(|x: i32| x + 1).boxed();
}
coyo.lower()
```

This allocates k `Box` values, which is the same allocation count as `Coyoneda`. The fusion benefit is that only one `F::map` call happens at lower time, but the intermediate boxes may negate the savings for small containers.

**Approaches:**

A. **Status quo.** The documentation is clear about the trade-off. Users can choose between raw `CoyonedaExplicit` (zero-cost for shallow chains) and boxed (one allocation per map but single `F::map`).

B. **Provide a `compose_boxed` method** that boxes the composition in a single step, reducing the double allocation (one for the compose result, one for the box). Currently `.map(f).boxed()` creates the compose, then moves it into a `Box`.

C. **Provide a stack-allocated small-function optimization** using something like `SmallBox`. This avoids heap allocation for small closures.

**Trade-offs:** Option A is the simplest. Option B provides a minor optimization. Option C adds a dependency and complexity.

**Recommendation:** Option A. The current design is sound. The boxed-per-iteration pattern is an inherent trade-off when a uniform type is needed. For performance-critical code, users should compose functions manually before a single `map` call.

---

### 2.8 `CoyonedaExplicitBrand` Requires `B: 'static`

**File:** `coyoneda_explicit.rs`, line 703

```rust
impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B> {
        type Of<'a, A: 'a>: 'a = BoxedCoyonedaExplicit<'a, F, B, A>;
    }
}
```

The `B: 'static` bound means `CoyonedaExplicitBrand` cannot be used with intermediate types that contain non-`'static` references. This is a constraint of the HKT encoding: the `Kind` trait's associated type `Of<'a, A>` introduces its own lifetime `'a`, so type parameters baked into the brand must outlive all possible `'a`.

This is the same constraint as `F: 'static` on `CoyonedaBrand<F>` (line 585 of `coyoneda.rs`) and is inherent to the brand pattern. The documentation on `CoyonedaBrand` (line 139-142 of `brands.rs`) explains: "In practice this is not a restriction because all brands in the library are zero-sized marker types, which are inherently `'static`." But for `B`, which is an actual data type (not a brand), this is a genuine restriction.

**Approaches:**

A. **Status quo with documentation.** The table in the module doc (line 38) already notes "B: 'static required for brand | Yes."

B. **Explore a brand design that does not require `'static`.** This would require changes to the `Kind` trait machinery.

**Trade-offs:** Option A is honest. Option B is a deep library design change.

**Recommendation:** Option A. This is a known limitation of the brand pattern and not specific to Coyoneda.

---

### 2.9 `Coyoneda::bind` and `Coyoneda::hoist` Double-Lower for Chained Computations

**File:** `coyoneda.rs`, line 858

```rust
fn bind<'a, A: 'a, B: 'a>(
    ma: ...,
    func: impl Fn(A) -> ... + 'a,
) -> ... {
    Coyoneda::lift(F::bind(ma.lower(), move |a| func(a).lower()))
}
```

When `bind` is chained, each call lowers the input `Coyoneda`, binds, and re-lifts. The callback `func(a)` returns another `Coyoneda`, which is immediately `.lower()`-ed. This means every monadic step forces all accumulated maps, even if they could have been deferred further.

For example:

```rust
let result = bind(
    bind(
        Coyoneda::lift(v).map(f).map(g),
        |a| Coyoneda::lift(Some(a)).map(h)
    ),
    |b| Coyoneda::lift(Some(b))
);
```

The inner `bind` calls `lower()` twice: once for `ma` (applying f and g), once for the callback result (applying h). The outer `bind` calls `lower()` twice more. Every deferred map is eagerly applied at every `bind` boundary.

This is correct behavior (the monad laws are satisfied), but it means `Coyoneda` provides no deferral benefit across monadic boundaries. PureScript's Coyoneda has the same issue since `unCoyoneda` is called at each `bind`.

**Approaches:**

A. **Status quo.** This is inherent to how `Coyoneda` interacts with `bind`. The Free monad (`Free<F, A>`) is the proper solution for deferred monadic computation.

B. **Document this behavior.** Add a note that `bind` forces all accumulated maps.

**Trade-offs:** Option B is cheap and informative.

**Recommendation:** Option B. Add a brief note to `Semimonad`'s doc comment.

---

### 2.10 No `Debug` Implementation

**Files:** All four Coyoneda files.

None of the four Coyoneda types implement `Debug`. This makes debugging difficult. `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` store `Box<dyn Trait>` or `Rc<dyn Trait>`/`Arc<dyn Trait>` internally, so a derived `Debug` is not possible.

However, a manual `Debug` implementation could print structural information (e.g., "Coyoneda(... k layers ...)") or at least "Coyoneda(<opaque>)".

**Approaches:**

A. **Implement `Debug` with opaque output.** Print `Coyoneda(<opaque>)` or similar.

B. **Implement `Debug` with layer count.** Add a method to count layers (requires adding `depth` to the inner trait) and print `Coyoneda(<depth=3>)`.

C. **Do nothing.** Many functional types in the library likely lack `Debug` since they store closures.

**Trade-offs:** Option A is trivial. Option B is slightly more informative but requires trait changes. Option C is consistent with library conventions.

**Recommendation:** Option A if other closure-containing types in the library implement `Debug`; otherwise Option C for consistency.

---

### 2.11 No Benchmarks for `RcCoyoneda` or `ArcCoyoneda`

**File:** `benches/benchmarks/coyoneda.rs`

The benchmark file only compares Direct vs `Coyoneda` vs `CoyonedaExplicit` (boxed). `RcCoyoneda` and `ArcCoyoneda` are not benchmarked, making it impossible to quantify the overhead of the `Rc`/`Arc` wrapping and double-indirection.

**Approaches:**

A. **Add benchmark cases for `RcCoyoneda` and `ArcCoyoneda`.** Include both `lower_ref` and the `Rc`/`Arc` clone overhead.

B. **Add benchmark cases comparing `lower_ref` called once vs multiple times** on the same `RcCoyoneda`, to measure the clone cost.

**Trade-offs:** Both options are straightforward to implement with minimal effort.

**Recommendation:** Options A and B. Benchmarking is essential for types whose primary justification involves performance trade-offs.

---

### 2.12 `CoyonedaExplicit::traverse` Returns a Type with `fn(C) -> C`

**File:** `coyoneda_explicit.rs`, lines 413-426

```rust
pub fn traverse<G: Applicative + 'a, C: 'a + Clone>(
    self,
    f: impl Fn(A) -> <G as Kind_cdc7cd43dac7585f>::Of<'a, C> + 'a,
) -> <G as Kind_cdc7cd43dac7585f>::Of<'a, CoyonedaExplicit<'a, F, C, C, fn(C) -> C>>
```

The return type wraps the result in `CoyonedaExplicit<'a, F, C, C, fn(C) -> C>`, meaning the accumulated function is reset to the identity. This is correct: after traversal, the accumulated function has been applied (it was composed with `f` and delegated to `F::traverse`), so the result is in identity position.

However, this means that any maps accumulated before `traverse` are materialized during the traversal call itself (via `compose(f, self.func)`), and the resulting `CoyonedaExplicit` starts fresh with identity. The fusion pipeline is broken across `traverse` boundaries. This is semantically correct and matches PureScript's behavior, but may surprise users expecting full laziness.

The same pattern appears in `apply` (line 485) and `bind` (line 520-524). All three operations are "fusion barriers" that reset the pipeline.

**Approaches:**

A. **Document the fusion barrier behavior.** The `apply` method already documents it (line 437: "This is a fusion barrier"), but `traverse` and `bind` do not.

B. **Status quo.** The behavior is inherent and correct.

**Trade-offs:** Option A improves documentation at no cost.

**Recommendation:** Option A. Add "This is a fusion barrier" notes to `traverse` and `bind` doc comments.

---

### 2.13 `Coyoneda` `Semiapplicative` and `Semimonad` Require `F: Functor`

**File:** `coyoneda.rs`, lines 773, 820

```rust
impl<F: Functor + Semiapplicative + 'static> Semiapplicative for CoyonedaBrand<F> { ... }
impl<F: Functor + Semimonad + 'static> Semimonad for CoyonedaBrand<F> { ... }
```

Both require `F: Functor` because they call `lower()` internally, which applies accumulated maps via `F::map`. In PureScript, `Coyoneda f` gets `Monad` from `Monad f` without requiring `Functor f` separately, because `unCoyoneda` can open the existential directly.

This means `Coyoneda` cannot provide `Semiapplicative` or `Semimonad` for non-`Functor` type constructors, even if those constructors implement `Semiapplicative`/`Semimonad`. This partially defeats the purpose of the free functor construction ("provides a Functor for any type constructor").

**Approaches:**

A. **Status quo.** The `Functor` requirement is inherent to the layered trait-object encoding. It cannot be removed without fundamental redesign.

B. **Add inner trait methods for `apply_inner` and `bind_inner`.** These would compose the accumulated function with the `Semiapplicative`/`Semimonad` operation before delegating to `F`. However, these methods would need to be generic (breaking dyn-compatibility) or require additional trait object layers.

C. **Use `CoyonedaExplicit` for these operations.** It already provides inherent `apply` and `bind` that work without requiring `F: Functor` (they compose the accumulated function with the callback and delegate directly to `F`).

**Trade-offs:** Option A is the most pragmatic. Option B is likely impossible. Option C is already available.

**Recommendation:** Option A. Add a note in the documentation pointing users to `CoyonedaExplicit`'s inherent methods if they need `apply`/`bind` without `F: Functor`.

---

### 2.14 No `Eq`, `PartialEq`, or `Hash` Implementations

**Files:** All four Coyoneda files.

None of the types implement `Eq`, `PartialEq`, or `Hash`. For `Coyoneda` and the Rc/Arc variants, this would require lowering (applying `F: Functor`) and comparing the results. For `CoyonedaExplicit`, it would require comparing both the stored functor value and the function, which is generally impossible for closures.

PureScript provides `Eq` for `Coyoneda f a` when `f` is a `Functor` and `f a` has `Eq`. This could be done in Rust via:

```rust
impl<F: Functor + Kind, A: PartialEq> PartialEq for Coyoneda<'_, F, A>
where F::Of<'_, A>: PartialEq { ... }
```

The implementation would `lower()` both sides and compare. This consumes both values though (since `lower` takes `self`).

**Approaches:**

A. **Do nothing.** Comparing lazily-evaluated structures is semantically questionable and consumes the values.

B. **Implement `PartialEq` for `RcCoyoneda` and `ArcCoyoneda`** since they have `lower_ref(&self)`. This is feasible and does not consume the values.

**Trade-offs:** Option B is useful for testing and assertions. The cost is one `lower_ref` call per comparison, but this matches what users would write manually.

**Recommendation:** Option B for `RcCoyoneda` and `ArcCoyoneda` where it is both safe and non-consuming. Skip it for `Coyoneda` and `CoyonedaExplicit` where it would be consuming.

---

### 2.15 `Foldable` Implementation for `RcCoyonedaBrand`/`ArcCoyonedaBrand` Consumes the Value

**Files:** `rc_coyoneda.rs` lines 413-421, `arc_coyoneda.rs` lines 417-425

```rust
fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
    func: impl Fn(A) -> M + 'a,
    fa: Apply!(...),
) -> M { ... }
```

The `Foldable::fold_map` trait method takes `fa` by value. For `RcCoyoneda`, this drops the `RcCoyoneda` after folding, even though `lower_ref` only borrows it. This is a trait design constraint, not a bug in the Coyoneda implementation.

However, since `RcCoyoneda` is `Clone`, users can clone before folding if they need to retain the value. The cost is one `Rc` refcount bump, which is cheap.

**Approaches:**

A. **Status quo.** This is a consequence of the `Foldable` trait design.

B. **Add inherent `fold_map` and `fold_map_with_index` methods** that take `&self`, parallel to `CoyonedaExplicit`'s inherent methods. These would provide a non-consuming fold path.

**Trade-offs:** Option B adds convenience without changing the trait. It is a small amount of code.

**Recommendation:** Option B. Inherent `fold_map` on `RcCoyoneda`/`ArcCoyoneda` taking `&self` would be a natural complement to `lower_ref`.

---

### 2.16 `ArcCoyonedaBase` Unsafe Send/Sync Impls Are Sound but Could Be Conditional

**File:** `arc_coyoneda.rs`, lines 103-118

```rust
unsafe impl<'a, F, A: 'a> Send for ArcCoyonedaBase<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send,
{ }
```

The `unsafe impl Send` is guarded by `Of<'a, A>: Send`, which is correct: the struct only contains `fa: Of<'a, A>`, so it is `Send` if and only if `fa` is `Send`.

The `ArcCoyonedaMapLayer` unsafe impls (lines 171-184) are also sound: both fields are `Arc<dyn ... + Send + Sync>`, which are `Send + Sync` by Arc's guarantees.

However, the unsafe impls are necessary only because the compiler cannot auto-derive `Send`/`Sync` for types with complex generic bounds. A safer alternative would be to add `Send + Sync` bounds to the struct definition itself, allowing the compiler to auto-derive the marker traits.

**Approaches:**

A. **Status quo.** The unsafe impls are correct and well-commented.

B. **Restructure to avoid `unsafe impl`.** Add `where` clauses to the struct definitions that constrain fields to be `Send + Sync`, and let the compiler auto-derive.

**Trade-offs:** Option B reduces `unsafe` surface area but may make the struct definitions more verbose and harder to construct in generic contexts.

**Recommendation:** Option A. The `unsafe` code is minimal, well-commented, and correct. The safety reasoning is straightforward.

---

### 2.17 `document_module(no_validation)` on `ArcCoyoneda`

**File:** `arc_coyoneda.rs`, line 42

```rust
#[fp_macros::document_module(no_validation)]
```

The `ArcCoyoneda` module uses `no_validation` to suppress documentation validation warnings. All other Coyoneda modules use the default `document_module` (with validation). This suggests there may be documentation attributes missing or incorrect in the `ArcCoyoneda` module.

**Approaches:**

A. **Fix the documentation issues and remove `no_validation`.** Identify what validation failures occur and fix them.

B. **Status quo.** If the validation issues are false positives or not worth fixing.

**Trade-offs:** Option A ensures documentation consistency across all Coyoneda modules.

**Recommendation:** Option A. Investigate what triggers the validation failures and fix them. Consistency with the other modules is important.

---

### 2.18 Asymmetric `map` Signature: `self` vs `&self`

**Files:** All four Coyoneda files.

`Coyoneda::map` and `CoyonedaExplicit::map` take `self` (consuming). `RcCoyoneda::map` and `ArcCoyoneda::map` also take `self` (consuming), despite `RcCoyoneda` and `ArcCoyoneda` being `Clone`.

For `RcCoyoneda`/`ArcCoyoneda`, consuming `self` means the original value is moved into the new map layer. If the user wants to keep a copy, they must clone first:

```rust
let coyo = RcCoyoneda::lift(vec![1, 2, 3]);
let coyo2 = coyo.clone();  // must clone before map
let mapped = coyo.map(|x| x + 1);
```

This is fine and consistent with Rust's ownership model. But an alternative would be to have `map` take `&self` for the Rc/Arc variants, since the inner data is reference-counted:

```rust
pub fn map<B: 'a>(&self, f: impl Fn(A) -> B + 'a) -> RcCoyoneda<'a, F, B> {
    RcCoyoneda(Rc::new(RcCoyonedaMapLayer {
        inner: self.0.clone(),  // Rc clone is cheap
        func: Rc::new(f),
    }))
}
```

**Approaches:**

A. **Status quo.** Consuming `self` is idiomatic Rust and the Rc clone in the function body (via `self.0`) is implicit. Users who need both the original and the mapped version clone first.

B. **Change `map` to take `&self`.** This is more ergonomic for the Rc/Arc variants and mirrors how `lower_ref` takes `&self`.

**Trade-offs:** Option B is more ergonomic but deviates from the `Coyoneda`/`CoyonedaExplicit` API. Option A is consistent across all variants. The `Functor::map` trait takes `fa` by value, so the brand-level `Functor` implementation would need to take ownership anyway. Having the inherent `map` differ from the brand-level `map` in ownership semantics could cause confusion.

**Recommendation:** Option A. Consistency with the broader API is more important than the minor ergonomic gain. The `Clone` impl on `RcCoyoneda`/`ArcCoyoneda` makes the explicit clone cheap and obvious.

---

## 3. Summary of Recommendations

| Issue                              | Priority | Recommendation                                        |
| ---------------------------------- | -------- | ----------------------------------------------------- |
| 2.1 No map fusion                  | Low      | Status quo; documentation is adequate                 |
| 2.2 Missing stack safety in Rc/Arc | High     | Add `stacker` support and `collapse`                  |
| 2.3 Missing consuming `lower`      | Medium   | Add `lower(self)` that delegates to `lower_ref`       |
| 2.4 Missing API surface            | Medium   | Add `new`, `collapse`, `hoist` to Rc/Arc variants     |
| 2.5 Arc cannot impl Functor        | Low      | Status quo; inherent to HKT design                    |
| 2.6 Double-indirection in Rc/Arc   | Low      | Status quo; document the per-element cost             |
| 2.7 Compile-time explosion         | Low      | Status quo; well-documented                           |
| 2.8 B: 'static on brand            | Low      | Status quo; inherent to brand pattern                 |
| 2.9 bind double-lowers             | Low      | Add documentation note                                |
| 2.10 No Debug                      | Low      | Consider `Debug` with opaque output                   |
| 2.11 No Rc/Arc benchmarks          | Medium   | Add benchmark cases                                   |
| 2.12 Traverse fusion barrier       | Low      | Add documentation notes                               |
| 2.13 Semiapplicative needs Functor | Low      | Status quo with doc note pointing to CoyonedaExplicit |
| 2.14 No Eq/PartialEq               | Low      | Implement for RcCoyoneda/ArcCoyoneda                  |
| 2.15 Foldable consumes Rc value    | Low      | Add inherent `fold_map(&self)` methods                |
| 2.16 Unsafe Send/Sync              | Low      | Status quo; impls are correct                         |
| 2.17 no_validation on ArcCoyoneda  | Low      | Fix docs and remove no_validation                     |
| 2.18 map takes self on Rc/Arc      | Low      | Status quo; consistency is more important             |

The highest-priority item is **2.2 (stack safety)** because it can cause runtime crashes. Items **2.3**, **2.4**, and **2.11** are medium priority because they affect usability and verifiability. The remaining items are low priority, involving documentation improvements or design trade-offs that are already well-understood.
