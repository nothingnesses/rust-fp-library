//! Dispatch for [`Bifoldable::bi_fold_right`](crate::classes::Bifoldable::bi_fold_right) and
//! [`RefBifoldable::ref_bi_fold_right`](crate::classes::RefBifoldable::ref_bi_fold_right).
//!
//! Provides the [`BiFoldRightDispatch`] trait and a unified [`bi_fold_right`] free function
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
//! // Owned: dispatches to Bifoldable::bi_fold_right
//! let x: Result<i32, i32> = Err(3);
//! let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
//! 	(|e, acc| acc - e, |s, acc| acc + s),
//! 	10,
//! 	x,
//! );
//! assert_eq!(y, 7);
//!
//! // By-ref: dispatches to RefBifoldable::ref_bi_fold_right
//! let x: Result<i32, i32> = Err(3);
//! let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
//! 	(|e: &i32, acc| acc - *e, |s: &i32, acc| acc + *s),
//! 	10,
//! 	&x,
//! );
//! assert_eq!(y, 7);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Bifoldable,
				LiftFn,
				RefBifoldable,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// Trait that routes a bi_fold_right operation to the appropriate type class method.
	///
	/// `(Fn(A, C) -> C, Fn(B, C) -> C)` resolves to [`Val`],
	/// `(Fn(&A, C) -> C, Fn(&B, C) -> C)` resolves to [`Ref`].
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the bifoldable structure.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The type of the accumulator.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure tuple implementing this dispatch.")]
	pub trait BiFoldRightDispatch<
		'a,
		FnBrand,
		Brand: Kind_266801a817966495,
		A: 'a,
		B: 'a,
		C: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched bi_fold_right operation.
		#[document_signature]
		#[document_parameters("The initial accumulator value.", "The structure to fold.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let x: Result<i32, i32> = Err(3);
		/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e, acc| acc - e, |s, acc| acc + s),
		/// 	10,
		/// 	x,
		/// );
		/// assert_eq!(y, 7);
		/// ```
		fn dispatch(
			self,
			z: C,
			fa: FA,
		) -> C;
	}

	/// Routes `(Fn(A, C) -> C, Fn(B, C) -> C)` closure tuples to [`Bifoldable::bi_fold_right`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The bifoldable brand.",
		"The first element type.",
		"The second element type.",
		"The accumulator type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, FnBrand, Brand, A, B, C, F, G>
		BiFoldRightDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			C,
			Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Val,
		> for (F, G)
	where
		Brand: Bifoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a,
		F: Fn(A, C) -> C + 'a,
		G: Fn(B, C) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters("The initial accumulator value.", "The structure to fold.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let x: Result<i32, i32> = Err(3);
		/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e, acc| acc - e, |s, acc| acc + s),
		/// 	10,
		/// 	x,
		/// );
		/// assert_eq!(y, 7);
		/// ```
		fn dispatch(
			self,
			z: C,
			fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			Brand::bi_fold_right::<FnBrand, A, B, C>(self.0, self.1, z, fa)
		}
	}

	/// Routes `(Fn(&A, C) -> C, Fn(&B, C) -> C)` closure tuples to [`RefBifoldable::ref_bi_fold_right`].
	///
	/// The container must be passed by reference (`&p`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The bifoldable brand.",
		"The first element type.",
		"The second element type.",
		"The accumulator type.",
		"The first closure type.",
		"The second closure type."
	)]
	#[document_parameters("The closure tuple.")]
	impl<'a, 'b, FnBrand, Brand, A, B, C, F, G>
		BiFoldRightDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			C,
			&'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
			Ref,
		> for (F, G)
	where
		Brand: RefBifoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		C: 'a,
		F: Fn(&A, C) -> C + 'a,
		G: Fn(&B, C) -> C + 'a,
	{
		#[document_signature]
		#[document_parameters(
			"The initial accumulator value.",
			"A reference to the structure to fold."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let x: Result<i32, i32> = Err(3);
		/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
		/// 	(|e: &i32, acc| acc - *e, |s: &i32, acc| acc + *s),
		/// 	10,
		/// 	&x,
		/// );
		/// assert_eq!(y, 7);
		/// ```
		fn dispatch(
			self,
			z: C,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			Brand::ref_bi_fold_right::<FnBrand, A, B, C>(self.0, self.1, z, fa)
		}
	}

	/// Folds a bifoldable structure from the right.
	///
	/// Dispatches to [`Bifoldable::bi_fold_right`] or [`RefBifoldable::ref_bi_fold_right`]
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
		"The type of the accumulator.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters(
		"A tuple of (first step function, second step function).",
		"The initial accumulator value.",
		"The structure to fold (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("The final accumulator value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned
	/// let x: Result<i32, i32> = Err(3);
	/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|e, acc| acc - e, |s, acc| acc + s),
	/// 	10,
	/// 	x,
	/// );
	/// assert_eq!(y, 7);
	///
	/// // By-ref
	/// let x: Result<i32, i32> = Err(3);
	/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	(|e: &i32, acc| acc - *e, |s: &i32, acc| acc + *s),
	/// 	10,
	/// 	&x,
	/// );
	/// assert_eq!(y, 7);
	/// ```
	pub fn bi_fold_right<
		'a,
		FnBrand,
		Brand: Kind_266801a817966495,
		A: 'a,
		B: 'a,
		C: 'a,
		FA,
		Marker,
	>(
		fg: impl BiFoldRightDispatch<'a, FnBrand, Brand, A, B, C, FA, Marker>,
		z: C,
		fa: FA,
	) -> C {
		fg.dispatch(z, fa)
	}
}

pub use inner::*;
