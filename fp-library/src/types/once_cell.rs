//! Implementations for [`OnceCell`]

use crate::{Apply, brands::OnceCellBrand, classes::once::Once, impl_kind, kinds::*};
use std::cell::OnceCell;

impl_kind! {
	for OnceCellBrand {
		type Of<A> = OnceCell<A>;
	}
}

impl Once for OnceCellBrand {
	type Of<A> = Apply!(brand: Self, signature: (A), lifetimes: (), types: (A));

	/// Creates a new, uninitialized `OnceCell`.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => () -> OnceCell a`
	///
	/// # Returns
	///
	/// A new, empty `OnceCell`.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert_eq!(<OnceCellBrand as Once>::get(&cell), None);
	/// ```
	fn new<A>() -> Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)) {
		OnceCell::new()
	}

	/// Gets a reference to the value if it has been initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => OnceCell a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// # Returns
	///
	/// A reference to the value, or `None` if uninitialized.
	fn get<A>(a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<&A> {
		OnceCell::get(a)
	}

	/// Gets a mutable reference to the value if it has been initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => OnceCell a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// # Returns
	///
	/// A mutable reference to the value, or `None` if uninitialized.
	fn get_mut<A>(
		a: &mut Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))
	) -> Option<&mut A> {
		OnceCell::get_mut(a)
	}

	/// Sets the value of the `OnceCell`.
	///
	/// Returns `Ok(())` if the value was set, or `Err(value)` if the cell was already initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => (OnceCell a, a) -> Result<(), a>`
	///
	/// # Parameters
	///
	/// * `a`: The `OnceCell`.
	/// * `value`: The value to set.
	///
	/// # Returns
	///
	/// `Ok(())` on success, or `Err(value)` if already initialized.
	fn set<A>(
		a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)),
		value: A,
	) -> Result<(), A> {
		OnceCell::set(a, value)
	}

	/// Gets the value, initializing it with the closure `f` if it is not already initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => (OnceCell a, () -> a) -> a`
	///
	/// # Parameters
	///
	/// * `a`: The `OnceCell`.
	/// * `f`: The initialization function.
	///
	/// # Returns
	///
	/// A reference to the value.
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)),
		f: B,
	) -> &A {
		OnceCell::get_or_init(a, f)
	}

	/// Consumes the `OnceCell` and returns the value if it has been initialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => OnceCell a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// # Returns
	///
	/// The value, or `None` if uninitialized.
	fn into_inner<A>(a: Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<A> {
		OnceCell::into_inner(a)
	}

	/// Takes the value out of the `OnceCell`, leaving it uninitialized.
	///
	/// # Type Signature
	///
	/// `forall a. Once OnceCellBrand => OnceCell a -> Option a`
	///
	/// # Parameters
	///
	/// * `a`: The `OnceCell`.
	///
	/// # Returns
	///
	/// The value, or `None` if uninitialized.
	fn take<A>(a: &mut Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<A> {
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
