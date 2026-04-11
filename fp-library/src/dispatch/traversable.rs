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
//! let y = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
//! 	|x: i32| Some(x * 2),
//! 	Some(5),
//! );
//! assert_eq!(y, Some(Some(10)));
//!
//! // By-ref: dispatches to RefTraversable::ref_traverse
//! let y = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
//! 	|x: &i32| Some(*x * 2),
//! 	&Some(5),
//! );
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
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a traverse operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FTA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand for the computation.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait TraverseDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		F: Kind_cdc7cd43dac7585f,
		FTA,
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
		/// let result = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
		/// 	|x: i32| Some(x * 2),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch(
			self,
			ta: FTA,
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
	impl<'a, FnBrand, Brand, A, B, F, Func>
		TraverseDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			F,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for Func
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
		/// let result = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
		/// 	|x: i32| Some(x * 2),
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch(
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
	///
	/// The container must be passed by reference (`&ta`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, FnBrand, Brand, A, B, F, Func>
		TraverseDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			F,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for Func
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
		#[document_parameters("A reference to the structure to traverse.")]
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
		/// let result = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
		/// 	|x: &i32| Some(*x * 2),
		/// 	&Some(5),
		/// );
		/// assert_eq!(result, Some(Some(10)));
		/// ```
		fn dispatch(
			self,
			ta: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
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
	/// - If the closure takes owned values (`Fn(A) -> F::Of<B>`) and the
	///   container is owned, dispatches to [`Traversable::traverse`]. The
	///   `FnBrand` parameter is unused but must be specified for uniformity.
	/// - If the closure takes references (`Fn(&A) -> F::Of<B>`) and the
	///   container is borrowed (`&ta`), dispatches to
	///   [`RefTraversable::ref_traverse`]. The `FnBrand` parameter is passed
	///   through as the function brand.
	///
	/// The `Marker` and `FTA` type parameters are inferred automatically by
	/// the compiler from the closure's argument type and the container
	/// argument. Callers write
	/// `traverse_explicit::<FnBrand, Brand, _, _, F, _, _>(...)` and never need to
	/// specify `Marker` or `FTA` explicitly.
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
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a value in an applicative context.",
		"The traversable structure (owned for Val, borrowed for Ref)."
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
	/// let y = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
	/// 	|x: i32| Some(x * 2),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some(Some(10)));
	///
	/// // By-ref: dispatches to RefTraversable::ref_traverse
	/// let y = traverse_explicit::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(
	/// 	|x: &i32| Some(*x * 2),
	/// 	&Some(5),
	/// );
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn traverse<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		B: 'a,
		F: Kind_cdc7cd43dac7585f,
		FTA,
		Marker,
	>(
		func: impl TraverseDispatch<'a, FnBrand, Brand, A, B, F, FTA, Marker>,
		ta: FTA,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	{
		func.dispatch(ta)
	}
}

pub use inner::*;
