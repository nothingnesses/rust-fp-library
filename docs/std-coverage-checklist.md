# Checklist: `fp-library` vs Rust's `std` library

This document tracks the coverage of `fp-library` against functionality provided by Rust's `std` library, mapping functional programming concepts to their standard library equivalents. It serves as a checklist for implemented features and a roadmap for future additions.

## Type Classes (Traits)

| FP Concept                   | Rust `std` Equivalent / Use Case         | Implementation Path                    |
| :--------------------------- | :--------------------------------------- | :------------------------------------- |
| **`Alt`**                    | `Option::or`, `Result::or`               | `classes/alt.rs`                       |
| **`Alternative`**            | `Option::or`, `Option::xor`              | `classes/alternative.rs`               |
| **`Applicative`**            | N/A                                      | `classes/applicative.rs`               |
| **`ApplyFirst`**             | N/A                                      | `classes/apply_first.rs`               |
| **`ApplySecond`**            | N/A                                      | `classes/apply_second.rs`              |
| **`Bifoldable`**             | N/A                                      | `classes/bifoldable.rs`                |
| **`Bifunctor`**              | `Result::map_err`, Tuple operations      | `classes/bifunctor.rs`                 |
| **`Bitraversable`**          | N/A                                      | `classes/bitraversable.rs`             |
| **`Category`**               | N/A                                      | `classes/category.rs`                  |
| **`Choice`**                 | N/A                                      | `classes/profunctor/choice.rs`         |
| **`CloneableFn`**            | `Clone + Fn`                             | `classes/cloneable_fn.rs`              |
| **`Closed`**                 | N/A                                      | `classes/profunctor/closed.rs`         |
| **`Cochoice`**               | N/A                                      | `classes/profunctor/cochoice.rs`       |
| **`CommutativeRing`**        | N/A                                      | `classes/commutative_ring.rs`          |
| **`Comonad`**                | `&` (references), `Box` (context access) | `classes/comonad.rs`                   |
| **`Compactable`**            | `Iterator::flatten` (for Option)         | `classes/compactable.rs`               |
| **`Contravariant`**          | `cmp::Ordering`, Comparison functions    | `classes/contravariant.rs`             |
| **`Costrong`**               | N/A                                      | `classes/profunctor/costrong.rs`       |
| **`Deferrable`**             | Lazy evaluation                          | `classes/deferrable.rs`                |
| **`DivisionRing`**           | N/A                                      | `classes/division_ring.rs`             |
| **`EuclideanRing`**          | N/A                                      | `classes/euclidean_ring.rs`            |
| **`Extend`**                 | N/A                                      | `classes/extend.rs`                    |
| **`Extract`**                | N/A                                      | `classes/extract.rs`                   |
| **`Field`**                  | N/A                                      | `classes/field.rs`                     |
| **`Filterable`**             | `Iterator::filter`, `Vec::retain`        | `classes/filterable.rs`                |
| **`FilterableWithIndex`**    | `Iterator::enumerate + filter`           | `classes/filterable_with_index.rs`     |
| **`Foldable`**               | `Iterator::fold`                         | `classes/foldable.rs`                  |
| **`FoldableWithIndex`**      | `Iterator::enumerate + fold`             | `classes/foldable_with_index.rs`       |
| **`Function`**               | `Fn`                                     | `classes/function.rs`                  |
| **`Functor`**                | `Iterator::map`, `Option::map`           | `classes/functor.rs`                   |
| **`FunctorWithIndex`**       | `Iterator::enumerate + map`              | `classes/functor_with_index.rs`        |
| **`HeytingAlgebra`**         | `bool` operations                        | `classes/heyting_algebra.rs`           |
| **`LazyConfig`**             | `LazyCell`, `LazyLock`                   | `classes/lazy_config.rs`               |
| **`Lift`**                   | N/A                                      | `classes/lift.rs`                      |
| **`Monad`**                  | `Option::and_then`, `Result::and_then`   | `classes/monad.rs`                     |
| **`MonadPlus`**              | N/A                                      | `classes/monad_plus.rs`                |
| **`MonadRec`**               | N/A                                      | `classes/monad_rec.rs`                 |
| **`Monoid`**                 | `Default + Add`                          | `classes/monoid.rs`                    |
| **`NaturalTransformation`**  | N/A                                      | `classes/natural_transformation.rs`    |
| **`ParCompactable`**         | `rayon` parallel compact                 | `classes/par_compactable.rs`           |
| **`ParFilterable`**          | `rayon` parallel filter                  | `classes/par_filterable.rs`            |
| **`ParFilterableWithIndex`** | `rayon` parallel indexed filter          | `classes/par_filterable_with_index.rs` |
| **`ParFoldable`**            | `rayon` parallel fold                    | `classes/par_foldable.rs`              |
| **`ParFoldableWithIndex`**   | `rayon` parallel indexed fold            | `classes/par_foldable_with_index.rs`   |
| **`ParFunctor`**             | `rayon` parallel map                     | `classes/par_functor.rs`               |
| **`ParFunctorWithIndex`**    | `rayon` parallel indexed map             | `classes/par_functor_with_index.rs`    |
| **`Pipe`**                   | Method chaining                          | `classes/pipe.rs`                      |
| **`Plus`**                   | `Default` (empty for Alt)                | `classes/plus.rs`                      |
| **`Pointed`**                | N/A                                      | `classes/pointed.rs`                   |
| **`Pointer`**                | `Box`, `Rc`, `Arc`                       | `classes/pointer.rs`                   |
| **`Profunctor`**             | `Fn(A) -> B`                             | `classes/profunctor.rs`                |
| **`RefCountedPointer`**      | `Rc`, `Arc`                              | `classes/ref_counted_pointer.rs`       |
| **`RefFunctor`**             | N/A                                      | `classes/ref_functor.rs`               |
| **`Ring`**                   | N/A                                      | `classes/ring.rs`                      |
| **`Semiapplicative`**        | N/A                                      | `classes/semiapplicative.rs`           |
| **`Semigroup`**              | `Add`                                    | `classes/semigroup.rs`                 |
| **`Semigroupoid`**           | N/A                                      | `classes/semigroupoid.rs`              |
| **`Semimonad`**              | N/A                                      | `classes/semimonad.rs`                 |
| **`Semiring`**               | N/A                                      | `classes/semiring.rs`                  |
| **`SendCloneableFn`**        | `Clone + Fn + Send + Sync`               | `classes/send_cloneable_fn.rs`         |
| **`SendDeferrable`**         | Lazy evaluation (thread-safe)            | `classes/send_deferrable.rs`           |
| **`SendRefCountedPointer`**  | `Arc`                                    | `classes/send_ref_counted_pointer.rs`  |
| **`SendRefFunctor`**         | N/A                                      | `classes/send_ref_functor.rs`          |
| **`SendUnsizedCoercible`**   | N/A                                      | `classes/send_unsized_coercible.rs`    |
| **`Strong`**                 | N/A                                      | `classes/profunctor/strong.rs`         |
| **`Traversable`**            | `Iterator::collect`                      | `classes/traversable.rs`               |
| **`TraversableWithIndex`**   | N/A                                      | `classes/traversable_with_index.rs`    |
| **`UnsizedCoercible`**       | N/A                                      | `classes/unsized_coercible.rs`         |
| **`Wander`**                 | N/A                                      | `classes/profunctor/wander.rs`         |
| **`Witherable`**             | N/A                                      | `classes/witherable.rs`                |
| **`WithIndex`**              | N/A                                      | `classes/with_index.rs`                |

### Not yet implemented

| FP Concept         | Rust `std` Equivalent / Use Case       | Notes                                                                                                         |
| :----------------- | :------------------------------------- | :------------------------------------------------------------------------------------------------------------ |
| **`Arrow`**        | `Fn(A) -> B`                           | Not a trait; operations exist as free functions (`arrow`, `split_strong`, `fan_out`) via `Category + Strong`. |
| **`Distributive`** | `Fn(A) -> B`                           | Dual of Traversable.                                                                                          |
| **`Invariant`**    | `Cell`, `UnsafeCell`                   | Functors that map in both directions.                                                                         |
| **`MonadError`**   | `?` operator, `Try` trait (unstable)   | Abstracts over computation that can fail.                                                                     |
| **`Show`**         | `std::fmt::Display`, `std::fmt::Debug` | Configurable string representation in FP contexts.                                                            |

## Data Types (Structs/Brands)

| FP Data Type              | Rust `std` Equivalent        | Implementation Path          |
| :------------------------ | :--------------------------- | :--------------------------- |
| **`ArcBrand`**            | `std::sync::Arc`             | `types/arc_ptr.rs`           |
| **`ArcCoyonedaBrand<F>`** | N/A                          | `types/arc_coyoneda.rs`      |
| **`ArcFnBrand`**          | `Arc<dyn Fn>`                | `types/fn_brand.rs`          |
| **`ArcLazy`**             | `Arc<LazyLock>`              | `types/lazy.rs`              |
| **`ArcTryLazy`**          | `Arc<LazyLock<Result>>`      | `types/try_lazy.rs`          |
| **`CatList`**             | N/A                          | `types/cat_list.rs`          |
| **`Const`**               | N/A                          | `types/const_val.rs`         |
| **`ControlFlow`**         | `std::ops::ControlFlow`      | `types/control_flow.rs`      |
| **`Coyoneda`**            | N/A                          | `types/coyoneda.rs`          |
| **`CoyonedaExplicit`**    | N/A                          | `types/coyoneda_explicit.rs` |
| **`Endofunction`**        | `Fn(A) -> A`                 | `types/endofunction.rs`      |
| **`Endomorphism`**        | N/A                          | `types/endomorphism.rs`      |
| **`FnBrand<P>`**          | `Rc<dyn Fn>` / `Arc<dyn Fn>` | `types/fn_brand.rs`          |
| **`Free`**                | N/A                          | `types/free.rs`              |
| **`Identity`**            | `convert::identity`          | `types/identity.rs`          |
| **`Option`**              | `Option`                     | `types/option.rs`            |
| **`Pair`**                | `(A, B)`                     | `types/pair.rs`              |
| **`RcBrand`**             | `std::rc::Rc`                | `types/rc_ptr.rs`            |
| **`RcCoyonedaBrand<F>`**  | N/A                          | `types/rc_coyoneda.rs`       |
| **`RcFnBrand`**           | `Rc<dyn Fn>`                 | `types/fn_brand.rs`          |
| **`RcLazy`**              | `Rc<LazyCell>`               | `types/lazy.rs`              |
| **`RcTryLazy`**           | `Rc<LazyCell<Result>>`       | `types/try_lazy.rs`          |
| **`Result`**              | `Result`                     | `types/result.rs`            |
| **`SendThunk`**           | N/A                          | `types/send_thunk.rs`        |
| **`String`**              | `String`                     | `types/string.rs`            |
| **`Thunk`**               | N/A                          | `types/thunk.rs`             |
| **`Trampoline`**          | N/A                          | `types/trampoline.rs`        |
| **`TrySendThunk`**        | N/A                          | `types/try_send_thunk.rs`    |
| **`TryThunk`**            | N/A                          | `types/try_thunk.rs`         |
| **`TryTrampoline`**       | N/A                          | `types/try_trampoline.rs`    |
| **`Tuple1`**              | `(A,)`                       | `types/tuple_1.rs`           |
| **`Tuple2`**              | `(A, B)`                     | `types/tuple_2.rs`           |
| **`Vec`**                 | `Vec`                        | `types/vec.rs`               |

### Not yet implemented

| FP Data Type          | Rust `std` Equivalent          | Notes                                              |
| :-------------------- | :----------------------------- | :------------------------------------------------- |
| **`BinaryHeapBrand`** | `std::collections::BinaryHeap` | Priority queue.                                    |
| **`BoxBrand`**        | `std::boxed::Box`              | Fundamental smart pointer.                         |
| **`BTreeMapBrand`**   | `std::collections::BTreeMap`   | Sorted key-value store.                            |
| **`BTreeSetBrand`**   | `std::collections::BTreeSet`   | Sorted set.                                        |
| **`HashMapBrand`**    | `std::collections::HashMap`    | Key-value store.                                   |
| **`HashSetBrand`**    | `std::collections::HashSet`    | Set of unique values.                              |
| **`IO`**              | `main`, `std::fs`, `std::net`  | Encapsulates side effects.                         |
| **`LinkedListBrand`** | `std::collections::LinkedList` | Doubly-linked list.                                |
| **`Reader`**          | Function arguments, `std::env` | Encapsulates dependency injection.                 |
| **`State`**           | `let mut`, `RefCell`           | Encapsulates stateful computations.                |
| **`Validation`**      | `Result` (but accumulating)    | Similar to `Result`, but collects all errors.      |
| **`VecDequeBrand`**   | `std::collections::VecDeque`   | Double-ended queue.                                |
| **`Writer`**          | `std::io::Write`, Logging      | Encapsulates logging or accumulating side outputs. |

## Helper Wrappers (Newtypes)

| Wrapper Type         | Rust `std` Equivalent        | Implementation Path       |
| :------------------- | :--------------------------- | :------------------------ |
| **`Additive`**       | `std::iter::Sum` (trait)     | `types/additive.rs`       |
| **`Conjunctive`**    | `bool` AND                   | `types/conjunctive.rs`    |
| **`Disjunctive`**    | `bool` OR                    | `types/disjunctive.rs`    |
| **`Dual`**           | `std::cmp::Reverse`          | `types/dual.rs`           |
| **`Endofunction`**   | `Fn(A) -> A`                 | `types/endofunction.rs`   |
| **`Endomorphism`**   | N/A                          | `types/endomorphism.rs`   |
| **`First`**          | `Option::or`                 | `types/first.rs`          |
| **`Last`**           | `Option` (overwrite)         | `types/last.rs`           |
| **`Multiplicative`** | `std::iter::Product` (trait) | `types/multiplicative.rs` |

### Not yet implemented

| Wrapper Type | Rust `std` Equivalent | Notes                                   |
| :----------- | :-------------------- | :-------------------------------------- |
| **`Max`**    | `std::cmp::max`       | Semigroup that takes the maximum value. |
| **`Min`**    | `std::cmp::min`       | Semigroup that takes the minimum value. |

## Free Functions

| Function       | Rust `std` Equivalent    | Implementation Path |
| :------------- | :----------------------- | :------------------ |
| **`compose`**  | N/A                      | `functions.rs`      |
| **`constant`** | N/A                      | `functions.rs`      |
| **`flip`**     | N/A                      | `functions.rs`      |
| **`identity`** | `std::convert::identity` | `functions.rs`      |
| **`on`**       | N/A                      | `functions.rs`      |
| **`pipe`**     | Method chaining          | `classes/pipe.rs`   |

All type class free functions (`map`, `bind`, `pure`, `fold_map`, `traverse`, etc.) are defined in their respective trait modules under `classes/` and re-exported via `functions.rs`.

## Summary of Priorities

1.  **Collections**: Adding brands for `HashMap`, `HashSet`, `BTreeMap`, and `Box` would immediately widen the library's utility for standard Rust applications.
2.  **Error Handling**: `Validation` (accumulating `Result`) is high-value for robust error handling and would enable the `ParTraversable` error-accumulation flavour.
3.  **Extensible Effects**: `State`, `Reader`, and `Writer` are planned as effects within an extensible effects system rather than standalone monads. See [effects plan](../plans/effects/effects.md).
