//! Composable function wrappers with [`Category`](crate::classes::Category) and [`Strong`](crate::classes::profunctor::Strong) instances.
//!
//! The [`Arrow`] trait provides composable, callable wrappers over closures.
//! It extends [`Category`](crate::classes::Category) and [`Strong`](crate::classes::profunctor::Strong),
//! aligning with Haskell's `Arrow` type class.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::classes::{
			profunctor::*,
			*,
		},
		fp_macros::*,
		std::ops::Deref,
	};

	/// A trait for composable function wrappers with [`Category`](crate::classes::Category) and [`Strong`] instances.
	///
	/// This trait is implemented by "Brand" types (like [`ArcFnBrand`][crate::brands::ArcFnBrand]
	/// and [`RcFnBrand`][crate::brands::RcFnBrand]) to provide a way to construct
	/// and type-check wrappers over closures (`Arc<dyn Fn...>`, `Rc<dyn Fn...>`,
	/// etc.) in a generic context, allowing library users to choose between
	/// implementations at function call sites.
	///
	/// Unlike [`CloneFn`](crate::classes::CloneFn), which provides cloneable
	/// wrappers for use in applicative contexts, `Arrow` provides composable
	/// wrappers for use in the optics system.
	///
	/// The lifetime `'a` ensures the function doesn't outlive referenced data,
	/// while generic types `A` and `B` represent the input and output types, respectively.
	pub trait Arrow: Category + Strong {
		/// The type of the function wrapper.
		///
		/// This associated type represents the concrete type of the wrapper (e.g., `Rc<dyn Fn(A) -> B>`)
		/// that dereferences to the underlying closure.
		type Of<'a, A: 'a, B: 'a>: Deref<Target = dyn 'a + Fn(A) -> B>;

		/// Lifts a pure function into an arrow.
		///
		/// This function wraps the provided closure `f` into a composable function wrapper.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the function and its captured data.",
			"The input type of the function.",
			"The output type of the function."
		)]
		///
		#[document_parameters("The closure to lift into an arrow.")]
		#[document_returns("The wrapped function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// assert_eq!(f(5), 10);
		/// ```
		fn arrow<'a, A: 'a, B: 'a>(f: impl 'a + Fn(A) -> B) -> <Self as Arrow>::Of<'a, A, B>;
	}

	// No free function here: the `arrow` free function already exists in
	// `profunctor.rs` and is re-exported via `functions.rs`. It lifts a
	// pure function into any `Category + Profunctor` (which `Arrow`
	// satisfies via its supertraits).
}

pub use inner::*;
