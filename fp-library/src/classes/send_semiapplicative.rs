//! Thread-safe by-value function application within contexts with [`send_apply`].
//!
//! Like [`Semiapplicative::apply`](crate::classes::Semiapplicative::apply),
//! but uses [`SendCloneFn`](crate::classes::SendCloneFn) for thread-safe
//! function wrappers and requires element types to be `Send + Sync`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f: Option<std::sync::Arc<dyn Fn(i32) -> i32 + Send + Sync>> =
//! 	Some(std::sync::Arc::new(|x: i32| x * 2));
//! let x = Some(5);
//! let result = send_apply::<ArcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(result, Some(10));
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
		},
		fp_macros::*,
	};

	/// A type class for applying wrapped thread-safe by-value functions
	/// within contexts.
	///
	/// The wrapped functions have type `Fn(A) -> B + Send + Sync` (via
	/// [`SendCloneFn`]). This is the thread-safe by-value counterpart of
	/// [`Semiapplicative`].
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendSemiapplicative: SendLift + SendFunctor {
		/// Applies a wrapped thread-safe by-value function to a value within a context.
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
			"The context containing the wrapped thread-safe by-value function.",
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
		/// };
		///
		/// let f: Option<std::sync::Arc<dyn Fn(i32) -> i32 + Send + Sync>> =
		/// 	Some(std::sync::Arc::new(|x: i32| x * 2));
		/// let x = Some(5);
		/// let result = OptionBrand::send_apply::<ArcFnBrand, _, _>(f, x);
		/// assert_eq!(result, Some(10));
		/// ```
		fn send_apply<
			'a,
			FnBrand: 'a + SendCloneFn,
			A: Clone + Send + Sync + 'a,
			B: Send + Sync + 'a,
		>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as SendCloneFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	/// Applies a wrapped thread-safe by-value function to a value within a context.
	///
	/// Free function version that dispatches to [the type class' associated function][`SendSemiapplicative::send_apply`].
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
		"The context containing the wrapped thread-safe by-value function.",
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
	/// };
	///
	/// let f: Option<std::sync::Arc<dyn Fn(i32) -> i32 + Send + Sync>> =
	/// 	Some(std::sync::Arc::new(|x: i32| x * 2));
	/// let x = Some(5);
	/// let result = send_apply::<ArcFnBrand, OptionBrand, _, _>(f, x);
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn send_apply<
		'a,
		FnBrand: 'a + SendCloneFn,
		Brand: SendSemiapplicative,
		A: Clone + Send + Sync + 'a,
		B: Send + Sync + 'a,
	>(
		ff: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as SendCloneFn>::Of<'a, A, B>>),
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		Brand::send_apply::<FnBrand, A, B>(ff, fa)
	}
}

pub use inner::*;
