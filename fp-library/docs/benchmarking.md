# Benchmarking Comparison

This document outlines benchmarks implemented for the `fp-library` against Rust's standard library.

## Key Results

Results collected on: AMD Ryzen 9 7940HS (16 cores), Linux 6.12.77, rustc 1.93.1, bench profile.

### CoyonedaExplicit Map Fusion

CoyonedaExplicit composes maps at the type level, resulting in a single call to `F::map` at lower time regardless of chain depth. The fused line is flat while Direct scales linearly.

![Coyoneda Fusion](../../benchmarks/coyoneda-fusion.svg)

### Coyoneda Variants

Compares all Coyoneda variants across map chain depths (1-100). Direct and Box-Coyoneda are fastest; Rc/Arc variants pay per-map allocation costs.

![Coyoneda Variants](../../benchmarks/coyoneda-variants.svg)

### CatList Cons

CatList cons vs Vec insert(0) vs LinkedList push_front across sizes. Vec's O(n) insert(0) crosses over CatList at ~2000-2500 elements.

![CatList Cons](../../benchmarks/catlist-cons.svg)

### Parallel vs Sequential Fold Map

Vec par_fold_map vs sequential fold_map across sizes (100-100K) with string monoid. Rayon parallelism crosses over at ~3000-4000 elements. Log scale on both axes.

![Vec Par Fold Map](../../benchmarks/vec-par-fold-map.svg)

### Trampoline vs Iterative Loop

Cost of stack safety: Trampoline vs plain recursion vs a hand-written while loop, all doing equivalent work per step. Trampoline is ~230x slower than recursion but never overflows (recursion overflows at ~500K depth). Log scale on Y axis.

![Trampoline vs Iterative](../../benchmarks/trampoline-vs-iterative.svg)

## Detailed Comparisons

The following tables list all implemented benchmarks.

### Vec

| Feature            | `fp-library`               | `std`                                                      |
| :----------------- | :------------------------- | :--------------------------------------------------------- |
| **Map**            | `VecBrand::map`            | `iter().map().collect()`                                   |
| **Fold Right**     | `VecBrand::fold_right`     | `iter().rev().fold()`                                      |
| **Fold Left**      | `VecBrand::fold_left`      | `iter().fold()`                                            |
| **Fold Map**       | `VecBrand::fold_map`       | `iter().map().fold()`                                      |
| **Traverse**       | `VecBrand::traverse`       | `iter().map().collect::<Result<Vec<_>, _>>()` (for Result) |
| **Sequence**       | `VecBrand::sequence`       | `iter().collect::<Result<Vec<_>, _>>()` (for Result)       |
| **Bind**           | `VecBrand::bind`           | `iter().flat_map().collect()`                              |
| **Append**         | `Semigroup::append`        | `[a, b].concat()`                                          |
| **Empty**          | `Monoid::empty`            | `Vec::new()`                                               |
| **Construct**      | `VecBrand::construct`      | `[vec![x], y].concat()`                                    |
| **Deconstruct**    | `VecBrand::deconstruct`    | `slice.split_first()`                                      |
| **Filter**         | `VecBrand::filter`         | `iter().filter().collect()`                                |
| **Filter Map**     | `VecBrand::filter_map`     | `iter().filter_map().collect()`                            |
| **Partition**      | `VecBrand::partition`      | `iter().partition()`                                       |
| **Partition Map**  | `VecBrand::partition_map`  | Manual loop with two accumulators                          |
| **Compact**        | `VecBrand::compact`        | `iter().flatten().collect()`                               |
| **Separate**       | `VecBrand::separate`       | Manual loop splitting `Result`s                            |
| **Wither**         | `VecBrand::wither`         | Manual loop with conditional push                          |
| **Wilt**           | `VecBrand::wilt`           | Manual loop with two accumulators                          |
| **Lift2**          | `VecBrand::lift2`          | `flat_map` + `map` combination                             |
| **Pure**           | `VecBrand::pure`           | `vec![x]`                                                  |
| **Apply**          | `VecBrand::apply`          | `flat_map` + `map` combination                             |
| **Par Map**        | `VecBrand::par_map`        | Sequential `map` (100, 1K, 10K, 100K)                      |
| **Par Fold Map**   | `VecBrand::par_fold_map`   | Sequential `fold_map` (100, 1K, 10K, 100K)                 |
| **Par Filter Map** | `VecBrand::par_filter_map` | Sequential `filter_map` (100, 1K, 10K, 100K)               |
| **Par Compact**    | `VecBrand::par_compact`    | Sequential `compact` (100, 1K, 10K, 100K)                  |

### Option

| Feature           | `fp-library`                 | `std`                                    |
| :---------------- | :--------------------------- | :--------------------------------------- |
| **Map**           | `OptionBrand::map`           | `Option::map`                            |
| **Fold Right**    | `OptionBrand::fold_right`    | `map_or`                                 |
| **Fold Left**     | `OptionBrand::fold_left`     | `map_or`                                 |
| **Traverse**      | `OptionBrand::traverse`      | `Option::map().transpose()` (for Result) |
| **Sequence**      | `OptionBrand::sequence`      | `Option::transpose()` (for Result)       |
| **Bind**          | `OptionBrand::bind`          | `Option::and_then`                       |
| **Filter**        | `OptionBrand::filter`        | `Option::filter`                         |
| **Filter Map**    | `OptionBrand::filter_map`    | `Option::and_then`                       |
| **Partition**     | `OptionBrand::partition`     | Conditional split                        |
| **Partition Map** | `OptionBrand::partition_map` | `map_or` with conditional                |
| **Compact**       | `OptionBrand::compact`       | `Option::flatten`                        |
| **Separate**      | `OptionBrand::separate`      | Pattern match on `Option<Result>`        |
| **Wither**        | `OptionBrand::wither`        | `map` + `unwrap_or`                      |
| **Wilt**          | `OptionBrand::wilt`          | `map` + `unwrap_or`                      |
| **Lift2**         | `OptionBrand::lift2`         | `Option::zip` + `map`                    |
| **Pure**          | `OptionBrand::pure`          | `Some(x)`                                |
| **Apply**         | `OptionBrand::apply`         | Pattern match                            |

### Result

| Feature        | `fp-library`                        | `std`                                    |
| :------------- | :---------------------------------- | :--------------------------------------- |
| **Map**        | `ResultErrAppliedBrand::map`        | `Result::map`                            |
| **Fold Right** | `ResultErrAppliedBrand::fold_right` | `map_or`                                 |
| **Fold Left**  | `ResultErrAppliedBrand::fold_left`  | `map_or`                                 |
| **Traverse**   | `ResultErrAppliedBrand::traverse`   | `Result::map().transpose()` (for Option) |
| **Sequence**   | `ResultErrAppliedBrand::sequence`   | `Result::transpose()` (for Option)       |
| **Bind**       | `ResultErrAppliedBrand::bind`       | `Result::and_then`                       |
| **Lift2**      | `ResultErrAppliedBrand::lift2`      | `and_then` + `map`                       |
| **Pure**       | `ResultErrAppliedBrand::pure`       | `Ok(x)`                                  |
| **Apply**      | `ResultErrAppliedBrand::apply`      | Pattern match                            |

### String

| Feature    | `fp-library`        | `std`            |
| :--------- | :------------------ | :--------------- |
| **Append** | `Semigroup::append` | `+` / `push_str` |
| **Empty**  | `Monoid::empty`     | `String::new()`  |

### Pair

| Feature        | `fp-library`                        | `std`                                    |
| :------------- | :---------------------------------- | :--------------------------------------- |
| **Map**        | `PairFirstAppliedBrand::map`        | Manual tuple construction                |
| **Fold Right** | `PairFirstAppliedBrand::fold_right` | Direct field access                      |
| **Fold Left**  | `PairFirstAppliedBrand::fold_left`  | Direct field access                      |
| **Traverse**   | `PairFirstAppliedBrand::traverse`   | `map` + tuple reconstruction             |
| **Sequence**   | `PairFirstAppliedBrand::sequence`   | `map` + tuple reconstruction             |
| **Bind**       | `PairFirstAppliedBrand::bind`       | Manual semigroup append + extraction     |
| **Lift2**      | `PairFirstAppliedBrand::lift2`      | Manual semigroup append + field combine  |
| **Pure**       | `PairFirstAppliedBrand::pure`       | `Pair(Monoid::empty(), x)`               |
| **Apply**      | `PairFirstAppliedBrand::apply`      | Manual semigroup append + function apply |

### Functions

| Feature      | `fp-library` | `std`                    |
| :----------- | :----------- | :----------------------- |
| **Identity** | `identity`   | `std::convert::identity` |

### Lazy Evaluation

| Feature                     | `fp-library`                                            | Description                                        |
| :-------------------------- | :------------------------------------------------------ | :------------------------------------------------- |
| **Thunk Baseline**          | `Thunk::new` + `evaluate`                               | Baseline overhead                                  |
| **Thunk Map Chain**         | `Thunk::map` chains (1, 5, 10, 25, 50, 100)             | Cost of chained maps                               |
| **Thunk Bind Chain**        | `Thunk::bind` chains (1, 5, 10, 25, 50, 100)            | Cost of chained binds                              |
| **Trampoline Baseline**     | `Trampoline::new` + `evaluate`                          | Baseline overhead                                  |
| **Trampoline Bind Chain**   | `Trampoline::bind` chains (100-10K, 6 sizes)            | Stack-safe bind performance                        |
| **Trampoline Map Chain**    | `Trampoline::map` chains (100-10K, 6 sizes)             | Stack-safe map performance                         |
| **Trampoline tail_rec_m**   | Countdown from 10K via `ControlFlow`                    | Monadic tail recursion                             |
| **Trampoline vs Iterative** | `tail_rec_m` vs hand-written loop                       | Overhead vs imperative code                        |
| **RcLazy First Access**     | `Lazy::<_, RcLazyConfig>::new` + first `evaluate`       | Memoization first-access cost                      |
| **RcLazy Cached Access**    | Repeated `evaluate` on cached value                     | Memoization cache-hit cost                         |
| **RcLazy ref_map Chain**    | `ref_map` chains (1, 5, 10, 25, 50, 100)                | Cost of chained ref-mapped lazy values             |
| **ArcLazy First Access**    | `ArcLazy::new` + first `evaluate`                       | Thread-safe memoization first-access cost          |
| **ArcLazy Cached Access**   | Repeated `evaluate` on cached value                     | Thread-safe memoization cache-hit cost             |
| **ArcLazy ref_map Chain**   | `ref_map` chains (1, 5, 10, 25, 50, 100)                | Cost of chained ref-mapped thread-safe lazy values |
| **Free Left-Assoc Bind**    | Left-associated `Free::bind` chains (100-10K, 6 sizes)  | CatList-backed O(1) bind reassociation             |
| **Free Right-Assoc Bind**   | Right-associated `Free::bind` chains (100-10K, 6 sizes) | Nested right-bind performance                      |
| **Free Evaluate**           | `Free::wrap` + `bind` chains (100-10K, 6 sizes)         | Evaluation of suspended computations               |

### CatList

| Feature               | Compared Against             | Description                                       |
| :-------------------- | :--------------------------- | :------------------------------------------------ |
| **Cons**              | CatList vs LinkedList vs Vec | Prepend element (O(1))                            |
| **Snoc**              | CatList vs Vec               | Append element (O(1))                             |
| **Append**            | CatList vs Vec               | Concatenation (O(1) vs O(n))                      |
| **Uncons**            | CatList vs Vec vs LinkedList | Head/Tail decomposition (amortized O(1))          |
| **Left-Assoc Append** | CatList vs Vec vs LinkedList | Repeated left-associated appends (O(n) vs O(n^2)) |
| **Iteration**         | CatList vs Vec vs LinkedList | Full iteration overhead                           |
| **Nested Uncons**     | CatList (nested vs flat)     | Uncons on deeply nested structures                |
| **Fold Map**          | CatList vs Vec (fp + std)    | fold_map performance                              |
| **Fold Left**         | CatList vs Vec (fp + std)    | fold_left performance                             |
| **Traverse**          | CatList vs Vec (fp)          | traverse with Option                              |
| **Filter**            | CatList vs Vec (fp + std)    | filter performance                                |
| **Compact**           | CatList vs Vec (fp + std)    | compact performance                               |

### Coyoneda

| Feature                | Compared Against                      | Description                                    |
| :--------------------- | :------------------------------------ | :--------------------------------------------- |
| **Direct vs Variants** | Direct map vs all 4 Coyoneda variants | Map chain cost at depths 1, 5, 10, 25, 50, 100 |
| **Repeated Lower**     | RcCoyoneda vs ArcCoyoneda             | Re-evaluation cost (3x lower_ref)              |
| **Clone Map**          | RcCoyoneda vs ArcCoyoneda             | Clone + map + lower_ref pattern                |
