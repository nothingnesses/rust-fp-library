//! A `ParFilterable` with an additional index.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A `ParFilterable` with an additional index.
	///
	/// `ParFilterableWithIndex` is the parallel counterpart to
	/// [`FilterableWithIndex`](crate::classes::FilterableWithIndex). Implementors define
	/// [`par_filter_map_with_index`][ParFilterableWithIndex::par_filter_map_with_index] directly;
	/// [`par_filter_with_index`][ParFilterableWithIndex::par_filter_with_index] is derived from it
	/// by default.
	///
	/// ### Laws
	///
	/// `ParFilterableWithIndex` instances must satisfy the same laws as `FilterableWithIndex`:
	/// * Compatibility with `ParFilterable`:
	///   `par_filter_map(f, fa) = par_filter_map_with_index(|_, a| f(a), fa)`.
	///
	/// ### Thread Safety
	///
	/// The index type must satisfy `Self::Index: Send + Sync + Copy` when calling
	/// methods on this trait. All closures must be `Send + Sync`. These bounds apply even
	/// when the `rayon` feature is disabled, so that code compiles identically in both
	/// configurations.
	///
	/// **Note: The `rayon` feature must be enabled to use actual parallel execution. Without
	/// it, all `par_*` functions fall back to equivalent sequential operations.**
	#[document_examples]
	///
	/// ParFilterableWithIndex laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3, 4, 5];
	///
	/// // Compatibility with ParFilterable:
	/// // par_filter_map(f, fa) = par_filter_map_with_index(|_, a| f(a), fa)
	/// let f = |a: i32| if a > 3 { Some(a * 10) } else { None };
	/// assert_eq!(
	/// 	par_filter_map::<VecBrand, _, _>(f, xs.clone()),
	/// 	VecBrand::par_filter_map_with_index(|_, a| f(a), xs),
	/// );
	/// ```
	pub trait ParFilterableWithIndex: ParFilterable + FilterableWithIndex {
		/// Maps and filters a data structure in parallel with the index, discarding elements
		/// for which `f` returns [`None`].
		///
		/// Override this method with a single-pass implementation for better performance.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and its index, returning an [`Option`]. Must be `Send + Sync`.",
			"The data structure to filter and map."
		)]
		///
		#[document_returns(
			"A new data structure containing only the values where `f` returned [`Some`]."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
		/// };
		///
		/// let result = VecBrand::par_filter_map_with_index(
		/// 	|i, x: i32| if i < 3 { Some(x * 10) } else { None },
		/// 	vec![1, 2, 3, 4, 5],
		/// );
		/// assert_eq!(result, vec![10, 20, 30]);
		/// ```
		fn par_filter_map_with_index<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(Self::Index, A) -> Option<B> + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Self::Index: Send + Sync + Copy + 'a;

		/// Filters a data structure in parallel with the index, retaining only elements
		/// satisfying `f`.
		///
		/// The default implementation derives from
		/// [`par_filter_map_with_index`][Self::par_filter_map_with_index].
		/// No [`Clone`] bound is required: ownership of each element is passed to the closure,
		/// which either returns `Some(a)` (retain) or `None` (discard).
		///
		/// Override this method with a single-pass implementation for better performance.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters(
			"The predicate receiving the index and a reference to the element. Must be `Send + Sync`.",
			"The data structure to filter."
		)]
		///
		#[document_returns("A new data structure containing only the elements that satisfy `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable_with_index::ParFilterableWithIndex,
		/// };
		///
		/// let result =
		/// 	VecBrand::par_filter_with_index(|i, x: &i32| i < 3 && x % 2 != 0, vec![1, 2, 3, 4, 5]);
		/// assert_eq!(result, vec![1, 3]);
		/// ```
		fn par_filter_with_index<'a, A: 'a + Send>(
			f: impl Fn(Self::Index, &A) -> bool + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		where
			Self::Index: Send + Sync + Copy + 'a, {
			Self::par_filter_map_with_index(move |i, a| if f(i, &a) { Some(a) } else { None }, fa)
		}
	}

	/// Maps and filters a data structure in parallel with the index, discarding elements
	/// for which `f` returns [`None`].
	///
	/// Free function version that dispatches to
	/// [`ParFilterableWithIndex::par_filter_map_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the collection.",
		"The input element type.",
		"The output element type."
	)]
	///
	#[document_parameters(
		"The function to apply to each element and its index, returning an [`Option`]. Must be `Send + Sync`.",
		"The data structure to filter and map."
	)]
	///
	#[document_returns("A new collection containing only the values where `f` returned [`Some`].")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3, 4, 5];
	/// let result: Vec<i32> = par_filter_map_with_index::<VecBrand, _, _>(
	/// 	|i, x: i32| if i < 3 { Some(x * 10) } else { None },
	/// 	xs,
	/// );
	/// assert_eq!(result, vec![10, 20, 30]);
	/// ```
	pub fn par_filter_map_with_index<'a, Brand, A: 'a + Send, B: 'a + Send>(
		f: impl Fn(Brand::Index, A) -> Option<B> + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: ParFilterableWithIndex,
		Brand::Index: Send + Sync + Copy + 'a, {
		Brand::par_filter_map_with_index(f, fa)
	}

	/// Filters a data structure in parallel with the index, retaining only elements
	/// satisfying `f`.
	///
	/// Free function version that dispatches to
	/// [`ParFilterableWithIndex::par_filter_with_index`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the collection.",
		"The element type."
	)]
	///
	#[document_parameters(
		"The predicate receiving the index and a reference to the element. Must be `Send + Sync`.",
		"The data structure to filter."
	)]
	///
	#[document_returns("A new collection containing only the elements satisfying `f`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3, 4, 5];
	/// let result: Vec<i32> =
	/// 	par_filter_with_index::<VecBrand, _>(|i, x: &i32| i < 3 && x % 2 != 0, xs);
	/// assert_eq!(result, vec![1, 3]);
	/// ```
	pub fn par_filter_with_index<'a, Brand, A: 'a + Send>(
		f: impl Fn(Brand::Index, &A) -> bool + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: ParFilterableWithIndex,
		Brand::Index: Send + Sync + Copy + 'a, {
		Brand::par_filter_with_index(f, fa)
	}
}

pub use inner::*;
