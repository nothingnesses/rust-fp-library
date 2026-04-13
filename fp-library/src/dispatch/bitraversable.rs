//! Dispatch for [`Bitraversable::bi_traverse`](crate::classes::Bitraversable::bi_traverse) and
//! [`RefBitraversable::ref_bi_traverse`](crate::classes::RefBitraversable::ref_bi_traverse).
//!
//! Provides the [`BiTraverseDispatch`] trait and a unified
//! [`explicit::bi_traverse`] free function that routes to the appropriate trait
//! method based on the closures' argument types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // Owned: dispatches to Bitraversable::bi_traverse
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
//! 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
//! 	x,
//! );
//! assert_eq!(y, Some(Ok(10)));
//!
//! // By-ref: dispatches to RefBitraversable::ref_bi_traverse
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
//! 	(|e: &i32| Some(e + 1), |s: &i32| Some(s * 2)),
//! 	&x,
//! );
//! assert_eq!(y, Some(Ok(10)));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Applicative,
				Bitraversable,
				LiftFn,
				RefBitraversable,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a bi_traverse operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closures' argument types; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bitraversable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The output type for first-position elements.",
		"The output type for second-position elements.",
		"The applicative functor brand for the computation.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure tuple implementing this dispatch.")]
	pub trait BiTraverseDispatch<
		'a,
		FnBrand,
		Brand: Kind_266801a817966495,
		A: 'a,
		B: 'a,
		C: 'a,
		D: 'a,
		F: Kind_cdc7cd43dac7585f,
		FA,
		Marker,
	> {
		/// Perform the dispatched bi_traverse operation.
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// let result = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
		/// 	x,
		/// );
		/// assert_eq!(result, Some(Ok(10)));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>);
	}

	// -- Val: (Fn(A) -> F<C>, Fn(B) -> F<D>) -> Bitraversable::bi_traverse --

	/// Routes `(Fn(A) -> F::Of<C>, Fn(B) -> F::Of<D>)` closure tuples to [`Bitraversable::bi_traverse`].
	///
	/// The `FnBrand` parameter is unused by the Val path but is accepted for
	/// uniformity with the Ref path.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand (unused by Val path).",
		"The brand of the bitraversable structure.",
		"The first input type.",
		"The second input type.",
		"The first output type.",
		"The second output type.",
		"The applicative functor brand.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, FnBrand, Brand, A, B, C, D, F, Func1, Func2>
		BiTraverseDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			C,
			D,
			F,
			Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Val,
		> for (Func1, Func2)
	where
		Brand: Bitraversable,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
		Func1: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		Func2: Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// let result = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
		/// 	x,
		/// );
		/// assert_eq!(result, Some(Ok(10)));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		{
			Brand::bi_traverse::<A, B, C, D, F>(self.0, self.1, fa)
		}
	}

	// -- Ref: (Fn(&A) -> F<C>, Fn(&B) -> F<D>) -> RefBitraversable::ref_bi_traverse --

	/// Routes `(Fn(&A) -> F::Of<C>, Fn(&B) -> F::Of<D>)` closure tuples to [`RefBitraversable::ref_bi_traverse`].
	///
	/// The `FnBrand` parameter is passed through to the underlying
	/// [`ref_bi_traverse`](RefBitraversable::ref_bi_traverse) call.
	///
	/// The container must be passed by reference (`&p`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The brand of the bitraversable structure.",
		"The first input type.",
		"The second input type.",
		"The first output type.",
		"The second output type.",
		"The applicative functor brand.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, 'b, FnBrand, Brand, A, B, C, D, F, Func1, Func2>
		BiTraverseDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			C,
			D,
			F,
			&'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Ref,
		> for (Func1, Func2)
	where
		Brand: RefBitraversable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a + Clone,
		D: 'a + Clone,
		F: Applicative,
		Func1: Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		Func2: Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone,
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let x: Result<i32, i32> = Ok(5);
		/// let result = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: &i32| Some(e + 1), |s: &i32| Some(s * 2)),
		/// 	&x,
		/// );
		/// assert_eq!(result, Some(Ok(10)));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		{
			Brand::ref_bi_traverse::<FnBrand, A, B, C, D, F>(self.0, self.1, fa)
		}
	}

	// -- Explicit dispatch free function --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Traverses a bitraversable structure, mapping each element to a computation and combining the results.
		///
		/// Dispatches to either [`Bitraversable::bi_traverse`] or
		/// [`RefBitraversable::ref_bi_traverse`] based on the closures' argument types:
		///
		/// - If the closures take owned values and the container is owned,
		///   dispatches to [`Bitraversable::bi_traverse`]. The `FnBrand` parameter
		///   is unused but must be specified for uniformity.
		/// - If the closures take references and the container is borrowed,
		///   dispatches to [`RefBitraversable::ref_bi_traverse`]. The `FnBrand`
		///   parameter is passed through as the function brand.
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by
		/// the compiler from the closures' argument types and the container
		/// argument.
		///
		/// The dispatch is resolved at compile time with no runtime cost.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The brand of the bitraversable structure.",
			"The type of the first-position elements.",
			"The type of the second-position elements.",
			"The output type for first-position elements.",
			"The output type for second-position elements.",
			"The applicative functor brand.",
			"The container type (owned or borrowed), inferred from the argument.",
			"Dispatch marker type, inferred automatically."
		)]
		///
		#[document_parameters(
			"A tuple of (first function, second function), each returning a value in an applicative context.",
			"The bitraversable structure (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns("The structure wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned: dispatches to Bitraversable::bi_traverse
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: i32| Some(e + 1), |s: i32| Some(s * 2)),
		/// 	x,
		/// );
		/// assert_eq!(y, Some(Ok(10)));
		///
		/// // By-ref: dispatches to RefBitraversable::ref_bi_traverse
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_traverse::<RcFnBrand, ResultBrand, _, _, _, _, OptionBrand, _, _>(
		/// 	(|e: &i32| Some(e + 1), |s: &i32| Some(s * 2)),
		/// 	&x,
		/// );
		/// assert_eq!(y, Some(Ok(10)));
		/// ```
		pub fn bi_traverse<
			'a,
			FnBrand,
			Brand: Kind_266801a817966495,
			A: 'a,
			B: 'a,
			C: 'a,
			D: 'a,
			F: Kind_cdc7cd43dac7585f,
			FA,
			Marker,
		>(
			fg: impl BiTraverseDispatch<'a, FnBrand, Brand, A, B, C, D, F, FA, Marker>,
			fa: FA,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		{
			fg.dispatch(fa)
		}
	}
}

pub use inner::*;
