//! Implementations for [`Option`].

use crate::{
	functions::{map, pure},
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable,
		Functor, Pure, Traversable, clonable_fn::ApplyFn,
	},
};

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
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::{identity, map}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, OptionBrand, _, _>(Rc::new(identity::<()>))(None),
	///     None
	/// );
	/// assert_eq!(
	///     map::<RcFnBrand, OptionBrand, _, _>(Rc::new(identity))(Some(())),
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
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::{apply, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, OptionBrand, (), ()>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, OptionBrand, (), ()>(None)(Some(())),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, OptionBrand, (), _>(Some(Rc::new(identity::<()>)))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply::<RcFnBrand, OptionBrand, (), _>(Some(Rc::new(identity)))(Some(())),
	///     Some(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match (ff.to_owned(), &fa) {
			(Some(f), _) => map::<ClonableFnBrand, Self, _, _>(f)(fa),
			_ => None::<B>,
		})
	}
}

impl ApplyFirst for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::{apply_first, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, OptionBrand, bool, bool>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, OptionBrand, bool, _>(None)(Some(false)),
	///     None
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, OptionBrand, _, bool>(Some(true))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_first::<RcFnBrand, OptionBrand, _, _>(Some(true))(Some(false)),
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
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::{apply_second, identity}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, OptionBrand, bool, bool>(None)(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, OptionBrand, bool, _>(None)(Some(false)),
	///     None
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, OptionBrand, _, bool>(Some(true))(None),
	///     None
	/// );
	/// assert_eq!(
	///     apply_second::<RcFnBrand, OptionBrand, _, _>(Some(true))(Some(false)),
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
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::{bind, pure}};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, OptionBrand, _, _>(None)(Rc::new(pure::<OptionBrand, ()>)),
	///     None
	/// );
	/// assert_eq!(
	///     bind::<RcFnBrand, OptionBrand, _, _>(Some(()))(Rc::new(pure::<OptionBrand, _>)),
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
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::fold_right};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, OptionBrand, _, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(Some(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<RcFnBrand, OptionBrand, i32, _>(Rc::new(|a| Rc::new(move |b| a + b)))(1)(None),
	///     1
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
				let f = f.clone();
				move |fa| match (f.clone(), b.to_owned(), fa) {
					(_, b, None) => b,
					(f, b, Some(a)) => f(a)(b),
				}
			})
		})
	}
}

impl Traversable for OptionBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{OptionBrand, RcFnBrand}, functions::traverse};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, OptionBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Some(3)),
	///     Some(Some(6))
	/// );
	/// assert_eq!(
	///     traverse::<RcFnBrand, OptionBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(None),
	///     Some(None)
	/// );
	/// ```
	fn traverse<
		'a,
		ClonableFnBrand: 'a + ClonableFn,
		F: Applicative,
		A: 'a + Clone,
		B: 'a + Clone,
	>(
		f: ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<F, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: Clone,
		Apply0L1T<F, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
		Apply0L1T<Self, B>: 'a,
		Apply0L1T<Self, Apply0L1T<F, B>>: 'a,
	{
		ClonableFnBrand::new(move |ta: Apply0L1T<Self, _>| match (f.clone(), ta) {
			(_, None) => pure::<F, _>(None),
			(f, Some(a)) => {
				map::<ClonableFnBrand, F, B, _>(ClonableFnBrand::new(pure::<Self, _>))(f(a))
			}
		})
	}
}
