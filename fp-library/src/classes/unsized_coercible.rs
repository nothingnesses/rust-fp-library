//! Unsized coercible pointer trait.

use super::ref_counted_pointer::RefCountedPointer;

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
	/// use fp_library::{brands::*, classes::unsized_coercible::*, functions::*};
	///
	/// let f = RcBrand::coerce_fn(|x: i32| x + 1);
	/// assert_eq!(f(1), 2);
	/// ```
	fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::CloneableOf<dyn 'a + Fn(A) -> B>;
}
