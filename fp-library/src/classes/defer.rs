use super::clonable_fn::ClonableFn;
use crate::Apply;

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
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::brands::OnceCellBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let lazy = Lazy::<OnceCellBrand, RcFnBrand, _>::defer::<RcFnBrand>(
	///     <RcFnBrand as ClonableFn>::new(|_| Lazy::new(<RcFnBrand as ClonableFn>::new(|_| 42)))
	/// );
	/// assert_eq!(Lazy::force(lazy), 42);
	/// ```
	fn defer<ClonableFnBrand: 'a + ClonableFn>(
		f: Apply!(brand: ClonableFnBrand, kind: ClonableFn, lifetimes: ('a), types: ((), Self))
	) -> Self
	where
		Self: Sized;
}

/// Creates a value from a computation that produces the value.
///
/// Free function version that dispatches to [the type class' associated function][`Defer::defer`].
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
/// use fp_library::classes::defer::defer;
/// use fp_library::types::lazy::Lazy;
/// use fp_library::brands::RcFnBrand;
/// use fp_library::brands::OnceCellBrand;
/// use fp_library::classes::clonable_fn::ClonableFn;
///
/// let lazy = defer::<Lazy<OnceCellBrand, RcFnBrand, _>, RcFnBrand>(
///     <RcFnBrand as ClonableFn>::new(|_| Lazy::new(<RcFnBrand as ClonableFn>::new(|_| 42)))
/// );
/// assert_eq!(Lazy::force(lazy), 42);
/// ```
pub fn defer<'a, D, ClonableFnBrand>(
	f: Apply!(brand: ClonableFnBrand, kind: ClonableFn, lifetimes: ('a), types: ((), D))
) -> D
where
	D: Defer<'a>,
	ClonableFnBrand: 'a + ClonableFn,
{
	D::defer::<ClonableFnBrand>(f)
}
