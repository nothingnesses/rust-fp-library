//! Profunctors, which are functors contravariant in the first argument and covariant in the second.
//!
//! A profunctor represents a morphism between two categories, mapping objects and morphisms from one to the other.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! // Function is a profunctor
//! let f = |x: i32| x + 1;
//! let g = dimap::<RcFnBrand, _, _, _, _, _, _>(|x: i32| x * 2, |x: i32| x - 1, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
//! assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
//! ```

use crate::{Apply, kinds::*};
use fp_macros::document_parameters;
use fp_macros::document_signature;
use fp_macros::document_type_parameters;

/// A type class for profunctors.
///
/// A profunctor is a type constructor that is contravariant in its first type parameter
/// and covariant in its second type parameter. This means it can pre-compose with a
/// function on the input and post-compose with a function on the output.
///
/// ### Laws
///
/// `Profunctor` instances must satisfy the following laws:
/// * Identity: `dimap(identity, identity, p) = p`.
/// * Composition: `dimap(f1 ∘ f2, g2 ∘ g1, p) = dimap(f1, g1, dimap(f2, g2, p))`.
pub trait Profunctor: Kind_140eb1e35dc7afb3 {
	/// Maps over both arguments of the profunctor.
	///
	/// This method applies a contravariant function to the first argument and a covariant
	/// function to the second argument, transforming the profunctor.
	///
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The new input type (contravariant position).",
		"The original input type.",
		"The original output type.",
		"The new output type (covariant position).",
		"The type of the contravariant function.",
		"The type of the covariant function."
	)]
	///
	#[document_parameters(
		"The contravariant function to apply to the input.",
		"The covariant function to apply to the output.",
		"The profunctor instance."
	)]
	///
	/// ### Returns
	///
	/// A new profunctor instance with transformed input and output types.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = |x: i32| x + 1;
	/// let g = dimap::<RcFnBrand, _, _, _, _, _, _>(|x: i32| x * 2, |x: i32| x - 1, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
	/// ```
	fn dimap<'a, A, B: 'a, C: 'a, D, FuncAB, FuncCD>(
		ab: FuncAB,
		cd: FuncCD,
		pbc: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, D>)
	where
		FuncAB: Fn(A) -> B + 'a,
		FuncCD: Fn(C) -> D + 'a;

	/// Maps contravariantly over the first argument.
	///
	/// This is a convenience method that maps only over the input (contravariant position).
	///
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The new input type.",
		"The original input type.",
		"The output type.",
		"The type of the contravariant function."
	)]
	///
	#[document_parameters(
		"The contravariant function to apply to the input.",
		"The profunctor instance."
	)]
	///
	/// ### Returns
	///
	/// A new profunctor instance with transformed input type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = |x: i32| x + 1;
	/// let g = lmap::<RcFnBrand, _, _, _, _>(|x: i32| x * 2, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g(10), 21); // (10 * 2) + 1 = 21
	/// ```
	fn lmap<'a, A, B: 'a, C: 'a, FuncAB>(
		ab: FuncAB,
		pbc: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, C>)
	where
		FuncAB: Fn(A) -> B + 'a,
	{
		Self::dimap(ab, crate::functions::identity, pbc)
	}

	/// Maps covariantly over the second argument.
	///
	/// This is a convenience method that maps only over the output (covariant position).
	///
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The input type.",
		"The original output type.",
		"The new output type.",
		"The type of the covariant function."
	)]
	///
	#[document_parameters(
		"The covariant function to apply to the output.",
		"The profunctor instance."
	)]
	///
	/// ### Returns
	///
	/// A new profunctor instance with transformed output type.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = |x: i32| x + 1;
	/// let g = rmap::<RcFnBrand, _, _, _, _>(|x: i32| x * 2, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
	/// assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
	/// ```
	fn rmap<'a, A: 'a, B: 'a, C, FuncBC>(
		bc: FuncBC,
		pab: Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T, U>; )>::Of<'a, A, C>)
	where
		FuncBC: Fn(B) -> C + 'a,
	{
		Self::dimap(crate::functions::identity, bc, pab)
	}
}

/// Maps over both arguments of the profunctor.
///
/// Free function version that dispatches to [the type class' associated function][`Profunctor::dimap`].
///
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the profunctor.",
	"The new input type (contravariant position).",
	"The original input type.",
	"The original output type.",
	"The new output type (covariant position).",
	"The type of the contravariant function.",
	"The type of the covariant function."
)]
///
#[document_parameters(
	"The contravariant function to apply to the input.",
	"The covariant function to apply to the output.",
	"The profunctor instance."
)]
///
/// ### Returns
///
/// A new profunctor instance with transformed input and output types.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let f = |x: i32| x + 1;
/// let g = dimap::<RcFnBrand, _, _, _, _, _, _>(|x: i32| x * 2, |x: i32| x - 1, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g(10), 20); // (10 * 2) + 1 - 1 = 20
/// ```
pub fn dimap<'a, Brand: Profunctor, A, B: 'a, C: 'a, D, FuncAB, FuncCD>(
	ab: FuncAB,
	cd: FuncCD,
	pbc: Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
) -> Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, A, D>)
where
	FuncAB: Fn(A) -> B + 'a,
	FuncCD: Fn(C) -> D + 'a,
{
	Brand::dimap(ab, cd, pbc)
}

/// Maps contravariantly over the first argument.
///
/// Free function version that dispatches to [the type class' associated function][`Profunctor::lmap`].
///
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the profunctor.",
	"The new input type.",
	"The original input type.",
	"The output type.",
	"The type of the contravariant function."
)]
///
#[document_parameters(
	"The contravariant function to apply to the input.",
	"The profunctor instance."
)]
///
/// ### Returns
///
/// A new profunctor instance with transformed input type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let f = |x: i32| x + 1;
/// let g = lmap::<RcFnBrand, _, _, _, _>(|x: i32| x * 2, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g(10), 21); // (10 * 2) + 1 = 21
/// ```
pub fn lmap<'a, Brand: Profunctor, A, B: 'a, C: 'a, FuncAB>(
	ab: FuncAB,
	pbc: Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, B, C>),
) -> Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, A, C>)
where
	FuncAB: Fn(A) -> B + 'a,
{
	Brand::lmap(ab, pbc)
}

/// Maps covariantly over the second argument.
///
/// Free function version that dispatches to [the type class' associated function][`Profunctor::rmap`].
///
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the profunctor.",
	"The input type.",
	"The original output type.",
	"The new output type.",
	"The type of the covariant function."
)]
///
#[document_parameters("The covariant function to apply to the output.", "The profunctor instance.")]
///
/// ### Returns
///
/// A new profunctor instance with transformed output type.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let f = |x: i32| x + 1;
/// let g = rmap::<RcFnBrand, _, _, _, _>(|x: i32| x * 2, std::rc::Rc::new(f) as std::rc::Rc<dyn Fn(i32) -> i32>);
/// assert_eq!(g(10), 22); // (10 + 1) * 2 = 22
/// ```
pub fn rmap<'a, Brand: Profunctor, A: 'a, B: 'a, C, FuncBC>(
	bc: FuncBC,
	pab: Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, A, B>),
) -> Apply!(<Brand as Kind!( type Of<'a, T, U>; )>::Of<'a, A, C>)
where
	FuncBC: Fn(B) -> C + 'a,
{
	Brand::rmap(bc, pab)
}
