//! Data structures that can be traversed, accumulating results in an applicative context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! let x = Some(5);
//! let y = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(|a| Some(a * 2), x);
//! assert_eq!(y, Some(Some(10)));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			functions::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for traversable functors.
	///
	/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
	///
	/// ### Laws
	///
	/// `Traversable` instances must satisfy:
	/// * Traverse/sequence consistency: `traverse(f, xs) = sequence(map(f, xs))`.
	/// * Sequence/traverse consistency: `sequence(xs) = traverse(identity, xs)`.
	#[document_examples]
	///
	/// Traversable laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::{
	/// 		explicit::{
	/// 			map,
	/// 			traverse,
	/// 		},
	/// 		*,
	/// 	},
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let f = |a: i32| if a > 0 { Some(a * 2) } else { None };
	///
	/// // Traverse/sequence consistency:
	/// // traverse(f, xs) = sequence(map(f, xs))
	/// assert_eq!(
	/// 	traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(f, xs.clone()),
	/// 	sequence::<VecBrand, _, OptionBrand>(map::<VecBrand, _, _, _>(f, xs.clone())),
	/// );
	///
	/// // Sequence/traverse consistency:
	/// // sequence(xs) = traverse(identity, xs)
	/// let ys: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
	/// assert_eq!(
	/// 	sequence::<VecBrand, _, OptionBrand>(ys.clone()),
	/// 	traverse::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(identity, ys),
	/// );
	/// ```
	pub trait Traversable: Functor + Foldable {
		/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
		///
		/// The default implementation is defined in terms of [`sequence`] and [`map`](crate::functions::map).
		///
		/// **Note**: This default implementation may be less efficient than a direct implementation because it performs two passes:
		/// first mapping the function to create an intermediate structure of computations, and then sequencing that structure.
		/// A direct implementation can often perform the traversal in a single pass without allocating an intermediate container.
		/// Types should provide their own implementation if possible.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The traversable structure."
		)]
		///
		#[document_returns("The traversable structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(|a| Some(a * 2), x);
		/// assert_eq!(y, Some(Some(10)));
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			Self::sequence::<B, F>(Self::map::<
				A,
				Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
			>(func, ta))
		}

		/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
		///
		/// The default implementation is defined in terms of [`traverse`] and [`identity`].
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The traversable structure containing values in an applicative context."
		)]
		///
		#[document_returns("The traversable structure wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(Some(5));
		/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
		/// assert_eq!(y, Some(Some(5)));
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			Self::traverse::<Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>), A, F>(
				identity, ta,
			)
		}
	}

	/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Traversable::traverse`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the traversable structure.",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a value in an applicative context.",
		"The traversable structure."
	)]
	///
	#[document_returns("The traversable structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = traverse::<RcFnBrand, OptionBrand, _, _, OptionBrand, _, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn traverse<'a, Brand: Traversable, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
		func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
		Brand::traverse::<A, B, F>(func, ta)
	}

	/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
	///
	/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the traversable structure.",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]
	///
	#[document_parameters("The traversable structure containing values in an applicative context.")]
	///
	#[document_returns("The traversable structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(Some(5));
	/// let y = sequence::<OptionBrand, _, OptionBrand>(x);
	/// assert_eq!(y, Some(Some(5)));
	/// ```
	pub fn sequence<'a, Brand: Traversable, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
		Brand::sequence::<A, F>(ta)
	}
}

pub use inner::*;
