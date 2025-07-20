use crate::{
	brands::Brand,
	functions::map,
	hkt::{Apply, Kind},
	typeclasses::{Bind, Functor, Pure, Sequence},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

pub struct SoloBrand;

impl<A> Kind<A> for SoloBrand {
	type Output = Solo<A>;
}

impl<A> Brand<Solo<A>, A> for SoloBrand {
	fn inject(a: Solo<A>) -> Apply<Self, A> {
		a
	}
	fn project(a: Apply<Self, A>) -> Solo<A> {
		a
	}
}

impl Bind for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::bind, types::Solo};
	///
	/// let zero = Solo(0);
	/// let add_one = |a: &_| Solo(a + 1);
	/// assert_eq!(bind::<SoloBrand, _, _, _>(&zero)(&add_one), Solo(1));
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, A>) -> impl Fn(F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B> + Sized,
		F: Fn(A) -> Apply<Self, B>,
		Apply<Self, A>: Clone,
	{
		move |f| f(Self::project(ma.to_owned()).0)
	}
}

impl Pure for SoloBrand {
	fn pure<A>(a: A) -> Apply<Self, A> {
		Solo(a)
	}
}

impl Functor for SoloBrand {
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(A) -> B,
	{
		move |fa| Self::inject(Solo(f(Self::project(fa).0)))
	}
}

impl Sequence for SoloBrand {
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<F> + Kind<A> + Kind<B>,
		F: Fn(A) -> B,
		Apply<Self, F>: Clone,
	{
		map::<Self, _, _, _>(<Self as Brand<Solo<F>, _>>::project(ff.to_owned()).0)
	}
}
