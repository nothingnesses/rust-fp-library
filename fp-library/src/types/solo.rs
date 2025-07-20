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

impl Pure for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::pure, types::Solo};
	///
	/// assert_eq!(pure::<SoloBrand, _>(()), Solo(()));
	/// ```
	fn pure<A>(a: A) -> Apply<Self, A> {
		Solo(a)
	}
}

impl Functor for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{identity, map}, types::Solo};
	///
	/// assert_eq!(map::<SoloBrand, _, _, _>(identity)(Solo(())), Solo(()));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(A) -> B,
	{
		move |fa| Self::inject(Solo(f(Self::project(fa).0)))
	}
}

impl Sequence for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{identity, sequence}, types::Solo};
	///
	/// assert_eq!(sequence::<SoloBrand, _, _, _>(Solo(identity))(Solo(())), Solo(()));
	/// ```
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<F> + Kind<A> + Kind<B>,
		F: Fn(A) -> B,
		Apply<Self, F>: Clone,
	{
		map::<Self, _, _, _>(<Self as Brand<Solo<F>, _>>::project(ff.to_owned()).0)
	}
}

impl Bind for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{bind, pure}, types::Solo};
	///
	/// assert_eq!(bind::<SoloBrand, _, _, _>(Solo(()))(pure::<SoloBrand, _>), Solo(()));
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
