//! Thread-safe by-value combining of two contexts, keeping the first value, with [`send_apply_first`].
//!
//! By-value parallel of [`SendRefApplyFirst`](crate::classes::SendRefApplyFirst).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(3);
//! let y = Some(4);
//! let result = send_apply_first::<OptionBrand, _, _>(x, y);
//! assert_eq!(result, Some(3));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for combining two thread-safe contexts, keeping the
	/// first value.
	///
	/// Requires `A: Clone + Send + Sync` because the closure receives `A`
	/// by value (consumed) and must produce an owned `A`. The default
	/// implementation uses
	/// [`SendLift::send_lift2`](crate::classes::SendLift::send_lift2).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendApplyFirst: crate::classes::SendLift {
		/// Combines two contexts, keeping the value from the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value in the first context. Must be `Clone + Send + Sync`.",
			"The type of the value in the second context. Must be `Clone + Send + Sync`."
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
		/// };
		///
		/// let x = Some(3);
		/// let y = Some(4);
		/// let result = OptionBrand::send_apply_first(x, y);
		/// assert_eq!(result, Some(3));
		/// ```
		fn send_apply_first<'a, A: Clone + Send + Sync + 'a, B: Clone + Send + Sync + 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Self::send_lift2(|a: A, _: B| a, fa, fb)
		}
	}

	/// Blanket implementation of [`SendApplyFirst`].
	#[document_type_parameters("The brand type.")]
	impl<Brand: crate::classes::SendLift> SendApplyFirst for Brand {}

	/// Combines two thread-safe contexts, keeping the value from the first.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendApplyFirst::send_apply_first`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value in the first context. Must be `Clone + Send + Sync`.",
		"The type of the value in the second context. Must be `Clone + Send + Sync`."
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
	/// };
	///
	/// let x = Some(3);
	/// let y = Some(4);
	/// let result = send_apply_first::<OptionBrand, _, _>(x, y);
	/// assert_eq!(result, Some(3));
	/// ```
	pub fn send_apply_first<
		'a,
		Brand: SendApplyFirst,
		A: Clone + Send + Sync + 'a,
		B: Clone + Send + Sync + 'a,
	>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Brand::send_apply_first(fa, fb)
	}
}

pub use inner::*;
