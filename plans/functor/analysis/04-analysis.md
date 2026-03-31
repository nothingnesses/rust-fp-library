# Coyoneda Implementations: Flaws, Issues, and Limitations

Analysis of `Coyoneda` and `CoyonedaExplicit` in the `fp-library` crate, focusing on
correctness, performance, ergonomics, and completeness.

Files analyzed:

- `/home/nixos/projects/rust-fp-lib/fp-library/src/types/coyoneda.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/types/coyoneda_explicit.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/classes/functor.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/brands.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/kinds.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/functions.rs`

---

## 1. CoyonedaExplicit: Boxing on Every Map Contradicts "Zero-Cost" Claim

**Location:** `coyoneda_explicit.rs`, lines 167-175 (the `map` method).

**Problem:** The module documentation (line 1) and comparison table (line 22) claim
"zero-cost map fusion" and "0" heap allocations per map. This is false. Every call to
`map` allocates a new `Box<dyn Fn(B) -> C + 'a>`:

```rust
pub fn map<C: 'a>(
    self,
    f: impl Fn(A) -> C + 'a,
) -> CoyonedaExplicit<'a, F, B, C> {
    CoyonedaExplicit {
        fb: self.fb,
        func: Box::new(compose(f, self.func)),  // <-- heap allocation
    }
}
```

The `compose` call itself is zero-cost (returns `impl Fn`), but wrapping the result in
`Box::new(...)` allocates on every map. The `lift` method (line 434-438) also boxes the
identity function. So a chain of k maps produces k+1 heap allocations total (one for
lift, one per map).

The documentation table claims `CoyonedaExplicit` has "0" heap allocations per map vs
Coyoneda's "2 boxes". In reality, `CoyonedaExplicit` allocates 1 box per map, while
`Coyoneda` allocates 2 boxes per map (one for the inner trait object, one for the
function). The relative improvement is real but the absolute claim of zero is wrong.

**Why boxing exists here:** The `func` field is typed as `Box<dyn Fn(B) -> A + 'a>`.
This is necessary because `map` changes the output type from `A` to `C`, and without
boxing, the composed function's concrete type would grow with each map (nested `impl Fn`
layers), which cannot be stored in a struct field of fixed type.

### Approaches

**Approach A: Keep `Box<dyn Fn>`, fix the documentation.**
Change "zero-cost" to "single-pass" or "fused." Update the table to say "1 box" per map
instead of "0." This is honest and costs nothing.

Trade-offs: No code change needed. Users get accurate expectations. The fundamental
benefit (single `F::map` call at lower time) is preserved and correctly documented.

**Approach B: Use a generic function type parameter to avoid boxing entirely.**
Make the composed function a type parameter of the struct:

```rust
pub struct CoyonedaExplicit<'a, F, B, A, Func>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    Func: Fn(B) -> A + 'a,
{
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: Func,
}
```

Each `map` returns a `CoyonedaExplicit` with a different `Func` type. True zero-cost
with no boxing and no dynamic dispatch.

Trade-offs: The `Func` type parameter leaks into every signature. Type errors become
much harder to read because `Func` is a deeply nested `impl Fn(impl Fn(...))` type.
Methods that need to store or return `CoyonedaExplicit` values in containers become
impossible without boxing or type erasure. The `lift` method can no longer just return
`CoyonedaExplicit<F, A, A>` because `Func` would differ. Builder-pattern chaining
still works, but the type grows with each step.

**Approach C: Offer both variants.**
Keep the current boxed version as-is (with corrected docs) and add a second generic
variant (perhaps `CoyonedaZeroCost`) that uses a type parameter for the function. Users
pick the variant based on whether they need uniform types or zero allocation.

Trade-offs: Two similar types may confuse users. More maintenance surface. But it gives
maximum flexibility.

**Recommendation:** Approach A. The boxing cost is negligible compared to the real win
(fusing k `F::map` calls into 1). Documenting it honestly is sufficient. If benchmarks
later show the boxing matters, Approach C can be added without breaking changes.

---

## 2. CoyonedaExplicit Uses `Fn` Instead of `FnOnce`

**Location:** `coyoneda_explicit.rs`, lines 96, 130-131, 167-169.

**Problem:** The `func` field is `Box<dyn Fn(B) -> A + 'a>`, and both `new` and `map`
accept `impl Fn(B) -> A + 'a`. The `lower` method (line 199-203) calls `F::map(self.func,
self.fb)`, and `Functor::map` also takes `impl Fn(A) -> B + 'a`.

Using `Fn` instead of `FnOnce` means:

1. Functions passed to `map` must be callable multiple times, even when the structure
   contains at most one element (e.g., `Option`). This prevents using closures that
   consume captured values.
2. This is a fundamental constraint that flows from `Functor::map`'s signature
   (`functor.rs`, line 121-124), which takes `impl Fn(A) -> B + 'a`. For containers
   like `Vec`, the function genuinely must be callable multiple times.

This is not a bug per se; it is a consequence of the library's `Functor` design. But
it does mean `CoyonedaExplicit` cannot be used with move-only transformations, even in
contexts (like `Option`) where the function would only be called once.

### Approaches

**Approach A: Accept the status quo.**
`Fn` is the correct bound for a generic `Functor::map` that must work for multi-element
containers. Document the limitation.

**Approach B: Provide a separate `map_once` that takes `FnOnce` and immediately lowers.**
This would only work when `F: Functor`, defeating the deferred-fusion purpose.

**Recommendation:** Approach A. This is an inherent trade-off of the library's `Functor`
design and not specific to Coyoneda. Changing it would require a fundamental redesign of
the `Functor` trait.

---

## 3. Coyoneda: No Map Fusion (k Calls to F::map)

**Location:** `coyoneda.rs`, lines 244-249 (`CoyonedaMapLayer::lower`), lines 394-402
(`map` method).

**Problem:** Each `map` wraps the previous value in a new `CoyonedaMapLayer` containing a
boxed trait object and a boxed function. At `lower` time, the layers unwind recursively:
each layer lowers its inner value (calling `F::map`), then applies its own function via
another `F::map`. For k chained maps on a `Vec` of n elements, this produces k full
traversals of the vector, totaling O(k \* n) work.

The documentation (lines 38-44) correctly describes this limitation. The root cause is
that `CoyonedaInner` is a trait object, and composing functions across the existential
boundary would require a generic method, which is not dyn-compatible.

This means `Coyoneda` provides no performance benefit over directly chaining `F::map`
calls. Its only value is providing a `Functor` instance for non-`Functor` types, and
even that is limited since `lower` still requires `F: Functor`.

### Approaches

**Approach A: Accept the limitation; direct users to `CoyonedaExplicit` for fusion.**
This is the current strategy.

**Approach B: Use an enum-based approach instead of trait objects.**
Replace the `dyn CoyonedaInner` trait object with an enum that stores the base value
and a single composed function (similar to `CoyonedaExplicit` but with the intermediate
type existentially hidden via boxing). The key insight is that you only need to box
the _composed_ function, not every intermediate layer.

This would look roughly like:

```rust
pub struct Coyoneda<'a, F, A: 'a> {
    inner: Box<dyn ErasedCoyoneda<'a, F, A>>,
}

trait ErasedCoyoneda<'a, F, A> {
    fn lower(self: Box<Self>) -> F::Of<'a, A> where F: Functor;
}

struct CoyonedaRepr<'a, F, B, A> {
    fb: F::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
```

When `map` is called, instead of adding a new layer, it composes the new function with
the existing `func` and re-wraps in a single `CoyonedaRepr`. This is essentially what
`CoyonedaExplicit` does, but with the `B` hidden behind a trait object so it can still
participate in HKT.

Trade-offs: The `map` method must consume and reconstruct the inner representation,
requiring an additional method on the erased trait: `fn map_erased(self: Box<Self>,
f: ???)`. The problem is that this method needs `f: Box<dyn Fn(A) -> C>` for some
unknown `C`, making it generic and thus not dyn-compatible. This is the exact same
problem the current design faces. There is no clean way around it without either:

- Giving up the existential (i.e., becoming `CoyonedaExplicit`), or
- Using unsafe code to perform type erasure on the composed function.

**Approach C: Deprecate `Coyoneda` in favor of `CoyonedaExplicit` + `into_coyoneda`.**
Since `Coyoneda` provides no fusion and `CoyonedaExplicit` does, users should always
build with `CoyonedaExplicit` and convert via `into_coyoneda` only when HKT is needed.
Make this the recommended pattern in documentation.

**Recommendation:** Approach C. The `Coyoneda` type should remain for HKT integration,
but documentation should strongly guide users toward `CoyonedaExplicit` for building
pipelines. The `into_coyoneda` bridge already exists (line 314-316). Consider adding a
`from_coyoneda` method in the other direction (which would require `F: Functor` to
lower and re-lift, losing any deferred maps).

---

## 4. Coyoneda: Stack Overflow Risk with Deep Nesting

**Location:** `coyoneda.rs`, lines 244-249 (`CoyonedaMapLayer::lower`).

**Problem:** `lower` calls `self.inner.lower()` recursively. For k chained maps, this
creates k frames on the call stack before unwinding. The `many_chained_maps` test
(line 672-678) uses 100 layers, which is fine, but thousands of layers would overflow.

`CoyonedaExplicit` does not have this problem (its `lower` makes a single `F::map`
call), which is correctly noted in the comparison table.

### Approaches

**Approach A: Document the stack depth limitation explicitly.**
Note in the doc that chains beyond a few thousand maps may overflow.

**Approach B: Use a trampoline / iterative loop for lowering.**
Replace the recursive `lower` with an iterative approach. This is difficult because each
layer has a different type (the trait object hides it), and you cannot collect them into
a `Vec` without a common type. An approach would be to collect all layers' functions
into a `Vec<Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>>` and apply them in a loop,
but the type erasure overhead and `Any` downcasting would be expensive and fragile.

**Approach C: Guide users to `CoyonedaExplicit` for deep chains.**
Since `CoyonedaExplicit` composes into a single function, it is naturally stack-safe.

**Recommendation:** Approach A + C. Document the limitation and direct users to
`CoyonedaExplicit`. Trying to make `Coyoneda` stack-safe would add significant
complexity for minimal benefit, given that the type already does not fuse maps.

---

## 5. CoyonedaExplicit: `apply` and `bind` Defeat Fusion

**Location:** `coyoneda_explicit.rs`, lines 358-366 (`apply`), lines 395-402 (`bind`).

**Problem:** Both `apply` and `bind` call `lower()` on their inputs, which forces all
accumulated maps to be applied via `F::map`. They then re-lift the result into a fresh
`CoyonedaExplicit` with the identity function:

```rust
pub fn apply<...>(ff: ..., fa: Self) -> CoyonedaExplicit<'a, F, C, C>
where ...
{
    CoyonedaExplicit::lift(F::apply::<FnBrand, A, C>(ff.lower(), fa.lower()))
}
```

This means any maps accumulated before `apply` or `bind` are flushed into a full
`F::map` traversal. The fusion pipeline is "reset" after each `apply`/`bind`. This is
documented in the method comments ("After the operation the fusion pipeline is reset")
but has a subtle consequence: in an applicative or monadic pipeline interleaved with
maps, users get no fusion benefit. Only consecutive chains of pure `map` calls benefit.

### Approaches

**Approach A: Accept and document clearly.**
Monadic bind inherently requires evaluating the structure (you need the value of `A` to
produce `F C`). Applicative apply similarly needs the concrete `F`-wrapped function.
There is no way around this; fusion only applies to pure functor maps.

**Approach B: Add a `map_then_bind` combinator that composes the pre-bind function
with the bind callback.**
Instead of lowering, compose the accumulated function with the bind callback:

```rust
pub fn map_then_bind<C: 'a>(
    self,
    f: impl Fn(A) -> CoyonedaExplicit<'a, F, C, C> + 'a,
) -> CoyonedaExplicit<'a, F, C, C>
where
    F: Functor + Semimonad,
{
    let composed = compose(f, self.func);
    CoyonedaExplicit::lift(F::bind(self.fb, move |b| composed(b).lower()))
}
```

This avoids the intermediate `F::map` for the accumulated function; instead, the
composed function is applied inside the bind callback. Only one `F::map` equivalent
(the bind itself) runs.

Trade-offs: Adds API surface. The semantics are identical to `map(...).bind(...)` but
with better performance. Users must know to use this variant.

**Recommendation:** Approach B for `bind`, Approach A for `apply`. The `bind`
optimization is straightforward and meaningful. For `apply`, the function is inside an
`F` context, so there is no way to compose without extracting it first (which requires
lowering).

---

## 6. CoyonedaExplicit: `fold_map` Requires `B: Clone`

**Location:** `coyoneda_explicit.rs`, lines 281-291.

**Problem:** The `fold_map` method requires `B: Clone`:

```rust
pub fn fold_map<FnBrand, M>(self, func: impl Fn(A) -> M + 'a) -> M
where
    B: Clone,
    ...
```

This `Clone` bound comes from the `Foldable::fold_map` signature on the underlying
functor, which requires `A: Clone` (seen in `foldable.rs` line 572:
`fn fold_map<'a, FnBrand, A: 'a + Clone, M>`). Since `CoyonedaExplicit` folds over
`F::Of<'a, B>`, the clone bound applies to `B`.

For types like `Vec<String>` where `B = String`, this is fine. But for `Vec<File>` or
other non-Clone types, `fold_map` cannot be used even though folding conceptually
should not require cloning.

### Approaches

**Approach A: Accept as an upstream limitation.**
The `Clone` bound exists on `Foldable::fold_map` because the `CloneableFn` mechanism
needs to clone the mapping function. Fixing this would require changing the `Foldable`
trait itself.

**Approach B: Add a `fold_map_consuming` variant that uses `FnOnce` semantics.**
This would only work for single-element structures and would break the generic
`Foldable` abstraction.

**Recommendation:** Approach A. The `Clone` bound is inherited from `Foldable` and is
not specific to `CoyonedaExplicit`. Fixing it requires a broader redesign of `Foldable`.

---

## 7. Missing Type Class Instances on `CoyonedaBrand`

**Location:** `coyoneda.rs`, lines 454-581.

**Problem:** `CoyonedaBrand<F>` only implements `Functor`, `Pointed`, and `Foldable`
(with the `Functor` constraint on `Foldable`). The module documentation (lines 70-73)
lists many missing instances: `Apply`, `Applicative`, `Bind`, `Monad`, `Traversable`,
`Extend`, `Comonad`, `Eq`, `Ord`.

The reason given is that `Coyoneda` is not `Clone` (line 64-68), which prevents
`Semiapplicative` (needs `Clone` for the function container) and `Traversable` (needs
`Clone` for the values). The lack of `Clone` stems from `Box<dyn CoyonedaInner>` being
non-cloneable.

`CoyonedaExplicit` partially addresses this by providing `apply`, `bind`, and
`fold_map` as inherent methods. But these are not type class instances; they cannot be
used in generic code that requires `Semiapplicative` or `Semimonad` bounds.

### Approaches

**Approach A: Implement `Semimonad` for `CoyonedaBrand<F>` where `F: Semimonad +
Functor`.**
This would lower, bind via `F::bind`, and re-lift. It does not require `Clone`.

```rust
impl<F: Functor + Semimonad + 'static> Semimonad for CoyonedaBrand<F> {
    fn bind<'a, A: 'a, B: 'a>(
        fa: Coyoneda<'a, F, A>,
        f: impl Fn(A) -> Coyoneda<'a, F, B> + 'a,
    ) -> Coyoneda<'a, F, B> {
        Coyoneda::lift(F::bind(fa.lower(), move |a| f(a).lower()))
    }
}
```

Trade-offs: Requires `F: Functor` for lowering, which is more restrictive than
PureScript's version. But this is consistent with the existing `Foldable`
implementation.

**Approach B: Use `Rc<dyn CoyonedaInner>` instead of `Box` for cloneability.**
This would enable `Clone` on `Coyoneda`, unlocking `Semiapplicative` and `Traversable`.

Trade-offs: Adds reference-counting overhead. Makes `Coyoneda` non-`Send` unless
`Arc` is used. Complicates the ownership model since `lower` currently takes
`self: Box<Self>`.

**Approach C: Add `Semimonad` (Approach A) now, defer `Semiapplicative`/`Traversable`
to a future `Rc`/`Arc`-wrapped variant.**

**Recommendation:** Approach C. Adding `Semimonad` (and by extension `Monad` if
`Pointed` is already implemented) is straightforward and useful. The
`Semiapplicative`/`Traversable` instances require `Clone`, which needs a bigger design
change.

---

## 8. CoyonedaExplicit: No HKT Integration

**Location:** `coyoneda_explicit.rs`, lines 18-19 (comparison table).

**Problem:** `CoyonedaExplicit` has no brand type and does not implement any type class
traits (`Functor`, `Foldable`, etc.). This means it cannot be used in generic code that
is parameterized over a `Functor` brand.

The `into_coyoneda` method (lines 314-316) serves as a bridge, but it requires boxing
the accumulated function into a `Coyoneda`, which then cannot fuse maps.

### Approaches

**Approach A: Accept as a fundamental trade-off.**
The intermediate type `B` is visible, which prevents defining a brand
`CoyonedaExplicitBrand<F>` that maps `A -> CoyonedaExplicit<F, ?, A>` (the `?` is
the hidden `B`). This is the same existential-quantification problem in a different
form.

**Approach B: Provide a brand that fixes `B = A` (i.e., the identity-function case).**
A `CoyonedaExplicitBrand<F>` where `Of<'a, A> = CoyonedaExplicit<'a, F, A, A>`. The
`Functor` instance would call the inherent `map`, but the resulting type would be
`CoyonedaExplicit<'a, F, A, B>`, which does not match `Of<'a, B> =
CoyonedaExplicit<'a, F, B, B>` (note `B != A` for the first type parameter).

This does not work: after one `map`, the type is no longer `Self::Of<'a, B>`.

**Approach C: Define a wrapper that type-erases `B` back to `A` after each map.**
This would require lowering (calling `F::map`) after each map to reset `B`, defeating
the purpose entirely.

**Recommendation:** Approach A. This is a fundamental tension between type-level fusion
and HKT integration. The `into_coyoneda` bridge is the correct design: build the
fusion pipeline with `CoyonedaExplicit`, then convert when HKT is needed. Document
this pattern prominently.

---

## 9. Coyoneda: `hoist` Requires `F: Functor` Unnecessarily

**Location:** `coyoneda.rs`, lines 443-450.

**Problem:** `hoist` lowers the entire `Coyoneda` (applying all accumulated maps via
k calls to `F::map`), then applies the natural transformation, then re-lifts. This
requires `F: Functor` and defeats any deferred computation.

PureScript's `hoistCoyoneda` applies the natural transformation directly to the hidden
`F B` without lowering. This is possible because PureScript can "open" the existential
via `unCoyoneda`. Rust cannot because of dyn-compatibility constraints.

`CoyonedaExplicit::hoist` (line 241-249) correctly applies the nat-trans to the stored
`fb` directly, without lowering, and without requiring `F: Functor`.

### Approaches

**Approach A: Add a `hoist_inner` method to `CoyonedaInner`.**
This would require the method to be dyn-compatible. The signature would be:

```rust
fn hoist<G>(self: Box<Self>, nat: &dyn NaturalTransformation<F, G>)
    -> Box<dyn CoyonedaInner<'a, G, A>>
```

But `G` is a type parameter, making this generic and not dyn-compatible.

**Approach B: Use a visitor/callback pattern.**
Instead of a generic method, add a method that accepts a function pointer or trait
object for the transformation:

```rust
fn hoist_erased(
    self: Box<Self>,
    nat: &dyn Fn(/* ??? */) -> /* ??? */,
) -> Box<dyn CoyonedaInner<'a, G, A>>
```

The problem is that the function must transform `F::Of<'a, B>` to `G::Of<'a, B>` for
the hidden type `B`, and we cannot express this without generics.

**Approach C: Accept the limitation; guide users to `CoyonedaExplicit` for efficient
hoist.**

**Recommendation:** Approach C. The dyn-compatibility issue is fundamental. Users who
need efficient `hoist` should use `CoyonedaExplicit`.

---

## 10. Thread Safety: Neither Type is Send or Sync

**Location:** `coyoneda.rs` line 269 (`Box<dyn CoyonedaInner>`), `coyoneda_explicit.rs`
line 96 (`Box<dyn Fn(B) -> A + 'a>`).

**Problem:** Both types use `Box<dyn Trait>` without `Send`/`Sync` bounds. This means
neither `Coyoneda` nor `CoyonedaExplicit` can be sent across threads. For
`CoyonedaExplicit`, this also means it cannot be used with `ArcFnBrand` or any
thread-safe operation.

The library has a pattern of providing `Send` variants (e.g., `SendThunk`, `ArcLazy`).
No such variants exist for the Coyoneda types.

### Approaches

**Approach A: Add `Send` bound variants.**
Create `SendCoyonedaExplicit` with `Box<dyn Fn(B) -> A + Send + 'a>`, or parameterize
over a pointer/bound brand.

Trade-offs: Doubles the API surface. But this follows the established library pattern
(`Thunk`/`SendThunk`, `RcLazy`/`ArcLazy`).

**Approach B: Parameterize over a "send-ness" marker.**
Use a trait like `MaybeSend` to conditionally add the `Send` bound. This avoids type
duplication but adds complexity.

**Approach C: Make the default `Send`.**
Use `Box<dyn Fn(B) -> A + Send + 'a>` by default. Non-`Send` closures are rare in
practice.

Trade-offs: Restricts what closures can be used. Users who capture `Rc` values in their
map functions would be unable to use the type.

**Recommendation:** Approach A for consistency with the rest of the library. Add
`SendCoyonedaExplicit` as a separate type. `Coyoneda` is less urgent since it is
primarily used for HKT integration where `Send` is less often needed.

---

## 11. CoyonedaExplicit: `new` and `lift` Box the Identity Function

**Location:** `coyoneda_explicit.rs`, lines 130-138 (`new`), lines 434-438 (`lift`).

**Problem:** `lift` creates a `CoyonedaExplicit` with `func: Box::new(identity)`. This
boxes the identity function, which is a zero-sized function item. The box allocates
a (small) heap block for no functional reason. When immediately followed by `map`, the
identity is composed away, but a box was still allocated.

Similarly, `new` takes `impl Fn(B) -> A + 'a` and immediately boxes it.

### Approaches

**Approach A: Use an `Option<Box<dyn Fn>>` or enum to represent "no function yet."**
Store `None` for the identity case and only box when a real function is provided.

Trade-offs: Adds a branch at `lower` time. Slightly more complex internal
representation.

**Approach B: Accept the allocation.**
A box of a zero-sized type allocates no actual memory (the allocator returns a
dangling pointer for zero-sized allocations in Rust). So `Box::new(identity)` is
effectively free.

**Approach C: Use the generic-function-type approach (see Issue 1, Approach B).**
If the function is a type parameter, `identity` is stored inline with zero allocation.

**Recommendation:** Approach B. Boxing a ZST like `identity` (a function item, not a
closure) is a no-op in practice. The allocator returns a non-null dangling pointer for
zero-sized types. This is a non-issue.

However, note that `Box<dyn Fn(B) -> A + 'a>` is _not_ zero-sized; the trait object
has a vtable pointer. The `Box` allocates space for the vtable dispatch, which is
typically pointer-sized. This is still negligible but worth being precise about in
documentation.

---

## 12. Coyoneda: `Foldable` Requires `F: Functor` (PureScript Does Not)

**Location:** `coyoneda.rs`, lines 533-581.

**Problem:** The `Foldable` instance for `CoyonedaBrand<F>` requires `F: Functor +
Foldable + 'static`. It works by lowering (which requires `Functor`), then folding.
PureScript's version only requires `Foldable f` because it opens the existential to
compose the fold function with the accumulated mapping function.

`CoyonedaExplicit::fold_map` (line 281-291) correctly avoids the `Functor` requirement
by composing directly.

### Approaches

**Approach A: Add a `fold_map_inner` method to `CoyonedaInner`.**
This would need to be generic over the monoid type `M`, making it not dyn-compatible.

**Approach B: Use a different erasure strategy for folding.**
Instead of erasing via `dyn CoyonedaInner`, store the fold-composition capability as
part of the erased interface. For example, add:

```rust
fn fold_map_string(&self, f: &dyn Fn(&dyn Any) -> String) -> String;
```

This is not general and would need one method per monoid type.

**Approach C: Accept the limitation; `CoyonedaExplicit` fills the gap.**

**Recommendation:** Approach C. The dyn-compatibility constraint is fundamental. Users
who need `Foldable` without `Functor` should use `CoyonedaExplicit::fold_map`.

---

## 13. CoyonedaExplicit: `into_coyoneda` Loses Fusion

**Location:** `coyoneda_explicit.rs`, lines 314-316.

**Problem:** `into_coyoneda` converts to `Coyoneda` by calling `Coyoneda::new(self.func,
self.fb)`. This wraps the accumulated `Box<dyn Fn(B) -> A>` inside a `CoyonedaMapLayer`,
creating a single-layer `Coyoneda`. Any further `map` calls on the resulting `Coyoneda`
add new layers, each requiring a separate `F::map` at lower time.

So the pattern `explicit.map(f).map(g).into_coyoneda().map(h).lower()` results in 2
calls to `F::map`: one for the fused `f . g` from the explicit phase, and one for `h`
from the Coyoneda phase. This is better than 3 calls but not ideal.

### Approaches

**Approach A: Document the semantics clearly.**
Users should call `into_coyoneda` last, after all maps.

**Approach B: Add `lower_then_lift` as an alternative to `into_coyoneda`.**
This would lower via `F::map` (1 call) and then lift the result into a base
`Coyoneda` with no map layers. Further maps on the Coyoneda would then be the only
layers.

Trade-offs: Forces evaluation at conversion time. But after conversion, the Coyoneda
starts clean.

Wait; `into_coyoneda` already produces a single-layer `Coyoneda`. The "fusion" from
the explicit phase is preserved in the composed function. The issue is only that
further Coyoneda maps add new layers.

**Recommendation:** Approach A. Document that `into_coyoneda` should be the last step
before passing to HKT-generic code. Any maps after conversion use Coyoneda's
non-fusing behavior.

---

## 14. CoyonedaExplicit: `apply` Has Confusing Type Signature

**Location:** `coyoneda_explicit.rs`, lines 358-366.

**Problem:** The `apply` method has a complex signature:

```rust
pub fn apply<FnBrand: CloneableFn + 'a, Bf: 'a, C: 'a>(
    ff: CoyonedaExplicit<'a, F, Bf, <FnBrand as CloneableFn>::Of<'a, A, C>>,
    fa: Self,
) -> CoyonedaExplicit<'a, F, C, C>
```

The type `<FnBrand as CloneableFn>::Of<'a, A, C>` is the cloneable function wrapper
type. The `Bf` parameter is the hidden intermediate type of the function container.
Users must provide explicit type parameters (`CoyonedaExplicit::apply::<RcFnBrand, _,
_>(ff, fa)`), and the relationship between the types is not immediately clear.

This mirrors the complexity of the library's `Semiapplicative::apply` design, so it is
not unique to `CoyonedaExplicit`. But as an inherent method (not a type class), it
could potentially offer a simpler API.

### Approaches

**Approach A: Accept the complexity.**
It matches the library's conventions.

**Approach B: Provide a convenience wrapper that defaults `FnBrand` to `RcFnBrand`.**
Add `apply_rc` and `apply_arc` methods that fix the function brand.

Trade-offs: More methods, but much simpler call sites. `fa.apply_rc(ff)` vs
`CoyonedaExplicit::apply::<RcFnBrand, _, _>(ff, fa)`.

**Recommendation:** Approach B. The `apply_rc`/`apply_arc` convenience methods would
significantly improve ergonomics for the common case.

---

## 15. No Benchmarks Comparing Coyoneda vs CoyonedaExplicit vs Direct Map Chains

**Problem:** The crate has a benchmarking infrastructure (`just bench`), but there are
no benchmarks that measure the actual performance difference between:

1. Direct chained `F::map` calls.
2. `Coyoneda` with k maps then `lower`.
3. `CoyonedaExplicit` with k maps then `lower`.
4. Manual `compose` then single `F::map`.

Without benchmarks, the claims about performance characteristics are unverified.

### Approaches

**Approach A: Add criterion benchmarks comparing the four approaches.**
Use `Vec` with varying sizes (10, 1000, 100000) and varying chain depths (1, 10, 100).
Measure allocation count and wall-clock time.

**Recommendation:** Approach A. Benchmarks are essential for a library that makes
performance claims. They would also validate or refute the "zero-cost" documentation.

---

## Summary Table

| #   | Issue                                                    | Severity | Recommendation                             |
| --- | -------------------------------------------------------- | -------- | ------------------------------------------ |
| 1   | CoyonedaExplicit boxes per map despite "zero-cost" claim | Medium   | Fix documentation                          |
| 2   | `Fn` instead of `FnOnce`                                 | Low      | Accept (inherent to Functor design)        |
| 3   | Coyoneda has no map fusion                               | High     | Guide users to CoyonedaExplicit            |
| 4   | Coyoneda stack overflow on deep chains                   | Medium   | Document + guide to CoyonedaExplicit       |
| 5   | apply/bind defeat fusion in CoyonedaExplicit             | Medium   | Add map_then_bind combinator               |
| 6   | fold_map requires B: Clone                               | Low      | Accept (upstream Foldable constraint)      |
| 7   | Missing type class instances on CoyonedaBrand            | Medium   | Add Semimonad; defer rest                  |
| 8   | CoyonedaExplicit has no HKT integration                  | Low      | Accept (fundamental trade-off)             |
| 9   | Coyoneda hoist requires F: Functor                       | Medium   | Guide to CoyonedaExplicit                  |
| 10  | Neither type is Send/Sync                                | Medium   | Add Send variants                          |
| 11  | lift boxes the identity function                         | Low      | Non-issue (ZST optimization)               |
| 12  | Coyoneda Foldable requires Functor                       | Medium   | Guide to CoyonedaExplicit                  |
| 13  | into_coyoneda loses fusion for subsequent maps           | Low      | Document ordering                          |
| 14  | apply has confusing type signature                       | Low      | Add apply_rc/apply_arc convenience methods |
| 15  | No benchmarks                                            | Medium   | Add criterion benchmarks                   |
