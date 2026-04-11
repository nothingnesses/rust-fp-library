//! By-reference withering (effectful filtering) of structures.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![1, 2, 3, 4, 5];
//! let result: Option<Vec<i32>> = wither_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
//! 	|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
//! 	&v,
//! );
//! assert_eq!(result, Some(vec![4, 5]));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// By-reference withering (effectful filtering) of structures.
	///
	/// Similar to [`Witherable`], but closures receive `&A` instead of `A`.
	/// Combines by-reference traversal with filtering in an applicative context.
	///
	/// Default implementations derive:
	/// * `ref_wilt` from `ref_traverse` + `separate`.
	/// * `ref_wither` from `ref_traverse` + `compact`.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefWitherable: RefFilterable + RefTraversable {
		/// Partitions by reference in an applicative context.
		///
		/// Maps each element by reference to a computation producing `Result<O, E>`,
		/// traverses the structure, then separates the results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The applicative context.",
			"The type of the input elements.",
			"The error type.",
			"The success type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The structure to partition."
		)]
		///
		#[document_returns("The partitioned structure in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result: Option<(Vec<i32>, Vec<i32>)> =
		/// 	wilt_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
		/// 		|x: &i32| Some(if *x > 3 { Ok(*x) } else { Err(*x) }),
		/// 		&v,
		/// 	);
		/// assert_eq!(result, Some((vec![1, 2, 3], vec![4, 5])));
		/// ```
		fn ref_wilt<'a, FnBrand, M: Applicative, A: 'a + Clone, E: 'a + Clone, O: 'a + Clone>(
			func: impl Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
			+ 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			(
				Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
				Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
			),
		>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone, {
			M::map(
				|res| Self::separate::<E, O>(res),
				Self::ref_traverse::<FnBrand, A, Result<O, E>, M>(func, ta),
			)
		}

		/// Filters by reference in an applicative context.
		///
		/// Maps each element by reference to a computation producing `Option<B>`,
		/// traverses the structure, then compacts the results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The applicative context.",
			"The type of the input elements.",
			"The type of the output elements."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The structure to filter."
		)]
		///
		#[document_returns("The filtered structure in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![1, 2, 3, 4, 5];
		/// let result: Option<Vec<i32>> = wither_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
		/// 	|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
		/// 	&v,
		/// );
		/// assert_eq!(result, Some(vec![4, 5]));
		/// ```
		fn ref_wither<'a, FnBrand, M: Applicative, A: 'a + Clone, B: 'a + Clone>(
			func: impl Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone, {
			M::map(
				|opt| Self::compact(opt),
				Self::ref_traverse::<FnBrand, A, Option<B>, M>(func, ta),
			)
		}
	}

	/// Partitions by reference in an applicative context.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefWitherable::ref_wilt`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The brand of the cloneable function wrapper.",
		"The applicative context.",
		"The type of the input elements.",
		"The error type.",
		"The success type."
	)]
	///
	#[document_parameters(
		"The function to apply to each element reference.",
		"The structure to partition."
	)]
	///
	#[document_returns("The partitioned structure in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Option<(Vec<i32>, Vec<i32>)> =
	/// 	wilt_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _, _>(
	/// 		|x: &i32| Some(if *x > 3 { Ok(*x) } else { Err(*x) }),
	/// 		&v,
	/// 	);
	/// assert_eq!(result, Some((vec![1, 2, 3], vec![4, 5])));
	/// ```
	pub fn ref_wilt<
		'a,
		Brand: RefWitherable,
		FnBrand,
		M: Applicative,
		A: 'a + Clone,
		E: 'a + Clone,
		O: 'a + Clone,
	>(
		func: impl Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>) + 'a,
		ta: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>): Clone, {
		Brand::ref_wilt::<FnBrand, M, A, E, O>(func, ta)
	}

	/// Filters by reference in an applicative context.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefWitherable::ref_wither`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The brand of the cloneable function wrapper.",
		"The applicative context.",
		"The type of the input elements.",
		"The type of the output elements."
	)]
	///
	#[document_parameters(
		"The function to apply to each element reference.",
		"The structure to filter."
	)]
	///
	#[document_returns("The filtered structure in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Option<Vec<i32>> = wither_explicit::<RcFnBrand, VecBrand, OptionBrand, _, _, _, _>(
	/// 	|x: &i32| if *x > 3 { Some(Some(*x)) } else { Some(None) },
	/// 	&v,
	/// );
	/// assert_eq!(result, Some(vec![4, 5]));
	/// ```
	pub fn ref_wither<
		'a,
		Brand: RefWitherable,
		FnBrand,
		M: Applicative,
		A: 'a + Clone,
		B: 'a + Clone,
	>(
		func: impl Fn(&A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>) + 'a,
		ta: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>): Clone, {
		Brand::ref_wither::<FnBrand, M, A, B>(func, ta)
	}
}

pub use inner::*;
