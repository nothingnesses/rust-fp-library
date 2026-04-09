//! Thread-safe by-ref monadic sequencing with [`send_ref_bind`].
//!
//! Like [`RefSemimonad::ref_bind`](crate::classes::RefSemimonad::ref_bind), but
//! the continuation must be `Send` and element types must be `Send + Sync`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let lazy = ArcLazy::new(|| 5);
//! let result = send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(&lazy, |x: &i32| {
//! 	let v = *x * 2;
//! 	ArcLazy::new(move || v)
//! });
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for thread-safe monadic sequencing via references.
	///
	/// This is the thread-safe counterpart of [`RefSemimonad`](crate::classes::RefSemimonad).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefSemimonad {
		/// Sequences a thread-safe computation using a reference to the value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the context.",
			"The type of the value in the resulting context."
		)]
		///
		#[document_parameters(
			"The context containing the value.",
			"A thread-safe function that receives a reference to the value and returns a new context."
		)]
		///
		#[document_returns("A new context produced by the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 5);
		/// let result = LazyBrand::<ArcLazyConfig>::send_ref_bind(&lazy, |x: &i32| {
		/// 	let v = *x * 2;
		/// 	ArcLazy::new(move || v)
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn send_ref_bind<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			ma: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + Send + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Sequences a thread-safe computation using a reference to the value.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefSemimonad::send_ref_bind`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value inside the context.",
		"The type of the value in the resulting context."
	)]
	///
	#[document_parameters(
		"The context containing the value.",
		"A thread-safe function that receives a reference to the value and returns a new context."
	)]
	///
	#[document_returns("A new context produced by the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = ArcLazy::new(|| 5);
	/// let result = send_ref_bind::<LazyBrand<ArcLazyConfig>, _, _>(&lazy, |x: &i32| {
	/// 	let v = *x * 2;
	/// 	ArcLazy::new(move || v)
	/// });
	/// assert_eq!(*result.evaluate(), 10);
	/// ```
	pub fn send_ref_bind<'a, Brand: SendRefSemimonad, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		ma: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + Send + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_ref_bind(ma, f)
	}
}

pub use inner::*;
