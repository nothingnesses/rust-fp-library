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
	///
	/// Takes the function first and the monadic value second.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the computation.",
		"The brand of the semimonad.",
		"The type of the value inside the context.",
		"The type of the value in the resulting context."
	)]
	#[document_parameters(
		"A function that receives a reference and returns a new context.",
		"The context containing the value."
	)]
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
	/// let result = ref_bind_flipped::<LazyBrand<RcLazyConfig>, _, _>(
	/// 	|x: &i32| {
	/// 		let v = *x * 2;
	/// 		RcLazy::new(move || v)
	/// 	},
	/// 	lazy,
	/// );
	/// assert_eq!(*result.evaluate(), 10);
	/// ```
	pub fn ref_bind_flipped<'a, Brand: RefSemimonad, A: 'a, B: 'a>(
		f: impl Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_bind(ma, f)
	}

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

	/// Forwards Kleisli composition for by-ref semimonads.
	///
	/// Composes two by-ref monadic functions left-to-right.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the semimonad.",
		"The input type of the first function.",
		"The output type of the first function and input type of the second.",
		"The output type of the second function."
	)]
	#[document_parameters(
		"The first monadic function (receives &A).",
		"The second monadic function (receives &B).",
		"The input value."
	)]
	#[document_returns("The result of composing both monadic functions.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let double = |x: &i32| {
	/// 	let v = *x * 2;
	/// 	RcLazy::new(move || v)
	/// };
	/// let to_str = |x: &i32| {
	/// 	let s = x.to_string();
	/// 	RcLazy::new(move || s)
	/// };
	/// let result = ref_compose_kleisli::<LazyBrand<RcLazyConfig>, _, _, _>(double, to_str, 5);
	/// assert_eq!(*result.evaluate(), "10");
	/// ```
	pub fn ref_compose_kleisli<'a, Brand: RefSemimonad, A: 'a, B: 'a, C: 'a>(
		f: impl Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		g: impl Fn(&B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		Brand::ref_bind(f(&a), g)
	}

	/// Backwards Kleisli composition for by-ref semimonads.
	///
	/// Composes two by-ref monadic functions right-to-left.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the computations.",
		"The brand of the semimonad.",
		"The input type of the second function.",
		"The output type of the second function and input type of the first.",
		"The output type of the first function."
	)]
	#[document_parameters(
		"The second monadic function (applied after g, receives &B).",
		"The first monadic function (applied first, receives &A).",
		"The input value."
	)]
	#[document_returns("The result of composing both monadic functions.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let double = |x: &i32| {
	/// 	let v = *x * 2;
	/// 	RcLazy::new(move || v)
	/// };
	/// let to_str = |x: &i32| {
	/// 	let s = x.to_string();
	/// 	RcLazy::new(move || s)
	/// };
	/// let result = ref_compose_kleisli_flipped::<LazyBrand<RcLazyConfig>, _, _, _>(to_str, double, 5);
	/// assert_eq!(*result.evaluate(), "10");
	/// ```
	pub fn ref_compose_kleisli_flipped<'a, Brand: RefSemimonad, A: 'a, B: 'a, C: 'a>(
		f: impl Fn(&B) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
		g: impl Fn(&A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		a: A,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
		Brand::ref_bind(g(&a), f)
	}
}

pub use inner::*;
