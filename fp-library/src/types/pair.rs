//! Two-value container with [`Bifunctor`](crate::classes::Bifunctor) and dual [`Functor`](crate::classes::Functor) instances.
//!
//! Can be used as a bifunctor over both values, or as a functor/monad by fixing either the first value [`PairFirstAppliedBrand`](crate::brands::PairFirstAppliedBrand) or second value [`PairSecondAppliedBrand`](crate::brands::PairSecondAppliedBrand).

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				PairBrand,
				PairFirstAppliedBrand,
				PairSecondAppliedBrand,
			},
			classes::{
				Applicative,
				ApplyFirst,
				ApplySecond,
				Bifoldable,
				Bifunctor,
				Bitraversable,
				CloneableFn,
				Foldable,
				Functor,
				Lift,
				Monoid,
				ParFoldable,
				Pointed,
				Semiapplicative,
				Semigroup,
				Semimonad,
				SendCloneableFn,
				Traversable,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Wraps two values.
	///
	/// A simple tuple struct that holds two values of potentially different types.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// This type has multiple higher-kinded representations:
	/// - [`PairBrand`](crate::brands::PairBrand): fully polymorphic over both values (bifunctor).
	/// - [`PairFirstAppliedBrand<First>`](crate::brands::PairFirstAppliedBrand): the first value type is fixed, polymorphic over the second (functor over second).
	/// - [`PairSecondAppliedBrand<Second>`](crate::brands::PairSecondAppliedBrand): the second value type is fixed, polymorphic over the first (functor over first).
	///
	/// ### Serialization
	///
	/// This type supports serialization and deserialization via [`serde`](https://serde.rs) when the `serde` feature is enabled.
	#[document_type_parameters("The type of the first value.", "The type of the second value.")]
	///
	#[document_fields("The first value.", "The second value.")]
	///
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Pair<First, Second>(pub First, pub Second);

	impl_kind! {
		for PairBrand {
			type Of<First,Second> = Pair<First, Second>;
		}
	}

	impl_kind! {
		for PairBrand {
			type Of<'a, First: 'a, Second: 'a>: 'a = Pair<First, Second>;
		}
	}

	#[document_type_parameters("The type of the first value.", "The type of the second value.")]
	#[document_parameters("The pair instance.")]
	impl<First, Second> Pair<First, Second> {
		/// Maps functions over both values in the pair.
		///
		/// See [`Bifunctor::bimap`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type of the mapped first value.",
			"The type of the mapped second value."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The function to apply to the second value."
		)]
		///
		#[document_returns("A new pair containing the mapped values.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(1, 5);
		/// assert_eq!(x.bimap(|a| a + 1, |b| b * 2), Pair(2, 10));
		/// ```
		pub fn bimap<B, D>(
			self,
			f: impl FnOnce(First) -> B,
			g: impl FnOnce(Second) -> D,
		) -> Pair<B, D> {
			Pair(f(self.0), g(self.1))
		}

		/// Maps a function over the first value in the pair.
		///
		/// See [`Bifunctor::bimap`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The type of the mapped first value.")]
		///
		#[document_parameters("The function to apply to the first value.")]
		///
		#[document_returns("A new pair with the transformed first value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(1, 5);
		/// assert_eq!(x.map_first(|a| a + 1), Pair(2, 5));
		/// ```
		pub fn map_first<B>(
			self,
			f: impl FnOnce(First) -> B,
		) -> Pair<B, Second> {
			Pair(f(self.0), self.1)
		}

		/// Maps a function over the second value in the pair.
		///
		/// See [`Bifunctor::bimap`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The type of the mapped second value.")]
		///
		#[document_parameters("The function to apply to the second value.")]
		///
		#[document_returns("A new pair with the transformed second value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(1, 5);
		/// assert_eq!(x.map_second(|b| b * 2), Pair(1, 10));
		/// ```
		pub fn map_second<D>(
			self,
			g: impl FnOnce(Second) -> D,
		) -> Pair<First, D> {
			Pair(self.0, g(self.1))
		}

		/// Folds both values into a single result.
		///
		/// Applies two functions to the first and second values respectively,
		/// then combines the results using `FnOnce`.
		#[document_signature]
		///
		#[document_type_parameters("The result type.")]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The function to apply to the second value.",
			"The function to combine the results."
		)]
		///
		#[document_returns("The combined result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(1, 2);
		/// let y = x.fold(|a| a.to_string(), |b| b.to_string(), |a, b| format!("{a},{b}"));
		/// assert_eq!(y, "1,2");
		/// ```
		pub fn fold<C>(
			self,
			f: impl FnOnce(First) -> C,
			g: impl FnOnce(Second) -> C,
			combine: impl FnOnce(C, C) -> C,
		) -> C {
			combine(f(self.0), g(self.1))
		}

		/// Folds the pair from right to left using two step functions.
		///
		/// See [`Bifoldable::bi_fold_right`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The accumulator type.")]
		///
		#[document_parameters(
			"The step function for the first value.",
			"The step function for the second value.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding: `f(first, g(second, z))`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(x.bi_fold_right(|a, acc| acc - a, |b, acc| acc + b, 0), 2);
		/// ```
		pub fn bi_fold_right<C>(
			self,
			f: impl FnOnce(First, C) -> C,
			g: impl FnOnce(Second, C) -> C,
			z: C,
		) -> C {
			f(self.0, g(self.1, z))
		}

		/// Folds the pair from left to right using two step functions.
		///
		/// See [`Bifoldable::bi_fold_left`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The accumulator type.")]
		///
		#[document_parameters(
			"The step function for the first value.",
			"The step function for the second value.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding: `g(f(z, first), second)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(x.bi_fold_left(|acc, a| acc - a, |acc, b| acc + b, 0), 2);
		/// ```
		pub fn bi_fold_left<C>(
			self,
			f: impl FnOnce(C, First) -> C,
			g: impl FnOnce(C, Second) -> C,
			z: C,
		) -> C {
			g(f(z, self.0), self.1)
		}

		/// Maps both values to a monoid and combines the results.
		///
		/// See [`Bifoldable::bi_fold_map`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type.")]
		///
		#[document_parameters(
			"The function mapping the first value to the monoid.",
			"The function mapping the second value to the monoid."
		)]
		///
		#[document_returns("The combined monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(x.bi_fold_map(|a: i32| a.to_string(), |b: i32| b.to_string()), "35".to_string());
		/// ```
		pub fn bi_fold_map<M: Semigroup>(
			self,
			f: impl FnOnce(First) -> M,
			g: impl FnOnce(Second) -> M,
		) -> M {
			Semigroup::append(f(self.0), g(self.1))
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The type of the first value.",
		"The type of the second value."
	)]
	#[document_parameters("The pair instance.")]
	impl<'a, First: 'a, Second: 'a> Pair<First, Second> {
		/// Traverses the pair with two effectful functions.
		///
		/// See [`Bitraversable::bi_traverse`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The output type for the first value.",
			"The output type for the second value.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function for the first value.",
			"The function for the second value."
		)]
		///
		#[document_returns("A pair of the transformed values wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(3, 5);
		/// let y = x.bi_traverse::<_, _, OptionBrand>(|a| Some(a + 1), |b| Some(b * 2));
		/// assert_eq!(y, Some(Pair(4, 10)));
		/// ```
		pub fn bi_traverse<C: 'a + Clone, D: 'a + Clone, F: Applicative>(
			self,
			f: impl Fn(First) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(Second) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Pair<C, D>>)
		where
			Pair<C, D>: Clone, {
			F::lift2(|c, d| Pair(c, d), f(self.0), g(self.1))
		}
	}

	#[document_type_parameters("The type of the first value.", "The type of the second value.")]
	#[document_parameters("The pair instance.")]
	impl<First: Semigroup, Second> Pair<First, Second> {
		/// Chains a computation over the second value, combining first values via their semigroup.
		///
		/// See [`Semimonad::bind`] for the type class version
		/// (via [`PairFirstAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The type of the new second value.")]
		///
		#[document_parameters("The function to apply to the second value.")]
		///
		#[document_returns(
			"A new pair where the first values are combined and the second value is transformed."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// assert_eq!(
		/// 	Pair("a".to_string(), 5).bind(|x| Pair("b".to_string(), x * 2)),
		/// 	Pair("ab".to_string(), 10)
		/// );
		/// ```
		pub fn bind<C>(
			self,
			f: impl FnOnce(Second) -> Pair<First, C>,
		) -> Pair<First, C> {
			let Pair(first, second) = self;
			let Pair(next_first, next_second) = f(second);
			Pair(Semigroup::append(first, next_first), next_second)
		}
	}

	#[document_type_parameters("The type of the first value.", "The type of the second value.")]
	#[document_parameters("The pair instance.")]
	impl<First, Second: Semigroup> Pair<First, Second> {
		/// Chains a computation over the first value, combining second values via their semigroup.
		///
		/// See [`Semimonad::bind`] for the type class version
		/// (via [`PairSecondAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The type of the new first value.")]
		///
		#[document_parameters("The function to apply to the first value.")]
		///
		#[document_returns(
			"A new pair where the first value is transformed and the second values are combined."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// assert_eq!(
		/// 	Pair(5, "a".to_string()).bind_first(|x| Pair(x * 2, "b".to_string())),
		/// 	Pair(10, "ab".to_string())
		/// );
		/// ```
		pub fn bind_first<C>(
			self,
			f: impl FnOnce(First) -> Pair<C, Second>,
		) -> Pair<C, Second> {
			let Pair(first, second) = self;
			let Pair(next_first, next_second) = f(first);
			Pair(next_first, Semigroup::append(second, next_second))
		}
	}

	impl Bifunctor for PairBrand {
		/// Maps functions over the values in the pair.
		///
		/// This method applies one function to the first value and another to the second value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the mapped first value.",
			"The type of the second value.",
			"The type of the mapped second value."
		)]
		///
		#[document_parameters(
			"The function to apply to the first value.",
			"The function to apply to the second value.",
			"The pair to map over."
		)]
		///
		#[document_returns("A new pair containing the mapped values.")]
		#[document_examples]
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
		/// assert_eq!(bimap::<PairBrand, _, _, _, _>(|a| a + 1, |b| b * 2, x), Pair(2, 10));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			p.bimap(f, g)
		}
	}

	impl Bifoldable for PairBrand {
		/// Folds the pair from right to left using two step functions.
		///
		/// Folds `Pair(a, b)` as `f(a, g(b, z))`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first value.",
			"The type of the second value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for the first value.",
			"The step function for the second value.",
			"The initial accumulator.",
			"The pair to fold."
		)]
		///
		#[document_returns("`f(a, g(b, z))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, PairBrand, _, _, _>(|a, acc| acc - a, |b, acc| acc + b, 0, x,),
		/// 	2
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(A, C) -> C + 'a,
			g: impl Fn(B, C) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			p.bi_fold_right(f, g, z)
		}

		/// Folds the pair from left to right using two step functions.
		///
		/// Folds `Pair(a, b)` as `g(f(z, a), b)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first value.",
			"The type of the second value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for the first value.",
			"The step function for the second value.",
			"The initial accumulator.",
			"The pair to fold."
		)]
		///
		#[document_returns("`g(f(z, a), b)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, PairBrand, _, _, _>(|acc, a| acc - a, |acc, b| acc + b, 0, x,),
		/// 	2
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(C, A) -> C + 'a,
			g: impl Fn(C, B) -> C + 'a,
			z: C,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			p.bi_fold_left(f, g, z)
		}

		/// Maps both values to a monoid and combines the results.
		///
		/// Computes `Semigroup::append(f(a), g(b))` for `Pair(a, b)`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the first value.",
			"The type of the second value.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping the first value to the monoid.",
			"The function mapping the second value to the monoid.",
			"The pair to fold."
		)]
		///
		#[document_returns("`Semigroup::append(f(a), g(b))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, PairBrand, _, _, _>(
		/// 		|a: i32| a.to_string(),
		/// 		|b: i32| b.to_string(),
		/// 		Pair(3, 5),
		/// 	),
		/// 	"35".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
			f: impl Fn(A) -> M + 'a,
			g: impl Fn(B) -> M + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> M
		where
			M: Monoid + 'a, {
			p.bi_fold_map(f, g)
		}
	}

	impl Bitraversable for PairBrand {
		/// Traverses the pair with two effectful functions.
		///
		/// Applies `f` to the first value and `g` to the second value,
		/// combining the effects via `lift2`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The output type for the first value.",
			"The output type for the second value.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to the first value.",
			"The function applied to the second value.",
			"The pair to traverse."
		)]
		///
		#[document_returns("`lift2(Pair, f(a), g(b))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(
		/// 	bi_traverse::<PairBrand, _, _, _, _, OptionBrand>(
		/// 		|a: i32| Some(a + 1),
		/// 		|b: i32| Some(b * 2),
		/// 		x,
		/// 	),
		/// 	Some(Pair(4, 10))
		/// );
		/// ```
		fn bi_traverse<
			'a,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		{
			p.bi_traverse::<C, D, F>(f, g)
		}
	}

	// PairFirstAppliedBrand<First> (Functor over Second)

	impl_kind! {
		#[document_type_parameters("The type of the first value in the pair.")]
		impl<First: 'static> for PairFirstAppliedBrand<First> {
			type Of<'a, A: 'a>: 'a = Pair<First, A>;
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: 'static> Functor for PairFirstAppliedBrand<First> {
		/// Maps a function over the second value in the pair.
		///
		/// This method applies a function to the second value inside the pair, producing a new pair with the transformed second value. The first value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the second value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the second value.",
			"The pair to map over."
		)]
		///
		#[document_returns(
			"A new pair containing the result of applying the function to the second value."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(map::<PairFirstAppliedBrand<_>, _, _>(|x: i32| x * 2, Pair(1, 5)), Pair(1, 10));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map_second(func)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> Lift for PairFirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Lifts a binary function into the pair context (over second).
		///
		/// This method lifts a binary function to operate on the second values within the pair context. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first second value.",
			"The type of the second second value.",
			"The type of the result second value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the second values.",
			"The first pair.",
			"The second pair."
		)]
		///
		#[document_returns(
			"A new pair where the first values are combined using `Semigroup::append` and the second values are combined using `f`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<PairFirstAppliedBrand<String>, _, _, _>(
		/// 		|x, y| x + y,
		/// 		Pair("a".to_string(), 1),
		/// 		Pair("b".to_string(), 2)
		/// 	),
		/// 	Pair("ab".to_string(), 3)
		/// );
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			let Pair(fa_first, fa_second) = fa;
			let Pair(fb_first, fb_second) = fb;
			Pair(Semigroup::append(fa_first, fb_first), func(fa_second, fb_second))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> Pointed for PairFirstAppliedBrand<First>
	where
		First: Monoid,
	{
		/// Wraps a value in a pair (with empty first).
		///
		/// This method wraps a value in a pair, using the `Monoid::empty()` value for the first element.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A pair containing the empty value of the first type and `a`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(pure::<PairFirstAppliedBrand<String>, _>(5), Pair("".to_string(), 5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Pair(Monoid::empty(), a)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + Semigroup + 'static> ApplyFirst for PairFirstAppliedBrand<First> {}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + Semigroup + 'static> ApplySecond for PairFirstAppliedBrand<First> {}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> Semiapplicative for PairFirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Applies a wrapped function to a wrapped value (over second).
		///
		/// This method applies a function wrapped in a pair to a value wrapped in a pair. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns(
			"A new pair where the first values are combined and the function is applied to the second value."
		)]
		#[document_examples]
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
		/// 	apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(f, Pair("b".to_string(), 5)),
		/// 	Pair("ab".to_string(), 10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Pair(Semigroup::append(ff.0, fa.0), ff.1(fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> Semimonad for PairFirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Chains pair computations (over second).
		///
		/// This method chains two computations, where the second computation depends on the result of the first. The first values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters("The first pair.", "The function to apply to the second value.")]
		///
		#[document_returns("A new pair where the first values are combined.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bind::<PairFirstAppliedBrand<String>, _, _>(Pair("a".to_string(), 5), |x| Pair(
		/// 		"b".to_string(),
		/// 		x * 2
		/// 	)),
		/// 	Pair("ab".to_string(), 10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: 'static> Foldable for PairFirstAppliedBrand<First> {
		/// Folds the pair from the right (over second).
		///
		/// This method performs a right-associative fold of the pair (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The pair to fold.")]
		///
		#[document_returns("`func(a, initial)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, PairFirstAppliedBrand<()>, _, _>(|x, acc| x + acc, 0, Pair((), 5)),
		/// 	5
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(fa.1, initial)
		}

		/// Folds the pair from the left (over second).
		///
		/// This method performs a left-associative fold of the pair (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The identity to fold."
		)]
		///
		#[document_returns("`func(initial, a)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, PairFirstAppliedBrand<()>, _, _>(|acc, x| acc + x, 0, Pair((), 5)),
		/// 	5
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(initial, fa.1)
		}

		/// Maps the value to a monoid and returns it (over second).
		///
		/// This method maps the element of the pair to a monoid and then returns it (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The pair to fold.")]
		///
		#[document_returns("`func(a)`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, PairFirstAppliedBrand<()>, _, _>(|x: i32| x.to_string(), Pair((), 5)),
		/// 	"5".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			func(fa.1)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> Traversable for PairFirstAppliedBrand<First> {
		/// Traverses the pair with an applicative function (over second).
		///
		/// This method maps the element of the pair to a computation, evaluates it, and combines the result into an applicative context (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function to apply to each element, returning a value in an applicative context.",
			"The pair to traverse."
		)]
		///
		#[document_returns("The pair wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<PairFirstAppliedBrand<()>, _, _, OptionBrand>(|x| Some(x * 2), Pair((), 5)),
		/// 	Some(Pair((), 10))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let Pair(first, second) = ta;
			F::map(move |b| Pair(first.clone(), b), func(second))
		}

		/// Sequences a pair of applicative (over second).
		///
		/// This method evaluates the computation inside the pair and accumulates the result into an applicative context (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The pair containing the applicative value.")]
		///
		#[document_returns("The pair wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	sequence::<PairFirstAppliedBrand<()>, _, OptionBrand>(Pair((), Some(5))),
		/// 	Some(Pair((), 5))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			let Pair(first, second) = ta;
			F::map(move |a| Pair(first.clone(), a), second)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: 'static> ParFoldable for PairFirstAppliedBrand<First> {
		/// Maps the value to a monoid and returns it in parallel (over second).
		///
		/// This method maps the element of the pair to a monoid and then returns it (over second). The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("The combined monoid value.")]
		#[document_examples]
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
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, PairFirstAppliedBrand<String>, _, _>(f, x),
		/// 	"1".to_string()
		/// );
		/// ```
		fn par_fold_map<'a, FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			M: Monoid + Send + Sync + 'a, {
			func(fa.1)
		}

		/// Folds the pair from the right in parallel (over second).
		///
		/// This method folds the pair by applying a function from right to left, potentially in parallel (over second).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("The final accumulator value.")]
		#[document_examples]
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
		/// assert_eq!(par_fold_right::<ArcFnBrand, PairFirstAppliedBrand<String>, _, _>(f, 10, x), 11);
		/// ```
		fn par_fold_right<'a, FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			B: Send + Sync + 'a, {
			func((fa.1, initial))
		}
	}
	// PairSecondAppliedBrand<Second> (Functor over First)

	impl_kind! {
		#[document_type_parameters("The type of the second value in the pair.")]
		impl<Second: 'static> for PairSecondAppliedBrand<Second> {
			type Of<'a, A: 'a>: 'a = Pair<A, Second>;
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: 'static> Functor for PairSecondAppliedBrand<Second> {
		/// Maps a function over the first value in the pair.
		///
		/// This method applies a function to the first value inside the pair, producing a new pair with the transformed first value. The second value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply to the first value.", "The pair to map over.")]
		///
		#[document_returns(
			"A new pair containing the result of applying the function to the first value."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(map::<PairSecondAppliedBrand<_>, _, _>(|x: i32| x * 2, Pair(5, 1)), Pair(10, 1));
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map_first(func)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> Lift for PairSecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Lifts a binary function into the pair context (over first).
		///
		/// This method lifts a binary function to operate on the first values within the pair context. The second values are combined using their `Semigroup` implementation.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first first value.",
			"The type of the second first value.",
			"The type of the result first value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the first values.",
			"The first pair.",
			"The second pair."
		)]
		///
		#[document_returns(
			"A new pair where the first values are combined using `f` and the second values are combined using `Semigroup::append`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	lift2::<PairSecondAppliedBrand<String>, _, _, _>(
		/// 		|x, y| x + y,
		/// 		Pair(1, "a".to_string()),
		/// 		Pair(2, "b".to_string())
		/// 	),
		/// 	Pair(3, "ab".to_string())
		/// );
		/// ```
		fn lift2<'a, A, B, C>(
			func: impl Fn(A, B) -> C + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			A: Clone + 'a,
			B: Clone + 'a,
			C: 'a, {
			Pair(func(fa.0, fb.0), Semigroup::append(fa.1, fb.1))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> Pointed for PairSecondAppliedBrand<Second>
	where
		Second: Monoid,
	{
		/// Wraps a value in a pair (with empty second).
		///
		/// This method wraps a value in a pair, using the `Monoid::empty()` value for the second element.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("A pair containing `a` and the empty value of the second type.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(pure::<PairSecondAppliedBrand<String>, _>(5), Pair(5, "".to_string()));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Pair(a, Monoid::empty())
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + Semigroup + 'static> ApplyFirst for PairSecondAppliedBrand<Second> {}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + Semigroup + 'static> ApplySecond for PairSecondAppliedBrand<Second> {}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> Semiapplicative for PairSecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Applies a wrapped function to a wrapped value (over first).
		///
		/// This method applies a function wrapped in a result (as error) to a value wrapped in a result (as error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the input value.",
			"The type of the output value."
		)]
		///
		#[document_parameters(
			"The pair containing the function (in Err).",
			"The pair containing the value (in Err)."
		)]
		///
		#[document_returns(
			"`Err(f(a))` if both are `Err`, otherwise the first success encountered."
		)]
		#[document_examples]
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
		/// 	apply::<RcFnBrand, PairSecondAppliedBrand<String>, _, _>(f, Pair(5, "b".to_string())),
		/// 	Pair(10, "ab".to_string())
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Pair(ff.0(fa.0), Semigroup::append(ff.1, fa.1))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> Semimonad for PairSecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Chains pair computations (over first).
		///
		/// This method chains two computations, where the second computation depends on the result of the first (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters("The first result.", "The function to apply to the error value.")]
		///
		#[document_returns(
			"The result of applying `f` to the error if `ma` is `Err`, otherwise the original success."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	bind::<PairSecondAppliedBrand<String>, _, _>(Pair(5, "a".to_string()), |x| Pair(
		/// 		x * 2,
		/// 		"b".to_string()
		/// 	)),
		/// 	Pair(10, "ab".to_string())
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind_first(func)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: 'static> Foldable for PairSecondAppliedBrand<Second> {
		/// Folds the pair from the right (over first).
		///
		/// This method performs a right-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Err(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, PairSecondAppliedBrand<()>, _, _>(|x, acc| x + acc, 0, Pair(5, ())),
		/// 	5
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(fa.0, initial)
		}

		/// Folds the pair from the left (over first).
		///
		/// This method performs a left-associative fold of the result (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The result to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Err(a)`, otherwise `initial`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, PairSecondAppliedBrand<()>, _, _>(|acc, x| acc + x, 0, Pair(5, ())),
		/// 	5
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			func(initial, fa.0)
		}

		/// Maps the value to a monoid and returns it (over first).
		///
		/// This method maps the element of the result to a monoid and then returns it (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The result to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Err(a)`, otherwise `M::empty()`.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, PairSecondAppliedBrand<()>, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Pair(5, ())
		/// 	),
		/// 	"5".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			func(fa.0)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> Traversable for PairSecondAppliedBrand<Second> {
		/// Traverses the pair with an applicative function (over first).
		///
		/// This method maps the element of the result to a computation, evaluates it, and combines the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The result to traverse.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	traverse::<PairSecondAppliedBrand<()>, _, _, OptionBrand>(|x| Some(x * 2), Pair(5, ())),
		/// 	Some(Pair(10, ()))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let Pair(first, second) = ta;
			F::map(move |b| Pair(b, second.clone()), func(first))
		}

		/// Sequences a pair of applicative (over first).
		///
		/// This method evaluates the computation inside the result and accumulates the result into an applicative context (over error).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The result containing the applicative value.")]
		///
		#[document_returns("The result wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// assert_eq!(
		/// 	sequence::<PairSecondAppliedBrand<()>, _, OptionBrand>(Pair(Some(5), ())),
		/// 	Some(Pair(5, ()))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			let Pair(first, second) = ta;
			F::map(move |a| Pair(a, second.clone()), first)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: 'static> ParFoldable for PairSecondAppliedBrand<Second> {
		/// Maps the value to a monoid and returns it in parallel (over first).
		///
		/// This method maps the element of the pair to a monoid and then returns it (over first). The mapping operation may be executed in parallel.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("The combined monoid value.")]
		#[document_examples]
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
		/// 	par_fold_map::<ArcFnBrand, PairSecondAppliedBrand<String>, _, _>(f, x),
		/// 	"1".to_string()
		/// );
		/// ```
		fn par_fold_map<'a, FnBrand, A, M>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			M: Monoid + Send + Sync + 'a, {
			func(fa.0)
		}

		/// Folds the pair from the right in parallel (over first).
		///
		/// This method folds the pair by applying a function from right to left, potentially in parallel (over first).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
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
		#[document_returns("The final accumulator value.")]
		#[document_examples]
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
		/// assert_eq!(par_fold_right::<ArcFnBrand, PairSecondAppliedBrand<String>, _, _>(f, 10, x), 11);
		/// ```
		fn par_fold_right<'a, FnBrand, A, B>(
			func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: 'a + SendCloneableFn,
			A: 'a + Clone + Send + Sync,
			B: Send + Sync + 'a, {
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
			classes::{
				CloneableFn,
				bifunctor::*,
			},
			functions::*,
		},
		quickcheck_macros::quickcheck,
	};

	// Bifunctor Tests

	/// Tests `bimap` on `Pair`.
	#[test]
	fn test_bimap() {
		let x = Pair(1, 5);
		assert_eq!(bimap::<PairBrand, _, _, _, _>(|a| a + 1, |b| b * 2, x), Pair(2, 10));
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		bimap::<PairBrand, _, _, _, _>(identity, identity, x.clone()) == x
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

		bimap::<PairBrand, _, _, _, _>(compose(f, g), compose(h, i), x)
			== bimap::<PairBrand, _, _, _, _>(f, h, bimap::<PairBrand, _, _, _, _>(g, i, x))
	}

	// Functor Laws

	/// Tests the identity law for Functor.
	#[quickcheck]
	fn functor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		map::<PairFirstAppliedBrand<String>, _, _>(identity, x.clone()) == x
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
		map::<PairFirstAppliedBrand<String>, _, _>(compose(f, g), x.clone())
			== map::<PairFirstAppliedBrand<String>, _, _>(
				f,
				map::<PairFirstAppliedBrand<String>, _, _>(g, x),
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
		apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			pure::<PairFirstAppliedBrand<String>, _>(<RcFnBrand as CloneableFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			pure::<PairFirstAppliedBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f)),
			pure::<PairFirstAppliedBrand<String>, _>(x),
		) == pure::<PairFirstAppliedBrand<String>, _>(f(x))
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
		let u = pure::<PairFirstAppliedBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as CloneableFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<PairFirstAppliedBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as CloneableFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as CloneableFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as CloneableFn>::new(move |x| f(g(x)))
			})
		});

		let pure_compose = pure::<PairFirstAppliedBrand<String>, _>(compose_fn);
		let u_applied = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(pure_compose, u);
		let uv = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(u_applied, v);
		let lhs = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(uv, w);

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
		let u = pure::<PairFirstAppliedBrand<String>, _>(<RcFnBrand as CloneableFn>::new(f));

		let lhs = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			u.clone(),
			pure::<PairFirstAppliedBrand<String>, _>(y),
		);

		let rhs_fn =
			<RcFnBrand as CloneableFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			pure::<PairFirstAppliedBrand<String>, _>(rhs_fn),
			u,
		);

		lhs == rhs
	}

	// Monad Laws

	/// Tests the left identity law for Monad.
	#[quickcheck]
	fn monad_left_identity(a: i32) -> bool {
		let f = |x: i32| Pair("f".to_string(), x.wrapping_mul(2));
		bind::<PairFirstAppliedBrand<String>, _, _>(pure::<PairFirstAppliedBrand<String>, _>(a), f)
			== f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(
		first: String,
		second: i32,
	) -> bool {
		let m = Pair(first, second);
		bind::<PairFirstAppliedBrand<String>, _, _>(
			m.clone(),
			pure::<PairFirstAppliedBrand<String>, _>,
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
		bind::<PairFirstAppliedBrand<String>, _, _>(
			bind::<PairFirstAppliedBrand<String>, _, _>(m.clone(), f),
			g,
		) == bind::<PairFirstAppliedBrand<String>, _, _>(m, |x| {
			bind::<PairFirstAppliedBrand<String>, _, _>(f(x), g)
		})
	}

	// ParFoldable Tests for PairFirstAppliedBrand (Functor over Second)

	/// Tests `par_fold_map` on `PairFirstAppliedBrand`.
	#[test]
	fn par_fold_map_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairFirstAppliedBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairFirstAppliedBrand`.
	#[test]
	fn par_fold_right_pair_with_first() {
		let x = Pair("a".to_string(), 1);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(par_fold_right::<ArcFnBrand, PairFirstAppliedBrand<String>, _, _>(f, 10, x), 11);
	}

	// ParFoldable Tests for PairSecondAppliedBrand (Functor over First)

	/// Tests `par_fold_map` on `PairSecondAppliedBrand`.
	#[test]
	fn par_fold_map_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		assert_eq!(
			par_fold_map::<ArcFnBrand, PairSecondAppliedBrand<String>, _, _>(f, x),
			"1".to_string()
		);
	}

	/// Tests `par_fold_right` on `PairSecondAppliedBrand`.
	#[test]
	fn par_fold_right_pair_with_second() {
		let x = Pair(1, "a".to_string());
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		assert_eq!(
			par_fold_right::<ArcFnBrand, PairSecondAppliedBrand<String>, _, _>(f, 10, x),
			11
		);
	}
}
