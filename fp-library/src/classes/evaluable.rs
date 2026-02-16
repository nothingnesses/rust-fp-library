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

use {
	crate::{Apply, classes::functor::Functor, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A functor whose effects can be evaluated to produce the inner value.
///
/// This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects
/// in a [`Free`](crate::types::Free) monad.
pub trait Evaluable: Functor {
	/// Evaluates the effect, producing the inner value.
	#[document_signature]
	///
	#[document_type_parameters(
		"The type of the value inside the functor."
	)]
	///
	#[document_parameters("The functor instance to evaluate.")]
	///
	/// ### Returns
	///
	/// The inner value.
	///
	/// ### Examples
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
	fn evaluate<A>(fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>)) -> A;
}

/// Evaluates the effect, producing the inner value.
///
/// Free function version that dispatches to [the type class' associated function][`Evaluable::evaluate`].
#[document_signature]
///
#[document_type_parameters(
	"The evaluable functor.",
	"The type of the value inside the functor."
)]
///
#[document_parameters("The functor instance to evaluable.")]
///
/// ### Returns
///
/// The inner value.
///
/// ### Examples
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
pub fn evaluate<F: Evaluable, A>(fa: Apply!(<F as Kind!( type Of<T>; )>::Of<A>)) -> A {
	F::evaluate(fa)
}
