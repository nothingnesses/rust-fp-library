//! Implementations for [`Solo`], a type that wraps a value.

use crate::{
	functions::{identity, map, pure},
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, Apply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable, Functor, Pure,
		Traversable, clonable_fn::ApplyFn,
	},
};

/// Wraps a value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

pub struct SoloBrand;

impl Kind0L1T for SoloBrand {
	type Output<A> = Solo<A>;
}

impl Functor for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, RcFnBrand}, functions::{identity, map}, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, SoloBrand, _, _>(Rc::new(identity))(Solo(())),
	///     Solo(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| Solo(f(fa.0)))
	}
}

impl Apply for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, RcFnBrand}, functions::{apply, identity}, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, SoloBrand, _, _>(Solo(Rc::new(identity)))(Solo(())),
	///     Solo(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		map::<ClonableFnBrand, Self, A, B>(ff.0)
	}
}

impl ApplyFirst for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, RcFnBrand}, functions::{apply_first, identity}, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, SoloBrand, _, _>(Solo(true))(Solo(false)),
	///     Solo(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |_fb| fa.to_owned())
	}
}

impl ApplySecond for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, RcFnBrand}, functions::{apply_second, identity}, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, SoloBrand, _, _>(Solo(true))(Solo(false)),
	///     Solo(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		_fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(identity)
	}
}

impl Pure for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{RcFnBrand, SoloBrand}, functions::pure, types::Solo};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, SoloBrand, _>(()),
	///     Solo(())
	/// );
	/// ```
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		Solo(a)
	}
}

impl Bind for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, RcFnBrand}, functions::{bind, pure}, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, SoloBrand, _, _>(Solo(()))(Rc::new(pure::<RcFnBrand, SoloBrand, _>)),
	///     Solo(())
	/// );
	/// ```
	fn bind<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		ma: Apply0L1T<Self, A>
	) -> ApplyFn<
		'a,
		ClonableFnBrand,
		ApplyFn<'a, ClonableFnBrand, A, Apply0L1T<Self, B>>,
		Apply0L1T<Self, B>,
	> {
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, _, _>| f(ma.to_owned().0))
	}
}

impl Foldable for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, RcFnBrand}, functions::fold_right, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, SoloBrand, i32, i32>(Rc::new(|x| Rc::new(move |y| x + y)))(0)(Solo(3)),
	///     3
	/// );
	/// assert_eq!(
	///     fold_right::<RcFnBrand, SoloBrand, i32, i32>(Rc::new(|x| Rc::new(move |y| x * y)))(2)(Solo(4)),
	///     8
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		f: ApplyFn<'a, ClonableFnBrand, A, ApplyFn<'a, ClonableFnBrand, B, B>>
	) -> ApplyFn<'a, ClonableFnBrand, B, ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, B>> {
		ClonableFnBrand::new(move |b: B| {
			ClonableFnBrand::new({
				let f = f.clone();
				move |fa: Apply0L1T<Self, _>| f(fa.0)(b.to_owned())
			})
		})
	}
}

impl Traversable for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{SoloBrand, OptionBrand, RcFnBrand}, functions::traverse, types::Solo};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, SoloBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Solo(3)),
	///     Some(Solo(6))
	/// );
	/// assert_eq!(
	///     traverse::<RcFnBrand, SoloBrand, OptionBrand, i32, i32>(Rc::new(|x| None))(Solo(3)),
	///     None
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
		ClonableFnBrand::new(move |ta: Apply0L1T<Self, _>| {
			map::<ClonableFnBrand, F, B, _>(ClonableFnBrand::new(pure::<ClonableFnBrand, Self, _>))(
				f(ta.0),
			)
		})
	}
}
