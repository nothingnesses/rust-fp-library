use crate::{
	Apply,
	classes::{applicative::Applicative, filterable::Filterable, traversable::Traversable},
	kinds::*,
	types::Pair,
};

pub trait Witherable: Filterable + Traversable {
	fn wilt<'a, Func, M: Applicative, A: 'a, E: 'a, O: 'a>(
		func: Func,
		ta: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> Apply!(
		brand: M,
		signature: (
			'a, Pair<
				Apply!(brand: Self, signature: ('a, E: 'a) -> 'a),
				Apply!(brand: Self, signature: ('a, O: 'a) -> 'a)
			>: 'a
		) -> 'a,
	)
	where
		Func: Fn(A) -> Apply!(brand: M, signature: ('a, Result<O, E>: 'a) -> 'a);

	fn wither<'a, Func, M: Applicative, A: 'a, B: 'a>(
		func: Func,
		ta: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> Apply!(
		brand: M,
		signature: (
			'a, Apply!(
				brand: Self,
				signature: ('a, B: 'a) -> 'a,
			): 'a
		) -> 'a,
	)
	where
		Func: Fn(A) -> Apply!(brand: M, signature: ('a, Option<B>: 'a) -> 'a);
}

pub fn wilt<'a, Brand: Witherable, Func, M: Applicative, A: 'a, E: 'a, O: 'a>(
	func: Func,
	ta: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
) -> Apply!(
	brand: M,
	signature: (
		'a, Pair<
			Apply!(brand: Brand, signature: ('a, E: 'a) -> 'a),
			Apply!(brand: Brand, signature: ('a, O: 'a) -> 'a)
		>: 'a
	) -> 'a,
)
where
	Func: Fn(A) -> Apply!(brand: M, signature: ('a, Result<O, E>: 'a) -> 'a),
{
	Brand::wilt::<_, M, _, _, _>(func, ta)
}

pub fn wither<'a, Brand: Witherable, Func, M: Applicative, A: 'a, B: 'a>(
	func: Func,
	ta: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
) -> Apply!(
	brand: M,
	signature: (
		'a, Apply!(
			brand: Brand,
			signature: ('a, B: 'a) -> 'a,
		): 'a
	) -> 'a,
)
where
	Func: Fn(A) -> Apply!(brand: M, signature: ('a, Option<B>: 'a) -> 'a),
{
	Brand::wither::<_, M, _, _>(func, ta)
}
