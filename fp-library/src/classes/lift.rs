//! A type class for lifting binary functions into a context.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(1);
//! let y = Some(2);
//! let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
//! assert_eq!(z, Some(3));
//! ```

use crate::{Apply, kinds::*};

/// A type class for lifting binary functions into a context.
pub trait Lift: Kind_cdc7cd43dac7585f {
	/// Lifts a binary function into the context.
	///
	/// This method lifts a binary function to operate on values within the context.
	///
	/// ### Type Signature
	///
	/// `forall f a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the first value.
	/// * `B`: The type of the second value.
	/// * `C`: The type of the result.
	/// * `Func`: The type of the binary function.
	///
	/// ### Parameters
	///
	/// * `func`: The binary function to apply.
	/// * `fa`: The first context.
	/// * `fb`: The second context.
	///
	/// ### Returns
	///
	/// A new context containing the result of applying the function.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(1);
	/// let y = Some(2);
	/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	/// ```
	fn lift2<'a, A, B, C, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		Func: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a;
}

/// Lifts a binary function into the context.
///
/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
///
/// ### Type Signature
///
/// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the context.
/// * `A`: The type of the first value.
/// * `B`: The type of the second value.
/// * `C`: The type of the result.
/// * `Func`: The type of the binary function.
///
/// ### Parameters
///
/// * `func`: The binary function to apply.
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// ### Returns
///
/// A new context containing the result of applying the function.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(1);
/// let y = Some(2);
/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
/// assert_eq!(z, Some(3));
/// ```
pub fn lift2<'a, Brand: Lift, A, B, C, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	fb: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
where
	Func: Fn(A, B) -> C + 'a,
	A: Clone + 'a,
	B: Clone + 'a,
	C: 'a,
{
	Brand::lift2::<A, B, C, Func>(func, fa, fb)
}
