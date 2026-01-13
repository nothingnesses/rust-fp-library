use crate::hkt::{Apply_L1_T2, Kind_L1_T2};

/// A type class for semigroupoids.
///
/// A `Semigroupoid` is a set of objects and composable relationships
/// (morphisms) between them.
///
/// # Laws
///
/// Semigroupoid instances must satisfy the associative law:
/// * Associativity: `compose(p, compose(q, r)) = compose(compose(p, q), r)`.
pub trait Semigroupoid: Kind_L1_T2 {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// # Type Signature
	///
	/// `forall b c d. Semigroupoid a => (a c d, a b c) -> a b d`
	///
	/// # Parameters
	///
	/// * `f`: The second morphism to apply (from C to D).
	/// * `g`: The first morphism to apply (from B to C).
	///
	/// # Returns
	///
	/// The composed morphism (from B to D).
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::semigroupoid::Semigroupoid;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let g = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	/// let h = RcFnBrand::compose(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply_L1_T2<'a, Self, C, D>,
		g: Apply_L1_T2<'a, Self, B, C>,
	) -> Apply_L1_T2<'a, Self, B, D>;
}

/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
///
/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
///
/// # Type Signature
///
/// `forall b c d. Semigroupoid a => (a c d, a b c) -> a b d`
///
/// # Parameters
///
/// * `f`: The second morphism to apply (from C to D).
/// * `g`: The first morphism to apply (from B to C).
///
/// # Returns
///
/// The composed morphism (from B to D).
///
/// # Examples
///
/// ```
/// use fp_library::classes::semigroupoid::compose;
/// use fp_library::brands::RcFnBrand;
/// use fp_library::classes::clonable_fn::ClonableFn;
///
/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
/// let g = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
/// let h = compose::<RcFnBrand, _, _, _>(f, g);
/// assert_eq!(h(5), 12); // (5 + 1) * 2
/// ```
pub fn compose<'a, Brand: Semigroupoid, B: 'a, C: 'a, D: 'a>(
	f: Apply_L1_T2<'a, Brand, C, D>,
	g: Apply_L1_T2<'a, Brand, B, C>,
) -> Apply_L1_T2<'a, Brand, B, D> {
	Brand::compose(f, g)
}
