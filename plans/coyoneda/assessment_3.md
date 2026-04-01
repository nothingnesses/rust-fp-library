# Coyoneda Implementation Assessment

## 1. Overview

This assessment covers four Coyoneda implementations in the `fp-library` crate:

- **`Coyoneda`** (`fp-library/src/types/coyoneda.rs`): The primary free functor using layered `Box<dyn CoyonedaInner>` trait objects to hide the existential type. Supports full HKT integration (brand, `Functor`, `Pointed`, `Foldable`, `Semiapplicative`, `Semimonad`). Not `Clone`, not `Send`.
- **`CoyonedaExplicit`** (`fp-library/src/types/coyoneda_explicit.rs`): A variant that exposes the intermediate type `B` as a parameter, enabling zero-cost map fusion via compile-time function composition. No full HKT integration for the unboxed form; `BoxedCoyonedaExplicit` has a brand with `Functor` and `Foldable`.
- **`RcCoyoneda`** (`fp-library/src/types/rc_coyoneda.rs`): An `Rc`-based variant enabling `Clone`. Uses `Rc<dyn RcCoyonedaLowerRef>` layers. Not `Send`. Has `Functor` and `Foldable` via brand.
- **`ArcCoyoneda`** (`fp-library/src/types/arc_coyoneda.rs`): An `Arc`-based variant enabling `Clone + Send + Sync`. Uses `Arc<dyn ArcCoyonedaLowerRef>` layers with `unsafe` `Send`/`Sync` impls. Has `Foldable` via brand but no `Functor` (due to HKT closure bound limitations).

Related files reviewed: `fp-library/src/brands.rs`, `fp-library/src/kinds.rs`, `fp-library/src/classes/functor.rs`, `fp-library/src/classes/foldable.rs`, `fp-library/src/functions.rs`, `fp-library/benches/benchmarks/coyoneda.rs`, and inline test modules.

---

## 2. Identified Flaws, Issues, and Limitations

### 2.1. No Map Fusion in `Coyoneda` (Fundamental Design Limitation)

**Location:** `coyoneda.rs`, lines 279-294 (`CoyonedaMapLayer::lower`)

Each chained `.map()` on `Coyoneda` adds a new `Box<dyn CoyonedaInner>` layer. At `lower()` time, each layer calls `F::map` independently. For k chained maps on a `Vec` of size n, this results in k full traversals of the container, giving O(k \* n) work instead of the O(n) that PureScript's Coyoneda achieves.

The documentation at lines 56-63 acknowledges this and correctly identifies the root cause (Rust's dyn-compatibility rules prevent generic methods on trait objects, so functions cannot be composed across the existential boundary). However, the practical consequence is that `Coyoneda` does not deliver the primary performance benefit that motivates Coyoneda in functional programming literature. A user reading about Coyoneda expects map fusion; this implementation provides deferred mapping but not fused mapping.

**Approaches:**

A. **Status quo with better documentation.** The limitation is already documented. Emphasize more prominently in the module-level docs that users should prefer `CoyonedaExplicit` for performance-sensitive paths and that `Coyoneda` is primarily useful for HKT polymorphism over non-`Functor` types.

- _Trade-off:_ No code change, but users may still be surprised.

B. **Internal `CoyonedaExplicit` accumulation.** Store a `CoyonedaExplicit` inside each layer, fusing functions where possible, and only creating a new trait-object layer when the type changes. This is complex and may not be feasible because the type change happens on every `.map()` call with a different output type.

- _Trade-off:_ Significant implementation complexity for marginal gain in the common case.

C. **Type-erased function composition via `Box<dyn Any>`.** Use `Any` downcasting to attempt runtime function composition when types match. This adds runtime overhead and is fragile.

- _Trade-off:_ Complexity, unsafety, and marginal benefit.

**Recommendation:** Approach A. The limitation is fundamental to Rust's type system. The documentation already explains it well. Consider adding a "When to use which" section at the top of `coyoneda.rs` that more prominently directs users to `CoyonedaExplicit`.

---

### 2.2. Stack Overflow Risk in `Coyoneda::lower()` (Partially Mitigated)

**Location:** `coyoneda.rs`, lines 279-294

`CoyonedaMapLayer::lower()` calls `self.inner.lower()` recursively. With k chained maps, this creates k stack frames. The `stacker` feature provides adaptive stack growth (line 282-288), but:

1. **The `stacker` feature is optional.** Without it, deep chains (thousands of maps) silently overflow the stack. There is no compile-time or runtime warning.
2. **`collapse()` requires `F: Functor`.** Users who use `Coyoneda` specifically to defer the `Functor` requirement cannot use `collapse()` to manage stack depth.
3. **`RcCoyoneda` and `ArcCoyoneda` have the same recursive lowering pattern but have NO `stacker` integration and NO `collapse()` method.** (`rc_coyoneda.rs` line 175; `arc_coyoneda.rs` line 215). These variants are completely unprotected against stack overflow.

**Approaches:**

A. **Add `stacker` support to `RcCoyoneda` and `ArcCoyoneda`.** Mirror the `#[cfg(feature = "stacker")]` guard from `CoyonedaMapLayer::lower()`.

- _Trade-off:_ Minimal code change, consistent behavior across variants.

B. **Add `collapse()` to `RcCoyoneda` and `ArcCoyoneda`.** Both support `lower_ref()`, so `collapse()` could clone, lower, and re-lift.

- _Trade-off:_ Requires `F: Functor`, which defeats part of the purpose of Coyoneda.

C. **Make `stacker` a default feature.** Users who need stack safety get it without opting in.

- _Trade-off:_ Adds a dependency by default; may be undesirable for `no_std` or embedded targets.

D. **Convert recursive lowering to iterative (trampoline-based) lowering.** This would require restructuring the inner trait to support iterative unwinding, which is challenging with trait objects.

- _Trade-off:_ Significant refactoring; may require changing the inner trait's API.

**Recommendation:** Approach A (add `stacker` to `RcCoyoneda` and `ArcCoyoneda`) is the minimum fix. Approach B (add `collapse()`) is also valuable. The combination of A and B provides consistent behavior across all variants.

---

### 2.3. `Fn` Instead of `FnOnce` for Mapping Functions

**Location:** All four files. Examples: `coyoneda.rs` line 525, `rc_coyoneda.rs` line 318, `arc_coyoneda.rs` line 356, `coyoneda_explicit.rs` line 210.

All `.map()` methods require `impl Fn(A) -> B`, not `impl FnOnce(A) -> B`. This means:

1. Closures that capture values by move and consume them (e.g., `move |x| { drop(expensive); transform(x) }`) cannot be used.
2. For `Coyoneda`, the function is stored inline in `CoyonedaMapLayer` and called exactly once at `lower()` time (via `Box<Self>` which is consumed). `FnOnce` would be safe and more permissive here.
3. For `RcCoyoneda` and `ArcCoyoneda`, the function is wrapped in `Rc`/`Arc` and called via `&self` in `lower_ref()`, so `Fn` is genuinely required (the function may be called multiple times across clones).
4. For `CoyonedaExplicit`, the function is used in `compose()` which requires `Fn` (for repeated application across container elements).

The constraint is ultimately driven by `F::map` which takes `impl Fn(A) -> B + 'a`, not `FnOnce`. Since `Functor::map` processes containers that may have multiple elements (e.g., `Vec`), `Fn` is correct at the type class level. But the `Fn` requirement on the user-facing `.map()` API is a restriction that could be relaxed for `Coyoneda::map()` since the function is stored, not immediately applied, and `Fn` is only needed at `lower()` time. However, since lowering calls `F::map` which requires `Fn`, the function ultimately needs to be `Fn` anyway.

**Approaches:**

A. **Keep `Fn` everywhere.** It is consistent with the `Functor::map` signature.

- _Trade-off:_ Restricts users who want to use `FnOnce` closures.

B. **Accept `FnOnce` in `Coyoneda::map()` and coerce to `Fn` internally via `OnceCell` or `Option<F>` wrapping.** Store the function in an `Option`, take it out on first call, panic on second call.

- _Trade-off:_ Adds runtime overhead and a potential panic. Breaks the `Fn` contract if the function is somehow called twice.

**Recommendation:** Approach A. The `Fn` requirement is inherent to functor mapping over containers with multiple elements. This is not a bug but a fundamental property of the design. Documenting why `Fn` is required (because the function may be applied to each element of the container) would be helpful.

---

### 2.4. `RcCoyoneda` and `ArcCoyoneda` Re-Compute on Every `lower_ref()` Call

**Location:** `rc_coyoneda.rs` lines 172-178, `arc_coyoneda.rs` lines 212-218

`lower_ref()` borrows `&self` and recomputes the entire map chain on every call. For a chain of k maps over a `Vec` of size n, each `lower_ref()` call does O(k \* n) work. Despite the `Rc`/`Arc` wrapper enabling sharing, there is no memoization of the lowered result.

This is especially problematic because the cloneable design of `RcCoyoneda`/`ArcCoyoneda` encourages multiple `lower_ref()` calls (e.g., "compute once, read many times"). The current design computes every time.

**Approaches:**

A. **Add a `LazyCell`/`OnceCell` cache.** Store the lowered result in an internal `RefCell<Option<...>>` (for `Rc`) or `OnceLock` (for `Arc`). First `lower_ref()` computes and caches; subsequent calls return the cached value.

- _Trade-off:_ Requires the result type to be `Clone` (it already is, since the base layer requires `Clone`). Adds a `RefCell`/`OnceLock` to the outer struct, increasing size. The cached result type depends on the output type `A`, which changes with each `.map()`, making this structurally complex with trait objects. The trait object `dyn RcCoyonedaLowerRef` would need to cache results of different types, which is not feasible without type erasure.

B. **Provide a separate `memoize()` method.** This would call `lower_ref()`, cache the result, and return a new `RcCoyoneda` that is just a base layer wrapping the computed value. This is essentially `collapse()` for the ref-counted variants.

- _Trade-off:_ Explicit, simple, and does not change the type. Requires `F: Functor`. The user must remember to call it.

C. **Document the re-computation behavior prominently.** Make it clear that `lower_ref()` re-evaluates the chain each time.

- _Trade-off:_ No code change; relies on user awareness.

**Recommendation:** Approach B (add `collapse()` or `memoize()` to `RcCoyoneda` and `ArcCoyoneda`). This gives users control without adding structural complexity. Combine with approach C for documentation.

---

### 2.5. Missing Type Class Instances for `RcCoyoneda` and `ArcCoyoneda`

**Location:** `rc_coyoneda.rs`, `arc_coyoneda.rs`

`Coyoneda` implements `Functor`, `Pointed`, `Foldable`, `Lift`, `Semiapplicative`, `Semimonad`, and `Monad` (via blanket). `RcCoyoneda` only implements `Functor` and `Foldable`. `ArcCoyoneda` only implements `Foldable` (no `Functor` at all).

Missing instances for `RcCoyonedaBrand`:

- `Pointed`: Straightforward, delegate to `F::pure` and `lift`.
- `Semiapplicative`: Possible since `RcCoyoneda` is `Clone`.
- `Semimonad`: Could lower, bind, and re-lift.
- `Lift`: Could lower both, delegate to `F::lift2`, and re-lift.
- `Traversable`: Since `RcCoyoneda` is `Clone`, this becomes feasible (unlike `Coyoneda`).

Missing instances for `ArcCoyonedaBrand`:

- `Functor`: Documented as impossible due to HKT closure signatures lacking `Send + Sync` bounds (`arc_coyoneda.rs` lines 375-379). This is a genuine limitation of the HKT system.

**Approaches:**

A. **Implement `Pointed`, `Lift`, `Semiapplicative`, `Semimonad` for `RcCoyonedaBrand`.** These are all mechanically straightforward: lower, delegate to `F`, re-lift.

- _Trade-off:_ More code, but improves API parity with `CoyonedaBrand`. All require `F: Functor` (for lowering).

B. **For `ArcCoyonedaBrand`, add a `SendFunctor` or similar trait.** Extend the type class hierarchy with `Send`-aware variants.

- _Trade-off:_ Significant architectural change to the type class system. May not be worth it for this single use case.

**Recommendation:** Approach A for `RcCoyonedaBrand`. For `ArcCoyonedaBrand`, the documented limitation is acceptable; users can work with `ArcCoyoneda` directly via inherent methods.

---

### 2.6. `ArcCoyonedaMapLayer` Uses `unsafe` `Send`/`Sync` Impls

**Location:** `arc_coyoneda.rs`, lines 164-184

The `unsafe impl Send` and `unsafe impl Sync` for `ArcCoyonedaMapLayer` have a blanket bound of only `F: Kind_cdc7cd43dac7585f + 'a` with no `Send`/`Sync` bounds. The SAFETY comment at line 164 says "Both fields are `Arc<dyn ... + Send + Sync>`, which are `Send + Sync`." This reasoning is correct for the two fields:

- `inner: Arc<dyn ArcCoyonedaLowerRef<'a, F, B> + 'a>` -- `ArcCoyonedaLowerRef` requires `Send + Sync + 'a`, and `Arc<dyn Send + Sync>` is itself `Send + Sync`.
- `func: Arc<dyn Fn(B) -> A + Send + Sync + 'a>` -- `Arc<dyn Fn(B) -> A + Send + Sync>` is `Send + Sync`.

The safety argument is sound because all data reachable through the struct is behind `Arc` pointers to `Send + Sync` trait objects. There are no raw pointers or non-`Send`/`Sync` fields.

However, the `ArcCoyonedaBase` unsafe impls (lines 106-118) have a subtlety: they bound on `Of<'a, A>: Send` and `Of<'a, A>: Sync` respectively, but the `Send` impl does not require `Sync` and vice versa. Since `ArcCoyonedaBase` is stored inside `Arc`, and `Arc<T>` requires `T: Send + Sync` for `Arc<T>: Send`, the trait object bound `ArcCoyonedaLowerRef: Send + Sync` provides the necessary protection at the point of use (line 126-129 requires `Clone + Send + Sync`). So the unsafe impls on `ArcCoyonedaBase` itself are broader than needed but not unsound because the `ArcCoyonedaLowerRef` impl constrains `Of<'a, A>: Clone + Send + Sync`.

**Assessment:** The unsafe code is sound. The safety comments could be more thorough in explaining the full reasoning chain (i.e., why the broader bounds on the raw struct impls do not lead to unsoundness due to the narrower bounds at the trait impl level).

**Recommendation:** Add a more detailed safety comment explaining that while `ArcCoyonedaBase` itself has separate `Send` and `Sync` bounds, the struct is only ever constructed behind an `Arc` via `ArcCoyonedaLowerRef` impl, which requires both `Send + Sync`.

---

### 2.7. `CoyonedaExplicitBrand` Requires `B: 'static`

**Location:** `coyoneda_explicit.rs` line 703, `brands.rs` line 160

```rust
impl_kind! {
    impl<F: Kind_cdc7cd43dac7585f + 'static, B: 'static> for CoyonedaExplicitBrand<F, B> {
        type Of<'a, A: 'a>: 'a = BoxedCoyonedaExplicit<'a, F, B, A>;
    }
}
```

The intermediate type `B` must be `'static` when used through the brand. This is a restriction that does not exist for `CoyonedaExplicit` itself (which only requires `B: 'a`). The `'static` bound comes from the fact that brand type parameters must outlive all possible lifetimes `'a` introduced by the `Kind` trait.

This means that `CoyonedaExplicitBrand` cannot be used with borrowed data as the intermediate type. In practice, `B` is typically a value type (e.g., `i32`, `String`), so this is rarely an issue, but it is a theoretical limitation.

**Approaches:**

A. **Document the limitation.** The brands.rs doc comment for `CoyonedaExplicitBrand` (line 149-158) already partially documents this.

- _Trade-off:_ No code change.

B. **Consider a lifetime-parameterized brand.** This is not supported by the current `Kind` system and would require fundamental changes.

- _Trade-off:_ Massive architectural change.

**Recommendation:** Approach A. The `'static` bound is inherent to how brands work in the HKT system. Document it clearly.

---

### 2.8. No `Debug`, `Display`, `PartialEq`, or Other Standard Trait Implementations

**Location:** All four files.

None of the Coyoneda types implement `Debug`, `Display`, `PartialEq`, `Eq`, `Hash`, or `Ord`. This makes debugging difficult: you cannot `println!("{:?}", coyo)` or compare two `Coyoneda` values.

For `Coyoneda` and `RcCoyoneda`/`ArcCoyoneda`, implementing `Debug` would require the inner trait objects to be `Debug`, which is possible but adds bounds. `PartialEq` is fundamentally difficult because comparing functions for equality is undecidable.

For `CoyonedaExplicit`, `Debug` could be implemented when `F::Of<'a, B>: Debug` (just show the stored value), with a note that the function is opaque.

**Approaches:**

A. **Implement `Debug` with opaque function representation.** Show the stored value and mark the function as `<fn>` or `<closure>`.

- _Trade-off:_ Useful for debugging. Requires `F::Of<'a, B>: Debug` bound, which may not always hold.

B. **Skip standard trait implementations.** Functions cannot meaningfully implement `PartialEq` or `Hash`.

- _Trade-off:_ Harder to debug.

**Recommendation:** Implement `Debug` for `CoyonedaExplicit` (where the stored value is accessible). For the trait-object-based variants, a minimal `Debug` impl showing the type name and a placeholder for the contents would be helpful for debugging.

---

### 2.9. `Coyoneda::hoist` Requires `F: Functor` (Diverges from PureScript)

**Location:** `coyoneda.rs`, lines 572-579

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

PureScript's `hoistCoyoneda` does not require `Functor f` because it opens the existential to apply the natural transformation directly to the hidden `F B`. This Rust implementation lowers first (requiring `Functor`), transforms, then re-lifts. This means you cannot use `hoist` to transform between non-`Functor` type constructors, which is one of the key use cases for Coyoneda.

In contrast, `CoyonedaExplicit::hoist` (lines 285-294) does NOT require `F: Functor` because `B` is visible. This is the correct behavior.

The documentation at lines 72-76 acknowledges this limitation.

**Approaches:**

A. **Add a `hoist_inner` method to `CoyonedaInner`.** This would require `hoist_inner<G>` to be generic over the target brand `G`, which is not dyn-compatible.

- _Trade-off:_ Not possible with current Rust trait objects.

B. **Convert to `CoyonedaExplicit` first.** Users could lower to `CoyonedaExplicit`, hoist, and convert back.

- _Trade-off:_ Requires `F: Functor` for the conversion anyway (since `From<Coyoneda> for CoyonedaExplicit` calls `lower()`).

C. **Accept the limitation.** Document it and recommend `CoyonedaExplicit` for functor-free hoist.

- _Trade-off:_ Clear guidance, no code change.

**Recommendation:** Approach C. The limitation is fundamental to Rust's trait object system. `CoyonedaExplicit::hoist` is the correct tool for this use case.

---

### 2.10. `Foldable` for `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` Requires `F: Functor`

**Location:** `coyoneda.rs` lines 662-709, `rc_coyoneda.rs` lines 380-422, `arc_coyoneda.rs` lines 384-426

All three trait-object-based variants require `F: Functor` for `Foldable` because they must lower first, then fold. PureScript's `Foldable (Coyoneda f)` only requires `Foldable f` because it opens the existential to compose the fold function with the accumulated mapping.

In contrast, `CoyonedaExplicit::fold_map` (lines 326-336) only requires `F: Foldable`, matching PureScript's semantics.

The documentation explains this at lines 65-71 of `coyoneda.rs`. The same issue applies to `RcCoyoneda`/`ArcCoyoneda` but is not documented in those files.

**Approaches:**

A. **Add a `fold_map_inner` method to the inner trait.** This would need to be generic over the monoid type `M`, which is not dyn-compatible.

- _Trade-off:_ Not possible with current Rust trait objects.

B. **Document the limitation consistently across all variants.**

- _Trade-off:_ No code change.

C. **Recommend `CoyonedaExplicit` for Functor-free foldable operations.**

- _Trade-off:_ Clear guidance.

**Recommendation:** Approaches B and C combined. Ensure `RcCoyoneda` and `ArcCoyoneda` module docs mention this limitation.

---

### 2.11. `RcCoyoneda` and `ArcCoyoneda` Lack `hoist`, `collapse`, `new`, and Other Methods

**Location:** `rc_coyoneda.rs`, `arc_coyoneda.rs`

`Coyoneda` provides: `new`, `lift`, `lower`, `collapse`, `map`, `hoist`.
`CoyonedaExplicit` provides: `new`, `lift`, `lower`, `map`, `hoist`, `boxed`, `boxed_send`, `fold_map`, `fold_map_with_index`, `traverse`, `apply`, `bind`, `pure`.
`RcCoyoneda` provides: `lift`, `lower_ref`, `map`.
`ArcCoyoneda` provides: `lift`, `lower_ref`, `map`.

The `Rc`/`Arc` variants are missing:

- `new(f, fb)` (general constructor)
- `collapse()` / `memoize()` (flatten accumulated layers)
- `hoist(nat)` (natural transformation)

These omissions limit the utility of the `Rc`/`Arc` variants.

**Approaches:**

A. **Add `new`, `collapse`, and `hoist` to both `RcCoyoneda` and `ArcCoyoneda`.**

- `new` is straightforward: create a `RcCoyonedaMapLayer` directly.
- `collapse` calls `lower_ref()` and wraps the result in a new `lift()`.
- `hoist` requires `F: Functor` (same limitation as `Coyoneda::hoist`).
- _Trade-off:_ More code, better API parity.

B. **Add conversion methods between variants.** E.g., `RcCoyoneda::into_coyoneda()` that calls `lower_ref()` and wraps in `Coyoneda::lift()`.

- _Trade-off:_ Gives users a path to the richer API without duplicating it.

**Recommendation:** Approach A. These are basic operations that users will expect.

---

### 2.12. Benchmark Coverage Is Incomplete

**Location:** `fp-library/benches/benchmarks/coyoneda.rs`

The benchmark only tests `Coyoneda`, `CoyonedaExplicit`, and direct `Vec::map`. It does not benchmark:

- `RcCoyoneda` or `ArcCoyoneda` (to measure the overhead of `Rc`/`Arc` wrapping and cloning).
- Multiple `lower_ref()` calls on `RcCoyoneda`/`ArcCoyoneda` (to measure the re-computation cost).
- `Foldable` operations through Coyoneda.
- The `stacker` feature's overhead.

**Recommendation:** Add benchmarks for `RcCoyoneda`, `ArcCoyoneda`, repeated `lower_ref()` calls, and fold-through-lower patterns to give users data for choosing the right variant.

---

### 2.13. `CoyonedaExplicit::traverse` Returns a Complex Nested Type

**Location:** `coyoneda_explicit.rs`, lines 413-426

```rust
pub fn traverse<G: Applicative + 'a, C: 'a + Clone>(
    self,
    f: impl Fn(A) -> <G as Kind_cdc7cd43dac7585f>::Of<'a, C> + 'a,
) -> <G as Kind_cdc7cd43dac7585f>::Of<'a, CoyonedaExplicit<'a, F, C, C, fn(C) -> C>>
where
    B: Clone,
    F: Traversable,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, C>: Clone,
    <G as Kind_cdc7cd43dac7585f>::Of<'a, C>: Clone, {
    G::map(
        |fc| CoyonedaExplicit::lift(fc),
        F::traverse::<B, C, G>(compose(f, self.func), self.fb),
    )
}
```

The return type `G::Of<'a, CoyonedaExplicit<'a, F, C, C, fn(C) -> C>>` resets the fusion pipeline (both `B` and `A` become `C`, function becomes identity `fn(C) -> C`). This means any maps accumulated before `traverse` are composed into the traversal function (good), but the result starts fresh with an identity function. This is correct semantics, but the return type is verbose and may be confusing to users.

Additionally, the `B: Clone` bound is required because `compose(f, self.func)` creates a closure that needs `self.func: Fn(B) -> A` to be called on each element of the traversal. The `B: Clone` bound is inherited from `F::traverse` which requires `B: Clone` for its input elements. However, this means `traverse` cannot be used with non-`Clone` intermediate types, which could be limiting.

**Recommendation:** This is inherent to the design. The verbose return type could be mitigated with a type alias. Consider adding `type TraversedCoyonedaExplicit<'a, F, C> = CoyonedaExplicit<'a, F, C, C, fn(C) -> C>`.

---

### 2.14. `Coyoneda::Semimonad::bind` Loses Accumulated Maps

**Location:** `coyoneda.rs`, lines 854-859

```rust
fn bind<'a, A: 'a, B: 'a>(
    ma: Apply!(...),
    func: impl Fn(A) -> Apply!(...) + 'a,
) -> Apply!(...) {
    Coyoneda::lift(F::bind(ma.lower(), move |a| func(a).lower()))
}
```

This lowers the input `Coyoneda`, binds via `F::bind`, and the callback `func` returns a `Coyoneda` that is also lowered inside the bind. Both lowerings are O(k \* n) for k accumulated maps. If the user had accumulated maps on `ma`, those are applied during `ma.lower()`, producing a concrete `F A`. The callback then produces a new `Coyoneda<F, B>` which is also lowered.

This is correct but wasteful: the entire map chain is fully materialized before binding. There is no way to avoid this with the current trait-object design, since `F::bind` needs a concrete `F A`.

Similarly, `Semiapplicative::apply` (lines 809-814) lowers both arguments before applying.

**Recommendation:** This is inherent to the design. Consider documenting that `bind` and `apply` act as "fusion barriers" that force evaluation of all accumulated maps.

---

### 2.15. `CoyonedaExplicit::apply` and `bind` Reset the Fusion Pipeline

**Location:** `coyoneda_explicit.rs`, lines 473-486, 517-525

Both `apply` and `bind` return `CoyonedaExplicit<'a, F, C, C, fn(C) -> C>`, resetting the intermediate type to `C` and the function to identity. This means any maps accumulated after `apply`/`bind` start a new fusion chain. This is correct and well-documented (line 434: "This is a fusion barrier"), but users should be aware that interleaving `apply`/`bind` with `map` chains prevents full fusion across the entire computation.

**Recommendation:** The documentation already covers this. No action needed beyond ensuring it is visible.

---

### 2.16. No Conversion Between `RcCoyoneda`/`ArcCoyoneda` and `Coyoneda`/`CoyonedaExplicit`

**Location:** All four files.

There are `From` conversions between `Coyoneda` and `CoyonedaExplicit` (both directions), but no conversions involving `RcCoyoneda` or `ArcCoyoneda`. Users cannot:

- Convert `Coyoneda` to `RcCoyoneda` (to get cloneability).
- Convert `RcCoyoneda` to `Coyoneda` (to access the richer type class instances).
- Convert between `RcCoyoneda` and `ArcCoyoneda`.

**Approaches:**

A. **Add `From` implementations.** `RcCoyoneda`/`ArcCoyoneda` -> `Coyoneda` would require lowering (via `lower_ref`) and re-lifting. `Coyoneda` -> `RcCoyoneda`/`ArcCoyoneda` would require lowering (via `lower`) and re-lifting (requires `Clone` + possibly `Send + Sync`).

- _Trade-off:_ All conversions require `F: Functor`, which is limiting. Some require `Clone`/`Send`/`Sync` on the underlying value.

B. **Add inherent conversion methods with clear names.** E.g., `RcCoyoneda::into_coyoneda()`, `Coyoneda::into_rc_coyoneda()`.

- _Trade-off:_ More explicit about the cost (lowering + re-lifting).

**Recommendation:** Approach B. Inherent methods with descriptive names make the cost obvious.

---

### 2.17. `ArcCoyoneda` Uses `document_module(no_validation)`

**Location:** `arc_coyoneda.rs` line 42

The `ArcCoyoneda` module uses `#[fp_macros::document_module(no_validation)]` while `RcCoyoneda` and `Coyoneda` use `#[fp_macros::document_module]` (with validation). The `CoyonedaExplicit` also uses `#[fp_macros::document_module]` (with validation).

This inconsistency means `ArcCoyoneda`'s documentation is not validated by the macro. If documentation attributes are missing or incorrect, no compile-time warning is emitted. The `no_validation` flag may have been added to work around a specific issue (perhaps related to the `unsafe` impls or the lack of `Functor`), but it reduces documentation quality assurance.

**Recommendation:** Investigate why `no_validation` is needed for `ArcCoyoneda` and fix the underlying issue, or document why validation cannot be applied.

---

### 2.18. `RcCoyonedaMapLayer::lower_ref` Clones the `Rc<dyn Fn>` Unnecessarily

**Location:** `rc_coyoneda.rs`, lines 172-178

```rust
fn lower_ref(&self) -> <F as Kind_cdc7cd43dac7585f>::Of<'a, A>
where
    F: Functor, {
    let lowered = self.inner.lower_ref();
    let func = self.func.clone();
    F::map(move |b| (*func)(b), lowered)
}
```

The `self.func.clone()` clones the `Rc<dyn Fn>`, which is an `Rc` bump (cheap), but then wraps it in a closure `move |b| (*func)(b)`. This closure captures the cloned `Rc` by move. The same pattern appears in `ArcCoyonedaMapLayer` (line 215-218 of `arc_coyoneda.rs`).

The `Rc` clone is necessary because `F::map` takes `impl Fn(B) -> A + 'a` by value, and the closure needs to own the function reference. This is correct but adds a reference count bump per layer per `lower_ref()` call. For deep chains, this means k reference count operations.

**Approaches:**

A. **Keep the current approach.** `Rc` cloning is O(1) and the overhead is negligible compared to the `F::map` call.

- _Trade-off:_ Minimal overhead, simple code.

B. **Use `&self.func` directly.** This would require `F::map` to accept `&dyn Fn` or a reference to the function, which it does not.

- _Trade-off:_ Would require changing the `Functor::map` signature.

**Recommendation:** Approach A. The overhead is negligible. No change needed.

---

### 2.19. Test Coverage Gaps

**Location:** Test modules in all four files.

The test suites are generally solid for basic functionality but have gaps:

1. **No property-based tests.** The library uses QuickCheck elsewhere, but there are no property tests for Coyoneda. Functor laws are tested with specific values, not random inputs.
2. **No stack overflow tests.** There are no tests verifying behavior with deep chains (thousands or tens of thousands of maps). The `many_chained_maps` tests only go to depth 100.
3. **No thread safety tests for `ArcCoyoneda` with shared state.** The single `send_across_thread` test moves the value, not shares it. Testing concurrent `lower_ref()` on a shared `ArcCoyoneda` would be valuable.
4. **No tests for `RcCoyoneda`/`ArcCoyoneda` with `Foldable`.** Only one `fold_map` test for each.
5. **No negative tests.** No compile-fail tests verifying that `RcCoyoneda` is `!Send`, or that `Coyoneda` is `!Clone`.

**Recommendation:** Add property-based tests for functor laws, stack depth tests (with and without `stacker`), concurrent access tests for `ArcCoyoneda`, and compile-fail tests for trait bound violations.

---

### 2.20. Inconsistent `map` Ownership Semantics

**Location:** `rc_coyoneda.rs` line 318-326, `arc_coyoneda.rs` lines 356-364

`RcCoyoneda::map` and `ArcCoyoneda::map` consume `self` by value:

```rust
pub fn map<B: 'a>(
    self,
    f: impl Fn(A) -> B + 'a,
) -> RcCoyoneda<'a, F, B> {
```

Since `RcCoyoneda` is `Clone`, consuming `self` is somewhat surprising. The user must clone before mapping if they want to keep the original. This is consistent with method chaining patterns (`.map(f).map(g).lower_ref()`), but inconsistent with the `Clone`-friendly design. An alternative design would take `&self`, clone the inner `Rc`, and wrap it in a new layer.

**Approaches:**

A. **Keep consuming `self`.** Consistent with `Coyoneda::map` and functional programming conventions where `map` produces a new value.

- _Trade-off:_ Users must clone explicitly if they want to keep the original.

B. **Change to `&self`.** Clone the inner `Rc` internally.

- _Trade-off:_ Breaks the chaining pattern (`.map(f).map(g)` would require `let x = coyo.map(f); let y = x.map(g);` instead of chaining since each `.map` borrows).

**Recommendation:** Approach A. Consuming `self` is the standard functional pattern and enables clean chaining. The `Clone` trait is available for users who need to branch.

---

## 3. Summary of Recommendations

**High Priority (bugs or significant usability issues):**

1. Add `stacker` support to `RcCoyoneda::lower_ref` and `ArcCoyoneda::lower_ref` (issue 2.2).
2. Add `collapse()` method to `RcCoyoneda` and `ArcCoyoneda` (issues 2.4 and 2.11).
3. Add `new(f, fb)` constructor to `RcCoyoneda` and `ArcCoyoneda` (issue 2.11).
4. Implement `Pointed`, `Lift`, `Semiapplicative`, `Semimonad` for `RcCoyonedaBrand` (issue 2.5).

**Medium Priority (usability and completeness):**

5. Add `hoist()` to `RcCoyoneda` and `ArcCoyoneda` (issue 2.11).
6. Add conversion methods between variants (issue 2.16).
7. Add benchmarks for `RcCoyoneda` and `ArcCoyoneda` (issue 2.12).
8. Investigate and resolve the `no_validation` flag on `ArcCoyoneda` (issue 2.17).
9. Implement `Debug` for `CoyonedaExplicit` (issue 2.8).

**Low Priority (documentation and testing):**

10. Document the re-computation behavior of `lower_ref()` prominently (issue 2.4).
11. Document `bind`/`apply` as fusion barriers in `Coyoneda` (issue 2.14).
12. Document the `Foldable` requires `Functor` limitation consistently across all variants (issue 2.10).
13. Add property-based tests, stack depth tests, thread safety tests, and compile-fail tests (issue 2.19).
14. Improve safety comments on `ArcCoyoneda` unsafe impls (issue 2.6).
