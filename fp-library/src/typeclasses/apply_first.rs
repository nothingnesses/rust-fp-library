use crate::{
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{ClonableFn, clonable_fn::ApplyFn},
};

/// A typeclass for types that support combining two contexts, keeping the first value.
///
/// `ApplyFirst` provides the ability to sequence two computations but discard
/// the result of the second computation, keeping only the result of the first.
/// This is useful for executing side effects in sequence while preserving the
/// primary result.
pub trait ApplyFirst: Kind0L1T {
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
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>>;
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
/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::apply_first};
///
/// assert_eq!(apply_first::<RcFnBrand, OptionBrand, _, _>(Some(5))(Some("hello")), Some(5));
/// ```
pub fn apply_first<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: ApplyFirst,
	A: 'a + Clone,
	B: Clone,
>(
	fa: Apply0L1T<Brand, A>
) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, B>, Apply0L1T<Brand, A>> {
	Brand::apply_first::<ClonableFnBrand, A, B>(fa)
}
