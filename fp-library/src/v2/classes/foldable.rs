use crate::hkt::{Apply1L1T, Kind1L1T};
use super::monoid::Monoid;

/// A type class for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result.
pub trait Foldable: Kind1L1T {
    /// Folds the structure by applying a function from right to left.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Foldable t => ((a, b) -> b, b, t a) -> b`
    ///
    /// # Parameters
    ///
    /// * `f`: The function to apply to each element and the accumulator.
    /// * `init`: The initial value of the accumulator.
    /// * `fa`: The structure to fold.
    ///
    /// # Returns
    ///
    /// The final accumulator value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::Foldable;
    /// use fp_library::brands::OptionBrand;
    ///
    /// let x = Some(5);
    /// let y = OptionBrand::fold_right(|a, b| a + b, 10, x);
    /// assert_eq!(y, 15);
    /// ```
    fn fold_right<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(A, B) -> B;

    /// Folds the structure by applying a function from left to right.
    ///
    /// # Type Signature
    ///
    /// `forall a b. Foldable t => ((b, a) -> b, b, t a) -> b`
    ///
    /// # Parameters
    ///
    /// * `f`: The function to apply to the accumulator and each element.
    /// * `init`: The initial value of the accumulator.
    /// * `fa`: The structure to fold.
    ///
    /// # Returns
    ///
    /// The final accumulator value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::Foldable;
    /// use fp_library::brands::OptionBrand;
    ///
    /// let x = Some(5);
    /// let y = OptionBrand::fold_left(|b, a| b + a, 10, x);
    /// assert_eq!(y, 15);
    /// ```
    fn fold_left<'a, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Self, A>) -> B
    where
        F: Fn(B, A) -> B;

    /// Maps values to a monoid and combines them.
    ///
    /// # Type Signature
    ///
    /// `forall a m. (Foldable t, Monoid m) => ((a) -> m, t a) -> m`
    ///
    /// # Parameters
    ///
    /// * `f`: The function to map each element to a monoid.
    /// * `fa`: The structure to fold.
    ///
    /// # Returns
    ///
    /// The combined monoid value.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::foldable::Foldable;
    /// use fp_library::brands::OptionBrand;
    /// use fp_library::v2::types::string; // Import Monoid impl for String
    ///
    /// let x = Some(5);
    /// let y = OptionBrand::fold_map(|a: i32| a.to_string(), x);
    /// assert_eq!(y, "5".to_string());
    /// ```
    fn fold_map<'a, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Self, A>) -> M
    where
        M: Monoid,
        F: Fn(A) -> M;
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
///
/// # Type Signature
///
/// `forall a b. Foldable t => ((a, b) -> b, b, t a) -> b`
///
/// # Parameters
///
/// * `f`: The function to apply to each element and the accumulator.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// # Returns
///
/// The final accumulator value.
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::foldable::fold_right;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = fold_right::<OptionBrand, _, _, _>(|a, b| a + b, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_right<'a, Brand: Foldable, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Brand, A>) -> B
where
    F: Fn(A, B) -> B
{
    Brand::fold_right(f, init, fa)
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
///
/// # Type Signature
///
/// `forall a b. Foldable t => ((b, a) -> b, b, t a) -> b`
///
/// # Parameters
///
/// * `f`: The function to apply to the accumulator and each element.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// # Returns
///
/// The final accumulator value.
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::foldable::fold_left;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = fold_left::<OptionBrand, _, _, _>(|b, a| b + a, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_left<'a, Brand: Foldable, A: 'a, B: 'a, F: 'a>(f: F, init: B, fa: Apply1L1T<'a, Brand, A>) -> B
where
    F: Fn(B, A) -> B
{
    Brand::fold_left(f, init, fa)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
///
/// # Type Signature
///
/// `forall a m. (Foldable t, Monoid m) => ((a) -> m, t a) -> m`
///
/// # Parameters
///
/// * `f`: The function to map each element to a monoid.
/// * `fa`: The structure to fold.
///
/// # Returns
///
/// The combined monoid value.
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::foldable::fold_map;
/// use fp_library::brands::OptionBrand;
/// use fp_library::v2::types::string; // Import Monoid impl for String
///
/// let x = Some(5);
/// let y = fold_map::<OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
/// assert_eq!(y, "5".to_string());
/// ```
pub fn fold_map<'a, Brand: Foldable, A: 'a, M: 'a, F: 'a>(f: F, fa: Apply1L1T<'a, Brand, A>) -> M
where
    M: Monoid,
    F: Fn(A) -> M
{
    Brand::fold_map(f, fa)
}
