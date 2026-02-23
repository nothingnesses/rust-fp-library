//! Types that can be mapped over two type arguments simultaneously.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Result::<i32, i32>::Ok(5);
//! let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
//! assert_eq!(y, Ok(10));
//! ```

use {
	crate::{Apply, kinds::*},
	fp_macros::{document_parameters, document_signature, document_type_parameters, impl_kind},
};

/// A type class for types that can be mapped over two type arguments.
///
/// A `Bifunctor` represents a context or container with two type parameters,
/// allowing functions to be applied to values of both types.
///
/// ### Hierarchy Unification
///
/// This trait now inherits from [`Kind_266801a817966495`], ensuring that all bifunctor
/// contexts satisfy the strict lifetime requirements where both type arguments must
/// outlive the context's application lifetime.
///
/// By explicitly requiring that both type parameters outlive the application lifetime `'a`,
/// we provide the compiler with the necessary guarantees to handle trait objects
/// (like `dyn Fn`) commonly used in bifunctor implementations. This resolves potential
/// E0310 errors where the compiler cannot otherwise prove that captured variables in
/// closures satisfy the required lifetime bounds.
///
/// ### Laws
///
/// `Bifunctor` instances must satisfy the following laws:
/// * Identity: `bimap(identity, identity, p) = p`.
/// * Composition: `bimap(compose(f, g), compose(h, i), p) = bimap(f, h, bimap(g, i, p))`.
pub trait Bifunctor: Kind_266801a817966495 {
	/// Maps functions over the values in the bifunctor context.
	///
	/// This method applies two functions to the values inside the bifunctor context, producing a new bifunctor context with the transformed values.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The type of the first value.",
		"The type of the first result.",
		"The type of the second value.",
		"The type of the second result.",
		"The type of the first function.",
		"The type of the second function."
	)]
	///
	#[document_parameters(
		"The function to apply to the first value.",
		"The function to apply to the second value.",
		"The bifunctor instance."
	)]
	///
	/// ### Returns
	///
	/// A new bifunctor instance containing the results of applying the functions.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Result::<i32, i32>::Ok(5);
	/// let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
	/// assert_eq!(y, Ok(10));
	/// ```
	fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
		f: F,
		g: G,
		p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
	where
		F: Fn(A) -> B + 'a,
		G: Fn(C) -> D + 'a;
}

/// Maps functions over the values in the bifunctor context.
///
/// Free function version that dispatches to [the type class' associated function][`Bifunctor::bimap`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the bifunctor.",
	"The type of the first value.",
	"The type of the first result.",
	"The type of the second value.",
	"The type of the second result.",
	"The type of the first function.",
	"The type of the second function."
)]
///
#[document_parameters(
	"The function to apply to the first value.",
	"The function to apply to the second value.",
	"The bifunctor instance."
)]
///
/// ### Returns
///
/// A new bifunctor instance containing the results of applying the functions.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Result::<i32, i32>::Ok(5);
/// let y = bimap::<ResultBrand, _, _, _, _, _, _>(|e| e + 1, |s| s * 2, x);
/// assert_eq!(y, Ok(10));
/// ```
pub fn bimap<'a, Brand: Bifunctor, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
	f: F,
	g: G,
	p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
where
	F: Fn(A) -> B + 'a,
	G: Fn(C) -> D + 'a,
{
	Brand::bimap::<A, B, C, D, F, G>(f, g, p)
}

use {crate::classes::Functor, core::marker::PhantomData};

/// An adapter that partially applies a `Bifunctor` to its first argument, creating a `Functor`.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	classes::{
/// 		bifunctor::BifunctorFixedFirst,
/// 		functor::map,
/// 	},
/// };
///
/// let x = Result::<i32, i32>::Ok(5);
/// let y = map::<BifunctorFixedFirst<ResultBrand, i32>, _, _, _>(|s| s * 2, x);
/// assert_eq!(y, Ok(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BifunctorFixedFirst<Brand, A>(PhantomData<(Brand, A)>);

impl_kind! {
	impl<Brand: Bifunctor, A: 'static> for BifunctorFixedFirst<Brand, A> {
		type Of<'a, B: 'a>: 'a = Apply!(<Brand as Kind!(type Of<'a, T: 'a, U: 'a>: 'a;)>::Of<'a, A, B>);
	}
}

impl<Brand: Bifunctor, A: 'static> Functor for BifunctorFixedFirst<Brand, A> {
	fn map<'a, B: 'a, C: 'a, Func>(
		f: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		Func: Fn(B) -> C + 'a,
	{
		Brand::bimap(crate::functions::identity, f, fa)
	}
}
