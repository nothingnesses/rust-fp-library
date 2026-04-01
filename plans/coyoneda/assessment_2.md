# Coyoneda Implementation Assessment

## 1. Overview

This assessment reviews the four Coyoneda implementations in `fp-library`:

- **`Coyoneda`** (`types/coyoneda.rs`): The primary free functor using `Box<dyn CoyonedaInner>` for existential quantification. Supports full HKT integration (`CoyonedaBrand`), `Functor`, `Pointed`, `Foldable`, `Lift`, `Semiapplicative`, `Semimonad`, and `Monad` (via blanket impl). Not `Clone`, `Send`, or `Sync`.
- **`CoyonedaExplicit`** (`types/coyoneda_explicit.rs`): Exposes the intermediate type `B` as a parameter, enabling zero-cost compile-time function composition. No existential; true single-pass fusion. Has a `BoxedCoyonedaExplicit` alias and `CoyonedaExplicitBrand` for HKT integration. Provides inherent methods for `fold_map`, `fold_map_with_index`, `traverse`, `apply`, `bind`, `hoist`.
- **`RcCoyoneda`** (`types/rc_coyoneda.rs`): `Rc`-wrapped variant that is `Clone`. Uses `Rc<dyn RcCoyonedaLowerRef>` internally. Supports `Functor` and `Foldable` via `RcCoyonedaBrand`. `!Send`, `!Sync`.
- **`ArcCoyoneda`** (`types/arc_coyoneda.rs`): `Arc`-wrapped variant that is `Clone + Send + Sync`. Uses `Arc<dyn ArcCoyonedaLowerRef>` internally. Supports `Foldable` via `ArcCoyonedaBrand`. Does **not** implement `Functor` at the brand level due to missing `Send + Sync` bounds on `Functor::map`'s closure parameter.

Related files reviewed: `brands.rs` (brand definitions), `classes/functor.rs` (`Functor` trait), `kinds.rs` (HKT machinery), `benches/benchmarks/coyoneda.rs` (benchmarks).

---

## 2. Identified Issues

### 2.1. No Map Fusion in `Coyoneda`, `RcCoyoneda`, or `ArcCoyoneda`

**Severity: Design limitation (documented)**

The core `Coyoneda` type, as well as `RcCoyoneda` and `ArcCoyoneda`, do not fuse mapped functions. Each `.map()` call creates a new layer (a trait-object-wrapped struct). When `lower`/`lower_ref` is called, each layer independently calls `F::map`, resulting in k traversals of the underlying structure for k chained maps.

For `Vec` with 1000 elements and 100 chained maps, this means 100 separate `Vec::map` calls, each allocating a new `Vec`. PureScript's `Coyoneda` composes functions eagerly and calls `F::map` exactly once.

The module-level documentation in `coyoneda.rs` (lines 56-63) correctly explains this and points users to `CoyonedaExplicit` for single-pass fusion. However, this means the primary motivation of Coyoneda (fusing maps for performance) is not achieved by the main `Coyoneda` type.

**Root cause:** Rust's dyn-compatibility rules prevent a `map_inner<C>` method on the inner trait, which would be needed to compose functions across the existential boundary.

**Approaches:**

A. **Status quo (documented limitation).** Continue pointing users to `CoyonedaExplicit` for fusion. Trade-off: the most natural-to-use type (`Coyoneda`) is also the worst-performing one, which is a usability trap.

B. **Hybrid approach.** Store a `Vec<Box<dyn FnOnce>>` or a function-composition chain inside `Coyoneda` that collapses to a single composed `Box<dyn Fn>` at lower time. This would require type-erased function composition, which is possible for `A -> A` (endomorphisms) but not for `A -> B -> C` chains where types differ. Trade-off: only works when consecutive maps share compatible types; complex implementation.

C. **Auto-collapse.** Automatically call `collapse()` every N layers to bound the overhead. Trade-off: requires `F: Functor` at `map` time, which defeats the purpose of the free functor (deferring the `Functor` constraint).

D. **Make `CoyonedaExplicit` the primary API.** Since `CoyonedaExplicit` achieves actual fusion, consider promoting it and deprecating `Coyoneda` for performance-sensitive paths. Trade-off: `CoyonedaExplicit` exposes `B` as a type parameter, making it harder to use in generic contexts.

**Recommendation:** Approach A is reasonable given Rust's constraints. However, the documentation should more prominently warn that `Coyoneda` does not provide the fusion benefit that motivates Coyoneda in other languages. Consider adding a performance note to the struct-level docs of `Coyoneda` itself (not just the module docs) and to `RcCoyoneda`/`ArcCoyoneda`.

---

### 2.2. `RcCoyoneda` and `ArcCoyoneda` Lack Stack Safety Mitigations

**Severity: Medium**

`Coyoneda` has two stack safety mitigations: the `stacker` feature (adaptive stack growth, line 282-288 of `coyoneda.rs`) and the `collapse()` method (line 494-498). Neither `RcCoyoneda` nor `ArcCoyoneda` has either of these.

The `RcCoyonedaMapLayer::lower_ref` (line 172-178 of `rc_coyoneda.rs`) recursively calls `self.inner.lower_ref()`, which for k chained maps creates k frames of recursion. Similarly for `ArcCoyonedaMapLayer::lower_ref` (line 212-218 of `arc_coyoneda.rs`). With thousands of maps, this will overflow the stack.

**Approaches:**

A. **Add `stacker` support to `RcCoyoneda`/`ArcCoyoneda`.** Wrap the `lower_ref` body in `stacker::maybe_grow(...)` behind the `stacker` feature flag, matching `Coyoneda`'s approach. Trade-off: adds a feature-gated dependency; minimal code change.

B. **Add `collapse_ref()` method.** Since `lower_ref` borrows `&self`, a `collapse` method would need to return a new `RcCoyoneda`/`ArcCoyoneda` (not mutate in-place). This is natural since both types are `Clone`. Trade-off: user must remember to call it.

C. **Iterative lowering.** Restructure `lower_ref` to iterate rather than recurse. This is challenging because each layer has a different existential type `B` hidden behind the trait object, making it impossible to build a flat function chain without type erasure. Trade-off: likely not possible without major redesign.

**Recommendation:** Implement approach A (add `stacker` support) since it is consistent with `Coyoneda` and requires minimal changes. Also implement approach B (add `collapse_ref`) for parity with `Coyoneda::collapse`.

---

### 2.3. `RcCoyoneda`/`ArcCoyoneda` `lower_ref` Clones the Rc/Arc Function Per-Layer on Each Call

**Severity: Low-Medium (performance)**

In `RcCoyonedaMapLayer::lower_ref` (line 176 of `rc_coyoneda.rs`):

```rust
let func = self.func.clone();
F::map(move |b| (*func)(b), lowered)
```

The `Rc` wrapping the function is cloned (refcount bump) on every `lower_ref` call. For k layers and repeated `lower_ref` calls, this performs k `Rc::clone` operations per call. While `Rc::clone` is O(1), the indirection through `(*func)(b)` also means every element passes through an extra pointer dereference and a closure that wraps the actual function.

The same pattern exists in `ArcCoyonedaMapLayer::lower_ref` (line 216-217 of `arc_coyoneda.rs`), where `Arc::clone` involves an atomic increment, which is slightly more expensive.

**Approaches:**

A. **Use a reference instead of cloning.** Replace `let func = self.func.clone();` with a direct reference: `let func = &self.func;` and then `F::map(move |b| func(b), lowered)`. This works because `lower_ref` borrows `&self`, so `&self.func` lives long enough. The closure would capture a `&&dyn Fn(B) -> A` reference. Trade-off: may require adjusting lifetimes; eliminates the refcount bump.

B. **Accept the current cost.** Rc/Arc clone is O(1) per layer. Trade-off: marginal overhead for simplicity.

**Recommendation:** Approach A should be investigated. If `F::map` accepts `impl Fn(A) -> B + 'a` and `lower_ref` returns a value with lifetime `'a`, then the reference to `self.func` (which has lifetime `'a` because `self: &'a_borrow Self` where `Self: 'a`) should be usable. However, the closure capturing `&Rc<dyn Fn(B) -> A>` needs to satisfy `Fn(B) -> A + 'a`, which it would since the `Rc` (and thus its content) lives for `'a`. This is a straightforward optimization.

---

### 2.4. `ArcCoyonedaBrand` Does Not Implement `Functor`

**Severity: Design limitation (documented)**

`ArcCoyonedaBrand` cannot implement `Functor` because `Functor::map` accepts `impl Fn(A) -> B + 'a`, which lacks `Send + Sync` bounds. Storing such a closure in an `Arc`-wrapped layer would be unsound. This is documented at line 375-379 of `arc_coyoneda.rs`.

This means `ArcCoyoneda` cannot participate in generic functor-polymorphic code through the HKT system. Users must use its inherent `map` method directly.

**Approaches:**

A. **Add a `SendFunctor` trait.** Define a separate `SendFunctor` trait whose `map` requires `impl Fn(A) -> B + Send + Sync + 'a`. `ArcCoyonedaBrand` would implement this. Trade-off: proliferates the type class hierarchy; code generic over `Functor` still cannot use `ArcCoyoneda`.

B. **Parameterize `Functor` by closure bounds.** This would require a major redesign of the trait system, likely with associated types or additional type parameters. Trade-off: massive breaking change.

C. **Accept the limitation.** The library already has a `SendThunkBrand` with the same limitation. Trade-off: consistent but limits HKT polymorphism for thread-safe types.

**Recommendation:** Approach C (accept the limitation) is appropriate. This is a fundamental tension between Rust's type system and HKT encoding. The existing documentation is clear. If `SendFunctor` is ever added for other purposes, `ArcCoyonedaBrand` should implement it.

---

### 2.5. `ArcCoyoneda` Uses `no_validation` for `document_module`

**Severity: Low (documentation quality)**

Line 42 of `arc_coyoneda.rs`:

```rust
#[fp_macros::document_module(no_validation)]
```

All other Coyoneda modules use `#[fp_macros::document_module]` (with validation). The `no_validation` flag suppresses compile-time warnings about missing documentation attributes. This suggests `ArcCoyoneda` may have documentation gaps that the macro would normally flag.

**Approaches:**

A. **Remove `no_validation` and fix any resulting warnings.** Trade-off: small effort for better documentation consistency.

B. **Keep `no_validation`.** Trade-off: documentation may be incomplete.

**Recommendation:** Approach A. Remove the flag and fix any warnings.

---

### 2.6. `RcCoyoneda`/`ArcCoyoneda` Missing API Methods Compared to `Coyoneda`

**Severity: Medium (API consistency)**

`Coyoneda` provides: `new`, `lift`, `lower`, `map`, `collapse`, `hoist`.
`CoyonedaExplicit` provides: `new`, `lift`, `lower`, `map`, `hoist`, `fold_map`, `fold_map_with_index`, `traverse`, `apply`, `bind`, `boxed`, `boxed_send`, `pure`.
`RcCoyoneda` provides: `lift`, `lower_ref`, `map`.
`ArcCoyoneda` provides: `lift`, `lower_ref`, `map`.

Missing from `RcCoyoneda`/`ArcCoyoneda`:

1. **`new(f, fb)` constructor.** Neither variant has a general constructor that takes a function and a functor value directly. Users must `lift` then `map`.
2. **`collapse` / `collapse_ref`.** As discussed in issue 2.2, no way to flatten layers.
3. **`hoist`.** No natural transformation support.
4. **`Pointed`/`Semiapplicative`/`Semimonad` brand instances.** `CoyonedaBrand` has these; `RcCoyonedaBrand`/`ArcCoyonedaBrand` do not.

**Approaches:**

A. **Add missing methods.** Implement `new`, `collapse_ref`, and `hoist` on both types. Add `Pointed`, `Semiapplicative`, `Semimonad` brand instances for `RcCoyonedaBrand`. Trade-off: more code to maintain; but improves consistency.

B. **Document the deliberate omissions.** If some methods are intentionally absent, add notes explaining why. Trade-off: at least users know what to expect.

**Recommendation:** Approach A for `new` and `collapse_ref` (small effort, high utility). Approach A for `Pointed` on `RcCoyonedaBrand` (straightforward via `F::pure` then `lift`). `hoist` on `RcCoyoneda`/`ArcCoyoneda` would require `F: Functor` (same as `Coyoneda::hoist`), so it is feasible.

---

### 2.7. No `lower` (Consuming) Method on `RcCoyoneda`/`ArcCoyoneda`

**Severity: Low-Medium**

Both `RcCoyoneda` and `ArcCoyoneda` only provide `lower_ref(&self)`, which clones the base value. There is no `lower(self)` that could avoid the clone when the caller is the sole owner.

For `Rc`, one could check `Rc::try_unwrap` at each layer to avoid cloning if the refcount is 1. For `Arc`, `Arc::try_unwrap` provides the same capability.

**Approaches:**

A. **Add `lower(self)` method.** Attempt `Rc::try_unwrap`/`Arc::try_unwrap` on the inner layers; fall back to cloning if the refcount is > 1. Trade-off: adds complexity; the fallback path is the same as `lower_ref`.

B. **Add `into_inner(self)` that returns `Option<F::Of<'a, A>>`.** Only succeeds if all refcounts are 1. Trade-off: API is awkward for callers.

C. **Keep `lower_ref` only.** Users who want consuming lower should use `Coyoneda` instead. Trade-off: forces users to choose the right variant upfront.

**Recommendation:** Approach A would be ideal but requires traversing the layer stack with `try_unwrap`, which is complex due to the trait object design (the inner trait would need a `lower_owned(self: Box<Self>)` fallback method or similar). Approach C is acceptable if the documentation clearly states the trade-off.

---

### 2.8. `CoyonedaExplicitBrand` Functor Re-boxes on Every `map`

**Severity: Medium (performance misleading)**

The `Functor` implementation for `CoyonedaExplicitBrand` (line 747-752 of `coyoneda_explicit.rs`) calls `fa.map(func).boxed()`:

```rust
fn map<'a, A: 'a, C: 'a>(
    func: impl Fn(A) -> C + 'a,
    fa: ...,
) -> ... {
    fa.map(func).boxed()
}
```

This means every `map` through the HKT brand allocates a new `Box` for the composed function. While the functions still compose (single `F::map` at lower time), each HKT `map` allocates one box. This undermines the "zero-cost" property that `CoyonedaExplicit` is designed around.

The inherent `map` method on `CoyonedaExplicit` is truly zero-cost (inline composition), but the brand path is not.

**Approaches:**

A. **Document the per-map boxing cost for the brand path.** Make it clear that zero-cost fusion only applies when using inherent methods, not the HKT brand. Trade-off: transparency.

B. **Remove the `CoyonedaExplicitBrand` `Functor` instance.** Force users who want zero-cost fusion to use inherent methods. Trade-off: reduces HKT integration.

C. **Accept the trade-off.** The brand path must use `BoxedCoyonedaExplicit` to have a uniform type for `Kind::Of<'a, A>`, so boxing is inherent. Trade-off: users need to understand when to use the brand vs. inherent methods.

**Recommendation:** Approach A. The current documentation on `CoyonedaExplicitBrand` at line 149-151 of `brands.rs` mentions "single-pass fusion" but does not mention the per-map boxing cost. Add a note.

---

### 2.9. No Conversions Between Coyoneda Variants

**Severity: Low**

There are no `From`/`Into` conversions between `Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda`. There are conversions between `Coyoneda` and `CoyonedaExplicit`, but none involving the reference-counted variants.

A user who builds a `Coyoneda` chain and later needs `Clone` cannot convert it to `RcCoyoneda` without lowering and re-lifting (requiring `F: Functor`).

**Approaches:**

A. **Add `From<Coyoneda> for RcCoyoneda` and `From<Coyoneda> for ArcCoyoneda`.** These would lower the `Coyoneda` (requiring `F: Functor`), then lift into the target. Trade-off: requires `F: Functor` and `F::Of<'a, A>: Clone` (for `Rc`) or `+ Send + Sync` (for `Arc`).

B. **Add `From<RcCoyoneda> for ArcCoyoneda`.** This could lower via `lower_ref` and re-lift. Trade-off: requires `F: Functor` and the appropriate bounds.

C. **Keep the current situation.** Users can manually do `RcCoyoneda::lift(coyoneda.lower())`. Trade-off: verbose but explicit.

**Recommendation:** Approach A for the common case (`Coyoneda -> RcCoyoneda`). The conversion is lossy (it forces lowering), so documenting the cost is important. Approach C is also acceptable.

---

### 2.10. `Foldable` Implementations for All Brand-Based Coyoneda Types Require `F: Functor`

**Severity: Medium (semantic deviation from PureScript)**

PureScript's `Foldable` for `Coyoneda` does **not** require `Functor f`; it opens the existential to compose the fold function with the accumulated mapping function, folding `F B` in a single pass. The Rust implementations all lower first (requiring `F: Functor`), then fold:

- `CoyonedaBrand<F>`: `Foldable` requires `F: Functor + Foldable` (line 662 of `coyoneda.rs`).
- `RcCoyonedaBrand<F>`: `Foldable` requires `F: Functor + Foldable` (line 380 of `rc_coyoneda.rs`).
- `ArcCoyonedaBrand<F>`: `Foldable` requires `F: Functor + Foldable` (line 384 of `arc_coyoneda.rs`).

Only `CoyonedaExplicit` provides `Foldable` without `F: Functor` (both the inherent `fold_map` method and the `CoyonedaExplicitBrand` implementation). This is correct but means the main `Coyoneda` type has a strictly weaker `Foldable` than the PureScript equivalent.

The documentation at lines 64-71 of `coyoneda.rs` correctly explains this.

**Approaches:**

A. **Add a `fold_map_inner` method to the `CoyonedaInner` trait.** This would be generic over the monoid type `M`, which breaks dyn-compatibility. Not possible without fundamental redesign.

B. **Accept the limitation.** This is inherent to Rust's dyn-compatibility. Trade-off: documented, understood.

C. **Recommend `CoyonedaExplicit` for `Foldable`-without-`Functor`.** Already done in the docs. Trade-off: requires users to know about both types.

**Recommendation:** Approach B. The limitation is fundamental. The documentation is good. Consider adding a cross-reference from the `Foldable` impl's doc comment to `CoyonedaExplicit` as the alternative.

---

### 2.11. `RcCoyoneda`/`ArcCoyoneda` Allocate Two Pointers Per `map`

**Severity: Low (performance, documented)**

Each `RcCoyoneda::map` creates:

1. An `Rc` for the `RcCoyonedaMapLayer` (wrapping the layer struct).
2. An `Rc` for the function (`Rc<dyn Fn(B) -> A>`).

Similarly for `ArcCoyoneda::map` with `Arc`. In contrast, `Coyoneda::map` creates only one `Box` (the function is stored inline in `CoyonedaMapLayer` and erased by the outer `Box<dyn CoyonedaInner>`).

The `RcCoyoneda` module documentation (lines 10-11) correctly documents this. However, the `Coyoneda` approach of storing the function inline is more efficient and could potentially be applied here.

**Approaches:**

A. **Store the function inline in the map layer struct.** Change `RcCoyonedaMapLayer` to be generic over `Func: Fn(B) -> A`, store `func: Func` inline, and let the outer `Rc<dyn RcCoyonedaLowerRef>` erase the type. Trade-off: reduces to 1 `Rc` per `map`; mirrors `Coyoneda`'s approach. However, this requires the `Func` type to be known at construction time and erased by the outer `Rc`, which should work.

B. **Keep the current approach.** Trade-off: 2 Rc/Arc allocations per map, but simpler code.

**Recommendation:** Approach A. The `Coyoneda` type already demonstrates this pattern with `CoyonedaMapLayer<..., Func>`. The same technique should work for `RcCoyonedaMapLayer` and `ArcCoyonedaMapLayer`, reducing allocations from 2 to 1 per `map`.

---

### 2.12. No `Debug`, `Display`, or `PartialEq` Implementations

**Severity: Low**

None of the four Coyoneda types implement `Debug`, `Display`, or `PartialEq`. This makes debugging difficult; printing a `Coyoneda` value requires lowering it first.

**Approaches:**

A. **Implement `Debug` where possible.** For `Coyoneda`/`CoyonedaExplicit`, `Debug` could print a placeholder like `Coyoneda(<deferred>)`. For `CoyonedaExplicit` specifically, `Debug` could show the underlying `F B` value if `F::Of<'a, B>: Debug`. Trade-off: limited utility for the boxed variants; more useful for `CoyonedaExplicit`.

B. **Skip `Debug`.** The trait-object-based types cannot meaningfully print their contents without lowering. Trade-off: debuggability suffers.

**Recommendation:** Approach A for `CoyonedaExplicit` (it has a concrete `fb` field that could be printed). For the trait-object-based types, a minimal `Debug` impl showing the type name would be helpful for diagnostics.

---

### 2.13. No Property-Based Tests

**Severity: Medium (testing)**

The test suites for all four Coyoneda types consist entirely of hand-written example-based tests. There are no property-based tests (QuickCheck) verifying functor laws (identity, composition), foldable laws, or monad laws across randomized inputs. The project's `CLAUDE.md` mentions QuickCheck as part of the testing strategy, but no property-based tests exist for any Coyoneda type.

The existing tests cover basic cases but do not exercise edge cases like:

- Empty containers.
- Very large containers.
- Functions that panic.
- Stack overflow thresholds for deep chains.

**Approaches:**

A. **Add QuickCheck property tests.** Test functor identity and composition laws for all brands that implement `Functor`. Test `Foldable` laws. Trade-off: more test code; stronger correctness guarantees.

B. **Add targeted edge-case tests.** At minimum, test with empty `Vec`, `None`, and a chain depth near the stack limit. Trade-off: less comprehensive but catches common issues.

**Recommendation:** Both A and B. Property-based tests for the functor laws and a targeted stack-overflow test are both important.

---

### 2.14. `CoyonedaExplicit::traverse` Returns a Complex Type

**Severity: Low (ergonomics)**

The `traverse` method on `CoyonedaExplicit` (line 413 of `coyoneda_explicit.rs`) returns:

```rust
<G as Kind_cdc7cd43dac7585f>::Of<'a, CoyonedaExplicit<'a, F, C, C, fn(C) -> C>>
```

The inner type is `CoyonedaExplicit<'a, F, C, C, fn(C) -> C>`, which is the identity-function variant. This resets the fusion pipeline: the accumulated function is lost (it was applied during traversal), and the result starts fresh. This is semantically correct but the return type is unwieldy.

Users must annotate the result type explicitly, as shown in test code (lines 1000-1002):

```rust
let result: Option<CoyonedaExplicit<VecBrand, _, _, _>> =
    coyo.traverse::<OptionBrand, _>(|x| if x > 0 { Some(x) } else { None });
```

**Approaches:**

A. **Add a type alias.** Define `type LiftedCoyonedaExplicit<'a, F, A> = CoyonedaExplicit<'a, F, A, A, fn(A) -> A>`. Trade-off: cleaner signatures.

B. **Accept the complexity.** The type is correct and inference handles most cases. Trade-off: ergonomics suffer in some contexts.

**Recommendation:** Approach A if the alias can be used consistently. The `lift` constructor already produces this type, so the alias would be natural.

---

### 2.15. `Coyoneda::hoist` Requires `F: Functor` (Deviates from PureScript)

**Severity: Low (documented)**

PureScript's `hoistCoyoneda` applies the natural transformation directly to the hidden `F B` via `unCoyoneda`, without requiring `Functor f`. The Rust implementation (line 572-579 of `coyoneda.rs`) lowers first, transforms, then re-lifts:

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

This requires `F: Functor` and loses the deferred computation property (all maps are applied before the transformation). `CoyonedaExplicit::hoist` does not have this limitation (line 285-294 of `coyoneda_explicit.rs`).

The limitation is documented at lines 73-76 of `coyoneda.rs`.

**Approaches:**

A. **Accept the limitation.** Same root cause as issue 2.10 (dyn-compatibility). Trade-off: documented.

B. **Add a `hoist_inner` method to `CoyonedaInner`.** This would be generic over the target brand `G`, breaking dyn-compatibility. Not possible.

**Recommendation:** Approach A. Direct users to `CoyonedaExplicit::hoist` for the functor-free variant.

---

### 2.16. `Coyoneda::map` Uses `Fn`, Not `FnOnce`

**Severity: Low (design constraint)**

All `map` methods across all four variants require `Fn`, not `FnOnce`. This is a fundamental constraint: `Functor::map` requires `Fn` because multi-element containers (like `Vec`) need to call the function multiple times.

However, for single-element containers (like `Option`, `Identity`), `FnOnce` would suffice and would allow closures that capture non-`Clone` values by move. This is not a bug but a consequence of the universal `Functor` design.

**Approaches:**

A. **Accept the constraint.** `Fn` is required by the `Functor` trait. Trade-off: cannot use `FnOnce` closures with Coyoneda.

B. **Add a `map_once` method for `Coyoneda`.** This would store a `FnOnce` and only work with functors that call the function at most once. Trade-off: the method cannot call `F::map` (which requires `Fn`), so it would need a separate `FunctorOnce` trait.

**Recommendation:** Approach A. This is a trait-level decision, not a Coyoneda-specific issue.

---

### 2.17. Soundness of `unsafe impl Send/Sync` for `ArcCoyonedaBase`

**Severity: Low (currently sound, but fragile)**

Lines 106-118 of `arc_coyoneda.rs` provide manual `unsafe impl Send` and `unsafe impl Sync` for `ArcCoyonedaBase`:

```rust
unsafe impl<'a, F, A: 'a> Send for ArcCoyonedaBase<'a, F, A>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    <F as Kind_cdc7cd43dac7585f>::Of<'a, A>: Send,
{ }
```

The only field is `fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A>`, so the `Send` bound on `fa` is sufficient. This is sound.

Similarly, the `unsafe impl Send/Sync` for `ArcCoyonedaMapLayer` (lines 171-184) is sound because:

- `inner: Arc<dyn ArcCoyonedaLowerRef<...> + 'a>` where `ArcCoyonedaLowerRef: Send + Sync`, so the trait object is `Send + Sync`, and `Arc<T: Send + Sync>` is `Send + Sync`.
- `func: Arc<dyn Fn(B) -> A + Send + Sync + 'a>` is explicitly `Send + Sync`.

However, the `unsafe impl` for `ArcCoyonedaMapLayer` has no bounds beyond `F: Kind_cdc7cd43dac7585f + 'a`. This is correct because both fields are `Arc<dyn ... + Send + Sync>`, which are unconditionally `Send + Sync`. But the absence of bounds makes the safety argument non-local: if someone adds a non-`Send` field to the struct, the `unsafe impl` would become unsound silently.

**Approaches:**

A. **Add `// SAFETY` comments that are more specific.** List the exact invariants that make the impl sound. Trade-off: documentation effort.

B. **Use `static_assertions` or `assert_impl_all!` to verify at compile time.** Trade-off: adds a dev dependency.

C. **Restructure to avoid `unsafe`.** If the struct fields all implement `Send + Sync`, the compiler should derive these traits automatically. The reason it does not is that `Kind_cdc7cd43dac7585f` (the brand trait) does not require `Send + Sync`, so the compiler cannot prove `ArcCoyonedaBase` is `Send`. Trade-off: adding `Send + Sync` to `Kind_cdc7cd43dac7585f` would break other uses.

**Recommendation:** Approach A is already partially done. The safety comments at lines 103-104 and 164 are adequate but could be more explicit about what would break the invariant if the struct were modified.

---

### 2.18. Benchmark Coverage Is Limited

**Severity: Low**

The benchmarks in `benches/benchmarks/coyoneda.rs` only test `Coyoneda` and `CoyonedaExplicit` against direct `Vec::map`. They do not benchmark `RcCoyoneda` or `ArcCoyoneda`, and they do not measure the overhead of `lower_ref` (with its cloning) vs. `lower`.

**Approaches:**

A. **Add benchmarks for `RcCoyoneda` and `ArcCoyoneda`.** Include `lower_ref` with cloning overhead, multi-call `lower_ref` scenarios, and comparison against `Coyoneda::lower`. Trade-off: more benchmark code; better performance visibility.

B. **Add benchmarks for `CoyonedaExplicit` without `.boxed()`.** The current benchmark always calls `.boxed()` in the loop, which measures the boxed path. A benchmark without boxing would show the true zero-cost behavior. Trade-off: the non-boxed type changes on each iteration, so benchmarking it in a loop requires a different approach.

**Recommendation:** Approach A. `RcCoyoneda`/`ArcCoyoneda` benchmarks would provide useful data about the cost of refcounting and cloning.

---

## 3. Summary of Recommendations

| Issue                                             | Priority | Action                                                           |
| ------------------------------------------------- | -------- | ---------------------------------------------------------------- |
| 2.2. No stack safety in Rc/ArcCoyoneda            | High     | Add `stacker` support and `collapse_ref`                         |
| 2.6. Missing API methods on Rc/ArcCoyoneda        | Medium   | Add `new`, `collapse_ref`, `hoist`; add `Pointed` brand instance |
| 2.13. No property-based tests                     | Medium   | Add QuickCheck tests for functor/foldable laws                   |
| 2.3. Unnecessary Rc/Arc clone in `lower_ref`      | Medium   | Use references instead of cloning                                |
| 2.11. Two allocations per `map` in Rc/ArcCoyoneda | Medium   | Store function inline (like `Coyoneda`)                          |
| 2.8. CoyonedaExplicitBrand re-boxes per map       | Low-Med  | Document the per-map cost                                        |
| 2.5. `no_validation` on ArcCoyoneda               | Low      | Remove flag, fix warnings                                        |
| 2.18. Limited benchmark coverage                  | Low      | Add Rc/ArcCoyoneda benchmarks                                    |
| 2.12. No Debug/Display/PartialEq                  | Low      | Add minimal Debug impls                                          |
| 2.1. No map fusion in Coyoneda                    | N/A      | Accept; documented design limitation                             |
| 2.4. ArcCoyonedaBrand lacks Functor               | N/A      | Accept; documented design limitation                             |
| 2.10. Foldable requires Functor                   | N/A      | Accept; documented design limitation                             |
| 2.14. Complex traverse return type                | Low      | Consider type alias                                              |
| 2.15. hoist requires Functor                      | N/A      | Accept; documented design limitation                             |
| 2.7. No consuming `lower` on Rc/Arc               | Low      | Accept unless try_unwrap pattern is worth the complexity         |
| 2.9. No conversions between variants              | Low      | Add `From` impls for common cases                                |
| 2.16. Fn not FnOnce                               | N/A      | Accept; trait-level constraint                                   |
| 2.17. unsafe impl soundness                       | Low      | Improve SAFETY comments                                          |
