# Coyoneda Implementations: Analysis of Flaws, Issues, and Limitations

This document analyzes the two Coyoneda implementations in the library:

- `Coyoneda` (`fp-library/src/types/coyoneda.rs`) -- the HKT-integrated, trait-object-based variant.
- `CoyonedaExplicit` (`fp-library/src/types/coyoneda_explicit.rs`) -- the explicit-type-parameter variant claiming zero-cost map fusion.

For each issue, I describe the problem, reference exact code locations, propose approaches to address it, discuss trade-offs, and give a recommendation.

---

## Issue 1: CoyonedaExplicit's `map` Still Allocates a Box per Call

**Location:** `coyoneda_explicit.rs`, lines 167-175.

**Problem:** The module documentation (line 1) and the comparison table (line 22) claim "zero-cost map fusion" with "0" heap allocations per map. However, each call to `map` creates a `Box::new(compose(f, self.func))` (line 174). The `compose` call itself is zero-cost (it returns an `impl Fn`), but wrapping the result in a new `Box<dyn Fn(B) -> C>` is a heap allocation with dynamic dispatch. This means each `map` call:

1. Allocates a new `Box` on the heap.
2. Drops the old `Box`.
3. Introduces a `dyn Fn` vtable indirection for every composed layer.

For k chained maps, this performs k heap allocations (one per `map` call). The documentation's claim of "0 heap allocations per map" is inaccurate. The only improvement over `Coyoneda` is that `lower` calls `F::map` once instead of k times, which avoids k intermediate container allocations (e.g., k `Vec` allocations). But the function composition chain itself is not zero-cost.

Additionally, `lift` (line 434) also boxes the identity function unnecessarily, performing a heap allocation even when no mapping has been requested.

**Approaches:**

### A. Remove Box, use generic type parameter for the function

Replace `Box<dyn Fn(B) -> A + 'a>` with a generic type parameter `K: Fn(B) -> A + 'a`:

```rust
pub struct CoyonedaExplicit<'a, F, B: 'a, A: 'a, K: Fn(B) -> A + 'a>
where
    F: Kind_cdc7cd43dac7585f + 'a,
{
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: K,
    _phantom: PhantomData<&'a (B, A)>,
}
```

Each `map` returns a `CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C>` where the composed function is inlined. No heap allocation, no dynamic dispatch.

**Trade-offs:** The function type becomes part of the struct's generic signature, making the type unnameable in many contexts. Users cannot store heterogeneous `CoyonedaExplicit` values in a collection. The type grows in complexity with each `map` call due to nested `compose` closures. However, since `CoyonedaExplicit` is explicitly documented as not having HKT integration, unnameability is an acceptable cost.

### B. Keep Box but fix the documentation

Simply correct the documentation to say "single-pass fusion (1 call to `F::map`)" rather than "zero-cost" or "0 heap allocations." This is the lowest-effort fix and accurately represents the current behavior: one box allocation per `map` call, but only one `F::map` call at `lower` time.

### C. Use `FnOnce` instead of `Fn`

Since `CoyonedaExplicit` is consumed by `lower`, `map`, etc. (all take `self`), the stored function only needs to be called once. Using `FnOnce` instead of `Fn` would allow more functions to be stored without cloning. This does not eliminate the boxing but makes the API more permissive. Combined with approach A, `FnOnce` closures would be fully inlined.

**Recommendation:** Approach A (generic function parameter) is the correct fix for a type that advertises zero-cost fusion. If the ergonomic cost of the unnameable type is deemed too high, approach B (fix documentation) is the minimum required change. Approach C (FnOnce) should be applied regardless, since the struct is move-only.

---

## Issue 2: Coyoneda Does Not Achieve Map Fusion

**Location:** `coyoneda.rs`, lines 244-249 (`CoyonedaMapLayer::lower`), lines 394-402 (`Coyoneda::map`).

**Problem:** Each `map` call creates a new `CoyonedaMapLayer` wrapping the previous layer in a `Box<dyn CoyonedaInner>` with a `Box<dyn Fn(B) -> A>`. At `lower` time, each layer calls `F::map` independently (line 248). For k chained maps on a `Vec`, this produces k full `Vec` allocations and k traversals. The module documentation (lines 38-44) correctly documents this, but it means `Coyoneda` provides no performance benefit over calling `F::map` directly; it only provides the ability to defer the `Functor` constraint.

This is 2 heap allocations per `map` call (one for the `CoyonedaInner` trait object, one for the `dyn Fn` closure), plus k calls to `F::map` at `lower` time.

**Approaches:**

### A. Accept the limitation; rely on CoyonedaExplicit for fusion

The current documentation already explains this limitation. The purpose of `Coyoneda` is HKT integration (it has a `CoyonedaBrand` and `Functor` instance), not performance. Users who need fusion should use `CoyonedaExplicit` and convert via `into_coyoneda` when needed.

### B. Attempt function composition via unsafe type erasure

Store a single `Box<dyn FnOnce(???) -> A>` alongside a `Box<dyn Any>` (the erased `F B`), composing functions eagerly by transmuting the intermediate types. This is extremely fragile and unsound in general.

### C. Use an enum-based approach with type-erased function chains

Store a `Vec<Box<dyn Any>>` of functions and compose them at `lower` time via `downcast`. This adds runtime type checking overhead and panic risk.

**Recommendation:** Approach A. The trait-object boundary is a fundamental Rust limitation. The library already provides `CoyonedaExplicit` for fusion and the documentation is honest about the limitation. No change needed beyond ensuring users are guided to `CoyonedaExplicit` when performance matters.

---

## Issue 3: CoyonedaExplicit Uses `Fn` Where `FnOnce` Would Suffice

**Location:** `coyoneda_explicit.rs`, lines 96, 131, 168-169, 290.

**Problem:** The stored function is `Box<dyn Fn(B) -> A + 'a>` (line 96), and `new` accepts `impl Fn(B) -> A + 'a` (line 131), `map` accepts `impl Fn(A) -> C + 'a` (line 168). Since `CoyonedaExplicit` is always consumed (all methods take `self` by value), the function only needs to be called once per element. Using `Fn` instead of `FnOnce` prevents users from passing closures that capture non-Clone, move-only values.

For example, this will not compile:

```rust
let name = String::from("hello");
CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(1))
    .map(move |_| name)  // Error: cannot move out of captured variable in Fn
```

The same issue applies to `Coyoneda` (line 307, 396), but there it is somewhat forced by the layered architecture where `F::map` takes `impl Fn`.

**Root cause:** The library's `Functor::map` signature (functor.rs, line 121) takes `impl Fn(A) -> B + 'a`, not `impl FnOnce(A) -> B`. This is because `Functor::map` may need to call the function multiple times (e.g., once per element in a `Vec`). So the constraint propagates.

**Approaches:**

### A. Accept the constraint for lower/fold_map, use FnOnce for map composition

The `map` method does not call the function; it only stores/composes it. Using `FnOnce` for `map`'s parameter is fine as long as the composition result satisfies `Fn` (which it does, since `compose` returns `impl Fn` when both inputs are `Fn`). However, the stored function must eventually be passed to `F::map` which requires `Fn`, so the stored function must be `Fn`. This means `FnOnce` in `map` alone does not help; the composition must produce an `Fn`.

### B. Split into FnOnce-based CoyonedaExplicit for single-element containers

For containers that call the function exactly once (e.g., `Option`, `Identity`), a variant using `FnOnce` would be strictly more permissive. This adds API surface area.

**Recommendation:** No change. The `Fn` requirement is inherent to how `Functor::map` works for multi-element containers. The documentation should note this constraint.

---

## Issue 4: `fold_map` Requires `B: Clone` Unnecessarily

**Location:** `coyoneda_explicit.rs`, line 286.

**Problem:** The `fold_map` method has a `B: Clone` bound. This comes from the `Foldable::fold_map` signature requiring `A: Clone` on the element type. However, `CoyonedaExplicit`'s `fold_map` composes the fold function with the accumulated mapping function, so the fold function receives values of type `A` (the output), not `B` (the stored type). The `B: Clone` bound appears because `Foldable::fold_map` requires `A: Clone` and it is called with type parameter `B` as the element type.

Looking at the call: `F::fold_map::<FnBrand, B, M>(compose(func, self.func), self.fb)`. Here, `F::fold_map` is called with element type `B`, so `Foldable`'s constraint demands `B: Clone`. This is a real requirement imposed by the library's `Foldable` trait design, not a bug in `CoyonedaExplicit`. But it means users cannot `fold_map` over a `CoyonedaExplicit` whose stored type `B` is not `Clone`, which limits usability with move-only inner types.

**Approaches:**

### A. Redesign Foldable to not require Clone

The `Clone` requirement on `Foldable::fold_map`'s element type exists because `CloneableFn` wraps functions in `Rc`/`Arc` which requires cloning. This is a deeper library design issue beyond Coyoneda.

### B. Accept the limitation

Document that `fold_map` requires `B: Clone` and explain why.

**Recommendation:** Approach B for now. The `Clone` bound comes from `Foldable`'s design and is not specific to Coyoneda.

---

## Issue 5: `apply` and `bind` Destroy the Fusion Pipeline

**Location:** `coyoneda_explicit.rs`, lines 358-366 (`apply`), lines 395-402 (`bind`).

**Problem:** Both `apply` and `bind` call `self.lower()` and `ff.lower()`, which collapses the accumulated maps into the underlying functor. They then re-lift the result with `CoyonedaExplicit::lift(...)`, resetting the fusion pipeline to an identity function. This means:

1. Any maps accumulated before `apply`/`bind` are fused into a single `F::map` call (good).
2. But the `apply`/`bind` operation itself forces materialization of intermediate containers.
3. Maps after `apply`/`bind` start a new pipeline from scratch.

For a chain like `.map(f).map(g).bind(h).map(i).map(j)`, this produces: one `F::map` call (fusing f and g), one `F::bind` call, one `F::map` call (fusing i and j), and two intermediate container materializations. This is better than no fusion at all (which would be 5 `F::map` calls plus the bind), but the forced materialization at `apply`/`bind` boundaries is a significant limitation.

Furthermore, `apply` requires `F: Semiapplicative` and `bind` requires `F: Functor + Semimonad`, which are strong constraints. PureScript's `Coyoneda` derives `Apply` and `Bind` from the underlying functor's instances, but `CoyonedaExplicit` cannot participate in the type class hierarchy (no brand), so these are ad-hoc methods.

**Approaches:**

### A. Accept the limitation with clear documentation

Fusion across monadic boundaries is fundamentally difficult. Even in Haskell with GHC rewrite rules, fusion across `>>=` is not automatic. The current behavior of fusing within each "segment" between effectful operations is reasonable.

### B. Provide a `map_then_bind` combinator

A single method that composes the accumulated maps and the bind function, avoiding the intermediate `F::map` + `F::bind` and instead doing a single `F::bind` with the composed function:

```rust
pub fn map_then_bind<C: 'a>(
    self,
    f: impl Fn(A) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, C> + 'a,
) -> CoyonedaExplicit<'a, F, C, C>
where
    F: Semimonad,
{
    CoyonedaExplicit::lift(F::bind(self.fb, compose(f, self.func)))
}
```

This avoids the intermediate `F::map` call entirely by fusing the maps into the bind's callback.

**Recommendation:** Approach B in addition to approach A. The `map_then_bind` combinator eliminates the unnecessary `F::map` before `F::bind`, and the pattern of "accumulate maps then bind" is common. Document that `apply` and `bind` force materialization.

---

## Issue 6: `Coyoneda` Has Stack Overflow Risk for Deep Chains

**Location:** `coyoneda.rs`, lines 244-249.

**Problem:** `CoyonedaMapLayer::lower` is recursive: it calls `self.inner.lower()` which may itself be a `CoyonedaMapLayer` that calls its inner's `lower()`, and so on. For k chained maps, this creates k stack frames. The test `many_chained_maps` (line 672) only tests 100 layers, but a production use with thousands of maps could overflow the stack.

`CoyonedaExplicit` does not have this problem because maps compose the function rather than nesting layers.

**Approaches:**

### A. Convert the recursive lowering to an iterative loop

This is not straightforward because each layer has a different type (`CoyonedaMapLayer<F, B, A>` where B differs). The trait-object erasure prevents collecting layers into a homogeneous structure.

### B. Use trampolining

Wrap the `lower` return in a `Trampoline` to make it stack-safe. This adds overhead and requires `'static` lifetimes.

### C. Document the limitation and recommend CoyonedaExplicit for deep chains

**Recommendation:** Approach C. The recursive lowering is inherent to the layered trait-object design. Users with deep chains should use `CoyonedaExplicit`. Document a rough depth limit (typically 1000s of frames before overflow, depending on platform).

---

## Issue 7: CoyonedaExplicit Lacks a `Functor` Instance (No Brand)

**Location:** `coyoneda_explicit.rs`, module-level documentation lines 18-19.

**Problem:** `CoyonedaExplicit` has no brand type and no `Functor` implementation. This means it cannot be used in generic code that requires `F: Functor`. The `into_coyoneda` method (line 314) provides an escape hatch, but this forces materialization of the boxed trait object, losing the explicit type information.

This is a deliberate design choice (documented in the comparison table), but it limits composability. For example, you cannot write a function generic over any `Functor` and have it transparently fuse maps when given a `CoyonedaExplicit`.

**Approaches:**

### A. Accept the limitation

The whole point of `CoyonedaExplicit` is to trade HKT integration for performance. This is the stated design.

### B. Create a brand with a fixed function type

A `CoyonedaExplicitBrand<F>` could exist if the function type were fixed (e.g., always `Box<dyn Fn>`). But this reintroduces boxing and defeats the purpose.

### C. Create a CoyonedaExplicit brand using approach A from Issue 1

If the function type is made generic (approach A from Issue 1), a brand is impossible because `Kind::Of<'a, A>` has a fixed number of type parameters and cannot accommodate the function type parameter.

**Recommendation:** Approach A. The lack of a brand is the correct trade-off for this type. Ensure `into_coyoneda` is well-documented as the interoperability bridge.

---

## Issue 8: `Coyoneda`'s `Foldable` Requires `F: Functor`, Defeating the Free Functor Purpose

**Location:** `coyoneda.rs`, line 533.

**Problem:** `Foldable for CoyonedaBrand<F>` requires `F: Functor + Foldable + 'static`. This means you cannot fold a `Coyoneda<F, A>` unless `F` is already a `Functor`. But the primary purpose of `Coyoneda` is to give a `Functor` instance to types that lack one. If `F` already has `Functor`, you can just `map` directly and fold; `Coyoneda` adds no value for the fold path.

`CoyonedaExplicit::fold_map` (line 281) correctly avoids this by composing the fold function with the accumulated mapping function, requiring only `F: Foldable`. This is possible because the intermediate type `B` is visible.

**Approaches:**

### A. Add a fold_map_inner method to CoyonedaInner

Add a method like:

```rust
fn fold_map_boxed<M: Monoid>(
    self: Box<Self>,
    func: Box<dyn Fn(A) -> M>,
) -> M
where
    F: Foldable;
```

The problem is that `M` is generic, making this method not dyn-compatible. However, if we type-erase `M` via `Box<dyn Any>` and `downcast`, we can make it work at the cost of runtime type checking.

### B. Accept the limitation

The module documentation already explains this. Users who need `Foldable` without `Functor` should use `CoyonedaExplicit`.

### C. Use a different existential encoding

Replace trait objects with an enum-based approach or use `Box<dyn Any>` for the stored value, enabling pattern matching. This would be a significant rewrite.

**Recommendation:** Approach B. The limitation is well-documented and `CoyonedaExplicit` provides the workaround. Approaches A and C introduce unsoundness risks or significant complexity.

---

## Issue 9: Thread Safety -- Neither Implementation is `Send`

**Location:** `coyoneda.rs`, line 269 (`Box<dyn CoyonedaInner>`); `coyoneda_explicit.rs`, line 96 (`Box<dyn Fn(B) -> A + 'a>`).

**Problem:** Both types use `Box<dyn Trait>` without `Send` bounds, making them `!Send`. This means neither can be sent across thread boundaries. The library has a pattern of providing `Send` variants (e.g., `SendThunk`, `ArcLazy`), but no such variants exist for either Coyoneda type.

For `Coyoneda`, the `Box<dyn CoyonedaInner>` cannot be `Send` without adding `Send` bounds to the trait and all closures stored in map layers.

For `CoyonedaExplicit`, the `Box<dyn Fn(B) -> A + 'a>` could be made `Box<dyn Fn(B) -> A + Send + 'a>`, but this requires all composed functions to be `Send`.

**Approaches:**

### A. Add Send variants

Create `SendCoyoneda` and `SendCoyonedaExplicit` that require `Send` on all closures and inner values. This follows the library's existing pattern (`Thunk`/`SendThunk`, `RcLazy`/`ArcLazy`).

### B. For CoyonedaExplicit with generic function type (Issue 1, Approach A), Send is automatic

If the function is stored as a generic type parameter rather than `Box<dyn Fn>`, the compiler will automatically derive `Send` when the closure and the stored value are both `Send`. No separate type needed.

### C. Parameterize over Send via a marker trait

Use a trait like `MaybeSend` to make the Send bound optional at the type level. This adds complexity.

**Recommendation:** If Issue 1, Approach A is adopted, approach B gives `Send` for free. Otherwise, approach A (explicit `Send` variants) is consistent with the library's existing patterns.

---

## Issue 10: `Coyoneda::new` Creates an Unnecessary Extra Layer

**Location:** `coyoneda.rs`, lines 306-316.

**Problem:** `Coyoneda::new` creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
Coyoneda(Box::new(CoyonedaMapLayer {
    inner: Box::new(CoyonedaBase { fa: fb }),
    func: Box::new(f),
}))
```

This performs 3 heap allocations (one for the outer `Box<dyn CoyonedaInner>` wrapping the `CoyonedaMapLayer`, one for the inner `Box<dyn CoyonedaInner>` wrapping the `CoyonedaBase`, and one for `Box<dyn Fn>`). Compare with `lift` (line 338) which only performs 1 allocation.

A more efficient encoding would be a single `CoyonedaMapLayer` variant that stores `F B` directly (eliminating the inner `CoyonedaBase` and its box).

**Approaches:**

### A. Add a CoyonedaNewLayer that stores F B and a function directly

```rust
struct CoyonedaNewLayer<'a, F, B: 'a, A: 'a>
where
    F: Kind_cdc7cd43dac7585f + 'a,
{
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
```

This eliminates one `Box` allocation in `new`. The `lower` implementation would call `F::map(self.func, self.fb)` directly.

### B. Accept the overhead

`new` is called once; the extra allocation is a constant cost.

**Recommendation:** Approach A is a clean improvement with no downsides. The `CoyonedaMapLayer` already stores an `inner` and a `func`; a variant that stores `fb` and `func` directly is natural.

---

## Issue 11: Missing Type Class Instances for Coyoneda

**Location:** `coyoneda.rs`, lines 463-581 (only `Functor`, `Pointed`, `Foldable` implemented).

**Problem:** The module documentation (lines 70-73) acknowledges that PureScript provides `Apply`, `Applicative`, `Bind`, `Monad`, `Traversable`, `Extend`, `Comonad`, `Eq`, `Ord`, and others. The current implementation only provides `Functor`, `Pointed`, and `Foldable`. The documentation (lines 64-68) explains that `Semiapplicative` and `Traversable` are blocked by the non-Clone nature of `Box<dyn CoyonedaInner>`.

`CoyonedaExplicit` provides ad-hoc `apply` and `bind` methods (lines 358-402), but these are not type class instances and cannot be used generically.

**Missing instances that could be added:**

1. `Eq` and `Ord` -- by lowering and delegating (requires `F: Functor + Eq`/`Ord`).
2. `Debug`/`Display` -- by lowering and delegating.
3. `Semimonad` (Bind) -- `fn bind(fa, f) = f(fa.lower())` (requires `F: Functor + Semimonad`).
4. `Semiapplicative` (Apply) -- requires `Clone` on `Coyoneda`, which requires `Rc`/`Arc` wrapping.

**Approaches:**

### A. Add instances that only require lowering

`Semimonad`, `Eq`, `Ord`, `Debug` can all be implemented by lowering first. These require `F: Functor` but do not require `Clone`.

### B. Add an Rc-wrapped Coyoneda variant for Semiapplicative/Traversable

Wrap the inner trait object in `Rc<dyn CoyonedaInner>` instead of `Box`. This makes `Coyoneda` cloneable, enabling `Semiapplicative` and `Traversable`. Trade-off: reference counting overhead.

**Recommendation:** Approach A for the easy instances (`Semimonad` especially). Approach B can be deferred or offered as a separate type (e.g., `RcCoyoneda`).

---

## Issue 12: Ergonomic Friction -- Verbose Type Annotations

**Location:** `coyoneda_explicit.rs`, lines 489-493 (test code).

**Problem:** Using `CoyonedaExplicit` requires verbose turbofish annotations:

```rust
CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(42))
CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
```

The three type parameters (`F`, `B`, `A`) must be specified even though two are inferred. `Coyoneda` is slightly better with two parameters:

```rust
Coyoneda::<VecBrand, _>::lift(vec![1, 2, 3])
```

**Approaches:**

### A. Provide convenience type aliases

```rust
pub type VecCoyonedaExplicit<'a, A> = CoyonedaExplicit<'a, VecBrand, A, A>;
pub type OptionCoyonedaExplicit<'a, A> = CoyonedaExplicit<'a, OptionBrand, A, A>;
```

### B. Provide module-level lift functions

```rust
pub fn lift_vec<'a, A: 'a>(v: Vec<A>) -> CoyonedaExplicit<'a, VecBrand, A, A> {
    CoyonedaExplicit::lift(v)
}
```

### C. Accept the verbosity

This is standard for Rust code using HKT patterns. The turbofish syntax is expected.

**Recommendation:** Approach C. Type aliases proliferate and become maintenance burdens. The turbofish syntax is idiomatic Rust and users of an HKT library expect it.

---

## Issue 13: `into_coyoneda` Re-boxes Without Composing

**Location:** `coyoneda_explicit.rs`, lines 314-316.

**Problem:** `into_coyoneda` calls `Coyoneda::new(self.func, self.fb)`, which creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase` (see Issue 10). The boxed `dyn Fn` in `self.func` is moved into a new `Box<dyn Fn>` inside `Coyoneda::new`. Since `self.func` is already a `Box<dyn Fn(B) -> A>`, this should be a zero-cost move of the box pointer, not a new allocation. However, `Coyoneda::new` takes `impl Fn(B) -> A + 'a`, which means the `Box<dyn Fn>` is wrapped in _another_ `Box<dyn Fn>` (the `Box` itself implements `Fn` via deref, but the new layer re-boxes it). This results in double indirection.

**Approaches:**

### A. Add a Coyoneda constructor that accepts Box<dyn Fn> directly

```rust
pub fn from_boxed(func: Box<dyn Fn(B) -> A + 'a>, fb: ...) -> Self
```

This avoids the double-boxing.

### B. Accept the overhead

The conversion is a one-time cost and the double indirection is unlikely to be measurable.

**Recommendation:** Approach A is a clean improvement. A `pub(crate)` constructor on `Coyoneda` that accepts a pre-boxed function would eliminate the double indirection.

---

## Issue 14: No Benchmarks to Validate Fusion Claims

**Problem:** The module documentation makes performance claims about zero-cost fusion, single-pass lowering, and comparative performance against `Coyoneda`. There are no benchmarks in the repository that measure:

1. CoyonedaExplicit map fusion vs. direct chained `F::map` calls.
2. CoyonedaExplicit vs. Coyoneda for k chained maps.
3. Heap allocation counts for each approach.
4. The overhead of the `Box<dyn Fn>` in CoyonedaExplicit's current implementation.

Without benchmarks, the performance claims are unsubstantiated.

**Approaches:**

### A. Add Criterion benchmarks

Create benchmarks comparing:

- `CoyonedaExplicit` with k maps then lower vs. k direct `Vec::map` calls.
- `Coyoneda` with k maps then lower vs. the same.
- Manual `compose` then single `map` vs. `CoyonedaExplicit`.

### B. Add allocation-counting tests

Use a global allocator wrapper to count allocations in tests.

**Recommendation:** Approach A at minimum. The benchmarks should live in `benches/` alongside existing Criterion benchmarks. They will either validate the claims or reveal that Issue 1 matters in practice.

---

## Summary Table

| #   | Issue                                           | Severity            | Recommendation                                                    |
| --- | ----------------------------------------------- | ------------------- | ----------------------------------------------------------------- |
| 1   | CoyonedaExplicit `map` allocates a Box per call | High                | Make function type generic, or fix documentation                  |
| 2   | Coyoneda does not fuse maps                     | Low (documented)    | Accept; guide users to CoyonedaExplicit                           |
| 3   | Fn used where FnOnce would suffice              | Medium              | Constrained by Functor::map; no change                            |
| 4   | fold_map requires B: Clone                      | Low                 | Document; inherent to Foldable design                             |
| 5   | apply/bind destroy fusion pipeline              | Medium              | Add map_then_bind combinator                                      |
| 6   | Coyoneda stack overflow for deep chains         | Medium              | Document depth limit                                              |
| 7   | CoyonedaExplicit lacks Functor instance         | Low (by design)     | Accept; document into_coyoneda bridge                             |
| 8   | Coyoneda Foldable requires F: Functor           | Medium (documented) | Accept; use CoyonedaExplicit for Functor-free folds               |
| 9   | Neither implementation is Send                  | Medium              | Generic function type gives Send for free; else add Send variants |
| 10  | Coyoneda::new creates extra layer               | Low                 | Add direct CoyonedaNewLayer                                       |
| 11  | Missing type class instances                    | Medium              | Add Semimonad and simple delegation instances                     |
| 12  | Verbose type annotations                        | Low                 | Accept; standard for HKT in Rust                                  |
| 13  | into_coyoneda double-boxes                      | Low                 | Add Box-accepting constructor                                     |
| 14  | No benchmarks for fusion claims                 | High                | Add Criterion benchmarks                                          |
