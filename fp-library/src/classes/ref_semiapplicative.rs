//! Applying wrapped by-ref functions within contexts with [`ref_apply`].
//!
//! Like [`Semiapplicative::apply`](crate::classes::Semiapplicative::apply), but the
//! wrapped functions receive `&A` instead of owned values. Uses
//! [`CloneFn<Ref>`](crate::classes::CloneFn) for the function wrappers.
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
//! let f = RcLazy::pure(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
//! let x = RcLazy::pure(5);
//! let result = ref_apply::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(f, x);
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				functor_dispatch::Ref,
				*,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for applying wrapped by-ref functions within contexts.
	///
	/// The wrapped functions have type `Fn(&A) -> B` (via [`CloneFn<Ref>`]),
	/// so no `Clone` bound on `A` is needed. The function receives a reference
	/// and produces an owned result.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefSemiapplicative: RefLift + RefFunctor {
		/// Applies a wrapped by-ref function within a context to a value within a context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The context containing the wrapped by-ref function(s).",
			"The context containing the value(s)."
		)]
		///
		#[document_returns(
			"A new context containing the result(s) of applying the function(s) to the value(s)."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = RcLazy::pure(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
		/// let x = RcLazy::pure(5);
		/// let result = LazyBrand::<RcLazyConfig>::ref_apply::<RcFnBrand, _, _>(f, x);
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Applies a wrapped by-ref function within a context to a value within a context.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefSemiapplicative::ref_apply`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The brand of the context.",
		"The type of the input value.",
		"The type of the output value."
	)]
	///
	#[document_parameters(
		"The context containing the wrapped by-ref function(s).",
		"The context containing the value(s)."
	)]
	///
	#[document_returns(
		"A new context containing the result(s) of applying the function(s) to the value(s)."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let f = RcLazy::pure(std::rc::Rc::new(|x: &i32| *x * 2) as std::rc::Rc<dyn Fn(&i32) -> i32>);
	/// let x = RcLazy::pure(5);
	/// let result = ref_apply::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _>(f, x);
	/// assert_eq!(*result.evaluate(), 10);
	/// ```
	pub fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, Brand: RefSemiapplicative, A: 'a, B: 'a>(
		ff: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_apply::<FnBrand, A, B>(ff, fa)
	}
}

pub use inner::*;
