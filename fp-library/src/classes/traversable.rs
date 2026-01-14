use super::{applicative::Applicative, foldable::Foldable, functor::Functor};
use crate::{Apply, functions::identity, hkt::Kind_L1_T1_B0l0_Ol0};

/// A type class for traversable functors.
///
/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
pub trait Traversable: Functor + Foldable {
	/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
	///
	/// The default implementation is defined in terms of [`sequence`] and [`Functor::map`].
	///
	/// **Note**: This default implementation may be less efficient than a direct implementation because it performs two passes:
	/// first mapping the function to create an intermediate structure of computations, and then sequencing that structure.
	/// A direct implementation can often perform the traversal in a single pass without allocating an intermediate container.
	/// Types should provide their own implementation if possible.
	///
	/// # Type Signature
	///
	/// `forall a b f. (Traversable t, Applicative f) => (a -> f b, t a) -> f (t b)`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The traversable structure.
	///
	/// # Returns
	///
	/// The traversable structure wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::Traversable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::traverse::<OptionBrand, _, _, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
		f: Func,
		ta: Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
	) -> Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B))))
	where
		Func: Fn(A) -> Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (B)) + 'a,
		Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (B)): Clone,
		Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (B)): Clone,
	{
		Self::sequence::<F, B>(Self::map(f, ta))
	}

	/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
	///
	/// The default implementation is defined in terms of [`traverse`] and [`identity`].
	///
	/// # Type Signature
	///
	/// `forall a f. (Traversable t, Applicative f) => (t (f a)) -> f (t a)`
	///
	/// # Parameters
	///
	/// * `ta`: The traversable structure containing values in an applicative context.
	///
	/// # Returns
	///
	/// The traversable structure wrapped in the applicative context.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::traversable::Traversable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(Some(5));
	/// let y = OptionBrand::sequence::<OptionBrand, _>(x);
	/// assert_eq!(y, Some(Some(5)));
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (A))))
	) -> Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (A))))
	where
		Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (A)): Clone,
		Apply!(Self, Kind_L1_T1_B0l0_Ol0, ('a), (A)): Clone,
	{
		Self::traverse::<F, Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (A)), A, _>(identity, ta)
	}
}

/// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::traverse`].
///
/// # Type Signature
///
/// `forall a b f. (Traversable t, Applicative f) => (a -> f b, t a) -> f (t b)`
///
/// # Parameters
///
/// * `f`: The function to apply to each element, returning a value in an applicative context.
/// * `ta`: The traversable structure.
///
/// # Returns
///
/// The traversable structure wrapped in the applicative context.
///
/// # Examples
///
/// ```
/// use fp_library::classes::traversable::traverse;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = traverse::<OptionBrand, OptionBrand, _, _, _>(|a| Some(a * 2), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn traverse<'a, Brand: Traversable, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func>(
	f: Func,
	ta: Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (A)),
) -> Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (B))))
where
	Func: Fn(A) -> Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (B)) + 'a,
	Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (B)): Clone,
	Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (B)): Clone,
{
	Brand::traverse::<F, A, B, Func>(f, ta)
}

/// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
///
/// Free function version that dispatches to [the type class' associated function][`Traversable::sequence`].
///
/// # Type Signature
///
/// `forall a f. (Traversable t, Applicative f) => (t (f a)) -> f (t a)`
///
/// # Parameters
///
/// * `ta`: The traversable structure containing values in an applicative context.
///
/// # Returns
///
/// The traversable structure wrapped in the applicative context.
///
/// # Examples
///
/// ```
/// use fp_library::classes::traversable::sequence;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(Some(5));
/// let y = sequence::<OptionBrand, OptionBrand, _>(x);
/// assert_eq!(y, Some(Some(5)));
/// ```
pub fn sequence<'a, Brand: Traversable, F: Applicative, A: 'a + Clone>(
	ta: Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (A))))
) -> Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (A))))
where
	Apply!(F, Kind_L1_T1_B0l0_Ol0, ('a), (A)): Clone,
	Apply!(Brand, Kind_L1_T1_B0l0_Ol0, ('a), (A)): Clone,
{
	Brand::sequence::<F, A>(ta)
}
