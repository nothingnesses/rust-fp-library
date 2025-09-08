//! Implementations for [`Identity`], a type that wraps a value.

use crate::{
	classes::{
		Applicative, ApplyFirst, ApplySecond, ClonableFn, Foldable, Functor, Pointed,
		Semiapplicative, Semimonad, Traversable, clonable_fn::ApplyFn,
	},
	functions::{identity, map, pure},
	hkt::{Apply0L1T, Kind0L1T},
};

/// Wraps a value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identity<A>(pub A);

pub struct IdentityBrand;

impl Kind0L1T for IdentityBrand {
	type Output<A> = Identity<A>;
}

impl Functor for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, RcFnBrand}, functions::{identity, map}, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     map::<RcFnBrand, IdentityBrand, _, _>(Rc::new(identity))(Identity(())),
	///     Identity(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| Identity(f(fa.0)))
	}
}

impl Semiapplicative for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, RcFnBrand}, functions::{apply, identity}, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply::<RcFnBrand, IdentityBrand, _, _>(Identity(Rc::new(identity)))(Identity(())),
	///     Identity(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		map::<ClonableFnBrand, Self, A, B>(ff.0)
	}
}

impl ApplyFirst for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, RcFnBrand}, functions::{apply_first, identity}, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_first::<RcFnBrand, IdentityBrand, _, _>(Identity(true))(Identity(false)),
	///     Identity(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |_fb| fa.to_owned())
	}
}

impl ApplySecond for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, RcFnBrand}, functions::{apply_second, identity}, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     apply_second::<RcFnBrand, IdentityBrand, _, _>(Identity(true))(Identity(false)),
	///     Identity(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		_fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(identity)
	}
}

impl Pointed for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{RcFnBrand, IdentityBrand}, functions::pure, types::Identity};
	///
	/// assert_eq!(
	///     pure::<RcFnBrand, IdentityBrand, _>(()),
	///     Identity(())
	/// );
	/// ```
	fn pure<ClonableFnBrand: ClonableFn, A: Clone>(a: A) -> Apply0L1T<Self, A> {
		Identity(a)
	}
}

impl Semimonad for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, RcFnBrand}, functions::{bind, pure}, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     bind::<RcFnBrand, IdentityBrand, _, _>(Identity(()))(Rc::new(pure::<RcFnBrand, IdentityBrand, _>)),
	///     Identity(())
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

impl Foldable for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, RcFnBrand}, functions::fold_right, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     fold_right::<RcFnBrand, IdentityBrand, i32, i32>(Rc::new(|x| Rc::new(move |y| x + y)))(0)(Identity(3)),
	///     3
	/// );
	/// assert_eq!(
	///     fold_right::<RcFnBrand, IdentityBrand, i32, i32>(Rc::new(|x| Rc::new(move |y| x * y)))(2)(Identity(4)),
	///     8
	/// );
	/// ```
	fn fold_right<'a, ClonableFnBrand: 'a + ClonableFn, A: Clone, B: Clone>(
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

impl Traversable for IdentityBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{IdentityBrand, OptionBrand, RcFnBrand}, functions::traverse, types::Identity};
	/// use std::rc::Rc;
	///
	/// assert_eq!(
	///     traverse::<RcFnBrand, IdentityBrand, OptionBrand, i32, i32>(Rc::new(|x| Some(x * 2)))(Identity(3)),
	///     Some(Identity(6))
	/// );
	/// assert_eq!(
	///     traverse::<RcFnBrand, IdentityBrand, OptionBrand, i32, i32>(Rc::new(|x| None))(Identity(3)),
	///     None
	/// );
	/// ```
	fn traverse<'a, ClonableFnBrand: 'a + ClonableFn, F: Applicative, A: Clone, B: 'a + Clone>(
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
