use crate::hkt::Apply1L2T;
use super::semigroupoid::Semigroupoid;

/// A type class for categories.
///
/// `Category` extends [`Semigroupoid`] with an identity element.
///
/// # Laws
///
/// `Category` instances must satisfy the identity law:
/// * Identity: `compose(identity, p) = compose(p, identity)`.
pub trait Category: Semigroupoid {
    /// Returns the identity morphism.
    ///
    /// # Type Signature
    ///
    /// `forall a. Category cat => () -> cat a a`
    ///
    /// # Returns
    ///
    /// The identity morphism.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::category::Category;
    /// use fp_library::v2::types::rc_fn::RcFnBrand;
    ///
    /// let id = RcFnBrand::identity::<i32>();
    /// assert_eq!(id(5), 5);
    /// ```
    fn identity<'a, A>() -> Apply1L2T<'a, Self, A, A>;
}

/// Returns the identity morphism.
///
/// Free function version that dispatches to [the type class' associated function][`Category::identity`].
///
/// # Type Signature
///
/// `forall a. Category cat => () -> cat a a`
///
/// # Returns
///
/// The identity morphism.
///
/// # Examples
///
/// ```
/// use fp_library::v2::classes::category::identity;
/// use fp_library::v2::types::rc_fn::RcFnBrand;
///
/// let id = identity::<RcFnBrand, i32>();
/// assert_eq!(id(5), 5);
/// ```
pub fn identity<'a, Brand: Category, A>() -> Apply1L2T<'a, Brand, A, A> {
    Brand::identity()
}
