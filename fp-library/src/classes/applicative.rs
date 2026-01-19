//! Applicative functors.
//!
//! This module defines the [`Applicative`] trait, which extends [`Pointed`] and [`Semiapplicative`].
//! Applicative functors allow for values to be wrapped in a context and for functions within a context to be applied to values within a context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*};
//!
//! // Applicative combines Pointed (pure) and Semiapplicative (apply)
//! let f = pure::<OptionBrand, _>(clonable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = pure::<OptionBrand, _>(5);
//! let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(y, Some(10));
//! ```

use super::{
	apply_first::ApplyFirst, apply_second::ApplySecond, pointed::Pointed,
	semiapplicative::Semiapplicative,
};

/// A type class for applicative functors.
///
/// `Applicative` extends [`Pointed`] and [`Semiapplicative`].
/// It allows for values to be wrapped in a context and for functions within a context to be applied to values within a context.
///
/// ### Type Signature
///
/// `class (Pointed f, Semiapplicative f) => Applicative f`
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// // Applicative combines Pointed (pure) and Semiapplicative (apply)
/// let f = pure::<OptionBrand, _>(clonable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
/// let x = pure::<OptionBrand, _>(5);
/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
/// assert_eq!(y, Some(10));
/// ```
pub trait Applicative: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}

impl<Brand> Applicative for Brand where Brand: Pointed + Semiapplicative + ApplyFirst + ApplySecond {}
