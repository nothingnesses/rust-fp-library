use crate::{
	Apply,
	classes::{applicative::Applicative, filterable::Filterable, traversable::Traversable},
	kinds::*,
	types::Pair,
};

pub trait Witherable: Filterable + Traversable {
	fn wilt<'a, Func, M: Applicative, A: 'a, E: 'a, O: 'a>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Pair<
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		>,
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>);

	fn wither<'a, Func, M: Applicative, A: 'a, B: 'a>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>);
}

pub fn wilt<'a, Brand: Witherable, Func, M: Applicative, A: 'a, E: 'a, O: 'a>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'a,
	Pair<
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	>,
>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>),
{
	Brand::wilt::<_, M, _, _, _>(func, ta)
}

pub fn wither<'a, Brand: Witherable, Func, M: Applicative, A: 'a, B: 'a>(
	func: Func,
	ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
	'a,
	Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
>)
where
	Func: Fn(A) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Option<B>>),
{
	Brand::wither::<_, M, _, _>(func, ta)
}
