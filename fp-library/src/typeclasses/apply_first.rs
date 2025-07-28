use crate::hkt::{Apply, Kind};

/// A typeclass for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
/// This is useful for executing side effects in sequence while preserving the
/// primary result.
pub trait ApplyFirst {
	/// Combines two contexts, keeping the value from the first context.
	///
	/// # Type Signature
	///
	/// `forall f a b. ApplyFirst f => f a -> f b -> f a`
	///
	/// # Parameters
	///
	/// * `fa`: The first context containing a value.
	/// * `fb`: The second context containing a value (will be discarded).
	///
	/// # Returns
	///
	/// The first context with its value preserved.
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		A: Clone,
		B: Clone;
}

/// Combines two contexts, keeping the value from the first context.
///
/// Free function version that dispatches to [the typeclass method][`ApplyFirst::apply_first`].
///
/// # Type Signature
///
/// `forall f a b. ApplyFirst f => f a -> f b -> f a`
///
/// # Examples
///
/// ```
/// use fp_library::{brands::OptionBrand, functions::apply_first};
///
/// assert_eq!(apply_first::<OptionBrand, _, _>(Some(5))(Some("hello")), Some(5));
/// ```
pub fn apply_first<Brand, A, B>(
	fa: Apply<Brand, (A,)>
) -> impl Fn(Apply<Brand, (B,)>) -> Apply<Brand, (A,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + ApplyFirst,
	Apply<Brand, (A,)>: Clone,
	A: Clone,
	B: Clone,
{
	Brand::apply_first::<A, B>(fa)
}
