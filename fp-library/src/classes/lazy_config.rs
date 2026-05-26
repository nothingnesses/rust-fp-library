//! Configuration trait for infallible memoization strategy.
//!
//! [`LazyConfig`] bundles the pointer type, lazy cell type, and initializer
//! thunk type needed by [`Lazy`](crate::types::Lazy). The library ships two
//! built-in implementations:
//!
//! - [`RcLazyConfig`](crate::types::RcLazyConfig) for single-threaded use.
//! - [`ArcLazyConfig`](crate::types::ArcLazyConfig) for thread-safe use.
//!
//! Third-party crates can implement this trait to provide custom memoization
//! strategies (for example, `parking_lot`-based locks or async-aware cells).

#[fp_macros::document_module]
mod inner {
	use fp_macros::*;

	/// Configuration for infallible memoization strategy.
	///
	/// This trait bundles together the choices for:
	/// - Pointer type ([`Rc`](std::rc::Rc) vs [`Arc`](std::sync::Arc)).
	/// - Lazy cell type ([`LazyCell`](std::cell::LazyCell) vs [`LazyLock`](std::sync::LazyLock)).
	///
	/// # Note on Standard Library Usage
	///
	/// This design leverages Rust 1.80's `LazyCell` and `LazyLock` types,
	/// which encapsulate the initialization-once logic.
	///
	/// # Extensibility
	///
	/// This trait is open for third-party implementations. You can define a custom
	/// `LazyConfig` to use alternative lazy cell or pointer types (for example,
	/// `parking_lot`-based locks or async-aware cells). Implement the two associated
	/// types ([`Lazy`](LazyConfig::Lazy), [`Thunk`](LazyConfig::Thunk)) and the
	/// two methods ([`lazy_new`](LazyConfig::lazy_new),
	/// [`evaluate`](LazyConfig::evaluate)), then use your config as the
	/// `Config` parameter on [`Lazy`](crate::types::Lazy).
	///
	/// For fallible memoization, use [`TryLazy`](crate::types::TryLazy), which is
	/// a newtype over `Lazy<Result<A, E>>` and requires only `LazyConfig`.
	pub trait LazyConfig: 'static {
		/// The pointer brand used by this configuration.
		///
		/// Links the lazy configuration to the pointer hierarchy, enabling
		/// generic code to obtain the underlying pointer brand from a
		/// `LazyConfig` without hard-coding `RcBrand` or `ArcBrand`.
		type PointerBrand: crate::classes::RefCountedPointer;

		/// The lazy cell type for infallible memoization.
		type Lazy<'a, A: 'a>: Clone;

		/// The type of the initializer thunk.
		type Thunk<'a, A: 'a>: ?Sized;

		/// Creates a new lazy cell from an initializer.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the computation.", "The type of the value.")]
		///
		#[document_parameters("The initializer thunk.")]
		///
		#[document_returns("A new lazy cell.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A>;

		/// Forces evaluation and returns a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The borrow lifetime.",
			"The type of the value."
		)]
		///
		#[document_parameters("The lazy cell to evaluate.")]
		///
		#[document_returns("A reference to the value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazyConfig::lazy_new(Box::new(|| 42));
		/// assert_eq!(*RcLazyConfig::evaluate(&lazy), 42);
		/// ```
		fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A;
	}
}

pub use inner::*;
