//! Thread-safe by-value monadic sequencing with [`send_bind`].
//!
//! Like [`Semimonad::bind`](crate::classes::Semimonad::bind), but the
//! continuation must be `Send + Sync` and the value types `Send + Sync` so
//! the result can cross thread boundaries. By-value parallel of
//! [`SendRefSemimonad`](crate::classes::SendRefSemimonad).
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = send_bind::<OptionBrand, _, _>(x, |i: i32| Some(i * 2));
//! assert_eq!(y, Some(10));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for thread-safe by-value monadic sequencing.
	///
	/// This is the thread-safe by-value counterpart of
	/// [`Semimonad`](crate::classes::Semimonad). The `Send + Sync` bound on
	/// the continuation parameter ensures it can be stored in
	/// thread-safe containers, and the `Send + Sync` bounds on `A` and `B`
	/// ensure the values flowing through the chain are themselves
	/// thread-safe.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendSemimonad {
		/// Sequences a thread-safe computation by-value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value inside the context. Must be `Send + Sync`.",
			"The type of the value in the resulting context. Must be `Send + Sync`."
		)]
		///
		#[document_parameters(
			"The context containing the value.",
			"A thread-safe function that consumes the value and returns a new context."
		)]
		///
		#[document_returns("A new context produced by the function.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = OptionBrand::send_bind(x, |i: i32| Some(i * 2));
		/// assert_eq!(y, Some(10));
		/// ```
		fn send_bind<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
			+ Send
			+ Sync
			+ 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Sequences a thread-safe computation by-value.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendSemimonad::send_bind`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the context.",
		"The type of the value inside the context.",
		"The type of the value in the resulting context."
	)]
	///
	#[document_parameters(
		"The context containing the value.",
		"A thread-safe function that consumes the value and returns a new context."
	)]
	///
	#[document_returns("A new context produced by the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = send_bind::<OptionBrand, _, _>(x, |i: i32| Some(i * 2));
	/// assert_eq!(y, Some(10));
	/// ```
	pub fn send_bind<'a, Brand: SendSemimonad, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		ma: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: impl Fn(A) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		+ Send
		+ Sync
		+ 'a,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_bind(ma, f)
	}
}

pub use inner::*;
