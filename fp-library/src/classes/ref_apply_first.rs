//! Combining two by-ref contexts, keeping the first value, with [`ref_apply_first`].
//!
//! This is the by-ref counterpart of [`ApplyFirst`](crate::classes::ApplyFirst).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::{
//! 		explicit::apply_first,
//! 		*,
//! 	},
//! 	types::*,
//! };
//!
//! let x = RcLazy::pure(3);
//! let y = RcLazy::pure(4);
//! let result = apply_first::<LazyBrand<RcLazyConfig>, _, _, _, _>(&x, &y);
//! assert_eq!(*result.evaluate(), 3);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for combining two by-ref contexts, keeping the first value.
	///
	/// Requires `A: Clone` because the closure receives `&A` and must produce
	/// an owned `A`. The default implementation uses [`RefLift::ref_lift2`](crate::classes::RefLift::ref_lift2).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefApplyFirst: crate::classes::RefLift {
		/// Combines two contexts, keeping the value from the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the first context. Must be `Clone`.",
			"The type of the value in the second context."
		)]
		///
		#[document_parameters("The first context.", "The second context.")]
		///
		#[document_returns("A new context containing the value from the first context.")]
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
		/// let result = LazyBrand::<RcLazyConfig>::ref_apply_first(&x, &y);
		/// assert_eq!(*result.evaluate(), 3);
		/// ```
		fn ref_apply_first<'a, A: Clone + 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::ref_lift2(|a: &A, _: &B| a.clone(), fa, fb)
		}
	}

	/// Blanket implementation of [`RefApplyFirst`].
	#[document_type_parameters("The brand type.")]
	impl<Brand: crate::classes::RefLift> RefApplyFirst for Brand {}

	/// Combines two contexts, keeping the value from the first.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefApplyFirst::ref_apply_first`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value in the first context. Must be `Clone`.",
		"The type of the value in the second context."
	)]
	///
	#[document_parameters("The first context.", "The second context.")]
	///
	#[document_returns("A new context containing the value from the first context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::{
	/// 		explicit::apply_first,
	/// 		*,
	/// 	},
	/// 	types::*,
	/// };
	///
	/// let x = RcLazy::pure(3);
	/// let y = RcLazy::pure(4);
	/// let result = apply_first::<LazyBrand<RcLazyConfig>, _, _, _, _>(&x, &y);
	/// assert_eq!(*result.evaluate(), 3);
	/// ```
	pub fn ref_apply_first<'a, Brand: RefApplyFirst, A: Clone + 'a, B: 'a>(
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::ref_apply_first(fa, fb)
	}
}

pub use inner::*;
