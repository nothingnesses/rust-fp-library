//! Strong profunctors, which can lift profunctors through product types.
//!
//! A strong profunctor allows lifting a profunctor `P A B` to `P (A, C) (B, C)`,
//! preserving the extra context `C`. This is the key constraint for lenses.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::profunctor::*,
//! 	functions::*,
//! };
//!
//! // Functions are strong profunctors
//! let f = |x: i32| x + 1;
//! let g = first::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
//! assert_eq!(g((10, 20)), (11, 20));
//! ```

use {
	crate::{
		Apply,
		classes::profunctor::Profunctor,
		kinds::*,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
};

/// A type class for strong profunctors.
///
/// A strong profunctor can lift a profunctor through product types (tuples).
/// This is the profunctor constraint that characterizes lenses.
///
/// ### Hierarchy Unification
///
/// This trait uses the strict Kind signature from [`Kind_266801a817966495`]. This ensures
/// that when lifting a profunctor, the secondary component of the product type (the context)
/// correctly satisfies lifetime requirements relative to the profunctor's application.
///
/// ### Laws
///
/// `Strong` instances must satisfy the following laws:
/// * Identity: `first(identity) = identity`.
/// * Composition: `first(p ∘ q) = first(p) ∘ first(q)`.
/// * Naturality: `dimap(fst, fst) ∘ first(p) = first(p) ∘ dimap(fst, fst)`.
pub trait Strong: Profunctor {
	/// Lift a profunctor to operate on the first component of a pair.
	///
	/// This method takes a profunctor `P A B` and returns `P (A, C) (B, C)`,
	/// threading the extra context `C` through unchanged.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the profunctor.",
		"The output type of the profunctor.",
		"The type of the second component (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to lift.")]
	///
	/// ### Returns
	///
	/// A new profunctor that operates on pairs.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = first::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g((10, 20)), (11, 20));
	/// ```
	fn first<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>);

	/// Lift a profunctor to operate on the second component of a pair.
	///
	/// This method takes a profunctor `P A B` and returns `P (C, A) (C, B)`,
	/// threading the extra context `C` through unchanged in the first position.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type of the profunctor.",
		"The output type of the profunctor.",
		"The type of the first component (threaded through unchanged)."
	)]
	///
	#[document_parameters("The profunctor instance to lift.")]
	///
	/// ### Returns
	///
	/// A new profunctor that operates on pairs.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::profunctor::*,
	/// 	functions::*,
	/// };
	///
	/// let f = |x: i32| x + 1;
	/// let g = second::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g((20, 10)), (20, 11));
	/// ```
	fn second<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
		Self::dimap(|(c, a)| (a, c), |(b, c)| (c, b), Self::first(pab))
	}
}

/// Lift a profunctor to operate on the first component of a pair.
///
/// Free function version that dispatches to [the type class' associated function][`Strong::first`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the strong profunctor.",
	"The input type of the profunctor.",
	"The output type of the profunctor.",
	"The type of the second component (threaded through unchanged)."
)]
///
#[document_parameters("The profunctor instance to lift.")]
///
/// ### Returns
///
/// A new profunctor that operates on pairs.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::profunctor::*,
/// 	functions::*,
/// };
///
/// let f = |x: i32| x + 1;
/// let g = first::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g((10, 20)), (11, 20));
/// ```
pub fn first<'a, Brand: Strong, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
	Brand::first(pab)
}

/// Lift a profunctor to operate on the second component of a pair.
///
/// Free function version that dispatches to [the type class' associated function][`Strong::second`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the strong profunctor.",
	"The input type of the profunctor.",
	"The output type of the profunctor.",
	"The type of the first component (threaded through unchanged)."
)]
///
#[document_parameters("The profunctor instance to lift.")]
///
/// ### Returns
///
/// A new profunctor that operates on pairs.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::profunctor::*,
/// 	functions::*,
/// };
///
/// let f = |x: i32| x + 1;
/// let g = second::<RcFnBrand, _, _, i32>(std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g((20, 10)), (20, 11));
/// ```
pub fn second<'a, Brand: Strong, A: 'a, B: 'a, C: 'a>(
	pab: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (C, A), (C, B)>) {
	Brand::second(pab)
}
