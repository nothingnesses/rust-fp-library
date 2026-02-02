//! Applying functions within a context to values within a context, without an identity element.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, classes::*, functions::*};
//!
//! let f = Some(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
//! let x = Some(5);
//! let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
//! assert_eq!(y, Some(10));
//! ```

use super::{cloneable_fn::CloneableFn, functor::Functor, lift::Lift};
use crate::{Apply, kinds::*};
use fp_macros::doc_params;
use fp_macros::doc_type_params;
use fp_macros::hm_signature;

/// A type class for applying functions within a context to values within a context.
///
/// `Semiapplicative` provides the ability to apply functions that are themselves
/// wrapped in a context to values that are also wrapped in a context.
///
/// ### Laws
///
/// `Semiapplicative` instances must satisfy the following law:
/// * Composition: `apply(apply(f, g), x) = apply(f, apply(g, x))`.
pub trait Semiapplicative: Lift + Functor {
	/// Applies a function within a context to a value within a context.
	///
	/// This method applies a function wrapped in a context to a value wrapped in a context.
	///
	/// **Important**: This operation requires type erasure for heterogeneous functions.
	/// When a container (like `Vec`) holds multiple different closures, they must be
	/// type-erased via `Rc<dyn Fn>` or `Arc<dyn Fn>` because each Rust closure is a
	/// distinct anonymous type.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The lifetime of the values.",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]
	///
	/// ### Parameters
	///
	#[doc_params("The context containing the function(s).", "The context containing the value(s).")]
	///
	/// ### Returns
	///
	/// A new context containing the result(s) of applying the function(s) to the value(s).
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*};
	///
	/// let f = Some(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// let x = Some(5);
	/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
}

/// Applies a function within a context to a value within a context.
///
/// Free function version that dispatches to [the type class' associated function][`Semiapplicative::apply`].
///
/// ### Type Signature
///
#[hm_signature(Semiapplicative)]
///
/// ### Type Parameters
///
#[doc_type_params(
	"The lifetime of the values.",
	"The brand of the cloneable function wrapper.",
	"The brand of the context.",
	"The type of the input value.",
	"The type of the output value."
)]
///
/// ### Parameters
///
#[doc_params("The context containing the function(s).", "The context containing the value(s).")]
///
/// ### Returns
///
/// A new context containing the result(s) of applying the function(s) to the value(s).
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::*, functions::*};
///
/// let f = Some(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
/// let x = Some(5);
/// let y = apply::<RcFnBrand, OptionBrand, _, _>(f, x);
/// assert_eq!(y, Some(10));
/// ```
pub fn apply<'a, FnBrand: 'a + CloneableFn, Brand: Semiapplicative, A: 'a + Clone, B: 'a>(
	ff: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
	Brand::apply::<FnBrand, A, B>(ff, fa)
}
