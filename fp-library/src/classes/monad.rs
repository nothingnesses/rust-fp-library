//! Monads, allowing for sequencing computations where the structure depends on previous results.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! // Monad combines Pointed (pure) and Semimonad (bind)
//! let x = pure::<OptionBrand, _>(5);
//! let y = bind::<OptionBrand, _, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
//! assert_eq!(y, Some(10));
//! ```

use super::{applicative::Applicative, semimonad::Semimonad};

/// A type class for monads, allowing for sequencing computations where the structure of the computation depends on the result of the previous computation.
///
/// ### Type Signature
///
/// `class (Applicative m, Semimonad m) => Monad m`
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// // Monad combines Pointed (pure) and Semimonad (bind)
/// let x = pure::<OptionBrand, _>(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub trait Monad: Applicative + Semimonad {}

impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}
