//! Macros for generating higher-kinded type traits and implementations.
//!
//! These macros provide a systematic way to generate traits for different kind arities,
//! allowing the library to work with types of varying complexity from concrete types
//! (kind `*`) to type constructors (kinds `* -> *`, `* -> * -> *`, etc.).

/// Generates a [`KindN` trait][crate::hkt::kinds] of a specific arity and its corresponding blanket implementation.
///
/// This macro creates traits that represent type-level applications for different kind arities.
/// Each generated trait has an `Output` associated type that represents the concrete type
/// produced when the brand is applied to the appropriate type parameters.
///
/// # Parameters
///
/// - `$KindN`: The name of the trait to generate (e.g., `Kind0`, `Kind1`, `Kind2`).
/// - `$ApplyN`: The corresponding type alias name (e.g., `Apply0`, `Apply1`, `Apply2`).
/// - `$kind_string`: A string representation of the kind (e.g., `"*"`, `"* -> *"`, `"* -> * -> *"`).
/// - `$Generics`: A tuple of generic type parameters (e.g., `()`, `(A)`, `(A, B)`).
#[macro_export]
macro_rules! make_trait_kind {
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		()
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub trait $KindN {
			type Output;
		}

		impl<Brand> Kind<()> for Brand
		where
			Brand: $KindN,
		{
			type Output = $ApplyN<Brand>;
		}
	};
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"Trait for [brands][crate::brands] of [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub trait $KindN<$($Generics),+> {
			type Output;
		}

		impl<Brand, $($Generics),+> Kind<($($Generics,)+)> for Brand
		where
			Brand: $KindN<$($Generics),+>,
		{
			type Output = $ApplyN<Brand, $($Generics),+>;
		}
	};
}

/// Generates an [`ApplyN` type alias][crate::hkt::apply] of a specific arity.
///
/// This macro creates type aliases that simplify the usage of kind traits by providing
/// a more convenient syntax for type applications. These aliases are used throughout
/// the library to make type signatures more readable.
///
/// # Parameters
///
/// - `$KindN`: The kind trait name (e.g., `Kind0`, `Kind1`, `Kind2`).
/// - `$ApplyN`: The name of the type alias to generate (e.g., `Apply0`, `Apply1`, `Apply2`).
/// - `$kind_string`: A string representation of the kind (e.g., `"*"`, `"* -> *"`, `"* -> * -> *"`).
/// - `$Generics`: A tuple of generic type parameters (e.g., `()`, `(A)`, `(A, B)`).
#[macro_export]
macro_rules! make_type_apply {
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		()
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub type $ApplyN<Brand> = <Brand as $KindN>::Output;
	};
	(
		$KindN:ident,
		$ApplyN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"Alias for [types][crate::types] of kind `",
			$kind_string,
			"`."
		)]
		pub type $ApplyN<Brand, $($Generics),+> = <Brand as $KindN<$($Generics),+>>::Output;
	};
}

/// Generates a [`BrandN` trait][crate::hkt::brands] of a specific arity and its corresponding blanket implementation.
///
/// This macro creates traits that provide `inject` and `project` methods which enable
/// bi-directional conversion between concrete types and their higher-kinded representations.
///
/// # Parameters
///
/// - `$BrandN`: The name of the brand trait to generate (e.g., `Brand0`, `Brand1`, `Brand2`).
/// - `$kind_string`: A string representation of the kind (e.g., `"*"`, `"* -> *"`, `"* -> * -> *"`).
/// - `$Generics`: A tuple of generic type parameters (e.g., `()`, `(A)`, `(A, B)`).
#[macro_export]
macro_rules! make_trait_brand {
	(
		$BrandN:ident,
		$kind_string:literal,
		()
	) => {
		#[doc = concat!(
			"[`BrandN` trait][crate::hkt::brands] for [types][crate::types] with kind `",
			$kind_string,
			"`."
		)]
		pub trait $BrandN<Concrete>
		where
			Self: Kind<()>,
		{
			fn inject(a: Concrete) -> Apply<Self, ()>;
			fn project(a: Apply<Self, ()>) -> Concrete;
		}

		impl<Me, Concrete> Brand<Concrete, ()> for Me
		where
			Me: Kind<()> + $BrandN<Concrete>,
		{
			fn inject(a: Concrete) -> Apply<Self, ()> {
				<Me as $BrandN<Concrete>>::inject(a)
			}

			fn project(a: Apply<Self, ()>) -> Concrete {
				<Me as $BrandN<Concrete>>::project(a)
			}
		}
	};
	(
		$BrandN:ident,
		$kind_string:literal,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"[`BrandN` trait][crate::hkt::brands] for [types][crate::types] with kind `",
			$kind_string,
			"`."
		)]
		pub trait $BrandN<Concrete, $($Generics),+>
		where
			Self: Kind<($($Generics,)+)>,
		{
			fn inject(a: Concrete) -> Apply<Self, ($($Generics,)+)>;
			fn project(a: Apply<Self, ($($Generics,)+)>) -> Concrete;
		}

		impl<Me, Concrete, $($Generics),+> Brand<Concrete, ($($Generics,)+)> for Me
		where
			Me: Kind<($($Generics,)+)> + $BrandN<Concrete, $($Generics),+>,
		{
			fn inject(a: Concrete) -> Apply<Self, ($($Generics,)+)> {
				<Me as $BrandN<Concrete, $($Generics),+>>::inject(a)
			}

			fn project(a: Apply<Self, ($($Generics,)+)>) -> Concrete {
				<Me as $BrandN<Concrete, $($Generics),+>>::project(a)
			}
		}
	};
}

/// Generates a [brand type][crate::brands] and its [`BrandN` trait][crate::hkt::brands] implementation.
///
/// This macro creates a concrete brand struct and implements the appropriate kind and brand traits
/// for it. It's the primary way to create brand types that connect concrete types with their
/// higher-kinded representations, enabling them to work with typeclasses.
///
/// # Parameters
///
/// - `$Brand`: The name of the brand struct to generate (e.g., `StringBrand`, `OptionBrand`).
/// - `$Concrete`: The concrete type this brand represents (e.g., `String`, `Option`).
/// - `$KindN`: The kind trait to implement (e.g., `Kind0`, `Kind1`, `Kind2`).
/// - `$BrandN`: The brand trait to implement (e.g., `Brand0`, `Brand1`, `Brand2`).
/// - `$Generics`: A tuple of generic type parameters (e.g., `()`, `(A)`, `(A, B)`).
#[macro_export]
macro_rules! impl_brand {
	(
		$Brand:ident,
		$Concrete:ident,
		$KindN:ident,
		$BrandN:ident,
		()
	) => {
		#[doc = concat!(
			"[Brand][crate::brands] for [`",
			stringify!($Concrete),
			"`]."
		)]
		pub struct $Brand;

		impl $KindN for $Brand {
			type Output = $Concrete;
		}

		impl $BrandN<$Concrete> for $Brand {
			fn inject(a: $Concrete) -> Apply<Self, ()> {
				a
			}

			fn project(a: Apply<Self, ()>) -> $Concrete {
				a
			}
		}
	};
	(
		$Brand:ident,
		$Concrete:ident,
		$KindN:ident,
		$BrandN:ident,
		($($Generics:ident),+)
	) => {
		#[doc = concat!(
			"[Brand][crate::brands] for [`",
			stringify!($Concrete),
			"`]."
		)]
		pub struct $Brand;

		impl<$($Generics),+> $KindN<$($Generics),+> for $Brand {
			type Output = $Concrete<$($Generics),+>;
		}

		impl<$($Generics),+> $BrandN<$Concrete<$($Generics),+>, $($Generics,)+> for $Brand {
			fn inject(a: $Concrete<$($Generics),+>) -> Apply<Self, ($($Generics,)+)> {
				a
			}

			fn project(a: Apply<Self, ($($Generics,)+)>) -> $Concrete<$($Generics),+> {
				a
			}
		}
	}
}
