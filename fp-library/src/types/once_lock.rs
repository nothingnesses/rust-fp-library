//! [`OnceLock`] wrapper.
//!
//! This module defines the [`OnceLockBrand`] struct, which provides implementations for [`OnceLock`].
//! It implements [`Once`].

use crate::{Apply, brands::OnceLockBrand, classes::once::Once, impl_kind, kinds::*};
use std::sync::OnceLock;

impl_kind! {
	for OnceLockBrand {
		type Of<A> = OnceLock<A>;
	}
}

impl Once for OnceLockBrand {
	type Of<A> = Apply!(<Self as Kind!( type Of<T>; )>::Of<A>);

	/// Creates a new, uninitialized `Once` container.
	///
	/// This method creates a new instance of the `OnceLock` that is initially empty.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => () -> OnceLock a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to be stored in the container.
	///
	/// ### Returns
	///
	/// A new, empty `OnceLock`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let cell = once_new::<OnceLockBrand, i32>();
	/// assert_eq!(once_get::<OnceLockBrand, _>(&cell), None);
	/// ```
	fn new<A>() -> <Self as Once>::Of<A> {
		OnceLock::new()
	}

	/// Gets a reference to the value if it has been initialized.
	///
	/// This method returns a reference to the value stored in the `OnceLock` if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => OnceLock a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceLock`.
	///
	/// ### Returns
	///
	/// A reference to the value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let cell = once_new::<OnceLockBrand, i32>();
	/// assert_eq!(once_get::<OnceLockBrand, _>(&cell), None);
	/// once_set::<OnceLockBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_get::<OnceLockBrand, _>(&cell), Some(&42));
	/// ```
	fn get<A>(a: &<Self as Once>::Of<A>) -> Option<&A> {
		OnceLock::get(a)
	}

	/// Gets a mutable reference to the value if it has been initialized.
	///
	/// This method returns a mutable reference to the value stored in the `OnceLock` if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => OnceLock a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceLock`.
	///
	/// ### Returns
	///
	/// A mutable reference to the value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let mut cell = once_new::<OnceLockBrand, i32>();
	/// once_set::<OnceLockBrand, _>(&cell, 42).unwrap();
	/// if let Some(val) = once_get_mut::<OnceLockBrand, _>(&mut cell) {
	///     *val += 1;
	/// }
	/// assert_eq!(once_get_mut::<OnceLockBrand, _>(&mut cell), Some(&mut 43));
	/// ```
	fn get_mut<A>(a: &mut <Self as Once>::Of<A>) -> Option<&mut A> {
		OnceLock::get_mut(a)
	}

	/// Sets the value of the container.
	///
	/// This method attempts to set the value of the `OnceLock`. If the `OnceLock` is already initialized, it returns the value in the `Err` variant.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => (OnceLock a, a) -> Result<(), a>`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to set.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceLock`.
	/// * `value`: The value to set.
	///
	/// ### Returns
	///
	/// `Ok(())` on success, or `Err(value)` if already initialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let cell = once_new::<OnceLockBrand, i32>();
	/// assert!(once_set::<OnceLockBrand, _>(&cell, 42).is_ok());
	/// assert!(once_set::<OnceLockBrand, _>(&cell, 10).is_err());
	/// ```
	fn set<A>(
		a: &<Self as Once>::Of<A>,
		value: A,
	) -> Result<(), A> {
		OnceLock::set(a, value)
	}

	/// Gets the value, initializing it with the closure `f` if it is not already initialized.
	///
	/// This method returns a reference to the value stored in the `OnceLock`. If the `OnceLock` is not initialized, it initializes it using the provided closure `f` and then returns a reference to the value.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => (OnceLock a, () -> a) -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	/// * `B`: The type of the initialization function.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceLock`.
	/// * `f`: The initialization function.
	///
	/// ### Returns
	///
	/// A reference to the value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let cell = once_new::<OnceLockBrand, i32>();
	/// assert_eq!(*once_get_or_init::<OnceLockBrand, _, _>(&cell, || 42), 42);
	/// assert_eq!(*once_get_or_init::<OnceLockBrand, _, _>(&cell, || 10), 42);
	/// ```
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &<Self as Once>::Of<A>,
		f: B,
	) -> &A {
		OnceLock::get_or_init(a, f)
	}

	/// Consumes the container and returns the value if it has been initialized.
	///
	/// This method consumes the `OnceLock` and returns the value stored in it if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => OnceLock a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceLock`.
	///
	/// ### Returns
	///
	/// The value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let cell = once_new::<OnceLockBrand, i32>();
	/// once_set::<OnceLockBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_into_inner::<OnceLockBrand, _>(cell), Some(42));
	/// ```
	fn into_inner<A>(a: <Self as Once>::Of<A>) -> Option<A> {
		OnceLock::into_inner(a)
	}

	/// Takes the value out of the container, leaving it uninitialized.
	///
	/// This method takes the value out of the `OnceLock`, leaving the `OnceLock` in an uninitialized state. It returns the value if it was initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceLock => OnceLock a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceLock`.
	///
	/// ### Returns
	///
	/// The value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let mut cell = once_new::<OnceLockBrand, i32>();
	/// once_set::<OnceLockBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_take::<OnceLockBrand, _>(&mut cell), Some(42));
	/// assert_eq!(once_take::<OnceLockBrand, _>(&mut cell), None);
	/// ```
	fn take<A>(a: &mut <Self as Once>::Of<A>) -> Option<A> {
		OnceLock::take(a)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::once::Once;

	/// Tests the `Once` trait implementation for `OnceLock`.
	#[test]
	fn test_once_lock() {
		let mut cell = <OnceLockBrand as Once>::new::<i32>();
		assert_eq!(<OnceLockBrand as Once>::get(&cell), None);

		assert_eq!(<OnceLockBrand as Once>::set(&cell, 42), Ok(()));
		assert_eq!(<OnceLockBrand as Once>::get(&cell), Some(&42));
		assert_eq!(<OnceLockBrand as Once>::set(&cell, 100), Err(100));
		assert_eq!(<OnceLockBrand as Once>::get(&cell), Some(&42));

		let val = <OnceLockBrand as Once>::get_or_init(&cell, || 99);
		assert_eq!(val, &42);

		let cell2 = <OnceLockBrand as Once>::new::<i32>();
		let val2 = <OnceLockBrand as Once>::get_or_init(&cell2, || 99);
		assert_eq!(val2, &99);
		assert_eq!(<OnceLockBrand as Once>::get(&cell2), Some(&99));

		assert_eq!(<OnceLockBrand as Once>::take(&mut cell), Some(42));
		assert_eq!(<OnceLockBrand as Once>::get(&cell), None);

		assert_eq!(<OnceLockBrand as Once>::into_inner(cell2), Some(99));
	}
}
