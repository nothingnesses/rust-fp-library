use crate::{hkt::Kind0L1T, make_type_apply};

/// A type class for types that can be initialized once.
///
/// `Once` represents a container that holds a value that is initialized at most once.
/// It provides methods for initialization, access, and consumption.
pub trait Once: Kind0L1T {
	type Output<A>;

	/// Creates a new, uninitialized `Once` container.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => () -> f a`
	///
	/// # Returns
	///
	/// A new, empty container.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::types::once_cell::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert_eq!(<OnceCellBrand as Once>::get(&cell), None);
	/// ```
	fn new<A>() -> ApplyOnce<Self, A>;

	/// Gets a reference to the value if it has been initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The container.
	///
	/// # Returns
	///
	/// A reference to the value, or `None` if uninitialized.
	fn get<A>(a: &ApplyOnce<Self, A>) -> Option<&A>;

	/// Gets a mutable reference to the value if it has been initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The container.
	///
	/// # Returns
	///
	/// A mutable reference to the value, or `None` if uninitialized.
	fn get_mut<A>(a: &mut ApplyOnce<Self, A>) -> Option<&mut A>;

	/// Sets the value of the container.
	///
	/// Returns `Ok(())` if the value was set, or `Err(value)` if the container was already initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => (f a, a) -> Result<(), a>`
	///
	/// # Parameters
	///
	/// * `a`: The container.
	/// * `value`: The value to set.
	///
	/// # Returns
	///
	/// `Ok(())` on success, or `Err(value)` if already initialized.
	fn set<A>(
		a: &ApplyOnce<Self, A>,
		value: A,
	) -> Result<(), A>;

	/// Gets the value, initializing it with the closure `f` if it is not already initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => (f a, () -> a) -> a`
	///
	/// # Parameters
	///
	/// * `a`: The container.
	/// * `f`: The initialization function.
	///
	/// # Returns
	///
	/// A reference to the value.
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &ApplyOnce<Self, A>,
		f: B,
	) -> &A;

	/// Consumes the container and returns the value if it has been initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The container.
	///
	/// # Returns
	///
	/// The value, or `None` if uninitialized.
	fn into_inner<A>(a: ApplyOnce<Self, A>) -> Option<A>;

	/// Takes the value out of the container, leaving it uninitialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The container.
	///
	/// # Returns
	///
	/// The value, or `None` if uninitialized.
	fn take<A>(a: &mut ApplyOnce<Self, A>) -> Option<A>;
}

make_type_apply!(ApplyOnce, Once, (), (A), "* -> *");
