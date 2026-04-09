//! Lifting binary functions into contexts via references with [`lift2`](crate::functions::lift2).
//!
//! Like [`Lift::lift2`](crate::classes::Lift::lift2), but the function receives
//! `&A` and `&B` instead of owned values. No `Clone` bound is needed because the
//! closure controls whether to clone.
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
//! let x = RcLazy::pure(3);
//! let y = RcLazy::pure(4);
//! let z = lift2::<LazyBrand<RcLazyConfig>, _, _, _, _, _, _>(|a: &i32, b: &i32| *a + *b, &x, &y);
//! assert_eq!(*z.evaluate(), 7);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for lifting a binary function into a context using references.
	///
	/// The function receives `&A` and `&B`, so no `Clone` bound is needed.
	/// What the function does with the references (including whether to clone)
	/// is controlled by the caller.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefLift {
		/// Lifts a binary function over two contexts using references.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters("The function to lift.", "The first context.", "The second context.")]
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
		/// let x = RcLazy::pure(3);
		/// let y = RcLazy::pure(4);
		/// let z = LazyBrand::<RcLazyConfig>::ref_lift2(|a: &i32, b: &i32| *a + *b, &x, &y);
		/// assert_eq!(*z.evaluate(), 7);
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>);
	}
}

pub use inner::*;
