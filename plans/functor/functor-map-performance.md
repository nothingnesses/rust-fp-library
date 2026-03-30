# Functor::map Performance Analysis

Research into the relationship between `std::iter::Iterator::map`, `std::iter::Map`,
`fp_library::classes::Functor::map`, and the performance/memory implications of
nesting vs chaining map operations.

Date: 2026-03-30

---

## Table of Contents

1. [Key Concepts](#key-concepts)
2. [How std Iterator map Works](#how-std-iterator-map-works)
3. [How fp-library Functor::map Works](#how-fp-library-functormap-works)
4. [Inner vs Outer Iteration](#inner-vs-outer-iteration)
5. [Nesting vs Chaining: Performance Matrix](#nesting-vs-chaining-performance-matrix)
6. [Detailed Explanations](#detailed-explanations)
7. [Optimization Approaches](#optimization-approaches)
8. [Recommendations](#recommendations)

---

## Key Concepts

- **Lazy evaluation**: computation is deferred until the result is demanded.
- **Eager evaluation**: computation happens immediately when called.
- **Outer iteration** (pull-based): the consumer calls `.next()` to pull elements through
  a chain of adapters. The pipeline is lazy; you only pay for elements actually consumed.
- **Inner iteration** (push-based): the collection drives its own traversal. The caller
  receives the fully materialized result and does not control the pace.
- **Intermediate allocations**: heap allocations for collections that exist only as
  temporaries between transformation stages.

---

## How std Iterator map Works

Source: `library/core/src/iter/adapters/map.rs`

`Iterator::map` is a lazy adapter. Calling `.map(f)` on an iterator returns a
`Map<I, F>` struct that stores the inner iterator `I` and the closure `F`. No
computation happens at call time.

```rust
// The Map adapter struct (simplified)
pub struct Map<I, F> {
    iter: I,
    f: F,
}
```

The `Map` struct implements `Iterator` itself. Its `next()` method pulls one element
from the inner iterator and applies `f`:

```rust
fn next(&mut self) -> Option<B> {
    self.iter.next().map(&mut self.f)
}
```

Chaining `.map(f).map(g).map(h)` builds a nested tower of `Map<Map<Map<I, F>, G>, H>`
structs. Each `.next()` call propagates inward through the entire tower, applying all
transformations to a single element before moving to the next. There are zero
intermediate collections.

The `fold` implementation uses `map_fold` to fuse the mapping function into the
accumulator, avoiding per-element virtual dispatch:

```rust
fn map_fold<T, B, Acc>(
    mut f: impl FnMut(T) -> B,
    mut g: impl FnMut(Acc, B) -> Acc,
) -> impl FnMut(Acc, T) -> Acc {
    move |acc, elt| g(acc, f(elt))
}
```

The `InPlaceIterable` and `SourceIter` implementations enable `collect::<Vec<_>>()` to
reuse the source allocation when source and destination types have compatible layouts.

---

## How fp-library Functor::map Works

Source: `fp-library/src/classes/functor.rs`

`Functor::map` is a type-class-polymorphic function dispatched by brand. Its evaluation
semantics depend entirely on the implementing type.

### VecBrand (eager, allocating)

Source: `fp-library/src/types/vec.rs:159-164`

```rust
fn map<'a, A: 'a, B: 'a>(
    func: impl Fn(A) -> B + 'a,
    fa: Vec<A>,
) -> Vec<B> {
    fa.into_iter().map(func).collect()
}
```

This consumes the input `Vec`, lazily maps through the iterator, then eagerly collects
into a new `Vec`. The result is a fully materialized vector. Internally, `.collect()`
may reuse the source allocation via `InPlaceIterable` when types are layout-compatible,
but the traversal itself is unavoidable.

### OptionBrand (eager, non-allocating)

Source: `fp-library/src/types/option.rs:78-83`

```rust
fn map<'a, A: 'a, B: 'a>(
    func: impl Fn(A) -> B + 'a,
    fa: Option<A>,
) -> Option<B> {
    fa.map(func)
}
```

Delegates to `Option::map`, which pattern-matches on `Some(v)` and immediately applies
`func`. No heap allocation.

### ThunkBrand (lazy, box-allocating)

Source: `fp-library/src/types/thunk.rs:258-263`

```rust
pub fn map<B: 'a>(
    self,
    f: impl FnOnce(A) -> B + 'a,
) -> Thunk<'a, B> {
    Thunk::new(move || f((self.0)()))
}
```

Wraps the composition in a new `Box<dyn FnOnce>` closure. Nothing executes until
`.evaluate()` is called. Each chained map allocates one new box.

### LazyBrand (lazy, memoized, ref-based)

Source: `fp-library/src/types/lazy.rs`, `fp-library/src/classes/ref_functor.rs`

`Lazy` does not implement `Functor` because `evaluate()` returns `&A`, not `A`. It
implements `RefFunctor` instead, whose `ref_map` takes `&A -> B`. Each `ref_map` creates
a new `Lazy` cell that holds an `Rc`/`Arc` reference to its predecessor, forming a linked
chain of memoization cells.

### Summary table

| Brand       | Evaluation | Heap allocation per map | Result type     |
| ----------- | ---------- | ----------------------- | --------------- |
| VecBrand    | Eager      | 1 Vec (may reuse)       | Vec<B>          |
| OptionBrand | Eager      | 0                       | Option<B>       |
| ThunkBrand  | Lazy       | 1 Box<dyn FnOnce>       | Thunk<'a, B>    |
| LazyBrand   | Lazy       | 1 Rc/Arc cell           | Lazy<B, Config> |

---

## Inner vs Outer Iteration

**Outer iteration** (the `Iterator` model): the consumer drives the process by calling
`.next()`. The pipeline is lazy. Chained `.map()` calls build a tower of adapter structs
on the stack. Each `.next()` walks the tower and applies all functions to one element.
Zero intermediate collections.

**Inner iteration** (`VecBrand::Functor::map`): the collection drives its own traversal
internally (inside `.into_iter().map(f).collect()`). The caller receives a fully
materialized result and does not control the pace. Each call to `map` is a complete
pass over the data.

The key distinction: `Iterator::map` is always lazy regardless of the collection type;
`Functor::map` has semantics determined by the brand. For `Vec`, `Functor::map` forces
a complete traversal and materialization, making chained maps fundamentally more
expensive than chained iterator adapters.

---

## Nesting vs Chaining: Performance Matrix

Consider applying three functions `f`, `g`, `h` to a `Vec<i32>` of `n` elements.

**Chaining** means composing at the same level:

- Iterator: `v.into_iter().map(f).map(g).map(h).collect::<Vec<_>>()`
- Functor: `map(h, map(g, map(f, v)))`

**Nesting** means composing the functions themselves:

- Iterator: `v.into_iter().map(|x| h(g(f(x)))).collect::<Vec<_>>()`
- Functor: `map(|x| h(g(f(x))), v)` or `map(compose(h, compose(g, f)), v)`

### Vec matrix (k = number of map stages, n = number of elements)

| Aspect                                 | Iter chained  | Iter nested   | Functor chained | Functor nested |
| -------------------------------------- | ------------- | ------------- | --------------- | -------------- |
| Intermediate Vec allocations           | 0             | 0             | k-1             | 0              |
| Function applications per element      | k             | k             | k               | k              |
| Total heap allocations                 | 1 (collect)   | 1 (collect)   | k               | 1              |
| Cache locality                         | Good (1 pass) | Good (1 pass) | Poor (k passes) | Good (1 pass)  |
| Lazy until consumed                    | Yes           | Yes           | No              | No             |
| Compiler can fuse/inline across stages | Yes           | Yes           | Unlikely        | Yes            |
| Stack-allocated adapter structs        | k-1           | 0             | 0               | 0              |
| Asymptotic memory                      | O(n)          | O(n)          | O(k \* n)       | O(n)           |
| Asymptotic time                        | O(k \* n)     | O(k \* n)     | O(k \* n)       | O(k \* n)      |

### Thunk matrix

| Aspect                  | Thunk chained                   | Thunk nested      |
| ----------------------- | ------------------------------- | ----------------- |
| Heap allocs at map time | k Box<dyn FnOnce> (one per map) | 1 Box<dyn FnOnce> |
| Evaluation cost         | k nested function calls         | k nested fn calls |
| Closure captures        | Each captures previous Thunk    | Single closure    |

---

## Detailed Explanations

### Iterator chaining vs nesting: nearly identical

The chained form `.map(f).map(g).map(h)` creates a tower of `Map` structs on the stack,
but each `.next()` call walks the tower and applies all three functions to one element
before allocating anything. LLVM typically inlines the entire chain into a single loop
body (monomorphization makes each `Map<Map<...>>` a concrete type with no indirection).

The nested form `.map(|x| h(g(f(x))))` skips the adapter structs entirely but produces
the same machine code in practice. The difference is negligible. The chained form is
sometimes marginally better because `fold()` can be specialized via `map_fold`, which
fuses the mapping function into the fold's accumulator function.

### Functor chaining on Vec: the expensive case

Each `map::<VecBrand, _, _>(f, v)` call does `.into_iter().map(f).collect()`, producing
a new `Vec`. Chaining three maps means:

1. Allocate `Vec<B>` of size n, iterate all of v, apply f.
2. Allocate `Vec<C>` of size n, iterate all of result 1, apply g.
3. Allocate `Vec<D>` of size n, iterate all of result 2, apply h.

That is 3 allocations, 3 full traversals, and 2 intermediate `Vec`s that exist only to
be consumed and dropped. Each traversal touches n cache lines in the input and n in the
output, so cache pressure is roughly 3x worse than a single-pass solution.

Note: `collect()` may reuse the source allocation when types are layout-compatible
(`InPlaceIterable`), so the actual allocation count can be lower. But the multiple
traversals remain, and the compiler cannot fuse function applications across the
`collect()` boundary.

### Functor nesting: equivalent to the best case

`map::<VecBrand, _, _>(|x| h(g(f(x))), v)` makes exactly one pass, one allocation,
and applies the composed function inline. This is identical in cost to the iterator
approaches. The library already provides `compose` in `fp-library/src/functions.rs:88-93`
to make this ergonomic:

```rust
pub fn compose<A, B, C>(
    f: impl Fn(B) -> C,
    g: impl Fn(A) -> B,
) -> impl Fn(A) -> C {
    move |a| f(g(a))
}
```

### Thunk chaining: moderate overhead

For `ThunkBrand`, chaining `map(h, map(g, map(f, thunk)))` creates a chain of
`Thunk::new(move || h((prev.0)()))` wrappers. Each is a `Box<dyn FnOnce>`. At
evaluation time the closures unwind in sequence. You pay for k box allocations at
composition time, whereas the nested form pays for just one. The actual computation
at evaluation time is the same either way.

### Lazy chaining: Rc/Arc chain accumulation

Chaining `ref_map` on `Lazy` types creates a linked list of memoization cells. Each
cell holds an `Rc`/`Arc` reference to its predecessor. The `RefFunctor` documentation
warns about this: "long chains can accumulate memory that is only freed when the final
value in the chain is dropped."

---

## Optimization Approaches

### Approach 1: Coyoneda (free fusion for any Functor)

The Coyoneda lemma: for any type constructor F, the type `Coyoneda F` is a Functor
that accumulates `map` calls as function composition and defers the actual mapping to
a single `lower` step.

Conceptually: `Coyoneda F A = exists B. (B -> A, F B)`

Multiple maps compose the function without touching the inner structure:

```
map(h, map(g, map(f, coyo)))
  = Coyoneda { fb: original_fb, func: h . g . f }
```

When you `lower` (requiring `Brand: Functor`), it does a single `Brand::map(composed_func, fb)`.

#### Encoding option A: closure-based (erase B behind FnOnce)

```rust
struct Coyoneda<'a, Brand: Kind<...>, A: 'a> {
    lower_fn: Box<dyn FnOnce() -> Apply!(Brand::Of<'a, A>) + 'a>,
}
```

Captures the original `F<B>` and the `B -> A` function in a single closure. Mapping
wraps another closure around it. Lowering calls it. Each `map` allocates a new `Box`.
Same cost profile as chaining `Thunk::map`.

#### Encoding option B: trait-object-based (type-erase the inner value)

```rust
trait CoyonedaInner<'a, Brand, A: 'a> {
    fn lower(self: Box<Self>) -> Apply!(Brand::Of<'a, A>);
}

struct CoyonedaImpl<'a, Brand, B: 'a, A: 'a> {
    fb: Apply!(Brand::Of<'a, B>),
    func: Box<dyn Fn(B) -> A + 'a>,
}

impl CoyonedaInner<'a, Brand, A> for CoyonedaImpl<'a, Brand, B, A> {
    fn lower(self: Box<Self>) -> Apply!(Brand::Of<'a, A>) {
        Brand::map(self.func, self.fb)
    }
}
```

Mapping replaces `func` with `compose(new_f, old_func)` without touching `fb`. When
you `lower`, one `Brand::map` call applies the fully composed function.

**Trade-offs:**

- (+) Universal: works for any Functor brand without modifying the brand's impl.
- (+) Chaining k maps on Coyoneda<VecBrand> is O(1) per map (function composition),
  then O(n) once at `lower`. Total: O(n) instead of O(k\*n).
- (+) Natural fit for the HKT/brand system; could have its own `CoyonedaBrand<Brand>`.
- (-) Requires explicit `lower` call; user must opt into the Coyoneda wrapper.
- (-) Involves Box<dyn ...> per map (closure-based) or a single Box<dyn ...> that gets
  replaced (trait-object-based).
- (-) Existential encoding is less ergonomic in Rust than in Haskell.

### Approach 2: Build/foldr fusion (short-cut deforestation)

Represent list-producing operations as their `build` function (church encoding) and
list-consuming operations as `foldr`. The rewrite `foldr k z (build g) = g k z`
eliminates intermediate lists entirely.

In Rust, rewrite rules are not possible, but sequences can be represented as their fold:

```rust
struct FoldVec<'a, A> {
    fold: Box<dyn FnOnce(/* consumer */) -> /* result */ + 'a>,
}
```

`map` on this representation composes into the fold function with no allocation.
Materializing runs the fold once.

**Trade-offs:**

- (+) True deforestation: intermediate structures never exist.
- (-) Hard to implement correctly in Rust (lifetimes, ownership).
- (-) Only applies to list-like types, not general across all functors.
- (-) Church-encoded sequences lose random access, length, etc.
- (-) Would need a `FoldVecBrand` alongside `VecBrand`, fragmenting the API.

### Approach 3: Iterator-based lazy brand

```rust
struct LazyVecBrand;
impl_kind! {
    for LazyVecBrand {
        type Of<'a, A: 'a>: 'a = Box<dyn Iterator<Item = A> + 'a>;
    }
}
```

`Functor::map` wraps the iterator in a `Map` adapter and re-boxes. No intermediate
Vec allocations.

**Trade-offs:**

- (+) Maps are lazy with zero intermediate collection allocations.
- (+) Fits into the existing HKT system.
- (-) Each map re-boxes the iterator (one allocation per map).
- (-) Loses Vec-specific capabilities (indexing, known length, Clone).
- (-) Single-use: Box<dyn Iterator> is not Clone. Breaks Applicative, Monad, etc.
- (-) Dynamic dispatch on .next() for each element at each layer.

### Approach 4: Fused map combinators (pragmatic)

Provide ergonomic composition utilities:

```rust
pub fn map_compose<'a, Brand: Functor, A: 'a, B: 'a, C: 'a>(
    f: impl Fn(B) -> C + 'a,
    g: impl Fn(A) -> B + 'a,
    fa: Apply!(Brand::Of<'a, A>),
) -> Apply!(Brand::Of<'a, C>) {
    Brand::map(compose(f, g), fa)
}
```

Or a builder pattern:

```rust
let result = FunctorPipeline::new::<VecBrand>(vec![1, 2, 3])
    .map(|x| x + 1)
    .map(|x| x * 2)
    .map(|x| x.to_string())
    .run();  // single Brand::map with the composed function
```

The pipeline accumulates function compositions internally and only calls `Brand::map`
once at `.run()`.

**Trade-offs:**

- (+) No new types, brands, or trait changes.
- (+) Essentially Coyoneda without the categorical name.
- (+) Works for all Functor brands.
- (+) Zero-cost if closures inline (which they will for concrete types).
- (-) Opt-in: users must remember to use the pipeline.
- (-) Storing composed functions across multiple stages needs boxing or recursive generics.

### Approach 5: Specialized Vec implementation

Override `VecBrand`'s `Functor::map` to reuse allocations when A and B have the same
layout. However, `collect()` already handles this via `InPlaceIterable` / `SourceIter`.
The real cost of chaining is the multiple traversals, not the allocations, and this
approach does not address that.

**Trade-offs:**

- (+) Could eliminate allocation overhead in some cases.
- (-) `collect()` already does this for layout-compatible types.
- (-) Does not address the multi-traversal problem.
- (-) Limited applicability.

---

## Recommendations

These approaches are not mutually exclusive. In priority order:

1. **Coyoneda** (approach 1, encoding B) as a new type in `types/`, with
   `CoyonedaBrand<Brand>`. The canonical FP answer. Fits the library's philosophy.
   Works universally. Can be built without touching existing code.

2. **FunctorPipeline** (approach 4) as a convenience layer for users who want a builder
   API without the categorical framing.

3. **Document the cost model** so users understand that
   `map(f, map(g, v))` on VecBrand is O(2n) with 2 allocations, while
   `map(compose(f, g), v)` is O(n) with 1. The existing `compose` function already
   enables the fast path. This is the most important step regardless of which
   optimization path is taken.

---

## Existing relevant infrastructure in fp-library

- `compose` function: `fp-library/src/functions.rs:88-93`
- `Functor` trait: `fp-library/src/classes/functor.rs`
- `RefFunctor` trait: `fp-library/src/classes/ref_functor.rs`
- `SendRefFunctor` trait: `fp-library/src/classes/send_ref_functor.rs`
- Vec Functor impl: `fp-library/src/types/vec.rs:133-164`
- Option Functor impl: `fp-library/src/types/option.rs:48-83`
- Thunk Functor impl: `fp-library/src/types/thunk.rs:472-506`
- Thunk::map method: `fp-library/src/types/thunk.rs:258-263`
- Lazy/RefFunctor impl: `fp-library/src/types/lazy.rs`
- Free monad: `fp-library/src/types/free.rs` (uses CatList for O(1) bind)
- Codensity mentioned in: `plans/lazy/plan.md:302`
- Compose Kleisli: `fp-library/src/classes/semimonad.rs:220-225`
