use crate::hkt::{Apply, Kind};

/// A typeclass for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
/// This is useful for executing side effects in sequence while preserving the
/// final result.
pub trait ApplySecond {
	/// Combines two contexts, keeping the value from the second context.
	///
	/// # Type Signature
	///
	/// `forall f a b. ApplySecond f => f a -> f b -> f b`
	///
	/// # Parameters
	///
	/// * `fa`: The first context containing a value (will be discarded).
	/// * `fb`: The second context containing a value.
	///
	/// # Returns
	///
	/// The second context with its value preserved.
	fn apply_second<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		B: Clone;
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the typeclass method][`ApplySecond::apply_second`].
///
/// # Type Signature
///
/// `forall f a b. ApplySecond f => f a -> f b -> f b`
///
/// # Examples
///
/// ```
/// use fp_library::{brands::OptionBrand, functions::apply_second};
///
/// assert_eq!(apply_second::<OptionBrand, _, _>(Some(5))(Some("hello")), Some("hello"));
/// ```
pub fn apply_second<Brand, A, B>(
	fa: Apply<Brand, (A,)>
) -> impl Fn(Apply<Brand, (B,)>) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + ApplySecond,
	Apply<Brand, (A,)>: Clone,
	B: Clone,
{
	Brand::apply_second::<A, B>(fa)
}
