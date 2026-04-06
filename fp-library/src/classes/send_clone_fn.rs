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
//! let f = send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
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
		crate::classes::dispatch::{
			ClosureMode,
			Ref,
			Val,
		},
		fp_macros::*,
		std::ops::Deref,
	};

	/// Abstraction for thread-safe cloneable wrappers over closures.
	///
	/// This trait extends [`CloneFn`] to enforce `Send + Sync` bounds on the
	/// wrapped closure and the wrapper itself. This is implemented by types like
	/// [`ArcFnBrand`][crate::brands::ArcFnBrand] but not [`RcFnBrand`][crate::brands::RcFnBrand].
	///
	/// The `Mode` parameter selects whether the wrapped closure takes its input
	/// by value (`Val`, the default) or by reference (`Ref`).
	///
	/// The lifetime `'a` ensures the function doesn't outlive referenced data,
	/// while generic types `A` and `B` represent the input and output types, respectively.
	///
	/// By explicitly requiring that both type parameters outlive the application lifetime `'a`,
	/// we provide the compiler with the necessary guarantees to handle trait objects
	/// (like `dyn Fn`) commonly used in thread-safe function wrappers. This resolves potential
	/// E0310 errors where the compiler cannot otherwise prove that captured variables in
	/// closures satisfy the required lifetime bounds.
	#[document_type_parameters(
		"Selects whether the wrapped closure takes its input by value (`Val`) or by reference (`Ref`). Defaults to `Val`."
	)]
	pub trait SendCloneFn<Mode: ClosureMode = Val> {
		/// The type of the thread-safe cloneable function wrapper.
		///
		/// This associated type represents the concrete type of the wrapper (e.g., `Arc<dyn Fn(A) -> B + Send + Sync>`)
		/// that implements `Clone`, `Send`, `Sync` and dereferences to the underlying closure.
		type Of<'a, A: 'a, B: 'a>: Clone + Send + Sync + Deref<Target = Mode::SendTarget<'a, A, B>>;
	}

	/// A trait for constructing thread-safe cloneable function wrappers from closures.
	///
	/// Separated from [`SendCloneFn`] because the `new` method's parameter type
	/// depends on the closure mode (`Fn(A) -> B + Send + Sync` for `Val`), and a single
	/// trait method cannot have a mode-dependent signature.
	pub trait SendLiftFn: SendCloneFn<Val> {
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
		#[document_parameters("The closure to wrap. Must be `Send + Sync`.")]
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
		/// let f = send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
		///
		/// // Can be sent to another thread
		/// let handle = thread::spawn(move || {
		/// 	assert_eq!(f(5), 10);
		/// });
		/// handle.join().unwrap();
		/// ```
		fn new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(A) -> B + Send + Sync
		) -> <Self as SendCloneFn>::Of<'a, A, B>;
	}

	/// Creates a new thread-safe cloneable function wrapper.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendLiftFn::new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the thread-safe cloneable function wrapper.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to wrap. Must be `Send + Sync`.")]
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
	/// let f = send_lift_fn_new::<ArcFnBrand, _, _>(|x: i32| x * 2);
	///
	/// // Can be sent to another thread
	/// let handle = thread::spawn(move || {
	/// 	assert_eq!(f(5), 10);
	/// });
	/// handle.join().unwrap();
	/// ```
	pub fn new<'a, Brand, A, B>(
		f: impl 'a + Fn(A) -> B + Send + Sync
	) -> <Brand as SendCloneFn>::Of<'a, A, B>
	where
		Brand: SendLiftFn, {
		<Brand as SendLiftFn>::new(f)
	}

	/// A trait for constructing thread-safe by-reference cloneable function wrappers.
	///
	/// This is the `Ref`-mode counterpart of [`SendLiftFn`]. While `SendLiftFn` wraps
	/// `Fn(A) -> B + Send + Sync` closures, `SendLiftRefFn` wraps
	/// `Fn(&A) -> B + Send + Sync` closures.
	pub trait SendLiftRefFn: SendCloneFn<Ref> {
		/// Creates a new thread-safe cloneable by-reference function wrapper.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type (received by reference).",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to wrap. Must be `Send + Sync`.")]
		#[document_returns("The wrapped thread-safe cloneable by-reference function.")]
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
		/// let f = send_lift_ref_fn_new::<ArcFnBrand, _, _>(|x: &i32| *x * 2);
		///
		/// let handle = thread::spawn(move || {
		/// 	assert_eq!(f(&5), 10);
		/// });
		/// handle.join().unwrap();
		/// ```
		fn new<'a, A: 'a, B: 'a>(
			f: impl 'a + Fn(&A) -> B + Send + Sync
		) -> <Self as SendCloneFn<Ref>>::Of<'a, A, B>;
	}

	/// Creates a new thread-safe cloneable by-reference function wrapper.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendLiftRefFn::new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the thread-safe cloneable function wrapper.",
		"The input type (received by reference).",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to wrap. Must be `Send + Sync`.")]
	#[document_returns("The wrapped thread-safe cloneable by-reference function.")]
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
	/// let f = send_lift_ref_fn_new::<ArcFnBrand, _, _>(|x: &i32| *x * 2);
	///
	/// let handle = thread::spawn(move || {
	/// 	assert_eq!(f(&5), 10);
	/// });
	/// handle.join().unwrap();
	/// ```
	pub fn send_lift_ref_fn_new<'a, Brand, A, B>(
		f: impl 'a + Fn(&A) -> B + Send + Sync
	) -> <Brand as SendCloneFn<Ref>>::Of<'a, A, B>
	where
		Brand: SendLiftRefFn, {
		<Brand as SendLiftRefFn>::new(f)
	}
}

pub use inner::*;
