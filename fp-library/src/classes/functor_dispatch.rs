//! Unified dispatch for mapping operations.
//!
//! Provides a single [`map`] free function that dispatches to either
//! [`Functor::map`](crate::classes::Functor::map) (when the closure takes owned values)
//! or [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map) (when the closure
//! takes references), using marker-type dispatch resolved by the compiler.
//!
//! ## How it works
//!
//! The dispatch uses two zero-sized marker types ([`Val`] and [`Ref`]) and a
//! [`FunctorDispatch`] trait with separate blanket implementations for each marker.
//! The compiler selects the correct implementation based on the closure's
//! argument type:
//!
//! - A closure `Fn(A) -> B` satisfies `FunctorDispatch<..., Val>`, which calls
//!   [`Functor::map`](crate::classes::Functor::map).
//! - A closure `Fn(&A) -> B` satisfies `FunctorDispatch<..., Ref>`, which calls
//!   [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map).
//!
//! The `Marker` type parameter is inferred automatically. Callers write
//! `map::<Brand, _, _, _>(...)` and never need to specify the marker
//! explicitly. The dispatch is resolved at compile time with no runtime cost.
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
//! // Closure takes owned i32 -> dispatches to Functor::map
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
	use {
		crate::{
			classes::{
				Functor,
				RefFunctor,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- Marker types --

	/// Marker type indicating the closure receives owned values.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `A` (not `&A`). Routes to [`Functor::map`](crate::classes::Functor::map).
	pub struct Val;

	/// Marker type indicating the closure receives references.
	///
	/// Selected automatically by the compiler when the closure's argument
	/// type is `&A`. Routes to [`RefFunctor::ref_map`](crate::classes::RefFunctor::ref_map).
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

	// -- Dispatch trait --

	/// Trait that routes a map operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically. Either [`Val`] or [`Ref`]."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FunctorDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched map operation.
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(A) -> B -> Functor::map --

	/// Routes `Fn(A) -> B` closures to [`Functor::map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, Val> for F
	where
		Brand: Functor,
		A: 'a,
		B: 'a,
		F: Fn(A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::map(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> B -> RefFunctor::ref_map --

	/// Routes `Fn(&A) -> B` closures to [`RefFunctor::ref_map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, Ref> for F
	where
		Brand: RefFunctor,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::pure(10);
		/// let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
		/// assert_eq!(*result.evaluate(), 20);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_map(self, fa)
		}
	}

	// -- Unified free function --

	/// Maps a function over the values in a functor context.
	///
	/// Dispatches to either [`Functor::map`] or [`RefFunctor::ref_map`]
	/// based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(A) -> B`), dispatches to
	///   [`Functor::map`].
	/// - If the closure takes references (`Fn(&A) -> B`), dispatches to
	///   [`RefFunctor::ref_map`].
	///
	/// The `Marker` type parameter is inferred automatically by the compiler
	/// from the closure's argument type. Callers write `map::<Brand, _, _, _>(...)`
	/// and never need to specify `Marker` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // Owned: dispatches to Functor::map
	/// let y = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
	/// assert_eq!(y, Some(10));
	///
	/// // By-ref: dispatches to RefFunctor::ref_map
	/// let lazy = RcLazy::pure(10);
	/// let mapped = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	pub fn map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		f: impl FunctorDispatch<'a, Brand, A, B, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch(fa)
	}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::map,
		crate::{
			brands::*,
			types::*,
		},
	};

	#[test]
	fn test_val_option() {
		let result = map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn test_val_vec() {
		let result = map::<VecBrand, _, _, _>(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	#[test]
	fn test_ref_lazy() {
		let lazy = RcLazy::pure(10);
		let result = map::<LazyBrand<RcLazyConfig>, _, _, _>(|x: &i32| *x * 2, lazy);
		assert_eq!(*result.evaluate(), 20);
	}

	#[test]
	fn test_val_none() {
		let result = map::<OptionBrand, i32, i32, _>(|x| x * 2, None);
		assert_eq!(result, None);
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
		classes::functor_dispatch::inner::FunctorDispatch,
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
		FA: DefaultBrand + 'a,
		<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>, {
		f.dispatch(fa)
	}

	// -- Val dispatch (Functor::map) --

	#[test]
	fn infer_option_val() {
		let result = map_infer(|x: i32| x * 2, Some(5));
		assert_eq!(result, Some(10));
	}

	#[test]
	fn infer_option_none() {
		let result = map_infer(|x: i32| x * 2, None::<i32>);
		assert_eq!(result, None);
	}

	#[test]
	fn infer_vec_val() {
		let result = map_infer(|x: i32| x + 1, vec![1, 2, 3]);
		assert_eq!(result, vec![2, 3, 4]);
	}

	#[test]
	fn infer_vec_strings() {
		let result = map_infer(|x: i32| x.to_string(), vec![1, 2]);
		assert_eq!(result, vec!["1", "2"]);
	}

	// -- Ref dispatch (RefFunctor::ref_map) --

	#[test]
	fn infer_lazy_ref() {
		let lazy = RcLazy::pure(10);
		let result = map_infer(|x: &i32| *x * 2, lazy);
		assert_eq!(*result.evaluate(), 20);
	}

	// Note: ArcLazy implements SendRefFunctor, not RefFunctor, so it
	// cannot be dispatched via FunctorDispatch's Ref path. This will be
	// resolved when the ref-hierarchy plan adds SendRefFunctor to the
	// dispatch system.
}
