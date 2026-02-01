//! Functional programming type classes.
//!
//! This module defines traits for common algebraic structures and functional abstractions,
//! such as [`Functor`], [`Applicative`] and [`Monad`].
//!
//! Traits representing higher-kinded types (e.g., `Functor`) are implemented by
//! [`Brand` types][crate::brands] to simulate higher-kinded polymorphism, as Rust does not
//! natively support it.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = map::<OptionBrand, _, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

/// Applicative functors, allowing for values and functions to be wrapped and applied within a context.
pub mod applicative;

/// Sequencing of two computations while keeping the result of the first.
pub mod apply_first;

/// Sequencing of two computations while keeping the result of the second.
pub mod apply_second;

/// Types that can be mapped over two type arguments simultaneously.
pub mod bifunctor;

/// Categories, which are semigroupoids with an identity element for each object.
pub mod category;

/// Cloneable wrappers over closures for generic handling of functions in higher-kinded contexts.
pub mod cloneable_fn;

/// Data structures that can be compacted by filtering out [`None`] or separated by splitting [`Result`] values.
pub mod compactable;

/// Types that can be constructed lazily from a computation.
pub mod deferrable;

/// Functors whose effects can be evaluated to produce an inner value.
pub mod evaluable;

/// Data structures that can be filtered and partitioned based on predicates or mapping functions.
pub mod filterable;

/// Data structures that can be folded into a single value from the left or right.
pub mod foldable;

/// Wrappers over closures for generic handling of functions in higher-kinded contexts.
pub mod function;

/// Types that can be mapped over, allowing functions to be applied to values within a context.
pub mod functor;

/// Lifting of binary functions to operate on values within a context.
pub mod lift;

/// Monads, allowing for sequencing computations where the structure depends on previous results.
pub mod monad;

/// Monads that support stack-safe tail recursion via the [`Step`](crate::types::Step) type.
pub mod monad_rec;

/// Types that have an identity element and an associative binary operation.
pub mod monoid;

/// Data structures that can be folded in parallel using thread-safe functions.
pub mod par_foldable;

/// Contexts that can be initialized with a value via the [`pure`](crate::functions::pure) operation.
pub mod pointed;

/// Hierarchy of traits for abstracting over different types of pointers and their capabilities.
pub mod pointer;

/// Reference-counted pointers with shared ownership and unwrapping capabilities.
pub mod ref_counted_pointer;

/// Types that can be mapped over by receiving or returning references to their contents.
pub mod ref_functor;

/// Applying functions within a context to values within a context, without an identity element.
pub mod semiapplicative;

/// Types that support an associative binary operation.
pub mod semigroup;

/// Semigroupoids, representing objects and composable relationships (morphisms) between them.
pub mod semigroupoid;

/// Sequencing of computations where the structure depends on previous results, without an identity element.
pub mod semimonad;

/// Thread-safe cloneable wrappers over closures that carry `Send + Sync` bounds.
pub mod send_cloneable_fn;

/// Deferred lazy evaluation using thread-safe thunks.
pub mod send_deferrable;

/// Thread-safe reference-counted pointers that carry `Send + Sync` bounds.
pub mod send_ref_counted_pointer;

/// Pointer brands that can perform unsized coercion to thread-safe `dyn Fn` trait objects.
pub mod send_unsized_coercible;

/// Data structures that can be traversed, accumulating results in an applicative context.
pub mod traversable;

/// Pointer brands that can perform unsized coercion to `dyn Fn` trait objects.
pub mod unsized_coercible;

/// Data structures that can be traversed and filtered simultaneously in an applicative context.
pub mod witherable;

// Automatically re-export all traits defined in submodules.
fp_macros::generate_trait_re_exports!("src/classes", {});
