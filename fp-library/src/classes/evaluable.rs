//! Functors whose effects can be evaluated to produce an inner value.
//!
//! This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects
//! in a [`Free`](crate::types::Free) monad.
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
//! let thunk = Thunk::new(|| 42);
//! assert_eq!(evaluate::<ThunkBrand, _>(thunk), 42);
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

	/// A functor containing exactly one extractable value, providing a natural
	/// transformation `F ~> Id`.
	///
	/// This trait witnesses that a functor always holds a single value that can be
	/// extracted by running its effect. It is used by
	/// [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects in a
	/// [`Free`](crate::types::Free) monad.
	///
	/// Currently only [`ThunkBrand`](crate::brands::ThunkBrand) implements this trait.
	/// [`Lazy`](crate::types::Lazy) cannot implement it because `evaluate` returns `&A`
	/// (a reference), not an owned `A`. [`Trampoline`](crate::types::Trampoline) does not
	/// have a brand and therefore cannot participate in HKT traits.
	pub trait Evaluable: Functor {
		/// Evaluates the effect, producing the inner value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the value.",
			"The type of the value inside the functor."
		)]
		///
		#[document_parameters("The functor instance to evaluate.")]
		///
		#[document_returns("The inner value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let eval = Thunk::new(|| 42);
		/// assert_eq!(evaluate::<ThunkBrand, _>(eval), 42);
		/// ```
		fn evaluate<'a, A: 'a>(
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
		) -> A;
	}

	/// Evaluates the effect, producing the inner value.
	///
	/// Free function version that dispatches to [the type class' associated function][`Evaluable::evaluate`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the value.",
		"The evaluable functor.",
		"The type of the value inside the functor."
	)]
	///
	#[document_parameters("The functor instance to evaluate.")]
	///
	#[document_returns("The inner value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let eval = Thunk::new(|| 42);
	/// assert_eq!(evaluate::<ThunkBrand, _>(eval), 42);
	/// ```
	pub fn evaluate<'a, F, A>(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A
	where
		F: Evaluable,
		A: 'a, {
		F::evaluate(fa)
	}
}

pub use inner::*;
