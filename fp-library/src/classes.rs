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

pub mod applicative;
pub mod apply_first;
pub mod apply_second;
pub mod bifunctor;
pub mod category;
pub mod cloneable_fn;
pub mod compactable;
pub mod deferrable;
pub mod evaluable;
pub mod filterable;
pub mod foldable;
pub mod function;
pub mod functor;
pub mod lift;
pub mod monad;
pub mod monad_rec;
pub mod monoid;
pub mod par_foldable;
pub mod pointed;
pub mod pointer;
pub mod ref_counted_pointer;
pub mod ref_functor;
pub mod semiapplicative;
pub mod semigroup;
pub mod semigroupoid;
pub mod semimonad;
pub mod send_cloneable_fn;
pub mod send_deferrable;
pub mod send_ref_counted_pointer;
pub mod send_unsized_coercible;
pub mod traversable;
pub mod unsized_coercible;
pub mod witherable;

// Automatically re-export all traits defined in submodules.
fp_macros::generate_trait_re_exports!("src/classes", {});
