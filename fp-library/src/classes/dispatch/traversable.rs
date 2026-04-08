//! Dispatch for [`Traversable::traverse`](crate::classes::Traversable::traverse) and
//! [`RefTraversable::ref_traverse`](crate::classes::RefTraversable::ref_traverse).
//!
//! Provides the [`TraverseDispatch`] trait and a unified [`traverse`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Owned: dispatches to Traversable::traverse
//! let y = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Some(5));
//! assert_eq!(y, Some(Some(10)));
//!
//! // By-ref: dispatches to RefTraversable::ref_traverse
//! let y =
//! 	traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: &i32| Some(*x * 2), Some(5));
//! assert_eq!(y, Some(Some(10)));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Applicative,
				LiftFn,
				RefTraversable,
				Traversable,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a traverse operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand for the computation.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait TraverseDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		F: Kind_cdc7cd43dac7585f,
		Marker,
	> {
		/// Perform the dispatched traverse operation.
		#[document_signature]
		///
		#[document_parameters("The structure to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result =
		/// 	traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Some(5));
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch_traverse(
			self,
			ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>);
	}

	// -- Val: Fn(A) -> F<B> -> Traversable::traverse --

	/// Routes `Fn(A) -> F::Of<B>` closures to [`Traversable::traverse`].
	///
	/// The `FnBrand` parameter is unused by the Val path but is accepted for
	/// uniformity with the Ref path.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand (unused by Val path).",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, FnBrand, Brand, A, B, F, Func> TraverseDispatch<'a, FnBrand, Brand, A, B, F, Val> for Func
	where
		Brand: Traversable,
		A: 'a + Clone,
		B: 'a + Clone,
		F: Applicative,
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters("The structure to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result =
		/// 	traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Some(5));
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch_traverse(
			self,
			ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		{
			Brand::traverse::<A, B, F>(self, ta)
		}
	}

	// -- Ref: Fn(&A) -> F<B> -> RefTraversable::ref_traverse --

	/// Routes `Fn(&A) -> F::Of<B>` closures to [`RefTraversable::ref_traverse`].
	///
	/// The `FnBrand` parameter is passed through to the underlying
	/// [`ref_traverse`](RefTraversable::ref_traverse) call, allowing callers
	/// to choose between [`RcFnBrand`](crate::brands::RcFnBrand) and
	/// [`ArcFnBrand`](crate::brands::ArcFnBrand).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, FnBrand, Brand, A, B, F, Func> TraverseDispatch<'a, FnBrand, Brand, A, B, F, Ref> for Func
	where
		Brand: RefTraversable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		F: Applicative,
		Func: Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters("The structure to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result =
		/// 	traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: &i32| Some(*x * 2), Some(5));
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch_traverse(
			self,
			ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		{
			Brand::ref_traverse::<FnBrand, A, B, F>(self, ta)
		}
	}

	// -- Unified free function --

	/// Traverses a structure, mapping each element to a computation and combining the results.
	///
	/// Dispatches to either [`Traversable::traverse`] or
	/// [`RefTraversable::ref_traverse`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(A) -> F::Of<B>`), dispatches to
	///   [`Traversable::traverse`]. The `FnBrand` parameter is unused but must
	///   be specified for uniformity.
	/// - If the closure takes references (`Fn(&A) -> F::Of<B>`), dispatches to
	///   [`RefTraversable::ref_traverse`]. The `FnBrand` parameter is passed
	///   through as the function brand.
	///
	/// The `Marker` type parameter is inferred automatically by the compiler
	/// from the closure's argument type. Callers write
	/// `traverse::<FnBrand, Brand, _, _, F, _>(...)` and never need to specify
	/// `Marker` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a value in an applicative context.",
		"The traversable structure."
	)]
	///
	#[document_returns("The structure wrapped in the applicative context.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned: dispatches to Traversable::traverse
	/// let y = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Some(5));
	/// assert_eq!(y, Some(Some(10)));
	///
	/// // By-ref: dispatches to RefTraversable::ref_traverse
	/// let y =
	/// 	traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _>(|x: &i32| Some(*x * 2), Some(5));
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn traverse<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		F: Kind_cdc7cd43dac7585f,
		Marker,
	>(
		func: impl TraverseDispatch<'a, FnBrand, Brand, A, B, F, Marker>,
		ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	{
		func.dispatch_traverse(ta)
	}
}

pub use inner::*;
