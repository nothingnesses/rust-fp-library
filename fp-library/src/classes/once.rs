//! A type class for containers that hold a value that is initialized at most once.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let cell = once_new::<OnceCellBrand, i32>();
//! assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
//! ```

use crate::kinds::*;

/// A type class for containers that hold a value that is initialized at most once.
///
/// It provides methods for initialization, access, and consumption.
pub trait Once: Kind_ad6c20556a82a1f0 {
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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
	/// ```
	fn new<A>() -> <Self as Once>::Of<A>;

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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), Some(&42));
	/// ```
	fn get<A>(a: &<Self as Once>::Of<A>) -> Option<&A>;

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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let mut cell = once_new::<OnceCellBrand, i32>();
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// if let Some(val) = once_get_mut::<OnceCellBrand, _>(&mut cell) {
	///     *val += 1;
	/// }
	/// assert_eq!(once_get_mut::<OnceCellBrand, _>(&mut cell), Some(&mut 43));
	/// ```
	fn get_mut<A>(a: &mut <Self as Once>::Of<A>) -> Option<&mut A>;

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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert!(once_set::<OnceCellBrand, _>(&cell, 42).is_ok());
	/// assert!(once_set::<OnceCellBrand, _>(&cell, 10).is_err());
	/// ```
	fn set<A>(
		a: &<Self as Once>::Of<A>,
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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert_eq!(*once_get_or_init::<OnceCellBrand, _, _>(&cell, || 42), 42);
	/// assert_eq!(*once_get_or_init::<OnceCellBrand, _, _>(&cell, || 10), 42);
	/// ```
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &<Self as Once>::Of<A>,
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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_into_inner::<OnceCellBrand, _>(cell), Some(42));
	/// ```
	fn into_inner<A>(a: <Self as Once>::Of<A>) -> Option<A>;

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
	/// use fp_library::{brands::*, functions::*};
	///
	/// let mut cell = once_new::<OnceCellBrand, i32>();
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_take::<OnceCellBrand, _>(&mut cell), Some(42));
	/// assert_eq!(once_take::<OnceCellBrand, _>(&mut cell), None);
	/// ```
	fn take<A>(a: &mut <Self as Once>::Of<A>) -> Option<A>;
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
/// * `Brand`: The brand of the container.
/// * `A`: The type of the value to be stored in the container.
///
/// ### Returns
///
/// A new, empty container.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let cell = once_new::<OnceCellBrand, i32>();
/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
/// ```
pub fn new<Brand, A>() -> <Brand as Once>::Of<A>
where
	Brand: Once,
{
	Brand::new()
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
/// * `Brand`: The brand of the container.
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
/// use fp_library::{brands::*, functions::*};
///
/// let cell = once_new::<OnceCellBrand, i32>();
/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), Some(&42));
/// ```
pub fn get<Brand, A>(a: &<Brand as Once>::Of<A>) -> Option<&A>
where
	Brand: Once,
{
	Brand::get(a)
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
/// * `Brand`: The brand of the container.
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
/// use fp_library::{brands::*, functions::*};
///
/// let mut cell = once_new::<OnceCellBrand, i32>();
/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// if let Some(val) = once_get_mut::<OnceCellBrand, _>(&mut cell) {
///     *val += 1;
/// }
/// assert_eq!(once_get_mut::<OnceCellBrand, _>(&mut cell), Some(&mut 43));
/// ```
pub fn get_mut<Brand, A>(a: &mut <Brand as Once>::Of<A>) -> Option<&mut A>
where
	Brand: Once,
{
	Brand::get_mut(a)
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
/// * `Brand`: The brand of the container.
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
/// use fp_library::{brands::*, functions::*};
///
/// let cell = once_new::<OnceCellBrand, i32>();
/// assert!(once_set::<OnceCellBrand, _>(&cell, 42).is_ok());
/// assert!(once_set::<OnceCellBrand, _>(&cell, 10).is_err());
/// ```
pub fn set<Brand, A>(
	a: &<Brand as Once>::Of<A>,
	value: A,
) -> Result<(), A>
where
	Brand: Once,
{
	Brand::set(a, value)
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
/// * `Brand`: The brand of the container.
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
/// use fp_library::{brands::*, functions::*};
///
/// let cell = once_new::<OnceCellBrand, i32>();
/// assert_eq!(*once_get_or_init::<OnceCellBrand, _, _>(&cell, || 42), 42);
/// assert_eq!(*once_get_or_init::<OnceCellBrand, _, _>(&cell, || 10), 42);
/// ```
pub fn get_or_init<Brand, A, B>(
	a: &<Brand as Once>::Of<A>,
	f: B,
) -> &A
where
	Brand: Once,
	B: FnOnce() -> A,
{
	Brand::get_or_init(a, f)
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
/// * `Brand`: The brand of the container.
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
/// use fp_library::{brands::*, functions::*};
///
/// let cell = once_new::<OnceCellBrand, i32>();
/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// assert_eq!(once_into_inner::<OnceCellBrand, _>(cell), Some(42));
/// ```
pub fn into_inner<Brand, A>(a: <Brand as Once>::Of<A>) -> Option<A>
where
	Brand: Once,
{
	Brand::into_inner(a)
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
/// * `Brand`: The brand of the container.
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
/// use fp_library::{brands::*, functions::*};
///
/// let mut cell = once_new::<OnceCellBrand, i32>();
/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
/// assert_eq!(once_take::<OnceCellBrand, _>(&mut cell), Some(42));
/// assert_eq!(once_take::<OnceCellBrand, _>(&mut cell), None);
/// ```
pub fn take<Brand, A>(a: &mut <Brand as Once>::Of<A>) -> Option<A>
where
	Brand: Once,
{
	Brand::take(a)
}
