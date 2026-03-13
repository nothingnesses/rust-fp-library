//! Defines traits for common algebraic structures and functional abstractions,
//! such as [`Functor`], [`Applicative`] and [`Monad`].
//!
//! Traits representing higher-kinded types (e.g., `Functor`) are implemented by
//! [`Brand` types][crate::brands] to simulate higher-kinded polymorphism, as Rust does not
//! natively support it.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = map::<OptionBrand, _, _>(|i| i * 2, x);
//! assert_eq!(y, Some(10));
//! ```

pub mod alt;
pub mod alternative;
pub mod applicative;
pub mod apply_first;
pub mod apply_second;
pub mod bifoldable;
pub mod bifunctor;
pub mod bitraversable;
pub mod category;
pub mod cloneable_fn;
pub mod commutative_ring;
pub mod compactable;
pub mod contravariant;
pub mod deferrable;
pub mod division_ring;
pub mod euclidean_ring;
pub mod evaluable;
pub mod field;
pub mod filterable;
pub mod foldable;
pub mod foldable_with_index;
pub mod function;
pub mod functor;
pub mod functor_with_index;
pub mod heyting_algebra;
pub mod lift;
pub mod monad;
pub mod monad_rec;
pub mod monoid;
pub mod optics;
pub mod par_compactable;
pub mod par_filterable;
pub mod par_foldable;
pub mod par_foldable_with_index;
pub mod par_functor;
pub mod par_functor_with_index;
pub mod pipe;
pub mod plus;
pub mod pointed;
pub mod pointer;
pub mod profunctor;
pub mod ref_counted_pointer;
pub mod ref_functor;
pub mod ring;
pub mod semiapplicative;
pub mod semigroup;
pub mod semigroupoid;
pub mod semimonad;
pub mod semiring;
pub mod send_cloneable_fn;
pub mod send_deferrable;
pub mod send_ref_counted_pointer;
pub mod send_unsized_coercible;
pub mod traversable;
pub mod traversable_with_index;
pub mod unsized_coercible;
pub mod with_index;
pub mod witherable;

// Automatically re-export all traits defined in submodules.
fp_macros::generate_trait_re_exports!("src/classes", {});
