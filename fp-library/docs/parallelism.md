## Thread Safety and Parallelism

The library provides a parallel trait hierarchy that mirrors the sequential one.
All `par_*` free functions accept plain `impl Fn + Send + Sync` closures: no wrapper
types required. Element types require `A: Send`; closures require `Send + Sync`.

```mermaid
graph TD
    ParFunctor --> ParFilterable
    ParCompactable --> ParFilterable
    ParFunctor --> ParFunctorWithIndex
    ParFoldable --> ParFoldableWithIndex
    ParFilterable --> ParFilterableWithIndex
    ParFoldableWithIndex --> ParFilterableWithIndex
```

| Parallel trait           | Operations                     | Supertraits                                        |
| ------------------------ | ------------------------------ | -------------------------------------------------- |
| `ParFunctor`             | `par_map`                      | `Kind`                                             |
| `ParCompactable`         | `par_compact`, `par_separate`  | `Kind`                                             |
| `ParFilterable`          | `par_filter_map`, `par_filter` | `ParFunctor + ParCompactable`                      |
| `ParFoldable`            | `par_fold_map`                 | `Kind`                                             |
| `ParFunctorWithIndex`    | `par_map_with_index`           | `ParFunctor + FunctorWithIndex`                    |
| `ParFoldableWithIndex`   | `par_fold_map_with_index`      | `ParFoldable + FoldableWithIndex`                  |
| `ParFilterableWithIndex` | `par_filter_map_with_index`    | `ParFilterable + ParFoldableWithIndex + WithIndex` |

`ParFilterable` provides default implementations of `par_filter_map` and `par_filter`
derived from `par_map` + `par_compact`; types can override them for single-pass efficiency.

- **`SendCloneFn`**: Thread-safe cloneable function wrappers with `Send + Sync` bounds. Implemented by `ArcFnBrand`.
- **Rayon Support**: When the `rayon` feature is enabled, `par_*` functions use rayon for true parallel execution. Otherwise they fall back to sequential equivalents.

```
use fp_library::{
	brands::*,
	functions::*,
};

let v = vec![1, 2, 3, 4, 5];
// Map in parallel (uses rayon if feature is enabled)
let doubled: Vec<i32> = par_map::<VecBrand, _, _>(|x: i32| x * 2, v.clone());
assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
// Compact options in parallel
let opts = vec![Some(1), None, Some(3), None, Some(5)];
let compacted: Vec<i32> = par_compact::<VecBrand, _>(opts);
assert_eq!(compacted, vec![1, 3, 5]);
// Fold in parallel
let result = par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v);
assert_eq!(result, "12345".to_string());
```
