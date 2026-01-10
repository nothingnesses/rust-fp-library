use super::{applicative::Applicative, semimonad::Semimonad};

/// A type class for monads.
///
/// `Monad` extends [`Applicative`] and [`Semimonad`].
/// It allows for sequencing computations where the structure of the computation depends on the result of the previous computation.
///
/// # Type Signature
///
/// `class (Applicative m, Semimonad m) => Monad m`
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::monad::Monad;
/// use fp_library::v2::classes::pointed::pure;
/// use fp_library::v2::classes::semimonad::bind;
/// use fp_library::brands::OptionBrand;
///
/// // Monad combines Pointed (pure) and Semimonad (bind)
/// let x = pure::<OptionBrand, _>(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| pure::<OptionBrand, _>(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub trait Monad: Applicative + Semimonad {}

impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}
