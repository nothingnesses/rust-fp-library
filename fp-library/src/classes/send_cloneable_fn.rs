//! Thread-safe cloneable wrappers over closures that carry `Send + Sync` bounds.
//!
//! ### Examples
//!
//! ```
//! use {
//! 	fp_library::{
//! 		brands::*,
//! 		functions::*,
//! 	},
//! 	std::thread,
//! };
//!
//! let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
//!
//! // Can be sent to another thread
//! let handle = thread::spawn(move || {
//! 	assert_eq!(f(5), 10);
//! });
//! handle.join().unwrap();
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
		std::ops::Deref,
	};

	/// Abstraction for thread-safe cloneable wrappers over closures.
	///
	/// This trait extends [`CloneableFn`] to enforce `Send + Sync` bounds on the
	/// wrapped closure and the wrapper itself. This is implemented by types like
	/// [`ArcFnBrand`][crate::brands::ArcFnBrand] but not [`RcFnBrand`][crate::brands::RcFnBrand].
	///
	/// The lifetime `'a` ensures the function doesn't outlive referenced data,
	/// while generic types `A` and `B` represent the input and output types, respectively.
	///
	/// By explicitly requiring that both type parameters outlive the application lifetime `'a`,
	/// we provide the compiler with the necessary guarantees to handle trait objects
	/// (like `dyn Fn`) commonly used in thread-safe function wrappers. This resolves potential
	/// E0310 errors where the compiler cannot otherwise prove that captured variables in
	/// closures satisfy the required lifetime bounds.
	pub trait SendCloneableFn: CloneableFn {
		/// The type of the thread-safe cloneable function wrapper.
		///
		/// This associated type represents the concrete type of the wrapper (e.g., `Arc<dyn Fn(A) -> B + Send + Sync>`)
		/// that implements `Clone`, `Send`, `Sync` and dereferences to the underlying closure.
		type SendOf<'a, A: 'a, B: 'a>: Clone
			+ Send
			+ Sync
			+ Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>;

		/// Creates a new thread-safe cloneable function wrapper.
		///
		/// This method wraps a closure into a thread-safe cloneable function wrapper.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters(
			"The closure to wrap. Must be `Send + Sync`.",
			"The input value to the function."
		)]
		#[document_returns("The wrapped thread-safe cloneable function.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 	},
		/// 	std::thread,
		/// };
		///
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
		///
		/// // Can be sent to another thread
		/// let handle = thread::spawn(move || {
		/// 	assert_eq!(f(5), 10);
		/// });
		/// handle.join().unwrap();
		/// ```
		fn send_cloneable_fn_new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B + Send + Sync
		) -> <Self as SendCloneableFn>::SendOf<'a, A, B>;
	}

	/// Creates a new thread-safe cloneable function wrapper.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendCloneableFn::send_cloneable_fn_new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the thread-safe cloneable function wrapper.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	#[document_parameters(
		"The closure to wrap. Must be `Send + Sync`.",
		"The input value to the function."
	)]
	#[document_returns("The wrapped thread-safe cloneable function.")]
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::*,
	/// 		functions::*,
	/// 	},
	/// 	std::thread,
	/// };
	///
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
	///
	/// // Can be sent to another thread
	/// let handle = thread::spawn(move || {
	/// 	assert_eq!(f(5), 10);
	/// });
	/// handle.join().unwrap();
	/// ```
	pub fn new<'a, Brand, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> <Brand as SendCloneableFn>::SendOf<'a, A, B>
	where
		Brand: SendCloneableFn, {
		<Brand as SendCloneableFn>::send_cloneable_fn_new(f)
	}
}

pub use inner::*;
