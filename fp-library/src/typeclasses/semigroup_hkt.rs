use crate::hkt::{Apply, Kind};

pub trait Semigroup {
	/// Associative operation that combines two values of the same type
	/// forall a. Semigroup a => a -> a -> a
	fn append(
		a: Apply<Self, ()>,
		b: Apply<Self, ()>,
	) -> Apply<Self, ()>
	where
		Self: Kind<()>;
}

/// Free function version that dispatches to the typeclass method
/// forall a. Semigroup a => a -> a -> a
pub fn append<Brand>(
	a: Apply<Brand, ()>,
	b: Apply<Brand, ()>,
) -> Apply<Brand, ()>
where
	Brand: Kind<()> + Semigroup,
{
	Brand::append(a, b)
}

#[cfg(test)]
mod tests {
	use crate::{hkt::Brand, types::StringBrandHKT};

	#[test]
	fn test_string_semigroup() {
		let s1 = <StringBrandHKT as Brand<String, ()>>::inject("Hello, ".to_string());
		let s2 = <StringBrandHKT as Brand<String, ()>>::inject("World!".to_string());
		let result = super::append::<StringBrandHKT>(s1, s2);
		let string_result = <StringBrandHKT as Brand<String, ()>>::project(result);
		assert_eq!(string_result, "Hello, World!");
	}
}
