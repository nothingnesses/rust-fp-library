# Coyoneda Implementations

The library provides two Coyoneda designs: the `Store`-parameterised
`Coyoneda<'a, F, A, Store>`, whose one definition covers the owning `Box`
store (the default), the cloneable `Rc` store, and the thread-safe `Arc`
store; and `CoyonedaExplicit`, a type-level fusion variant. Each makes
different trade-offs around ownership, cloning, thread safety, HKT
integration, and map fusion.

**User story:** "I want to chain maps without calling `F::map` until later."
Useful for map fusion, lazy mapping, and getting a `Functor` instance for
any type constructor for free.

All of them implement the same core idea: wrap a functor value `F B`
together with a deferred function `B -> A`, delaying the call to `F::map`
until `lower` time. This lets you chain `map` calls in O(1) each,
regardless of whether `F` is a `Functor`.

## Quick Reference

|                 | `Coyoneda` (Box, default) | `Coyoneda` (Rc)                   | `Coyoneda` (Arc)                                      | `CoyonedaExplicit`    |
| --------------- | ------------------------- | --------------------------------- | ----------------------------------------------------- | --------------------- |
| Store           | `Box`                     | `Rc`                              | `Arc`                                                 | None (type-level)     |
| Lower           | `lower(self)`             | `lower_ref(&self)`                | `lower_ref(&self)`                                    | `lower(self)`         |
| `lift` bound    | None                      | `F::Of<'a, A>: Clone`             | `F::Of<'a, A>: Clone + Send + Sync`, `A: Send + Sync` | None                  |
| Clone           | No                        | Yes, O(1)                         | Yes, O(1)                                             | No                    |
| Send + Sync     | No                        | No                                | Yes                                                   | Conditional           |
| Heap per map    | 1 Box                     | 2 Rc                              | 2 Arc                                                 | 0                     |
| Map fusion      | No (k calls)              | No (k calls)                      | No (k calls)                                          | Yes (1 call)          |
| Stack safe      | No                        | No                                | No                                                    | Yes                   |
| Brand instances | Full (see below)          | `Functor`, `Foldable`, `WrapDrop` | `SendFunctor`, `SendFoldable`, `WrapDrop`             | `Functor`, `Foldable` |

## Coyoneda (one type, three stores)

**File:** `fp-library/src/types/coyoneda.rs` (the store machinery lives in
`fp-library/src/types/coyo_store.rs`)
**Brand:** `CoyonedaBrand<F, Store = BoxBrand>`

One struct, `Coyoneda<'a, F, A, Store = BoxBrand>`, whose single field is
the store's pointer to the layer trait object,
`<Store as CoyoStore>::Ptr<'a, F, A>`. The `CoyoStore` trait supplies that
pointer type per store (`Box<dyn CoyonedaInner>` for `Box`,
`Rc<dyn RcCoyonedaLowerRef>` for `Rc`,
`Arc<dyn ArcCoyonedaLowerRef>`, whose trait carries `Send + Sync`
supertraits, for `Arc`), and the sibling
`CoyoLift` trait supplies construction, so `Coyoneda::lift` is one
definition across all three stores, each store carrying its own bound (the
table above). Functions are stored inline in each layer for the `Box`
store, and behind a reference-counted `dyn Fn` for the `Rc` and `Arc`
stores; lowering and mapping are per-store inherent methods, because each
store's receiver and bounds differ.

### The Box store (owning, default)

The baseline form; the trailing `Store` default means plain
`Coyoneda<'a, F, A>` is this arm. Consuming `lower(self)` applies the
accumulated maps via `F::map`.

**When to use:** General-purpose deferred mapping when you do not need
cloning or thread safety, and want full HKT type class coverage.

**Allocation:** `lift` allocates 1 Box. Each `map` allocates 1 Box (the
layer; the function is stored inline).

**Surface:** `lift`, `new`, `lower`, `map`, `collapse`, `hoist`, and the
`From` conversion into `CoyonedaExplicit`.

**HKT brand instances:** `Functor`, `Pointed`, `Foldable`, `Lift`,
`ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, and
`WrapDrop`. This is the only arm with full type class coverage because the
one-shot `Box<dyn CoyonedaInner>` cell imposes no `Clone` or `Send`
requirements.

**Limitations:**

- Not cloneable (`Box<dyn>` is not `Clone`).
- Not `Send`/`Sync`.
- Each chained `map` adds a layer of recursion to `lower`. Deep chains
  (thousands of maps) can overflow the stack. Mitigations: `stacker`
  feature (automatic stack growth), `collapse()` (manual flattening), or
  switching to `CoyonedaExplicit`.
- No `unCoyoneda`-style rank-2 eliminator (Rust lacks rank-2 types), so
  `hoist` and `Foldable` require `F: Functor`.

### The Rc store (cloneable)

`Coyoneda<'a, F, A, RcBrand>` wraps layers in `Rc`, making the structure
cheaply cloneable (`Clone` is an O(1) refcount bump). Uses
`lower_ref(&self)`, which clones the base value internally, allowing
repeated lowering without consuming.

**When to use:** When you need to share or reuse a Coyoneda value (e.g.,
lowering it multiple times, storing it in a data structure) and do not
need thread safety.

**Allocation:** `lift` allocates 1 Rc. Each `map` allocates 2 Rc (one for
the layer trait object, one for the `Rc<dyn Fn>` function wrapper).

**Surface:** `lift`, `lower_ref`, `map`, `collapse`, and `Clone`.

**HKT brand instances:** `Functor`, `Foldable`, and `WrapDrop`. `Pointed`,
`Lift`, `Semiapplicative`, and `Semimonad` are not implementable at the
brand level because constructing a value at this store requires
`F::Of<'a, A>: Clone`, a bound that cannot be expressed in those trait
method signatures.

**Limitations:**

- Not `Send`/`Sync` (`Rc` is single-threaded).
- Same stack safety concerns as the Box store. Use `stacker` or
  `collapse()`.
- `lower_ref` clones the base functor value on every call.

### The Arc store (thread-safe)

`Coyoneda<'a, F, A, ArcBrand>` (with `A: Send + Sync`) wraps layers in
`Arc` with `Send + Sync` requirements on functions and inner layers. The
inner machinery uses associated type bounds on the `Kind` trait
(`Kind<Of<'a, A>: Send + Sync>`) to let the compiler auto-derive
`Send`/`Sync` without `unsafe`.

**When to use:** When you need to share a Coyoneda value across threads.

**Allocation:** `lift` allocates 1 Arc. Each `map` allocates 2 Arc (one
for the layer trait object, one for the `Arc<dyn Fn + Send + Sync>`
function wrapper).

**Surface:** `lift`, `lower_ref`, `map`, `collapse`, and `Clone`; `map`
requires `B: Send + Sync` on the target type.

**HKT brand instances:** `SendFunctor`, `SendFoldable`, and `WrapDrop`;
the mapping and folding coverage is the `Send`-aware hierarchy. The plain
`Functor::map` signature lacks
`Send + Sync` bounds on its closure parameter, so closures passed through
it cannot be stored in Arc-wrapped layers; `SendFunctor` carries those
bounds. `Pointed`, `Lift`, `Semiapplicative`, and `Semimonad` are blocked
by the `Clone + Send + Sync` construction bound on `F::Of`, as at the Rc
store.

**Limitations:**

- Atomic reference counting overhead versus the Rc store.
- `map` requires closures (and the target type) to be `Send + Sync`.
- `lift` requires `F::Of<'a, A>: Clone + Send + Sync`.
- Same stack safety concerns. Use `stacker` or `collapse()`.

## CoyonedaExplicit (type-level fusion)

**File:** `fp-library/src/types/coyoneda_explicit.rs`
**Brand:** `CoyonedaExplicitBrand<F, B>`

A fundamentally different design. Instead of hiding the intermediate type
`B` behind a trait object, `CoyonedaExplicit` keeps it as an explicit type
parameter. Functions are composed at the type level (compile time), not via
dynamic dispatch.

```rust,ignore
struct CoyonedaExplicit<'a, F, B, A, Func: Fn(B) -> A + 'a = Box<dyn Fn(B) -> A + 'a>> {
    fb: F::Of<'a, B>,
    func: Func,
    // (a PhantomData field elided)
}
```

**When to use:** When you want true zero-cost map fusion. Ideal for
pipelines where many maps compose into a single `F::map` call. Also the
only stack-safe Coyoneda variant (no recursion in `lower`).

**Allocation:** Zero heap allocation per `map` (functions composed inline).
The `.boxed()` method erases the function type to `Box<dyn Fn>` when a
uniform type is needed (struct fields, collections, loops).

**Map fusion:** `lower` calls `F::map` exactly once, applying the fully
composed function. The store-based forms call `F::map` once per chained
`map` layer.

**HKT brand instances:** `Functor` and `Foldable`. The brand fixes `B` as
a type parameter, which prevents implementing `Pointed`, `Lift`,
`Semiapplicative`, and `Semimonad` at the brand level (they would need to
construct values with different `B` types). `pure`, `apply`, and `bind`
are available as inherent methods; there is no `lift2` counterpart.

**Notable advantages over the store-based `Coyoneda`:**

- `Foldable` does not require `F: Functor`. The fold function composes
  directly with the stored function, folding `F B` in a single pass
  without materializing an intermediate `F A`.
- `hoist` does not require `F: Functor`. The natural transformation is
  applied directly to the stored `F B`.
- Stack safe: no recursion depth regardless of chain length.

**Limitations:**

- Type complexity grows linearly with map depth (each `map` produces a
  nested closure type). For chains deeper than ~20-30 maps, insert
  `.boxed()` to bound compile-time complexity.
- Not cloneable (closures are generally not `Clone`).
- `Send`/`Sync` is conditional on the function type and `F::Of<'a, B>`.

## Choosing a Form

1. **Need full HKT type class coverage?** Use `Coyoneda` at the default
   Box store.
2. **Need to clone or lower multiple times?** Use the Rc store (single
   thread) or the Arc store (multi-thread).
3. **Need thread safety?** Use the Arc store.
4. **Need zero-cost fusion or stack safety?** Use `CoyonedaExplicit`.
5. **Building a pipeline with many maps?** Use `CoyonedaExplicit` for
   O(1) `lower`, or periodically call `collapse()` on the store-based
   forms.

## Design Notes

### Why `lower` vs `lower_ref`

The Box store and `CoyonedaExplicit` use consuming `lower(self)` because
their inner layers are not cloneable. The Rc and Arc stores use borrowing
`lower_ref(&self)` because reference counting allows cloning the base
value internally, enabling repeated lowering without consuming the
structure.

### Why brand-level type classes are store-conditional

The `Clone` bound blocker affects the Rc and Arc stores: both require
`F::Of<'a, A>: Clone` to construct a value (the base layer must clone on
`lower_ref`), but this bound cannot be expressed in the trait method
signatures of `Pointed`, `Lift`, etc. Coherence permits the differing
per-store instance sets because each impl is pinned to a disjoint `Store`
instantiation of the one `CoyonedaBrand<F, Store>`.

The Arc store has an additional blocker: the plain HKT `Functor::map`
signature does not include `Send + Sync` bounds on its closure parameter,
so closures passed through `Functor::map` cannot be stored in Arc-wrapped
layers. The `SendFunctor`/`SendFoldable` hierarchy carries those bounds,
and the Arc store implements that hierarchy instead.

`CoyonedaExplicit` is blocked differently: its brand fixes `B` as a type
parameter, so trait methods that need to construct values with arbitrary
intermediate types cannot be expressed.

### Stack safety

All three stores add one level of recursion per chained `map`. Three
mitigations:

1. **`stacker` feature (recommended).** Automatic adaptive stack growth
   with near-zero overhead when the stack is sufficient.
2. **`collapse()`** Periodically flatten accumulated layers back to a
   single base layer. Requires `F: Functor` (`F: SendFunctor` at the Arc
   store).
3. **Switch to `CoyonedaExplicit`.** Zero recursion depth regardless of
   chain length.

### Send/Sync at the Arc store

The Arc store's inner machinery uses associated type bounds on the `Kind`
trait (stable since Rust 1.79) to let the compiler auto-derive
`Send`/`Sync` for its base layer:

```rust,ignore
struct Base<'a, F, A: 'a>
where
    F: Kind<Of<'a, A>: Send + Sync> + 'a,
{
    fa: F::Of<'a, A>,
}
// Compiler auto-derives Send + Sync; no unsafe needed.
```

The map layers auto-derive unconditionally because `F` only appears inside
erased trait object bounds (`Arc<dyn ... + Send + Sync>`), not as concrete
field data.
