use crate::{
	brands::Brand,
	functions::map,
	hkt::{Apply, Kind},
	typeclasses::{Bind, Empty, Functor, Pure, Sequence},
};

pub struct OptionBrand;

impl<A> Kind<A> for OptionBrand {
	type Output = Option<A>;
}

impl<A> Brand<Option<A>, A> for OptionBrand {
	fn inject(a: &Option<A>) -> &Apply<Self, A> {
		a
	}
	fn project(a: &Apply<Self, A>) -> &Option<A> {
		a
	}
}

impl Bind for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::bind};
	///
	/// let zero = Some(0);
	/// let add_one = |a: &_| Some(a + 1);
	/// assert_eq!(bind::<OptionBrand, _, _, _>(&zero)(&add_one), Some(1));
	/// ```
	fn bind<F, A, B>(ma: &Apply<Self, A>) -> impl Fn(&F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(&A) -> Apply<Self, B>,
		Apply<Self, B>: Clone,
	{
		|f| match OptionBrand::project(ma) {
			Some(a) => f(a),
			_ => (*<OptionBrand as Brand<Option<B>, _>>::inject(&None)).to_owned(),
		}
	}
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
