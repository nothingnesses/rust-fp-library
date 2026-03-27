# Analysis: `Thunk<'a, A>` (`fp-library/src/types/thunk.rs`)

## 1. Design

### Overview

`Thunk<'a, A>` is a non-memoized, lifetime-polymorphic deferred computation wrapping `Box<dyn FnOnce() -> A + 'a>`. It serves as the primary workhorse for lazy evaluation with full HKT support in the library.

### Comparison to PureScript's `Data.Lazy`

PureScript's `Lazy a` is a **memoized** type: it evaluates at most once and caches the result. Rust's `Thunk` is explicitly **not memoized**; it is consumed on evaluation (`evaluate` takes `self` by value). This is a deliberate and well-documented divergence.

The naming is potentially confusing: PureScript's `Lazy` maps to Rust's `RcLazy`/`ArcLazy`, while Rust's `Thunk` has no direct PureScript counterpart (PureScript's `defer` always produces a memoized `Lazy`). The module documentation at line 3 correctly positions `Thunk` relative to `Lazy` and `Trampoline`, which mitigates this.

**Type class coverage comparison:**

| PureScript `Lazy` | Rust `Thunk` | Notes |
|---|---|---|
| `Functor` | `Functor` | Present. |
| `Apply` | `Semiapplicative` | Present (via `CloneableFn`). |
| `Applicative` | `Applicative` (blanket) | Present. |
| `Bind` | `Semimonad` | Present. |
| `Monad` | `Monad` (blanket) | Present. |
| `Foldable` | `Foldable` | Present. |
| `Traversable` | N/A | Cannot implement; documented at lines 90-101. |
| `FoldableWithIndex Unit` | `FoldableWithIndex` (Index = `()`) | Present. |
| `FunctorWithIndex Unit` | `FunctorWithIndex` (Index = `()`) | Present. |
| `TraversableWithIndex Unit` | N/A | Cannot implement (same `Traversable` limitation). |
| `Foldable1` | N/A | No `Foldable1` trait in the library. |
| `Traversable1` | N/A | No `Traversable1` trait in the library. |
| `Extend` | N/A | No `Extend`/`Comonad` traits in the library. |
| `Comonad` | N/A | No `Extend`/`Comonad` traits in the library. |
| `Semigroup a => Semigroup (Lazy a)` | `Semigroup` | Present (line 916). |
| `Monoid a => Monoid (Lazy a)` | `Monoid` | Present (line 950). |
| `Semiring`, `Ring`, etc. | N/A | No types in this library implement these algebraic traits on wrapper types. Consistent omission. |
| `Eq`, `Ord` | N/A | Cannot implement without forcing evaluation, which would violate Rust's expectation that `Eq`/`Ord` are pure. Also, `Thunk` is consumed on evaluation, so comparison would destroy the value. Correct to omit. |
| `Show` (forces evaluation) | `Debug` (does not force) | PureScript's `Show` forces evaluation and prints the value. Rust's `Debug` impl at line 993 prints `"Thunk(<unevaluated>)"` without forcing, which is the right choice for Rust since `Debug` should be side-effect-free. |
| `Lazy` (the class, not the type) | `Deferrable` | Present (line 415). |

### Design verdict

The design is sound. `Thunk` fills a clear niche: lifetime-polymorphic deferred computation with full HKT support, at the cost of no memoization and no stack safety (except via `tail_rec_m`). The trade-off table in the doc comment (lines 59-67) is accurate and helpful.

## 2. Implementation Quality

### Correctness

The core operations are correct:

- **`new`** (line 134): Straightforward box wrapping. Correct.
- **`pure`** (line 158): Wraps a value in a closure. The `A: 'a` bound is redundant given `impl<'a, A: 'a>` on the `impl` block (line 117), but harmless and makes the contract explicit.
- **`defer`** (line 184): Flattens `Thunk<Thunk<A>>` into `Thunk<A>` by chaining evaluation. Correct.
- **`bind`** (line 217): Evaluates `self`, passes result to `f`, evaluates the result. Correct.
- **`map`** (line 256): Evaluates and transforms. Correct.
- **`evaluate`** (line 280): Calls the inner closure. Correct.

### Potential issues

1. **`Semimonad::bind` accepts `Fn` but `Thunk` only evaluates once.** The HKT `Semimonad::bind` trait requires `impl Fn(A) -> ...` (line 61 of `semimonad.rs`), which is more restrictive than needed for `Thunk` since a single-element container only calls the function once. The inherent `Thunk::bind` correctly accepts `FnOnce`. This is not a bug, just a consequence of the uniform trait signature. The documentation at lines 193-196 correctly explains this.

2. **`Lift::lift2` does not need `Clone` for `Thunk`.** The `Lift::lift2` implementation (line 542) has trait bounds `A: Clone + 'a, B: Clone + 'a` inherited from the trait signature. For `Thunk`, these bounds are unnecessary since each value is used exactly once. However, since the trait mandates these bounds for general use (e.g., `Vec` needs `Clone`), this is an inherent limitation of the trait design, not a bug in `Thunk`.

3. **`From<Lazy<'a, A, Config>>` clones on every evaluation.** The `From<Lazy>` impl (line 360) creates a thunk that calls `lazy.evaluate().clone()`. Since `Lazy::evaluate()` returns `&A`, cloning is necessary. The documentation at line 347 correctly notes this. No issue.

4. **No panics or unsafe code.** The implementation is entirely safe.

### Edge cases

- **Zero-sized types:** Work correctly since `Box<dyn FnOnce() -> ()>` is valid.
- **Recursive `bind` chains:** Will overflow the stack as documented. `tail_rec_m` is the escape hatch.
- **`into_arc_lazy` evaluates eagerly** (line 327-332): Correctly documented at lines 306-312. The thunk's closure is `!Send`, so it must be evaluated before crossing into `Arc<LazyLock<...>>`.

## 3. Type Class Instances

### Implemented (all correct and lawful)

| Instance | Lines | Lawfulness |
|---|---|---|
| `Functor` | 443-477 | Correct. Identity and composition laws hold trivially for a single-element container. QuickCheck tests at lines 1170-1184. |
| `Pointed` | 479-507 | Correct. `pure(a).evaluate() == a`. |
| `Lift` | 509-553 | Correct. Delegates to `bind`+`map`. |
| `ApplyFirst`, `ApplySecond` | 555-556 | Blanket impls; correct. |
| `Semiapplicative` | 558-601 | Correct. `apply(ff, fa)` evaluates `ff` to get the function, then maps it over `fa`. |
| `Semimonad` | 603-637 | Correct. Delegates to inherent `bind`. QuickCheck tests at lines 1188-1216. |
| `MonadRec` | 639-690 | Correct. Uses an iterative loop (lines 681-688), making `tail_rec_m` stack-safe as long as each step produces a shallow thunk. Well-documented caveat at lines 642-645. Stack safety test at line 1342. |
| `Evaluable` | 692-723 | Correct. Delegates to `Thunk::evaluate`. |
| `Foldable` | 725-839 | Correct. All three methods (`fold_right`, `fold_left`, `fold_map`) evaluate the thunk and apply the function. Trivially lawful for a single-element container. |
| `WithIndex` (Index = `()`) | 841-843 | Correct. Unit index for a single-element container matches PureScript convention. |
| `FunctorWithIndex` | 845-876 | Correct. Passes `()` as the index. |
| `FoldableWithIndex` | 878-910 | Correct. Passes `()` as the index. |
| `Semigroup` | 916-944 | Correct. Defers the append. QuickCheck associativity test at line 1222. |
| `Monoid` | 950-970 | Correct. Defers the empty. QuickCheck identity tests at lines 1240-1254. |
| `Deferrable` | 415-441 | Correct. Delegates to `Thunk::defer`. |
| `Debug` | 977-995 | Correct. Does not force evaluation. |
| `Applicative` (blanket) | N/A | Obtained via `Pointed + Semiapplicative + ApplyFirst + ApplySecond`. |
| `Monad` (blanket) | N/A | Obtained via `Applicative + Semimonad`. |

### Missing type class instances

1. **`Traversable` / `TraversableWithIndex`:** Cannot be implemented. `Traversable::traverse` requires `Self::Of<'a, B>: Clone` (line 105 of `traversable.rs`), and `Thunk` wraps `Box<dyn FnOnce()>` which is not `Clone`. Correctly documented at lines 90-101 of `thunk.rs`.

2. **`Extend` / `Comonad`:** PureScript's `Lazy` implements both. `Extend` would be `extend f x = defer \_ -> f x`, and `Comonad` would be `extract = force`. These are straightforward for `Thunk`:
   - `extend(f, thunk)` = `Thunk::new(move || f(thunk))` (the function receives the whole thunk and extracts what it needs).
   - `extract(thunk)` = `thunk.evaluate()`.
   However, there are no `Extend`/`Comonad` traits in this library, so this is not actionable.

3. **`Alt` / `Plus`:** Not implemented in PureScript's `Lazy` either. Not applicable.

4. **`NaturalTransformation` from `ThunkBrand` to other brands:** The `From` conversions (`Thunk` <-> `Trampoline`, `Thunk` -> `Lazy`) serve this role at the value level. A formal `NaturalTransformation` instance could be added but is not essential.

## 4. API Surface

### Public inherent methods

| Method | Takes | Notes |
|---|---|---|
| `new(impl FnOnce() -> A + 'a)` | Constructor | Good. |
| `pure(A)` | Constructor | Good. Mirrors `Pointed::pure`. |
| `defer(impl FnOnce() -> Thunk<'a, A> + 'a)` | Constructor | Good. Mirrors `Deferrable::defer`. |
| `bind<B>(self, impl FnOnce(A) -> Thunk<'a, B> + 'a)` | `FnOnce` | Good. More permissive than HKT version. |
| `map<B>(self, impl FnOnce(A) -> B + 'a)` | `FnOnce` | Good. More permissive than HKT version. |
| `evaluate(self)` | Consumes | Good. |
| `into_rc_lazy(self)` | Consumes | Good. Lazy memoization. |
| `into_arc_lazy(self)` | Consumes + `A: Send + Sync` | Good. Documented eager evaluation. |

### Conversions (`From` impls)

| From | To | Notes |
|---|---|---|
| `Lazy<'a, A, Config>` | `Thunk<'a, A>` | Requires `A: Clone`. Correct. |
| `Trampoline<A>` | `Thunk<'static, A>` | Correct. |
| `Thunk<'static, A>` | `Trampoline<A>` | Correct. |

### Missing or potentially useful additions

1. **`from_value` / `eager`:** An alias for `pure` that makes intent clearer. Minor; `pure` already serves this purpose.

2. **`and_then`:** An alias for `bind`. Some Rust users expect `and_then` (used by `Option`, `Result`, `Future`). However, `bind` is the standard FP name and consistent with the library's style.

3. **`From<SendThunk<'a, A>> for Thunk<'a, A>`:** A `SendThunk` is strictly more constrained than a `Thunk` (its closure is `Send`), so this conversion is sound and would enable easy interop. Currently missing.

4. **`is_evaluated` / state inspection:** Not possible without interior mutability, since `Thunk` wraps `FnOnce` and has no state to inspect. Correct to omit.

## 5. Consistency

### With `SendThunk`

`SendThunk` mirrors the API closely: `new`, `pure`, `defer`, `bind`, `map`, `evaluate`, `into_rc_lazy`, `into_arc_lazy`, `Semigroup`, `Monoid`, `Deferrable`, `Debug`. The key difference is that `SendThunk` cannot implement HKT traits (no `Functor`, `Semimonad`, etc.) due to missing `Send` bounds in trait signatures. This asymmetry is well-documented in `SendThunk`'s module docs.

### With `TryThunk`

`TryThunk<'a, A, E>` wraps `Thunk<'a, Result<A, E>>` and adds error-handling methods. Consistent layering.

### With `Trampoline`

`Trampoline` is stack-safe but requires `'static`. The `From` conversions between `Thunk` and `Trampoline` correctly enforce `'static`. Consistent.

### With `Lazy` (`RcLazy` / `ArcLazy`)

`Lazy` is memoized and returns `&A` from `evaluate`. `Thunk` is non-memoized and returns `A` from `evaluate`. The `into_rc_lazy` / `into_arc_lazy` methods and `From` impls provide clean interop. Consistent.

### Naming conventions

All methods and trait implementations follow the library's conventions. Documentation uses the standard `document_signature`, `document_type_parameters`, `document_parameters`, `document_returns`, `document_examples` macro suite consistently.

## 6. Limitations and Issues

### Known limitations (correctly documented)

1. **No memoization.** Each evaluation re-runs the closure chain. `into_rc_lazy` is the escape hatch.
2. **Not stack-safe.** Deep `bind` chains overflow. `tail_rec_m` or `Trampoline` are the alternatives.
3. **Not `Send`.** The inner closure is `Box<dyn FnOnce() -> A + 'a>` without `Send`. `SendThunk` is the thread-safe counterpart.
4. **Cannot implement `Traversable`.** Requires `Clone` which `Box<dyn FnOnce()>` cannot satisfy.
5. **`into_arc_lazy` evaluates eagerly.** Because the closure is `!Send`.

### Undocumented or subtle issues

1. **`Foldable` and `FoldableWithIndex` have unused `FnBrand` parameter.** The `Foldable` trait methods (e.g., `fold_right` at line 755) take a `FnBrand: CloneableFn` type parameter that is completely unused in the `Thunk` implementation. This is a trait-level concern, not specific to `Thunk`, but it means callers must specify a dummy `FnBrand` even for types that do not need it.

2. **`MonadRec::tail_rec_m` requires `Clone` on `f`.** The trait requires `f: impl Fn(A) -> ... + Clone + 'a` (line 676). For `Thunk`, the step function is only called iteratively (not concurrently), so `Clone` is not strictly needed. This is a trait-level constraint inherited from the general `MonadRec` design.

3. **Memory allocation on every `map`/`bind`.** Each `map` or `bind` call allocates a new `Box` for the composed closure. For long chains, this creates a linked list of heap allocations. This is an inherent cost of the `Box<dyn FnOnce()>` representation and is acceptable for the intended use case (glue code, short chains).

## 7. Documentation

### Accuracy

The documentation is accurate throughout. Key claims verified:

- The trade-off table (lines 59-67) correctly describes HKT support, stack safety, lifetime, and thread safety for `Thunk` vs `Trampoline`.
- The `Traversable` limitation explanation (lines 90-101) correctly identifies the `Clone` requirement as the blocker.
- The stack safety warnings on `bind` (lines 78-86) are accurate.
- The `into_arc_lazy` eager evaluation explanation (lines 306-312) is correct and well-reasoned.
- The `FnOnce` vs `Fn` explanations on inherent methods (lines 193-196, 230-235) are accurate.

### Completeness

The documentation is thorough. Every public method has:
- A short description.
- Type parameter documentation.
- Parameter documentation.
- Return value documentation.
- A working code example.

The struct-level documentation includes:
- A description of what `Thunk` is and is not (memoization).
- HKT representation.
- A trade-off comparison table.
- Algebraic properties (monad laws).
- Stack safety caveats.
- Limitations with rationale.
- List of implemented type classes.

### Minor documentation issues

1. **Line 62, table formatting:** The right column of the `Trampoline` row has an extra space before the closing `|`:
   ```
   |------------------------------ |
   ```
   This creates a slightly misaligned table in rendered markdown. Should be `|------------------------------|`.

2. **Line 63-64, emoji in documentation:** The table uses emoji characters (check marks, warning sign, x marks). While these render well in most contexts, they may not display correctly in all terminals. This is a minor style concern.

3. **`Evaluable` documentation (line 693):** Says "Runs the eval, producing the inner value." The word "eval" is informal; "thunk" or "computation" would be more precise and consistent with the rest of the documentation.

### Test coverage

The test suite is comprehensive:
- Basic operations: `new`, `pure`, `map`, `bind`, `defer`, `evaluate` (lines 1017-1067).
- Conversions: `From<Lazy>`, `From<Trampoline>`, `Into<Trampoline>` (lines 1069-1163).
- `Semigroup`/`Monoid` (lines 1078-1102).
- QuickCheck property tests for Functor, Monad, Semigroup, and Monoid laws (lines 1165-1254).
- HKT-level trait tests: `Foldable`, `Lift`, `Semiapplicative`, `Evaluable` (lines 1256-1290).
- Memoization tests for `into_rc_lazy` and `into_arc_lazy` (lines 1292-1335).
- Stack safety for `tail_rec_m` (lines 1337-1360).
- `FunctorWithIndex` and `FoldableWithIndex` (lines 1362-1452).

No significant gaps in test coverage.

## Summary

`Thunk<'a, A>` is a well-designed, well-implemented, and thoroughly documented type. It fills a clear role in the lazy evaluation hierarchy as the lifetime-polymorphic, HKT-compatible deferred computation type. The implementation is correct, the type class instances are lawful, and the limitations are accurately documented. The main areas for potential improvement are minor: a table formatting issue in the doc comment, an informal word choice in the `Evaluable` doc, and a missing `From<SendThunk>` conversion.
