//! Contexts supporting by-reference monadic sequencing via [`ref_bind`].
//!
//! Like [`Semimonad::bind`](crate::classes::Semimonad::bind), but the closure
//! receives `&A` instead of `A`. This enables memoized types like
//! [`Lazy`](crate::types::Lazy) to participate in monadic sequencing without
//! giving up their cached value.
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
//! let lazy = RcLazy::pure(5);
//! let result = LazyBrand::<RcLazyConfig>::ref_bind(lazy, |x: &i32| {
//! 	Lazy::<_, RcLazyConfig>::new({
//! 		let v = *x;
//! 		move || v * 2
//! 	})
//! });
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for contexts supporting by-reference monadic sequencing.
	///
	/// The closure receives `&A` and decides what to do with it, including
	/// whether to clone. No `Clone` bound is imposed by the trait.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefSemimonad {
		/// Sequences a computation using a reference to the contained value.
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
			"A function that receives a reference to the value and returns a new context."
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
		/// let lazy = RcLazy::pure(5);
		/// let result = LazyBrand::<RcLazyConfig>::ref_bind(lazy, |x: &i32| {
		/// 	Lazy::<_, RcLazyConfig>::new({
		/// 		let v = *x;
		/// 		move || v * 2
		/// 	})
		/// });
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Sequences a computation using a reference to the contained value.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefSemimonad::ref_bind`].
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
		"A function that receives a reference to the value and returns a new context."
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
	/// let lazy = RcLazy::pure(5);
	/// let result = ref_bind::<LazyBrand<RcLazyConfig>, _, _>(lazy, |x: &i32| {
	/// 	Lazy::<_, RcLazyConfig>::new({
	/// 		let v = *x;
	/// 		move || v * 2
	/// 	})
	/// });
	/// assert_eq!(*result.evaluate(), 10);
	/// ```
	pub fn ref_bind<'a, Brand: RefSemimonad, A: 'a, B: 'a>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_bind(fa, f)
	}
}

pub use inner::*;
