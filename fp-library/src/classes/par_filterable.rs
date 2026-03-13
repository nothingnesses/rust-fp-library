//! Data structures that can be filtered and filter-mapped in parallel.
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
//! let result: Vec<i32> =
//! 	par_filter_map::<VecBrand, _, _>(|x: i32| if x % 2 == 0 { Some(x * 10) } else { None }, v);
//! assert_eq!(result, vec![20, 40]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				ParCompactable,
				ParFunctor,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for data structures that can be filtered and filter-mapped in parallel.
	///
	/// `ParFilterable` is the parallel counterpart to [`Filterable`](crate::classes::Filterable),
	/// extending [`ParFunctor`] and [`ParCompactable`].
	///
	/// All methods have default implementations based on [`par_map`][ParFunctor::par_map] and
	/// [`par_compact`][ParCompactable::par_compact]. However, it is recommended to override
	/// [`ParFilterable::par_filter_map`] and [`ParFilterable::par_filter`] with single-pass
	/// implementations to avoid the intermediate allocation created by the default.
	///
	/// *   If [`ParFilterable::par_filter_map`] is overridden, [`ParFilterable::par_filter`] is
	///     automatically derived from it (no [`Clone`] bound required).
	///
	/// ### Laws
	///
	/// `ParFilterable` instances must satisfy the same laws as `Filterable`:
	/// * Identity: `par_filter_map(Some, fa) = fa`.
	/// * Composition: `par_filter_map(|a| r(a).and_then(l), fa) = par_filter_map(l, par_filter_map(r, fa))`.
	/// * Consistency: `par_filter(p, fa) = par_filter_map(|a| if p(&a) { Some(a) } else { None }, fa)`.
	///
	/// ### Thread Safety
	///
	/// All `par_*` functions require `A: Send`, `B: Send`, and closures to be `Send + Sync`.
	/// These bounds apply even when the `rayon` feature is disabled, so that code compiles
	/// identically in both configurations.
	///
	/// **Note: The `rayon` feature must be enabled to use actual parallel execution. Without
	/// it, all `par_*` functions fall back to equivalent sequential operations.**
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let mapped: Vec<i32> = par_filter_map::<VecBrand, _, _>(
	/// 	|x: i32| if x % 2 == 0 { Some(x * 10) } else { None },
	/// 	v.clone(),
	/// );
	/// assert_eq!(mapped, vec![20, 40]);
	///
	/// let filtered: Vec<i32> = par_filter::<VecBrand, _>(|x: &i32| x % 2 == 0, v);
	/// assert_eq!(filtered, vec![2, 4]);
	/// ```
	pub trait ParFilterable: ParFunctor + ParCompactable {
		/// Maps and filters a data structure in parallel, discarding elements for which `f` returns
		/// [`None`].
		///
		/// The default implementation uses [`par_map`][ParFunctor::par_map] followed by
		/// [`par_compact`][ParCompactable::par_compact]. Override this method with a single-pass
		/// implementation for better performance.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning an [`Option`]. Must be `Send + Sync`.",
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
		/// 	classes::par_filterable::ParFilterable,
		/// };
		///
		/// let result = VecBrand::par_filter_map(
		/// 	|x: i32| if x % 2 == 0 { Some(x * 10) } else { None },
		/// 	vec![1, 2, 3, 4, 5],
		/// );
		/// assert_eq!(result, vec![20, 40]);
		/// ```
		fn par_filter_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> Option<B> + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::par_compact::<B>(Self::par_map::<A, Option<B>>(f, fa))
		}

		/// Filters a data structure in parallel, retaining only elements satisfying `f`.
		///
		/// The default implementation derives from [`par_filter_map`][Self::par_filter_map].
		/// No [`Clone`] bound is required: ownership of each element is passed to the closure,
		/// which either returns `Some(a)` (retain) or `None` (discard).
		///
		/// Override this method with a single-pass implementation for better performance.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters(
			"The predicate. Must be `Send + Sync`.",
			"The data structure to filter."
		)]
		///
		#[document_returns("A new data structure containing only the elements that satisfy `f`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_filterable::ParFilterable,
		/// };
		///
		/// let result = VecBrand::par_filter(|x: &i32| x % 2 == 0, vec![1, 2, 3, 4, 5]);
		/// assert_eq!(result, vec![2, 4]);
		/// ```
		fn par_filter<'a, A: 'a + Send>(
			f: impl Fn(&A) -> bool + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::par_filter_map::<A, A>(move |a| if f(&a) { Some(a) } else { None }, fa)
		}
	}

	/// Maps and filters a data structure in parallel, discarding elements for which `f` returns
	/// [`None`].
	///
	/// Free function version that dispatches to [`ParFilterable::par_filter_map`].
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
		"The function to apply to each element, returning an [`Option`]. Must be `Send + Sync`.",
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
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Vec<i32> =
	/// 	par_filter_map::<VecBrand, _, _>(|x: i32| if x % 2 == 0 { Some(x * 10) } else { None }, v);
	/// assert_eq!(result, vec![20, 40]);
	/// ```
	pub fn par_filter_map<'a, Brand: ParFilterable, A: 'a + Send, B: 'a + Send>(
		f: impl Fn(A) -> Option<B> + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::par_filter_map(f, fa)
	}

	/// Filters a data structure in parallel, retaining only elements satisfying `f`.
	///
	/// Free function version that dispatches to [`ParFilterable::par_filter`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the collection.",
		"The element type."
	)]
	///
	#[document_parameters("The predicate. Must be `Send + Sync`.", "The data structure to filter.")]
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
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Vec<i32> = par_filter::<VecBrand, _>(|x: &i32| x % 2 == 0, v);
	/// assert_eq!(result, vec![2, 4]);
	/// ```
	pub fn par_filter<'a, Brand: ParFilterable, A: 'a + Send>(
		f: impl Fn(&A) -> bool + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::par_filter(f, fa)
	}
}

pub use inner::*;
