# Trampoline Analysis

## Overview

`Trampoline<A>` is a newtype wrapper around `Free<ThunkBrand, A>` providing stack-safe, lazy computation with O(1) bind. It lives at `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/trampoline.rs`.

The type is intentionally simple: `struct Trampoline<A: 'static>(Free<ThunkBrand, A>)`. All heavy lifting (the trampoline loop, CatList-based continuation management, type erasure) is delegated to `Free`.

---

## 1. Design

### Relationship to Free

`Trampoline` is `Free<ThunkBrand, A>`, which is the standard encoding: a free monad over an identity-like functor (Thunk, which is essentially `() -> A`) specializes to a trampoline. This is well-established in the FP literature and matches PureScript's `purescript-free` where `type Trampoline = Free Lazy`.

The delegation is clean. Every `Trampoline` method (`pure`, `new`, `defer`, `bind`, `map`, `evaluate`) is a thin wrapper that constructs or delegates to the corresponding `Free` operation. This avoids code duplication and keeps `Trampoline` as a focused ergonomic API.

### Why no HKT traits

`Trampoline` cannot implement the library's HKT traits (`Functor`, `Monad`, etc.) for the same reason `Free` cannot: the "Reflection without Remorse" implementation uses `Box<dyn Any>` for type erasure in the continuation CatList, and `dyn Any` requires `A: 'static`. The library's `Kind` trait requires `type Of<'a, A: 'a>: 'a` with lifetime polymorphism, which is incompatible.

This is the correct design decision. The alternative (naive recursive Free without type erasure) would sacrifice:
- Stack safety (recursive `evaluate` would blow the stack).
- O(1) bind (left-associated binds would be O(n^2)).

The tradeoff is clearly documented in both `free.rs` and `trampoline.rs`.

### Role in the lazy hierarchy

The three-type hierarchy is well-motivated:

| Type | HKT | Stack safe | Memoized | Lifetimes |
|------|-----|-----------|----------|-----------|
| `Thunk<'a, A>` | Yes | No (partial via `tail_rec_m`) | No | `'a` |
| `Trampoline<A>` | No | Yes | No | `'static` |
| `Lazy<'a, A>` | Partial (`RefFunctor`) | N/A | Yes | `'a` |

Each type fills a distinct niche. `Trampoline` is the "heavy duty" option for deep recursion.

---

## 2. Implementation Correctness

### Construction methods

- `pure(a)` wraps `Free::pure(a)`. Correct.
- `new(f)` creates `Free::wrap(Thunk::new(move || Free::pure(f())))`. This wraps a thunk that, when forced, produces a `Free::pure` of the closure result. Correct.
- `defer(f)` creates `Free::wrap(Thunk::new(move || f().0))`. This wraps a thunk that, when forced, extracts the inner `Free` from the produced `Trampoline`. This is the critical combinator for stack-safe recursion: it defers the construction of the next trampoline step. Correct.

### bind and map

Both delegate directly to `Free::bind` and `Free::map`. The `bind` implementation in `Free` uses CatList snoc for O(1) append. Correct.

### evaluate

Delegates to `Free::evaluate`, which runs the iterative trampoline loop:
1. Type-erase the initial Free.
2. Loop: match on `FreeInner` variant.
   - `Pure(val)`: pop a continuation from the CatList and apply it, or return the final value.
   - `Wrap(fa)`: call `Evaluable::evaluate` on the thunk to get the next `Free`.
   - `Map`: convert to a continuation and prepend.
   - `Bind`: merge inner CatList with outer CatList (O(1) append).

The loop is constant-stack: no recursion, just iteration with CatList operations.

### tail_rec_m

The implementation uses `defer` and `bind` to build a chain that is evaluated iteratively:

```rust
fn go<A, B, F>(f: F, a: A) -> Trampoline<B>
where F: Fn(A) -> Trampoline<Step<A, B>> + Clone + 'static {
    let f_clone = f.clone();
    Trampoline::defer(move || {
        f(a).bind(move |step| match step {
            Step::Loop(next) => go(f_clone.clone(), next),
            Step::Done(b) => Trampoline::pure(b),
        })
    })
}
```

This is stack safe because:
- `Trampoline::defer` wraps the recursive call in a `Thunk`, so `go` returns immediately without recursing.
- The actual recursion happens lazily during `evaluate`, which processes it iteratively via the trampoline loop.
- Each `Step::Loop` produces a new `defer` + `bind` chain that is again processed iteratively.

**Potential concern with `go` recursion**: The `go` function itself is recursive, but it is only called once eagerly (to build the initial deferred computation). Subsequent calls to `go` happen inside `defer` closures, which are not evaluated until the trampoline loop forces the thunk. At that point, `go` returns a new `Trampoline::defer(...)` immediately, so the Rust call stack never grows beyond a constant depth during evaluation. This is correct.

### arc_tail_rec_m

Wraps the closure in `Arc` and delegates to `tail_rec_m`. The `Arc` makes the closure `Clone`, which `tail_rec_m` requires. This is a clean solution for non-Clone closures.

### From impls

- `From<Lazy<'static, A, Config>>`: Creates `Trampoline::new(move || lazy.evaluate().clone())`. Requires `A: Clone` because `Lazy::evaluate` returns `&A`. Correct.
- `From<Thunk<'static, A>>`: Creates `Trampoline::new(move || thunk.evaluate())`. Correct, consumes the thunk.

### memoize and memoize_arc

- `memoize()` returns `Lazy::from(self)`, which (presumably via a `From` impl on `Lazy`) wraps the trampoline evaluation in a lazy cell. Returns `Lazy<'static, A, RcLazyConfig>`.
- `memoize_arc()` eagerly evaluates the trampoline and wraps the result in `ArcLazy`. The eager evaluation is necessary because `Trampoline`'s internal closures are not `Send`, so they cannot be stored in an `Arc`-based lazy cell. The documentation correctly explains this. Good design.

### Debug impl

Returns `"Trampoline(<unevaluated>)"` without forcing evaluation. Correct; forcing evaluation in `Debug` would be a side-effect and would consume `self` (which `Debug` cannot do since it takes `&self`).

---

## 3. Stack Safety

### The core guarantee

Stack safety is upheld by the `Free::evaluate` trampoline loop. The loop processes `FreeInner` variants iteratively:
- `Bind` nodes have their continuations merged into the outer CatList without recursion.
- `Wrap` nodes force a single thunk (constant stack) to produce the next `Free` step.
- `Map` nodes are converted to continuations.
- `Pure` nodes apply the next continuation.

No step in this loop grows the stack proportionally to the computation depth.

### Edge cases

**Deep defer chains**: A chain of 1,000,000 `Trampoline::defer` calls produces 1,000,000 nested `Wrap` nodes. Each iteration of the loop forces one thunk (O(1) stack) and produces the next `Free`. Stack safe.

**Deep bind chains**: Left-associated `t.bind(f1).bind(f2)...bind(fn)` produces a single `Bind` node with a CatList of n continuations (thanks to O(1) snoc). During evaluation, each `Pure` result pops one continuation. Stack safe.

**Right-associated bind chains**: `t.bind(|a| f1(a).bind(|b| f2(b).bind(...)))` produces nested `Bind` nodes. The evaluate loop handles this by merging inner CatLists with the outer one via `inner_continuations.append(continuations)`. Stack safe.

**Mixed defer + bind**: `tail_rec_m` produces exactly this pattern. Each iteration creates a `defer` (Wrap) containing a `bind` (Bind). The loop alternates between forcing thunks and processing continuations. Stack safe.

**CatList operations**: `CatList::uncons` and `CatList::append` must be stack safe themselves. CatList is a catenable list (likely based on Okasaki's design); if it uses amortized O(1) operations via lazy rebuilding, it should not introduce stack growth. This is a dependency, not a concern within `trampoline.rs` itself.

### Confirmed by tests

The test `test_task_tail_rec_m` and the doc example for `defer` (with `recursive_sum(1_000, 0)`) verify stack safety at moderate depth. The doc comment claims "n = 1_000_000" works, though the test uses only 1,000. A deeper stress test would strengthen confidence, though the architecture clearly supports arbitrary depth.

---

## 4. Consistency with Library Patterns

### Documentation style

Follows the project's documentation conventions: `#[document_signature]`, `#[document_type_parameters(...)]`, `#[document_parameters(...)]`, `#[document_returns(...)]`, `#[document_examples]`. Consistent.

### Module structure

Uses `#[fp_macros::document_module] mod inner { ... } pub use inner::*;` pattern. Consistent with `thunk.rs`, `free.rs`, and other type modules.

### Naming conventions

- `evaluate` for forcing computation (matches `Evaluable` trait and `Thunk::evaluate`).
- `pure` for wrapping a value (matches `Pointed` / standard FP terminology).
- `bind` for monadic chaining (matches the library's conventions).
- `map` for functor mapping.
- `defer` for lazy construction.
- `memoize` for converting to `Lazy`.

All consistent.

### Test structure

Tests cover: `pure`, `new`, `bind`, `map`, `defer`, `tail_rec_m`, `lift2`, `then`, `arc_tail_rec_m`, `From<Lazy>`, `From<Thunk>`, `append`, `empty`, and QuickCheck property tests for functor/monad laws. Also includes tests for `!Send` types (`Rc`). Thorough.

### Minor inconsistencies

- Test `test_task_map2` tests `lift2` but is named `map2`. Similarly `test_task_and_then` tests `then` but is named `and_then`. These likely reflect a rename that was not propagated to test names. Harmless but worth fixing for clarity.
- The test comments reference "map2" and "and_then" which are not the current method names.

---

## 5. Limitations

### `'static` requirement

All type parameters must be `'static` because `Free` uses `Box<dyn Any>` for type erasure, and `Any` requires `'static`. This prevents:
- Borrowing data into a `Trampoline` (e.g., `Trampoline<&'a str>` is impossible).
- Interoperating with the HKT trait hierarchy.

This is an inherent limitation of the "Reflection without Remorse" technique in Rust. Addressing it would require either:
1. Unsafe code to bypass `Any`'s `'static` requirement (risky, hard to verify).
2. A different approach to type erasure (e.g., `TypeId`-based, but this also requires `'static`).
3. Abandoning O(1) bind and stack safety (unacceptable).

The limitation is well-documented and the right tradeoff.

### Not `Send`

`Trampoline` is not `Send` because its internal closures (stored as `Box<dyn FnOnce>` in `Free`'s CatList) do not require `Send`. This means trampolines cannot be sent across threads. The `memoize_arc` method works around this by eagerly evaluating before wrapping in `ArcLazy`.

A `SendTrampoline` variant could be built by parameterizing `Free` over `Send` bounds, but this would require significant refactoring of the `Free` infrastructure.

### No memoization

Each `evaluate` call re-runs the entire computation. The `memoize` and `memoize_arc` methods provide an escape hatch by converting to `Lazy`. This is the correct design; memoization adds overhead (interior mutability, reference counting) that should be opt-in.

### No `Eq`, `Ord`, `Display`, or other standard traits

`Trampoline` only implements `Debug` (which does not force evaluation). Implementing `Eq` or `Ord` would require evaluating both sides, which consumes them (since `evaluate` takes `self`). This is a fundamental tension with move semantics. PureScript's `Lazy` can implement `Eq` because `force` is not destructive.

### Clone is not implemented

`Trampoline` wraps `Free`, which contains `Box<dyn FnOnce>` closures. `FnOnce` cannot be cloned. This means trampolines are single-use. This is consistent with the move-based evaluation (`evaluate(self)`) but limits composability in some patterns.

---

## 6. Alternative Designs

### Direct trampoline (without Free)

A simpler encoding would be:

```rust
enum Trampoline<A> {
    Done(A),
    More(Box<dyn FnOnce() -> Trampoline<A>>),
}
```

This avoids the `Free` machinery but has O(n) left-associated bind (each bind wraps another closure layer). The current design's O(1) bind via CatList is strictly better for monadic pipelines.

### Parameterizing over Send

The `Free` type could be parameterized over a marker trait to support both `Send` and `!Send` variants. This would be a significant refactoring effort. For now, the single-threaded `Trampoline` with `memoize_arc` as an escape hatch is pragmatic.

### Coroutine / generator-based

Rust's upcoming coroutine support could potentially replace the trampoline pattern. However, coroutines are unstable and their interaction with the type system is still evolving. The current Free-based approach is well-understood and portable.

### Removing the 'static requirement

The most significant design question is whether `Free` could be refactored to avoid `Box<dyn Any>`. One approach: use a type-indexed heterogeneous list instead of type-erased CatList. However, this would make the continuation list's type grow with each `bind`, defeating the purpose of O(1) append. The current approach is the standard solution and the `'static` cost is acceptable.

---

## 7. Documentation

### Accuracy

The documentation is accurate:
- Correctly states `'static` requirement and why.
- Correctly describes O(1) bind.
- Correctly differentiates from `Thunk` and `Lazy`.
- Correctly notes lack of memoization.
- Examples compile and demonstrate the claimed behavior.

### Completeness

The module-level docs and struct-level docs are thorough. Each method has signature docs, parameter docs, return docs, and examples. The comparison table with `Thunk` is helpful.

### Minor issues

- The `memoize` doc example uses `*lazy.evaluate()` with a deref, which is correct (`Lazy::evaluate` returns `&A`) but might confuse readers unfamiliar with `Lazy`. A brief note explaining the deref would help.
- The `recursive_sum` example in `defer` claims "n = 1_000_000" works but the actual test uses `n = 1_000`. Running with 1,000,000 in a doc test might be too slow; the claim is correct but untested in CI.
- The `CatList` link in the struct docs resolves correctly, but the `bind` link uses `crate::functions::bind` which points to the free function, not the inherent method. This could be confusing since `Trampoline::bind` is an inherent method, not a dispatch through the trait.

---

## Summary of Findings

**Overall assessment**: The implementation is solid, well-designed, and well-documented. It correctly delegates to `Free` for the heavy lifting and provides a clean ergonomic API. The stack safety guarantee is upheld by the iterative trampoline loop in `Free::evaluate`. The `'static` limitation is an inherent consequence of the chosen approach and is clearly documented.

**Issues to consider fixing**:
1. Rename test functions `test_task_map2` and `test_task_and_then` to match current method names (`lift2` and `then`).
2. Add a deeper stress test (e.g., 100,000 iterations) to validate stack safety claims more rigorously.
3. Clarify the `memoize` example regarding the deref on `Lazy::evaluate`'s return type.
4. The `bind` doc link points to `crate::functions::bind` rather than the inherent method; consider updating.

**No bugs or correctness issues found.** The implementation is a faithful, correct encoding of the standard Free-monad-based trampoline pattern adapted for Rust's ownership model.
