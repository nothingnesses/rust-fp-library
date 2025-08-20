use crate::{
	aliases::ArcFn,
	hkt::{Apply1, Kind1},
};

/// A typeclass for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
/// This is useful for executing side effects in sequence while preserving the
/// final result.
pub trait ApplySecond: Kind1 {
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
	fn apply_second<'a, A: 'a, B: 'a + Clone>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>
	where
		Apply1<Self, A>: Clone;
}

/// Combines two contexts, keeping the value from the second context.
///
/// Free function version that dispatches to [the typeclass' associated function][`ApplySecond::apply_second`].
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
///
/// # Examples
///
/// ```
/// use fp_library::{brands::OptionBrand, functions::apply_second};
///
/// assert_eq!(apply_second::<OptionBrand, _, _>(Some(5))(Some("hello")), Some("hello"));
/// ```
pub fn apply_second<'a, Brand: ApplySecond, A: 'a, B: 'a + Clone>(
	fa: Apply1<Brand, A>
) -> ArcFn<'a, Apply1<Brand, B>, Apply1<Brand, B>>
where
	Apply1<Brand, A>: Clone,
{
	Brand::apply_second::<A, B>(fa)
}
