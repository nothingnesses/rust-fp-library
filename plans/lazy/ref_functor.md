# Analysis: `ref_functor.rs`

**File:** `fp-library/src/classes/ref_functor.rs`
**Role:** Defines `RefFunctor`, a variant of `Functor` where the mapping function receives `&A`.

## Design

`RefFunctor` exists because `Lazy::evaluate()` returns `&A`, not `A`. Standard `Functor::map` takes `(A -> B, F<A>) -> F<B>`, which requires an owned `A`. Since `Lazy` cannot provide owned values without `Clone`, `RefFunctor` provides:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: ...,
) -> ...;
```

## Assessment

### Correct decisions

1. **Independent from `SendRefFunctor`.** The two traits are deliberately not in a subtype relationship. This avoids forcing `Send` bounds on `RcLazy`'s mapping closures.

2. **`FnOnce` on the mapping function.** Since memoized types evaluate the closure at most once, `FnOnce` is sufficient and less restrictive than `Fn`.

3. **Cache chain behavior is documented.** The documentation warns about linked lists of `Rc`-referenced cells from chained `ref_map` calls.

4. **Comprehensive law specification.** Identity and composition laws are stated, with the identity law correctly noting the `Clone` requirement (`ref_map(|x| x.clone(), fa)` is the identity).

### Issues

#### 1. `RefFunctor` is only useful for `RcLazy`

Currently, `LazyBrand<RcLazyConfig>` is the only implementor. The trait was designed for extensibility, but in practice it serves a single type. Any third-party `LazyConfig` using `Rc` would also be a candidate, but the trait's utility is narrow.

**Impact:** Low. Having a trait for a single implementor is not inherently problematic; it enables generic code over any `RefFunctor`.

#### 2. No `RefApplicative`, `RefMonad`, etc.

`RefFunctor` stands alone without a corresponding applicative or monad hierarchy. In PureScript, `Lazy` has full `Functor -> Applicative -> Monad` instances. The Rust `RefFunctor` provides only mapping, not sequencing or binding. Users who want monadic composition with `RcLazy` must convert to `Thunk` and back.

**Impact:** Moderate. This limits the composability of `Lazy` types in monadic pipelines.

#### 3. The identity law requires `Clone`, weakening it

The identity law states `ref_map(|x| x.clone(), fa)` is equivalent to `fa`. This is weaker than the standard functor identity law (`map(id, fa) = fa`) because the `clone()` operation is not semantically identity; it creates a new memoized cell with a cloned value. The original and mapped values are equal but not the same cell.

**Impact:** Low. The law is the best possible formulation given the reference-based design.

#### 4. Verbose type annotations required at call sites

The free function `ref_map` requires explicit brand annotation: `ref_map::<LazyBrand<RcLazyConfig>, _, _>(f, fa)`. This is ergonomically heavy compared to PureScript's `map f lazy` or the inherent method `fa.ref_map(f)`.

**Impact:** Low. Users can use the inherent method for better ergonomics.
