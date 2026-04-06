//! Contexts supporting by-reference monadic sequencing via [`bind`](crate::functions::bind).
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

	/// Like [`ref_bind`](crate::functions::bind), but with the arguments flipped.
	/// Collapses two nested layers of a by-ref semimonad into one.
	///
	/// Equivalent to `ref_bind(mma, |ma| ma.clone())`.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the semimonad.",
		"The type of the value inside the nested semimonad."
	)]
	#[document_parameters("The doubly-wrapped semimonadic value.")]
	#[document_returns("The singly-wrapped semimonadic value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let inner = RcLazy::pure(5);
	/// let outer = RcLazy::new({
	/// 	let inner = inner.clone();
	/// 	move || inner.clone()
	/// });
	/// let result = ref_join::<LazyBrand<RcLazyConfig>, _>(outer);
	/// assert_eq!(*result.evaluate(), 5);
	/// ```
	pub fn ref_join<'a, Brand: RefSemimonad, A: 'a>(
		mma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
		Brand::ref_bind(mma, |ma| ma.clone())
	}
}

pub use inner::*;
