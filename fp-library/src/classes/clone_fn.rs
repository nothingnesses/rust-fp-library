//! Cloneable wrappers over closures for generic handling of functions in higher-kinded contexts.
//!
//! The [`CloneFn`] trait defines the type of the wrapper, parameterized by
//! a [`ClosureMode`](crate::classes::dispatch::ClosureMode) that determines whether the
//! wrapped closure takes owned values ([`Val`](crate::classes::dispatch::Val)) or
//! references ([`Ref`](crate::classes::dispatch::Ref)).
//!
//! The [`LiftFn`] trait provides construction of Val-mode wrapped functions.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::{
			RefCountedPointer,
			dispatch::{
				ClosureMode,
				Val,
			},
		},
		fp_macros::*,
		std::ops::Deref,
	};

	/// A trait for cloneable wrappers over closures, parameterized by closure mode.
	///
	/// This trait is implemented by "Brand" types (like [`RcFnBrand`][crate::brands::RcFnBrand])
	/// to provide a way to type-check cloneable wrappers over closures (`Rc<dyn Fn...>` or
	/// `Arc<dyn Fn...>`) in a generic context.
	///
	/// The `Mode` parameter determines the `Deref` target:
	/// - [`Val`]: wraps `Fn(A) -> B` (by-value closures)
	/// - [`Ref`]: wraps `Fn(&A) -> B` (by-reference closures)
	///
	/// The default mode is [`Val`], so existing code using `CloneFn` without
	/// a mode parameter is unchanged.
	///
	/// For construction of wrapped functions, see [`LiftFn`].
	#[document_type_parameters(
		"The closure mode. Either [`Val`] (by-value, default) or [`Ref`] (by-reference)."
	)]
	pub trait CloneFn<Mode: ClosureMode = Val> {
		/// The pointer brand backing this function wrapper.
		///
		/// Each `CloneFn` implementor is backed by exactly one reference-counted
		/// pointer type. For [`FnBrand<P>`](crate::brands::FnBrand), this is `P`.
		type PointerBrand: RefCountedPointer;

		/// The type of the cloneable function wrapper.
		///
		/// This associated type represents the concrete type of the wrapper (e.g., `Rc<dyn Fn(A) -> B>`)
		/// that implements `Clone` and dereferences to the underlying closure.
		type Of<'a, A: 'a, B: 'a>: 'a + Clone + Deref<Target = Mode::Target<'a, A, B>>;
	}

	/// A trait for constructing cloneable function wrappers from closures.
	///
	/// This is separated from [`CloneFn`] because the closure parameter type
	/// (`Fn(A) -> B`) is specific to [`Val`] mode. By-reference mode
	/// (`CloneFn<Ref>`) uses
	/// `coerce_ref_fn` for
	/// construction instead.
	pub trait LiftFn: CloneFn<Val> {
		/// Creates a new cloneable function wrapper.
		///
		/// This function wraps the provided closure `f` into a cloneable function.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to wrap.")]
		#[document_returns("The wrapped cloneable function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(5), 10);
		/// ```
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as CloneFn>::Of<'a, A, B>;
	}

	/// Creates a new cloneable function wrapper.
	///
	/// Free function version that dispatches to [the type class' associated function][`LiftFn::new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the cloneable function wrapper.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to wrap.")]
	#[document_returns("The wrapped cloneable function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	pub fn new<'a, Brand, A, B>(f: impl 'a + Fn(A) -> B) -> <Brand as CloneFn>::Of<'a, A, B>
	where
		Brand: LiftFn, {
		<Brand as LiftFn>::new(f)
	}
}

pub use inner::*;
