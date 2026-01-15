//! Implementations for [`OnceLock`]

use crate::{Apply, brands::OnceLockBrand, classes::once::Once, impl_kind, kinds::*};
use std::sync::OnceLock;

impl_kind! {
	for OnceLockBrand {
		type Of<A> = OnceLock<A>;
	}
}

impl Once for OnceLockBrand {
	type Of<A> = Apply!(brand: Self, signature: (A), lifetimes: (), types: (A));

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
	/// use fp_library::classes::once::Once;
	/// use fp_library::brands::OnceLockBrand;
	///
	/// let cell = <OnceLockBrand as Once>::new::<i32>();
	/// assert_eq!(<OnceLockBrand as Once>::get(&cell), None);
	/// ```
	fn new<A>() -> Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)) {
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
	fn get<A>(a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<&A> {
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
	fn get_mut<A>(
		a: &mut Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))
	) -> Option<&mut A> {
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
	fn set<A>(
		a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)),
		value: A,
	) -> Result<(), A> {
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
	fn get_or_init<A, B: FnOnce() -> A>(
		a: &Apply!(brand: Self, kind: Once, lifetimes: (), types: (A)),
		f: B,
	) -> &A {
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
	fn into_inner<A>(a: Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<A> {
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
	fn take<A>(a: &mut Apply!(brand: Self, kind: Once, lifetimes: (), types: (A))) -> Option<A> {
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
