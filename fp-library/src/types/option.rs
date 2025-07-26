//! Implementations for [`Option`].

use crate::{
	brands::{Brand, Brand1},
	functions::map,
	hkt::{Apply, Kind, Kind1},
	typeclasses::{Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Functor, Pure},
};

/// [Brand][crate::brands] for [`Option`].
pub struct OptionBrand;

impl<A> Kind1<A> for OptionBrand {
	type Output = Option<A>;
}

impl<A> Brand1<Option<A>, A> for OptionBrand {
	fn inject(a: Option<A>) -> Apply<Self, (A,)> {
		a
	}
	fn project(a: Apply<Self, (A,)>) -> Option<A> {
		a
	}
}

impl Pure for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::pure};
	///
	/// assert_eq!(pure::<OptionBrand, _>(()), Some(()));
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>,
	{
		<Self as Brand<_, _>>::inject(Some(a))
	}
}

impl Functor for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{identity, map}};
	///
	/// assert_eq!(map::<OptionBrand, _, _, _>(identity::<()>)(None), None);
	/// assert_eq!(map::<OptionBrand, _, _, _>(identity)(Some(())), Some(()));
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
	{
		move |fa| <Self as Brand<_, _>>::inject(<Self as Brand<_, _>>::project(fa).map(&f))
	}
}

impl TypeclassApply for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{apply, identity}};
	///
	/// assert_eq!(apply::<OptionBrand, fn(()) -> (), _, _>(None)(None), None);
	/// assert_eq!(apply::<OptionBrand, fn(()) -> (), _, _>(None)(Some(())), None);
	/// assert_eq!(apply::<OptionBrand, _, _, _>(Some(identity::<()>))(None), None);
	/// assert_eq!(apply::<OptionBrand, _, _, _>(Some(identity))(Some(())), Some(()));
	/// ```
	fn apply<F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
		Apply<Self, (F,)>: Clone,
	{
		move |fa| match (<Self as Brand<_, _>>::project(ff.to_owned()), &fa) {
			(Some(f), _) => map::<Self, F, _, _>(f)(fa),
			_ => <Self as Brand<_, _>>::inject(None::<B>),
		}
	}
}

impl ApplyFirst for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{apply_first, identity}};
	///
	/// assert_eq!(apply_first::<OptionBrand, bool, bool>(None)(None), None);
	/// assert_eq!(apply_first::<OptionBrand, bool, _>(None)(Some(false)), None);
	/// assert_eq!(apply_first::<OptionBrand, _, bool>(Some(true))(None), None);
	/// assert_eq!(apply_first::<OptionBrand, _, _>(Some(true))(Some(false)), Some(true));
	/// ```
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |fb| {
			<Self as Brand<_, (A,)>>::inject(
				match (
					<Self as Brand<_, _>>::project(fa.to_owned()),
					<Self as Brand<_, (B,)>>::project(fb),
				) {
					(Some(a), Some(_)) => Some(a),
					_ => None,
				},
			)
		}
	}
}

impl ApplySecond for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{apply_second, identity}};
	///
	/// assert_eq!(apply_second::<OptionBrand, bool, bool>(None)(None), None);
	/// assert_eq!(apply_second::<OptionBrand, bool, _>(None)(Some(false)), None);
	/// assert_eq!(apply_second::<OptionBrand, _, bool>(Some(true))(None), None);
	/// assert_eq!(apply_second::<OptionBrand, _, _>(Some(true))(Some(false)), Some(false));
	/// ```
	fn apply_second<A, B>(_fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |fb| {
			<Self as Brand<_, (B,)>>::inject(
				match (
					<Self as Brand<_, (A,)>>::project(_fa.to_owned()),
					<Self as Brand<_, (B,)>>::project(fb),
				) {
					(Some(_), Some(a)) => Some(a),
					_ => None,
				},
			)
		}
	}
}

impl Bind for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{bind, pure}};
	///
	/// assert_eq!(bind::<OptionBrand, _, _, _>(None)(pure::<OptionBrand, ()>), None);
	/// assert_eq!(bind::<OptionBrand, _, _, _>(Some(()))(pure::<OptionBrand, _>), Some(()));
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, (A,)>) -> impl Fn(F) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Sized,
		F: Fn(A) -> Apply<Self, (B,)>,
		Apply<Self, (A,)>: Clone,
	{
		move |f| {
			<Self as Brand<_, _>>::inject(
				<Self as Brand<_, _>>::project(ma.to_owned())
					.and_then(|a| -> Option<B> { <Self as Brand<_, _>>::project(f(a)) }),
			)
		}
	}
}
