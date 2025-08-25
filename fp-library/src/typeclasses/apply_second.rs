use crate::{
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{ClonableFn, clonable_fn::ApplyFn},
};

/// A typeclass for types that support combining two contexts, keeping the second value.
///
/// `ApplySecond` provides the ability to sequence two computations but discard
/// the result of the first computation, keeping only the result of the second.
/// This is useful for executing side effects in sequence while preserving the
/// final result.
pub trait ApplySecond: Kind0L1T {
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
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>>;
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
/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::apply_second};
///
/// assert_eq!(apply_second::<RcFnBrand, OptionBrand, _, _>(Some(5))(Some("hello")), Some("hello"));
/// ```
pub fn apply_second<
	'a,
	ClonableFnBrand: 'a + ClonableFn,
	Brand: ApplySecond,
	A: 'a + Clone,
	B: 'a + Clone,
>(
	fa: Apply0L1T<Brand, A>
) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, B>, Apply0L1T<Brand, B>> {
	Brand::apply_second::<ClonableFnBrand, A, B>(fa)
}
