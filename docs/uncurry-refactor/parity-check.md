# Feature Parity Check

This document tracks the feature parity between v1 and v2 of the library.

## Methodology

To ensure comprehensive coverage, the following commands were used to generate the inventory of traits, types, and implementations. Future verifications should repeat this process.

### v1 Data Collection

```bash
# List all public traits in v1 classes
grep -r "pub trait" fp-library/src/classes

# List all public structs in v1 types
grep -r "pub struct" fp-library/src/types

# List all trait implementations in v1 types
grep -r "impl.*for" fp-library/src/types
```

### v2 Data Collection

```bash
# List all public traits in v2 classes
grep -r "pub trait" fp-library/src/v2/classes

# List all public structs in v2 types
grep -r "pub struct" fp-library/src/v2/types

# List all trait implementations in v2 types
grep -r "impl.*for" fp-library/src/v2/types
```

## v1 Inventory

### Classes (Traits)

| File                         | Trait                        |
| ---------------------------- | ---------------------------- |
| `classes/foldable.rs`        | `Foldable`                   |
| `classes/apply_second.rs`    | `ApplySecond`                |
| `classes/apply_first.rs`     | `ApplyFirst`                 |
| `classes/semiapplicative.rs` | `Semiapplicative`            |
| `classes/defer.rs`           | `Defer`                      |
| `classes/function.rs`        | `Function`                   |
| `classes/category.rs`        | `Category`                   |
| `classes/clonable_fn.rs`     | `ClonableFn`                 |
| `classes/monoid.rs`          | `Monoid`, `Monoid1L0T`       |
| `classes/functor.rs`         | `Functor`                    |
| `classes/applicative.rs`     | `Applicative`                |
| `classes/semigroup.rs`       | `Semigroup`, `Semigroup1L0T` |
| `classes/once.rs`            | `Once`                       |
| `classes/semimonad.rs`       | `Semimonad`                  |
| `classes/pointed.rs`         | `Pointed`                    |
| `classes/traversable.rs`     | `Traversable`                |
| `classes/semigroupoid.rs`    | `Semigroupoid`               |
| `classes/monad.rs`           | `Monad`                      |

### Types (Structs/Enums)

| File                              | Type                  | Traits Implemented                                                                                                       |
| --------------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `types/endofunction.rs`           | `Endofunction`        | `Clone`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`, `Semigroup`, `Monoid`                                  |
| `types/endofunction.rs`           | `EndofunctionBrand`   | `Kind1L0T`                                                                                                               |
| `types/result.rs`                 | `ResultBrand`         | `Kind0L2T`                                                                                                               |
| `types/vec.rs`                    | `VecBrand`            | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/arc_fn.rs`                 | `ArcFnBrand`          | `Kind1L2T`, `Function`, `ClonableFn`, `Semigroupoid`, `Category`                                                         |
| `types/rc_fn.rs`                  | `RcFnBrand`           | `Kind1L2T`, `Function`, `ClonableFn`, `Semigroupoid`, `Category`                                                         |
| `types/identity.rs`               | `IdentityBrand`       | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/lazy.rs`                   | `Lazy`                | `Defer`                                                                                                                  |
| `types/lazy.rs`                   | `LazyBrand`           | (None listed in grep, likely Kind1L0T?)                                                                                  |
| `types/once_cell.rs`              | `OnceCellBrand`       | `Kind0L1T`, `Once`                                                                                                       |
| `types/result/result_with_ok.rs`  | `ResultWithOkBrand`   | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/result/result_with_err.rs` | `ResultWithErrBrand`  | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/option.rs`                 | `OptionBrand`         | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/pair.rs`                   | `PairBrand`           | `Kind0L2T`                                                                                                               |
| `types/endomorphism.rs`           | `Endomorphism`        | `Clone`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`, `Semigroup`, `Monoid`                                  |
| `types/endomorphism.rs`           | `EndomorphismBrand`   | `Kind1L0T`                                                                                                               |
| `types/once_lock.rs`              | `OnceLockBrand`       | `Kind0L1T`, `Once`                                                                                                       |
| `types/pair/pair_with_second.rs`  | `PairWithSecondBrand` | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/pair/pair_with_first.rs`   | `PairWithFirstBrand`  | `Kind0L1T`, `Functor`, `Semiapplicative`, `ApplyFirst`, `ApplySecond`, `Pointed`, `Semimonad`, `Foldable`, `Traversable` |
| `types/string.rs`                 | `String`              | `Semigroup`, `Semigroup1L0T`, `Monoid`, `Monoid1L0T`                                                                     |
| `types/vec/concrete_vec.rs`       | `Vec<A>`              | `Kind1L0T`, `Semigroup`, `Semigroup1L0T`, `Monoid`, `Monoid1L0T`                                                         |

## v2 Inventory

### Classes (Traits)

| File                            | Trait             |
| ------------------------------- | ----------------- |
| `v2/classes/foldable.rs`        | `Foldable`        |
| `v2/classes/apply_second.rs`    | `ApplySecond`     |
| `v2/classes/apply_first.rs`     | `ApplyFirst`      |
| `v2/classes/semiapplicative.rs` | `Semiapplicative` |
| `v2/classes/defer.rs`           | `Defer`           |
| `v2/classes/function.rs`        | `Function`        |
| `v2/classes/category.rs`        | `Category`        |
| `v2/classes/clonable_fn.rs`     | `ClonableFn`      |
| `v2/classes/monoid.rs`          | `Monoid`          |
| `v2/classes/functor.rs`         | `Functor`         |
| `v2/classes/applicative.rs`     | `Applicative`     |
| `v2/classes/semigroup.rs`       | `Semigroup`       |
| `v2/classes/once.rs`            | `Once`            |
| `v2/classes/semimonad.rs`       | `Semimonad`       |
| `v2/classes/lift.rs`            | `Lift`            |
| `v2/classes/pointed.rs`         | `Pointed`         |
| `v2/classes/traversable.rs`     | `Traversable`     |
| `v2/classes/semigroupoid.rs`    | `Semigroupoid`    |
| `v2/classes/monad.rs`           | `Monad`           |

### Types (Structs/Enums)

| File                       | Type                  | Traits Implemented                                                                                                               |
| -------------------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `v2/types/endofunction.rs` | `Endofunction`        | `Clone`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`, `Semigroup`, `Monoid`                                          |
| `v2/types/result.rs`       | `ResultBrand`         | `Kind0L2T`                                                                                                                       |
| `v2/types/arc_fn.rs`       | `ArcFnBrand`          | `Kind1L2T`, `Function`, `ClonableFn`, `Semigroupoid`, `Category`                                                                 |
| `v2/types/rc_fn.rs`        | `RcFnBrand`           | `Kind1L2T`, `Function`, `ClonableFn`, `Semigroupoid`, `Category`                                                                 |
| `v2/types/lazy.rs`         | `Lazy`                | `Defer`, `Semigroup`, `Monoid`                                                                                                   |
| `v2/types/lazy.rs`         | `LazyBrand`           | `Kind1L1T`                                                                                                                       |
| `v2/types/once_cell.rs`    | `OnceCellBrand`       | `Kind0L1T`, `Once`                                                                                                               |
| `v2/types/pair.rs`         | `PairBrand`           | `Kind0L2T`                                                                                                                       |
| `v2/types/endomorphism.rs` | `Endomorphism`        | `Clone`, `Debug`, `Eq`, `Hash`, `Ord`, `PartialEq`, `PartialOrd`, `Semigroup`, `Monoid`                                          |
| `v2/types/once_lock.rs`    | `OnceLockBrand`       | `Kind0L1T`, `Once`                                                                                                               |
| `v2/types/result.rs`       | `ResultWithErrBrand`  | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/result.rs`       | `ResultWithOkBrand`   | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/vec.rs`          | `VecBrand`            | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/identity.rs`     | `IdentityBrand`       | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/option.rs`       | `OptionBrand`         | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/string.rs`       | `String`              | `Kind1L0T`, `Semigroup`, `Monoid`                                                                                                |
| `v2/types/pair.rs`         | `PairWithFirstBrand`  | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/pair.rs`         | `PairWithSecondBrand` | `Kind1L1T`, `Functor`, `Lift`, `Pointed`, `ApplyFirst`, `ApplySecond`, `Semiapplicative`, `Semimonad`, `Foldable`, `Traversable` |
| `v2/types/vec.rs`          | `Vec<A>`              | `Semigroup`, `Monoid`                                                                                                            |

## Comparison

| Item            | v1                          | v2                          | Status                |
| --------------- | --------------------------- | --------------------------- | --------------------- |
| `Functor`       | Yes                         | Yes                         | Parity                |
| `Applicative`   | Yes                         | Yes                         | Parity                |
| `Monad`         | Yes                         | Yes                         | Parity                |
| `Lift`          | No                          | Yes                         | **New**               |
| `OptionBrand`   | All traits                  | All traits + Lift           | Parity+               |
| `VecBrand`      | All traits                  | All traits + Lift           | Parity+               |
| `Vec<A>`        | Semigroup, Monoid           | Semigroup, Monoid           | Parity                |
| `String`        | Semigroup, Monoid, Kind1L0T | Semigroup, Monoid, Kind1L0T | Parity                |
| `PairBrand`     | Kind0L2T                    | Kind0L2T                    | Parity                |
| `ResultBrand`   | Kind0L2T                    | Kind0L2T                    | Parity                |
| `OnceLockBrand` | Once                        | Once                        | Parity                |
| `Lazy`          | Defer                       | Defer, Semigroup, Monoid    | Parity+               |
| `LazyBrand`     | Kind1L0T?                   | Kind1L1T                    | **Changed** (Upgrade) |
