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
	fn inject(a: Option<A>) -> Apply<Self, A> {
		a
	}
	fn project(a: Apply<Self, A>) -> Option<A> {
		a
	}
}

impl Empty for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::empty};
	///
	/// assert_eq!(empty::<OptionBrand, ()>(), None);
	fn empty<A>() -> Apply<Self, A> {
		None
	}
}

impl Pure for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::pure};
	///
	/// assert_eq!(pure::<OptionBrand, _>(()), Some(()));
	fn pure<A>(a: A) -> Apply<Self, A>
	where
		Self: Kind<A>,
	{
		Self::inject(Some(a))
	}
}

impl Functor for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{identity, map}};
	///
	/// assert_eq!(map::<OptionBrand, _, _, _>(identity)(Some(())), Some(()));
	/// assert_eq!(map::<OptionBrand, _, _, _>(identity::<()>)(None), None);
	/// ```
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(A) -> B,
	{
		move |fa| Self::inject(Self::project(fa).map(&f))
	}
}

impl Sequence for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{identity, sequence}};
	///
	/// assert_eq!(sequence::<OptionBrand, _, _, _>(Some(identity))(Some(())), Some(()));
	/// assert_eq!(sequence::<OptionBrand, _, _, _>(Some(identity::<()>))(None), None);
	/// assert_eq!(sequence::<OptionBrand, fn(()) -> (), _, _>(None)(Some(())), None);
	/// assert_eq!(sequence::<OptionBrand, fn(()) -> (), _, _>(None)(None), None);
	/// ```
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<F> + Kind<A> + Kind<B>,
		F: Fn(A) -> B,
		Apply<Self, F>: Clone,
	{
		move |fa| match (Self::project(ff.to_owned()), &fa) {
			(Some(f), _) => map::<Self, F, _, _>(f)(fa),
			_ => Self::inject(None::<B>),
		}
	}
}

impl Bind for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{bind, pure}};
	///
	/// assert_eq!(bind::<OptionBrand, _, _, _>(Some(()))(pure::<OptionBrand, _>), Some(()));
	/// assert_eq!(bind::<OptionBrand, _, _, _>(None)(pure::<OptionBrand, ()>), None);
	/// ```
	fn bind<F, A, B>(ma: Apply<Self, A>) -> impl Fn(F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B> + Sized,
		F: Fn(A) -> Apply<Self, B>,
		Apply<Self, A>: Clone,
	{
		move |f| {
			Self::inject(
				Self::project(ma.to_owned()).and_then(|a| -> Option<B> { Self::project(f(a)) }),
			)
		}
	}
}
