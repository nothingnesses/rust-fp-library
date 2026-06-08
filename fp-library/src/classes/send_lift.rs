//! Thread-safe by-value lifting of binary functions with [`send_lift2`].
//!
//! Like [`Lift::lift2`](crate::classes::Lift::lift2), but the function and
//! the value types must be `Send + Sync` so the result can cross thread
//! boundaries. By-value parallel of
//! [`SendRefLift`](crate::classes::SendRefLift).
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
//! let z = send_lift2::<OptionBrand, _, _, _>(|a: i32, b: i32| a + b, x, y);
//! assert_eq!(z, Some(7));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for thread-safe by-value lifting of binary functions
	/// into a context.
	///
	/// This is the thread-safe by-value counterpart of
	/// [`Lift`](crate::classes::Lift). The `Send + Sync` bounds on the
	/// closure parameter and on `A` / `B` / `C` ensure the lifted result
	/// can cross thread boundaries.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendLift {
		/// Lifts a thread-safe binary function over two contexts.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value. Must be `Clone + Send + Sync`.",
			"The type of the second value. Must be `Clone + Send + Sync`.",
			"The type of the result. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The function to lift. Must be `Send + Sync`.",
			"The first context.",
			"The second context."
		)]
		///
		#[document_returns("A new context containing the result of applying the function.")]
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
		/// let z = OptionBrand::send_lift2(|a: i32, b: i32| a + b, x, y);
		/// assert_eq!(z, Some(7));
		/// ```
		fn send_lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + Send + Sync + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + Send + Sync + 'a,
			B: Clone + Send + Sync + 'a,
			C: Send + Sync + 'a;
	}

	/// Lifts a thread-safe binary function over two contexts.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendLift::send_lift2`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the first value. Must be `Clone + Send + Sync`.",
		"The type of the second value. Must be `Clone + Send + Sync`.",
		"The type of the result. Must be `Send + Sync`."
	)]
	///
	#[document_parameters(
		"The function to lift. Must be `Send + Sync`.",
		"The first context.",
		"The second context."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
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
	/// let z = send_lift2::<OptionBrand, _, _, _>(|a: i32, b: i32| a + b, x, y);
	/// assert_eq!(z, Some(7));
	/// ```
	pub fn send_lift2<'a, Brand: SendLift, A, B, C>(
		func: impl Fn(A, B) -> C + Send + Sync + 'a,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		A: Clone + Send + Sync + 'a,
		B: Clone + Send + Sync + 'a,
		C: Send + Sync + 'a, {
		Brand::send_lift2(func, fa, fb)
	}
}

pub use inner::*;
