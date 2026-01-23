//! [`OnceCell`] wrapper.
//!
//! This module defines the [`OnceCellBrand`] struct, which provides implementations for [`OnceCell`].
//! It implements [`Once`].

use crate::{Apply, brands::OnceCellBrand, classes::once::Once, impl_kind, kinds::*};
use std::cell::OnceCell;

impl_kind! {
	for OnceCellBrand {
		type Of<A> = OnceCell<A>;
	}
}

impl Once for OnceCellBrand {
	type Of<A> = Apply!(<Self as Kind!( type Of<T>; )>::Of<A>);

	/// Creates a new, uninitialized `Once` container.
	///
	/// This method creates a new instance of the `OnceCell` that is initially empty.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => () -> OnceCell a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to be stored in the container.
	///
	/// ### Returns
	///
	/// A new, empty `OnceCell`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
	/// ```
	fn new<A>() -> <Self as Once>::Of<A> {
		OnceCell::new()
	}

	/// Gets a reference to the value if it has been initialized.
	///
	/// This method returns a reference to the value stored in the `OnceCell` if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => OnceCell a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// ### Returns
	///
	/// A reference to the value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), None);
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_get::<OnceCellBrand, _>(&cell), Some(&42));
	/// ```
	fn get<A>(a: &<Self as Once>::Of<A>) -> Option<&A> {
		OnceCell::get(a)
	}

	/// Gets a mutable reference to the value if it has been initialized.
	///
	/// This method returns a mutable reference to the value stored in the `OnceCell` if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => OnceCell a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// ### Returns
	///
	/// A mutable reference to the value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let mut cell = once_new::<OnceCellBrand, i32>();
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// if let Some(val) = once_get_mut::<OnceCellBrand, _>(&mut cell) {
	///     *val += 1;
	/// }
	/// assert_eq!(once_get_mut::<OnceCellBrand, _>(&mut cell), Some(&mut 43));
	/// ```
	fn get_mut<A>(a: &mut <Self as Once>::Of<A>) -> Option<&mut A> {
		OnceCell::get_mut(a)
	}

	/// Sets the value of the container.
	///
	/// This method attempts to set the value of the `OnceCell`. If the `OnceCell` is already initialized, it returns the value in the `Err` variant.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => (OnceCell a, a) -> Result<(), a>`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value to set.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceCell`.
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
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert!(once_set::<OnceCellBrand, _>(&cell, 42).is_ok());
	/// assert!(once_set::<OnceCellBrand, _>(&cell, 10).is_err());
	/// ```
	fn set<A>(
		a: &<Self as Once>::Of<A>,
		value: A,
	) -> Result<(), A> {
		OnceCell::set(a, value)
	}

	/// Gets the value, initializing it with the closure `f` if it is not already initialized.
	///
	/// This method returns a reference to the value stored in the `OnceCell`. If the `OnceCell` is not initialized, it initializes it using the provided closure `f` and then returns a reference to the value.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => (OnceCell a, () -> a) -> a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	/// * `B`: The type of the initialization function.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceCell`.
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
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// assert_eq!(*once_get_or_init::<OnceCellBrand, _, _>(&cell, || 42), 42);
	/// assert_eq!(*once_get_or_init::<OnceCellBrand, _, _>(&cell, || 10), 42);
	/// ```
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &<Self as Once>::Of<A>,
		f: B,
	) -> &A {
		OnceCell::get_or_init(a, f)
	}

	/// Consumes the container and returns the value if it has been initialized.
	///
	/// This method consumes the `OnceCell` and returns the value stored in it if it has been initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => OnceCell a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// ### Returns
	///
	/// The value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = once_new::<OnceCellBrand, i32>();
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_into_inner::<OnceCellBrand, _>(cell), Some(42));
	/// ```
	fn into_inner<A>(a: <Self as Once>::Of<A>) -> Option<A> {
		OnceCell::into_inner(a)
	}

	/// Takes the value out of the container, leaving it uninitialized.
	///
	/// This method takes the value out of the `OnceCell`, leaving the `OnceCell` in an uninitialized state. It returns the value if it was initialized, otherwise it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a. Once OnceCell => OnceCell a -> Option a`
	///
	/// ### Type Parameters
	///
	/// * `A`: The type of the value stored in the container.
	///
	/// ### Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// ### Returns
	///
	/// The value, or `None` if uninitialized.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::functions::*;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let mut cell = once_new::<OnceCellBrand, i32>();
	/// once_set::<OnceCellBrand, _>(&cell, 42).unwrap();
	/// assert_eq!(once_take::<OnceCellBrand, _>(&mut cell), Some(42));
	/// assert_eq!(once_take::<OnceCellBrand, _>(&mut cell), None);
	/// ```
	fn take<A>(a: &mut <Self as Once>::Of<A>) -> Option<A> {
		OnceCell::take(a)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::classes::once::Once;

	/// Tests the `Once` trait implementation for `OnceCell`.
	#[test]
	fn test_once_cell() {
		let mut cell = <OnceCellBrand as Once>::new::<i32>();
		assert_eq!(<OnceCellBrand as Once>::get(&cell), None);

		assert_eq!(<OnceCellBrand as Once>::set(&cell, 42), Ok(()));
		assert_eq!(<OnceCellBrand as Once>::get(&cell), Some(&42));
		assert_eq!(<OnceCellBrand as Once>::set(&cell, 100), Err(100));
		assert_eq!(<OnceCellBrand as Once>::get(&cell), Some(&42));

		let val = <OnceCellBrand as Once>::get_or_init(&cell, || 99);
		assert_eq!(val, &42);

		let cell2 = <OnceCellBrand as Once>::new::<i32>();
		let val2 = <OnceCellBrand as Once>::get_or_init(&cell2, || 99);
		assert_eq!(val2, &99);
		assert_eq!(<OnceCellBrand as Once>::get(&cell2), Some(&99));

		assert_eq!(<OnceCellBrand as Once>::take(&mut cell), Some(42));
		assert_eq!(<OnceCellBrand as Once>::get(&cell), None);

		assert_eq!(<OnceCellBrand as Once>::into_inner(cell2), Some(99));
	}
}
