use crate::{
	Apply,
	classes::{compactable::Compactable, functor::Functor},
	kinds::*,
	types::Pair,
};

pub trait Filterable: Compactable + Functor {
	fn partition_map<'a, Func, A: 'a, E: 'a, O: 'a>(
		func: Func,
		fa: Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, E>),
		Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, O>),
	>
	where
		Func: Fn(A) -> Result<O, E> + 'a;

	fn partition<'a, Func, A: 'a>(
		func: Func,
		fa: Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
		Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
	>
	where
		Func: Fn(A) -> bool;

	fn filter_map<'a, Func, A: 'a, B: 'a>(
		func: Func,
		fa: Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
	) -> Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, B>)
	where
		Func: Fn(A) -> Option<B> + 'a;

	fn filter<'a, Func, A: 'a>(
		func: Func,
		fa: Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
	) -> Apply!(<Self as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>)
	where
		Func: Fn(A) -> bool;
}

pub fn partition_map<'a, Brand: Filterable, Func, A: 'a, E: 'a, O: 'a>(
	func: Func,
	fa: Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
) -> Pair<
	Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, E>),
	Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, O>),
>
where
	Func: Fn(A) -> Result<O, E> + 'a,
{
	Brand::partition_map(func, fa)
}

pub fn partition<'a, Brand: Filterable, Func, A: 'a>(
	func: Func,
	fa: Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
) -> Pair<
	Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
	Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
>
where
	Func: Fn(A) -> bool,
{
	Brand::partition(func, fa)
}

pub fn filter_map<'a, Brand: Filterable, Func, A: 'a, B: 'a>(
	func: Func,
	fa: Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
) -> Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, B>)
where
	Func: Fn(A) -> Option<B> + 'a,
{
	Brand::filter_map(func, fa)
}

pub fn filter<'a, Brand: Filterable, Func, A: 'a>(
	func: Func,
	fa: Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>),
) -> Apply!(<Brand as trait { type Of<'a, T: 'a>: 'a; }>::Of<'a, A>)
where
	Func: Fn(A) -> bool,
{
	Brand::filter(func, fa)
}
