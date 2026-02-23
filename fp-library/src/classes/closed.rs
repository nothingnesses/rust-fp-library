//! Profunctors that can be closed under exponentiation.
//!
//! A `Closed` profunctor can lift a profunctor to operate on functions.
//! This is the profunctor constraint that characterizes grates.

use {
	crate::{Apply, classes::profunctor::Profunctor, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for closed profunctors.
///
/// A `Closed` profunctor can be closed under exponentiation.
///
/// ### Hierarchy Unification
///
/// This trait inherits from [`Profunctor`].
pub trait Closed: Profunctor {
	/// Lift a profunctor to operate on functions.
	///
	/// This method takes a profunctor `P A B` and returns `P (X -> A) (X -> B)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the profunctor.",
		"The input type of the functions.",
		"The source type of the profunctor.",
		"The target type of the profunctor."
	)]
	///
	#[document_parameters("The profunctor value to transform.")]
	///
	/// ### Returns
	///
	/// A new profunctor that operates on functions.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::Closed,
	/// };
	///
	/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
	/// let g = <RcFnBrand as Closed>::closed::<String, i32, i32>(f);
	/// // g is now a function: (String -> i32) -> (String -> i32)
	/// let h = Box::new(|s: String| s.len() as i32) as Box<dyn Fn(String) -> i32>;
	/// let result = g(h);
	/// assert_eq!(result("hi".to_string()), 3); // len("hi") + 1 = 3
	/// ```
	fn closed<'a, X: 'a, A: 'a, B: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Box<dyn Fn(X) -> A + 'a>, Box<dyn Fn(X) -> B + 'a>>);
}

/// Lift a profunctor to operate on functions.
///
/// Free function version that dispatches to [the type class' associated function][`Closed::closed`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the profunctor.",
	"The brand of the closed profunctor.",
	"The input type of the functions.",
	"The source type of the profunctor.",
	"The target type of the profunctor."
)]
///
#[document_parameters("The profunctor value to transform.")]
///
/// ### Returns
///
/// A new profunctor that operates on functions.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::Closed,
/// };
///
/// let f = std::rc::Rc::new(|x: i32| x + 1) as std::rc::Rc<dyn Fn(i32) -> i32>;
/// let g = fp_library::functions::closed::<RcFnBrand, String, i32, i32>(f);
/// // g is now a function: (String -> i32) -> (String -> i32)
/// let h = Box::new(|s: String| s.len() as i32) as Box<dyn Fn(String) -> i32>;
/// let result = g(h);
/// assert_eq!(result("hi".to_string()), 3); // len("hi") + 1 = 3
/// ```
pub fn closed<'a, Brand: Closed, X: 'a, A: 'a, B: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Box<dyn Fn(X) -> A + 'a>, Box<dyn Fn(X) -> B + 'a>>)
{
	Brand::closed::<X, A, B>(pab)
}
