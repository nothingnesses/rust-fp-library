//! Combining two by-ref contexts, keeping the second value, with [`ref_apply_second`].
//!
//! This is the by-ref counterpart of [`ApplySecond`](crate::classes::ApplySecond).
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
//! let result = apply_second_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _>(&x, &y);
//! assert_eq!(*result.evaluate(), 4);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for combining two by-ref contexts, keeping the second value.
	///
	/// Requires `B: Clone` because the closure receives `&B` and must produce
	/// an owned `B`. The default implementation uses [`RefLift::ref_lift2`](crate::classes::RefLift::ref_lift2).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefApplySecond: crate::classes::RefLift {
		/// Combines two contexts, keeping the value from the second.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the first context.",
			"The type of the value in the second context. Must be `Clone`."
		)]
		///
		#[document_parameters("The first context.", "The second context.")]
		///
		#[document_returns("A new context containing the value from the second context.")]
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
		/// let result = LazyBrand::<RcLazyConfig>::ref_apply_second(&x, &y);
		/// assert_eq!(*result.evaluate(), 4);
		/// ```
		fn ref_apply_second<'a, A: 'a, B: Clone + 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::ref_lift2(|_: &A, b: &B| b.clone(), fa, fb)
		}
	}

	/// Blanket implementation of [`RefApplySecond`].
	#[document_type_parameters("The brand type.")]
	impl<Brand: crate::classes::RefLift> RefApplySecond for Brand {}

	/// Combines two contexts, keeping the value from the second.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefApplySecond::ref_apply_second`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value in the first context.",
		"The type of the value in the second context. Must be `Clone`."
	)]
	///
	#[document_parameters("The first context.", "The second context.")]
	///
	#[document_returns("A new context containing the value from the second context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let x = RcLazy::pure(3);
	/// let y = RcLazy::pure(4);
	/// let result = apply_second_explicit::<LazyBrand<RcLazyConfig>, _, _, _, _>(&x, &y);
	/// assert_eq!(*result.evaluate(), 4);
	/// ```
	pub fn ref_apply_second<'a, Brand: RefApplySecond, A: 'a, B: Clone + 'a>(
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::ref_apply_second(fa, fb)
	}
}

pub use inner::*;
