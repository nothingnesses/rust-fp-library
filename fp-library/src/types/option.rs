//! Implementations for [`Option`].

use crate::{
	aliases::ClonableFn,
	functions::map,
	hkt::{Apply, Brand, Brand1, Kind, Kind1},
	impl_brand,
	typeclasses::{
		Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Foldable, Functor, Pure,
	},
};
use std::sync::Arc;

impl_brand!(OptionBrand, Option, Kind1, Brand1, (A));

impl Pure for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::pure};
	///
	/// assert_eq!(
	///     pure::<OptionBrand, _>(()),
	///     Some(())
	/// );
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
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<OptionBrand, _, _>(Arc::new(identity::<()>))(None),
	///     None
	/// );
	/// assert_eq!(
	///     map::<OptionBrand, _, _>(Arc::new(identity))(Some(())),
	///     Some(())
	/// );
	/// ```
	fn map<'a, A, B>(f: ClonableFn<'a, A, B>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
	{
		move |fa| <Self as Brand<_, _>>::inject(<Self as Brand<_, _>>::project(fa).map(&*f))
	}
}

impl TypeclassApply for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{apply, identity}};
	///
	/// assert_eq!(
	///     apply::<OptionBrand, fn(()) -> (), _, _>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<OptionBrand, fn(()) -> (), _, _>(None)(Some(())),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<OptionBrand, _, _, _>(Some(identity::<()>))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<OptionBrand, _, _, _>(Some(identity))(Some(())),
	///     Some(())
	/// );
	/// ```
	fn apply<'a, F, A, B>(ff: Apply<Self, (F,)>) -> impl Fn(Apply<Self, (A,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: 'a + Fn(A) -> B,
		Apply<Self, (F,)>: Clone,
	{
		move |fa| match (<Self as Brand<_, (F,)>>::project(ff.to_owned()), &fa) {
			(Some(f), _) => map::<Self, _, _>(Arc::new(f))(fa),
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
	/// assert_eq!(
	///     apply_first::<OptionBrand, bool, bool>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_first::<OptionBrand, bool, _>(None)(Some(false)),
	///     None
	/// );
	/// assert_eq!(
	///     apply_first::<OptionBrand, _, bool>(Some(true))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_first::<OptionBrand, _, _>(Some(true))(Some(false)),
	///     Some(true)
	/// );
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
	/// assert_eq!(
	///     apply_second::<OptionBrand, bool, bool>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_second::<OptionBrand, bool, _>(None)(Some(false)),
	///     None
	/// );
	/// assert_eq!(
	///     apply_second::<OptionBrand, _, bool>(Some(true))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_second::<OptionBrand, _, _>(Some(true))(Some(false)),
	///     Some(false)
	/// );
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
	/// assert_eq!(
	///     bind::<OptionBrand, _, _, _>(None)(pure::<OptionBrand, ()>),
	///     None
	/// );
	/// assert_eq!(
	///     bind::<OptionBrand, _, _, _>(Some(()))(pure::<OptionBrand, _>),
	///     Some(())
	/// );
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

impl Foldable for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::fold_right};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<OptionBrand, _, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Some(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<OptionBrand, i32, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(None),
	///     1
	/// );
	/// ```
	fn fold_right<'a, A, B>(
		f: ClonableFn<'a, A, ClonableFn<'a, B, B>>
	) -> ClonableFn<'a, B, ClonableFn<'a, Apply<Self, (A,)>, B>>
	where
		Self: 'a + Kind<(A,)>,
		A: 'a + Clone,
		B: 'a + Clone,
		Apply<Self, (A,)>: 'a,
	{
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| match (f.clone(), b.to_owned(), <OptionBrand as Brand<_, _>>::project(fa))
				{
					(_, b, None) => b,
					(f, b, Some(a)) => f(a)(b),
				}
			})
		})
	}
}
