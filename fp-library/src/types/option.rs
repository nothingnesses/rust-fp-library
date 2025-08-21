//! Implementations for [`Option`].

use crate::{
	aliases::ArcFn,
	functions::{map, pure},
	hkt::{Apply1, Kind1},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Foldable, Functor,
		Pure, Traversable,
	},
};
use std::sync::Arc;

pub struct OptionBrand;

impl Kind1 for OptionBrand {
	type Output<A> = Option<A>;
}

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
	fn pure<A>(a: A) -> Apply1<Self, A> {
		Some(a)
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
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa: Apply1<Self, A>| fa.map(&*f))
	}
}

impl TypeclassApply for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::OptionBrand, functions::{apply, identity}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     apply::<OptionBrand, (), ()>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<OptionBrand, (), ()>(None)(Some(())),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<OptionBrand, (), ()>(Some(Arc::new(identity::<()>)))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<OptionBrand, (), ()>(Some(Arc::new(identity)))(Some(())),
	///     Some(())
	/// );
	/// ```
	fn apply<'a, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, ArcFn<'a, A, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, ArcFn<'a, A, B>>: Clone,
	{
		Arc::new(move |fa| match (ff.to_owned(), &fa) {
			(Some(f), _) => map::<Self, _, _>(f)(fa),
			_ => None::<B>,
		})
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
	fn apply_first<'a, A: 'a + Clone, B>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, A>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| match (fa.to_owned(), fb) {
			(Some(a), Some(_)) => Some(a),
			_ => None,
		})
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
	fn apply_second<'a, A: 'a, B: 'a + Clone>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |fb| match (fa.to_owned(), fb) {
			(Some(_), Some(a)) => Some(a),
			_ => None,
		})
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
	fn bind<'a, F: Fn(A) -> Apply1<Self, B>, A: 'a + Clone, B>(
		ma: Apply1<Self, A>
	) -> ArcFn<'a, F, Apply1<Self, B>> {
		Arc::new(move |f| ma.to_owned().and_then(|a| -> Option<B> { f(a) }))
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
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply1<Self, A>, B>> {
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| match (f.clone(), b.to_owned(), fa) {
					(_, b, None) => b,
					(f, b, Some(a)) => f(a)(b),
				}
			})
		})
	}
}

impl<'a> Traversable<'a> for OptionBrand {
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a + Clone,
		Apply1<F, ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>>: Clone,
	{
		Arc::new(move |ta| match (f.clone(), ta) {
			(_, None) => pure::<F, _>(None),
			(f, Some(a)) => map::<F, B, _>(Arc::new(pure::<Self, _>))(f(a)),
		})
	}
}
