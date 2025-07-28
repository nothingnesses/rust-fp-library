//! Macros.

/// Generates a [`KindN` trait][crate::hkt::kinds] of a specific arity and its corresponding blanket implementation.
#[macro_export]
macro_rules! make_trait_kind {

	(
		// Kind trait name (e.g., Kind2).
		$KindN:ident,
		// Apply type alias name (e.g., Apply2).
		$ApplyN:ident,
		// String representation of the kind (e.g., "* -> * -> *").
		$kind_string:literal,
		// List of generic type parameters (e.g., (A, B)).
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
		// Kind trait name (e.g., Kind2).
		$KindN:ident,
		// Apply type alias name (e.g., Apply2).
		$ApplyN:ident,
		// String representation of the kind (e.g., "* -> * -> *").
		$kind_string:literal,
		// List of generic type parameters (e.g., (A, B)).
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
#[macro_export]
macro_rules! make_type_apply {
	(
		// Kind trait name (e.g., Kind2).
		$KindN:ident,
		// Apply type alias name (e.g., Apply2).
		$ApplyN:ident,
		// String representation of the kind (e.g., "* -> * -> *").
		$kind_string:literal,
		// List of generic type parameters (e.g., (A, B)).
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
		// Kind trait name (e.g., Kind2).
		$KindN:ident,
		// Apply type alias name (e.g., Apply2).
		$ApplyN:ident,
		// String representation of the kind (e.g., "* -> * -> *").
		$kind_string:literal,
		// List of generic type parameters (e.g., (A, B)).
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
#[macro_export]
macro_rules! make_trait_brand {
	(
		// Brand trait name (e.g., Brand2).
		$BrandN:ident,
		// String representation of the kind (e.g., "* -> * -> *").
		$kind_string:literal,
		// List of generic type parameters (e.g., (A, B)).
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
		// Brand trait name (e.g., Brand2).
		$BrandN:ident,
		// String representation of the kind (e.g., "* -> * -> *").
		$kind_string:literal,
		// List of generic type parameters (e.g., (A, B)).
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
#[macro_export]
macro_rules! impl_brand {
	(
		// Brand type name (e.g., StringBrand).
		$Brand:ident,
		// Concrete type name (e.g., String).
		$Concrete:ident,
		// Kind trait name (e.g., Kind0).
		$KindN:ident,
		// Brand trait name (e.g., Brand0).
		$BrandN:ident,
		// List of generic type parameters (e.g., ()).
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
		// Brand type name (e.g., PairBrand).
		$Brand:ident,
		// Concrete type name (e.g., Pair).
		$Concrete:ident,
		// Kind trait name (e.g., Kind2).
		$KindN:ident,
		// Brand trait name (e.g., Brand2).
		$BrandN:ident,
		// List of generic type parameters (e.g., (A, B)).
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
