# Coyoneda Implementations

The library provides four Coyoneda implementations, each making different
trade-offs around ownership, cloning, thread safety, HKT integration, and
map fusion.

**User story:** "I want to chain maps without calling `F::map` until later."
Useful for map fusion, lazy mapping, and getting a `Functor` instance for
any type constructor for free.

All four implement the same core idea: wrap a functor value `F B` together
with a deferred function `B -> A`, delaying the call to `F::map` until
`lower` time. This lets you chain `map` calls in O(1) each, regardless of
whether `F` is a `Functor`.

## Quick Reference

|                 | Coyoneda      | RcCoyoneda         | ArcCoyoneda        | CoyonedaExplicit  |
| --------------- | ------------- | ------------------ | ------------------ | ----------------- |
| Wrapper         | `Box`         | `Rc`               | `Arc`              | None (type-level) |
| Lower           | `lower(self)` | `lower_ref(&self)` | `lower_ref(&self)` | `lower(self)`     |
| Clone           | No            | Yes, O(1)          | Yes, O(1)          | No                |
| Send + Sync     | No            | No                 | Yes                | Conditional       |
| Heap per map    | 1 Box         | 2 Rc               | 2 Arc              | 0                 |
| Map fusion      | No (k calls)  | No (k calls)       | No (k calls)       | Yes (1 call)      |
| Stack safe      | No            | No                 | No                 | Yes               |
| Functor brand   | Yes           | Yes                | No                 | Yes               |
| Pointed brand   | Yes           | No                 | No                 | No                |
| Foldable brand  | Yes           | Yes                | Yes                | Yes               |
| Semimonad brand | Yes           | No                 | No                 | No                |

## Coyoneda (Box-based, consuming)

**File:** `fp-library/src/types/coyoneda.rs`
**Brand:** `CoyonedaBrand<F>`

The baseline implementation. Stores layers as `Box<dyn CoyonedaInner>` trait
objects with functions inlined in each layer (no separate allocation for the
function). Consuming `lower(self)` applies accumulated maps via `F::map`.

**When to use:** General-purpose deferred mapping when you do not need
cloning or thread safety, and want full HKT type class coverage.

**Allocation:** `lift` allocates 1 Box. Each `map` allocates 1 Box (the
layer; the function is stored inline).

**HKT brand instances:** Functor, Pointed, Foldable, Lift, ApplyFirst,
ApplySecond, Semiapplicative, Semimonad. This is the only Coyoneda variant
with full type class coverage because `Box<dyn FnOnce>` has no `Clone` or
`Send` requirements.

**Limitations:**

- Not cloneable (`Box<dyn>` is not `Clone`).
- Not `Send`/`Sync`.
- Each chained `map` adds a layer of recursion to `lower`. Deep chains
  (thousands of maps) can overflow the stack. Mitigations: `stacker`
  feature (automatic stack growth), `collapse()` (manual flattening), or
  switching to `CoyonedaExplicit`.
- No `unCoyoneda`-style rank-2 eliminator (Rust lacks rank-2 types), so
  `hoist` and `Foldable` require `F: Functor`.

## RcCoyoneda (Rc-based, cloneable)

**File:** `fp-library/src/types/rc_coyoneda.rs`
**Brand:** `RcCoyonedaBrand<F>`

Wraps layers in `Rc`, making the structure cheaply cloneable. Uses
`lower_ref(&self)` which clones the base value internally, allowing
repeated lowering without consuming.

**When to use:** When you need to share or reuse a Coyoneda value (e.g.,
lowering it multiple times, storing it in a data structure) and do not need
thread safety.

**Allocation:** `lift` allocates 1 Rc. Each `map` allocates 2 Rc (one for
the layer trait object, one for the `Rc<dyn Fn>` function wrapper).

**HKT brand instances:** Functor and Foldable only. Pointed, Lift,
Semiapplicative, and Semimonad are not implementable at the brand level
because constructing an `RcCoyoneda` requires `F::Of<'a, A>: Clone`, a
bound that cannot be expressed in the trait method signatures. These
operations are available as inherent methods (`pure`, `bind`, `apply`,
`lift2`) with explicit `Clone` bounds.

**Limitations:**

- Not `Send`/`Sync` (`Rc` is single-threaded).
- Same stack safety concerns as `Coyoneda`. Use `stacker` or `collapse()`.
- `lower_ref` clones the base functor value on every call.

## ArcCoyoneda (Arc-based, thread-safe)

**File:** `fp-library/src/types/arc_coyoneda.rs`
**Brand:** `ArcCoyonedaBrand<F>`

Wraps layers in `Arc` with `Send + Sync` requirements on functions and
inner layers. Uses associated type bounds on the `Kind` trait
(`Kind<Of<'a, A>: Send + Sync>`) to enable the compiler to auto-derive
`Send`/`Sync` without `unsafe`.

**When to use:** When you need to share a Coyoneda value across threads.

**Allocation:** `lift` allocates 1 Arc. Each `map` allocates 2 Arc (one
for the layer trait object, one for the `Arc<dyn Fn + Send + Sync>`
function wrapper).

**HKT brand instances:** Foldable only. Functor is not implementable
because the HKT `Functor::map` signature lacks `Send + Sync` bounds on
its closure parameter, so closures cannot be stored in Arc-wrapped layers.
Pointed, Lift, Semiapplicative, and Semimonad are blocked by both the
Functor issue and the `Clone + Send + Sync` bound on `F::Of` that cannot
be expressed in trait method signatures. All operations are available as
inherent methods.

**Limitations:**

- Atomic reference counting overhead vs `RcCoyoneda`.
- `map` requires closures to be `Send + Sync`.
- `lift` and `new` require `F::Of<'a, A>: Clone + Send + Sync`.
- Same stack safety concerns. Use `stacker` or `collapse()`.

## CoyonedaExplicit (type-level fusion)

**File:** `fp-library/src/types/coyoneda_explicit.rs`
**Brand:** `CoyonedaExplicitBrand<F, B>`

A fundamentally different design. Instead of hiding the intermediate type
`B` behind a trait object, `CoyonedaExplicit` keeps it as an explicit type
parameter. Functions are composed at the type level (compile time), not via
dynamic dispatch.

```rust,ignore
struct CoyonedaExplicit<'a, F, B, A, Func: Fn(B) -> A = Box<dyn Fn(B) -> A + 'a>> {
    fb: F::Of<'a, B>,
    func: Func,
}
```

**When to use:** When you want true zero-cost map fusion. Ideal for
pipelines where many maps compose into a single `F::map` call. Also the
only stack-safe Coyoneda variant (no recursion in `lower`).

**Allocation:** Zero heap allocation per `map` (functions composed inline).
The `.boxed()` method erases the function type to `Box<dyn Fn>` when a
uniform type is needed (struct fields, collections, loops).

**Map fusion:** `lower` calls `F::map` exactly once, applying the fully
composed function. The other three variants call `F::map` once per chained
`map` layer.

**HKT brand instances:** Functor and Foldable. The brand fixes `B` as a
type parameter, which prevents implementing Pointed, Lift,
Semiapplicative, and Semimonad at the brand level (they would need to
construct values with different `B` types). These operations are available
as inherent methods.

**Notable advantages over Coyoneda:**

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

## Choosing an Implementation

1. **Need full HKT type class coverage?** Use `Coyoneda`.
2. **Need to clone or lower multiple times?** Use `RcCoyoneda` (single
   thread) or `ArcCoyoneda` (multi-thread).
3. **Need thread safety?** Use `ArcCoyoneda`.
4. **Need zero-cost fusion or stack safety?** Use `CoyonedaExplicit`.
5. **Building a pipeline with many maps?** Use `CoyonedaExplicit` for
   O(1) `lower`, or periodically call `collapse()` on the other variants.

## Design Notes

### Why `lower` vs `lower_ref`

`Coyoneda` and `CoyonedaExplicit` use consuming `lower(self)` because
their inner layers are not cloneable. `RcCoyoneda` and `ArcCoyoneda` use
borrowing `lower_ref(&self)` because reference counting allows cloning the
base value internally, enabling repeated lowering without consuming the
structure.

### Why brand-level type classes are limited

The `Clone` bound blocker affects `RcCoyoneda` and `ArcCoyoneda`: both
require `F::Of<'a, A>: Clone` to construct a value (the base layer must
clone on `lower_ref`), but this bound cannot be expressed in the trait
method signatures of `Pointed`, `Lift`, etc. These operations are provided
as inherent methods instead.

`ArcCoyoneda` has an additional blocker: the HKT `Functor::map` signature
does not include `Send + Sync` bounds on its closure parameter, so
closures passed through `Functor::map` cannot be stored in Arc-wrapped
layers.

`CoyonedaExplicit` is blocked differently: its brand fixes `B` as a type
parameter, so trait methods that need to construct values with arbitrary
intermediate types cannot be expressed.

### Stack safety

`Coyoneda`, `RcCoyoneda`, and `ArcCoyoneda` all add one level of
recursion per chained `map`. Three mitigations:

1. **`stacker` feature (recommended).** Automatic adaptive stack growth
   with near-zero overhead when the stack is sufficient.
2. **`collapse()`** Periodically flatten accumulated layers back to a
   single base layer. Requires `F: Functor`.
3. **Switch to `CoyonedaExplicit`.** Zero recursion depth regardless of
   chain length.

### Send/Sync in ArcCoyoneda

`ArcCoyoneda` uses associated type bounds on the `Kind` trait (stable
since Rust 1.79) to let the compiler auto-derive `Send`/`Sync`:

```rust,ignore
struct ArcCoyonedaBase<'a, F, A: 'a>
where
    F: Kind<Of<'a, A>: Send + Sync> + 'a,
{
    fa: F::Of<'a, A>,
}
// Compiler auto-derives Send + Sync; no unsafe needed.
```

`ArcCoyonedaMapLayer` auto-derives unconditionally because `F` only
appears inside erased trait object bounds (`Arc<dyn ... + Send + Sync>`),
not as concrete field data.
