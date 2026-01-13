# Benchmarking Plan

This document outlines the plan for benchmarking the `fp-library` against Rust's standard library.

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

### Option

| Feature | `fp-library`   | `std`                     | Status                                   | [ ] |
| :------ | :------------- | :------------------------ | :--------------------------------------- | --- |
|         | **Map**        | `OptionBrand::map`        | `Option::map`                            | [x] |
|         | **Fold Right** | `OptionBrand::fold_right` | `map_or`                                 | [x] |
|         | **Fold Left**  | `OptionBrand::fold_left`  | `map_or`                                 | [x] |
|         | **Traverse**   | `OptionBrand::traverse`   | `Option::map().transpose()` (for Result) | [x] |
|         | **Sequence**   | `OptionBrand::sequence`   | `Option::transpose()` (for Result)       | [x] |
|         | **Bind**       | `OptionBrand::bind`       | `Option::and_then`                       | [x] |

### Result

| Feature | `fp-library`   | `std`                            | Status                                   | [ ] |
| :------ | :------------- | :------------------------------- | :--------------------------------------- | --- |
|         | **Map**        | `ResultWithErrBrand::map`        | `Result::map`                            | [x] |
|         | **Fold Right** | `ResultWithErrBrand::fold_right` | `map_or`                                 | [x] |
|         | **Fold Left**  | `ResultWithErrBrand::fold_left`  | `map_or`                                 | [x] |
|         | **Traverse**   | `ResultWithErrBrand::traverse`   | `Result::map().transpose()` (for Option) | [x] |
|         | **Sequence**   | `ResultWithErrBrand::sequence`   | `Result::transpose()` (for Option)       | [x] |
|         | **Bind**       | `ResultWithErrBrand::bind`       | `Result::and_then`                       | [x] |

### String

| Feature | `fp-library` | `std`               | Status           | [ ] |
| :------ | :----------- | :------------------ | :--------------- | --- |
|         | **Append**   | `Semigroup::append` | `+` / `push_str` | [x] |
|         | **Empty**    | `Monoid::empty`     | `String::new()`  | [x] |

### Functions

| Feature | `fp-library` | `std`      | Status                   | [ ] |
| :------ | :----------- | :--------- | :----------------------- | --- |
|         | **Identity** | `identity` | `std::convert::identity` | [x] |

### OnceCell

| Feature | `fp-library`    | `std`               | Status                  | [ ] |
| :------ | :-------------- | :------------------ | :---------------------- | --- |
|         | **New**         | `Once::new`         | `OnceCell::new`         | [x] |
|         | **Get**         | `Once::get`         | `OnceCell::get`         | [x] |
|         | **Set**         | `Once::set`         | `OnceCell::set`         | [x] |
|         | **Get Or Init** | `Once::get_or_init` | `OnceCell::get_or_init` | [x] |
|         | **Take**        | `Once::take`        | `OnceCell::take`        | [x] |

### OnceLock

| Feature | `fp-library`    | `std`               | Status                  | [ ] |
| :------ | :-------------- | :------------------ | :---------------------- | --- |
|         | **New**         | `Once::new`         | `OnceLock::new`         | [x] |
|         | **Get**         | `Once::get`         | `OnceLock::get`         | [x] |
|         | **Set**         | `Once::set`         | `OnceLock::set`         | [x] |
|         | **Get Or Init** | `Once::get_or_init` | `OnceLock::get_or_init` | [x] |
|         | **Take**        | `Once::take`        | `OnceLock::take`        | [x] |

## Checklist

- [x] Create `fp-library/benches/benchmarks.rs`
- [x] Implement `Vec` benchmarks
- [x] Implement `Option` benchmarks
- [x] Implement `Result` benchmarks
- [x] Implement `String` benchmarks
- [x] Implement `Functions` benchmarks
- [x] Implement `OnceCell` benchmarks
- [x] Implement `OnceLock` benchmarks
