//! Contexts that can be initialized from a reference via the [`ref_pure`] operation.
//!
//! Unlike [`Pointed::pure`](crate::classes::Pointed::pure), which takes ownership of
//! the value, `ref_pure` accepts a reference and clones the value to produce the
//! context. This enables by-reference generic code to construct contexts without
//! requiring ownership.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	types::*,
//! };
//!
//! let value = 42;
//! let lazy = LazyBrand::<RcLazyConfig>::ref_pure(&value);
//! assert_eq!(*lazy.evaluate(), 42);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for contexts that can be initialized from a reference.
	///
	/// The `Clone` bound on `A` is required because constructing an owned
	/// `Of<A>` from `&A` inherently requires cloning. This is the only
	/// by-reference trait with a `Clone` bound; all other by-ref traits
	/// pass `&A` to a user-supplied closure, letting the user control
	/// whether cloning happens.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefPointed {
		/// Wraps a cloned value in the context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value to wrap. Must be `Clone`."
		)]
		///
		#[document_parameters("A reference to the value to wrap.")]
		///
		#[document_returns("A new context containing a clone of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let value = 42;
		/// let lazy = LazyBrand::<RcLazyConfig>::ref_pure(&value);
		/// assert_eq!(*lazy.evaluate(), 42);
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	/// Wraps a cloned value in the context.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefPointed::ref_pure`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The brand of the context.",
		"The type of the value to wrap. Must be `Clone`."
	)]
	///
	#[document_parameters("A reference to the value to wrap.")]
	///
	#[document_returns("A new context containing a clone of the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let value = 42;
	/// let lazy = ref_pure::<LazyBrand<RcLazyConfig>, _>(&value);
	/// assert_eq!(*lazy.evaluate(), 42);
	/// ```
	pub fn ref_pure<'a, Brand: RefPointed, A: Clone + 'a>(
		a: &A
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::ref_pure(a)
	}
}

pub use inner::*;
