# Benchmarking Comparison

This document outlines benchmarks implemented for the `fp-library` against Rust's standard library.

## Comparisons

We will compare the performance of the following `fp-library` abstractions against their `std` equivalents.

### Vec

| Feature | `fp-library`    | `std`                   | Status                                                     | [ ] |
| :------ | :-------------- | :---------------------- | :--------------------------------------------------------- | --- |
|         | **Map**         | `VecBrand::map`         | `iter().map().collect()`                                   | [x] |
|         | **Fold Right**  | `VecBrand::fold_right`  | `iter().rev().fold()`                                      | [x] |
|         | **Fold Left**   | `VecBrand::fold_left`   | `iter().fold()`                                            | [x] |
|         | **Fold Map**    | `VecBrand::fold_map`    | `iter().map().fold()`                                      | [x] |
|         | **Traverse**    | `VecBrand::traverse`    | `iter().map().collect::<Result<Vec<_>, _>>()` (for Result) | [x] |
|         | **Sequence**    | `VecBrand::sequence`    | `iter().collect::<Result<Vec<_>, _>>()` (for Result)       | [x] |
|         | **Bind**        | `VecBrand::bind`        | `iter().flat_map().collect()`                              | [x] |
|         | **Append**      | `Semigroup::append`     | `[a, b].concat()`                                          | [x] |
|         | **Empty**       | `Monoid::empty`         | `Vec::new()`                                               | [x] |
|         | **Construct**   | `VecBrand::construct`   | `[vec![x], y].concat()`                                    | [x] |
|         | **Deconstruct** | `VecBrand::deconstruct` | `slice.split_first()`                                      | [x] |
|         | **Filter**      | `VecBrand::filter`      | `iter().filter().collect()`                                | [x] |
|         | **Filter Map**  | `VecBrand::filter_map`  | `iter().filter_map().collect()`                            | [x] |
|         | **Partition**   | `VecBrand::partition`   | `iter().partition()`                                       | [x] |
|         | **Partition Map** | `VecBrand::partition_map` | Manual loop with two accumulators                        | [x] |
|         | **Compact**     | `VecBrand::compact`     | `iter().flatten().collect()`                               | [x] |
|         | **Separate**    | `VecBrand::separate`    | Manual loop splitting `Result`s                            | [x] |
|         | **Wither**      | `VecBrand::wither`      | Manual loop with conditional push                          | [x] |
|         | **Wilt**        | `VecBrand::wilt`        | Manual loop with two accumulators                          | [x] |
|         | **Lift2**       | `VecBrand::lift2`       | `flat_map` + `map` combination                             | [x] |
|         | **Pure**        | `VecBrand::pure`        | `vec![x]`                                                  | [x] |
|         | **Apply**       | `VecBrand::apply`       | `flat_map` + `map` combination                             | [x] |
|         | **Par Fold Map** | `VecBrand::par_fold_map` | (fp-only, no std equivalent)                              | [x] |

### Option

| Feature | `fp-library`   | `std`                     | Status                                   | [ ] |
| :------ | :------------- | :------------------------ | :--------------------------------------- | --- |
|         | **Map**        | `OptionBrand::map`        | `Option::map`                            | [x] |
|         | **Fold Right** | `OptionBrand::fold_right` | `map_or`                                 | [x] |
|         | **Fold Left**  | `OptionBrand::fold_left`  | `map_or`                                 | [x] |
|         | **Traverse**   | `OptionBrand::traverse`   | `Option::map().transpose()` (for Result) | [x] |
|         | **Sequence**   | `OptionBrand::sequence`   | `Option::transpose()` (for Result)       | [x] |
|         | **Bind**       | `OptionBrand::bind`       | `Option::and_then`                       | [x] |
|         | **Filter**     | `OptionBrand::filter`     | `Option::filter`                         | [x] |
|         | **Filter Map** | `OptionBrand::filter_map` | `Option::and_then`                       | [x] |
|         | **Partition**  | `OptionBrand::partition`  | Conditional split                        | [x] |
|         | **Partition Map** | `OptionBrand::partition_map` | `map_or` with conditional           | [x] |
|         | **Compact**    | `OptionBrand::compact`    | `Option::flatten`                        | [x] |
|         | **Separate**   | `OptionBrand::separate`   | Pattern match on `Option<Result>`        | [x] |
|         | **Wither**     | `OptionBrand::wither`     | `map` + `unwrap_or`                      | [x] |
|         | **Wilt**       | `OptionBrand::wilt`       | `map` + `unwrap_or`                      | [x] |
|         | **Lift2**      | `OptionBrand::lift2`      | `Option::zip` + `map`                    | [x] |
|         | **Pure**       | `OptionBrand::pure`       | `Some(x)`                                | [x] |
|         | **Apply**      | `OptionBrand::apply`      | Pattern match                            | [x] |

### Result

| Feature | `fp-library`   | `std`                               | Status                                   | [ ] |
| :------ | :------------- | :---------------------------------- | :--------------------------------------- | --- |
|         | **Map**        | `ResultErrAppliedBrand::map`        | `Result::map`                            | [x] |
|         | **Fold Right** | `ResultErrAppliedBrand::fold_right` | `map_or`                                 | [x] |
|         | **Fold Left**  | `ResultErrAppliedBrand::fold_left`  | `map_or`                                 | [x] |
|         | **Traverse**   | `ResultErrAppliedBrand::traverse`   | `Result::map().transpose()` (for Option) | [x] |
|         | **Sequence**   | `ResultErrAppliedBrand::sequence`   | `Result::transpose()` (for Option)       | [x] |
|         | **Bind**       | `ResultErrAppliedBrand::bind`       | `Result::and_then`                       | [x] |
|         | **Lift2**      | `ResultErrAppliedBrand::lift2`      | `and_then` + `map`                       | [x] |
|         | **Pure**       | `ResultErrAppliedBrand::pure`       | `Ok(x)`                                  | [x] |
|         | **Apply**      | `ResultErrAppliedBrand::apply`      | Pattern match                            | [x] |

### String

| Feature | `fp-library` | `std`               | Status           | [ ] |
| :------ | :----------- | :------------------ | :--------------- | --- |
|         | **Append**   | `Semigroup::append` | `+` / `push_str` | [x] |
|         | **Empty**    | `Monoid::empty`     | `String::new()`  | [x] |

### Pair

| Feature | `fp-library` | `std`                                   | Status                                   | [ ] |
| :------ | :----------- | :-------------------------------------- | :--------------------------------------- | --- |
|         | **Map**        | `PairFirstAppliedBrand::map`        | Manual tuple construction                | [x] |
|         | **Fold Right** | `PairFirstAppliedBrand::fold_right` | Direct field access                      | [x] |
|         | **Fold Left**  | `PairFirstAppliedBrand::fold_left`  | Direct field access                      | [x] |
|         | **Traverse**   | `PairFirstAppliedBrand::traverse`   | `map` + tuple reconstruction             | [x] |
|         | **Sequence**   | `PairFirstAppliedBrand::sequence`   | `map` + tuple reconstruction             | [x] |
|         | **Bind**       | `PairFirstAppliedBrand::bind`       | Manual semigroup append + extraction     | [x] |
|         | **Lift2**      | `PairFirstAppliedBrand::lift2`      | Manual semigroup append + field combine  | [x] |
|         | **Pure**       | `PairFirstAppliedBrand::pure`       | `Pair(Monoid::empty(), x)`               | [x] |
|         | **Apply**      | `PairFirstAppliedBrand::apply`      | Manual semigroup append + function apply | [x] |

### Functions

| Feature | `fp-library` | `std`      | Status                   | [ ] |
| :------ | :----------- | :--------- | :----------------------- | --- |
|         | **Identity** | `identity` | `std::convert::identity` | [x] |

### Lazy Evaluation

| Feature | `fp-library` | Description | Status | [ ] |
| :------ | :----------- | :---------- | :----- | --- |
|         | **Thunk Baseline**            | `Thunk::new` + `evaluate`             | Baseline overhead                               | [x] |
|         | **Thunk Map Chain**           | `Thunk::map` chains (1, 10, 100)      | Cost of chained maps                            | [x] |
|         | **Thunk Bind Chain**          | `Thunk::bind` chains (1, 10, 100)     | Cost of chained binds                           | [x] |
|         | **Trampoline Baseline**       | `Trampoline::new` + `evaluate`        | Baseline overhead                               | [x] |
|         | **Trampoline Bind Chain**     | `Trampoline::bind` chains (100, 1K, 10K) | Stack-safe bind performance                 | [x] |
|         | **Trampoline Map Chain**      | `Trampoline::map` chains (100, 1K, 10K)  | Stack-safe map performance                  | [x] |
|         | **Trampoline tail_rec_m**     | Countdown from 10K via `ControlFlow`  | Monadic tail recursion                          | [x] |
|         | **Trampoline vs Iterative**   | `tail_rec_m` vs hand-written loop     | Overhead vs imperative code                     | [x] |
|         | **RcLazy First Access**       | `Lazy::<_, RcLazyConfig>::new` + first `evaluate` | Memoization first-access cost          | [x] |
|         | **RcLazy Cached Access**      | Repeated `evaluate` on cached value   | Memoization cache-hit cost                      | [x] |
|         | **RcLazy ref_map Chain**      | `ref_map` chains (1, 10, 100)         | Cost of chained ref-mapped lazy values          | [x] |
|         | **ArcLazy First Access**      | `ArcLazy::new` + first `evaluate`     | Thread-safe memoization first-access cost       | [x] |
|         | **ArcLazy Cached Access**     | Repeated `evaluate` on cached value   | Thread-safe memoization cache-hit cost          | [x] |
|         | **ArcLazy ref_map Chain**     | `ref_map` chains (1, 10, 100)         | Cost of chained ref-mapped thread-safe lazy values | [x] |
|         | **Free Left-Assoc Bind**      | Left-associated `Free::bind` chains (100, 1K, 10K) | CatList-backed O(1) bind reassociation | [x] |
|         | **Free Right-Assoc Bind**     | Right-associated `Free::bind` chains (100, 1K, 10K) | Nested right-bind performance         | [x] |
|         | **Free Evaluate**             | `Free::wrap` + `bind` chains (100, 1K, 10K) | Evaluation of suspended computations      | [x] |

### CatList

| Feature | `fp-library` | Compared Against | Description | Status | [ ] |
| :------ | :----------- | :--------------- | :---------- | :----- | --- |
|         | **Cons**                | CatList vs LinkedList vs Vec | Prepend element (O(1))               | [x] |
|         | **Snoc**                | CatList vs Vec               | Append element (O(1))                | [x] |
|         | **Append**              | CatList vs Vec               | Concatenation (O(1) vs O(n))         | [x] |
|         | **Uncons**              | CatList vs Vec vs LinkedList | Head/Tail decomposition (amortized O(1)) | [x] |
|         | **Left-Assoc Append**   | CatList vs Vec vs LinkedList | Repeated left-associated appends (O(n) vs O(n^2)) | [x] |
|         | **Iteration**           | CatList vs Vec vs LinkedList | Full iteration overhead              | [x] |
|         | **Nested Uncons**       | CatList (nested vs flat)     | Uncons on deeply nested structures   | [x] |
