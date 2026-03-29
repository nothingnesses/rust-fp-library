# Thunk Analysis

## Overview

`Thunk<'a, A>` is a non-memoizing, single-shot lazy computation wrapping `Box<dyn FnOnce() -> A + 'a>`. It is the only lazy type in the hierarchy with full HKT support (Functor, Monad, Foldable, etc.), because its `'a` lifetime parameter aligns with the `Kind` trait's `Of<'a, A: 'a>: 'a` signature. It is not `Send`, not `Clone`, and not stack-safe for `bind` chains.

File: `fp-library/src/types/thunk.rs`

---

## 1. Type Design

### Underlying Representation

```rust
pub struct Thunk<'a, A>(Box<dyn FnOnce() -> A + 'a>);
```

This is correct and well-chosen:

- **`Box<dyn FnOnce()>`** is the minimal representation for a deferred computation that produces exactly one value. `FnOnce` captures by ownership, which is the most permissive closure kind; it allows closures that move captured variables out.
- **`'a` lifetime parameter** enables borrowing from the enclosing scope, unlike `Trampoline` which requires `'static`. This is a major ergonomic win for short-lived computations.
- **Tuple struct with private field** prevents external construction, forcing users through `Thunk::new`.

### Alternatives Considered

| Alternative | Trade-off |
|---|---|
| `Box<dyn Fn() -> A>` | Would allow multiple evaluations but prevents move-captures. Wrong for a single-shot computation. |
| `enum { Value(A), Thunk(Box<dyn FnOnce() -> A>) }` | Would enable `pure` without boxing a closure, but complicates every combinator with an extra match arm. The optimizer likely already elides the closure for trivial `pure` cases. |
| Raw function pointer `fn() -> A` | Cannot capture environment. Not viable for a general lazy type. |
| Generic `F: FnOnce() -> A` instead of `dyn` | Would monomorphize (no allocation), but loses HKT: `Kind::Of<'a, A>` must be a single concrete type, not a family parameterized by `F`. |

The current `Box<dyn FnOnce()>` is the right choice for a type that must be a single concrete type (for HKT) while supporting arbitrary closures.

### One Design Subtlety

The struct wraps `Box<dyn FnOnce() -> A + 'a>` rather than using a named inner type or trait alias. This is clean, but it means `Thunk` is inherently `!Clone` (since `FnOnce` closures cannot be cloned through a trait object). This is correctly documented and is the right call; clonable lazy values are the domain of `Lazy`.

---

## 2. HKT Support

### Brand and Kind

```rust
impl_kind! {
    for ThunkBrand {
        type Of<'a, A: 'a>: 'a = Thunk<'a, A>;
    }
}
```

This is correct. The kind signature `type Of<'a, A: 'a>: 'a` maps to `Kind_cdc7cd43dac7585f`, which is the standard kind for lifetime-parameterized single-argument type constructors. The `A: 'a` bound ensures the inner value does not outlive the thunk's own lifetime, and `: 'a` on the output ensures the `Thunk` itself is bounded by `'a`. This matches the pattern used by all other `'a`-parameterized types in the library.

### ThunkBrand

Defined in `brands.rs` as:

```rust
pub struct ThunkBrand;
```

Zero-sized marker type, as expected. The documentation correctly notes that this is for `Thunk<'a, A>` and NOT for `Trampoline<A>` (which cannot implement HKT due to its `'static` requirement).

### Correctness

The HKT machinery is correctly integrated. All trait implementations use the `Apply!` macro to expand `<Self as Kind!(...)>::Of<'a, A>` into concrete `Thunk<'a, A>` types. This is consistent with the rest of the library.

---

## 3. Type Class Implementations

### Implemented

| Trait | Correct? | Notes |
|---|---|---|
| `Functor` | Yes | Delegates to inherent `map`. Uses `Fn` (not `FnOnce`) per the trait requirement. |
| `Pointed` | Yes | Wraps value in `Thunk::pure`. |
| `Semiapplicative` | Yes | Implemented via `bind` + `map`. |
| `Lift` | Yes | `lift2` via `bind` + `map`. Requires `Clone` on `A` and `B` per the trait. |
| `ApplyFirst` | Yes | Blanket (empty) impl, inherits from `Lift`. |
| `ApplySecond` | Yes | Same as above. |
| `Semimonad` | Yes | Delegates to inherent `bind`. The trait requires `Fn`, not `FnOnce`; the implementation correctly accepts `Fn`. |
| `Monad` | Yes (blanket) | Automatically derived from `Applicative + Semimonad`. |
| `Applicative` | Yes (blanket) | Automatically derived from `Pointed + Semiapplicative + ApplyFirst + ApplySecond`. |
| `MonadRec` | Yes | Iterative loop implementation. Stack-safe for the loop itself. |
| `Evaluable` | Yes | Delegates to inherent `evaluate`. |
| `Foldable` | Yes | Single-element fold; evaluates and applies function. |
| `FoldableWithIndex` | Yes | Index is `()`. |
| `FunctorWithIndex` | Yes | Index is `()`. |
| `WithIndex` | Yes | `type Index = ()`. |
| `Deferrable<'a>` | Yes | Delegates to `Thunk::defer`. |
| `Semigroup` | Yes | Combines results with inner `Semigroup::append`. |
| `Monoid` | Yes | Produces `Monoid::empty()`. |
| `Debug` | Yes | Prints `"Thunk(<unevaluated>)"` without forcing evaluation. |

### Intentionally Not Implemented

| Trait | Why | Correct Decision? |
|---|---|---|
| `Traversable` | Requires `Self::Of<'a, B>: Clone`, and `Thunk` is `!Clone` because `Box<dyn FnOnce>` cannot be cloned. | Yes. Well-documented in the struct's doc comment. |
| `TraversableWithIndex` | Same `Clone` constraint as `Traversable`. | Yes. |
| `Alt` | `Alt` provides choice between two values. For a single-element container, `Alt` would be trivially "pick one" with no meaningful semantics (unlike `Option` where `None` is meaningful). | Reasonable omission. |
| `Plus` | Requires `Alt`. | Follows from above. |
| `Compactable` / `Filterable` / `Witherable` | These concern filtering elements out of a structure. A single-element container that cannot be empty has no meaningful filtering. | Correct. |
| `Extend` / `Comonad` | These traits do not exist in the library. If they did, `Thunk` would be a candidate (PureScript's `Lazy` implements both). | N/A; see Section 4. |
| `Eq` / `Ord` / `Show` | PureScript implements these by forcing evaluation. In Rust, `Thunk` consumes itself on evaluate, so `Eq`/`Ord` cannot be implemented (they take `&self`). `Display` has the same problem. | Correct constraint of the type system. |

### Potential Missing Implementations

1. **`NaturalTransformation` from `ThunkBrand` to `OptionBrand`/`IdentityBrand`**: The `NaturalTransformation` trait exists and the documentation even shows a `ThunkToOption` example. But this would be a user-defined instance, not something the library provides. Not an issue.

2. **`Semiring` / `Ring` etc.**: PureScript implements `Semiring`, `Ring`, `EuclideanRing` for `Lazy a` when `a` has those instances. Since `Thunk` consumes itself on evaluation, these arithmetic traits cannot be meaningfully implemented (they'd need to evaluate both operands, consuming them). The `Semigroup`/`Monoid` implementations work because `append` takes `self` by value, which aligns with `Thunk`'s single-shot nature. The PureScript versions work because `Lazy` is memoized and can be forced multiple times. This is a correct divergence.

---

## 4. Comparison to PureScript's Lazy

PureScript's `Data.Lazy` is fundamentally different from `Thunk`:

| Aspect | PureScript `Lazy a` | Rust `Thunk<'a, A>` |
|---|---|---|
| Memoization | Yes (computed at most once) | No (single-shot, consumed on eval) |
| Multiple evaluations | Yes (returns cached result) | No (`evaluate` takes `self`) |
| Comonad | Yes (`extract = force`) | No (no Comonad trait in library) |
| Extend | Yes (`extend f x = defer \_ -> f x`) | No (no Extend trait in library) |
| Traversable | Yes (wraps result in `defer <<< const`) | No (`Thunk` is `!Clone`) |
| `Eq`/`Ord`/`Show` | Yes (forces evaluation) | No (`evaluate` consumes) |
| `Semiring`/`Ring` | Yes (defers arithmetic) | No (single-shot) |
| `Lazy (Lazy a)` | Yes (via `Control.Lazy`) | Analogous to `Deferrable` |

### Does the Divergence Make Sense?

Yes. PureScript's `Lazy` is semantically closer to Rust's `RcLazy`/`ArcLazy` (memoized, shared, multiple access). Rust's `Thunk` fills a different niche: a lightweight, non-allocating-beyond-the-box deferred computation that supports HKT. The naming could potentially cause confusion for PureScript users, but the documentation is clear about the distinction.

The library's `Lazy` type is the true analog to PureScript's `Lazy`, with memoization via `Rc<LazyCell<...>>` or `Arc<LazyLock<...>>`. `Thunk` is more like PureScript's raw `Unit -> a` functions but wrapped in a newtype for HKT support.

### Missing from PureScript That Could Be Added

- **Comonad/Extend**: If these traits were added to the library, `Thunk` could not implement them (Comonad's `extract :: w a -> a` takes `&self` in typical Rust encodings, but `Thunk::evaluate` takes `self`). However, `Lazy` (the memoized type) could implement them, matching PureScript's `Lazy`.
- **`fix`**: PureScript's `Data.Lazy` does not define `fix`; it comes from `Control.Lazy`. The Rust library correctly documents why `fix` is not on `Deferrable` (requires shared ownership) and provides `rc_lazy_fix`/`arc_lazy_fix` as concrete alternatives.

---

## 5. Lifetime Handling

### Correctness

The `'a` lifetime is threaded correctly throughout:

- `Thunk<'a, A>` holds `Box<dyn FnOnce() -> A + 'a>`, so the closure can borrow data with lifetime `'a`.
- All inherent methods (`new`, `map`, `bind`, `defer`) bound the closure to `'a`.
- The `Kind` impl uses `type Of<'a, A: 'a>: 'a`, correctly binding both the inner type and the thunk itself.
- All HKT trait methods use `'a` as the lifetime parameter, which the macro system propagates.

### The `'static` Boundary

Conversions between `Thunk` and `Trampoline` correctly require `'static`:

```rust
impl<A: 'static> From<Thunk<'static, A>> for Trampoline<A>
impl<A: 'static> From<Trampoline<A>> for Thunk<'static, A>
```

This is necessary because `Trampoline` is `Free<ThunkBrand, A>` with a `'static` requirement.

### Test Coverage for Borrowing

The `test_borrowing` test verifies that `Thunk` can capture stack references:

```rust
let x = 42;
let thunk = Thunk::new(|| &x);
assert_eq!(thunk.evaluate(), &42);
```

This is a key differentiator from `Trampoline` and is well-tested.

---

## 6. Stack Safety

### `bind` Chains: Not Stack-Safe

`Thunk::bind` creates a new `Thunk` whose closure calls both the original thunk and the continuation:

```rust
pub fn bind<B: 'a>(self, f: impl FnOnce(A) -> Thunk<'a, B> + 'a) -> Thunk<'a, B> {
    Thunk::new(move || {
        let a = (self.0)();
        let thunk_b = f(a);
        (thunk_b.0)()
    })
}
```

Each `bind` nests one more closure invocation inside the previous one. A chain of N `bind` calls creates N stack frames when evaluated. This is clearly documented as a limitation.

### `tail_rec_m`: Stack-Safe

The `MonadRec` implementation uses an iterative loop:

```rust
fn tail_rec_m<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> Thunk<'a, Step<A, B>> + 'a,
    a: A,
) -> Thunk<'a, B> {
    Thunk::new(move || {
        let mut current = a;
        loop {
            match f(current).evaluate() {
                Step::Loop(next) => current = next,
                Step::Done(res) => break res,
            }
        }
    })
}
```

This is correctly stack-safe: the loop evaluates each step's thunk and destructures the result without recursive calls. The caveat, correctly documented, is that if `f` itself builds deep `bind` chains inside the returned thunk, those chains are not stack-safe.

### Test Coverage

The test `test_tail_rec_m_stack_safety` runs 200,000 iterations, verifying that the iterative loop does not overflow. This is a good test.

### Remaining Risk

There is no test for the failure mode: a deep `bind` chain that overflows the stack. Such a test would be valuable as a regression guard (using `#[should_panic]` or a compile-fail test), though it's difficult to make reliable across platforms since stack sizes vary.

---

## 7. Ergonomics

### Strengths

1. **Dual API**: Inherent methods (`map`, `bind`) accept `FnOnce` for maximum flexibility; HKT-level traits (`Functor::map`, `Semimonad::bind`) accept `Fn` for generality. This is well-documented on each method.
2. **Fluent chaining**: `Thunk::pure(x).map(f).bind(g).evaluate()` reads naturally.
3. **Conversion ecosystem**: `From` impls connect `Thunk` to `Lazy`, `SendThunk`, and `Trampoline`, forming a coherent lazy type hierarchy.
4. **`into_rc_lazy` / `into_arc_lazy`**: Convenient bridge to memoized values. The `into_arc_lazy` method correctly evaluates eagerly (since `Thunk` is `!Send`) and documents why.

### Weaknesses

1. **`Fn` vs `FnOnce` confusion**: The Semimonad trait requires `Fn`, but `Thunk::bind` (inherent) accepts `FnOnce`. When using HKT-polymorphic code, users must provide `Fn` closures even though `Thunk` only calls them once. This is a fundamental tension in the library's design, not specific to `Thunk`, but it matters most here since `FnOnce` is the natural fit.

2. **No `and_then` alias**: Rust users expect `and_then` for monadic bind (from `Option`, `Result`, `Future`). The method is called `bind`, which is the FP-standard name but may trip up Rustaceans. This is a deliberate library-wide choice, so consistency trumps familiarity.

3. **No `map_or` / `unwrap_or` / fallback combinators**: Since `Thunk` always contains a value (never empty), these are less critical. But an `evaluate_with` or `or_else` could be useful for error-prone closures, though that domain is covered by `TryThunk`.

4. **`evaluate` consumes `self`**: This is correct for the semantics (single-shot computation), but it means you cannot evaluate a thunk and then do anything else with it. The documentation addresses this by pointing to `into_rc_lazy` for memoization.

---

## 8. Documentation Quality

### Strengths

- The struct-level doc comment is comprehensive, with a comparison table vs `Trampoline`, algebraic properties, stack safety notes, and an explicit list of which typeclasses are implemented and which are not (with reasons).
- Every method uses the library's custom documentation macros (`#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`).
- All doc examples include `assert!` statements and would be run as doc tests.
- The `Traversable` non-implementation is explicitly documented with a clear rationale.

### Weaknesses

1. **`bind` inherent method vs HKT `bind`**: The doc comments on the inherent `bind` and `map` explain the `FnOnce` vs `Fn` difference well, but a cross-link to the trait method (e.g., "See also [`Semimonad::bind`] for the HKT-level version") is missing from the trait impl direction.

2. **Module-level docs**: The module doc is a single line: "Deferred, non-memoized computation with higher-kinded type support." This could benefit from a brief usage example showing `Thunk::new`, chaining, and evaluation.

3. **`Debug` formatting**: The `Debug` impl prints `"Thunk(<unevaluated>)"` which is fine, but the doc comment says "Formats the thunk without evaluating it" without mentioning that this is always the output (it never shows the value, since there's no way to peek without consuming).

---

## 9. Issues, Limitations, and Design Flaws

### Issues

1. **`Semimonad::bind` requires `Fn`, but Thunk is single-element**: The HKT `bind` implementation on line 663 accepts `func: impl Fn(A) -> ...` and passes it to `ma.bind(func)`. The inherent `bind` accepts `FnOnce`. This works because `Fn` is a supertrait of `FnOnce`, so an `Fn` closure can always be used where `FnOnce` is expected. No bug here, just a mismatch in generality that is inherent to the HKT design.

2. **`Lift::lift2` requires `Clone` on `A` and `B`**: The trait signature requires `A: Clone + 'a, B: Clone + 'a`. For `Thunk`, since there is exactly one element, cloning is never actually needed. But the trait definition must accommodate multi-element structures like `Vec`. This means users calling `lift2::<ThunkBrand, ...>` must satisfy `Clone` bounds even when it is theoretically unnecessary. This is a known trade-off of the unified HKT approach.

3. **`Semiapplicative::apply` requires `Clone` on `A`**: Same issue as `lift2`. The `Clone` bound exists for multi-element containers but is unnecessary for single-element `Thunk`.

### Limitations

1. **Not `Clone`**: `Box<dyn FnOnce>` cannot be cloned. This prevents `Traversable`, `Eq`, `Ord`, and multiple evaluations. This is an inherent limitation of the `FnOnce` choice and is the correct trade-off.

2. **Not `Send`**: The inner closure is `dyn FnOnce() -> A + 'a` without a `Send` bound. `SendThunk` exists for this purpose. The separation is clean.

3. **Stack depth for `bind` chains**: Already discussed. The documentation correctly directs users to `Trampoline` for deep recursion.

4. **`Foldable` requires `Clone` on `A`**: The `Foldable` trait requires `A: Clone + 'a` in `fold_right` and `fold_left`. For `Thunk`, the value is used exactly once, so `Clone` is unnecessary. But the trait signature is fixed for multi-element containers. Same trade-off as `Lift::lift2`.

### Design Flaws

None identified. The type is well-designed for its stated purpose.

---

## 10. Alternatives and Improvements

### Potential Improvements

1. **`into_lazy` generic over config**: Currently there are two separate methods, `into_rc_lazy` and `into_arc_lazy`. A generic `into_lazy<Config: LazyConfig>` method could unify these, though the `Send` restriction on `into_arc_lazy` (which requires eager evaluation) makes a single generic method's semantics unclear.

2. **`zip` combinator**: A `zip(self, other: Thunk<'a, B>) -> Thunk<'a, (A, B)>` method would be convenient for combining two thunks without going through `lift2`. This is a minor ergonomic addition.

3. **`flatten` combinator**: `Thunk<'a, Thunk<'a, A>> -> Thunk<'a, A>` is a standard monadic operation. It is expressible as `.bind(identity)`, but a named method could improve readability. Some Rust types provide `flatten` (e.g., `Option::flatten`).

4. **Consider `impl FnOnce` for the HKT `Semimonad::bind`**: This would be a library-wide change to the `Semimonad` trait. It would break `Vec`'s implementation (which calls `f` multiple times) but would better serve single-element types. This is a fundamental design tension; the current `Fn` choice is correct for the library as a whole.

5. **Stack overflow test**: Add a test that demonstrates `bind`-chain stack overflow at a known depth (or at least documents the approximate limit). This would serve as both a regression test and documentation for users.

### Alternative Designs Rejected

1. **Memoizing Thunk**: Adding memoization would duplicate `Lazy`. The current non-memoizing design keeps the types orthogonal.

2. **Enum with eager/lazy variants**: Adding a `Value(A)` variant for `pure` would avoid a trivial closure allocation, but would complicate every combinator. The optimizer likely handles this already, and the simpler representation reduces maintenance burden.

3. **Generic over closure type**: Making `Thunk<'a, A, F: FnOnce() -> A>` generic would allow monomorphization but break HKT (since `Kind::Of` must be a single type). The current `dyn FnOnce` is the right choice for HKT support.

---

## Summary of Findings

`Thunk` is a well-designed, correctly implemented type that fills a clear niche in the lazy evaluation hierarchy: the only lazy type with full HKT support, at the cost of not being `Clone`, `Send`, or stack-safe for `bind` chains. Its documentation is thorough, its test suite covers both basic functionality and algebraic laws (via QuickCheck), and its conversions to other lazy types form a coherent ecosystem.

The main tension points are all inherent to the unified HKT design rather than specific to `Thunk`:
- `Fn` vs `FnOnce` in trait signatures.
- `Clone` bounds required by multi-element containers but unnecessary for single-element types.

No bugs or correctness issues were found.
