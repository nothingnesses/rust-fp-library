//! Two-value container with [`Bifunctor`](crate::classes::Bifunctor) and dual [`Functor`](crate::classes::Functor) instances.
//!
//! Can be used as a bifunctor over both values, or as a functor/monad by fixing either the first value [`PairWithFirstBrand`](crate::brands::PairWithFirstBrand) or second value [`PairWithSecondBrand`](crate::brands::PairWithSecondBrand).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{PairBrand, PairWithFirstBrand, PairWithSecondBrand},
			classes::{
				Applicative, ApplyFirst, ApplySecond, Bifunctor, CloneableFn, Foldable, Functor,
				Lift, Monoid, ParFoldable, Pointed, Semiapplicative, Semigroup, Semimonad,
				SendCloneableFn, Traversable,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{document_fields, document_parameters, document_type_parameters},
	};

	/// Wraps two values.
	///
	/// A simple tuple struct that holds two values of potentially different types.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// This type has multiple higher-kinded representations:
	/// - [`PairBrand`](crate::brands::PairBrand): fully polymorphic over both values (bifunctor).
	/// - [`PairWithFirstBrand<First>`](crate::brands::PairWithFirstBrand): the first value type is fixed, polymorphic over the second (functor over second).
	/// - [`PairWithSecondBrand<Second>`](crate::brands::PairWithSecondBrand): the second value type is fixed, polymorphic over the first (functor over first).
	///
	/// ### Serialization
	///
	/// This type supports serialization and deserialization via [`serde`](https://serde.rs) when the `serde` feature is enabled.
	#[document_type_parameters("The type of the first value.", "The type of the second value.")]
	///
	#[document_fields("The first value.", "The second value.")]
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let p = Pair(1, "hello");
	/// assert_eq!(p.0, 1);
	/// assert_eq!(p.1, "hello");
	/// ```
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Pair<First, Second>(pub First, pub Second);

	impl_kind! {
		for PairBrand {
			type Of<First, Second> = Pair<First, Second>;
		}
	}

	impl Bifunctor for PairBrand {
		/// Maps functions over the values in the pair.
		///
		/// This method applies one function to the first value and another to the second value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the first value.",
			"The type of the mapped first value.",
			"The type of the second value.",
			"The type of the mapped second value.",
			"The type of the function to apply to the first value.",
			"The type of the function to apply to the second value."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The function to apply to the second value.",
			"The pair to map over."
		)]
		///
		/// ### Returns
		///
		/// A new pair containing the mapped values.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::bifunctor::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(1, 5);
		/// assert_eq!(bimap::<PairBrand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, x), Pair(2, 10));
		/// ```
		fn bimap<A, B, C, D, F, G>(
			f: F,
			g: G,
			p: Apply!(<Self as Kind!( type Of<T, U>; )>::Of<A, C>),
		) -> Apply!(<Self as Kind!( type Of<T, U>; )>::Of<B, D>)
		where
			F: Fn(A) -> B,
			G: Fn(C) -> D,
		{
			let Pair(a, c) = p;
			Pair(f(a), g(c))
		}
	}

	// PairWithFirstBrand<First> (Functor over Second)

	impl_kind! {
		#[document_type_parameters("The type of the first value in the pair.")]
		impl<First> for PairWithFirstBrand<First> {
			type Of<A> = Pair<First, A>;
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First> Functor for PairWithFirstBrand<First> {
		/// Maps a function over the second value in the pair.
		///
		/// This method applies a function to the second value inside the pair, producing a new pair with the transformed second value. The first value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the second value.",
			"The type of the result of applying the function.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters(
			"The function to apply to the second value.",
			"The pair to map over."
		)]
		///
		/// ### Returns
		///
		/// A new pair containing the result of applying the function to the second value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(map::<PairWithFirstBrand<_>, _, _, _>(|x: i32| x * 2, Pair(1, 5)), Pair(1, 10));
		/// ```
		fn map<A, B, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> B,
		{
			Pair(fa.0, func(fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone> Lift for PairWithFirstBrand<First>
	where
		First: Semigroup,
	{
		/// Lifts a binary function into the pair context (over second).
		///
		/// This method lifts a binary function to operate on the second values within the pair context. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the first second value.",
			"The type of the second second value.",
			"The type of the result second value.",
			"The type of the binary function."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the second values.",
			"The first pair.",
			"The second pair."
		)]
		///
		/// ### Returns
		///
		/// A new pair where the first values are combined using `Semigroup::append` and the second values are combined using `f`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<PairWithFirstBrand<String>, _, _, _, _>(
		/// 		|x, y| x + y,
		/// 		Pair("a".to_string(), 1),
		/// 		Pair("b".to_string(), 2)
		/// 	),
		/// 	Pair("ab".to_string(), 3)
		/// );
		/// ```
		fn lift2<A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			fb: Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<C>)
		where
			Func: Fn(A, B) -> C,
			A: Clone,
			B: Clone,
		{
			Pair(Semigroup::append(fa.0, fb.0), func(fa.1, fb.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone> Pointed for PairWithFirstBrand<First>
	where
		First: Monoid,
	{
		/// Wraps a value in a pair (with empty first).
		///
		/// This method wraps a value in a pair, using the `Monoid::empty()` value for the first element.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		/// ### Returns
		///
		/// A pair containing the empty value of the first type and `a`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(pure::<PairWithFirstBrand<String>, _>(5), Pair("".to_string(), 5));
		/// ```
		fn pure<A>(a: A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<A>) {
			Pair(Monoid::empty(), a)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + Semigroup> ApplyFirst for PairWithFirstBrand<First> {}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + Semigroup> ApplySecond for PairWithFirstBrand<First> {}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone> Semiapplicative for PairWithFirstBrand<First>
	where
		First: Semigroup,
	{
		/// Applies a wrapped function to a wrapped value (over second).
		///
		/// This method applies a function wrapped in a pair to a value wrapped in a pair. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The pair containing the function.",
			"The pair containing the value."
		)]
		///
		/// ### Returns
		///
		/// A new pair where the first values are combined and the function is applied to the second value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Pair("a".to_string(), cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(f, Pair("b".to_string(), 5)),
		/// 	Pair("ab".to_string(), 10)
		/// );
		/// ```
		fn apply<FnBrand: CloneableFn, A: Clone, B>(
			ff: Apply!(<Self as Kind!( type Of<T>; )>::Of<<FnBrand as CloneableFn>::Of<A, B>>),
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>) {
			Pair(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone> Semimonad for PairWithFirstBrand<First>
	where
		First: Semigroup,
	{
		/// Chains pair computations (over second).
		///
		/// This method chains two computations, where the second computation depends on the result of the first. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the result of the first computation.",
			"The type of the result of the second computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The first pair.", "The function to apply to the second value.")]
		///
		/// ### Returns
		///
		/// A new pair where the first values are combined.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bind::<PairWithFirstBrand<String>, _, _, _>(Pair("a".to_string(), 5), |x| Pair(
		/// 		"b".to_string(),
		/// 		x * 2
		/// 	)),
		/// 	Pair("ab".to_string(), 10)
		/// );
		/// ```
		fn bind<A, B, Func>(
			ma: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		{
			let Pair(first, second) = ma;
			let Pair(next_first, next_second) = func(second);
			Pair(Semigroup::append(first, next_first), next_second)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First> Foldable for PairWithFirstBrand<First> {
		/// Folds the pair from the right (over second).
		///
		/// This method performs a right-associative fold of the pair (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The pair to fold.")]
		///
		/// ### Returns
		///
		/// `func(a, initial)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair((), 5)),
		/// 	5
		/// );
		/// ```
		fn fold_right<FnBrand, A, B, Func>(
			func: Func,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			Func: Fn(A, B) -> B,
			FnBrand: CloneableFn,
		{
			func(fa.1, initial)
		}

		/// Folds the pair from the left (over second).
		///
		/// This method performs a left-associative fold of the pair (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The identity to fold."
		)]
		///
		/// ### Returns
		///
		/// `func(initial, a)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair((), 5)),
		/// 	5
		/// );
		/// ```
		fn fold_left<FnBrand, A, B, Func>(
			func: Func,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			Func: Fn(B, A) -> B,
			FnBrand: CloneableFn,
		{
			func(initial, fa.1)
		}

		/// Maps the value to a monoid and returns it (over second).
		///
		/// This method maps the element of the pair to a monoid and then returns it (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The pair to fold.")]
		///
		/// ### Returns
		///
		/// `func(a)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, PairWithFirstBrand<()>, _, _, _>(|x: i32| x.to_string(), Pair((), 5)),
		/// 	"5".to_string()
		/// );
		/// ```
		fn fold_map<FnBrand, A, M, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			M: Monoid,
			Func: Fn(A) -> M,
			FnBrand: CloneableFn,
		{
			func(fa.1)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone> Traversable for PairWithFirstBrand<First> {
		/// Traverses the pair with an applicative function (over second).
		///
		/// This method maps the element of the pair to a computation, evaluates it, and combines the result into an applicative context (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The pair to traverse."
		)]
		///
		/// ### Returns
		///
		/// The pair wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<PairWithFirstBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Pair((), 5)),
		/// 	Some(Pair((), 10))
		/// );
		/// ```
		fn traverse<A: Clone, B: Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<T>; )>::Of<B>),
			Apply!(<Self as Kind!( type Of<T>; )>::Of<B>): Clone,
		{
			let Pair(first, second) = ta;
			F::map(move |b| Pair(first.clone(), b), func(second))
		}

		/// Sequences a pair of applicative (over second).
		///
		/// This method evaluates the computation inside the pair and accumulates the result into an applicative context (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The pair containing the applicative value.")]
		///
		/// ### Returns
		///
		/// The pair wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	sequence::<PairWithFirstBrand<()>, _, OptionBrand>(Pair((), Some(5))),
		/// 	Some(Pair((), 5))
		/// );
		/// ```
		fn sequence<A: Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<A>)>)
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<A>)>)
		where
			Apply!(<F as Kind!( type Of<T>; )>::Of<A>): Clone,
			Apply!(<Self as Kind!( type Of<T>; )>::Of<A>): Clone,
		{
			let Pair(first, second) = ta;
			F::map(move |a| Pair(first.clone(), a), second)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First> ParFoldable for PairWithFirstBrand<First> {
		/// Maps the value to a monoid and returns it in parallel (over second).
		///
		/// This method maps the element of the pair to a monoid and then returns it (over second). The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to map each element to a monoid.",
			"The pair to fold."
		)]
		///
		/// ### Returns
		///
		/// The combined monoid value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair("a".to_string(), 1);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		/// assert_eq!(par_fold_map::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, x), "1".to_string());
		/// ```
		fn par_fold_map<FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<A, M>,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			M: Monoid + Send + Sync,
		{
			func(fa.1)
		}

		/// Folds the pair from the right in parallel (over second).
		///
		/// This method folds the pair by applying a function from right to left, potentially in parallel (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to apply to each element and the accumulator.",
			"The initial value.",
			"The pair to fold."
		)]
		///
		/// ### Returns
		///
		/// The final accumulator value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair("a".to_string(), 1);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		/// assert_eq!(par_fold_right::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, 10, x), 11);
		/// ```
		fn par_fold_right<FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<(A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			B: Send + Sync,
		{
			func((fa.1, initial))
		}
	}
	// PairWithSecondBrand<Second> (Functor over First)

	impl_kind! {
		#[document_type_parameters("The type of the second value in the pair.")]
		impl<Second> for PairWithSecondBrand<Second> {
			type Of<A> = Pair<A, Second>;
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second> Functor for PairWithSecondBrand<Second> {
		/// Maps a function over the first value in the pair.
		///
		/// This method applies a function to the first value inside the pair, producing a new pair with the transformed first value. The second value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the first value.",
			"The type of the result of applying the function.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply to the first value.", "The pair to map over.")]
		///
		/// ### Returns
		///
		/// A new pair containing the result of applying the function to the first value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(map::<PairWithSecondBrand<_>, _, _, _>(|x: i32| x * 2, Pair(5, 1)), Pair(10, 1));
		/// ```
		fn map<A, B, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> B,
		{
			Pair(func(fa.0), fa.1)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone> Lift for PairWithSecondBrand<Second>
	where
		Second: Semigroup,
	{
		/// Lifts a binary function into the pair context (over first).
		///
		/// This method lifts a binary function to operate on the first values within the pair context. The second values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the first first value.",
			"The type of the second first value.",
			"The type of the result first value.",
			"The type of the binary function."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the first values.",
			"The first pair.",
			"The second pair."
		)]
		///
		/// ### Returns
		///
		/// A new pair where the first values are combined using `f` and the second values are combined using `Semigroup::append`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<PairWithSecondBrand<String>, _, _, _, _>(
		/// 		|x, y| x + y,
		/// 		Pair(1, "a".to_string()),
		/// 		Pair(2, "b".to_string())
		/// 	),
		/// 	Pair(3, "ab".to_string())
		/// );
		/// ```
		fn lift2<A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			fb: Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<C>)
		where
			Func: Fn(A, B) -> C,
			A: Clone,
			B: Clone,
		{
			Pair(func(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone> Pointed for PairWithSecondBrand<Second>
	where
		Second: Monoid,
	{
		/// Wraps a value in a pair (with empty second).
		///
		/// This method wraps a value in a pair, using the `Monoid::empty()` value for the second element.
		#[document_signature]
		///
		#[document_type_parameters("The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		/// ### Returns
		///
		/// A pair containing `a` and the empty value of the second type.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(pure::<PairWithSecondBrand<String>, _>(5), Pair(5, "".to_string()));
		/// ```
		fn pure<A>(a: A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<A>) {
			Pair(a, Monoid::empty())
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + Semigroup> ApplyFirst for PairWithSecondBrand<Second> {}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + Semigroup> ApplySecond for PairWithSecondBrand<Second> {}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone> Semiapplicative for PairWithSecondBrand<Second>
	where
		Second: Semigroup,
	{
		/// Applies a wrapped function to a wrapped value (over first).
		///
		/// This method applies a function wrapped in a result (as error) to a value wrapped in a result (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The pair containing the function.",
			"The pair containing the value."
		)]
		///
		/// ### Returns
		///
		/// A new pair where the first values are combined.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Pair(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2), "a".to_string());
		/// assert_eq!(
		/// 	apply::<RcFnBrand, PairWithSecondBrand<String>, _, _>(f, Pair(5, "b".to_string())),
		/// 	Pair(10, "ab".to_string())
		/// );
		/// ```
		fn apply<FnBrand: CloneableFn, A: Clone, B>(
			ff: Apply!(<Self as Kind!( type Of<T>; )>::Of<<FnBrand as CloneableFn>::Of<A, B>>),
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>) {
			Pair(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone> Semimonad for PairWithSecondBrand<Second>
	where
		Second: Semigroup,
	{
		/// Chains pair computations (over first).
		///
		/// This method chains two computations, where the second computation depends on the result of the first (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the result of the first computation.",
			"The type of the result of the second computation.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The first pair.", "The function to apply to the first value.")]
		///
		/// ### Returns
		///
		/// A new pair where the first values are combined.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bind::<PairWithSecondBrand<String>, _, _, _>(Pair(5, "a".to_string()), |x| Pair(
		/// 		x * 2,
		/// 		"b".to_string()
		/// 	)),
		/// 	Pair(10, "ab".to_string())
		/// );
		/// ```
		fn bind<A, B, Func>(
			ma: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<T>; )>::Of<B>),
		{
			let Pair(first, second) = ma;
			let Pair(next_first, next_second) = func(first);
			Pair(next_first, Semigroup::append(second, next_second))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second> Foldable for PairWithSecondBrand<Second> {
		/// Folds the pair from the right (over first).
		///
		/// This method performs a right-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The pair to fold.")]
		///
		/// ### Returns
		///
		/// `func(a, initial)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|x, acc| x + acc, 0, Pair(5, ())),
		/// 	5
		/// );
		/// ```
		fn fold_right<FnBrand, A, B, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			F: Fn(A, B) -> B,
			FnBrand: CloneableFn,
		{
			func(fa.0, initial)
		}

		/// Folds the pair from the left (over first).
		///
		/// This method performs a left-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator.",
			"The type of the folding function."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The pair to fold.")]
		///
		/// ### Returns
		///
		/// `func(initial, a)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(|acc, x| acc + x, 0, Pair(5, ())),
		/// 	5
		/// );
		/// ```
		fn fold_left<FnBrand, A, B, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			F: Fn(B, A) -> B,
			FnBrand: CloneableFn,
		{
			func(initial, fa.0)
		}

		/// Maps the value to a monoid and returns it (over first).
		///
		/// This method maps the element of the result to a monoid and then returns it (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid.",
			"The type of the mapping function."
		)]
		///
		#[document_parameters("The mapping function.", "The pair to fold.")]
		///
		/// ### Returns
		///
		/// `func(a)`.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, PairWithSecondBrand<()>, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Pair(5, ())
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn fold_map<FnBrand, A, M, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			M: Monoid,
			Func: Fn(A) -> M,
			FnBrand: CloneableFn,
		{
			func(fa.0)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone> Traversable for PairWithSecondBrand<Second> {
		/// Traverses the pair with an applicative function (over first).
		///
		/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context.",
			"The type of the function to apply."
		)]
		///
		#[document_parameters("The function to apply.", "The pair to traverse.")]
		///
		/// ### Returns
		///
		/// The pair wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<PairWithSecondBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Pair(5, ())),
		/// 	Some(Pair(10, ()))
		/// );
		/// ```
		fn traverse<A: Clone, B: Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<T>; )>::Of<B>),
			Apply!(<Self as Kind!( type Of<T>; )>::Of<B>): Clone,
		{
			let Pair(first, second) = ta;
			F::map(move |b| Pair(b, second.clone()), func(first))
		}

		/// Sequences a pair of applicative (over first).
		///
		/// This method evaluates the computation inside the result and accumulates the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The pair containing the applicative value.")]
		///
		/// ### Returns
		///
		/// The pair wrapped in the applicative context.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	sequence::<PairWithSecondBrand<()>, _, OptionBrand>(Pair(Some(5), ())),
		/// 	Some(Pair(5, ()))
		/// );
		/// ```
		fn sequence<A: Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<T>; )>::Of<Apply!(<F as Kind!( type Of<T>; )>::Of<A>)>)
		) -> Apply!(<F as Kind!( type Of<T>; )>::Of<Apply!(<Self as Kind!( type Of<T>; )>::Of<A>)>)
		where
			Apply!(<F as Kind!( type Of<T>; )>::Of<A>): Clone,
			Apply!(<Self as Kind!( type Of<T>; )>::Of<A>): Clone,
		{
			let Pair(first, second) = ta;
			F::map(move |a| Pair(a, second.clone()), first)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second> ParFoldable for PairWithSecondBrand<Second> {
		/// Maps the value to a monoid and returns it in parallel (over first).
		///
		/// This method maps the element of the pair to a monoid and then returns it (over first). The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to map each element to a monoid.",
			"The pair to fold."
		)]
		///
		/// ### Returns
		///
		/// The combined monoid value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(1, "a".to_string());
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, x),
		/// 	"1".to_string()
		/// );
		/// ```
		fn par_fold_map<FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<A, M>,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> M
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			M: Monoid + Send + Sync,
		{
			func(fa.0)
		}

		/// Folds the pair from the right in parallel (over first).
		///
		/// This method folds the pair by applying a function from right to left, potentially in parallel (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The brand of the cloneable function wrapper.",
			"The element type.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The thread-safe function to apply to each element and the accumulator.",
			"The initial value.",
			"The pair to fold."
		)]
		///
		/// ### Returns
		///
		/// The final accumulator value.
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(1, "a".to_string());
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		/// assert_eq!(par_fold_right::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, 10, x), 11);
		/// ```
		fn par_fold_right<FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<(A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
		) -> B
		where
			FnBrand: SendCloneableFn,
			A: Clone + Send + Sync,
			B: Send + Sync,
		{
			func((fa.0, initial))
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::inner::*,
		crate::{
			brands::*,
			classes::{CloneableFn, bifunctor::*},
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Bifunctor Tests

	/// Tests `bimap` on `Pair`.
	#[test]
	fn test_bimap() {
		let x = Pair(1, 5);
		assert_eq!(bimap::<PairBrand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, x), Pair(2, 10));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		bimap::<PairBrand, _, _, _, _, _, _>(identity, identity, x.clone()) == x
	}

	/// Tests the composition law for Bifunctor.
	#[quickcheck]
	fn bifunctor_composition(
		first: i32,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<PairBrand, _, _, _, _, _, _>(compose(f, g), compose(h, i), x)
			== bimap::<PairBrand, _, _, _, _, _, _>(
				f,
				h,
				bimap::<PairBrand, _, _, _, _, _, _>(g, i, x),
			)
	}

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		map::<PairWithFirstBrand<String>, _, _, _>(identity, x.clone()) == x
	}

	/// Tests the composition law for Functor.
	#[quickcheck]
	fn functor_composition(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<PairWithFirstBrand<String>, _, _, _>(compose(f, g), x.clone())
			== map::<PairWithFirstBrand<String>, _, _, _>(
				f,
				map::<PairWithFirstBrand<String>, _, _, _>(g, x),
			)
	}

	// Applicative Laws

	/// Tests the identity law for Applicative.
	#[quickcheck]
	fn applicative_identity(
		first: String,
		second: i32,
	) -> bool {
		let v = Pair(first, second);
		apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f)),
			pure::<PairWithFirstBrand<String>, _>(x),
		) == pure::<PairWithFirstBrand<String>, _>(f(x))
	}

	/// Tests the composition law for Applicative.
	#[quickcheck]
	fn applicative_composition(
		w_first: String,
		w_second: i32,
		u_seed: i32,
		v_seed: i32,
	) -> bool {
		let w = Pair(w_first, w_second);

		let u_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_add(u_seed));
		let u = pure::<PairWithFirstBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<PairWithFirstBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as CloneableFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as CloneableFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as CloneableFn>::new(move |x| f(g(x)))
			})
		});

		let pure_compose = pure::<PairWithFirstBrand<String>, _>(compose_fn);
		let u_applied = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(pure_compose, u);
		let uv = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(u_applied, v);
		let lhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(uv, w);

		lhs == rhs
	}

	/// Tests the interchange law for Applicative.
	#[quickcheck]
	fn applicative_interchange(
		y: i32,
		u_seed: i32,
	) -> bool {
		// u <*> pure y = pure ($ y) <*> u
		let f = move |x: i32| x.wrapping_mul(u_seed);
		let u = pure::<PairWithFirstBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			u.clone(),
			pure::<PairWithFirstBrand<String>, _>(y),
		);

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, PairWithFirstBrand<String>, _, _>(
			pure::<PairWithFirstBrand<String>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Pair("f".to_string(), x.wrapping_mul(2));
		bind::<PairWithFirstBrand<String>, _, _, _>(pure::<PairWithFirstBrand<String>, _>(a), f)
			== f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(
		first: String,
		second: i32,
	) -> bool {
		let m = Pair(first, second);
		bind::<PairWithFirstBrand<String>, _, _, _>(
			m.clone(),
			pure::<PairWithFirstBrand<String>, _>,
		) == m
	}

	/// Tests the associativity law for Monad.
	#[quickcheck]
	fn monad_associativity(
		first: String,
		second: i32,
	) -> bool {
		let m = Pair(first, second);
		let f = |x: i32| Pair("f".to_string(), x.wrapping_mul(2));
		let g = |x: i32| Pair("g".to_string(), x.wrapping_add(1));
		bind::<PairWithFirstBrand<String>, _, _, _>(
			bind::<PairWithFirstBrand<String>, _, _, _>(m.clone(), f),
			g,
		) == bind::<PairWithFirstBrand<String>, _, _, _>(m, |x| {
			bind::<PairWithFirstBrand<String>, _, _, _>(f(x), g)
		})
	}

	// ParFoldable Tests for PairWithFirstBrand (Functor over Second)

	/// Tests `par_fold_map` on `PairWithFirstBrand`.
	#[test]
	fn par_fold_map_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairWithFirstBrand`.
	#[test]
	fn par_fold_right_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairWithFirstBrand<String>, _, _>(f, 10, x), 11);
	}

	// ParFoldable Tests for PairWithSecondBrand (Functor over First)

	/// Tests `par_fold_map` on `PairWithSecondBrand`.
	#[test]
	fn par_fold_map_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairWithSecondBrand`.
	#[test]
	fn par_fold_right_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairWithSecondBrand<String>, _, _>(f, 10, x), 11);
	}
}
