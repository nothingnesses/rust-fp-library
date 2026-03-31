# Coyoneda Implementations: Analysis of Flaws, Issues, and Limitations

This document analyzes the two Coyoneda implementations in the fp-library codebase:

- `Coyoneda` in `/home/nixos/projects/rust-fp-lib/fp-library/src/types/coyoneda.rs`
- `CoyonedaExplicit` in `/home/nixos/projects/rust-fp-lib/fp-library/src/types/coyoneda_explicit.rs`

Each issue includes the problem description, affected code locations, proposed approaches, trade-offs, and a recommendation.

---

## Issue 1: CoyonedaExplicit Does Not Achieve Zero-Cost Map Fusion

### Problem

The module documentation claims "zero-cost map fusion" (line 1, line 9-13 of `coyoneda_explicit.rs`), but this is misleading. Each call to `map` allocates a new `Box<dyn Fn>` on the heap:

```rust
// coyoneda_explicit.rs, line 167-175
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C> {
    CoyonedaExplicit {
        fb: self.fb,
        func: Box::new(compose(f, self.func)),
    }
}
```

The `compose` function (at `functions.rs:88-93`) captures `self.func` (a `Box<dyn Fn(B) -> A>`) and `f` (an `impl Fn(A) -> C`) into a new closure, which is then boxed. So each `map` call:

1. Allocates a new `Box` for the composed closure.
2. Moves the previous `Box<dyn Fn>` into the capture of the new closure.
3. Builds a nested tower of closures, each containing its predecessor.

Similarly, `lift` (line 434-439) also allocates a `Box` for the identity function. The comment on `map` at line 143-145 ("No heap allocation occurs for the composition itself") is technically misleading; while `compose` itself does not allocate, the immediately following `Box::new(...)` does.

The comparison table at line 17-24 claims "0" heap allocations per map for `CoyonedaExplicit` vs. "2 boxes" for `Coyoneda`. In reality, `CoyonedaExplicit::map` performs 1 box allocation per map (for the composed closure), while `Coyoneda::map` performs 2 (one for the `CoyonedaMapLayer` itself via `Box<dyn CoyonedaInner>`, and one for `Box<dyn Fn>`).

### Impact

- Documentation overstates the performance benefit, potentially misleading users.
- The actual benefit is genuine but smaller than claimed: 1 allocation per map instead of 2, and 1 call to `F::map` at `lower` time instead of k calls. The latter is the real win for eager types like `Vec`.

### Approaches

**A. Fix the documentation to accurately describe the cost model.**
Remove "zero-cost" language. State that each `map` performs one heap allocation for the composed closure, but `lower` makes exactly one call to `F::map` regardless of map count. The win is single-pass traversal, not zero allocation.

**B. Eliminate boxing entirely by using a generic function type parameter.**
Replace `Box<dyn Fn(B) -> A>` with a generic type parameter `G: Fn(B) -> A` on the struct. Each `map` would return a `CoyonedaExplicit` whose function type is `Compose<F, G>` (a concrete composition wrapper). This achieves truly zero-cost fusion: no boxing, no dynamic dispatch, fully inlined.

The downside is that the type grows with each `map` call (`CoyonedaExplicit<F, B, Compose<Compose<..., G2>, G1>>`), making the type unnameable after multiple maps. This prevents storing the value in a struct field or passing it to non-generic functions. It also significantly complicates `new`, which would need to take a generic function parameter.

**C. Keep the current approach but use `FnOnce` instead of `Fn`.**
Since `CoyonedaExplicit` is consumed by `lower` (takes `self`), the accumulated function only needs to be called once per element in the functor. For `Option`, this means at most once; for `Vec`, once per element. But `F::map` takes `impl Fn(A) -> B` (not `FnOnce`), so this would require changing the `Functor` trait or providing a separate `map_once` method. This is a larger change with ecosystem-wide implications.

### Recommendation

**Approach A** as an immediate fix, combined with **Approach B** as a future enhancement. The documentation should be corrected now. The generic function parameter approach is the ideal long-term solution for users who need true zero-cost fusion, but it can coexist with the current boxed version (which is more ergonomic for named types). The two variants serve different use cases.

---

## Issue 2: Coyoneda Does Not Perform Map Fusion At All

### Problem

The `Coyoneda` type (in `coyoneda.rs`) uses a layered trait-object encoding where each `map` call wraps the previous value in a new `CoyonedaMapLayer`:

```rust
// coyoneda.rs, line 394-402
pub fn map<B: 'a>(
    self,
    f: impl Fn(A) -> B + 'a,
) -> Coyoneda<'a, F, B> {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: self.0,
        func: Box::new(f),
    }))
}
```

At `lower` time (line 244-249), each layer calls `F::map` independently:

```rust
fn lower(self: Box<Self>) -> ... {
    let lowered = self.inner.lower();
    F::map(self.func, lowered)
}
```

For k chained maps on a `Vec` of n elements, this results in k separate traversals of the vector, each producing an intermediate `Vec`. This is strictly worse than just calling `F::map` k times directly, because it adds 2 box allocations per map on top of the same traversal cost.

The module documentation (lines 12, 38-44) correctly describes this limitation. However, the mere existence of `Coyoneda` as a "free functor" without fusion is questionable; the theoretical purpose of Coyoneda in FP is map fusion, and without it, the type adds overhead with no compensating benefit beyond providing a `Functor` instance for non-functor types.

### Impact

- For types that already implement `Functor`, wrapping in `Coyoneda` is strictly worse than direct `map` calls.
- The only use case where `Coyoneda` provides value is giving a `Functor` instance to a type that lacks one; but for that use case the performance characteristics are not just "no fusion," they are actively harmful for eager containers.

### Approaches

**A. Accept the status quo and ensure documentation clearly steers users toward `CoyonedaExplicit` for performance.**
The limitation is fundamental to Rust's trait object system. The documentation already explains this well.

**B. Add an `optimize` or `fuse` method that converts `Coyoneda` to `CoyonedaExplicit` before lowering.**
This is not possible in general because `Coyoneda` hides the intermediate type `B` behind the existential; you cannot recover it to construct a `CoyonedaExplicit<F, B, A>`.

**C. Provide a `from_coyoneda_explicit` constructor on `Coyoneda` that preserves the fused function.**
`CoyonedaExplicit::into_coyoneda` already exists (line 314-316). This converts the fused pipeline into a single-layer `Coyoneda` with one `Box<dyn Fn>`, preserving the fusion benefit when crossing into HKT-generic code.

### Recommendation

**Approach A** combined with better guidance. The `into_coyoneda` bridge already exists. The documentation should more prominently recommend the workflow: build fusion pipelines with `CoyonedaExplicit`, then convert to `Coyoneda` via `into_coyoneda` when HKT polymorphism is needed.

---

## Issue 3: Stack Overflow Risk in CoyonedaExplicit Due to Nested Closures

### Problem

The module documentation comparison table (line 22) claims `CoyonedaExplicit` has "No" stack overflow risk. However, each `map` composes a new closure around the previous one via `compose`:

```rust
// functions.rs, line 88-93
pub fn compose<A, B, C>(
    f: impl Fn(B) -> C,
    g: impl Fn(A) -> B,
) -> impl Fn(A) -> C {
    move |a| f(g(a))
}
```

After k maps, calling the composed function creates a call stack of depth k (the outermost closure calls the next inner one, and so on). The `many_chained_maps` test (line 558-564 of `coyoneda_explicit.rs`) only tests 100 maps. For a sufficiently large k (typically around 10,000-100,000 depending on stack size and closure capture sizes), calling the composed function will overflow the stack.

This is exactly the same risk as `Coyoneda`, just manifested differently: `Coyoneda` overflows through nested `lower` calls; `CoyonedaExplicit` overflows through nested composed-function calls.

### Impact

- The documentation falsely claims no stack overflow risk.
- Users who chain thousands of maps (e.g., in generated code or recursive pipelines) will hit stack overflows.

### Approaches

**A. Fix the documentation to state "Partial" rather than "No" for stack overflow risk.**
The risk is real but typically requires many more maps than `Coyoneda` (because `CoyonedaExplicit` does not have the additional `lower` recursion overhead).

**B. Use a trampoline-style function composition that avoids deep call stacks.**
This would require representing the composed function as a data structure (e.g., a list of boxed functions) rather than nested closures. At call time, the list is iterated rather than recursed. This eliminates the stack overflow risk at the cost of dynamic dispatch per step (since each function in the list has a different type and must be boxed).

**C. Accept the limitation and document it.**
In practice, chains of thousands of maps are uncommon in handwritten code. The documentation should state that stack overflow is possible with very deep chains.

### Recommendation

**Approach A** immediately (fix the documentation). **Approach B** could be offered as an optional variant (`CoyonedaStacked` or similar) for users who need extremely deep chains, but it contradicts the zero-allocation design goal of `CoyonedaExplicit`.

---

## Issue 4: `Fn` Constraint Instead of `FnOnce` Prevents Move Semantics in Mapping Functions

### Problem

Both `CoyonedaExplicit::map` and `Coyoneda::map` require `impl Fn(A) -> B` rather than `impl FnOnce(A) -> B`:

```rust
// coyoneda_explicit.rs, line 167-169
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
```

```rust
// coyoneda.rs, line 394-396
pub fn map<B: 'a>(
    self,
    f: impl Fn(A) -> B + 'a,
```

This flows from the `Functor::map` signature (in `functor.rs`, line 121-124) which also requires `impl Fn`. The consequence is that mapping functions cannot move captured values out of their environment. Users who want to map with a closure that consumes a captured value must clone it or wrap it in `Rc`.

For example, this does not compile:

```rust
let expensive = compute_something();
CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
    .map(move |x| use_and_consume(x, expensive))  // Error: Fn, not FnOnce
```

### Impact

- Ergonomic friction for users who want to move values into mapping functions.
- Forces unnecessary cloning or reference counting.

### Approaches

**A. Accept this as a consequence of the Functor trait design.**
`Fn` is required because `map` may need to call the function multiple times (e.g., for each element of a `Vec`). A `FnOnce`-based map would only be sound for single-element functors. This is a correct design choice.

**B. Provide a separate `map_once` method for single-element functors.**
A `map_once` that takes `FnOnce` could be provided for `Option`, `Result`, etc. However, this cannot be encoded generically in the current type class system without a new trait.

### Recommendation

**Approach A.** The `Fn` constraint is correct for general functors. Document this in the `map` method documentation so users understand why `Fn` (not `FnOnce`) is required. The existing design is sound.

---

## Issue 5: `CoyonedaExplicit` Lacks HKT Integration

### Problem

`CoyonedaExplicit` has no brand type and no `Functor` implementation (as noted in the comparison table at line 17-24 of `coyoneda_explicit.rs`). This means it cannot be used in code that is generic over `Functor` or any other type class.

The `map` method is an inherent method rather than a `Functor::map` implementation. This means generic functions like `void`, `flap`, or `map_with_index` cannot operate on `CoyonedaExplicit` values.

### Impact

- Users must choose between fusion (`CoyonedaExplicit`) and HKT integration (`Coyoneda`) upfront.
- The `into_coyoneda` bridge (line 314-316) provides a path from `CoyonedaExplicit` to `Coyoneda`, but not the reverse.

### Approaches

**A. Accept the limitation and rely on `into_coyoneda` for HKT interop.**
This is the current design. It works but requires users to understand both types and when to convert.

**B. Create a brand for `CoyonedaExplicit` parameterized by the intermediate type `B`.**
Something like `CoyonedaExplicitBrand<F, B>` where `Of<'a, A> = CoyonedaExplicit<'a, F, B, A>`. The `Functor` implementation's `map` would call the inherent `map`. This is technically possible but the brand leaks the intermediate type `B`, which changes with each `map` call. The brand would only be valid for a specific point in the pipeline, severely limiting its usefulness.

**C. Use a two-phase API: build with `CoyonedaExplicit`, then convert to `Coyoneda` for consumption.**
This is essentially what `into_coyoneda` already provides. Documenting this as the canonical workflow would help.

### Recommendation

**Approach A/C.** The limitation is fundamental to the tension between type-level function composition (which requires the intermediate type to be visible) and existential quantification (which hides it). Document the two-phase workflow prominently.

---

## Issue 6: `fold_map` on CoyonedaExplicit Requires `B: Clone` Unnecessarily

### Problem

The `fold_map` method on `CoyonedaExplicit` (line 281-291) has a `B: Clone` bound:

```rust
pub fn fold_map<FnBrand, M>(
    self,
    func: impl Fn(A) -> M + 'a,
) -> M
where
    B: Clone,
    M: Monoid + 'a,
    F: Foldable,
    FnBrand: CloneableFn + 'a, {
    F::fold_map::<FnBrand, B, M>(compose(func, self.func), self.fb)
}
```

The `B: Clone` bound propagates from the `Foldable::fold_map` signature, which requires `A: Clone`:

```rust
// foldable.rs, line 572-578
fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
    func: impl Fn(A) -> M + 'a,
    fa: ...,
) -> M
```

In PureScript, `Foldable` for `Coyoneda` composes the fold function with the accumulated mapping function and folds the original `F B`, requiring only `Foldable f` (not `Functor f`). The `CoyonedaExplicit` implementation correctly achieves this (it calls `F::fold_map` on `self.fb` with the composed function). However, the `B: Clone` bound is an artifact of the Rust `Foldable` trait design, not a fundamental requirement.

### Impact

- Users cannot fold over `CoyonedaExplicit` values when `B` is not `Clone`, even though the fold function is `compose(func, self.func)` which maps `B -> A -> M` and never needs to clone `B` values.
- This is a false constraint: the composed function consumes `B` by value and produces `M`; cloning is not inherently needed.

### Approaches

**A. Accept as a limitation of the Foldable trait signature.**
The `Clone` bound exists because `Foldable::fold_right` and `fold_left` default implementations may need to clone elements. Removing it would require redesigning `Foldable`.

**B. Provide a specialized `fold_map` that bypasses the type class and folds directly.**
Write a `fold_map_no_clone` method that manually implements the fold for specific known brands (e.g., `Vec`, `Option`). This is not generic but avoids the spurious bound.

**C. Redesign `Foldable` to not require `Clone` on `fold_map`.**
This is a larger effort that would affect the entire codebase.

### Recommendation

**Approach A** for now, with a note in the documentation. **Approach C** as a long-term goal if the `Clone` bound proves to be a widespread pain point.

---

## Issue 7: `apply` and `bind` on CoyonedaExplicit Destroy Fusion

### Problem

Both `apply` (line 358-366) and `bind` (line 395-402) call `lower()` on their arguments, destroying any accumulated fusion:

```rust
// apply
pub fn apply<FnBrand: CloneableFn + 'a, Bf: 'a, C: 'a>(
    ff: CoyonedaExplicit<'a, F, Bf, ...>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C>
where
    A: Clone,
    F: Semiapplicative, {
    CoyonedaExplicit::lift(F::apply::<FnBrand, A, C>(ff.lower(), fa.lower()))
}
```

```rust
// bind
pub fn bind<C: 'a>(
    self,
    f: impl Fn(A) -> CoyonedaExplicit<'a, F, C, C> + 'a,
) -> CoyonedaExplicit<'a, F, C, C>
where
    F: Functor + Semimonad, {
    CoyonedaExplicit::lift(F::bind(self.lower(), move |a| f(a).lower()))
}
```

After `apply` or `bind`, the returned `CoyonedaExplicit` has `B = C` with the identity function. Any prior fusion is consumed. Furthermore, the `bind` closure calls `f(a).lower()` for each element, meaning that if `f` returns a `CoyonedaExplicit` with accumulated maps, those maps are immediately applied. There is no way to fuse across `bind` boundaries.

### Impact

- Users who intersperse `apply` or `bind` with `map` chains lose the fusion benefit.
- The `apply` method requires `A: Clone`, which is an additional constraint not present in the regular `Semiapplicative` interface. This comes from `F::apply` needing to clone values.
- The `bind` method requires `F: Functor + Semimonad`, meaning `CoyonedaExplicit` loses the "functor for free" property when bind is used.

### Approaches

**A. Accept and document.**
Fusion across `apply` and `bind` boundaries is not possible in general; these operations introduce data dependencies that prevent function composition. This is inherent to the operations, not a flaw.

**B. Warn users in documentation that apply/bind are "fusion barriers."**
Explicitly document that users should complete their `map` chains before calling `apply` or `bind`, and that another `map` chain can begin after.

### Recommendation

**Approach B.** The limitation is fundamental, but users should be warned. Add documentation to `apply` and `bind` stating that they act as fusion barriers: they lower the accumulated pipeline, perform the operation, and return a fresh `CoyonedaExplicit` with no pending maps.

---

## Issue 8: Coyoneda's `Foldable` Requires `F: Functor`, Diverging from PureScript

### Problem

The `Foldable` implementation for `CoyonedaBrand<F>` (line 533 of `coyoneda.rs`) requires `F: Functor + Foldable + 'static`:

```rust
impl<F: Functor + Foldable + 'static> Foldable for CoyonedaBrand<F> {
    fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
        func: impl Fn(A) -> M + 'a,
        fa: ...,
    ) -> M
    where ... {
        F::fold_map::<FnBrand, A, M>(func, fa.lower())
    }
}
```

It calls `fa.lower()` first (which requires `F: Functor`), then folds. PureScript's implementation only requires `Foldable f` because it can open the existential to compose the fold function with the accumulated mapping, folding the original `F B` directly.

### Impact

- Types that are `Foldable` but not `Functor` cannot be folded through `Coyoneda`. This eliminates one of the key use cases of Coyoneda: providing operations for types with fewer instances.
- `CoyonedaExplicit::fold_map` correctly avoids this issue by having access to the intermediate type `B`.

### Approaches

**A. Accept as a fundamental Rust limitation.**
The `CoyonedaInner` trait cannot have a `fold_map_inner` method because it would need to be generic over the monoid type `M`, which breaks dyn-compatibility.

**B. Add a `fold_map_inner` method with a fixed monoid type.**
This does not work in general since the monoid type varies per call site.

**C. Direct users to `CoyonedaExplicit` for folding non-functor types.**
`CoyonedaExplicit::fold_map` only requires `F: Foldable`, matching PureScript's semantics.

### Recommendation

**Approach C.** Document that `CoyonedaExplicit` is the correct choice when you need `Foldable` without `Functor`. The limitation in `Coyoneda` is fundamental to Rust's trait object system.

---

## Issue 9: Coyoneda's `hoist` Requires `F: Functor`, Diverging from PureScript

### Problem

`Coyoneda::hoist` (line 443-450) lowers first, then transforms:

```rust
pub fn hoist<G: Kind_cdc7cd43dac7585f + 'a>(
    self,
    nat: impl NaturalTransformation<F, G>,
) -> Coyoneda<'a, G, A>
where
    F: Functor, {
    Coyoneda::lift(nat.transform(self.lower()))
}
```

PureScript's `hoistCoyoneda` applies the natural transformation directly to the hidden `F B`, requiring no `Functor` constraint on `F`. The Rust implementation cannot do this because adding a `hoist_inner<G>` method to `CoyonedaInner` would be generic over the brand `G`, breaking dyn-compatibility.

`CoyonedaExplicit::hoist` (line 241-249) does not have this issue because the intermediate type `B` is visible.

### Impact

- Cannot hoist through natural transformations when `F` is not a `Functor`.
- This reduces the utility of `Coyoneda` for non-functor types.

### Approaches

**A. Accept the limitation; direct users to `CoyonedaExplicit::hoist` when `F` is not a `Functor`.**

**B. Provide a specialized `hoist` on `CoyonedaBase` that avoids the `Functor` bound.**
Since `CoyonedaBase` stores `F A` directly (no accumulated maps), it could apply the natural transformation without lowering. But this information is erased behind the trait object; you cannot know at runtime whether the `Coyoneda` is a `CoyonedaBase` or a `CoyonedaMapLayer`.

### Recommendation

**Approach A.** Document the limitation and point users to `CoyonedaExplicit::hoist`.

---

## Issue 10: Missing Type Class Instances

### Problem

The module documentation for `Coyoneda` (line 70-73) notes several missing instances that PureScript provides. The current state:

| Instance                | Coyoneda                  | CoyonedaExplicit                      | PureScript                   |
| ----------------------- | ------------------------- | ------------------------------------- | ---------------------------- |
| Functor                 | Yes (via brand)           | No (inherent method only)             | Yes                          |
| Pointed                 | Yes                       | Yes (inherent method)                 | Yes (Applicative)            |
| Foldable                | Yes (requires F: Functor) | Yes (inherent method, no F: Functor)  | Yes                          |
| Semiapplicative (Apply) | No                        | Yes (inherent method, fusion barrier) | Yes                          |
| Semimonad (Bind)        | No                        | Yes (inherent method, fusion barrier) | Yes                          |
| Traversable             | No                        | No                                    | Yes                          |
| Extend                  | No                        | No                                    | Yes                          |
| Comonad                 | No                        | No                                    | Yes                          |
| Eq/Ord                  | No                        | No                                    | Yes                          |
| Clone                   | No                        | No                                    | N/A (implicit in PureScript) |

Key missing instances for `Coyoneda`:

1. **Semiapplicative/Semimonad**: Could be implemented by lowering and delegating, similar to how `Foldable` is implemented. Would require `F: Functor + Semiapplicative` and `F: Functor + Semimonad` respectively.

2. **Traversable**: Blocked by the `Clone` requirement. `Coyoneda` wraps `Box<dyn CoyonedaInner>` which is not `Clone`. Without `Clone`, `Traversable` cannot be implemented.

3. **Eq/Ord**: Would require lowering and comparing the results. Only possible when `F: Functor` and the lowered type implements `Eq`/`Ord`.

### Impact

- Users cannot use `Coyoneda` in contexts that require `Semiapplicative`, `Semimonad`, `Traversable`, etc.
- The `CoyonedaExplicit` type partially fills this gap with inherent methods, but these are not interoperable with generic type class code.

### Approaches

**A. Implement Semiapplicative and Semimonad for CoyonedaBrand by lowering and delegating.**
This is straightforward and follows the same pattern as the existing `Foldable` implementation.

**B. Implement Eq and Ord by lowering and comparing.**
Requires `F: Functor` and the lowered type to implement `Eq`/`Ord`.

**C. For Traversable, consider an Rc-wrapped variant of Coyoneda.**
An `RcCoyoneda` that wraps `Rc<dyn CoyonedaInner>` instead of `Box<dyn CoyonedaInner>` would be `Clone`, enabling `Traversable`. This adds reference counting overhead.

### Recommendation

**Approach A** for `Semiapplicative` and `Semimonad` as these are low-hanging fruit. **Approach B** for `Eq`/`Ord` is reasonable. **Approach C** for `Traversable` should be considered carefully; an `Rc`-wrapped variant is a significant addition.

---

## Issue 11: Thread Safety

### Problem

Neither `Coyoneda` nor `CoyonedaExplicit` is `Send` or `Sync`:

- `Coyoneda` contains `Box<dyn CoyonedaInner>` which uses trait objects that are not `Send` (line 269 of `coyoneda.rs`).
- `CoyonedaExplicit` contains `Box<dyn Fn(B) -> A + 'a>` (line 96-97 of `coyoneda_explicit.rs`) which is not `Send`.

The documentation for `CoyonedaExplicit` comparison table (line 17-24) does not mention thread safety at all.

### Impact

- Neither type can be sent across thread boundaries or used in concurrent contexts.
- This limits their use in async code and parallel computation pipelines.
- The library already has a pattern for thread-safe variants (`SendThunk`, `ArcLazy`, `ArcFnBrand`) but no equivalent exists for Coyoneda.

### Approaches

**A. Create `SendCoyonedaExplicit` that uses `Box<dyn Fn(B) -> A + Send + 'a>`.**
Following the existing `Thunk`/`SendThunk` pattern. Requires that all composed functions are `Send`.

**B. Parameterize `CoyonedaExplicit` over the function wrapper type.**
Use a type parameter `G: Fn(B) -> A` or a pointer brand to choose between `Rc<dyn Fn>`, `Arc<dyn Fn>`, or `Box<dyn Fn>`. This avoids code duplication.

**C. Accept the limitation for now; add Send variants later.**
Thread safety for Coyoneda is a niche requirement.

### Recommendation

**Approach A** as an incremental step, following the library's existing pattern. **Approach B** is more elegant but requires more design work.

---

## Issue 12: `CoyonedaBrand` Requires `F: 'static`

### Problem

The `impl_kind` for `CoyonedaBrand` (line 455-459 of `coyoneda.rs`) requires `F: 'static`:

```rust
impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f + 'static> for CoyonedaBrand<F> {
        type Of<'a, A: 'a>: 'a = Coyoneda<'a, F, A>;
    }
}
```

This means `CoyonedaBrand<F>` cannot be used when `F` has a non-static lifetime. In practice, most brands are zero-sized marker types that are trivially `'static`, so this is not usually a problem. But it is a fundamental constraint.

### Impact

- Brands that borrow data (hypothetically) cannot be used with `Coyoneda`.
- The `'static` bound propagates to all type class implementations (`Functor`, `Pointed`, `Foldable`).

### Approaches

**A. Accept as a known limitation of the Kind trait system.**
The `'static` bound is documented (in the `CoyonedaBrand` doc comment in `brands.rs`, line 131). All current brands in the codebase are `'static`.

### Recommendation

**Approach A.** This is a non-issue in practice.

---

## Issue 13: `Coyoneda::new` Creates an Unnecessary Extra Layer

### Problem

`Coyoneda::new` (line 306-316) creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
pub fn new<B: 'a>(
    f: impl Fn(B) -> A + 'a,
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
) -> Self {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: Box::new(CoyonedaBase {
            fa: fb,
        }),
        func: Box::new(f),
    }))
}
```

This allocates two boxes: one for `CoyonedaBase` and one for `CoyonedaMapLayer`. If `f` is the identity, the extra layer is pure overhead. Even when `f` is not the identity, a single struct holding both `fb` and `f` would suffice; the base/map-layer split is only needed when the function type differs from what the outer `Coyoneda` type expects (i.e., when `B != A`).

### Impact

- Minor: one extra heap allocation compared to a more optimized encoding.
- The `lift` method (line 338-342) correctly avoids this by creating only a `CoyonedaBase`.

### Approaches

**A. Create a `CoyonedaNewLayer` struct that holds both `fb` and `f` in a single allocation.**
This would implement `CoyonedaInner<'a, F, A>` directly, reducing `new` from 2 box allocations to 1.

**B. Keep the current design for simplicity.**
The extra allocation in `new` is a one-time cost and unlikely to matter in practice.

### Recommendation

**Approach A** for consistency and to reduce allocation overhead. It is a small change with no downside.

---

## Issue 14: `CoyonedaExplicit::lift` Boxes the Identity Function Unnecessarily

### Problem

`CoyonedaExplicit::lift` (line 434-439) creates a `Box<dyn Fn(A) -> A>` for the identity function:

```rust
pub fn lift(fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>) -> Self {
    CoyonedaExplicit {
        fb: fa,
        func: Box::new(identity),
    }
}
```

This heap-allocates a boxed closure for the identity function. When `lower` is called without any intervening `map` calls, this identity function is passed to `F::map`, causing an unnecessary traversal of the container (e.g., mapping identity over every element of a `Vec`).

### Impact

- `lift` followed immediately by `lower` performs an unnecessary `F::map(identity, fa)` instead of returning `fa` directly.
- The allocation of the identity closure is wasted.

### Approaches

**A. Use an enum to distinguish "no function accumulated" from "function accumulated."**
Replace `Box<dyn Fn(B) -> A>` with:

```rust
enum Accumulated<'a, B, A> {
    Identity, // B == A, no function needed
    Mapped(Box<dyn Fn(B) -> A + 'a>),
}
```

At `lower` time, check the enum: if `Identity`, return `fb` directly; if `Mapped`, call `F::map`. This requires `B == A` to be expressible, which it is (the `lift` method already constrains `B = A`).

The downside is that `map` must check and handle both variants, adding a branch.

**B. Accept the overhead; identity-mapped `F::map` is cheap for most types.**
For `Option`, `F::map(identity, x)` is essentially a no-op. For `Vec`, it creates a new `Vec` with identity-mapped elements, which is wasteful but only happens when `lift` is called without any `map`.

**C. Use a generic function type parameter instead of `Box<dyn Fn>`.**
With approach B from Issue 1, `lift` would return a `CoyonedaExplicit<F, A, A, fn(A) -> A>` (or `Identity`), which the compiler can inline and optimize away entirely.

### Recommendation

**Approach B** for now (the common case is `lift` followed by at least one `map`). **Approach C** is the best long-term solution as part of the generic function type parameter redesign.

---

## Issue 15: `into_coyoneda` Loses Fusion Benefit

### Problem

`CoyonedaExplicit::into_coyoneda` (line 314-316) converts to `Coyoneda`:

```rust
pub fn into_coyoneda(self) -> crate::types::Coyoneda<'a, F, A> {
    crate::types::Coyoneda::new(self.func, self.fb)
}
```

This calls `Coyoneda::new`, which creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase` (see Issue 13). The fused function from `CoyonedaExplicit` is preserved in the single `CoyonedaMapLayer`. However, any subsequent `map` calls on the resulting `Coyoneda` will add new layers, each calling `F::map` independently at `lower` time.

This means the fusion benefit is preserved only if no further maps are applied after conversion. If the user does `explicit.into_coyoneda().map(g)`, the `lower` call will make 2 calls to `F::map`: one for the fused function from `CoyonedaExplicit`, and one for `g`.

### Impact

- The conversion point is a fusion barrier, but this is not documented.

### Approaches

**A. Document the limitation.**
State that `into_coyoneda` should be called after all `map` operations are complete.

### Recommendation

**Approach A.** Add a note to the `into_coyoneda` documentation.

---

## Issue 16: `compose` Creates Deeply Nested Closures That Inhibit Inlining

### Problem

The `compose` function (at `functions.rs:88-93`) returns an opaque `impl Fn` type. When used in `CoyonedaExplicit::map`, each composition wraps the previous `Box<dyn Fn>`:

```rust
func: Box::new(compose(f, self.func)),
```

The `compose` call captures `self.func` (a `Box<dyn Fn(B) -> A>`) and `f` (an `impl Fn(A) -> C>`). The resulting closure calls `self.func` through dynamic dispatch (the `Box<dyn Fn>` vtable), then calls `f` with static dispatch (since `f` is monomorphized).

For k maps, the final composed function has k-1 layers of dynamic dispatch. Each call goes: static call -> dyn dispatch -> static call -> dyn dispatch -> ... -> static call to identity. The compiler cannot inline across the `dyn Fn` boundary, so each intermediate function call pays the cost of an indirect call.

### Impact

- The composed function is not truly "zero-cost." Each intermediate step involves an indirect function call through a vtable.
- For tight loops (e.g., mapping over millions of elements), this can be significant compared to a hand-composed function.

### Approaches

**A. Accept as inherent to the boxed-function design.**
The dynamic dispatch cost is the price of type erasure. It is still better than k separate container traversals.

**B. Use the generic function type parameter approach (Issue 1, Approach B).**
With fully generic function types, the compiler can inline the entire composed function, achieving true zero-cost.

### Recommendation

**Approach A** for the current implementation, with documentation about the per-element cost. **Approach B** as a future enhancement for users who need absolute zero-cost.

---

## Summary of Recommendations

| Issue                                      | Severity | Immediate Action                                 | Long-Term                               |
| ------------------------------------------ | -------- | ------------------------------------------------ | --------------------------------------- |
| 1. Misleading "zero-cost" claim            | High     | Fix documentation                                | Generic function type parameter variant |
| 2. Coyoneda lacks fusion                   | Medium   | Better guidance toward CoyonedaExplicit          | N/A (fundamental)                       |
| 3. Stack overflow risk in CoyonedaExplicit | Medium   | Fix documentation table                          | Optional trampoline variant             |
| 4. Fn vs FnOnce                            | Low      | Document the rationale                           | N/A (correct design)                    |
| 5. No HKT for CoyonedaExplicit             | Medium   | Document two-phase workflow                      | N/A (fundamental)                       |
| 6. Spurious B: Clone on fold_map           | Low      | Document limitation                              | Redesign Foldable                       |
| 7. apply/bind destroy fusion               | Medium   | Document as fusion barriers                      | N/A (fundamental)                       |
| 8. Foldable requires F: Functor            | Medium   | Point to CoyonedaExplicit                        | N/A (fundamental)                       |
| 9. hoist requires F: Functor               | Medium   | Point to CoyonedaExplicit                        | N/A (fundamental)                       |
| 10. Missing type class instances           | Medium   | Implement Semiapplicative/Semimonad for Coyoneda | Rc variant for Traversable              |
| 11. No thread safety                       | Low      | Create SendCoyonedaExplicit                      | Parameterize over pointer brand         |
| 12. F: 'static requirement                 | Low      | Accept                                           | N/A                                     |
| 13. Extra allocation in Coyoneda::new      | Low      | Add CoyonedaNewLayer                             | N/A                                     |
| 14. Boxing identity in lift                | Low      | Accept                                           | Generic function type parameter         |
| 15. into_coyoneda fusion barrier           | Low      | Document limitation                              | N/A                                     |
| 16. Indirect dispatch in composed closures | Medium   | Document cost model                              | Generic function type parameter         |
