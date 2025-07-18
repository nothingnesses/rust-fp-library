use crate::{
	functions::map,
	hkt::{Apply, Kind},
	typeclasses::{Empty, Functor, Pure, Sequence},
};

pub struct OptionBrand;

impl<A> Kind<A> for OptionBrand {
	type Output = Option<A>;
}

impl Functor for OptionBrand {
	fn map<F, A, B>(f: &F) -> impl Fn(&Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(&A) -> B,
	{
		move |fa| fa.as_ref().map(f)
	}
}

impl Pure for OptionBrand {
	fn pure<A>(a: &A) -> Apply<Self, A>
	where
		A: Clone,
	{
		Some(a.to_owned())
	}
}

impl Empty for OptionBrand {
	fn empty<A>() -> Apply<Self, A> {
		None
	}
}

impl Sequence for OptionBrand {
	fn sequence<F, A, B>(ff: &Apply<Self, F>) -> impl Fn(&Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(&A) -> B,
		A: Clone,
	{
		move |fa| match (ff, fa) {
			(Some(f), _) => map::<Self, _, _, _>(&f)(fa),
			_ => None,
		}
	}
}
