# Coyoneda Implementation Assessment: Summary

This document synthesizes findings from five independent code reviews of the Coyoneda implementations (`Coyoneda`, `CoyonedaExplicit`, `RcCoyoneda`, `ArcCoyoneda`). Issues are grouped by consensus strength and priority.

---

## Consensus Legend

- **5/5**: Identified by all five reviewers.
- **4/5**, **3/5**, etc.: Number of reviewers who independently flagged the issue.

---

## 1. High-Priority Issues

### 1.1 No Stack Safety in `RcCoyoneda` or `ArcCoyoneda` (5/5)

`Coyoneda` has two stack overflow mitigations: `stacker` feature support in `CoyonedaMapLayer::lower` and a `collapse()` method. Neither `RcCoyoneda` nor `ArcCoyoneda` has either. Their `lower_ref` methods recurse through k layers identically to `Coyoneda::lower`, meaning deep chains (thousands of maps) will overflow the stack.

Since `RcCoyoneda`/`ArcCoyoneda` are `Clone`, users are more likely to build long chains in loops (clone-and-extend patterns), making this gap especially dangerous.

**Files:** `rc_coyoneda.rs:172-178`, `arc_coyoneda.rs:212-218`

**Consensus recommendation:** Add `stacker` support to `lower_ref` (approach matches `Coyoneda`) and add a `collapse` method to both types. All five reviewers agree on this.

---

### 1.2 Missing Type Class Instances for `RcCoyonedaBrand` (5/5)

`CoyonedaBrand` implements `Functor`, `Pointed`, `Foldable`, `Lift`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, and `Monad` (blanket). `RcCoyonedaBrand` only implements `Functor` and `Foldable`.

| Type class        | `CoyonedaBrand` | `RcCoyonedaBrand` | `ArcCoyonedaBrand` |
| ----------------- | --------------- | ----------------- | ------------------ |
| `Functor`         | Yes             | Yes               | No (documented)    |
| `Pointed`         | Yes             | **No**            | No                 |
| `Foldable`        | Yes             | Yes               | Yes                |
| `Lift`            | Yes             | **No**            | No                 |
| `ApplyFirst`      | Yes             | **No**            | No                 |
| `ApplySecond`     | Yes             | **No**            | No                 |
| `Semiapplicative` | Yes             | **No**            | No                 |
| `Semimonad`       | Yes             | **No**            | No                 |

**Feasibility assessment (post-review):** Brand-level trait impls for `RcCoyonedaBrand` are **not possible**. The blocker is a `Clone` bound that cannot be expressed in the trait method signatures:

- `RcCoyoneda` wraps `Rc<dyn RcCoyonedaLowerRef>`. Constructing this requires `F::Of<'a, A>: Clone` because `RcCoyonedaBase`'s `RcCoyonedaLowerRef` impl has that bound, and the bound must be satisfied to coerce the struct into the trait object.
- `CoyonedaBrand` avoids this because `Coyoneda::lift` has no `Clone` requirement; its `CoyonedaBase::lower` consumes `self: Box<Self>` (moving, not cloning).
- The trait method signatures (`Pointed::pure`, `Semimonad::bind`, `Lift::lift2`, `Semiapplicative::apply`) don't include a `Clone` bound on `F::Of<'a, A>`, and Rust doesn't allow adding extra where clauses to methods in trait impls beyond what the trait definition specifies.

For `ArcCoyonedaBrand`, the same `Clone` blocker applies, compounded by the existing `Send + Sync` limitation on `Functor::map` closures.

**Revised recommendation:** Add inherent methods (`pure`, `apply`, `bind`, `lift2`) directly on both `RcCoyoneda` and `ArcCoyoneda`, where the `Clone` (and `Send + Sync` for Arc) bounds can be specified freely. This follows the pattern established by `CoyonedaExplicit`'s inherent methods.

---

## 2. Medium-Priority Issues

### 2.1 Missing API Methods on Rc/Arc Variants (5/5)

`Coyoneda` provides `new`, `lift`, `lower`, `collapse`, `map`, `hoist`. `RcCoyoneda` and `ArcCoyoneda` only provide `lift`, `lower_ref`, and `map`.

Missing methods:

- **`new(f, fb)`**: General constructor taking a function and a functor value directly.
- **`collapse`**: Flatten accumulated layers by lowering and re-lifting (also serves as stack safety escape hatch).
- **`hoist`**: Apply a natural transformation. Requires `F: Functor` (same limitation as `Coyoneda::hoist`).

**Consensus recommendation:** Add all three methods to both `RcCoyoneda` and `ArcCoyoneda`.

---

### 2.2 No Benchmarks for `RcCoyoneda` or `ArcCoyoneda` (5/5)

The benchmark suite (`benches/benchmarks/coyoneda.rs`) only covers `Coyoneda` and `CoyonedaExplicit`. The overhead of Rc/Arc wrapping, double allocation per map, refcount bumps, and re-evaluation on repeated `lower_ref` calls is not quantified.

**Consensus recommendation:** Add benchmark cases including single `lower_ref` cost, multiple `lower_ref` calls, and clone-then-lower patterns.

---

### 2.3 `document_module(no_validation)` on `ArcCoyoneda` (5/5)

`ArcCoyoneda` uses `#[fp_macros::document_module(no_validation)]` while all other Coyoneda modules use `#[fp_macros::document_module]` with validation enabled. This suppresses compile-time checks for missing documentation attributes.

**Consensus recommendation:** Remove `no_validation`, fix any resulting warnings.

---

### 2.4 No Property-Based Tests (4/5)

Despite the project's testing strategy calling for QuickCheck property-based tests, none exist for any Coyoneda variant. The test suites consist entirely of example-based unit tests with specific values. Functor laws (identity, composition), Foldable laws, and roundtrip laws are not verified with randomized inputs.

Additional gaps noted:

- No stack overflow tests (chains only go to depth 100).
- No thread safety tests for concurrent `ArcCoyoneda::lower_ref` calls.
- No compile-fail tests verifying that `RcCoyoneda` is `!Send` or `Coyoneda` is `!Clone`.

**Consensus recommendation:** Add QuickCheck tests for Functor and Foldable laws. Add targeted stack overflow and compile-fail tests.

---

### 2.5 Unsafe `Send`/`Sync` Verification for `ArcCoyoneda` (3/5)

The `unsafe impl Send/Sync` on `ArcCoyonedaBase` and `ArcCoyonedaMapLayer` are sound (reviewers agree on this), but the safety depends on trait object bounds remaining correct. Three reviewers recommended additional verification:

- **Compile-time assertions** (`static_assertions` or `const` assertions) to verify `Send + Sync`.
- **Compile-fail tests** (`trybuild`) proving that `ArcCoyoneda` with a `!Send` payload fails to compile.
- **More detailed safety comments** explaining the full reasoning chain.

**Consensus recommendation:** Add compile-fail tests as a regression guard. Improve safety comments.

---

### 2.6 Two Allocations Per `map` in Rc/Arc Variants (2/5)

Each `RcCoyoneda::map` creates two `Rc` allocations: one for the `RcCoyonedaMapLayer` (the layer struct) and one for the function (`Rc<dyn Fn(B) -> A>`). `Coyoneda::map` only creates one `Box` because it stores the function inline (monomorphized) and erases it through the outer `Box<dyn CoyonedaInner>`.

The same inline-function technique could be applied to `RcCoyonedaMapLayer` and `ArcCoyonedaMapLayer`: make them generic over `Func: Fn(B) -> A`, store `func: Func` inline, and let the outer `Rc<dyn RcCoyonedaLowerRef>` erase the type. This would halve allocations per `map`.

**Noted by:** Agents 2 and 5.

**Trade-offs:** Reduces allocations from 2 to 1 per `map`. Changes the internal architecture but not the public API. Mirrors the proven `Coyoneda` pattern.

---

### 2.7 No Conversions Between Rc/Arc and Other Variants (5/5)

Conversions exist between `Coyoneda` and `CoyonedaExplicit` (both directions), but no conversions involve `RcCoyoneda` or `ArcCoyoneda`. Users who need to convert must manually `lower_ref()` and `lift()`.

**Consensus recommendation:** Add `From<RcCoyoneda> for Coyoneda` and `From<ArcCoyoneda> for Coyoneda` (both via lower_ref + lift, requiring `F: Functor`).

---

## 3. Low-Priority Issues

### 3.1 No `Debug` Implementations (4/5)

None of the four Coyoneda types implement `Debug`. A minimal implementation showing a placeholder (e.g., `Coyoneda(<opaque>)`) would aid debugging. For `CoyonedaExplicit`, the stored functor value could be shown when it implements `Debug`.

### 3.2 Inherent `fold_map(&self)` for Rc/Arc Variants (2/5)

The `Foldable::fold_map` trait takes `fa` by value, consuming the `RcCoyoneda`/`ArcCoyoneda`. Since the implementation only calls `lower_ref(&self)`, an inherent `fold_map` method taking `&self` would provide a non-consuming fold path.

### 3.3 Missing Consuming `lower(self)` on Rc/Arc (2/5)

Both variants only provide `lower_ref(&self)`, which always clones the base value. A `lower(self)` that attempts `Rc::try_unwrap`/`Arc::try_unwrap` could avoid the clone when the refcount is 1.

### 3.4 `CoyonedaExplicitBrand` Functor Re-boxes Per `map` (1/5)

The brand-level `Functor::map` for `CoyonedaExplicitBrand` calls `fa.map(func).boxed()`, allocating a `Box` on every map. This undermines the "zero-cost" property when used through the HKT system. The documentation should note that zero-cost fusion only applies via inherent methods.

### 3.5 Document `bind`/`traverse` as Fusion Barriers (2/5)

`CoyonedaExplicit::apply` documents itself as a "fusion barrier," but `traverse` and `bind` do not. All three operations force evaluation of accumulated maps and reset the pipeline.

### 3.6 Document `CoyonedaExplicit::boxed()` Loop Overhead (1/5)

When `.boxed()` is used in a loop, each iteration creates a closure chain. The composed function has O(k) per-element overhead, matching `Coyoneda`'s cost profile. The single-`F::map` advantage only materializes with static (compile-time) composition.

### 3.7 Improve Unsafe Impl Safety Comments (3/5)

The safety comments on `ArcCoyonedaMapLayer` could be more thorough about the full invariant chain and what would break if the struct were modified.

---

## 4. Accepted Design Limitations (No Action Needed)

The following issues were identified by multiple reviewers but are fundamental to Rust's type system or the library's HKT encoding. They are correctly documented and require no changes.

| Limitation                                                | Root Cause                                                | Mitigation                                                         |
| --------------------------------------------------------- | --------------------------------------------------------- | ------------------------------------------------------------------ |
| No map fusion in `Coyoneda`                               | Trait objects cannot have generic methods                 | Use `CoyonedaExplicit` for fusion                                  |
| `ArcCoyonedaBrand` cannot implement `Functor`             | HKT `Functor::map` lacks `Send + Sync` bounds             | Use inherent methods directly                                      |
| `Fn` bound instead of `FnOnce`                            | `Functor::map` requires `Fn` for multi-element containers | Library-wide design decision                                       |
| `Foldable`/`hoist`/`Semiapplicative` require `F: Functor` | Cannot open existential through trait objects             | Use `CoyonedaExplicit` for Functor-free variants                   |
| `CoyonedaExplicit` type explosion for deep chains         | Each `map` nests the function type                        | Use `.boxed()` for chains deeper than ~20-30                       |
| `CoyonedaExplicitBrand` requires `B: 'static`             | Brand type parameters must outlive all `'a`               | Use `CoyonedaExplicit` directly (not via brand) for non-static `B` |
| `map` consumes `self` on Clone types                      | Consistency with ownership model and `Functor` trait      | Use `.clone().map(f)` for branching                                |
