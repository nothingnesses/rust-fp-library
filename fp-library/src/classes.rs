//! Type classes defining shared behavior across different types.
//!
//! This module includes traits for common functional programming abstractions
//! such as `Functor`, `Monad`, `Applicative`, and others.
//!
//! Higher-kinded type classes (those with arities > 0, e.g., [`functor::Functor`],
//! which has arity 1) are usually implemented by
//! [`Brand` types][crate::brands], which are higher-kinded (arities > 0)
//! representation of [types][crate::types], instead of directly by concrete
//! types (which have arity 0).

pub mod applicative;
pub mod apply_first;
pub mod apply_second;
pub mod category;
pub mod clonable_fn;
pub mod defer;
pub mod foldable;
pub mod function;
pub mod functor;
pub mod lift;
pub mod monad;
pub mod monoid;
pub mod once;
pub mod pointed;
pub mod semiapplicative;
pub mod semigroup;
pub mod semigroupoid;
pub mod semimonad;
pub mod send_clonable_fn;
pub mod traversable;
