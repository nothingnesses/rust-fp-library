//! Implementations for [`Endomorphism`], a wrapper for endomorphisms (functions from a type to itself) that enables monoidal operations.

use crate::{
	aliases::ArcFn,
	functions::{compose, identity},
	hkt::{Apply0, Kind0},
	typeclasses::{Monoid, Semigroup},
};
use std::{marker::PhantomData, sync::Arc};

#[derive(Clone)]
/// A wrapper for endomorphisms (functions from a type to itself) that enables monoidal operations.
///
/// `Endomorphism<A>` represents a function `A -> A`.
/// It exists to provide a monoid instance where:
/// - The binary operation (`append`) is function composition
/// - The identity element (`empty`) is the identity function
///
/// This allows combining transformations in a composable, associative way with a clear identity,
/// which is useful for building pipelines of transformations or accumulating operations.
///
/// The wrapped function can be accessed directly via the `.0` field.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::EndomorphismBrand, functions::{append, empty}, types::endomorphism::Endomorphism};
/// use std::sync::Arc;
///
/// let double = Arc::new(|x: i32| x * 2);
/// let increment = Arc::new(|x: i32| x + 1);
///
/// // Create endomorphisms
/// let f = Endomorphism(double);
/// let g = Endomorphism(increment);
///
/// // Compose functions (f after g)
/// let fg = append::<EndomorphismBrand<i32>>(f)(g);
/// assert_eq!(fg.0(3), 8); // double(increment(3)) = 8
///
/// // Identity element
/// let id = empty::<EndomorphismBrand<i32>>();
/// assert_eq!(id.0(42), 42);
/// ```
pub struct Endomorphism<'a, A>(pub ArcFn<'a, A, A>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<'a, A>(&'a A, PhantomData<&'a A>);

impl<'a, A> Kind0 for EndomorphismBrand<'a, A> {
	type Output = Endomorphism<'a, A>;
}

impl<'a, A> Semigroup<'a> for EndomorphismBrand<'a, A> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::EndomorphismBrand, functions::append, types::endomorphism::Endomorphism};
	/// use std::sync::Arc;
	///
	/// let double = Arc::new(|x: i32| x * 2);
	/// let increment = Arc::new(|x: i32| x + 1);
	///
	/// assert_eq!(
	///     (append::<EndomorphismBrand<i32>>(Endomorphism(double))(Endomorphism(increment.clone()))).0(3),
	///     8
	/// );
	/// assert_eq!(
	///     (append::<EndomorphismBrand<i32>>(Endomorphism(increment.clone()))(Endomorphism(increment))).0(3),
	///     5
	/// );
	/// ```
	fn append(a: Apply0<Self>) -> ArcFn<'a, Apply0<Self>, Apply0<Self>> {
		Arc::new(move |b| Endomorphism(compose(a.0.clone())(b.0)))
	}
}

impl<'a, A> Monoid<'a> for EndomorphismBrand<'a, A> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::EndomorphismBrand, functions::empty};
	///
	/// assert_eq!(empty::<EndomorphismBrand<i32>>().0(5), 5);
	/// assert_eq!(empty::<EndomorphismBrand<String>>().0("test".to_string()), "test");
	/// ```
	fn empty() -> Apply0<Self> {
		Endomorphism(Arc::new(identity))
	}
}
