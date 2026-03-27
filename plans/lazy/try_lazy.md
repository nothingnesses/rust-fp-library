# Analysis: `try_lazy.rs` (Fallible Memoized Lazy Types)

**File:** `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/try_lazy.rs`
**Lines:** 2462 (including tests)

## 1. Design

The design is sound and well-motivated. `TryLazy<'a, A, E, Config>` wraps a `Config::TryLazy<'a, A, E>` (line 96-102), which resolves to `Rc<LazyCell<Result<A, E>, ...>>` or `Arc<LazyLock<Result<A, E>, ...>>` depending on the config. This is the natural fallible extension of `Lazy`, which wraps `Config::Lazy<'a, A>`.

Key design decisions that make sense:

- **Config-parameterized:** Reuses the same `TryLazyConfig` trait (defined in `lazy.rs`, lines 136-207) to abstract over `Rc`/`Arc` pointer choice, mirroring `Lazy`'s `LazyConfig`.
- **`evaluate` returns `Result<&A, &E>`:** Since the `Result` is cached behind a shared pointer, returning references to both `Ok` and `Err` variants is correct (line 167-169).
- **`map`/`map_err` receive `&A`/`&E`:** Because the cached value is behind a shared pointer, the mapping functions receive references. This is well-documented (lines 243-252, 283-292).
- **Clone requirements on the "other" side:** `map` requires `E: Clone` and `map_err` requires `A: Clone` because the new cell must own its complete `Result<B, E>` or `Result<A, E2>`. This is correctly documented with "Why `E: Clone`?" and "Why `A: Clone`?" sections.
- **Module-level documentation** (lines 1-33) clearly explains the three design choices (`TryLazy` vs `Lazy<Result<A, E>>` vs `Result<Lazy, E>`), the naming rationale for `map` vs `ref_map`, and the `Foldable` error-discarding behavior.

## 2. Implementation Quality

### Correctness

No bugs found. The implementation is straightforward and correct:

- **Memoization works correctly:** The underlying `LazyCell`/`LazyLock` handles the once-only evaluation. Tests at lines 1616-1667 verify caching for both `Ok` and `Err`, and sharing across clones.
- **`map`/`map_err` create new cells:** Each combinator creates a fresh `TryLazy` that captures `self` and evaluates lazily (lines 277-281, 317-321). This correctly builds a cache chain.
- **`and_then`/`or_else` are correct:** `and_then` (lines 659-668) short-circuits on `Err`, `or_else` (lines 690-699) short-circuits on `Ok`. Both clone the propagated value as required.
- **`Deferrable` for `RcLazyConfig`** (lines 1028-1063): Correctly flattens by evaluating the inner `TryLazy` and cloning the result.
- **`Deferrable` for `ArcLazyConfig`** (lines 1319-1354): Calls `f()` eagerly, matching the `Lazy` pattern (line 887). The documentation correctly explains why: `Deferrable::defer` lacks a `Send` bound on the thunk, so the thunk cannot be stored inside `ArcTryLazy::new`.
- **`SendDeferrable` for `ArcLazyConfig`** (lines 1362-1397): Correctly wraps in a new cell since `Send` is available.

### Potential Issue: `Deferrable` for `ArcTryLazy` eagerly evaluates the thunk

The `Deferrable<'a>` impl for `ArcTryLazy` (line 1353) just calls `f()`, which means `defer(|| ...)` provides no laziness for the thunk itself. The inner `ArcTryLazy` retains its own lazy semantics, but the thunk `f` runs immediately. This is documented and matches the `Lazy` behavior. However, it means `Deferrable` for `ArcTryLazy` is semantically weaker than for `RcTryLazy`, where the thunk is genuinely deferred.

### Potential Issue: Clone bounds on `Deferrable` for `RcLazyConfig`

The `Deferrable` impl for `RcTryLazy` (lines 1028-1031) requires `A: Clone + 'a` and `E: Clone + 'a`. This is because the implementation calls `.evaluate()` which returns `Result<&A, &E>`, and then clones to construct a new owned `Result`. The `Lazy` `Deferrable` impl (line 819) does NOT require `Clone` because infallible `Lazy` can simply forward the evaluate reference. This is an inherent cost of the fallible design; it cannot be avoided without changing the underlying representation.

## 3. Type Class Instances

### Present Instances

| Type Class | `RcTryLazy` | `ArcTryLazy` | Notes |
|---|---|---|---|
| `RefFunctor` | Yes (line 1408) | N/A | Maps over success value |
| `SendRefFunctor` | N/A | Yes (line 1455) | Thread-safe variant |
| `Foldable` | Yes (line 1172) | Yes (line 1172) | Generic over `Config` |
| `Deferrable` | Yes (line 1028) | Yes (line 1319) | |
| `SendDeferrable` | N/A | Yes (line 1362) | |
| `Semigroup` | Yes (line 1078) | Yes (line 1126) | Short-circuits on first `Err` |
| `Monoid` | Yes (line 1506) | Yes (line 1533) | `empty` = `Ok(A::empty())` |

### HKT (`Kind`) Instance

`impl_kind!` at line 1066 maps `TryLazyBrand<E, Config>` to `TryLazy<'a, A, E, Config>`. Requires `E: 'static` due to the Brand pattern's type erasure constraint, which is correctly documented on the brand (brands.rs, lines 233-240).

### Missing Instances (Compared to `Lazy`)

`Lazy` implements no additional type classes beyond what `TryLazy` has (both implement `Foldable`, `RefFunctor`/`SendRefFunctor`, `Deferrable`/`SendDeferrable`, `Semigroup`, `Monoid`). The parallel is complete.

### Missing Instances (Compared to `TryThunk`)

`TryThunk` implements significantly more type classes: `Functor`, `FunctorWithIndex`, `Pointed`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Semimonad`, `Lift`, `MonadRec`, `Bifunctor`, `Bifoldable`, `FoldableWithIndex`, `WithIndex`. These are not applicable to `TryLazy` because:

1. `TryLazy` cannot implement `Functor` (it returns `&A`, not `A`); it uses `RefFunctor` instead.
2. `TryLazy` cannot implement `Monad`/`Semimonad` because `bind` would need to consume the memoized value, but it is behind a shared pointer.
3. `Bifunctor` would require both `Functor` and error-mapping through the HKT machinery, which conflicts with the reference-based design.

These absences are justified and consistent with `Lazy`'s own limitations.

## 4. API Surface

### Constructors

- `new(f)` (lines 199, 809): Standard constructor for `Rc` and `Arc` variants.
- `ok(a)` / `err(e)` (lines 219/239, 829/852): Convenience constructors for pre-computed values.
- `catch_unwind(f)` (lines 617, 1018): Catches panics, converting to `Err(String)`.
- `catch_unwind_with(f, handler)` (lines 578, 973): Custom panic handler.
- `From<TryThunk>` (lines 329/391): Conversion from non-memoized fallible thunk.
- `From<TryTrampoline>` (lines 360/415): Conversion from stack-safe fallible computation.
- `From<Lazy>` (lines 439/465): Lifts infallible lazy into fallible.
- `From<Result>` (lines 491/515): Wraps an already-computed result.

### Combinators

- `map(f)` (lines 271, 884): Maps success value.
- `map_err(f)` (lines 311, 920): Maps error value.
- `and_then(f)` (lines 659, 739): Monadic chaining on success.
- `or_else(f)` (lines 690, 770): Recovery on error.

### Observations

**Well-designed API.** The surface mirrors `Result`'s combinators (`map`, `map_err`, `and_then`, `or_else`), making it intuitive for Rust developers. The `ArcTryLazy` methods correctly require `Send` bounds on closures and `Send + Sync` on captured values.

**Missing `pure` method.** `Lazy` has `pure(a)` as an alias for `new(|| a)`, but `TryLazy` uses `ok(a)` instead. This is the right choice; `ok`/`err` mirror `Result::Ok`/`Result::Err` and are more idiomatic for a fallible type than `pure` would be.

**No `bimap` method.** There is no method that transforms both `A` and `E` in one pass. This would require `A: Clone + E: Clone`, but could be a useful convenience. Minor omission.

**No `unwrap`/`expect` methods.** Unlike `Result`, there are no panicking extractors. Users must match on `evaluate()`. This is arguably better for a lazy library type.

## 5. Consistency

### Consistency with `Lazy`

The design closely parallels `Lazy`:

- Same `Config`-parameterized struct (single newtype over `Config::TryLazy`).
- Same Rc/Arc split with matching `Send` bounds.
- Same `Deferrable` strategy (Rc: genuine deferral; Arc: eager `f()` call).
- Same `SendDeferrable` strategy (wraps in new cell).
- Same `RefFunctor`/`SendRefFunctor` split (Rc gets `RefFunctor`, Arc gets `SendRefFunctor`).
- Same `Semigroup`/`Monoid` pattern.
- Same `Foldable` pattern (generic over `Config`).
- Same `Debug` format: `"TryLazy(..)"` vs `"Lazy(..)"`.

One minor inconsistency: `Lazy` implements `Hash`, `PartialEq`, `Eq`, `PartialOrd`, `Ord` (by forcing evaluation and delegating), but `TryLazy` does not. This is likely intentional since equality for `Result<&A, &E>` is more complex, but it is worth noting.

### Consistency with `TryThunk`

- Both use `Result<A, E>` semantics.
- Both provide `map`, `map_err`, `and_then`, `or_else`.
- `TryThunk::evaluate()` returns `Result<A, E>` (owned), while `TryLazy::evaluate()` returns `Result<&A, &E>` (borrowed). This is the fundamental difference driving the API divergence.
- `TryThunk` has far more type class instances (Functor, Monad, Bifunctor, etc.) because it works with owned values.

### Consistency with `TrySendThunk` / `TryTrampoline`

- `From` conversions exist from `TryThunk` and `TryTrampoline` to `TryLazy` (both Rc and Arc variants), providing the expected upgrade paths.
- There are no `From<TrySendThunk>` conversions, which makes sense because `TrySendThunk` could convert directly to `ArcTryLazy` (since it is `Send`), but this is a minor gap.

## 6. Limitations

1. **`E: 'static` for HKT:** The brand requires `E: 'static` (line 1067), preventing borrowed error types in generic/HKT contexts. This is inherent to the Brand pattern and is well-documented.

2. **Clone overhead in combinators:** `map` clones `E` on error, `map_err` clones `A` on success, `and_then`/`or_else` clone both. This is unavoidable given the `&`-reference design but means chaining many combinators incurs repeated cloning.

3. **Cache chain memory retention:** Documented (lines 79-84). Each `map`/`map_err` keeps predecessor cells alive. No mitigation is provided (e.g., no `force_and_collapse` method).

4. **No `Functor`/`Monad` instances:** Cannot participate in generic HKT code that requires `Functor` or `Monad`. Only `RefFunctor`/`SendRefFunctor` and `Foldable` are available. This is inherent to the memoization design.

5. **No `Traversable` instance:** Same limitation as `Lazy`; memoized types cannot implement `Traversable` because traversal would need to consume or clone the cached value.

6. **No `Bifunctor` instance:** Cannot map both type parameters through HKT machinery simultaneously. The inherent method `map_err` provides error mapping, but not through a type class.

7. **Panic poisoning:** Documented (lines 146-153). If the initializer panics, the cell is poisoned and subsequent calls also panic. The `catch_unwind` family of constructors provides a mitigation path.

## 7. Documentation

Documentation quality is excellent:

- **Module docs** (lines 1-33): Clear explanation of `Foldable` error discarding, choosing between `TryLazy`/`Lazy<Result>`/`Result<Lazy>`, and `map` vs `ref_map` naming rationale.
- **Struct docs** (lines 66-84): `When to Use` section, cache chain behavior, HKT representation.
- **Method docs**: Every public method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]` attributes, and working doc examples.
- **Clone requirement rationale:** Both `map` (line 248-252) and `map_err` (line 288-292) explain why the opposite type needs `Clone`.
- **`and_then` reference semantics:** Line 639-641 explains that the callback receives `&A` unlike standard Rust `and_then`. Same for `or_else`.
- **Brand docs** (brands.rs, lines 224-242): `'static` bound on `E` is explained.

### Minor Documentation Issues

1. **`Deferrable` for `RcTryLazy`** (line 1033-1036): Says "This flattens the nested structure: instead of `TryLazy<TryLazy<A, E>, E>`, we get `TryLazy<A, E>`." This is accurate but could note the `Clone` cost.

2. **`ArcLazyConfig`'s `Deferrable` impl** (line 1326-1330): Documentation clearly explains the eager-call strategy, which is good.

3. **No documentation on the absence of `PartialEq`/`Hash`/`Ord`:** Since `Lazy` has these, a note explaining why `TryLazy` omits them would be helpful.

## Summary

`TryLazy` is a well-designed, correctly implemented fallible extension of `Lazy`. The type class coverage is appropriate given the reference-based memoization design. The API mirrors `Result`'s combinators. Documentation is thorough. The main limitations (clone overhead, `E: 'static` for HKT, no `Functor`/`Monad`) are inherent to the design and well-documented. No bugs or correctness issues found.

### Suggested Improvements (Minor)

- Add `From<TrySendThunk>` for `ArcTryLazy` for completeness.
- Consider adding `bimap(&self, f: &A -> B, g: &E -> E2) -> TryLazy<B, E2>` as a convenience.
- Consider adding `PartialEq`/`Eq` implementations (comparing evaluated results), or document why they are omitted.
- Consider adding `is_ok()` / `is_err()` convenience methods that force evaluation and return a bool.
