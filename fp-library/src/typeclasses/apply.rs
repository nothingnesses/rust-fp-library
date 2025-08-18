use crate::{
	aliases::ArcFn,
	hkt::{Apply1, Kind1},
};

/// A typeclass for types that support function application within a context.
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
pub trait Apply: Kind1 {
	/// Applies a function within a context to a value within a context.
	///
	/// # Type Signature
	///
	/// `forall f a b. Apply f => f (a -> b) -> f a -> f b`
	///
	/// # Parameters
	///
	/// * `ff`: A function wrapped in the context.
	/// * `fa`: A value wrapped in the context.
	///
	/// # Returns
	///
	/// The result of applying the function to the value, all within the context.
	fn apply<'a, F: 'a + Fn(A) -> B, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, F>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, F>: Clone,;
}

/// Applies a function within a context to a value within a context.
///
/// Free function version that dispatches to [the typeclass' associated function][`Apply::apply`].
///
/// # Type Signature
///
/// `forall f a b. Apply f => f (a -> b) -> f a -> f b`
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
/// use fp_library::{brands::OptionBrand, functions::apply};
///
/// assert_eq!(
///     apply::<OptionBrand, _, _, _>(Some(|x: i32| x * 2))(Some(5)),
///     Some(10)
/// );
/// ```
pub fn apply<'a, Brand: Apply, F: 'a + Fn(A) -> B, A: 'a + Clone, B: 'a>(
	ff: Apply1<Brand, F>
) -> ArcFn<'a, Apply1<Brand, A>, Apply1<Brand, B>>
where
	Apply1<Brand, F>: Clone,
{
	Brand::apply::<F, _, _>(ff)
}
