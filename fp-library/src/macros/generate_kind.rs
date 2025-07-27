//! Contains the macro to generate boilerplate for [HKTs][crate::hkt] and the generated code.

use crate::{
	brands::Brand,
	hkt::{Apply, Kind},
};

/// Generates boilerplate for [HKTs][crate::hkt] of a specific arity.
#[macro_export]
macro_rules! generate_kind {
	(
		// Kind trait name (e.g., Kind1).
		$KindN:ident,
		// Apply type alias name (e.g., Apply1).
		$ApplyN:ident,
		// Brand trait name (e.g., Brand1).
		$BrandN:ident,
		// String representation of the kind (e.g., "* -> *").
		$kind_str:literal,
		// List of generic type parameters (e.g., (A, B)).
		($($T:ident),+)
	) => {
		#[doc = "Trait for [brands][crate::brands] of [types][crate::types] of kind `"]
		#[doc = stringify!($kind_str)]
		#[doc = "`."]
		pub trait $KindN<$($T),+> {
			type Output;
		}

		#[doc = "Alias for [types][crate::types] of kind `"]
		#[doc = stringify!($kind_str)]
		#[doc = "`."]
		pub type $ApplyN<Brand, $($T),+> = <Brand as $KindN<$($T),+>>::Output;

		impl<Brand, $($T),+> Kind<($($T,)+)> for Brand
		where
			Brand: $KindN<$($T),+>,
		{
			type Output = $ApplyN<Brand, $($T),+>;
		}

		#[doc = "Brand trait for [types][crate::types] with kind `"]
		#[doc = stringify!($kind_str)]
		#[doc = "`."]
		pub trait $BrandN<Concrete, $($T),+>
		where
			Self: Kind<($($T,)+)>,
		{
			fn inject(a: Concrete) -> Apply<Self, ($($T,)+)>;
			fn project(a: Apply<Self, ($($T,)+)>) -> Concrete;
		}

		impl<Me, Concrete, $($T),+> Brand<Concrete, ($($T,)+)> for Me
		where
			Me: Kind<($($T,)+)> + $BrandN<Concrete, $($T),+>,
		{
			fn inject(a: Concrete) -> Apply<Self, ($($T,)+)> {
				<Me as $BrandN<Concrete, $($T),+>>::inject(a)
			}

			fn project(a: Apply<Self, ($($T,)+)>) -> Concrete {
				<Me as $BrandN<Concrete, $($T),+>>::project(a)
			}
		}
	};
}

generate_kind!(Kind1, Apply1, Brand1, "* -> *", (A));
generate_kind!(Kind2, Apply2, Brand2, "* -> * -> *", (A, B));
generate_kind!(Kind3, Apply3, Brand3, "* -> * -> * -> *", (A, B, C));
generate_kind!(Kind4, Apply4, Brand4, "* -> * -> * -> * -> *", (A, B, C, D));
