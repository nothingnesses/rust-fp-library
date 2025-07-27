//! Implementations for [`Solo`], a type that wraps a value.

use crate::{
	brands::{Brand, Brand1},
	functions::{identity, map},
	hkt::{Apply, Kind, Kind1},
	impl_brand,
	typeclasses::{Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Functor, Pure},
};

/// Wraps a value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

impl_brand!(
	/// [Brand][crate::brands] for [`Solo`].
	SoloBrand,
	Solo,
	Kind1,
	Brand1,
	(A)
);

impl Pure for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::pure, types::Solo};
	///
	/// assert_eq!(pure::<SoloBrand, _>(()), Solo(()));
	/// ```
	fn pure<A>(a: A) -> Apply<Self, (A,)> {
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
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| <Self as Brand<_, _>>::inject(Solo(f(<Self as Brand<_, _>>::project(fa).0)))
	}
}

impl TypeclassApply for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply, identity}, types::Solo};
	///
	/// assert_eq!(apply::<SoloBrand, _, _, _>(Solo(identity))(Solo(())), Solo(()));
	/// ```
	fn apply<F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
		Apply<Self, (F,)>: Clone,
	{
		map::<Self, _, _, _>(<Self as Brand<Solo<F>, _>>::project(ff.to_owned()).0)
	}
}

impl ApplyFirst for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply_first, identity}, types::Solo};
	///
	/// assert_eq!(apply_first::<SoloBrand, _, _>(Solo(true))(Solo(false)), Solo(true));
	/// ```
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |_fb| fa.to_owned()
	}
}

impl ApplySecond for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply_second, identity}, types::Solo};
	///
	/// assert_eq!(apply_second::<SoloBrand, _, _>(Solo(true))(Solo(false)), Solo(false));
	/// ```
	fn apply_second<A, B>(_fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
	{
		identity
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
	fn bind<F, A, B>(ma: Apply<Self, (A,)>) -> impl Fn(F) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Sized,
		F: Fn(A) -> Apply<Self, (B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |f| f(<Self as Brand<_, _>>::project(ma.to_owned()).0)
	}
}
