//! Choice profunctors, which can lift profunctors through sum types.
//!
//! A choice profunctor allows lifting a profunctor `P A B` to `P (Either C A) (Either C B)`,
//! preserving the alternative context `C`. This is the key constraint for prisms.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Functions are choice profunctors
//! let f = |x: i32| x + 1;
//! let g =
//! 	right::<RcFnBrand, _, _, String>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
//! assert_eq!(g(Ok(10)), Ok(11));
//! assert_eq!(g(Err("error".to_string())), Err("error".to_string()));
//! ```

use {
	super::profunctor::Profunctor,
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for choice profunctors.
///
/// A choice profunctor can lift a profunctor through sum types (Result/Either).
/// This is the profunctor constraint that characterizes prisms.
///
/// ### Hierarchy Unification
///
/// This trait uses the strict Kind signature from [`Kind_266801a817966495`]. This ensures
/// that when lifting a profunctor, the alternative variants of the sum type correctly
/// satisfy lifetime requirements relative to the profunctor's application.
///
/// ### Laws
///
/// `Choice` instances must satisfy the following laws:
/// * Identity: `left(identity) = identity`.
/// * Composition: `left(p ∘ q) = left(p) ∘ left(q)`.
/// * Naturality: `dimap(Left, Left) ∘ left(p) = left(p) ∘ dimap(Left, Left)`.
pub trait Choice: Profunctor {
	/// Lift a profunctor to operate on the left (Err) variant of a Result.
	///
	/// This method takes a profunctor `P A B` and returns `P (Result<C, A>) (Result<C, B>)`,
	/// threading the alternative context `C` through unchanged in the Ok position.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the profunctor.",
		"The output type of the profunctor.",
		"The type of the alternative variant (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to lift.")]
	///
	/// ### Returns
	///
	/// A new profunctor that operates on Result types.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = left::<RcFnBrand, _, _, String>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g(Err(10)), Err(11));
	/// assert_eq!(g(Ok("success".to_string())), Ok("success".to_string()));
	/// ```
	fn left<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>);

	/// Lift a profunctor to operate on the right (Ok) variant of a Result.
	///
	/// This method takes a profunctor `P A B` and returns `P (Result<A, C>) (Result<B, C>)`,
	/// threading the alternative context `C` through unchanged in the Err position.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the profunctor.",
		"The output type of the profunctor.",
		"The type of the alternative variant (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to lift.")]
	///
	/// ### Returns
	///
	/// A new profunctor that operates on Result types.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g =
	/// 	right::<RcFnBrand, _, _, String>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g(Ok(10)), Ok(11));
	/// assert_eq!(g(Err("error".to_string())), Err("error".to_string()));
	/// ```
	fn right<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
	{
		Self::dimap(
			|r: Result<A, C>| match r {
				Ok(a) => Err(a),
				Err(c) => Ok(c),
			},
			|r: Result<C, B>| match r {
				Ok(c) => Err(c),
				Err(b) => Ok(b),
			},
			Self::left(pab),
		)
	}
}

/// Lift a profunctor to operate on the left (Err) variant of a Result.
///
/// Free function version that dispatches to [the type class' associated function][`Choice::left`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the choice profunctor.",
	"The input type of the profunctor.",
	"The output type of the profunctor.",
	"The type of the alternative variant (threaded through unchanged)."
)]
///
#[document_parameters("The profunctor instance to lift.")]
///
/// ### Returns
///
/// A new profunctor that operates on Result types.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let f = |x: i32| x + 1;
/// let g = left::<RcFnBrand, _, _, String>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g(Err(10)), Err(11));
/// assert_eq!(g(Ok("success".to_string())), Ok("success".to_string()));
/// ```
pub fn left<'a, Brand: Choice, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
{
	Brand::left(pab)
}

/// Lift a profunctor to operate on the right (Ok) variant of a Result.
///
/// Free function version that dispatches to [the type class' associated function][`Choice::right`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the choice profunctor.",
	"The input type of the profunctor.",
	"The output type of the profunctor.",
	"The type of the alternative variant (threaded through unchanged)."
)]
///
#[document_parameters("The profunctor instance to lift.")]
///
/// ### Returns
///
/// A new profunctor that operates on Result types.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let f = |x: i32| x + 1;
/// let g =
/// 	right::<RcFnBrand, _, _, String>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g(Ok(10)), Ok(11));
/// assert_eq!(g(Err("error".to_string())), Err("error".to_string()));
/// ```
pub fn right<'a, Brand: Choice, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
{
	Brand::right(pab)
}
