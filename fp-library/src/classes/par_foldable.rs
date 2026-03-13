//! Data structures that can be folded in parallel.
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
//! let result: String = par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v);
//! assert_eq!(result, "12345");
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

	/// A type class for data structures that can be folded in parallel.
	///
	/// `ParFoldable` is the parallel counterpart to [`Foldable`](crate::classes::Foldable).
	/// Implementors define [`par_fold_map`][ParFoldable::par_fold_map] directly.
	///
	/// **Note:** There is no `par_fold_right`: the endofunction encoding required for a truly
	/// general right fold has a sequential application step, making it not genuinely parallel.
	/// Use [`par_fold_map`][ParFoldable::par_fold_map] with a commutative [`Monoid`] instead.
	///
	/// ### Laws
	///
	/// `ParFoldable` instances must agree with sequential `fold_map` (up to monoid laws):
	/// * Compatibility: for commutative monoids, `par_fold_map(f, fa) = fold_map(f, fa)`.
	///
	/// ### Thread Safety
	///
	/// All `par_*` functions require `A: Send`, `M: Send`, and closures to be `Send + Sync`.
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
	/// let result: String = par_fold_map::<VecBrand, _, _>(|x: i32| x.to_string(), v);
	/// assert_eq!(result, "12345");
	/// ```
	pub trait ParFoldable: Kind_cdc7cd43dac7585f {
		/// Maps each element to a [`Monoid`] value and combines them in parallel.
		///
		/// Maps each element using `f`, then reduces the results using
		/// [`Semigroup::append`](crate::classes::Semigroup::append). When the `rayon` feature is
		/// enabled, the mapping and reduction are done across multiple threads. Otherwise falls
		/// back to a sequential fold.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
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
		/// 	brands::VecBrand,
		/// 	classes::par_foldable::ParFoldable,
		/// };
		///
		/// let result: String = VecBrand::par_fold_map(|x: i32| x.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		fn par_fold_map<'a, A: 'a + Send, M: Monoid + Send + 'a>(
			f: impl Fn(A) -> M + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M;
	}

	/// Maps each element to a [`Monoid`] value and combines them in parallel.
	///
	/// Maps each element using `f`, then reduces the results using
	/// [`Semigroup::append`](crate::classes::Semigroup::append). When the `rayon` feature is
	/// enabled, the mapping and reduction are done across multiple threads. Otherwise falls back
	/// to a sequential fold.
	///
	/// Free function version that dispatches to [`ParFoldable::par_fold_map`].
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
	pub fn par_fold_map<'a, Brand: ParFoldable, A: 'a + Send, M: Monoid + Send + 'a>(
		f: impl Fn(A) -> M + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M {
		Brand::par_fold_map(f, fa)
	}
}

pub use inner::*;
