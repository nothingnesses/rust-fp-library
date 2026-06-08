//! Thread-safe by-value combining of two contexts, keeping the second value, with [`send_apply_second`].
//!
//! By-value parallel of [`SendRefApplySecond`](crate::classes::SendRefApplySecond).
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
//! let result = send_apply_second::<OptionBrand, _, _>(x, y);
//! assert_eq!(result, Some(4));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for combining two thread-safe contexts, keeping the
	/// second value.
	///
	/// Requires `B: Clone + Send + Sync` because the closure receives `B`
	/// by value (consumed) and must produce an owned `B`. The default
	/// implementation uses
	/// [`SendLift::send_lift2`](crate::classes::SendLift::send_lift2).
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendApplySecond: crate::classes::SendLift {
		/// Combines two contexts, keeping the value from the second.
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
		#[document_returns("A new context containing the value from the second context.")]
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
		/// let result = OptionBrand::send_apply_second(x, y);
		/// assert_eq!(result, Some(4));
		/// ```
		fn send_apply_second<'a, A: Clone + Send + Sync + 'a, B: Clone + Send + Sync + 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Self::send_lift2(|_: A, b: B| b, fa, fb)
		}
	}

	/// Blanket implementation of [`SendApplySecond`].
	#[document_type_parameters("The brand type.")]
	impl<Brand: crate::classes::SendLift> SendApplySecond for Brand {}

	/// Combines two thread-safe contexts, keeping the value from the second.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendApplySecond::send_apply_second`].
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
	#[document_returns("A new context containing the value from the second context.")]
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
	/// let result = send_apply_second::<OptionBrand, _, _>(x, y);
	/// assert_eq!(result, Some(4));
	/// ```
	pub fn send_apply_second<
		'a,
		Brand: SendApplySecond,
		A: Clone + Send + Sync + 'a,
		B: Clone + Send + Sync + 'a,
	>(
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_apply_second(fa, fb)
	}
}

pub use inner::*;
