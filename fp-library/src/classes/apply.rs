use crate::{
	classes::{ClonableFn, clonable_fn::ApplyFn},
	hkt::{Apply0L1T, Kind0L1T},
};

/// A type class for types that support function application within a context.
///
/// `Apply` provides the ability to apply functions that are themselves
/// wrapped in a context to values that are also wrapped in a context.
/// This allows for sequencing computations where both the function and
/// the value are in a context.
///
/// # Laws
///
/// Apply instances must satisfy the following law:
/// * Composition: `apply(apply(f)(g))(x) = apply(f)(apply(g)(x))`.
pub trait Apply: Kind0L1T {
	/// Applies a function within a context to a value within a context.
	///
	/// # Type Signature
	///
	/// `forall a b. Apply f => f (a -> b) -> f a -> f b`
	///
	/// # Parameters
	///
	/// * `ff`: A function wrapped in the context.
	/// * `fa`: A value wrapped in the context.
	///
	/// # Returns
	///
	/// The result of applying the function to the value, all within the context.
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>>;
}

/// Applies a function within a context to a value within a context.
///
/// Free function version that dispatches to [the type class' associated function][`Apply::apply`].
///
/// # Type Signature
///
/// `forall a b. Apply f => f (a -> b) -> f a -> f b`
///
/// # Parameters
///
/// * `ff`: A function wrapped in the context.
/// * `fa`: A value wrapped in the context.
///
/// # Returns
///
/// The result of applying the function to the value, all within the context.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::apply};
/// use std::rc::Rc;
///
/// assert_eq!(
///     apply::<RcFnBrand, OptionBrand, _, _>(Some(Rc::new(|x: i32| x * 2)))(Some(5)),
///     Some(10)
/// );
/// ```
pub fn apply<'a, ClonableFnBrand: 'a + ClonableFn, Brand: Apply, A: 'a + Clone, B: 'a>(
	ff: Apply0L1T<Brand, ApplyFn<'a, ClonableFnBrand, A, B>>
) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Brand, A>, Apply0L1T<Brand, B>> {
	Brand::apply::<ClonableFnBrand, _, _>(ff)
}
