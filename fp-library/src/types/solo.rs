//! Implementations for [`Solo`], a type that wraps a value.

use std::sync::Arc;

use crate::{
	aliases::ArcFn,
	functions::{identity, map, pure},
	hkt::{Apply1, Brand1, Kind1},
	typeclasses::{
		Applicative, Apply as TypeclassApply, ApplyFirst, ApplySecond, Bind, Foldable, Functor,
		Pure, Traversable,
	},
};

/// Wraps a value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Solo<A>(pub A);

pub struct SoloBrand;

impl Kind1 for SoloBrand {
	type Output<A> = Solo<A>;
}

impl<A> Brand1<Solo<A>, A> for SoloBrand {
	fn inject(a: Solo<A>) -> Apply1<Self, A> {
		a
	}

	fn project(a: Apply1<Self, A>) -> Solo<A> {
		a
	}
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
	fn pure<A>(a: A) -> Apply1<Self, A> {
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
	fn map<'a, A: 'a, B: 'a>(f: ArcFn<'a, A, B>) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>> {
		Arc::new(move |fa| Solo(f(fa.0)))
	}
}

impl TypeclassApply for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{apply, identity}, types::Solo};
	///
	/// assert_eq!(
	///     apply::<SoloBrand, _, _, _>(Solo(identity))(Solo(())),
	///     Solo(())
	/// );
	/// ```
	fn apply<'a, F: 'a + Fn(A) -> B, A: 'a + Clone, B: 'a>(
		ff: Apply1<Self, F>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<Self, B>>
	where
		Apply1<Self, F>: Clone,
	{
		map::<Self, A, B>(Arc::new(ff.0))
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
	fn apply_first<'a, A: 'a + Clone, B>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, A>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(move |_fb| fa.to_owned())
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
	fn apply_second<'a, A: 'a, B: 'a + Clone>(
		fa: Apply1<Self, A>
	) -> ArcFn<'a, Apply1<Self, B>, Apply1<Self, B>>
	where
		Apply1<Self, A>: Clone,
	{
		Arc::new(identity)
	}
}

impl Bind for SoloBrand {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::SoloBrand, functions::{bind, pure}, types::Solo};
	///
	/// assert_eq!(
	///     bind::<SoloBrand, _, _, _>(Solo(()))(pure::<SoloBrand, _>),
	///     Solo(())
	/// );
	/// ```
	fn bind<'a, F: Fn(A) -> Apply1<Self, B>, A: 'a + Clone, B>(
		ma: Apply1<Self, A>
	) -> ArcFn<'a, F, Apply1<Self, B>> {
		Arc::new(move |f| f(ma.to_owned().0))
	}
}

impl Foldable for SoloBrand {
	fn fold_right<'a, A: 'a + Clone, B: 'a + Clone>(
		f: ArcFn<'a, A, ArcFn<'a, B, B>>
	) -> ArcFn<'a, B, ArcFn<'a, Apply1<Self, A>, B>> {
		Arc::new(move |b| {
			Arc::new({
				let f = f.clone();
				move |fa| f(fa.0)(b.to_owned())
			})
		})
	}
}

impl Traversable for SoloBrand {
	fn traverse<'a, F: Applicative, A: 'a, B>(
		f: ArcFn<'a, A, Apply1<F, B>>
	) -> ArcFn<'a, Apply1<Self, A>, Apply1<F, Apply1<Self, B>>>
	where
		Apply1<F, B>: 'a,
	{
		Arc::new(move |ta| map::<F, B, _>(Arc::new(pure::<Self, _>))(f(ta.0)))
	}
}
