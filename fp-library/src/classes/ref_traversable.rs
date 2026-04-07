//! By-reference traversal of structures.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![1, 2, 3];
//! let result: Option<Vec<String>> =
//! 	ref_traverse::<VecBrand, RcFnBrand, _, _, OptionBrand>(|x: &i32| Some(x.to_string()), v);
//! assert_eq!(result, Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]));
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

	/// By-reference traversal of structures.
	///
	/// Similar to [`Traversable`], but the closure receives `&A` instead of `A`.
	/// This enables traversing collections by reference without consuming elements,
	/// or traversing memoized types that only provide `&A` access.
	///
	/// `ref_traverse` is the required method.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefTraversable: RefFunctor + RefFoldable {
		/// Maps each element by reference to a computation, evaluates them,
		/// and combines the results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function wrapper.",
			"The type of the elements in the input structure.",
			"The type of the elements in the output structure.",
			"The applicative functor brand for the computation."
		)]
		///
		#[document_parameters(
			"The function to apply to each element reference.",
			"The structure to traverse."
		)]
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
		/// let v = vec![1, 2, 3];
		/// let result: Option<Vec<String>> =
		/// 	ref_traverse::<VecBrand, RcFnBrand, _, _, OptionBrand>(|x: &i32| Some(x.to_string()), v);
		/// assert_eq!(result, Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]));
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone;
	}

	/// Maps each element by reference to a computation, evaluates them,
	/// and combines the results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefTraversable::ref_traverse`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The brand of the cloneable function wrapper.",
		"The type of the input elements.",
		"The type of the output elements.",
		"The applicative functor brand."
	)]
	///
	#[document_parameters(
		"The function to apply to each element reference.",
		"The structure to traverse."
	)]
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
	/// let v = vec![1, 2, 3];
	/// let result: Option<Vec<String>> =
	/// 	ref_traverse::<VecBrand, RcFnBrand, _, _, OptionBrand>(|x: &i32| Some(x.to_string()), v);
	/// assert_eq!(result, Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]));
	/// ```
	pub fn ref_traverse<
		'a,
		Brand: RefTraversable,
		FnBrand,
		A: 'a + Clone,
		B: 'a + Clone,
		F: Applicative,
	>(
		func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		FnBrand: LiftFn + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::ref_traverse::<FnBrand, A, B, F>(func, ta)
	}
}

pub use inner::*;
