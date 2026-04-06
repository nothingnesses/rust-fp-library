//! Dispatch infrastructure for unified free functions that route to either
//! by-value or by-reference trait methods based on the closure's argument type.
//!
//! The dispatch system uses marker types ([`Val`] and [`Ref`]) to select the
//! appropriate trait at compile time. The compiler infers the marker from the
//! closure's argument type: `Fn(A) -> B` resolves to [`Val`], `Fn(&A) -> B`
//! resolves to [`Ref`].
//!
//! The [`ClosureMode`] trait maps each marker to the corresponding `dyn Fn`
//! trait object type, used by [`CloneFn`](crate::classes::CloneFn) and
//! [`SendCloneFn`](crate::classes::SendCloneFn) to parameterize the `Deref`
//! target of wrapped closures.
//!
//! ### Sub-modules
//!
//! Each sub-module provides a dispatch trait and unified free function for
//! a specific type class operation, mirroring the corresponding `classes/`
//! module:
//!
//! - [`functor`]: `FunctorDispatch` + `map`
//! - [`semimonad`]: `BindDispatch` + `bind`
//! - [`lift`]: `Lift2Dispatch`-`Lift5Dispatch` + `lift2`-`lift5`
//! - [`foldable`]: `FoldRightDispatch`, `FoldLeftDispatch`, `FoldMapDispatch` + `fold_right`, `fold_left`, `fold_map`
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! // Closure takes i32 -> dispatches to Functor::map
//! let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // Closure takes &i32 -> dispatches to RefFunctor::ref_map
//! let lazy = RcLazy::pure(10);
//! let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
mod inner {
	// -- Marker types --

	/// Marker type indicating the closure receives owned values.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `A` (not `&A`). Routes to by-value trait methods
	/// (e.g., [`Functor::map`](crate::classes::Functor::map),
	/// [`Semimonad::bind`](crate::classes::Semimonad::bind)).
	pub struct Val;

	/// Marker type indicating the closure receives references.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `&A`. Routes to by-reference trait methods
	/// (e.g., [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map),
	/// [`RefSemimonad::ref_bind`](crate::classes::RefSemimonad::ref_bind)).
	pub struct Ref;

	// -- Closure mode --

	/// Trait that maps a closure mode marker ([`Val`] or [`Ref`]) to the
	/// corresponding `dyn Fn` trait object type.
	///
	/// Used by [`CloneFn`](crate::classes::CloneFn) to parameterize
	/// the `Deref` target of wrapped closures. `Val` produces
	/// `dyn Fn(A) -> B` (by-value), `Ref` produces `dyn Fn(&A) -> B`
	/// (by-reference).
	pub trait ClosureMode {
		/// The unsized closure trait object type for this mode.
		type Target<'a, A: 'a, B: 'a>: ?Sized + 'a;

		/// The unsized closure trait object type for this mode with `Send + Sync` bounds.
		type SendTarget<'a, A: 'a, B: 'a>: ?Sized + 'a;
	}

	impl ClosureMode for Val {
		type SendTarget<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B + Send + Sync;
		type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B;
	}

	impl ClosureMode for Ref {
		type SendTarget<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B + Send + Sync;
		type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B;
	}
}

pub use inner::*;

pub mod foldable;
pub mod functor;
pub mod lift;
pub mod semimonad;

// Re-export dispatch free functions at the dispatch module level
// so they're accessible via `crate::classes::dispatch::map` etc.
pub use {
	foldable::{
		fold_left,
		fold_map,
		fold_right,
	},
	functor::map,
	lift::{
		lift2,
		lift3,
		lift4,
		lift5,
	},
	semimonad::bind,
};

#[cfg(test)]
mod tests {
	use {
		super::{
			functor::map,
			lift::lift2,
			semimonad::bind,
		},
		crate::{
			brands::*,
			types::*,
		},
	};

	#[test]
	fn test_val_option_map() {
		let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_vec_map() {
		let result = map::<VecBrand, _, _, _>(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	#[test]
	fn test_ref_lazy_map() {
		let lazy = RcLazy::pure(10);
		let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
		assert_eq!(*result.evaluate(), 20);
	}

	#[test]
	fn test_val_none_map() {
		let result = map::<OptionBrand, i32, i32, _>(|x| x * 2, None);
		assert_eq!(result, None);
	}

	#[test]
	fn test_val_option_bind() {
		let result = bind::<OptionBrand, _, _, _>(Some(5), |x: i32| Some(x * 2));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_option_lift2() {
		let result = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, Some(1), Some(2));
		assert_eq!(result, Some(3));
	}
}

// -- Brand inference POC --
//
// Validates that a DefaultBrand trait can enable turbofish-free map calls
// by inferring the Brand from the container's concrete type. This is a
// temporary module; the trait and function will move to their own files
// if the POC succeeds.

#[cfg(test)]
mod brand_inference_poc {
	use crate::{
		brands::*,
		classes::dispatch::functor::inner::FunctorDispatch,
		kinds::Kind_cdc7cd43dac7585f,
		types::*,
	};

	/// Reverse mapping from a concrete type to its canonical brand.
	trait DefaultBrand {
		type Brand: Kind_cdc7cd43dac7585f;
	}

	impl<A> DefaultBrand for Option<A> {
		type Brand = OptionBrand;
	}

	impl<A> DefaultBrand for Vec<A> {
		type Brand = VecBrand;
	}

	impl<'a, A: 'a, Config: crate::classes::LazyConfig + 'a> DefaultBrand for Lazy<'a, A, Config> {
		type Brand = LazyBrand<Config>;
	}

	/// Temporary inference-based map function for POC validation.
	fn map_infer<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>,
		fa: FA,
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		FA: DefaultBrand,
		FA: Into<<<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>>, {
		f.dispatch(fa.into())
	}

	#[test]
	fn test_val_option_infer() {
		let result: Option<i32> = map_infer(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_vec_infer() {
		let result: Vec<i32> = map_infer(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	// -- Val dispatch (Functor::map) --

	#[test]
	fn test_ref_lazy_infer() {
		let lazy = Lazy::<_, RcLazyConfig>::new(|| 10);
		let result: Lazy<i32, RcLazyConfig> = map_infer(|x: &i32| *x * 2, lazy);
		assert_eq!(*result.evaluate(), 20);
	}
}
