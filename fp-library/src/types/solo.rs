//! Implementations for [`Solo`], a type that wraps a value.

use std::sync::Arc;

use crate::{
	aliases::ArcFn,
	functions::{identity, map, pure},
	hkt::{Apply0L1T, Kind0L1T},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, ClonableFn, Foldable,
		Functor, Pure, Traversable, clonable_fn::ApplyFn,
	},
};

/// Wraps a value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

pub struct SoloBrand;

impl Kind0L1T for SoloBrand {
	type Output<A> = Solo<A>;
}

impl Pure for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::pure, types::Solo};
	///
	/// assert_eq!(
	///     pure::<SoloBrand, _>(()),
	///     Solo(())
	/// );
	/// ```
	fn pure<A>(a: A) -> Apply0L1T<Self, A> {
		Solo(a)
	}
}

impl Functor for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{identity, map}, types::Solo};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<SoloBrand, _, _>(Arc::new(identity))(Solo(())),
	///     Solo(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa: Apply0L1T<Self, _>| Solo(f(fa.0)))
	}
}

impl TypeclassApply for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply, identity}, types::Solo};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     apply::<SoloBrand, _, _>(Solo(Arc::new(identity)))(Solo(())),
	///     Solo(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		map::<Self, ClonableFnBrand, A, B>(ff.0)
	}
}

impl ApplyFirst for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply_first, identity}, types::Solo};
	///
	/// assert_eq!(
	///     apply_first::<SoloBrand, _, _>(Solo(true))(Solo(false)),
	///     Solo(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |_fb| fa.to_owned())
	}
}

impl ApplySecond for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply_second, identity}, types::Solo};
	///
	/// assert_eq!(
	///     apply_second::<SoloBrand, _, _>(Solo(true))(Solo(false)),
	///     Solo(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(identity)
	}
}

impl Bind for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{bind, pure}, types::Solo};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     bind::<SoloBrand, _, _>(Solo(()))(Arc::new(pure::<SoloBrand, _>)),
	///     Solo(())
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
		ClonableFnBrand::new(move |f: ApplyFn<'a, ClonableFnBrand, _, _>| f(ma.to_owned().0))
	}
}

impl Foldable for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::fold_right, types::Solo};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<SoloBrand, i32, i32>(Arc::new(|x| Arc::new(move |y| x + y)))(0)(Solo(3)),
	///     3
	/// );
	/// assert_eq!(
	///     fold_right::<SoloBrand, i32, i32>(Arc::new(|x| Arc::new(move |y| x * y)))(2)(Solo(4)),
	///     8
	/// );
	/// ```
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply0L1T<Self, A>, B>> {
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| f(fa.0)(b.to_owned())
			})
		})
	}
}

/// # Examples
///
/// ```
/// use fp_library::{brands::{SoloBrand, OptionBrand}, functions::traverse, types::Solo};
/// use std::sync::Arc;
///
/// assert_eq!(
///     traverse::<SoloBrand, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Solo(3)),
///     Some(Solo(6))
/// );
/// assert_eq!(
///     traverse::<SoloBrand, OptionBrand, i32, i32>(Arc::new(|x| None))(Solo(3)),
///     None
/// );
/// ```
impl<'a> Traversable<'a> for SoloBrand {
	fn traverse<F: Applicative, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, Apply0L1T<F, B>>
	) -> ArcFn<'a, Apply0L1T<Self, A>, Apply0L1T<F, Apply0L1T<Self, B>>>
	where
		Apply0L1T<F, B>: 'a + Clone,
		Apply0L1T<F, ArcFn<'a, Apply0L1T<Self, B>, Apply0L1T<Self, B>>>: Clone,
	{
		Arc::new(move |ta| map::<F, B, _>(Arc::new(pure::<Self, _>))(f(ta.0)))
	}
}
