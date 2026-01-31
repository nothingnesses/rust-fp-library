//! A type class for types that can be mapped over, returning references.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
//! let mapped = ref_map::<LazyBrand<RcLazyConfig>, _, _, _>(
//!     |x: &i32| *x * 2,
//!     memo
//! );
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

use crate::{Apply, kinds::*};

/// A type class for types that can be mapped over, returning references.
///
/// This is a variant of `Functor` for types where `map` receives/returns references.
/// This is required for types like `Lazy` where `get()` returns `&A`, not `A`.
pub trait RefFunctor: Kind_cdc7cd43dac7585f {
	/// Maps a function over the values in the functor context, where the function takes a reference.
	///
	/// ### Type Signature
	///
	/// `forall f b a. RefFunctor f => (a -> b, f a) -> f b`
	///
	/// ### Type Parameters
	///
	/// * `B`: The type of the result(s) of applying the function.
	/// * `A`: The type of the value(s) inside the functor.
	/// * `F`: The type of the function to apply.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the value(s) inside the functor.
	/// * `fa`: The functor instance containing the value(s).
	///
	/// ### Returns
	///
	/// A new functor instance containing the result(s) of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, types::*};
	///
	/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
	/// let mapped = LazyBrand::<RcLazyConfig>::ref_map(
	///     |x: &i32| *x * 2,
	///     memo
	/// );
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	fn ref_map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: FnOnce(&A) -> B + 'a;
}

/// Maps a function over the values in the functor context, where the function takes a reference.
///
/// Free function version that dispatches to [the type class' associated function][`RefFunctor::ref_map`].
///
/// ### Type Signature
///
/// `forall f b a. RefFunctor f => (a -> b, f a) -> f b`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the functor.
/// * `B`: The type of the result(s) of applying the function.
/// * `A`: The type of the value(s) inside the functor.
/// * `F`: The type of the function to apply.
///
/// ### Parameters
///
/// * `f`: The function to apply to the value(s) inside the functor.
/// * `fa`: The functor instance containing the value(s).
///
/// ### Returns
///
/// A new functor instance containing the result(s) of applying the function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
/// let mapped = ref_map::<LazyBrand<RcLazyConfig>, _, _, _>(
///     |x: &i32| *x * 2,
///     memo
/// );
/// assert_eq!(*mapped.evaluate(), 20);
/// ```
pub fn ref_map<'a, Brand: RefFunctor, A: 'a, B: 'a, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
where
	Func: FnOnce(&A) -> B + 'a,
{
	Brand::ref_map::<A, B, Func>(func, fa)
}
