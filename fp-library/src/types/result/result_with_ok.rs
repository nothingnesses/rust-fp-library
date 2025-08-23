//! Implementations for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.

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

/// [Brand][crate::brands] for the partially-applied form of [`Result`] with the [`Ok`] constructor filled in.
pub struct ResultWithOkBrand<T>(T);

impl<T> Kind0L1T for ResultWithOkBrand<T> {
	type Output<A> = Result<T, A>;
}

impl<T> Pure for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::pure};
	///
	/// assert_eq!(
	///     pure::<ResultWithOkBrand<()>, _>(()),
	///     Err(())
	/// );
	fn pure<A>(a: A) -> Apply0L1T<Self, A> {
		Err(a)
	}
}

impl<T> Functor for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{identity, map}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     map::<ResultWithOkBrand<_>, _, _>(Arc::new(identity::<()>))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     map::<ResultWithOkBrand<bool>, _, _>(Arc::new(identity))(Err(())),
	///     Err(())
	/// );
	/// ```
	fn map<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a, B: 'a>(
		f: ApplyFn<'a, ClonableFnBrand, A, B>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match fa {
			Ok(a) => Ok(a),
			Err(e) => Err(f(e)),
		})
	}
}

impl<T: Clone> TypeclassApply for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply, identity}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<_>, (), ()>(Ok(true))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<_>, (), ()>(Ok(true))(Err(())),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<_>, (), ()>(Err(Arc::new(identity::<()>)))(Ok(true)),
	///     Ok(true)
	/// );
	/// assert_eq!(
	///     apply::<ResultWithOkBrand<bool>, (), ()>(Err(Arc::new(identity)))(Err(())),
	///     Err(())
	/// );
	/// ```
	fn apply<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply0L1T<Self, ApplyFn<'a, ClonableFnBrand, A, B>>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, A>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fa| match (ff.to_owned(), &fa) {
			(Ok(e), _) => Ok::<_, B>(e),
			(Err(f), _) => map::<ResultWithOkBrand<_>, ClonableFnBrand, _, _>(f)(fa),
		})
	}
}

impl<T: Clone> ApplyFirst for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply_first, identity}};
	///
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_first::<ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)),
	///     Err(true)
	/// );
	/// ```
	fn apply_first<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, A>> {
		ClonableFnBrand::new(move |fb: Apply0L1T<Self, _>| match (fa.to_owned(), fb) {
			(Err(a), Err(_a)) => Err(a),
			(Ok(e), _) | (_, Ok(e)) => Ok(e),
		})
	}
}

impl<T: Clone> ApplySecond for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{apply_second, identity}};
	///
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<_>, bool, bool>(Ok(()))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<_>, bool, _>(Ok(()))(Err(false)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<_>, _, bool>(Err(true))(Ok(())),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     apply_second::<ResultWithOkBrand<()>, _, _>(Err(true))(Err(false)),
	///     Err(false)
	/// );
	/// ```
	fn apply_second<'a, ClonableFnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a + Clone>(
		fa: Apply0L1T<Self, A>
	) -> ApplyFn<'a, ClonableFnBrand, Apply0L1T<Self, B>, Apply0L1T<Self, B>> {
		ClonableFnBrand::new(move |fb| match (fa.to_owned(), fb) {
			(Err(_a), Err(a)) => Err(a),
			(Ok(e), _) | (_, Ok(e)) => Ok(e),
		})
	}
}

impl<T: Clone> Bind for ResultWithOkBrand<T>
where
	for<'a> T: 'a,
{
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::{bind, pure}};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<_>, _, _>(Ok(()))(Arc::new(pure::<ResultWithOkBrand<_>, ()>)),
	///     Ok(())
	/// );
	/// assert_eq!(
	///     bind::<ResultWithOkBrand<()>, _, _>(Err(()))(Arc::new(pure::<ResultWithOkBrand<_>, _>)),
	///     Err(())
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
			ma.to_owned().or_else(|a| -> Result<_, B> { f(a) })
		})
	}
}

impl<T> Foldable for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::ResultWithOkBrand, functions::fold_right};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     fold_right::<ResultWithOkBrand<i32>, _, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Err(1)),
	///     2
	/// );
	/// assert_eq!(
	///     fold_right::<ResultWithOkBrand<_>, i32, _>(Arc::new(|a| Arc::new(move |b| a + b)))(1)(Ok(())),
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
					(_, b, Ok(_)) => b,
					(f, b, Err(a)) => f(a)(b),
				}
			})
		})
	}
}

impl<'a, T> Traversable<'a> for ResultWithOkBrand<T> {
	/// # Examples
	///
	/// ```
	/// use fp_library::{brands::{ResultWithOkBrand, OptionBrand}, functions::traverse};
	/// use std::sync::Arc;
	///
	/// assert_eq!(
	///     traverse::<ResultWithOkBrand<String>, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Ok(String::from("success"))),
	///     Some(Ok(String::from("success")))
	/// );
	/// assert_eq!(
	///     traverse::<ResultWithOkBrand<String>, OptionBrand, i32, i32>(Arc::new(|x| Some(x * 2)))(Err(5)),
	///     Some(Err(10))
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
			(_, Ok(e)) => pure::<F, _>(Ok(e)),
			(f, Err(a)) => map::<F, B, _>(Arc::new(pure::<Self, _>))(f(a)),
		})
	}
}
