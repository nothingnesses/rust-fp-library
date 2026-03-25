//! Types that can be mapped over by receiving or returning references to their contents.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
//! let mapped = ref_map::<LazyBrand<RcLazyConfig>, _, _>(|x: &i32| *x * 2, memo);
//! assert_eq!(*mapped.evaluate(), 20);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for types that can be mapped over, returning references.
	///
	/// This is a variant of `Functor` for types where `map` receives/returns references.
	/// This is required for types like `Lazy` where `get()` returns `&A`, not `A`.
	///
	/// `RefFunctor` is intentionally independent from `SendRefFunctor`. Although one might
	/// expect `SendRefFunctor` to be a subtrait of `RefFunctor`, this is not the case because
	/// `ArcLazy::new` requires `Send` on the closure, which a generic `RefFunctor` cannot
	/// guarantee. As a result, `ArcLazy` implements only `SendRefFunctor`, not `RefFunctor`,
	/// and `RcLazy` implements only `RefFunctor`, not `SendRefFunctor`. A future
	/// `SendRefFunctor` trait will serve as the thread-safe counterpart.
	///
	/// # Laws
	///
	/// **Identity:** `ref_map(|x| x.clone(), fa)` is equivalent to `fa`, given `A: Clone`.
	/// The `Clone` requirement arises because the mapping function receives `&A` but must
	/// produce a value of type `A` to satisfy the identity law.
	///
	/// **Composition:** `ref_map(|x| g(&f(x)), fa)` is equivalent to
	/// `ref_map(g, ref_map(f, fa))`.
	///
	/// # Why `FnOnce`?
	///
	/// The `func` parameter uses `FnOnce` rather than `Fn` because memoized types like
	/// `Lazy` create a new `Lazy` value capturing the closure. Since the resulting `Lazy`
	/// will evaluate the closure at most once, `FnOnce` is sufficient and avoids imposing
	/// unnecessary `Clone` or multi-call constraints on the caller.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFunctor {
		/// Maps a function over the values in the functor context, where the function takes a reference.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
		/// let mapped = LazyBrand::<RcLazyConfig>::ref_map(|x: &i32| *x * 2, memo);
		/// assert_eq!(*mapped.evaluate(), 20);
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl FnOnce(&A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Maps a function over the values in the functor context, where the function takes a reference.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFunctor::ref_map`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s) inside the functor.",
		"The functor instance containing the value(s)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let memo = Lazy::<_, RcLazyConfig>::new(|| 10);
	/// let mapped = ref_map::<LazyBrand<RcLazyConfig>, _, _>(|x: &i32| *x * 2, memo);
	/// assert_eq!(*mapped.evaluate(), 20);
	/// ```
	pub fn ref_map<'a, Brand: RefFunctor, A: 'a, B: 'a>(
		func: impl FnOnce(&A) -> B + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_map(func, fa)
	}
}

pub use inner::*;
