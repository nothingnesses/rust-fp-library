//! Data structures that can be mapped over in parallel.
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
//! let result: Vec<i32> = par_map::<VecBrand, _, _>(|x| x * 2, v);
//! assert_eq!(result, vec![2, 4, 6]);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for data structures that can be mapped over in parallel.
	///
	/// `ParFunctor` is the parallel counterpart to [`Functor`](crate::classes::Functor).
	/// Implementors define [`par_map`][ParFunctor::par_map] directly: there is no intermediate
	/// `Vec` conversion imposed by the interface. Types backed by contiguous memory (e.g.,
	/// [`VecBrand`](crate::brands::VecBrand)) can use rayon's parallel iterators directly;
	/// other types may collect to `Vec` as an implementation detail.
	///
	/// ### Laws
	///
	/// `ParFunctor` instances must satisfy the same laws as `Functor`:
	/// * Identity: `par_map(identity, fa) = fa`.
	/// * Composition: `par_map(|a| f(g(a)), fa) = par_map(f, par_map(g, fa))`.
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
	/// ParFunctor laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::VecBrand,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	///
	/// // Identity: par_map(identity, fa) = fa
	/// assert_eq!(par_map::<VecBrand, _, _>(identity, xs.clone()), xs);
	///
	/// // Composition: par_map(|a| f(g(a)), fa) = par_map(f, par_map(g, fa))
	/// let f = |a: i32| a + 1;
	/// let g = |a: i32| a * 2;
	/// assert_eq!(
	/// 	par_map::<VecBrand, _, _>(|a| f(g(a)), xs.clone()),
	/// 	par_map::<VecBrand, _, _>(f, par_map::<VecBrand, _, _>(g, xs)),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait ParFunctor {
		/// Maps a function over a data structure in parallel.
		///
		/// When the `rayon` feature is enabled, elements are processed across multiple threads.
		/// Otherwise falls back to sequential mapping.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The input element type.",
			"The output element type."
		)]
		///
		#[document_parameters(
			"The function to apply to each element. Must be `Send + Sync`.",
			"The data structure to map over."
		)]
		///
		#[document_returns("A new data structure containing the mapped elements.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::VecBrand,
		/// 	classes::par_functor::ParFunctor,
		/// };
		///
		/// let result = VecBrand::par_map(|x: i32| x * 2, vec![1, 2, 3]);
		/// assert_eq!(result, vec![2, 4, 6]);
		/// ```
		fn par_map<'a, A: 'a + Send, B: 'a + Send>(
			f: impl Fn(A) -> B + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Maps a function over a data structure in parallel.
	///
	/// Applies `f` to each element independently. When the `rayon` feature is enabled,
	/// elements are processed across multiple threads. Otherwise falls back to sequential mapping.
	///
	/// Free function version that dispatches to [`ParFunctor::par_map`].
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
	pub fn par_map<'a, Brand: ParFunctor, A: 'a + Send, B: 'a + Send>(
		f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::par_map(f, fa)
	}
}

pub use inner::*;
