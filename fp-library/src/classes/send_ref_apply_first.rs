//! Thread-safe combining of two by-ref contexts, keeping the first value, with [`send_ref_apply_first`].
//!
//! This is the thread-safe counterpart of [`RefApplyFirst`](crate::classes::RefApplyFirst).
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
//! let x = ArcLazy::new(|| 3);
//! let y = ArcLazy::new(|| 4);
//! let result = send_ref_apply_first::<LazyBrand<ArcLazyConfig>, _, _>(&x, &y);
//! assert_eq!(*result.evaluate(), 3);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for combining two thread-safe by-ref contexts, keeping the first value.
	///
	/// Requires `A: Clone + Send + Sync` because the closure receives `&A` and must produce
	/// an owned `A`. The default implementation uses [`SendRefLift::send_ref_lift2`](crate::classes::SendRefLift::send_ref_lift2).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefApplyFirst: crate::classes::SendRefLift {
		/// Combines two contexts, keeping the value from the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the first context. Must be `Clone + Send + Sync`.",
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
		/// let x = ArcLazy::new(|| 3);
		/// let y = ArcLazy::new(|| 4);
		/// let result = LazyBrand::<ArcLazyConfig>::send_ref_apply_first(&x, &y);
		/// assert_eq!(*result.evaluate(), 3);
		/// ```
		fn send_ref_apply_first<'a, A: Clone + Send + Sync + 'a, B: Send + Sync + 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::send_ref_lift2(|a: &A, _: &B| a.clone(), fa, fb)
		}
	}

	/// Blanket implementation of [`SendRefApplyFirst`].
	#[document_type_parameters("The brand type.")]
	impl<Brand: crate::classes::SendRefLift> SendRefApplyFirst for Brand {}

	/// Combines two thread-safe contexts, keeping the value from the first.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefApplyFirst::send_ref_apply_first`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value in the first context. Must be `Clone + Send + Sync`.",
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
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let x = ArcLazy::new(|| 3);
	/// let y = ArcLazy::new(|| 4);
	/// let result = send_ref_apply_first::<LazyBrand<ArcLazyConfig>, _, _>(&x, &y);
	/// assert_eq!(*result.evaluate(), 3);
	/// ```
	pub fn send_ref_apply_first<
		'a,
		Brand: SendRefApplyFirst,
		A: Clone + Send + Sync + 'a,
		B: Send + Sync + 'a,
	>(
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::send_ref_apply_first(fa, fb)
	}
}

pub use inner::*;
