//! Defer type class.
//!
//! This module defines the [`Defer`] trait, which provides an abstraction for types that can be constructed lazily.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*, types::*};
//! use fp_library::types::lazy::{RcLazy, RcLazyConfig, LazyConfig};
//!
//! let lazy = defer::<RcLazy<'_, i32>, RcFnBrand>(
//!     clonable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
//! );
//! assert_eq!(Lazy::force(&lazy).unwrap(), &42);
//! ```

use super::clonable_fn::ClonableFn;

/// A type class for types that can be constructed lazily.
pub trait Defer<'a> {
	/// Creates a value from a computation that produces the value.
	///
	/// This function takes a thunk (wrapped in a clonable function) and creates a deferred value that will be computed using the thunk.
	///
	/// ### Type Signature
	///
	/// `forall a. Defer d => (() -> d a) -> d a`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function wrapper.
	///
	/// ### Parameters
	///
	/// * `f`: A thunk (wrapped in a clonable function) that produces the value.
	///
	/// ### Returns
	///
	/// The deferred value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	/// use fp_library::types::lazy::{RcLazy, RcLazyConfig, LazyConfig};
	///
	/// let lazy = defer::<RcLazy<'_, i32>, RcFnBrand>(
	///     clonable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force(&lazy).unwrap(), &42);
	/// ```
	fn defer<FnBrand: 'a + ClonableFn>(f: <FnBrand as ClonableFn>::Of<'a, (), Self>) -> Self
	where
		Self: Sized;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Defer::defer`].
///
/// ### Type Signature
///
/// `forall a. Defer d => (() -> d a) -> d a`
///
/// ### Type Parameters
///
/// * `D`: The type of the deferred value.
/// * `FnBrand`: The brand of the clonable function wrapper.
///
/// ### Parameters
///
/// * `f`: A thunk (wrapped in a clonable function) that produces the value.
///
/// ### Returns
///
/// The deferred value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*, types::*};
/// use fp_library::types::lazy::{RcLazy, RcLazyConfig, LazyConfig};
///
/// let lazy = defer::<RcLazy<'_, i32>, RcFnBrand>(
///     clonable_fn_new::<RcFnBrand, _, _>(|_| RcLazy::new(RcLazyConfig::new_thunk(|_| 42)))
/// );
/// assert_eq!(Lazy::force(&lazy).unwrap(), &42);
/// ```
pub fn defer<'a, D, FnBrand>(f: <FnBrand as ClonableFn>::Of<'a, (), D>) -> D
where
	D: Defer<'a>,
	FnBrand: 'a + ClonableFn,
{
	D::defer::<FnBrand>(f)
}
