//! Functional programming type classes.
//!
//! This module defines traits for common algebraic structures and functional abstractions,
//! such as [`Functor`][functor::Functor], [`Applicative`][applicative::Applicative] and [`Monad`][monad::Monad].
//!
//! Traits representing higher-kinded types (e.g., `Functor`) are implemented by
//! [`Brand` types][crate::brands] to simulate higher-kinded polymorphism, as Rust does not
//! natively support it.

pub mod applicative;
pub mod apply_first;
pub mod apply_second;
pub mod category;
pub mod clonable_fn;
pub mod compactable;
pub mod defer;
pub mod filterable;
pub mod foldable;
pub mod function;
pub mod functor;
pub mod lift;
pub mod monad;
pub mod monoid;
pub mod once;
pub mod par_foldable;
pub mod pointed;
pub mod semiapplicative;
pub mod semigroup;
pub mod semigroupoid;
pub mod semimonad;
pub mod send_clonable_fn;
pub mod traversable;
pub mod witherable;
