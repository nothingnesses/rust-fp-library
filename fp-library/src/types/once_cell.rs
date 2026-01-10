use crate::{
	classes::{once::ApplyOnce, once::Once},
	hkt::{Apply0L1T, Kind0L1T},
};
use std::cell::OnceCell;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OnceCellBrand;

impl Kind0L1T for OnceCellBrand {
	type Output<A> = OnceCell<A>;
}

impl Once for OnceCellBrand {
	type Output<A> = Apply0L1T<Self, A>;

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
	/// use fp_library::types::once_cell::OnceCellBrand;
	///
	/// let cell = <OnceCellBrand as Once>::new::<i32>();
	/// assert_eq!(<OnceCellBrand as Once>::get(&cell), None);
	/// ```
	fn new<A>() -> ApplyOnce<Self, A> {
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
	fn get<A>(a: &ApplyOnce<Self, A>) -> Option<&A> {
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
	fn get_mut<A>(a: &mut ApplyOnce<Self, A>) -> Option<&mut A> {
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
		a: &ApplyOnce<Self, A>,
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
		a: &ApplyOnce<Self, A>,
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
	fn into_inner<A>(a: ApplyOnce<Self, A>) -> Option<A> {
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
	fn take<A>(a: &mut ApplyOnce<Self, A>) -> Option<A> {
		OnceCell::take(a)
	}
}
