/// Represents types with an associative binary operation.
pub trait Semigroup<A> {
	/// Associative operation that combines two values of the same type.
	///
	/// forall a. Semigroup a => a -> a -> a
	fn append(
		a: A,
		b: A,
	) -> A;
}

/// Associative operation that combines two values of the same type.
///
/// Free function version that dispatches to the typeclass method.
///
/// forall a. Semigroup a => a -> a -> a
pub fn append<Brand, A>(
	a: A,
	b: A,
) -> A
where
	Brand: Semigroup<A>,
{
	Brand::append(a, b)
}

#[cfg(test)]
mod tests {
	use crate::{brands::StringBrand, functions::append};

	#[test]
	fn test_string_semigroup() {
		let s1 = "Hello, ".to_string();
		let s2 = "World!".to_string();
		let result = append::<StringBrand, _>(s1, s2);
		assert_eq!(result, "Hello, World!");
	}

	#[test]
	fn test_string_semigroup_associativity() {
		let s1 = "a".to_string();
		let s2 = "b".to_string();
		let s3 = "c".to_string();

		// (a <> b) <> c = a <> (b <> c)
		let left_associated =
			append::<StringBrand, _>(append::<StringBrand, _>(s1.clone(), s2.clone()), s3.clone());
		let right_associated =
			append::<StringBrand, _>(s1.clone(), append::<StringBrand, _>(s2.clone(), s3.clone()));

		assert_eq!(left_associated, right_associated);
		assert_eq!(left_associated, "abc");
	}
}
