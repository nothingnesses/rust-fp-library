use crate::typeclasses::{ClonableFn, clonable_fn::ApplyFn};

/// A typeclass for semigroups.
///
/// A `Semigroup` is a set equipped with an associative binary operation.
/// This means for any elements `a`, `b`, and `c` in the set, the operation
/// satisfies: `(a <> b) <> c = a <> (b <> c)`.
///
/// In functional programming, semigroups are useful for combining values
/// in a consistent way. They form the basis for more complex structures
/// like monoids.
///
/// # Laws
///
/// Semigroup instances must satisfy the associative law:
/// * Associativity: `append(append(x)(y))(z) = append(x)(append(y)(z))`.
///
/// # Examples
///
/// Common semigroups include:
/// * Strings with concatenation.
/// * Numbers with addition.
/// * Numbers with multiplication.
/// * Lists with concatenation.
pub trait Semigroup {
	/// Associative operation that combines two values of the same type.
	///
	/// # Type Signature
	///
	/// `forall a. Semigroup a => a -> a -> a`
	///
	/// # Parameters
	///
	/// * `a`: First value to combine.
	/// * `b`: Second value to combine.
	///
	/// # Returns
	///
	/// The result of combining the two values using the semigroup operation.
	fn append<'a, ClonableFnBrand: 'a + ClonableFn>(
		a: Self
	) -> ApplyFn<'a, ClonableFnBrand, Self, Self>
	where
		Self: Sized;
}

/// Associative operation that combines two values of the same type.
///
/// Free function version that dispatches to [the typeclass' associated function][`Semigroup::append`].
///
/// # Type Signature
///
/// `forall a. Semigroup a => a -> a -> a`
///
/// # Parameters
///
/// * `a`: First value to combine.
/// * `b`: Second value to combine.
///
/// # Returns
///
/// The result of combining the two values using the semigroup operation.
///
/// # Examples
///
/// ```
/// use fp_library::{brands::RcFnBrand, functions::append};
///
/// assert_eq!(
///     append::<RcFnBrand, String>("Hello, ".to_string())("World!".to_string()),
///     "Hello, World!"
/// );
/// ```
pub fn append<'a, ClonableFnBrand: 'a + ClonableFn, Brand: Semigroup + Sized>(
	a: Brand
) -> ApplyFn<'a, ClonableFnBrand, Brand, Brand> {
	Brand::append::<'a, ClonableFnBrand>(a)
}

// #[cfg(test)]
// mod tests {
// 	use crate::{brands::RcFnBrand, functions::append};

// 	#[test]
// 	fn test_string_semigroup() {
// 		let s1 = "Hello, ".to_string();
// 		let s2 = "World!".to_string();
// 		assert_eq!(append::<String, RcFnBrand>(s1)(s2), "Hello, World!");
// 	}

// 	#[test]
// 	fn test_string_semigroup_associativity() {
// 		let s1 = "a".to_string();
// 		let s2 = "b".to_string();
// 		let s3 = "c".to_string();

// 		// (a <> b) <> c = a <> (b <> c)
// 		let left_associated =
// 			append::<String, RcFnBrand>(append::<String, RcFnBrand>(s1.clone())(s2.clone()))(s3.clone());
// 		let right_associated =
// 			append::<String, RcFnBrand>(s1.clone())(append::<String, RcFnBrand>(s2.clone())(s3.clone()));

// 		assert_eq!(left_associated, right_associated);
// 		assert_eq!(left_associated, "abc");
// 	}
// }
