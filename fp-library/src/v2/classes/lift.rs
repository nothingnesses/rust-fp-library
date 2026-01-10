use crate::hkt::{Apply1L1T, Kind1L1T};

/// A type class for types that can be lifted.
///
/// `Lift` allows binary functions to be lifted into the context.
pub trait Lift: Kind1L1T {
    /// Lifts a binary function into the context.
    ///
    /// # Type Signature
    ///
    /// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
    ///
    /// # Parameters
    ///
    /// * `f`: The binary function to apply.
    /// * `fa`: The first context.
    /// * `fb`: The second context.
    ///
    /// # Returns
    ///
    /// A new context containing the result of applying the function.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::lift::Lift;
    /// use fp_library::brands::OptionBrand;
    ///
    /// let x = Some(1);
    /// let y = Some(2);
    /// let z = OptionBrand::lift2(|a, b| a + b, x, y);
    /// assert_eq!(z, Some(3));
    /// ```
    fn lift2<'a, A: 'a, B: 'a, C: 'a, F: 'a>(
        f: F,
        fa: Apply1L1T<'a, Self, A>,
        fb: Apply1L1T<'a, Self, B>
    ) -> Apply1L1T<'a, Self, C>
    where
        F: Fn(A, B) -> C,
        A: Clone,
        B: Clone;
}

/// Lifts a binary function into the context.
///
/// Free function version that dispatches to [the type class' associated function][`Lift::lift2`].
///
/// # Type Signature
///
/// `forall a b c. Lift f => ((a, b) -> c, f a, f b) -> f c`
///
/// # Parameters
///
/// * `f`: The binary function to apply.
/// * `fa`: The first context.
/// * `fb`: The second context.
///
/// # Returns
///
/// A new context containing the result of applying the function.
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::lift::lift2;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(1);
/// let y = Some(2);
/// let z = lift2::<OptionBrand, _, _, _, _>(|a, b| a + b, x, y);
/// assert_eq!(z, Some(3));
/// ```
pub fn lift2<'a, Brand: Lift, A: 'a, B: 'a, C: 'a, F: 'a>(
    f: F,
    fa: Apply1L1T<'a, Brand, A>,
    fb: Apply1L1T<'a, Brand, B>
) -> Apply1L1T<'a, Brand, C>
where
    F: Fn(A, B) -> C,
    A: Clone,
    B: Clone
{
    Brand::lift2(f, fa, fb)
}
