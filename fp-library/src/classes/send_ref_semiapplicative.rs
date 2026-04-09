//! Thread-safe by-ref function application within contexts with [`send_ref_apply`].
//!
//! Like [`RefSemiapplicative::ref_apply`](crate::classes::RefSemiapplicative::ref_apply),
//! but uses [`SendCloneFn<Ref>`](crate::classes::SendCloneFn) for thread-safe
//! function wrappers and requires element types to be `Send + Sync`.
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
//! let f = ArcLazy::new(|| {
//! 	std::sync::Arc::new(|x: &i32| *x * 2) as std::sync::Arc<dyn Fn(&i32) -> i32 + Send + Sync>
//! });
//! let x = ArcLazy::new(|| 5);
//! let result = send_ref_apply::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(&f, &x);
//! assert_eq!(*result.evaluate(), 10);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				dispatch::Ref,
				*,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for applying wrapped thread-safe by-ref functions within contexts.
	///
	/// The wrapped functions have type `Fn(&A) -> B + Send + Sync` (via
	/// [`SendCloneFn<Ref>`]). No `Clone` bound on `A` is needed; the
	/// function receives a reference and produces an owned result.
	///
	/// This is the thread-safe counterpart of [`RefSemiapplicative`].
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefSemiapplicative: SendRefLift + SendRefFunctor {
		/// Applies a wrapped thread-safe by-ref function to a value within a context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the thread-safe cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The context containing the wrapped thread-safe by-ref function.",
			"The context containing the value."
		)]
		///
		#[document_returns("A new context containing the result of applying the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	types::*,
		/// };
		///
		/// let f = ArcLazy::new(|| {
		/// 	std::sync::Arc::new(|x: &i32| *x * 2) as std::sync::Arc<dyn Fn(&i32) -> i32 + Send + Sync>
		/// });
		/// let x = ArcLazy::new(|| 5);
		/// let result = LazyBrand::<ArcLazyConfig>::send_ref_apply::<ArcFnBrand, _, _>(&f, &x);
		/// assert_eq!(*result.evaluate(), 10);
		/// ```
		fn send_ref_apply<
			'a,
			FnBrand: 'a + SendCloneFn<Ref>,
			A: Send + Sync + 'a,
			B: Send + Sync + 'a,
		>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as SendCloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Applies a wrapped thread-safe by-ref function to a value within a context.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefSemiapplicative::send_ref_apply`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the thread-safe cloneable function wrapper.",
		"The brand of the context.",
		"The type of the input value.",
		"The type of the output value."
	)]
	///
	#[document_parameters(
		"The context containing the wrapped thread-safe by-ref function.",
		"The context containing the value."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let f = ArcLazy::new(|| {
	/// 	std::sync::Arc::new(|x: &i32| *x * 2) as std::sync::Arc<dyn Fn(&i32) -> i32 + Send + Sync>
	/// });
	/// let x = ArcLazy::new(|| 5);
	/// let result = send_ref_apply::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(&f, &x);
	/// assert_eq!(*result.evaluate(), 10);
	/// ```
	pub fn send_ref_apply<
		'a,
		FnBrand: 'a + SendCloneFn<Ref>,
		Brand: SendRefSemiapplicative,
		A: Send + Sync + 'a,
		B: Send + Sync + 'a,
	>(
		ff: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as SendCloneFn<Ref>>::Of<'a, A, B>>),
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_ref_apply::<FnBrand, A, B>(ff, fa)
	}
}

pub use inner::*;
