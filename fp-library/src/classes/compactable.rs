use crate::{Apply, brands::OptionBrand, kinds::*, types::Pair};

pub trait Compactable: Kind_c3c3610c70409ee6 {
	fn compact<'a, A: 'a>(
		fa: Apply!(
			brand: Self,
			signature: ('a, Apply!(brand: OptionBrand, signature: ('a, A: 'a) -> 'a): 'a) -> 'a,
		)
	) -> Apply!(
		brand: Self,
		signature: ('a, A: 'a) -> 'a,
	);

	fn separate<'a, E: 'a, O: 'a>(
		fa: Apply!(
			brand: Self,
			signature: ('a, Result<O, E>: 'a) -> 'a,
		)
	) -> Pair<
		Apply!(
			brand: Self,
			signature: ('a, E: 'a) -> 'a,
		),
		Apply!(
			brand: Self,
			signature: ('a, O: 'a) -> 'a,
		),
	>;
}

pub fn compact<'a, Brand: Compactable, A: 'a>(
	fa: Apply!(
		brand: Brand,
		signature: ('a, Apply!(brand: OptionBrand, signature: ('a, A: 'a) -> 'a): 'a) -> 'a,
	)
) -> Apply!(
	brand: Brand,
	signature: ('a, A: 'a) -> 'a,
) {
	Brand::compact(fa)
}

pub fn separate<'a, Brand: Compactable, E: 'a, O: 'a>(
	fa: Apply!(
		brand: Brand,
		signature: ('a, Result<O, E>: 'a) -> 'a,
	)
) -> Pair<
	Apply!(
		brand: Brand,
		signature: ('a, E: 'a) -> 'a,
	),
	Apply!(
		brand: Brand,
		signature: ('a, O: 'a) -> 'a,
	),
> {
	Brand::separate(fa)
}
