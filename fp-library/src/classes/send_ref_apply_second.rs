//! Thread-safe combining of two by-ref contexts, keeping the second value, with [`send_ref_apply_second`].
//!
//! This is the thread-safe counterpart of [`RefApplySecond`](crate::classes::RefApplySecond).
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
//! let result = send_ref_apply_second::<LazyBrand<ArcLazyConfig>, _, _>(x, y);
//! assert_eq!(*result.evaluate(), 4);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for combining two thread-safe by-ref contexts, keeping the second value.
	///
	/// Requires `B: Clone + Send + Sync` because the closure receives `&B` and must produce
	/// an owned `B`. The default implementation uses [`SendRefLift::send_ref_lift2`](crate::classes::SendRefLift::send_ref_lift2).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefApplySecond: crate::classes::SendRefLift {
		/// Combines two contexts, keeping the value from the second.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the first context.",
			"The type of the value in the second context. Must be `Clone + Send + Sync`."
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
		/// let x = ArcLazy::new(|| 3);
		/// let y = ArcLazy::new(|| 4);
		/// let result = LazyBrand::<ArcLazyConfig>::send_ref_apply_second(x, y);
		/// assert_eq!(*result.evaluate(), 4);
		/// ```
		fn send_ref_apply_second<'a, A: Send + Sync + 'a, B: Clone + Send + Sync + 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::send_ref_lift2(|_: &A, b: &B| b.clone(), fa, fb)
		}
	}

	/// Blanket implementation of [`SendRefApplySecond`].
	#[document_type_parameters("The brand type.")]
	impl<Brand: crate::classes::SendRefLift> SendRefApplySecond for Brand {}

	/// Combines two thread-safe contexts, keeping the value from the second.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefApplySecond::send_ref_apply_second`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value in the first context.",
		"The type of the value in the second context. Must be `Clone + Send + Sync`."
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
	/// let x = ArcLazy::new(|| 3);
	/// let y = ArcLazy::new(|| 4);
	/// let result = send_ref_apply_second::<LazyBrand<ArcLazyConfig>, _, _>(x, y);
	/// assert_eq!(*result.evaluate(), 4);
	/// ```
	pub fn send_ref_apply_second<
		'a,
		Brand: SendRefApplySecond,
		A: Send + Sync + 'a,
		B: Clone + Send + Sync + 'a,
	>(
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_ref_apply_second(fa, fb)
	}
}

pub use inner::*;
