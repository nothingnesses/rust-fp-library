//! Once type class.
//!
//! This module defines the [`Once`] trait, which represents a container that holds a value that is initialized at most once.

use crate::{Apply, kinds::*};

/// A type class for types that can be initialized once.
///
/// `Once` represents a container that holds a value that is initialized at most once.
/// It provides methods for initialization, access, and consumption.
pub trait Once: Kind_bd4ddc17b95f4bc6 {
	type Of<A>;

	/// Creates a new, uninitialized `Once` container.
	///
	/// This method creates a new instance of the container that is initially empty.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => () -> f a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to be stored in the container.
	///
	/// ### Returns
	///
	/// A new, empty container.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert_eq!(<OnceCellBrand as Once>::get(&cell), None);
	/// ```
	fn new<A>() -> Apply!(brand: Self, kind: Once, lifetimes: (), types: (A));

	/// Gets a reference to the value if it has been initialized.
	///
	/// This method returns a reference to the value stored in the container if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The container.
	///
	/// ### Returns
	///
	/// A reference to the value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert_eq!(<OnceCellBrand as Once>::get(&cell), None);
	/// <OnceCellBrand as Once>::set(&cell, 42).unwrap();
	/// assert_eq!(<OnceCellBrand as Once>::get(&cell), Some(&42));
	/// ```
	fn get<A>(a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<&A>;

	/// Gets a mutable reference to the value if it has been initialized.
	///
	/// This method returns a mutable reference to the value stored in the container if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The container.
	///
	/// ### Returns
	///
	/// A mutable reference to the value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let mut cell = <OnceCellBrand as Once>::new::<i32>();
	/// <OnceCellBrand as Once>::set(&cell, 42).unwrap();
	/// if let Some(val) = <OnceCellBrand as Once>::get_mut(&mut cell) {
	///     *val += 1;
	/// }
	/// assert_eq!(<OnceCellBrand as Once>::get_mut(&mut cell), Some(&mut 43));
	/// ```
	fn get_mut<A>(
		a: &mut Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))
	) -> Option<&mut A>;

	/// Sets the value of the container.
	///
	/// This method attempts to set the value of the container. If the container is already initialized, it returns the value in the `Err` variant.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => (f a, a) -> Result<(), a>`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to set.
	///
	/// ### Parameters
	///
	/// * `a`: The container.
	/// * `value`: The value to set.
	///
	/// ### Returns
	///
	/// `Ok(())` on success, or `Err(value)` if already initialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert!(<OnceCellBrand as Once>::set(&cell, 42).is_ok());
	/// assert!(<OnceCellBrand as Once>::set(&cell, 10).is_err());
	/// ```
	fn set<A>(
		a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)),
		value: A,
	) -> Result<(), A>;

	/// Gets the value, initializing it with the closure `f` if it is not already initialized.
	///
	/// This method returns a reference to the value stored in the container. If the container is not initialized, it initializes it using the provided closure `f` and then returns a reference to the value.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => (f a, () -> a) -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	/// * `B`: The type of the initialization function.
	///
	/// ### Parameters
	///
	/// * `a`: The container.
	/// * `f`: The initialization function.
	///
	/// ### Returns
	///
	/// A reference to the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert_eq!(*<OnceCellBrand as Once>::get_or_init(&cell, || 42), 42);
	/// assert_eq!(*<OnceCellBrand as Once>::get_or_init(&cell, || 10), 42);
	/// ```
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)),
		f: B,
	) -> &A;

	/// Consumes the container and returns the value if it has been initialized.
	///
	/// This method consumes the container and returns the value stored in it if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The container.
	///
	/// ### Returns
	///
	/// The value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// <OnceCellBrand as Once>::set(&cell, 42).unwrap();
	/// assert_eq!(<OnceCellBrand as Once>::into_inner(cell), Some(42));
	/// ```
	fn into_inner<A>(a: Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<A>;

	/// Takes the value out of the container, leaving it uninitialized.
	///
	/// This method takes the value out of the container, leaving the container in an uninitialized state. It returns the value if it was initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once f => f a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The container.
	///
	/// ### Returns
	///
	/// The value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let mut cell = <OnceCellBrand as Once>::new::<i32>();
	/// <OnceCellBrand as Once>::set(&cell, 42).unwrap();
	/// assert_eq!(<OnceCellBrand as Once>::take(&mut cell), Some(42));
	/// assert_eq!(<OnceCellBrand as Once>::take(&mut cell), None);
	/// ```
	fn take<A>(a: &mut Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<A>;
}

/// Creates a new, uninitialized `Once` container.
///
/// Free function version that dispatches to [the type class' associated function][`Once::new`].
///
/// ### Type Signature
///
/// `forall a. Once f => () -> f a`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value to be stored in the container.
///
/// ### Returns
///
/// A new, empty container.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::new;
/// use fp_library::classes::once::get;
/// use fp_library::brands::OnceCellBrand;
///
/// let cell = new::<OnceCellBrand, i32>();
/// assert_eq!(get::<OnceCellBrand, _>(&cell), None);
/// ```
pub fn new<F, A>() -> Apply!(brand: F, kind: Once, lifetimes: (), types: (A))
where
	F: Once,
{
	F::new()
}

/// Gets a reference to the value if it has been initialized.
///
/// Free function version that dispatches to [the type class' associated function][`Once::get`].
///
/// ### Type Signature
///
/// `forall a. Once f => f a -> Option a`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value stored in the container.
///
/// ### Parameters
///
/// * `a`: The container.
///
/// ### Returns
///
/// A reference to the value, or `None` if uninitialized.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::{new, get, set};
/// use fp_library::brands::OnceCellBrand;
///
/// let cell = new::<OnceCellBrand, i32>();
/// assert_eq!(get::<OnceCellBrand, _>(&cell), None);
/// set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// assert_eq!(get::<OnceCellBrand, _>(&cell), Some(&42));
/// ```
pub fn get<F, A>(a: &Apply!(brand: F, kind: Once, lifetimes: (), types: (A))) -> Option<&A>
where
	F: Once,
{
	F::get(a)
}

/// Gets a mutable reference to the value if it has been initialized.
///
/// Free function version that dispatches to [the type class' associated function][`Once::get_mut`].
///
/// ### Type Signature
///
/// `forall a. Once f => f a -> Option a`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value stored in the container.
///
/// ### Parameters
///
/// * `a`: The container.
///
/// ### Returns
///
/// A mutable reference to the value, or `None` if uninitialized.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::{new, get_mut, set};
/// use fp_library::brands::OnceCellBrand;
///
/// let mut cell = new::<OnceCellBrand, i32>();
/// set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// if let Some(val) = get_mut::<OnceCellBrand, _>(&mut cell) {
///     *val += 1;
/// }
/// assert_eq!(get_mut::<OnceCellBrand, _>(&mut cell), Some(&mut 43));
/// ```
pub fn get_mut<F, A>(
	a: &mut Apply!(brand: F, kind: Once, lifetimes: (), types: (A))
) -> Option<&mut A>
where
	F: Once,
{
	F::get_mut(a)
}

/// Sets the value of the container.
///
/// This function attempts to set the value of the container. If the container is already initialized, it returns the value in the `Err` variant.
///
/// Free function version that dispatches to [the type class' associated function][`Once::set`].
///
/// ### Type Signature
///
/// `forall a. Once f => (f a, a) -> Result<(), a>`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value to set.
///
/// ### Parameters
///
/// * `a`: The container.
/// * `value`: The value to set.
///
/// ### Returns
///
/// `Ok(())` on success, or `Err(value)` if already initialized.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::{new, set};
/// use fp_library::brands::OnceCellBrand;
///
/// let cell = new::<OnceCellBrand, i32>();
/// assert!(set::<OnceCellBrand, _>(&cell, 42).is_ok());
/// assert!(set::<OnceCellBrand, _>(&cell, 10).is_err());
/// ```
pub fn set<F, A>(
	a: &Apply!(brand: F, kind: Once, lifetimes: (), types: (A)),
	value: A,
) -> Result<(), A>
where
	F: Once,
{
	F::set(a, value)
}

/// Gets the value, initializing it with the closure `f` if it is not already initialized.
///
/// Free function version that dispatches to [the type class' associated function][`Once::get_or_init`].
///
/// ### Type Signature
///
/// `forall a. Once f => (f a, () -> a) -> a`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value stored in the container.
/// * `B`: The type of the initialization function.
///
/// ### Parameters
///
/// * `a`: The container.
/// * `f`: The initialization function.
///
/// ### Returns
///
/// A reference to the value.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::{new, get_or_init};
/// use fp_library::brands::OnceCellBrand;
///
/// let cell = new::<OnceCellBrand, i32>();
/// assert_eq!(*get_or_init::<OnceCellBrand, _, _>(&cell, || 42), 42);
/// assert_eq!(*get_or_init::<OnceCellBrand, _, _>(&cell, || 10), 42);
/// ```
pub fn get_or_init<F, A, B>(
	a: &Apply!(brand: F, kind: Once, lifetimes: (), types: (A)),
	f: B,
) -> &A
where
	F: Once,
	B: FnOnce() -> A,
{
	F::get_or_init(a, f)
}

/// Consumes the container and returns the value if it has been initialized.
///
/// Free function version that dispatches to [the type class' associated function][`Once::into_inner`].
///
/// ### Type Signature
///
/// `forall a. Once f => f a -> Option a`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value stored in the container.
///
/// ### Parameters
///
/// * `a`: The container.
///
/// ### Returns
///
/// The value, or `None` if uninitialized.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::{new, set, into_inner};
/// use fp_library::brands::OnceCellBrand;
///
/// let cell = new::<OnceCellBrand, i32>();
/// set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// assert_eq!(into_inner::<OnceCellBrand, _>(cell), Some(42));
/// ```
pub fn into_inner<F, A>(a: Apply!(brand: F, kind: Once, lifetimes: (), types: (A))) -> Option<A>
where
	F: Once,
{
	F::into_inner(a)
}

/// Takes the value out of the container, leaving it uninitialized.
///
/// Free function version that dispatches to [the type class' associated function][`Once::take`].
///
/// ### Type Signature
///
/// `forall a. Once f => f a -> Option a`
///
/// ### Type Parameters
///
/// * `F`: The brand of the container.
/// * `A`: The type of the value stored in the container.
///
/// ### Parameters
///
/// * `a`: The container.
///
/// ### Returns
///
/// The value, or `None` if uninitialized.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::once::{new, set, take};
/// use fp_library::brands::OnceCellBrand;
///
/// let mut cell = new::<OnceCellBrand, i32>();
/// set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// assert_eq!(take::<OnceCellBrand, _>(&mut cell), Some(42));
/// assert_eq!(take::<OnceCellBrand, _>(&mut cell), None);
/// ```
pub fn take<F, A>(a: &mut Apply!(brand: F, kind: Once, lifetimes: (), types: (A))) -> Option<A>
where
	F: Once,
{
	F::take(a)
}
