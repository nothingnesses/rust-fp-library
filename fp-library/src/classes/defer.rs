use super::clonable_fn::{ApplyClonableFn, ClonableFn};

/// A type class for types that can be constructed lazily.
pub trait Defer<'a> {
	/// Creates a value from a computation that produces the value.
	///
	/// # Type Signature
	///
	/// `forall a. Defer d => (() -> d a) -> d a`
	///
	/// # Parameters
	///
	/// * `f`: A thunk (wrapped in a clonable function) that produces the value.
	///
	/// # Returns
	///
	/// The deferred value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::defer::Defer;
	/// use fp_library::types::lazy::Lazy;
	/// use fp_library::types::rc_fn::RcFnBrand;
	/// use fp_library::types::once_cell::OnceCellBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::defer::<RcFnBrand>(
	///     <RcFnBrand as ClonableFn>::new(|_| Lazy::new(<RcFnBrand as ClonableFn>::new(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force(lazy), 42);
	/// ```
	fn defer<ClonableFnBrand: 'a + ClonableFn>(
		f: ApplyClonableFn<'a, ClonableFnBrand, (), Self>
	) -> Self
	where
		Self: Sized;
}
