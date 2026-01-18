//! Semimonad type class.
//!
//! This module defines the [`Semimonad`] trait, which allows for sequencing computations where the second computation depends on the result of the first.

use crate::{Apply, kinds::*};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
pub trait Semimonad: Kind_cdc7cd43dac7585f {
	/// Sequences two computations, allowing the second to depend on the value computed by the first.
	///
	/// This method chains two computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semimonad m => (m a, a -> m b) -> m b`
	///
	/// ### Type Parameters
	///
	/// * `F`: The type of the function to apply.
	/// * `A`: The type of the result of the first computation.
	/// * `B`: The type of the result of the second computation.
	///
	/// ### Parameters
	///
	/// * `ma`: The first computation.
	/// * `f`: The function to apply to the result of the first computation.
	///
	/// ### Returns
	///
	/// The result of the second computation.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::Semimonad;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::bind(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: F,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
///
/// ### Type Signature
///
/// `forall a b. Semimonad m => (m a, a -> m b) -> m b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the semimonad.
/// * `F`: The type of the function to apply.
/// * `A`: The type of the result of the first computation.
/// * `B`: The type of the result of the second computation.
///
/// ### Parameters
///
/// * `ma`: The first computation.
/// * `f`: The function to apply to the result of the first computation.
///
/// ### Returns
///
/// The result of the second computation.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::semimonad::bind;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub fn bind<'a, Brand: Semimonad, F, A: 'a, B: 'a>(
	ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	f: F,
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	F: Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
{
	Brand::bind(ma, f)
}
