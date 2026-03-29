//! Configuration trait for fallible memoization strategy.
//!
//! [`TryLazyConfig`] extends [`LazyConfig`](crate::classes::LazyConfig) with
//! associated types and methods for memoizing computations that may fail. The
//! library ships two built-in implementations:
//!
//! - [`RcLazyConfig`](crate::types::RcLazyConfig) for single-threaded use.
//! - [`ArcLazyConfig`](crate::types::ArcLazyConfig) for thread-safe use.
//!
//! Third-party crates can implement only [`LazyConfig`](crate::classes::LazyConfig)
//! when fallible memoization is not needed, or both traits when it is.

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::LazyConfig,
		fp_macros::*,
	};

	/// Configuration for fallible memoization strategy.
	///
	/// This trait separates the fallible (error-producing) memoization types
	/// from the infallible ones in [`LazyConfig`]. Third-party implementations
	/// can choose to implement only `LazyConfig` when fallible memoization is
	/// not needed, or both traits when it is.
	///
	/// # Extensibility
	///
	/// Implement the two associated types
	/// ([`TryLazy`](TryLazyConfig::TryLazy), [`TryThunk`](TryLazyConfig::TryThunk))
	/// and the two methods
	/// ([`try_lazy_new`](TryLazyConfig::try_lazy_new),
	/// [`try_evaluate`](TryLazyConfig::try_evaluate)), then use your config as
	/// the `Config` parameter on [`TryLazy`](crate::types::TryLazy).
	pub trait TryLazyConfig: LazyConfig {
		/// The lazy cell type for fallible memoization.
		type TryLazy<'a, A: 'a, E: 'a>: Clone;

		/// The type of the fallible initializer thunk.
		type TryThunk<'a, A: 'a, E: 'a>: ?Sized;

		/// Creates a new fallible lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new fallible lazy cell.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
		/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
		/// ```
		fn try_lazy_new<'a, A: 'a, E: 'a>(
			f: Box<Self::TryThunk<'a, A, E>>
		) -> Self::TryLazy<'a, A, E>;

		/// Forces evaluation and returns a reference to the result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value.",
			"The type of the error."
		)]
		///
		#[document_parameters("The fallible lazy cell to evaluate.")]
		///
		#[document_returns("A result containing a reference to the value or error.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazyConfig::try_lazy_new(Box::new(|| Ok::<i32, ()>(42)));
		/// assert_eq!(RcLazyConfig::try_evaluate(&lazy), Ok(&42));
		/// ```
		fn try_evaluate<'a, 'b, A: 'a, E: 'a>(
			lazy: &'b Self::TryLazy<'a, A, E>
		) -> Result<&'b A, &'b E>;
	}
}

pub use inner::*;
