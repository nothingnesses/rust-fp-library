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

	/// A functor whose effects can be evaluated to produce the inner value.
	///
	/// This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects
	/// in a [`Free`](crate::types::Free) monad.
	///
	/// # Laws
	///
	/// **Naturality:** `evaluate` commutes with natural transformations. Given a natural
	/// transformation `nat: F<A> -> G<A>` and an evaluable functor `fa: F<A>`, the following
	/// must hold:
	///
	/// ```text
	/// evaluate(nat(fa)) == evaluate(fa)
	/// ```
	///
	/// In other words, if `nat` is a structure-preserving transformation between two
	/// evaluable functors, then evaluating after transforming is the same as evaluating
	/// directly. This ensures that `evaluate` extracts the "content" of the functor
	/// regardless of the particular functor wrapper used.
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
	#[document_parameters("The functor instance to evaluable.")]
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
