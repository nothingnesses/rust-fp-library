//! Cloneable wrappers over closures for generic handling of functions in higher-kinded contexts.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::*,
		fp_macros::*,
		std::ops::Deref,
	};

	/// A trait for cloneable wrappers over closures, allowing for generic handling of cloneable functions in higher-kinded contexts.
	///
	/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand]
	/// and [`RcFnBrand`][crate::brands::RcFnBrand]) to provide a way to construct
	/// and type-check cloneable wrappers over closures (`Arc<dyn Fn...>` or
	/// `Rc<dyn Fn...>`) in a generic context, allowing library users to choose
	/// between implementations at function call sites.
	///
	/// The lifetime `'a` ensures the function doesn't outlive referenced data,
	/// while generic types `A` and `B` represent the input and output types, respectively.
	pub trait CloneableFn: Function {
		/// The pointer brand backing this function wrapper.
		///
		/// Each `CloneableFn` implementor is backed by exactly one reference-counted
		/// pointer type. For [`FnBrand<P>`](crate::brands::FnBrand), this is `P`.
		type PointerBrand: RefCountedPointer;

		/// The type of the cloneable function wrapper.
		///
		/// This associated type represents the concrete type of the wrapper (e.g., `Rc<dyn Fn(A) -> B>`)
		/// that implements `Clone` and dereferences to the underlying closure.
		type Of<'a, A: 'a, B: 'a>: 'a + Clone + Deref<Target = dyn 'a + Fn(A) -> B>;

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
		#[document_parameters("The closure to wrap.", "The input value to the function.")]
		#[document_returns("The wrapped cloneable function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(5), 10);
		/// ```
		fn new<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as CloneableFn>::Of<'a, A, B>;
	}

	/// Creates a new cloneable function wrapper.
	///
	/// Free function version that dispatches to [the type class' associated function][`CloneableFn::new`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The brand of the cloneable function wrapper.",
		"The input type of the function.",
		"The output type of the function."
	)]
	///
	#[document_parameters("The closure to wrap.", "The input value to the function.")]
	#[document_returns("The wrapped cloneable function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	pub fn new<'a, Brand, A, B>(f: impl 'a + Fn(A) -> B) -> <Brand as CloneableFn>::Of<'a, A, B>
	where
		Brand: CloneableFn, {
		<Brand as CloneableFn>::new(f)
	}
}

pub use inner::*;
