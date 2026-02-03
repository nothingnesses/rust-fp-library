//! Functors whose effects can be evaluated to produce an inner value.
//!
//! This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects
//! in a [`Free`](crate::types::Free) monad.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*, types::*};
//!
//! let thunk = Thunk::new(|| 42);
//! assert_eq!(evaluate::<ThunkBrand, _>(thunk), 42);
//! ```

use crate::{Apply, classes::functor::Functor, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A functor whose effects can be evaluated to produce the inner value.
///
/// This trait is used by [`Free::evaluate`](crate::types::Free::evaluate) to execute the effects
/// in a [`Free`](crate::types::Free) monad.
pub trait Evaluable: Functor {
	/// Evaluates the effect, producing the inner value.
	///
	/// ### Type Signature
	///
	#[hm_signature]
	///
	/// ### Type Parameters
	///
	#[doc_type_params("The lifetime of the value.", "The type of the value inside the functor.")]
	///
	/// ### Parameters
	///
	#[doc_params("The functor instance to evaluate.")]
	///
	/// ### Returns
	///
	/// The inner value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// let eval = Thunk::new(|| 42);
	/// assert_eq!(evaluate::<ThunkBrand, _>(eval), 42);
	/// ```
	fn evaluate<'a, A: 'a>(fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A;
}

/// Evaluates the effect, producing the inner value.
///
/// Free function version that dispatches to [the type class' associated function][`Evaluable::evaluate`].
///
/// ### Type Signature
///
#[hm_signature]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the value.",
	"The evaluable functor.",
	"The type of the value inside the functor."
)]
///
/// ### Parameters
///
#[doc_params("The functor instance to evaluable.")]
///
/// ### Returns
///
/// The inner value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*, types::*};
///
/// let eval = Thunk::new(|| 42);
/// assert_eq!(evaluate::<ThunkBrand, _>(eval), 42);
/// ```
pub fn evaluate<'a, F, A>(fa: Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> A
where
	F: Evaluable,
	A: 'a,
{
	F::evaluate(fa)
}
