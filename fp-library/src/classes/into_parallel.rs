//! Data structures that can be processed in parallel.
//!
//! This module provides the [`IntoParallel`] trait, which is implemented by collection types
//! that can be converted to and from a flat [`Vec`] for parallel processing. All `par_*`
//! free functions use plain `impl Fn + Send + Sync` closures — no wrapper types required.
//!
//! **Note: The `rayon` feature must be enabled to use actual parallel execution. Without
//! it, all `par_*` functions fall back to equivalent sequential operations.**
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
//! let result: Vec<i32> = par_map::<VecBrand, _, _>(|x: i32| x * 2, v);
//! assert_eq!(result, vec![2, 4, 6, 8, 10]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::Monoid,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for collection types that can be processed in parallel.
	///
	/// Implementors provide two primitive operations:
	/// - [`collect_vec`][IntoParallel::collect_vec]: flattens the collection to a `Vec`
	/// - [`from_vec`][IntoParallel::from_vec]: reconstructs the collection from a `Vec`
	///
	/// All `par_*` free functions are derived from these two primitives. For [`Vec`],
	/// both operations are zero-cost identity conversions.
	///
	/// ### Minimal Implementation
	///
	/// Implement [`IntoParallel::collect_vec`] and [`IntoParallel::from_vec`].
	///
	/// ### Thread Safety
	///
	/// All `par_*` free functions require `A: Send` and closures to be `Send + Sync`.
	/// These bounds apply even when the `rayon` feature is disabled, so that code
	/// compiles identically in both configurations.
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3];
	/// let doubled: Vec<i32> = par_map::<VecBrand, _, _>(|x| x * 2, v);
	/// assert_eq!(doubled, vec![2, 4, 6]);
	/// ```
	pub trait IntoParallel: Kind_cdc7cd43dac7585f {
		/// Flattens the collection into a `Vec`.
		///
		/// For types already backed by a `Vec` (like [`VecBrand`][crate::brands::VecBrand]),
		/// this is a zero-cost identity conversion. For other types, it performs a linear traversal.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The collection to convert.")]
		///
		#[document_returns("A `Vec` containing all elements in order.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::IntoParallel,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result: Vec<i32> = VecBrand::collect_vec(v);
		/// assert_eq!(result, vec![1, 2, 3]);
		/// ```
		fn collect_vec<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> Vec<A>;

		/// Reconstructs the collection from a `Vec`.
		///
		/// For types already backed by a `Vec` (like [`VecBrand`][crate::brands::VecBrand]),
		/// this is a zero-cost identity conversion.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the elements.", "The element type.")]
		///
		#[document_parameters("The `Vec` to convert.")]
		///
		#[document_returns("The reconstructed collection.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::IntoParallel,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result: Vec<i32> = VecBrand::from_vec(v);
		/// assert_eq!(result, vec![1, 2, 3]);
		/// ```
		fn from_vec<'a, A: 'a>(
			v: Vec<A>
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Maps a function over a collection in parallel.
	///
	/// Applies `f` to each element independently. When the `rayon` feature is enabled,
	/// elements are processed across multiple threads. Otherwise falls back to sequential mapping.
	///
	/// Free function version that dispatches to [`IntoParallel::collect_vec`] and [`IntoParallel::from_vec`].
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
		"The function to apply to each element. Must be `Send + Sync`.",
		"The collection to map over."
	)]
	///
	#[document_returns("A new collection containing the mapped elements.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3];
	/// let result: Vec<i32> = par_map::<VecBrand, _, _>(|x| x * 2, v);
	/// assert_eq!(result, vec![2, 4, 6]);
	/// ```
	pub fn par_map<'a, F, A, B>(
		f: impl Fn(A) -> B + Send + Sync,
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: IntoParallel,
		A: 'a + Send,
		B: 'a + Send, {
		let v = F::collect_vec(fa);
		#[cfg(feature = "rayon")]
		let result: Vec<B> = {
			use rayon::prelude::*;
			v.into_par_iter().map(f).collect()
		};
		#[cfg(not(feature = "rayon"))]
		let result: Vec<B> = v.into_iter().map(f).collect();
		F::from_vec(result)
	}

	/// Filters a collection in parallel, retaining elements that satisfy `predicate`.
	///
	/// When the `rayon` feature is enabled, the predicate is evaluated across multiple threads.
	/// Otherwise falls back to sequential filtering.
	///
	/// Free function version that dispatches to [`IntoParallel::collect_vec`] and [`IntoParallel::from_vec`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the collection.",
		"The element type."
	)]
	///
	#[document_parameters("The predicate. Must be `Send + Sync`.", "The collection to filter.")]
	///
	#[document_returns("A new collection containing only the elements satisfying `predicate`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: Vec<i32> = par_filter::<VecBrand, _>(|x| x % 2 == 0, v);
	/// assert_eq!(result, vec![2, 4]);
	/// ```
	pub fn par_filter<'a, F, A>(
		predicate: impl Fn(&A) -> bool + Send + Sync,
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		F: IntoParallel,
		A: 'a + Send, {
		let v = F::collect_vec(fa);
		#[cfg(feature = "rayon")]
		let result: Vec<A> = {
			use rayon::prelude::*;
			v.into_par_iter().filter(|a| predicate(a)).collect()
		};
		#[cfg(not(feature = "rayon"))]
		let result: Vec<A> = v.into_iter().filter(|a| predicate(a)).collect();
		F::from_vec(result)
	}

	/// Filters and maps a collection in parallel.
	///
	/// Applies `f` to each element; elements for which `f` returns `None` are discarded.
	/// When the `rayon` feature is enabled, elements are processed across multiple threads.
	/// Otherwise falls back to sequential filter-mapping.
	///
	/// Free function version that dispatches to [`IntoParallel::collect_vec`] and [`IntoParallel::from_vec`].
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
		"The function to apply. Must be `Send + Sync`.",
		"The collection to filter-map."
	)]
	///
	#[document_returns("A new collection containing the `Some` results of applying `f`.")]
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
	pub fn par_filter_map<'a, F, A, B>(
		f: impl Fn(A) -> Option<B> + Send + Sync,
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: IntoParallel,
		A: 'a + Send,
		B: 'a + Send, {
		let v = F::collect_vec(fa);
		#[cfg(feature = "rayon")]
		let result: Vec<B> = {
			use rayon::prelude::*;
			v.into_par_iter().filter_map(f).collect()
		};
		#[cfg(not(feature = "rayon"))]
		let result: Vec<B> = v.into_iter().filter_map(f).collect();
		F::from_vec(result)
	}

	/// Maps each element to a [`Monoid`] value and combines them in parallel.
	///
	/// Maps each element using `f`, then reduces the results using [`crate::classes::Semigroup::append`].
	/// When the `rayon` feature is enabled, the mapping is done across multiple threads.
	/// Otherwise falls back to sequential fold-map.
	///
	/// Free function version that dispatches to [`IntoParallel::collect_vec`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the elements.",
		"The brand of the collection.",
		"The element type.",
		"The monoid type."
	)]
	///
	#[document_parameters(
		"The function mapping each element to a monoid value. Must be `Send + Sync`.",
		"The collection to fold."
	)]
	///
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let v = vec![1, 2, 3, 4, 5];
	/// let result: String = par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v);
	/// assert_eq!(result, "12345");
	/// ```
	pub fn par_fold_map<'a, F, A, M>(
		f: impl Fn(A) -> M + Send + Sync,
		fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		F: IntoParallel,
		A: 'a + Send,
		M: Monoid + Send + 'a, {
		let v = F::collect_vec(fa);
		#[cfg(feature = "rayon")]
		{
			use rayon::prelude::*;
			v.into_par_iter().map(f).reduce(M::empty, |acc, m| M::append(acc, m))
		}
		#[cfg(not(feature = "rayon"))]
		v.into_iter().map(f).fold(M::empty(), |acc, m| M::append(acc, m))
	}
}

pub use inner::*;
