//! Arrows, which represent computations that can be composed and satisfy the laws of category and strength.
//!
//! An arrow is a type constructor that is both a [`Category`] and a [`Strong`] profunctor,
//! with the additional ability to lift pure functions into the arrow context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
//! assert_eq!(f(5), 10);
//! ```

use {
	super::{category::Category, strong::Strong},
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for arrows.
///
/// An arrow is a [`Category`] that is also a [`Strong`] profunctor.
/// It provides the `arrow` method to lift a pure function into the arrow context.
///
/// ### Hierarchy Unification
///
/// By inheriting from both [`Category`] and [`Strong`], this trait is now part of the
/// unified hierarchy based on [`Kind_266801a817966495`]. This ensures that any lifted
/// pure function correctly respects lifetime bounds on its input and output types.
///
/// By explicitly requiring that both type parameters outlive the application lifetime `'a`,
/// we provide the compiler with the necessary guarantees to handle trait objects
/// (like `dyn Fn`) commonly used in arrow implementations. This resolves potential
/// E0310 errors where the compiler cannot otherwise prove that captured variables in
/// closures satisfy the required lifetime bounds.
///
/// ### Laws
///
/// `Arrow` instances must satisfy the following laws (in addition to `Category` and `Strong` laws):
/// * Identity: `arrow(id) = id`.
/// * Composition: `arrow(g ∘ f) = arrow(g) ∘ arrow(f)`.
/// * Naturality: `first(arrow(f)) ∘ arrow(fst) = arrow(fst) ∘ arrow(f)`.
pub trait Arrow: Category + Strong {
	/// Lifts a pure function into the arrow context.
	///
	/// This method takes a closure and returns an arrow instance that represents that function.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the function and its captured data.",
		"The input type of the arrow.",
		"The output type of the arrow."
	)]
	///
	#[document_parameters("The closure to lift into an arrow.")]
	///
	/// ### Returns
	///
	/// The arrow instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// assert_eq!(f(5), 10);
	/// ```
	fn arrow<'a, A, B: 'a>(
		f: impl 'a + Fn(A) -> B
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>);
}

impl<Brand> Arrow for Brand
where
	Brand: Category + Strong,
{
	fn arrow<'a, A, B: 'a>(
		f: impl 'a + Fn(A) -> B
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>) {
		Brand::lmap(f, Brand::identity())
	}
}

/// Lifts a pure function into the arrow context.
///
/// Free function version that dispatches to [the type class' associated function][`Arrow::arrow`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the arrow.",
	"The lifetime of the function and its captured data.",
	"The input type of the arrow.",
	"The output type of the arrow."
)]
///
#[document_parameters("The closure to lift into an arrow.")]
///
/// ### Returns
///
/// The arrow instance.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let f = arrow::<RcFnBrand, _, _>(|x: i32| x * 2);
/// assert_eq!(f(5), 10);
/// ```
pub fn arrow<'a, Brand, A, B: 'a>(
	f: impl 'a + Fn(A) -> B
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
where
	Brand: Arrow,
{
	Brand::arrow(f)
}
