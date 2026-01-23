//! A trait for pointer brands that can perform unsized coercion to `dyn Fn`.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
//! assert_eq!(f(1), 2);
//! ```

use super::RefCountedPointer;

/// Trait for pointer brands that can perform unsized coercion to `dyn Fn`.
pub trait UnsizedCoercible: RefCountedPointer + 'static {
	/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
	///
	/// ### Type Signature
	///
	/// `forall a b. (a -> b) -> UnsizedCoercible (a -> b)`
	///
	/// ### Type Parameters
	///
	/// * `A`: The input type of the function.
	/// * `B`: The output type of the function.
	///
	/// ### Parameters
	///
	/// * `f`: The closure to coerce.
	///
	/// ### Returns
	///
	/// The closure wrapped in the pointer type as a trait object.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
}

/// Coerces a sized closure to a `dyn Fn` wrapped in this pointer type.
///
/// Free function version that dispatches to [the type class' associated function][`UnsizedCoercible::coerce_fn`].
///
/// ### Type Signature
///
/// `forall a b. (a -> b) -> UnsizedCoercible (a -> b)`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the pointer.
/// * `A`: The input type of the function.
/// * `B`: The output type of the function.
/// * `F`: The type of the closure to coerce.
///
/// ### Parameters
///
/// * `f`: The closure to coerce.
///
/// ### Returns
///
/// The closure wrapped in the pointer type as a trait object.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, classes::unsized_coercible::*, functions::*};
///
/// let f = coerce_fn::<RcBrand, _, _, _>(|x: i32| x + 1);
/// assert_eq!(f(1), 2);
/// ```
pub fn coerce_fn<'a, Brand: UnsizedCoercible, A, B, F>(
	f: F
) -> Brand::CloneableOf<dyn 'a + Fn(A) -> B>
where
	F: 'a + Fn(A) -> B,
{
	Brand::coerce_fn::<A, B>(f)
}
