use crate::{
	aliases::ArcFn,
	hkt::{Apply1, Kind1},
};

/// A typeclass for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
/// This is useful for executing side effects in sequence while preserving the
/// primary result.
pub trait ApplyFirst: Kind1 {
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
	fn apply_first<'a, A: 'a + Clone, B>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, A>>
	where
		Apply1<Self, A>: Clone;
}

/// Combines two contexts, keeping the value from the first context.
///
/// Free function version that dispatches to [the typeclass' associated function][`ApplyFirst::apply_first`].
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
///
/// # Examples
///
/// ```
/// use fp_library::{brands::OptionBrand, functions::apply_first};
///
/// assert_eq!(apply_first::<OptionBrand, _, _>(Some(5))(Some("hello")), Some(5));
/// ```
pub fn apply_first<'a, Brand: ApplyFirst, A: 'a + Clone, B>(
	fa: Apply1<Brand, A>
) -> ArcFn<'a, Apply1<Brand, B>, Apply1<Brand, A>>
where
	Apply1<Brand, A>: Clone,
{
	Brand::apply_first::<A, B>(fa)
}
