//! Dispatch for [`Bifoldable::bi_fold_map`](crate::classes::Bifoldable::bi_fold_map) and
//! [`RefBifoldable::ref_bi_fold_map`](crate::classes::RefBifoldable::ref_bi_fold_map).
//!
//! Provides the [`BiFoldMapDispatch`] trait and a unified [`bi_fold_map`] free function
//! that routes to the appropriate trait method based on the closures' argument
//! types.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Owned: dispatches to Bifoldable::bi_fold_map
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
//! 	(|e: i32| e.to_string(), |s: i32| s.to_string()),
//! 	x,
//! );
//! assert_eq!(y, "5".to_string());
//!
//! // By-ref: dispatches to RefBifoldable::ref_bi_fold_map
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
//! 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
//! 	&x,
//! );
//! assert_eq!(y, "5".to_string());
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Bifoldable,
				LiftFn,
				Monoid,
				RefBifoldable,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a bi_fold_map operation to the appropriate type class method.
	///
	/// `(Fn(A) -> M, Fn(B) -> M)` resolves to [`Val`],
	/// `(Fn(&A) -> M, Fn(&B) -> M)` resolves to [`Ref`].
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bifoldable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The monoid type.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure tuple implementing this dispatch.")]
	pub trait BiFoldMapDispatch<
		'a,
		FnBrand,
		Brand: Kind_266801a817966495,
		A: 'a,
		B: 'a,
		M,
		FA,
		Marker,
	> {
		/// Perform the dispatched bi_fold_map operation.
		#[document_signature]
		#[document_parameters("The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e: i32| e.to_string(), |s: i32| s.to_string()),
		/// 	x,
		/// );
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> M;
	}

	/// Routes `(Fn(A) -> M, Fn(B) -> M)` closure tuples to [`Bifoldable::bi_fold_map`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The bifoldable brand.",
		"The first element type.",
		"The second element type.",
		"The monoid type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, FnBrand, Brand, A, B, M, F, G>
		BiFoldMapDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			M,
			Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Val,
		> for (F, G)
	where
		Brand: Bifoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		G: Fn(B) -> M + 'a,
	{
		#[document_signature]
		#[document_parameters("The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e: i32| e.to_string(), |s: i32| s.to_string()),
		/// 	x,
		/// );
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M {
			Brand::bi_fold_map::<FnBrand, A, B, M>(self.0, self.1, fa)
		}
	}

	/// Routes `(Fn(&A) -> M, Fn(&B) -> M)` closure tuples to [`RefBifoldable::ref_bi_fold_map`].
	///
	/// The container must be passed by reference (`&p`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The bifoldable brand.",
		"The first element type.",
		"The second element type.",
		"The monoid type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, 'b, FnBrand, Brand, A, B, M, F, G>
		BiFoldMapDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			M,
			&'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Ref,
		> for (F, G)
	where
		Brand: RefBifoldable,
		FnBrand: LiftFn + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		M: Monoid + 'a,
		F: Fn(&A) -> M + 'a,
		G: Fn(&B) -> M + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let x: Result<i32, i32> = Ok(5);
		/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
		/// 	&x,
		/// );
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M {
			Brand::ref_bi_fold_map::<FnBrand, A, B, M>(self.0, self.1, fa)
		}
	}

	/// Maps elements of both types to a monoid and combines them.
	///
	/// Dispatches to [`Bifoldable::bi_fold_map`] or [`RefBifoldable::ref_bi_fold_map`]
	/// based on whether the closures take owned or reference arguments.
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closures' argument types and the container argument.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bifoldable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The monoid type.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters(
		"A tuple of (first mapping function, second mapping function).",
		"The structure to fold (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|e: i32| e.to_string(), |s: i32| s.to_string()),
	/// 	x,
	/// );
	/// assert_eq!(y, "5".to_string());
	///
	/// // By-ref
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|e: &i32| e.to_string(), |s: &i32| s.to_string()),
	/// 	&x,
	/// );
	/// assert_eq!(y, "5".to_string());
	/// ```
	pub fn bi_fold_map<
		'a,
		FnBrand,
		Brand: Kind_266801a817966495,
		A: 'a,
		B: 'a,
		M: Monoid + 'a,
		FA,
		Marker,
	>(
		fg: impl BiFoldMapDispatch<'a, FnBrand, Brand, A, B, M, FA, Marker>,
		fa: FA,
	) -> M {
		fg.dispatch(fa)
	}
}

pub use inner::*;
