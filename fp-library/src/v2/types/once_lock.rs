use crate::{
    hkt::{Apply0L1T, Kind0L1T},
    v2::classes::{once::ApplyOnce, once::Once},
};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceLockBrand;

impl Kind0L1T for OnceLockBrand {
    type Output<A> = OnceLock<A>;
}

impl Once for OnceLockBrand {
    type Output<A> = Apply0L1T<Self, A>;

    /// Creates a new, uninitialized `OnceLock`.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => () -> OnceLock a`
    ///
    /// # Returns
    ///
    /// A new, empty `OnceLock`.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_library::v2::classes::once::Once;
    /// use fp_library::v2::types::once_lock::OnceLockBrand;
    ///
    /// let cell = <OnceLockBrand as Once>::new::<i32>();
    /// assert_eq!(<OnceLockBrand as Once>::get(&cell), None);
    /// ```
    fn new<A>() -> ApplyOnce<Self, A> {
        OnceLock::new()
    }

    /// Gets a reference to the value if it has been initialized.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => OnceLock a -> Option a`
    ///
    /// # Parameters
    ///
    /// * `a`: The `OnceLock`.
    ///
    /// # Returns
    ///
    /// A reference to the value, or `None` if uninitialized.
    fn get<A>(a: &ApplyOnce<Self, A>) -> Option<&A> {
        OnceLock::get(a)
    }

    /// Gets a mutable reference to the value if it has been initialized.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => OnceLock a -> Option a`
    ///
    /// # Parameters
    ///
    /// * `a`: The `OnceLock`.
    ///
    /// # Returns
    ///
    /// A mutable reference to the value, or `None` if uninitialized.
    fn get_mut<A>(a: &mut ApplyOnce<Self, A>) -> Option<&mut A> {
        OnceLock::get_mut(a)
    }

    /// Sets the value of the `OnceLock`.
    ///
    /// Returns `Ok(())` if the value was set, or `Err(value)` if the cell was already initialized.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => (OnceLock a, a) -> Result<(), a>`
    ///
    /// # Parameters
    ///
    /// * `a`: The `OnceLock`.
    /// * `value`: The value to set.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or `Err(value)` if already initialized.
    fn set<A>(a: &ApplyOnce<Self, A>, value: A) -> Result<(), A> {
        OnceLock::set(a, value)
    }

    /// Gets the value, initializing it with the closure `f` if it is not already initialized.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => (OnceLock a, () -> a) -> a`
    ///
    /// # Parameters
    ///
    /// * `a`: The `OnceLock`.
    /// * `f`: The initialization function.
    ///
    /// # Returns
    ///
    /// A reference to the value.
    fn get_or_init<A, B: FnOnce() -> A>(a: &ApplyOnce<Self, A>, f: B) -> &A {
        OnceLock::get_or_init(a, f)
    }

    /// Consumes the `OnceLock` and returns the value if it has been initialized.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => OnceLock a -> Option a`
    ///
    /// # Parameters
    ///
    /// * `a`: The `OnceLock`.
    ///
    /// # Returns
    ///
    /// The value, or `None` if uninitialized.
    fn into_inner<A>(a: ApplyOnce<Self, A>) -> Option<A> {
        OnceLock::into_inner(a)
    }

    /// Takes the value out of the `OnceLock`, leaving it uninitialized.
    ///
    /// # Type Signature
    ///
    /// `forall a. Once OnceLockBrand => OnceLock a -> Option a`
    ///
    /// # Parameters
    ///
    /// * `a`: The `OnceLock`.
    ///
    /// # Returns
    ///
    /// The value, or `None` if uninitialized.
    fn take<A>(a: &mut ApplyOnce<Self, A>) -> Option<A> {
        OnceLock::take(a)
    }
}
