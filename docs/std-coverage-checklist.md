# Checklist: `fp-library` vs Rust's `std` library

This document tracks the coverage of `fp-library` against functionality provided by Rust's `std` library, mapping functional programming concepts to their standard library equivalents. It serves as a checklist for implemented features and a roadmap for future additions.

## Type Classes (Traits)

| FP Concept                           | Rust `std` Equivalent / Use Case                    | Description                                                                                                  | Implementation Path                         |
| :----------------------------------- | :-------------------------------------------------- | :----------------------------------------------------------------------------------------------------------- | :------------------------------------------ |
| **`Alternative`**                    | `Option::or`, `Option::xor`, `Result::or`           | Represents a monoid on applicative functors. Useful for providing fallback values or combining alternatives. |                                             |
| **`Applicative`**                    | N/A                                                 | A functor with application, allowing function application within a context.                                  | `fp-library/src/classes/applicative.rs`     |
| **`ApplyFirst`**                     | N/A                                                 | Sequence actions, keeping the first result.                                                                  | `fp-library/src/classes/apply_first.rs`     |
| **`ApplySecond`**                    | N/A                                                 | Sequence actions, keeping the second result.                                                                 | `fp-library/src/classes/apply_second.rs`    |
| **`Arrow`**                          | `Fn(A) -> B`                                        | Abstraction for computation, allowing composition and tuple manipulation.                                    |                                             |
| **`Bifunctor`**                      | `Result::map_err`, Tuple operations                 | Allows mapping over two types independently. Essential for `Result<T, E>` and `Pair<A, B>`.                  |                                             |
| **`Category`**                       | N/A                                                 | Abstraction for composition (identity and composition).                                                      | `fp-library/src/classes/category.rs`        |
| **`ClonableFn`**                     | `Clone + Fn`                                        | A function trait that requires `Clone`, allowing the function itself to be cloned.                           | `fp-library/src/classes/clonable_fn.rs`     |
| **`Comonad`**                        | `&` (References), `Box` (context access)            | The dual of Monad. Represents context-dependent computation where you can extract a value.                   |                                             |
| **`Contravariant`**                  | `cmp::Ordering`, Comparison functions               | Functors that map inputs rather than outputs. Crucial for composable comparison logic.                       |                                             |
| **`Defer`**                          | Lazy evaluation                                     | Abstraction for deferring execution.                                                                         | `fp-library/src/classes/defer.rs`           |
| **`Distributive`**                   | `Fn(A) -> B`                                        | Dual of `Traversable`. Allows distributing a functor over a cotraversable.                                   |                                             |
| **`Filterable`** / **`Compactable`** | `Iterator::filter`, `Option::filter`, `Vec::retain` | Abstractions for data structures that can filter out elements or compact `Option`s inside them.              |                                             |
| **`Foldable`**                       | `Iterator`                                          | Data structures that can be folded (reduced) to a summary value.                                             | `fp-library/src/classes/foldable.rs`        |
| **`Function`**                       | `Fn`                                                | Abstraction for functions.                                                                                   | `fp-library/src/classes/function.rs`        |
| **`Functor`**                        | `Iterator::map`, `Option::map`                      | Types that can be mapped over.                                                                               | `fp-library/src/classes/functor.rs`         |
| **`Invariant`**                      | `Cell`, `UnsafeCell`                                | Functors that can map in both directions (isomorphism) but not just one.                                     |                                             |
| **`Lift`**                           | N/A                                                 | Lifting functions into a context (functor/applicative/monad).                                                | `fp-library/src/classes/lift.rs`            |
| **`Monad`**                          | `Option::and_then`, `Result::and_then`              | Sequencing of computations.                                                                                  | `fp-library/src/classes/monad.rs`           |
| **`MonadError`**                     | `?` operator, `Try` trait (unstable)                | Abstracts over computation that can fail.                                                                    |                                             |
| **`Monoid`**                         | `Default + Add`                                     | Associative binary operation with an identity element.                                                       | `fp-library/src/classes/monoid.rs`          |
| **`Once`**                           | `OnceCell`, `OnceLock`                              | Abstraction for run-once semantics.                                                                          | `fp-library/src/classes/once.rs`            |
| **`Pointed`**                        | N/A                                                 | Pointed functor (pure/return).                                                                               | `fp-library/src/classes/pointed.rs`         |
| **`Profunctor`**                     | `Fn(A) -> B`                                        | Generalization of functions that can map over input (contravariant) and output (covariant).                  |                                             |
| **`Semiapplicative`**                | N/A                                                 | Applicative without `pure` (Apply).                                                                          | `fp-library/src/classes/semiapplicative.rs` |
| **`Semigroup`**                      | `Add`                                               | Associative binary operation.                                                                                | `fp-library/src/classes/semigroup.rs`       |
| **`Semigroupoid`**                   | N/A                                                 | Category without identity.                                                                                   | `fp-library/src/classes/semigroupoid.rs`    |
| **`Semimonad`**                      | N/A                                                 | Monad without `pure` (Bind).                                                                                 | `fp-library/src/classes/semimonad.rs`       |
| **`Show`**                           | `std::fmt::Display`, `std::fmt::Debug`              | Configurable string representation in FP contexts.                                                           |                                             |
| **`Traversable`**                    | `Iterator::collect`                                 | Data structures that can be traversed, turning `T<F<A>>` into `F<T<A>>`.                                     | `fp-library/src/classes/traversable.rs`     |

## Data Types (Structs/Brands)

| FP Data Type            | Rust `std` Equivalent          | Description                                              | Implementation Path                    |
| :---------------------- | :----------------------------- | :------------------------------------------------------- | :------------------------------------- |
| **`ArcBrand`**          | `std::sync::Arc`               | Shared ownership smart pointer.                          |                                        |
| **`ArcFnBrand`**        | `Arc<dyn Fn>`                  | Brand for `Arc`-wrapped closures.                        | `fp-library/src/types/arc_fn.rs`       |
| **`BinaryHeapBrand`**   | `std::collections::BinaryHeap` | Priority queue.                                          |                                        |
| **`BoxBrand`**          | `std::boxed::Box`              | Fundamental smart pointer.                               |                                        |
| **`BTreeMapBrand`**     | `std::collections::BTreeMap`   | Sorted key-value store.                                  |                                        |
| **`BTreeSetBrand`**     | `std::collections::BTreeSet`   | Sorted set.                                              |                                        |
| **`CellBrand`**         | `std::cell::Cell`              | Interior mutability container.                           |                                        |
| **`ControlFlowBrand`**  | `std::ops::ControlFlow`        | Used for flow control.                                   |                                        |
| **`CowBrand`**          | `std::borrow::Cow`             | "Clone on write".                                        |                                        |
| **`EndofunctionBrand`** | N/A                            | Brand for functions with the same input and output type. | `fp-library/src/types/endofunction.rs` |
| **`EndomorphismBrand`** | N/A                            | Brand for endomorphisms in a category.                   | `fp-library/src/types/endomorphism.rs` |
| **`FutureBrand`**       | `std::future::Future`          | Represents asynchronous values.                          |                                        |
| **`HashMapBrand`**      | `std::collections::HashMap`    | Key-value store.                                         |                                        |
| **`HashSetBrand`**      | `std::collections::HashSet`    | Set of unique values.                                    |                                        |
| **`IdentityBrand`**     | `convert::identity`            | Identity container.                                      | `fp-library/src/types/identity.rs`     |
| **`IO`**                | `main`, `std::fs`, `std::net`  | Encapsulates side effects.                               |                                        |
| **`LazyBrand`**         | `LazyLock` (unstable/std)      | Lazy evaluation wrapper.                                 | `fp-library/src/types/lazy.rs`         |
| **`LinkedListBrand`**   | `std::collections::LinkedList` | Doubly-linked list.                                      |                                        |
| **`OnceCellBrand`**     | `std::cell::OnceCell`          | Single assignment cell.                                  | `fp-library/src/types/once_cell.rs`    |
| **`OnceLockBrand`**     | `std::sync::OnceLock`          | Thread-safe single assignment.                           | `fp-library/src/types/once_lock.rs`    |
| **`OptionBrand`**       | `Option`                       | Optional value.                                          | `fp-library/src/types/option.rs`       |
| **`Ordering`** (impls)  | `std::cmp::Ordering`           | Semigroup/Monoid for Ordering.                           |                                        |
| **`OsStringBrand`**     | `std::ffi::OsString`           | Owned OS string.                                         |                                        |
| **`PairBrand`**         | `(A, B)`                       | Product type.                                            | `fp-library/src/types/pair.rs`         |
| **`PathBufBrand`**      | `std::path::PathBuf`           | Owned path.                                              |                                        |
| **`RcBrand`**           | `std::rc::Rc`                  | Shared ownership smart pointer.                          |                                        |
| **`RcFnBrand`**         | `Rc<dyn Fn>`                   | Brand for `Rc`-wrapped closures.                         | `fp-library/src/types/rc_fn.rs`        |
| **`Reader`**            | Function arguments, `std::env` | Encapsulates dependency injection.                       |                                        |
| **`RefCellBrand`**      | `std::cell::RefCell`           | Interior mutability container.                           |                                        |
| **`ResultBrand`**       | `Result`                       | Success or failure.                                      | `fp-library/src/types/result.rs`       |
| **`State`**             | `let mut`, `RefCell`           | Encapsulates stateful computations.                      |                                        |
| **`Validation`**        | `Result` (but accumulating)    | Similar to `Result`, but collects all errors.            |                                        |
| **`VecBrand`**          | `Vec`                          | Growable array.                                          | `fp-library/src/types/vec.rs`          |
| **`VecDequeBrand`**     | `std::collections::VecDeque`   | Double-ended queue.                                      |                                        |
| **`Writer`**            | `std::io::Write`, Logging      | Encapsulates logging or accumulating side outputs.       |                                        |

## Helper Wrappers (Newtypes)

These are newtypes that provide specific `Semigroup` or `Monoid` implementations for primitive types, matching standard FP patterns.

| Wrapper Type       | Rust `std` Equivalent        | Description                                                                    | Implementation Path                    |
| :----------------- | :--------------------------- | :----------------------------------------------------------------------------- | :------------------------------------- |
| **`Dual`**         | `std::cmp::Reverse`          | Flips the `Semigroup` operation.                                               |                                        |
| **`Endofunction`** | `Fn(A) -> A`                 | Wrapper for endofunctions (a -> a) enabling monoidal operations (composition). | `fp-library/src/types/endofunction.rs` |
| **`Endomorphism`** | N/A                          | Wrapper for endomorphisms (c a a) enabling monoidal operations (composition).  | `fp-library/src/types/endomorphism.rs` |
| **`First`**        | `Option::or`                 | Monoid that keeps the first non-empty value.                                   |                                        |
| **`Last`**         | `Option` (overwrite)         | Monoid that keeps the last non-empty value.                                    |                                        |
| **`Max`**          | `std::cmp::max`              | Semigroup that takes the maximum value.                                        |                                        |
| **`Min`**          | `std::cmp::min`              | Semigroup that takes the minimum value.                                        |                                        |
| **`Product`**      | `std::iter::Product` (trait) | Monoid under multiplication.                                                   |                                        |
| **`Sum`**          | `std::iter::Sum` (trait)     | Monoid under addition.                                                         |                                        |

## Free Functions

| Function       | Rust `std` Equivalent    | Description                                             | Implementation Path           |
| :------------- | :----------------------- | :------------------------------------------------------ | :---------------------------- |
| **`compose`**  | N/A                      | Function composition (`f . g`).                         | `fp-library/src/functions.rs` |
| **`constant`** | N/A                      | Returns a function that always returns the given value. | `fp-library/src/functions.rs` |
| **`flip`**     | N/A                      | Returns a function with arguments flipped.              | `fp-library/src/functions.rs` |
| **`identity`** | `std::convert::identity` | Returns its input.                                      | `fp-library/src/functions.rs` |

## Summary of Priorities

1.  **Collections**: Adding Brands for `HashMap`, `HashSet`, and `Box` would immediately widen the library's utility for standard Rust applications.
2.  **Error Handling**: `Bifunctor` and `Alternative` are high-value additions for robust error handling patterns that match `std` ergonomics.
3.  **Effect Monads**: `State`, `Reader`, and `Writer` provide pure functional alternatives to Rust's imperative patterns, adhering to the library's spirit.
