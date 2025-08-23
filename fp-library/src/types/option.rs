//! Implementations for [`Option`].

use crate::{
	aliases::ArcFn,
	functions::{map, pure},
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable,
		Functor, Pure, Traversable, clonable_fn::ApplyFn,
	},
};
use std::sync::Arc;

pub struct OptionBrand;

impl Kind0L1T for OptionBrand {
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
	fn pure<A>(a: A) -> Apply0L1T<Self, A> {
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
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| fa.map(&*f))
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
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match (ff.to_owned(), &fa) {
			(Some(f), _) => map::<Self, ClonableFnBrand, _, _>(f)(fa),
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
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |fb| match (fa.to_owned(), fb) {
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
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fb| match (fa.to_owned(), fb) {
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
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     bind::<OptionBrand, _, _>(None)(Arc::new(pure::<OptionBrand, ()>)),
	///     None
	/// );
	/// assert_eq!(
	///     bind::<OptionBrand, _, _>(Some(()))(Arc::new(pure::<OptionBrand, _>)),
	///     Some(())
	/// );
	/// ```
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		ma: Apply0L1T<Self, A>
	) -> ApplyFn<
		'a,
		ClonableFnBrand,
		ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	> {
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, _, _>| {
			ma.to_owned().and_then(|a| -> Option<B> { f(a) })
		})
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
	) -> ArcFn<'a, B, ArcFn<'a, Apply0L1T<Self, A>, B>> {
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
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{OptionBrand}, functions::traverse};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     traverse::<OptionBrand, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Some(3)),
	///     Some(Some(6))
	/// );
	/// assert_eq!(
	///     traverse::<OptionBrand, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(None),
	///     Some(None)
	/// );
	/// ```
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply0L1T<F, B>>
	) -> ArcFn<'a, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: 'a + Clone,
		Apply0L1T<F, ArcFn<'a, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
	{
		Arc::new(move |ta| match (f.clone(), ta) {
			(_, None) => pure::<F, _>(None),
			(f, Some(a)) => map::<F, B, _>(Arc::new(pure::<Self, _>))(f(a)),
		})
	}
}
