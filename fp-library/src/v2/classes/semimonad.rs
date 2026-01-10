use crate::hkt::{Apply1L1T, Kind1L1T};

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// If `x` has type `m a` and `f` has type `a -> m b`, then `bind(x, f)` has type `m b`,
/// representing the result of executing `x` to get a value of type `a` and then
/// passing it to `f` to get a computation of type `m b`.
pub trait Semimonad: Kind1L1T {
    /// Sequences two computations, allowing the second to depend on the value computed by the first.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Semimonad m => (m a, a -> m b) -> m b`
    ///
    /// # Parameters
    ///
    /// * `ma`: The first computation.
    /// * `f`: The function to apply to the result of the first computation.
    ///
    /// # Returns
    ///
    /// The result of the second computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::semimonad::Semimonad;
    /// use fp_library::brands::OptionBrand;
    ///
    /// let x = Some(5);
    /// let y = OptionBrand::bind(x, |i| Some(i * 2));
    /// assert_eq!(y, Some(10));
    /// ```
    fn bind<'a, A: 'a, B: 'a, F: 'a>(
        ma: Apply1L1T<'a, Self, A>,
        f: F
    ) -> Apply1L1T<'a, Self, B>
    where
        F: Fn(A) -> Apply1L1T<'a, Self, B>;
}

/// Sequences two computations, allowing the second to depend on the value computed by the first.
///
/// Free function version that dispatches to [the type class' associated function][`Semimonad::bind`].
///
/// # Type Signature
///
/// `forall a b. Semimonad m => (m a, a -> m b) -> m b`
///
/// # Parameters
///
/// * `ma`: The first computation.
/// * `f`: The function to apply to the result of the first computation.
///
/// # Returns
///
/// The result of the second computation.
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::semimonad::bind;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = bind::<OptionBrand, _, _, _>(x, |i| Some(i * 2));
/// assert_eq!(y, Some(10));
/// ```
pub fn bind<'a, Brand: Semimonad, A: 'a, B: 'a, F: 'a>(
    ma: Apply1L1T<'a, Brand, A>,
    f: F
) -> Apply1L1T<'a, Brand, B>
where
    F: Fn(A) -> Apply1L1T<'a, Brand, B>
{
    Brand::bind(ma, f)
}
