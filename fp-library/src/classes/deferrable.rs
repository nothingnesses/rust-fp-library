//! Types that can be constructed lazily from a computation.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let eval: Thunk<i32> = defer::<ThunkBrand, i32, RcFnBrand>(
//!     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
//! );
//! assert_eq!(eval.evaluate(), 42);
//! ```

use super::CloneableFn;
use crate::{Apply, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for types that can be constructed lazily.
pub trait Deferrable: Kind_cdc7cd43dac7585f {
	/// Creates a value from a computation that produces the value.
	///
	/// This function takes a thunk (wrapped in a cloneable function) and creates a deferred value that will be computed using the thunk.
	///
	/// ### Type Signature
	///
	#[hm_signature(Deferrable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the computation.",
		"The type of the deferred value.",
		"The brand of the cloneable function wrapper."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("A thunk (wrapped in a cloneable function) that produces the value.")]
	///
	/// ### Returns
	///
	/// The deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let eval: Thunk<i32> = defer::<ThunkBrand, i32, RcFnBrand>(
	///     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
	/// );
	/// assert_eq!(eval.evaluate(), 42);
	/// ```
	fn defer<'a, A: 'a, FnBrand: 'a + CloneableFn>(
		f: <FnBrand as CloneableFn>::Of<
			'a,
			(),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		A: Clone;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Deferrable::defer`].
///
/// ### Type Signature
///
/// #[hm_signature(Deferrable)]
///
/// ### Type Parameters
///
/// #[doc_type_params("The lifetime of the computation", "The brand of the deferred type.", "The type of the deferred value.", "The brand of the cloneable function wrapper.")]
///
/// ### Parameters
///
/// #[doc_params("A thunk (wrapped in a cloneable function) that produces the value.")]
///
/// ### Returns
///
/// The deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let eval: Thunk<i32> = defer::<ThunkBrand, i32, RcFnBrand>(
///     cloneable_fn_new::<RcFnBrand, _, _>(|_| Thunk::new(|| 42))
/// );
/// assert_eq!(eval.evaluate(), 42);
/// ```
pub fn defer<'a, Brand, A, FnBrand>(
	f: <FnBrand as CloneableFn>::Of<
		'a,
		(),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	>
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
where
	Brand: Deferrable,
	FnBrand: 'a + CloneableFn,
	A: Clone + 'a,
{
	Brand::defer::<A, FnBrand>(f)
}
