//! Implementations for [`Option`].
//!
//! This module provides implementations of functional programming traits for the standard library [`Option`] type.

use crate::{
	Apply,
	brands::OptionBrand,
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		clonable_fn::ClonableFn, compactable::Compactable, filterable::Filterable,
		foldable::Foldable, functor::Functor, lift::Lift, monoid::Monoid,
		par_foldable::ParFoldable, pointed::Pointed, semiapplicative::Semiapplicative,
		semimonad::Semimonad, send_clonable_fn::SendClonableFn, traversable::Traversable,
		witherable::Witherable,
	},
	impl_kind,
	kinds::*,
	types::Pair,
};

impl_kind! {
	for OptionBrand {
		type Of<'a, A: 'a>: 'a = Option<A>;
	}
}

impl Functor for OptionBrand {
	/// Maps a function over the value in the option.
	///
	/// This method applies a function to the value inside the option, producing a new option with the transformed value. If the option is `None`, it returns `None`.
	///
	/// ### Type Signature
	///
	/// `forall a b. Functor Option => (a -> b, Option a) -> Option b`
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the value.
	/// * `fa`: The option to map over.
	///
	/// ### Returns
	///
	/// A new option containing the result of applying the function, or `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::functor::Functor;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::map(|i| i * 2, x);
	/// assert_eq!(y, Some(10));
	///
	/// // Using the free function
	/// use fp_library::classes::functor::map;
	/// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, Some(5)), Some(10));
	/// assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x * 2, None), None);
	/// ```
	fn map<'a, F, A: 'a, B: 'a>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a,
	{
		fa.map(f)
	}
}

impl Lift for OptionBrand {
	/// Lifts a binary function into the option context.
	///
	/// This method lifts a binary function to operate on values within the option context.
	///
	/// ### Type Signature
	///
	/// `forall a b c. Lift Option => ((a, b) -> c, Option a, Option b) -> Option c`
	///
	/// ### Parameters
	///
	/// * `f`: The binary function to apply.
	/// * `fa`: The first option.
	/// * `fb`: The second option.
	///
	/// ### Returns
	///
	/// `Some(f(a, b))` if both options are `Some`, otherwise `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::lift::Lift;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(1);
	/// let y = Some(2);
	/// let z = OptionBrand::lift2(|a, b| a + b, x, y);
	/// assert_eq!(z, Some(3));
	///
	/// // Using the free function
	/// use fp_library::classes::lift::lift2;
	/// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), Some(2)), Some(3));
	/// assert_eq!(lift2::<OptionBrand, _, _, _, _>(|x: i32, y: i32| x + y, Some(1), None), None);
	/// ```
	fn lift2<'a, F, A, B, C>(
		f: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		F: Fn(A, B) -> C + 'a,
		A: 'a,
		B: 'a,
		C: 'a,
	{
		fa.zip(fb).map(|(a, b)| f(a, b))
	}
}

impl Pointed for OptionBrand {
	/// Wraps a value in an option.
	///
	/// This method wraps a value in an option context.
	///
	/// ### Type Signature
	///
	/// `forall a. Pointed Option => a -> Option a`
	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// `Some(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::pointed::Pointed;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = OptionBrand::pure(5);
	/// assert_eq!(x, Some(5));
	///
	/// // Using the free function
	/// use fp_library::classes::pointed::pure;
	/// assert_eq!(pure::<OptionBrand, _>(5), Some(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Some(a)
	}
}

impl ApplyFirst for OptionBrand {}
impl ApplySecond for OptionBrand {}

impl Semiapplicative for OptionBrand {
	/// Applies a wrapped function to a wrapped value.
	///
	/// This method applies a function wrapped in an option to a value wrapped in an option.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semiapplicative Option => (Option (a -> b), Option a) -> Option b`
	///
	/// ### Parameters
	///
	/// * `ff`: The option containing the function.
	/// * `fa`: The option containing the value.
	///
	/// ### Returns
	///
	/// `Some(f(a))` if both are `Some`, otherwise `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semiapplicative::Semiapplicative;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	/// use fp_library::brands::{OptionBrand};
	/// use fp_library::brands::RcFnBrand;
	/// use std::rc::Rc;
	///
	/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// let x = Some(5);
	/// let y = OptionBrand::apply::<RcFnBrand, i32, i32>(f, x);
	/// assert_eq!(y, Some(10));
	///
	/// // Using the free function
	/// use fp_library::classes::semiapplicative::apply;
	/// let f = Some(<RcFnBrand as ClonableFn>::new(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, OptionBrand, _, _>(f, Some(5)), Some(10));
	/// ```
	fn apply<'a, FnBrand: 'a + ClonableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as ClonableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match (ff, fa) {
			(Some(f), Some(a)) => Some(f(a)),
			_ => None,
		}
	}
}

impl Semimonad for OptionBrand {
	/// Chains option computations.
	///
	/// This method chains two option computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	/// `forall a b. Semimonad Option => (Option a, a -> Option b) -> Option b`
	///
	/// ### Parameters
	///
	/// * `ma`: The first option.
	/// * `f`: The function to apply to the value inside the option.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the value if `ma` is `Some`, otherwise `None`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semimonad::Semimonad;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::bind(x, |i| Some(i * 2));
	/// assert_eq!(y, Some(10));
	///
	/// // Using the free function
	/// use fp_library::classes::semimonad::bind;
	/// assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |x| Some(x * 2)), Some(10));
	/// assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x * 2)), None);
	/// ```
	fn bind<'a, F, A: 'a, B: 'a>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		f: F,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		F: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		ma.and_then(f)
	}
}

impl Foldable for OptionBrand {
	/// Folds the option from the right.
	///
	/// This method performs a right-associative fold of the option. If the option is `Some(a)`, it applies the function to `a` and the initial value. If `None`, it returns the initial value.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable Option => ((a, b) -> b, b, Option a) -> b`
	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)` if `fa` is `Some(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	///
	/// // Using the free function
	/// use fp_library::classes::foldable::fold_right;
	/// assert_eq!(fold_right::<RcFnBrand, OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, Some(5)), 5);
	/// assert_eq!(fold_right::<RcFnBrand, OptionBrand, _, _, _>(|x: i32, acc| x + acc, 0, None), 0);
	/// ```
	fn fold_right<'a, FnBrand, Func, A: 'a, B: 'a>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Some(a) => func(a, initial),
			None => initial,
		}
	}

	/// Folds the option from the left.
	///
	/// This method performs a left-associative fold of the option. If the option is `Some(a)`, it applies the function to the initial value and `a`. If `None`, it returns the initial value.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable Option => ((b, a) -> b, b, Option a) -> b`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// `f(initial, a)` if `fa` is `Some(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_left::<RcFnBrand, _, _, _>(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	///
	/// // Using the free function
	/// use fp_library::classes::foldable::fold_left;
	/// assert_eq!(fold_left::<RcFnBrand, OptionBrand, _, _, _>(|acc, x: i32| acc + x, 0, Some(5)), 5);
	/// ```
	fn fold_left<'a, FnBrand, Func, A: 'a, B: 'a>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Some(a) => func(initial, a),
			None => initial,
		}
	}

	/// Maps the value to a monoid and returns it, or returns empty.
	///
	/// This method maps the element of the option to a monoid. If the option is `None`, it returns the monoid's identity element.
	///
	/// ### Type Signature
	///
	/// `forall a m. (Foldable Option, Monoid m) => ((a) -> m, Option a) -> m`
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// `func(a)` if `fa` is `Some(a)`, otherwise `M::empty()`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::string; // Import to bring Monoid impl for String into scope
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_map::<RcFnBrand, _, _, _>(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	///
	/// // Using the free function
	/// use fp_library::classes::foldable::fold_map;
	/// assert_eq!(fold_map::<RcFnBrand, OptionBrand, _, _, _>(|x: i32| x.to_string(), Some(5)), "5".to_string());
	/// ```
	fn fold_map<'a, FnBrand, Func, A: 'a, M>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
	{
		match fa {
			Some(a) => func(a),
			None => M::empty(),
		}
	}
}

impl Traversable for OptionBrand {
	/// Traverses the option with an applicative function.
	///
	/// This method maps the element of the option to a computation, evaluates it, and wraps the result in the applicative context. If `None`, it returns `pure(None)`.
	///
	/// ### Type Signature
	///
	/// `forall a b f. (Traversable Option, Applicative f) => (a -> f b, Option a) -> f (Option b)`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element, returning a value in an applicative context.
	/// * `ta`: The option to traverse.
	///
	/// ### Returns
	///
	/// The option wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::traversable::Traversable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::traverse::<OptionBrand, _, _, _>(|a| Some(a * 2), x);
	/// assert_eq!(y, Some(Some(10)));
	///
	/// // Using the free function
	/// use fp_library::classes::traversable::traverse;
	/// assert_eq!(traverse::<OptionBrand, OptionBrand, _, _, _>(|x| Some(x * 2), Some(5)), Some(Some(10)));
	/// ```
	fn traverse<'a, F: Applicative, Func, A: 'a + Clone, B: 'a + Clone>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		match ta {
			Some(a) => F::map(|b| Some(b), func(a)),
			None => F::pure(None),
		}
	}
	/// Sequences an option of applicative.
	///
	/// This method evaluates the computation inside the option and wraps the result in the applicative context. If `None`, it returns `pure(None)`.
	///
	/// ### Type Signature
	///
	/// `forall a f. (Traversable Option, Applicative f) => (Option (f a)) -> f (Option a)`
	///
	/// ### Parameters
	///
	/// * `ta`: The option containing the applicative value.
	///
	/// # Returns
	///
	/// The option wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::traversable::Traversable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(Some(5));
	/// let y = OptionBrand::sequence::<OptionBrand, _>(x);
	/// assert_eq!(y, Some(Some(5)));
	///
	/// // Using the free function
	/// use fp_library::classes::traversable::sequence;
	/// assert_eq!(sequence::<OptionBrand, OptionBrand, _>(Some(Some(5))), Some(Some(5)));
	/// ```
	fn sequence<'a, F: Applicative, A: 'a + Clone>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		match ta {
			Some(fa) => F::map(|a| Some(a), fa),
			None => F::pure(None),
		}
	}
}

impl<FnBrand: SendClonableFn> ParFoldable<FnBrand> for OptionBrand {
	/// Maps the value to a monoid and returns it, or returns empty, in parallel.
	///
	/// This method maps the element of the option to a monoid. Since `Option` contains at most one element, no actual parallelism occurs, but the interface is satisfied.
	///
	/// ### Type Signature
	///
	/// `forall a m. (ParFoldable Option, Monoid m, Send m, Sync m) => (f a m, Option a) -> m`
	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The option to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::par_foldable::ParFoldable;
	/// use fp_library::brands::{OptionBrand, ArcFnBrand};
	/// use fp_library::classes::send_clonable_fn::SendClonableFn;
	/// use fp_library::classes::send_clonable_fn::new_send;
	///
	/// let x = Some(1);
	/// let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// let y = <OptionBrand as ParFoldable<ArcFnBrand>>::par_fold_map(f, x);
	/// assert_eq!(y, "1".to_string());
	///
	/// // Using the free function
	/// use fp_library::classes::par_foldable::par_fold_map;
	/// let x = Some(1);
	/// let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, OptionBrand, _, _>(f, x), "1".to_string());
	/// ```
	fn par_fold_map<'a, A, M>(
		func: <FnBrand as SendClonableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		match fa {
			Some(a) => func(a),
			None => M::empty(),
		}
	}
}

impl Compactable for OptionBrand {
	/// Compacts a nested option.
	///
	/// This method flattens a nested option.
	///
	/// ### Type Signature
	///
	/// `forall a. Compactable Option => Option (Option a) -> Option a`
	///
	/// ### Parameters
	///
	/// * `fa`: The nested option.
	///
	/// ### Returns
	///
	/// The flattened option.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::compactable::Compactable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(Some(5));
	/// let y = OptionBrand::compact(x);
	/// assert_eq!(y, Some(5));
	/// ```
	fn compact<'a, A: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'a,
			Apply!(<OptionBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		fa.flatten()
	}

	/// Separates an option of result.
	///
	/// This method separates an option of result into a pair of options.
	///
	/// ### Type Signature
	///
	/// `forall e o. Compactable Option => Option (Result o e) -> (Option o, Option e)`
	///
	/// ### Parameters
	///
	/// * `fa`: The option of result.
	///
	/// ### Returns
	///
	/// A pair of options.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::compactable::Compactable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x: Option<Result<i32, &str>> = Some(Ok(5));
	/// let Pair(oks, errs) = OptionBrand::separate(x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	/// ```
	fn separate<'a, E: 'a, O: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Result<O, E>>)
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	> {
		match fa {
			Some(Ok(o)) => Pair(Some(o), None),
			Some(Err(e)) => Pair(None, Some(e)),
			None => Pair(None, None),
		}
	}
}

impl Filterable for OptionBrand {
	/// Partitions an option based on a function that returns a result.
	///
	/// This method partitions an option based on a function that returns a result.
	///
	/// ### Type Signature
	///
	/// `forall a e o. Filterable Option => (a -> Result o e) -> Option a -> (Option o, Option e)`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `fa`: The option to partition.
	///
	/// ### Returns
	///
	/// A pair of options.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = Some(5);
	/// let Pair(oks, errs) = OptionBrand::partition_map(|a| if a > 2 { Ok(a) } else { Err(a) }, x);
	/// assert_eq!(oks, Some(5));
	/// assert_eq!(errs, None);
	/// ```
	fn partition_map<'a, Func, A: 'a, E: 'a, O: 'a>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
	>
	where
		Func: Fn(A) -> Result<O, E> + 'a,
	{
		match fa {
			Some(a) => match func(a) {
				Ok(o) => Pair(Some(o), None),
				Err(e) => Pair(None, Some(e)),
			},
			None => Pair(None, None),
		}
	}

	/// Partitions an option based on a predicate.
	///
	/// This method partitions an option based on a predicate.
	///
	/// ### Type Signature
	///
	/// `forall a. Filterable Option => (a -> bool) -> Option a -> (Option a, Option a)`
	///
	/// ### Parameters
	///
	/// * `func`: The predicate.
	/// * `fa`: The option to partition.
	///
	/// ### Returns
	///
	/// A pair of options.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::Pair;
	///
	/// let x = Some(5);
	/// let Pair(satisfied, not_satisfied) = OptionBrand::partition(|a| a > 2, x);
	/// assert_eq!(satisfied, Some(5));
	/// assert_eq!(not_satisfied, None);
	/// ```
	fn partition<'a, Func, A: 'a + Clone>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Pair<
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	>
	where
		Func: Fn(A) -> bool + 'a,
	{
		match fa {
			Some(a) => {
				if func(a.clone()) {
					Pair(Some(a), None)
				} else {
					Pair(None, Some(a))
				}
			}
			None => Pair(None, None),
		}
	}

	/// Maps a function over an option and filters out `None` results.
	///
	/// This method maps a function over an option and filters out `None` results.
	///
	/// ### Type Signature
	///
	/// `forall a b. Filterable Option => (a -> Option b) -> Option a -> Option b`
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `fa`: The option to filter and map.
	///
	/// ### Returns
	///
	/// The filtered and mapped option.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::filter_map(|a| if a > 2 { Some(a * 2) } else { None }, x);
	/// assert_eq!(y, Some(10));
	/// ```
	fn filter_map<'a, Func, A: 'a, B: 'a>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Option<B> + 'a,
	{
		fa.and_then(func)
	}

	/// Filters an option based on a predicate.
	///
	/// This method filters an option based on a predicate.
	///
	/// ### Type Signature
	///
	/// `forall a. Filterable Option => (a -> bool) -> Option a -> Option a`
	///
	/// ### Parameters
	///
	/// * `func`: The predicate.
	/// * `fa`: The option to filter.
	///
	/// ### Returns
	///
	/// The filtered option.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::filterable::Filterable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::filter(|a| a > 2, x);
	/// assert_eq!(y, Some(5));
	/// ```
	fn filter<'a, Func, A: 'a + Clone>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Func: Fn(A) -> bool + 'a,
	{
		fa.filter(|a| func(a.clone()))
	}
}

impl Witherable for OptionBrand {}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::{ArcFnBrand, RcFnBrand},
		classes::{
			compactable::{compact, separate},
			filterable::{filter, filter_map, partition, partition_map},
			functor::map,
			par_foldable::{par_fold_map, par_fold_right},
			pointed::pure,
			semiapplicative::apply,
			semimonad::bind,
			send_clonable_fn::new_send,
			traversable::traverse,
			witherable::{wilt, wither},
		},
		functions::{compose, identity},
	};
	use quickcheck_macros::quickcheck;

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(x: Option<i32>) -> bool {
		map::<OptionBrand, _, _, _>(identity, x) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(x: Option<i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<OptionBrand, _, _, _>(compose(f, g), x)
			== map::<OptionBrand, _, _, _>(f, map::<OptionBrand, _, _, _>(g, x))
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(v: Option<i32>) -> bool {
		apply::<RcFnBrand, OptionBrand, _, _>(
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(identity)),
			v,
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, OptionBrand, _, _>(
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(f)),
			pure::<OptionBrand, _>(x),
		) == pure::<OptionBrand, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w: Option<i32>,
		u_is_some: bool,
		v_is_some: bool,
	) -> bool {
		let v_fn = |x: i32| x.wrapping_mul(2);
		let u_fn = |x: i32| x.wrapping_add(1);

		let v = if v_is_some {
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(v_fn))
		} else {
			None
		};
		let u = if u_is_some {
			pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(u_fn))
		} else {
			None
		};

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, OptionBrand, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		// equivalent to (u . v) <*> w
		let uv = match (u, v) {
			(Some(uf), Some(vf)) => {
				let composed = move |x| uf(vf(x));
				Some(<RcFnBrand as ClonableFn>::new(composed))
			}
			_ => None,
		};

		let lhs = apply::<RcFnBrand, OptionBrand, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(y: i32) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = |x: i32| x.wrapping_mul(2);
		let u = pure::<OptionBrand, _>(<RcFnBrand as ClonableFn>::new(f));

		let lhs = apply::<RcFnBrand, OptionBrand, _, _>(u.clone(), pure::<OptionBrand, _>(y));

		let rhs_fn = <RcFnBrand as ClonableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, OptionBrand, _, _>(pure::<OptionBrand, _>(rhs_fn), u);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Some(x.wrapping_mul(2));
		bind::<OptionBrand, _, _, _>(pure::<OptionBrand, _>(a), f) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(m: Option<i32>) -> bool {
		bind::<OptionBrand, _, _, _>(m, pure::<OptionBrand, _>) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(m: Option<i32>) -> bool {
		let f = |x: i32| Some(x.wrapping_mul(2));
		let g = |x: i32| Some(x.wrapping_add(1));
		bind::<OptionBrand, _, _, _>(bind::<OptionBrand, _, _, _>(m, f), g)
			== bind::<OptionBrand, _, _, _>(m, |x| bind::<OptionBrand, _, _, _>(f(x), g))
	}

	// Edge Cases

	/// Tests `map` on `None`.
	#[test]
	fn map_none() {
		assert_eq!(map::<OptionBrand, _, _, _>(|x: i32| x + 1, None), None);
	}

	/// Tests `bind` on `None`.
	#[test]
	fn bind_none() {
		assert_eq!(bind::<OptionBrand, _, _, _>(None, |x: i32| Some(x + 1)), None);
	}

	/// Tests `bind` returning `None`.
	#[test]
	fn bind_returning_none() {
		assert_eq!(bind::<OptionBrand, _, _, _>(Some(5), |_| None::<i32>), None);
	}

	/// Tests `fold_right` on `None`.
	#[test]
	fn fold_right_none() {
		assert_eq!(
			crate::classes::foldable::fold_right::<RcFnBrand, OptionBrand, _, _, _>(
				|x: i32, acc| x + acc,
				0,
				None
			),
			0
		);
	}

	/// Tests `fold_left` on `None`.
	#[test]
	fn fold_left_none() {
		assert_eq!(
			crate::classes::foldable::fold_left::<RcFnBrand, OptionBrand, _, _, _>(
				|acc, x: i32| acc + x,
				0,
				None
			),
			0
		);
	}

	/// Tests `traverse` on `None`.
	#[test]
	fn traverse_none() {
		assert_eq!(
			crate::classes::traversable::traverse::<OptionBrand, OptionBrand, _, _, _>(
				|x: i32| Some(x + 1),
				None
			),
			Some(None)
		);
	}

	/// Tests `traverse` returning `None`.
	#[test]
	fn traverse_returning_none() {
		assert_eq!(
			crate::classes::traversable::traverse::<OptionBrand, OptionBrand, _, _, _>(
				|_: i32| None::<i32>,
				Some(5)
			),
			None
		);
	}

	// ParFoldable Tests

	/// Tests `par_fold_map` on `None`.
	#[test]
	fn par_fold_map_none() {
		let x: Option<i32> = None;
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, OptionBrand, _, _>(f, x), "".to_string());
	}

	/// Tests `par_fold_map` on `Some`.
	#[test]
	fn par_fold_map_some() {
		let x = Some(5);
		let f = new_send::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, OptionBrand, _, _>(f, x), "5".to_string());
	}

	/// Tests `par_fold_right` on `Some`.
	#[test]
	fn par_fold_right_some() {
		let x = Some(5);
		let f = new_send::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, OptionBrand, _, _>(f, 10, x), 15);
	}

	// Filterable Laws

	/// Tests `filterMap identity ≡ compact`.
	#[quickcheck]
	fn filterable_filter_map_identity(x: Option<Option<i32>>) -> bool {
		filter_map::<OptionBrand, _, _, _>(identity, x.clone()) == compact::<OptionBrand, _>(x)
	}

	/// Tests `filterMap Just ≡ identity`.
	#[quickcheck]
	fn filterable_filter_map_just(x: Option<i32>) -> bool {
		filter_map::<OptionBrand, _, _, _>(Some, x.clone()) == x
	}

	/// Tests `filterMap (l <=< r) ≡ filterMap l <<< filterMap r`.
	#[quickcheck]
	fn filterable_filter_map_composition(x: Option<i32>) -> bool {
		let r = |i: i32| if i % 2 == 0 { Some(i) } else { None };
		let l = |i: i32| if i > 5 { Some(i) } else { None };
		let composed = |i| r(i).and_then(l);

		filter_map::<OptionBrand, _, _, _>(composed, x.clone())
			== filter_map::<OptionBrand, _, _, _>(l, filter_map::<OptionBrand, _, _, _>(r, x))
	}

	/// Tests `filter ≡ filterMap <<< maybeBool`.
	#[quickcheck]
	fn filterable_filter_consistency(x: Option<i32>) -> bool {
		let p = |i: i32| i % 2 == 0;
		let maybe_bool = |i| if p(i) { Some(i) } else { None };

		filter::<OptionBrand, _, _>(p, x.clone())
			== filter_map::<OptionBrand, _, _, _>(maybe_bool, x)
	}

	/// Tests `partitionMap identity ≡ separate`.
	#[quickcheck]
	fn filterable_partition_map_identity(x: Option<Result<i32, i32>>) -> bool {
		partition_map::<OptionBrand, _, _, _, _>(identity, x.clone())
			== separate::<OptionBrand, _, _>(x)
	}

	/// Tests `partitionMap Right ≡ identity` (on the right side).
	#[quickcheck]
	fn filterable_partition_map_right_identity(x: Option<i32>) -> bool {
		let Pair(oks, _) = partition_map::<OptionBrand, _, _, _, _>(Ok::<_, i32>, x.clone());
		oks == x
	}

	/// Tests `partitionMap Left ≡ identity` (on the left side).
	#[quickcheck]
	fn filterable_partition_map_left_identity(x: Option<i32>) -> bool {
		let Pair(_, errs) = partition_map::<OptionBrand, _, _, _, _>(Err::<i32, _>, x.clone());
		errs == x
	}

	/// Tests `f <<< partition ≡ partitionMap <<< eitherBool`.
	#[quickcheck]
	fn filterable_partition_consistency(x: Option<i32>) -> bool {
		let p = |i: i32| i % 2 == 0;
		let either_bool = |i| if p(i) { Ok(i) } else { Err(i) };

		let Pair(satisfied, not_satisfied) = partition::<OptionBrand, _, _>(p, x.clone());
		let Pair(oks, errs) = partition_map::<OptionBrand, _, _, _, _>(either_bool, x);

		satisfied == oks && not_satisfied == errs
	}

	// Witherable Laws

	/// Tests `wither (pure <<< Just) ≡ pure`.
	#[quickcheck]
	fn witherable_identity(x: Option<i32>) -> bool {
		wither::<OptionBrand, _, OptionBrand, _, _>(|i| Some(Some(i)), x.clone()) == Some(x)
	}

	/// Tests `wilt p ≡ map separate <<< traverse p`.
	#[quickcheck]
	fn witherable_wilt_consistency(x: Option<i32>) -> bool {
		let p = |i: i32| Some(if i % 2 == 0 { Ok(i) } else { Err(i) });

		let lhs = wilt::<OptionBrand, _, OptionBrand, _, _, _>(p, x.clone());
		let rhs = map::<OptionBrand, _, _, _>(
			|res| separate::<OptionBrand, _, _>(res),
			traverse::<OptionBrand, OptionBrand, _, _, _>(p, x),
		);

		lhs == rhs
	}

	/// Tests `wither p ≡ map compact <<< traverse p`.
	#[quickcheck]
	fn witherable_wither_consistency(x: Option<i32>) -> bool {
		let p = |i: i32| Some(if i % 2 == 0 { Some(i) } else { None });

		let lhs = wither::<OptionBrand, _, OptionBrand, _, _>(p, x.clone());
		let rhs = map::<OptionBrand, _, _, _>(
			|opt| compact::<OptionBrand, _>(opt),
			traverse::<OptionBrand, OptionBrand, _, _, _>(p, x),
		);

		lhs == rhs
	}

	// Edge Cases

	/// Tests `compact` on `Some(None)`.
	#[test]
	fn compact_some_none() {
		assert_eq!(compact::<OptionBrand, _>(Some(None::<i32>)), None);
	}

	/// Tests `compact` on `Some(Some(x))`.
	#[test]
	fn compact_some_some() {
		assert_eq!(compact::<OptionBrand, _>(Some(Some(5))), Some(5));
	}

	/// Tests `compact` on `None`.
	#[test]
	fn compact_none() {
		assert_eq!(compact::<OptionBrand, _>(None::<Option<i32>>), None);
	}

	/// Tests `separate` on `Some(Ok(x))`.
	#[test]
	fn separate_some_ok() {
		let Pair(oks, errs) = separate::<OptionBrand, _, _>(Some(Ok::<i32, &str>(5)));
		assert_eq!(oks, Some(5));
		assert_eq!(errs, None);
	}

	/// Tests `separate` on `Some(Err(e))`.
	#[test]
	fn separate_some_err() {
		let Pair(oks, errs) = separate::<OptionBrand, _, _>(Some(Err::<i32, &str>("error")));
		assert_eq!(oks, None);
		assert_eq!(errs, Some("error"));
	}

	/// Tests `separate` on `None`.
	#[test]
	fn separate_none() {
		let Pair(oks, errs) = separate::<OptionBrand, _, _>(None::<Result<i32, &str>>);
		assert_eq!(oks, None);
		assert_eq!(errs, None);
	}

	/// Tests `partition_map` on `None`.
	#[test]
	fn partition_map_none() {
		let Pair(oks, errs) =
			partition_map::<OptionBrand, _, _, _, _>(|x: i32| Ok::<i32, i32>(x), None::<i32>);
		assert_eq!(oks, None);
		assert_eq!(errs, None);
	}

	/// Tests `partition` on `None`.
	#[test]
	fn partition_none() {
		let Pair(satisfied, not_satisfied) =
			partition::<OptionBrand, _, _>(|x: i32| x > 0, None::<i32>);
		assert_eq!(satisfied, None);
		assert_eq!(not_satisfied, None);
	}

	/// Tests `filter_map` on `None`.
	#[test]
	fn filter_map_none() {
		assert_eq!(filter_map::<OptionBrand, _, _, _>(|x: i32| Some(x), None::<i32>), None);
	}

	/// Tests `filter` on `None`.
	#[test]
	fn filter_none() {
		assert_eq!(filter::<OptionBrand, _, _>(|x: i32| x > 0, None::<i32>), None);
	}

	/// Tests `wilt` on `None`.
	#[test]
	fn wilt_none() {
		let res = wilt::<OptionBrand, _, OptionBrand, _, _, _>(
			|x: i32| Some(Ok::<i32, i32>(x)),
			None::<i32>,
		);
		assert_eq!(res, Some(Pair(None, None)));
	}

	/// Tests `wither` on `None`.
	#[test]
	fn wither_none() {
		let res = wither::<OptionBrand, _, OptionBrand, _, _>(|x: i32| Some(Some(x)), None::<i32>);
		assert_eq!(res, Some(None));
	}
}
