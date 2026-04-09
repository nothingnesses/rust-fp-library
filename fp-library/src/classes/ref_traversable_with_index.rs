//! By-reference indexed traversal of structures.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let v = vec![10, 20, 30];
//! let result: Option<Vec<String>> = ref_traverse_with_index::<VecBrand, _, _, OptionBrand>(
//! 	|i, x: &i32| Some(format!("{}:{}", i, x)),
//! 	v,
//! );
//! assert_eq!(result, Some(vec!["0:10".to_string(), "1:20".to_string(), "2:30".to_string()]));
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

	/// By-reference indexed traversal of structures.
	///
	/// Similar to [`TraversableWithIndex`], but the closure receives `&A`
	/// instead of `A`. Combines indexed access with by-reference traversal.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefTraversableWithIndex: RefTraversable + RefFunctorWithIndex + WithIndex {
		/// Maps each element by reference with its index to a computation,
		/// evaluates them, and combines the results.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the input elements.",
			"The type of the output elements.",
			"The applicative functor brand."
		)]
		///
		#[document_parameters(
			"The function to apply to each (index, element reference) pair.",
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
		/// let v = vec![10, 20, 30];
		/// let result: Option<Vec<String>> = ref_traverse_with_index::<VecBrand, _, _, OptionBrand>(
		/// 	|i, x: &i32| Some(format!("{}:{}", i, x)),
		/// 	v,
		/// );
		/// assert_eq!(result, Some(vec!["0:10".to_string(), "1:20".to_string(), "2:30".to_string()]));
		/// ```
		fn ref_traverse_with_index<'a, A: 'a + Clone, B: 'a + Clone, M: Applicative>(
			f: impl Fn(Self::Index, &A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
			+ 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone;
	}

	/// Maps each element by reference with its index to a computation,
	/// evaluates them, and combines the results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefTraversableWithIndex::ref_traverse_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the structure.",
		"The type of the input elements.",
		"The type of the output elements.",
		"The applicative functor brand."
	)]
	///
	#[document_parameters(
		"The function to apply to each (index, element reference) pair.",
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
	/// let v = vec![10, 20, 30];
	/// let result: Option<Vec<String>> = ref_traverse_with_index::<VecBrand, _, _, OptionBrand>(
	/// 	|i, x: &i32| Some(format!("{}:{}", i, x)),
	/// 	v,
	/// );
	/// assert_eq!(result, Some(vec!["0:10".to_string(), "1:20".to_string(), "2:30".to_string()]));
	/// ```
	pub fn ref_traverse_with_index<
		'a,
		Brand: RefTraversableWithIndex,
		A: 'a + Clone,
		B: 'a + Clone,
		M: Applicative,
	>(
		f: impl Fn(Brand::Index, &A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		ta: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::ref_traverse_with_index::<A, B, M>(f, ta)
	}
}

pub use inner::*;
