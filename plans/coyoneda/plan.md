# Coyoneda Improvement Plan

Issues are ordered by impact and feasibility. Each issue includes alternative approaches where applicable, trade-offs, and recommendations.

---

## 1. Eliminate double allocation per `map` in `Coyoneda`

**File:** `coyoneda.rs`
**Impact:** Performance (halves heap allocations per `map`/`new` call)
**Difficulty:** Low

### Problem

`CoyonedaMapLayer` and `CoyonedaNewLayer` store the mapping function as `Box<dyn Fn(B) -> A>`, requiring a separate heap allocation. The layer struct itself is then boxed as `Box<dyn CoyonedaInner>`. This means two allocations per `map` call.

### Approach

Make both layer structs generic over the function type `Func`:

```rust
struct CoyonedaMapLayer<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> {
    inner: Box<dyn CoyonedaInner<'a, F, B> + 'a>,
    func: Func,  // stored inline
}

struct CoyonedaNewLayer<'a, F, B: 'a, A: 'a, Func: Fn(B) -> A + 'a> {
    fb: <F as Kind>::Of<'a, B>,
    func: Func,  // stored inline
}
```

The `Func` parameter is erased by the outer `Box<dyn CoyonedaInner>`, so it does not leak into the public API of `Coyoneda`. The `lower` method destructures `self: Box<Self>` and passes `self.func` (which implements `Fn`) to `F::map`.

### Trade-offs

- Slightly more monomorphization (one specialization per distinct closure type). Negligible in practice since each `map` call site already generates unique code.
- No API changes. Fully backwards-compatible.

### Recommendation

Implement. This is a straightforward change with no downsides.

---

## 2. Replace `CoyonedaExplicit::bind` with renamed `flat_map`

**File:** `coyoneda_explicit.rs`
**Impact:** API clarity and consistency
**Difficulty:** Low

### Problem

Two monadic bind methods exist:

- `bind` requires `F: Functor + Semimonad`, calls `self.lower()` (materializing accumulated maps), and requires the callback to return a `CoyonedaExplicit` in identity position.
- `flat_map` requires only `F: Semimonad`, composes the bind callback with the accumulated function directly (no intermediate materialization), and has a simpler callback signature (`Fn(A) -> F::Of<'a, C>`).

`flat_map` is strictly better, but `bind` is the canonical name in the library's type class hierarchy (`Semimonad::bind`). The name `flat_map` is a Scala/Kotlin-ism that is inconsistent with the library's vocabulary.

### Approach

Remove the old `bind` (the one with `F: Functor + Semimonad` that lowers before binding). Rename `flat_map` to `bind`. This gives the canonical name to the better implementation.

### Trade-offs

- The callback signature changes from `Fn(A) -> CoyonedaExplicit<...>` to `Fn(A) -> F::Of<'a, C>`. Any code using the old `bind` must adjust. Given the old `bind`'s restrictive signature, this is unlikely to affect real code.
- `bind` is now consistent with `Semimonad::bind` in naming, but its callback returns `F::Of<'a, C>` directly rather than `CoyonedaExplicit`. This is intentional: the direct-return form avoids unnecessary wrapping/unwrapping and is what `Semimonad::bind` does on the underlying functor.

### Recommendation

Implement. Remove old `bind`, rename `flat_map` to `bind`.

---

## 3. Add `RcCoyoneda` and `ArcCoyoneda` for `Clone` support

**File:** `coyoneda.rs` (new functionality)
**Impact:** Enables `Traversable`, `Semiapplicative`, `Extend`, `Comonad`
**Difficulty:** Medium

### Problem

`Coyoneda` wraps `Box<dyn CoyonedaInner>`, which is not `Clone`. This prevents implementing type classes that require cloning the structure (`Traversable`, `Semiapplicative`, `Extend`, `Comonad`).

### Why not a config trait or generic pointer parameter

Earlier iterations considered a `CoyonedaConfig` trait (following the `LazyConfig` pattern) or a `P: RefCountedPointer` parameter. Both were ruled out because `Coyoneda::lower` has consuming semantics (`self: Box<Self>` receiver) while `Lazy::evaluate` has borrowing semantics (`&self -> &A`). This fundamental difference creates two problems that can't be abstracted over cleanly:

1. **Unsizing coercion.** `P::cloneable_new(concrete_value)` returns `P::CloneableOf<'a, ConcreteType>`, but the layer struct needs `P::CloneableOf<'a, dyn Trait>`. Rust applies `CoerceUnsized` for concrete `Rc`/`Arc` but cannot apply it to generic associated types.

2. **Send + Sync for Arc.** `ArcCoyoneda` needs `Arc<dyn Trait + Send + Sync + 'a>` while `RcCoyoneda` needs `Rc<dyn Trait + 'a>`. The `+ Send + Sync` is part of the type, not something a pointer parameter can toggle. A single `RefCoyoneda<P>` would either require `Send + Sync` always (unnecessarily restricting Rc with non-Send data) or need conditional auto-trait bounds, which Rust doesn't support.

The library already handles both issues with separate types: `RcLazy`/`ArcLazy`, `Thunk`/`SendThunk`, `RcFnBrand`/`ArcFnBrand`.

### Design

**Inner trait.** A new `CoyonedaLowerRef` trait with a `lower_ref(&self)` method (borrow-based, works with any `Deref` wrapper). Separate from `CoyonedaInner` (which keeps `lower(self: Box<Self>)` for the Box variant):

```rust
trait CoyonedaLowerRef<'a, F, A: 'a>: 'a {
    fn lower_ref(&self) -> F::Of<'a, A> where F: Functor;
}
```

**Layer struct.** A single generic `CoyonedaMapLayer<Inner, Func>` is shared across all three variants (Box, Rc, Arc). Different `Inner`/`Func` type parameters yield different trait impls:

```rust
struct CoyonedaMapLayer<Inner, Func> {
    inner: Inner,
    func: Func,
}

// Box variant (issue #1): Inner = Box<dyn CoyonedaInner>, Func = impl Fn
// Rc variant:  Inner = Rc<dyn CoyonedaLowerRef>,  Func = Rc<dyn Fn(B) -> A>
// Arc variant: Inner = Arc<dyn CoyonedaLowerRef + Send + Sync>,
//              Func = Arc<dyn Fn(B) -> A + Send + Sync>
```

**Outer types:**

```rust
pub struct RcCoyoneda<'a, F, A: 'a>(
    Rc<dyn CoyonedaLowerRef<'a, F, A> + 'a>
);
pub struct ArcCoyoneda<'a, F, A: 'a>(
    Arc<dyn CoyonedaLowerRef<'a, F, A> + Send + Sync + 'a>
);
```

**`map` wraps the function in Rc/Arc** so the user's closure does not need `Clone`:

```rust
impl<'a, F, A: 'a> RcCoyoneda<'a, F, A> {
    pub fn map<B: 'a>(self, f: impl Fn(A) -> B + 'a) -> RcCoyoneda<'a, F, B> {
        RcCoyoneda(Rc::new(CoyonedaMapLayer {
            inner: self.0,
            func: Rc::new(f),  // unsizing coercion: Rc<impl Fn> -> Rc<dyn Fn>
        }))
    }
}
```

The unsizing coercion happens naturally at the concrete `Rc::new(layer)` / `Arc::new(layer)` call sites since the pointer type is known.

**`lower_ref` on map layers** clones the inner Rc (cheap refcount bump) and wraps the function clone in a forwarding closure for `F::map`:

```rust
impl CoyonedaLowerRef<'a, F, A> for CoyonedaMapLayer<
    Rc<dyn CoyonedaLowerRef<'a, F, B>>,
    Rc<dyn Fn(B) -> A + 'a>,
> {
    fn lower_ref(&self) -> F::Of<'a, A> where F: Functor {
        let lowered = self.inner.lower_ref();
        let func = self.func.clone();   // Rc clone: refcount bump
        F::map(move |b| (&*func)(b), lowered)
    }
}
```

**Base value cloning.** `lower_ref(&self)` on the base layer must clone `F::Of<'a, A>` to produce an owned value from a borrow. This requires `F::Of<'a, A>: Clone` at `lift` time. This is inherent to shared ownership, the same trade-off as `RcLazy`.

**Brands:**

```rust
pub struct RcCoyonedaBrand<F>(PhantomData<F>);
pub struct ArcCoyonedaBrand<F>(PhantomData<F>);
```

Both implement `Functor` (no `F: Functor` needed), `Pointed` (requires `F: Pointed`), `Foldable` (requires `F: Functor + Foldable`), `Semimonad` (requires `F: Functor + Semimonad`). Additionally, since `RcCoyoneda`/`ArcCoyoneda` are `Clone`, they can implement `Semiapplicative`, `Traversable`, `Extend`, and `Comonad` where the underlying `F` supports them.

### Conversions

- `Coyoneda -> RcCoyoneda`: lower first (requires `F: Functor`), then `RcCoyoneda::lift`. Cost: applies all accumulated maps.
- `RcCoyoneda -> Coyoneda`: `Coyoneda::lift(rc_coyo.lower())`. Same cost.
- `CoyonedaExplicit -> RcCoyoneda`: via `From<CoyonedaExplicit> for Coyoneda`, then `Coyoneda -> RcCoyoneda`. Or directly via `RcCoyoneda::new(explicit.func, explicit.fb)`.

### Allocation profile per `map`

| Variant                     | Allocations per `map` | What is allocated                                                      |
| --------------------------- | --------------------- | ---------------------------------------------------------------------- |
| `Coyoneda` (after issue #1) | 1                     | `Box<CoyonedaMapLayer>` (func stored inline)                           |
| `RcCoyoneda`                | 2                     | `Rc<CoyonedaMapLayer>` + `Rc<dyn Fn>` for the function                 |
| `ArcCoyoneda`               | 2                     | `Arc<CoyonedaMapLayer>` + `Arc<dyn Fn + Send + Sync>` for the function |
| `CoyonedaExplicit`          | 0                     | Nothing (compile-time composition)                                     |
| `BoxedCoyonedaExplicit`     | 1                     | `Box<dyn Fn>` (re-boxed composed function)                             |

---

## 4. Add `traverse` inherent method to `CoyonedaExplicit`

**File:** `coyoneda_explicit.rs`
**Impact:** Feature completeness (matches PureScript)
**Difficulty:** Medium

### Problem

PureScript's `Traversable (Coyoneda f)` composes the traversal function with the accumulated mapping function and traverses `F B` directly:

```purescript
traverse f = unCoyoneda \k -> map liftCoyoneda <<< traverse (f <<< k)
```

`CoyonedaExplicit` can do the same since `B` is visible, but no `traverse` method exists.

### Approach

Add an inherent method:

```rust
pub fn traverse<G: Applicative + 'a, C: 'a + Clone>(
    self,
    f: impl Fn(A) -> G::Of<'a, C> + 'a,
) -> G::Of<'a, CoyonedaExplicit<'a, F, C, C, fn(C) -> C>>
where
    B: Clone,
    F: Traversable,
    F::Of<'a, C>: Clone,
    G::Of<'a, C>: Clone,
    CoyonedaExplicit<'a, F, C, C, fn(C) -> C>: Clone,
{
    G::map(
        |fb| CoyonedaExplicit::lift(fb),
        F::traverse::<'a, B, C, G>(compose(f, self.func), self.fb),
    )
}
```

This requires `F: Traversable` (which implies `F: Functor + Foldable`). Note: PureScript's `Traversable` also extends `Functor + Foldable`, so this matches.

The `F::Of<'a, C>: Clone` and `G::Of<'a, C>: Clone` bounds are inherited from the library's `Traversable` trait definition.

### Trade-offs

- Adds another inherent method, increasing API surface.
- The `Clone` bounds (inherited from the library's `Traversable` trait) may limit usability.
- Cannot be implemented on the brand (`CoyonedaExplicitBrand`) because `Traversable` extends `Functor + Foldable` and the brand's `Traversable` would need the result to be `Clone`.

### Recommendation

Implement as an inherent method. It follows the established pattern of `fold_map` and provides a capability that `Coyoneda` cannot offer (traversal without `F: Functor`).

---

## 5. Address stack overflow risk in `Coyoneda`

**File:** `coyoneda.rs`
**Impact:** Correctness / robustness
**Difficulty:** Medium-High

Detailed feasibility research for each approach is in [feasibility/](feasibility/).

### Problem

`lower` on a `Coyoneda` with k chained maps produces k recursive calls (each `CoyonedaMapLayer::lower` calls `self.inner.lower()`). For large k, this overflows the stack.

### Fundamental constraint

Every approach to linearize the recursive lowering requires type-erasing the hidden intermediate type `B` at each layer. Rust's only safe mechanisms for heterogeneous type erasure are trait objects (which require dyn-compatible, i.e. non-generic, methods) and `Any` (which requires `'static`). Since `Coyoneda<'a, ...>` supports arbitrary lifetimes and the methods that would enable iterative resolution are inherently generic, the recursive structure is an inherent consequence of the trait-object encoding.

### Feasibility matrix

| Approach                                     | Stack-safe? | No unsafe? | No `'static`?  | No new bounds on `F`?           | API compatible? | Verdict                                |
| -------------------------------------------- | ----------- | ---------- | -------------- | ------------------------------- | --------------- | -------------------------------------- |
| A: Iterative lowering (explicit stack)       | Yes         | Yes        | No (`Any`)     | Yes                             | No              | Infeasible                             |
| B: Trampoline / CPS step protocol            | Yes         | Yes        | No (`Any`)     | Yes                             | No              | Infeasible                             |
| E: Flat Vec of erased functions              | Yes         | Yes        | No (`Any`)     | Yes                             | No              | Infeasible                             |
| F: Eager periodic collapse at `map` time     | Bounded     | Yes        | Yes            | No (`F: Functor` on `map`)      | No              | Infeasible (breaks Coyoneda's purpose) |
| F': Opt-in `collapse` method                 | Manual      | Yes        | Yes            | `F: Functor` on `collapse` only | Yes             | Partially feasible (ergonomic helper)  |
| G: CatList of erased functions (Free-style)  | Yes         | Yes        | No (`Any`)     | Yes                             | No              | Infeasible                             |
| H: Adaptive stack growth via `stacker`       | Practical   | Yes        | Yes            | Yes                             | Yes             | **Feasible**                           |
| I: Redesigned inner trait with peel protocol | Depends     | Yes        | No (`Any`)     | Yes                             | No              | Infeasible                             |
| J: `'static`-only variant (`CoyonedaSafe`)   | Yes         | Yes        | No (by design) | Yes                             | Yes (new type)  | Feasible but redundant (see below)     |
| K: Spawn on large-stack thread               | Yes         | Yes        | Partial        | No (`Send`)                     | No              | Infeasible                             |
| L: CPS transform of lowering                 | No (no TCE) | Yes        | Yes            | Yes                             | No (dyn-compat) | Infeasible                             |
| M: Chunked composition at `map` time         | N/A         | Yes        | Yes            | Yes                             | No              | Infeasible                             |

### Why most approaches are infeasible

All approaches labeled infeasible hit the same root cause from different angles:

1. **`Any` requires `'static`.** Iterative approaches (A, B, E, G, I) need to store intermediate `F::Of<'a, B>` values in a type-erased container during iteration. `Box<dyn Any>` is the only safe heterogeneous container, but it requires `'static`, which is incompatible with `Coyoneda<'a, ...>`.

2. **Generic methods break dyn-compatibility.** Step/peel protocols (B, I) and CPS transforms (L) require methods on `CoyonedaInner` that are generic over the output type or continuation type, which cannot appear on trait objects.

3. **No TCE in Rust.** CPS (L) just rearranges where stack frames go without reducing their count.

4. **Composition across existential boundary requires opening the hidden type.** Chunked composition (M) and periodic collapse (F) face the same barrier as map fusion: composing functions requires knowing the hidden type `B`.

### Feasible approaches

**H: `stacker` (adaptive stack growth).** The `stacker` crate provides `stacker::maybe_grow(red_zone, stack_size, closure)`, which checks remaining stack space and, if below threshold, allocates a new stack segment. Applied to `CoyonedaMapLayer::lower`:

```rust
fn lower(self: Box<Self>) -> F::Of<'a, A> where F: Functor {
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        let lowered = self.inner.lower();
        F::map(self.func, lowered)
    })
}
```

Properties:

- No structural changes, no new type parameters, no lifetime restrictions.
- Near-zero overhead when stack is sufficient (one pointer comparison).
- `stacker` is battle-tested (used by `rustc` itself). Its public API is safe Rust.
- Can be gated behind a feature flag (e.g., `stacker`).
- Does not eliminate recursion; makes stack overflow unreachable in practice.

**J: `'static`-only variant (`CoyonedaSafe`) - redundant.** Initially proposed as a separate type following the `Thunk`/`Trampoline` pattern. However, on closer analysis, `CoyonedaSafe` would be strictly worse than `BoxedCoyonedaExplicit` in every dimension:

| Property                | `CoyonedaSafe` (proposed) | `BoxedCoyonedaExplicit` (existing) |
| ----------------------- | ------------------------- | ---------------------------------- |
| Stack-safe lowering     | Yes (iterative)           | Yes (no recursion)                 |
| Map fusion              | No (k `F::map` calls)     | Yes (1 `F::map` call)              |
| Lifetime support        | `'static` only            | `'a` (arbitrary)                   |
| Heap allocation per map | 1 (push closure)          | 1 (re-box composed function)       |
| Deep chain ergonomics   | Uniform type always       | Uniform type via `.boxed()`        |
| `B` hidden              | Yes (via `Any`)           | No (type parameter)                |

The only advantage of `CoyonedaSafe` would be hiding `B`, which is only valuable for full HKT integration. But a `CoyonedaSafe` brand would still have the same limited HKT story as `CoyonedaExplicitBrand` (no `Pointed`, no `Semimonad`), because the `Any`-based downcasting cannot support those operations. So hiding `B` buys nothing practical while costing map fusion and lifetime polymorphism.

**Conclusion:** `CoyonedaSafe` is redundant. `CoyonedaExplicit` with `.boxed()` is the correct recommendation for users who need stack-safe, fused lowering.

**F': Opt-in `collapse` method.** A lightweight ergonomic helper:

```rust
impl<'a, F: Functor, A: 'a> Coyoneda<'a, F, A> {
    pub fn collapse(self) -> Self {
        Coyoneda::lift(self.lower())
    }
}
```

Users building deep chains can call `collapse` periodically to flatten accumulated layers. Requires `F: Functor`. Not automatic stack safety, but simple and useful.

### Recommendation

A three-part strategy:

1. **`stacker` (feature-gated):** Primary defense for `Coyoneda` itself. Automatic, zero-API-change, covers 99% of cases. Gate behind a `stacker` feature flag so users who do not need deep chains avoid the dependency.
2. **`collapse` method on `Coyoneda`:** Lightweight ergonomic helper for manual depth management. Trivial to implement.
3. **Document `CoyonedaExplicit` with `.boxed()` as the recommended alternative** when guaranteed stack-safe, fused lowering is needed. It is already lifetime-polymorphic, stack-safe, and provides single-pass fusion, making a separate `CoyonedaSafe` type redundant.

---

## 6. Add additional type class instances to `CoyonedaBrand`

**File:** `coyoneda.rs`
**Impact:** Feature completeness
**Difficulty:** Low-Medium

### Instances that can be added now (no Clone needed)

**Monad:** Combine existing `Pointed` and `Semimonad`. This is a marker trait in the library.

**Eq / PartialEq / Ord / PartialOrd:** Implement by lowering both values and comparing. Requires `F: Functor` (for lowering) and the underlying type to implement the corresponding trait.

```rust
// Conceptual:
impl<F: Functor + 'static> Eq for CoyonedaBrand<F>
where for<'a, A: 'a + Eq> F::Of<'a, A>: Eq
```

The exact bounds depend on the library's Eq trait pattern. If the library does not have HKT-level Eq, these can be added as inherent methods or `PartialEq` impls on `Coyoneda` directly.

### Instances blocked by Clone

`Semiapplicative`, `Traversable`, `Extend`, `Comonad`, `Traversable1` all require the structure to be cloneable. These are blocked until an `Rc`-wrapped variant is available (see issue #3).

### Recommendation

Add `Monad` as a marker trait impl immediately (trivial). Add `PartialEq`/`Eq` on `Coyoneda` directly if the library has such impls on other types. Defer Clone-dependent instances to after issue #3.

---

## 7. Add `FoldableWithIndex` support to `CoyonedaExplicit`

**File:** `coyoneda_explicit.rs`
**Impact:** Feature completeness
**Difficulty:** Low

### Problem

`CoyonedaExplicit` preserves the structure of `F B`. If `F` implements `FoldableWithIndex`, then `CoyonedaExplicit` can support indexed folding by composing the indexed fold function with the accumulated mapping function.

### Approach

Add an inherent method:

```rust
pub fn fold_map_with_index<FnBrand, I, M>(
    self,
    func: impl Fn(I, A) -> M + 'a,
) -> M
where
    B: Clone,
    M: Monoid + 'a,
    F: FoldableWithIndex<Index = I>,
    FnBrand: CloneableFn + 'a,
{
    let f = self.func;
    F::fold_map_with_index::<FnBrand, B, M>(move |i, b| func(i, f(b)), self.fb)
}
```

### Recommendation

Implement. Follows the same pattern as `fold_map` and is straightforward.

---

## 8. Update comparison table in `coyoneda_explicit.rs` documentation

**File:** `coyoneda_explicit.rs`
**Impact:** Documentation accuracy
**Difficulty:** Low

### Problem

The comparison table says `Coyoneda` has "2 boxes" per map. After fixing issue #1, this would become "1 box". The table should be updated to reflect the current state, and ideally link to the relevant section for context.

Additionally, the table is missing some rows that would help users choose:

- Pointed via brand: `Coyoneda` yes, `CoyonedaExplicit` no.
- Semimonad via brand: `Coyoneda` yes, `CoyonedaExplicit` no.
- B: 'static required for brand: `Coyoneda` no, `CoyonedaExplicit` yes.

### Recommendation

Update the table after implementing issue #1. Add the missing rows.

---

## 9. Consider `CoyonedaExplicit` with `FnOnce` semantics for `lower`

**File:** `coyoneda_explicit.rs`
**Impact:** Generality (allow non-repeatable functions)
**Difficulty:** Low-Medium (design exploration)

### Problem

`CoyonedaExplicit` stores `Func: Fn(B) -> A` because `Functor::map` requires `impl Fn(A) -> B`. This means `FnOnce` closures (e.g., closures that move captured values) cannot be used.

This is not a `CoyonedaExplicit`-specific issue; it is a library-wide design choice. `Fn` is correct because `F::map` may need to call the function multiple times (e.g., `Vec::map` calls it once per element).

### Approaches

**A. Accept the status quo.** `Fn` is the correct bound for `Functor::map`. Users who need `FnOnce` semantics can use the underlying functor directly.

**B. Provide a separate `lower_once` method** that takes ownership of `Func` and requires `Func: FnOnce`. This would only work for single-element containers (Option, Identity) where the function is called at most once. This is too specialized to be useful.

### Recommendation

**A (no change).** The `Fn` bound is correct and consistent with the library's design. This is not a flaw.

---

## Implementation Order

1. **Issue #1** (eliminate double allocation) - immediate, easy, pure performance win.
2. **Issue #2** (replace `bind` with renamed `flat_map`) - immediate, API cleanup.
3. **Issue #8** (update docs) - after issue #1, trivial.
4. **Issue #6** (add Monad marker) - trivial once identified.
5. **Issue #7** (FoldableWithIndex) - low effort, follows existing pattern.
6. **Issue #4** (traverse method) - medium effort, bounds verified.
7. **Issue #5** (stack overflow) - add `collapse` method now; `stacker` feature-gated; document `CoyonedaExplicit` with `.boxed()` as stack-safe alternative.
8. **Issue #3** (RcCoyoneda + ArcCoyoneda) - medium effort, enables many downstream type class instances. Requires `CoyonedaLowerRef` trait, shared generic layer struct with separate impls for Box/Rc/Arc inner types.
9. **Issue #9** (FnOnce) - no change needed.
