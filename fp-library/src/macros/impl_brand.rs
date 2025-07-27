//! Contains the macro to generate a [brand type][crate::brands] and its
//! `BrandN` trait implementation.

/// Generates a [brand type][crate::brands] and its `BrandN` trait implementation.
#[macro_export]
macro_rules! impl_brand {
	(
		// Brand type documentation comment.
		$(#[$($attrss:tt)*])*
		// Brand type name (e.g., PairBrand).
		$Brand:ident,
		// Concrete type name (e.g., Pair).
		$Concrete:ident,
		// Kind trait name (e.g., Kind2).
		$KindN:ident,
		// Brand trait name (e.g., Brand2).
		$BrandN:ident,
		// List of generic type parameters (e.g., (A, B)).
		($($T:ident),+)
	) => {
		$(#[$($attrss)*])*
		pub struct $Brand;

		impl<$($T),+> $KindN<$($T),+> for $Brand {
			type Output = $Concrete<$($T),+>;
		}

		impl<$($T),+> $BrandN<$Concrete<$($T),+>, $($T,)+> for $Brand {
			fn inject(a: $Concrete<$($T),+>) -> Apply<Self, ($($T,)+)> {
				a
			}

			fn project(a: Apply<Self, ($($T,)+)>) -> $Concrete<$($T),+> {
				a
			}
		}
	}
}
