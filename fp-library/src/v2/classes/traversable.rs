use crate::hkt::Apply1L1T;
use super::{applicative::Applicative, foldable::Foldable, functor::Functor};

/// A type class for traversable functors.
///
/// `Traversable` functors can be traversed, which accumulates results and effects in some [`Applicative`] context.
pub trait Traversable: Functor + Foldable {
    /// Map each element of the [`Traversable`] structure to a computation, evaluate those computations and combine the results into an [`Applicative`] context.
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
    /// use fp_library::v2::classes::traversable::Traversable;
    /// use fp_library::brands::OptionBrand;
    ///
    /// let x = Some(5);
    /// let y = OptionBrand::traverse::<OptionBrand, _, _, _>(|a| Some(a * 2), x);
    /// assert_eq!(y, Some(Some(10)));
    /// ```
    fn traverse<'a, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
        f: Func,
        ta: Apply1L1T<'a, Self, A>
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, B>>
    where
        Func: Fn(A) -> Apply1L1T<'a, F, B>,
        Apply1L1T<'a, Self, B>: Clone;

    /// Evaluate each computation in a [`Traversable`] structure and accumulate the results into an [`Applicative`] context.
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
    /// use fp_library::v2::classes::traversable::Traversable;
    /// use fp_library::brands::OptionBrand;
    ///
    /// let x = Some(Some(5));
    /// let y = OptionBrand::sequence::<OptionBrand, _>(x);
    /// assert_eq!(y, Some(Some(5)));
    /// ```
    fn sequence<'a, F: Applicative, A: 'a + Clone>(
        ta: Apply1L1T<'a, Self, Apply1L1T<'a, F, A>>
    ) -> Apply1L1T<'a, F, Apply1L1T<'a, Self, A>>
    where
        Apply1L1T<'a, F, A>: Clone,
        Apply1L1T<'a, Self, A>: Clone;
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
/// use fp_library::v2::classes::traversable::traverse;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = traverse::<OptionBrand, OptionBrand, _, _, _>(|a| Some(a * 2), x);
/// assert_eq!(y, Some(Some(10)));
/// ```
pub fn traverse<'a, Brand: Traversable, F: Applicative, A: 'a + Clone, B: 'a + Clone, Func: 'a>(
    f: Func,
    ta: Apply1L1T<'a, Brand, A>
) -> Apply1L1T<'a, F, Apply1L1T<'a, Brand, B>>
where
    Func: Fn(A) -> Apply1L1T<'a, F, B>,
    Apply1L1T<'a, Brand, B>: Clone
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
/// use fp_library::v2::classes::traversable::sequence;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(Some(5));
/// let y = sequence::<OptionBrand, OptionBrand, _>(x);
/// assert_eq!(y, Some(Some(5)));
/// ```
pub fn sequence<'a, Brand: Traversable, F: Applicative, A: 'a + Clone>(
    ta: Apply1L1T<'a, Brand, Apply1L1T<'a, F, A>>
) -> Apply1L1T<'a, F, Apply1L1T<'a, Brand, A>>
where
    Apply1L1T<'a, F, A>: Clone,
    Apply1L1T<'a, Brand, A>: Clone
{
    Brand::sequence::<F, A>(ta)
}
