//! Two-value container with [`Bifunctor`](crate::classes::Bifunctor) and dual [`Functor`](crate::classes::Functor) instances.
//!
//! Can be used as a bifunctor over both values [`PairBrand`](crate::brands::PairBrand), or as a functor/monad by fixing either the first value [`PairFirstAppliedBrand`](crate::brands::PairFirstAppliedBrand) or second value [`PairSecondAppliedBrand`](crate::brands::PairSecondAppliedBrand).

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
			classes::*,
			dispatch::Ref,
			impl_kind,
			kinds::*,
		},
		core::ops::ControlFlow,
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
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
	pub struct Pair<First, Second>(
		/// The first value.
		pub First,
		/// The second value.
		pub Second,
	);

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
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(1, 5);
		/// assert_eq!(
		/// 	explicit::bimap::<PairBrand, _, _, _, _, _, _>((|a| a + 1, |b| b * 2), x),
		/// 	Pair(2, 10)
		/// );
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			p.bimap(f, g)
		}
	}

	impl RefBifunctor for PairBrand {
		/// Maps functions over the values in the pair by reference.
		///
		/// This method applies one function to a reference of the first value and another
		/// to a reference of the second value, producing a new pair with mapped values.
		/// The original pair is borrowed, not consumed.
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
			"The function to apply to a reference of the first value.",
			"The function to apply to a reference of the second value.",
			"The pair to map over by reference."
		)]
		///
		#[document_returns("A new pair containing the mapped values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_bifunctor::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(1, 5);
		/// assert_eq!(ref_bimap::<PairBrand, _, _, _, _>(|a| *a + 1, |b| *b * 2, &x), Pair(2, 10));
		/// ```
		fn ref_bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(&A) -> B + 'a,
			g: impl Fn(&C) -> D + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			Pair(f(&p.0), g(&p.1))
		}
	}

	impl RefBifoldable for PairBrand {
		/// Folds the pair from right to left by reference using two step functions.
		///
		/// Folds `Pair(a, b)` as `f(&a, g(&b, z))` without consuming the pair.
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
			"The step function for a reference to the first value.",
			"The step function for a reference to the second value.",
			"The initial accumulator.",
			"The pair to fold by reference."
		)]
		///
		#[document_returns("`f(&a, g(&b, z))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_bifoldable::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(
		/// 	ref_bi_fold_right::<RcFnBrand, PairBrand, _, _, _>(
		/// 		|a: &i32, acc| acc - *a,
		/// 		|b: &i32, acc| acc + *b,
		/// 		0,
		/// 		&x,
		/// 	),
		/// 	2
		/// );
		/// ```
		fn ref_bi_fold_right<'a, FnBrand: LiftFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
			f: impl Fn(&A, C) -> C + 'a,
			g: impl Fn(&B, C) -> C + 'a,
			z: C,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> C {
			f(&p.0, g(&p.1, z))
		}
	}

	impl RefBitraversable for PairBrand {
		/// Traverses the pair by reference with two effectful functions.
		///
		/// Applies `f` to a reference to the first value and `g` to a reference to the second
		/// value, combining the effects via `lift2` without consuming the pair.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function wrapper.",
			"The type of the first value.",
			"The type of the second value.",
			"The output type for the first value.",
			"The output type for the second value.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to a reference to the first value.",
			"The function applied to a reference to the second value.",
			"The pair to traverse by reference."
		)]
		///
		#[document_returns("`lift2(Pair, f(&a), g(&b))`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_bitraversable::*,
		/// 	types::*,
		/// };
		///
		/// let x = Pair(3, 5);
		/// assert_eq!(
		/// 	ref_bi_traverse::<PairBrand, RcFnBrand, _, _, _, _, OptionBrand>(
		/// 		|a: &i32| Some(a + 1),
		/// 		|b: &i32| Some(b * 2),
		/// 		&x,
		/// 	),
		/// 	Some(Pair(4, 10))
		/// );
		/// ```
		fn ref_bi_traverse<
			'a,
			FnBrand,
			A: 'a + Clone,
			B: 'a + Clone,
			C: 'a + Clone,
			D: 'a + Clone,
			F: Applicative,
		>(
			f: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(&B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
			p: &Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, C, D>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>): Clone, {
			F::lift2(|c, d| Pair(c, d), f(&p.0), g(&p.1))
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
		/// 	explicit::bi_fold_right::<RcFnBrand, PairBrand, _, _, _, _, _>(
		/// 		(|a, acc| acc - a, |b, acc| acc + b),
		/// 		0,
		/// 		x
		/// 	),
		/// 	2
		/// );
		/// ```
		fn bi_fold_right<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
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
		/// 	explicit::bi_fold_left::<RcFnBrand, PairBrand, _, _, _, _, _>(
		/// 		(|acc, a| acc - a, |acc, b| acc + b),
		/// 		0,
		/// 		x
		/// 	),
		/// 	2
		/// );
		/// ```
		fn bi_fold_left<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a>(
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
		/// 	explicit::bi_fold_map::<RcFnBrand, PairBrand, _, _, _, _, _>(
		/// 		(|a: i32| a.to_string(), |b: i32| b.to_string()),
		/// 		Pair(3, 5),
		/// 	),
		/// 	"35".to_string()
		/// );
		/// ```
		fn bi_fold_map<'a, FnBrand: CloneFn + 'a, A: 'a + Clone, B: 'a + Clone, M>(
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
		/// 	explicit::bi_traverse::<RcFnBrand, PairBrand, _, _, _, _, OptionBrand, _, _>(
		/// 		(|a: i32| Some(a + 1), |b: i32| Some(b * 2)),
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
		#[multi_brand]
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
		/// assert_eq!(
		/// 	explicit::map::<PairFirstAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, Pair(1, 5)),
		/// 	Pair(1, 10)
		/// );
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
		/// 	explicit::lift2::<PairFirstAppliedBrand<String>, _, _, _, _, _, _>(
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
		/// 	classes::semiapplicative::apply as explicit_apply,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Pair("a".to_string(), lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
		/// 		f,
		/// 		Pair("b".to_string(), 5)
		/// 	),
		/// 	Pair("ab".to_string(), 10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
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
		/// 	explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(
		/// 		Pair("a".to_string(), 5),
		/// 		|x| { Pair("b".to_string(), x * 2) }
		/// 	),
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

	/// [`MonadRec`] implementation for [`PairFirstAppliedBrand`].
	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> MonadRec for PairFirstAppliedBrand<First>
	where
		First: Monoid,
	{
		/// Performs tail-recursive monadic computation over the second value, accumulating the first.
		///
		/// Iteratively applies the step function. Each iteration produces a pair
		/// whose first value is combined with the running accumulator via
		/// [`Semigroup::append`]. If the step returns `ControlFlow::Continue(a)`, the loop
		/// continues with the new state. If it returns `ControlFlow::Break(b)`, the
		/// computation completes with the accumulated first value and `b`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("A pair with the accumulated first value and the final result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<PairFirstAppliedBrand<String>, _, _>(
		/// 	|n| {
		/// 		if n < 3 {
		/// 			Pair(format!("{n},"), ControlFlow::Continue(n + 1))
		/// 		} else {
		/// 			Pair(format!("{n}"), ControlFlow::Break(n))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, Pair("0,1,2,3".to_string(), 3));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut acc: First = Monoid::empty();
			let mut current = initial;
			loop {
				let Pair(first, step) = func(current);
				acc = Semigroup::append(acc, first);
				match step {
					ControlFlow::Continue(next) => current = next,
					ControlFlow::Break(b) => return Pair(acc, b),
				}
			}
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
		/// 	explicit::fold_right::<RcFnBrand, PairFirstAppliedBrand<()>, _, _, _, _>(
		/// 		|x, acc| x + acc,
		/// 		0,
		/// 		Pair((), 5)
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
		/// 	explicit::fold_left::<RcFnBrand, PairFirstAppliedBrand<()>, _, _, _, _>(
		/// 		|acc, x| acc + x,
		/// 		0,
		/// 		Pair((), 5)
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
		/// 	explicit::fold_map::<RcFnBrand, PairFirstAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Pair((), 5)
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
			FnBrand: CloneFn + 'a, {
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
		/// 	explicit::traverse::<RcFnBrand, PairFirstAppliedBrand<()>, _, _, OptionBrand, _, _>(
		/// 		|x| Some(x * 2),
		/// 		Pair((), 5)
		/// 	),
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

	// -- By-reference trait implementations for PairFirstAppliedBrand --

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefFunctor for PairFirstAppliedBrand<First> {
		/// Maps a function over the second value in the pair by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The pair.")]
		#[document_returns("A new pair with the mapped second value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// assert_eq!(
		/// 	explicit::map::<PairFirstAppliedBrand<_>, _, _, _, _>(|x: &i32| *x * 2, &Pair(1, 5)),
		/// 	Pair(1, 10)
		/// );
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Pair(fa.0.clone(), func(&fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefFoldable for PairFirstAppliedBrand<First> {
		/// Folds the pair by reference (over second).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters("The mapping function.", "The pair.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let result = explicit::fold_map::<RcFnBrand, PairFirstAppliedBrand<()>, _, _, _, _>(
		/// 	|x: &i32| x.to_string(),
		/// 	&Pair((), 5),
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			func(&fa.1)
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefTraversable for PairFirstAppliedBrand<First> {
		/// Traverses the pair by reference (over second).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The input type.",
			"The output type.",
			"The applicative."
		)]
		#[document_parameters("The function.", "The pair.")]
		#[document_returns("The traversed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let result: Option<Pair<(), String>> =
		/// 	ref_traverse::<PairFirstAppliedBrand<()>, RcFnBrand, _, _, OptionBrand>(
		/// 		|x: &i32| Some(x.to_string()),
		/// 		&Pair((), 42),
		/// 	);
		/// assert_eq!(result, Some(Pair((), "42".to_string())));
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let first = ta.0.clone();
			F::map(move |b| Pair(first.clone(), b), func(&ta.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefPointed for PairFirstAppliedBrand<First>
	where
		First: Monoid,
	{
		/// Creates a pair from a reference by cloning (with empty first).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The value type.")]
		#[document_parameters("The reference to wrap.")]
		#[document_returns("A pair containing `Monoid::empty()` and a clone of the value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = 42;
		/// let result: Pair<String, i32> = ref_pure::<PairFirstAppliedBrand<String>, _>(&x);
		/// assert_eq!(result, Pair("".to_string(), 42));
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Pair(Monoid::empty(), a.clone())
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefLift for PairFirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Combines two pairs with a by-reference binary function (over second).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "First input.", "Second input.", "Output.")]
		#[document_parameters("The binary function.", "The first pair.", "The second pair.")]
		#[document_returns("A pair with combined first values and the function result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = explicit::lift2::<PairFirstAppliedBrand<String>, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32| *a + *b,
		/// 	&Pair("a".to_string(), 1),
		/// 	&Pair("b".to_string(), 2),
		/// );
		/// assert_eq!(result, Pair("ab".to_string(), 3));
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Pair(Semigroup::append(fa.0.clone(), fb.0.clone()), func(&fa.1, &fb.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefSemiapplicative for PairFirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Applies a wrapped by-ref function to a pair value (over second).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters(
			"The pair containing the function.",
			"The pair containing the value."
		)]
		#[document_returns("A pair with combined first values and the function result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x * 2);
		/// let result = ref_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
		/// 	&Pair("a".to_string(), f),
		/// 	&Pair("b".to_string(), 5),
		/// );
		/// assert_eq!(result, Pair("ab".to_string(), 10));
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Pair(Semigroup::append(ff.0.clone(), fa.0.clone()), (*ff.1)(&fa.1))
		}
	}

	#[document_type_parameters("The type of the first value in the pair.")]
	impl<First: Clone + 'static> RefSemimonad for PairFirstAppliedBrand<First>
	where
		First: Semigroup,
	{
		/// Chains pair computations by reference (over second).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The input pair.", "The function to apply by reference.")]
		#[document_returns("A pair with combined first values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result: Pair<String, String> = explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(
		/// 	&Pair("a".to_string(), 42),
		/// 	|x: &i32| Pair("b".to_string(), x.to_string()),
		/// );
		/// assert_eq!(result, Pair("ab".to_string(), "42".to_string()));
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let Pair(next_first, next_second) = f(&fa.1);
			Pair(Semigroup::append(fa.0.clone(), next_first), next_second)
		}
	}

	// PairSecondAppliedBrand<Second> (Functor over First)

	impl_kind! {
		#[multi_brand]
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
		/// assert_eq!(
		/// 	explicit::map::<PairSecondAppliedBrand<_>, _, _, _, _>(|x: i32| x * 2, Pair(5, 1)),
		/// 	Pair(10, 1)
		/// );
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
		/// 	explicit::lift2::<PairSecondAppliedBrand<String>, _, _, _, _, _, _>(
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
		/// 	classes::semiapplicative::apply as explicit_apply,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f = Pair(lift_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2), "a".to_string());
		/// assert_eq!(
		/// 	explicit_apply::<RcFnBrand, PairSecondAppliedBrand<String>, _, _>(
		/// 		f,
		/// 		Pair(5, "b".to_string())
		/// 	),
		/// 	Pair(10, "ab".to_string())
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn>::Of<'a, A, B>>),
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
		/// 	explicit::bind::<PairSecondAppliedBrand<String>, _, _, _, _>(
		/// 		Pair(5, "a".to_string()),
		/// 		|x| Pair(x * 2, "b".to_string())
		/// 	),
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

	/// [`MonadRec`] implementation for [`PairSecondAppliedBrand`].
	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> MonadRec for PairSecondAppliedBrand<Second>
	where
		Second: Monoid,
	{
		/// Performs tail-recursive monadic computation over the first value, accumulating the second.
		///
		/// Iteratively applies the step function. Each iteration produces a pair
		/// whose second value is combined with the running accumulator via
		/// [`Semigroup::append`]. If the step returns `ControlFlow::Continue(a)`, the loop
		/// continues with the new state. If it returns `ControlFlow::Break(b)`, the
		/// computation completes with `b` and the accumulated second value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the computation.",
			"The type of the initial value and loop state.",
			"The type of the result."
		)]
		///
		#[document_parameters("The step function.", "The initial value.")]
		///
		#[document_returns("A pair with the final result and the accumulated second value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	core::ops::ControlFlow,
		/// 	fp_library::{
		/// 		brands::*,
		/// 		functions::*,
		/// 		types::*,
		/// 	},
		/// };
		///
		/// let result = tail_rec_m::<PairSecondAppliedBrand<String>, _, _>(
		/// 	|n| {
		/// 		if n < 3 {
		/// 			Pair(ControlFlow::Continue(n + 1), format!("{n},"))
		/// 		} else {
		/// 			Pair(ControlFlow::Break(n), format!("{n}"))
		/// 		}
		/// 	},
		/// 	0,
		/// );
		/// assert_eq!(result, Pair(3, "0,1,2,3".to_string()));
		/// ```
		fn tail_rec_m<'a, A: 'a, B: 'a>(
			func: impl Fn(
				A,
			)
				-> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ControlFlow<B, A>>)
			+ 'a,
			initial: A,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let mut acc: Second = Monoid::empty();
			let mut current = initial;
			loop {
				let Pair(step, second) = func(current);
				acc = Semigroup::append(acc, second);
				match step {
					ControlFlow::Continue(next) => current = next,
					ControlFlow::Break(b) => return Pair(b, acc),
				}
			}
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
		/// 	explicit::fold_right::<RcFnBrand, PairSecondAppliedBrand<()>, _, _, _, _>(
		/// 		|x, acc| x + acc,
		/// 		0,
		/// 		Pair(5, ())
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
		/// 	explicit::fold_left::<RcFnBrand, PairSecondAppliedBrand<()>, _, _, _, _>(
		/// 		|acc, x| acc + x,
		/// 		0,
		/// 		Pair(5, ())
		/// 	),
		/// 	5
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneFn + 'a, {
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
		/// 	explicit::fold_map::<RcFnBrand, PairSecondAppliedBrand<()>, _, _, _, _>(
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
			FnBrand: CloneFn + 'a, {
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
		/// 	explicit::traverse::<RcFnBrand, PairSecondAppliedBrand<()>, _, _, OptionBrand, _, _>(
		/// 		|x| Some(x * 2),
		/// 		Pair(5, ())
		/// 	),
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
	// -- By-reference trait implementations for PairSecondAppliedBrand --

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefFunctor for PairSecondAppliedBrand<Second> {
		/// Maps a function over the first value in the pair by reference.
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The function.", "The pair.")]
		#[document_returns("A new pair with the mapped first value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// assert_eq!(
		/// 	explicit::map::<PairSecondAppliedBrand<_>, _, _, _, _>(|x: &i32| *x * 2, &Pair(5, 1)),
		/// 	Pair(10, 1)
		/// );
		/// ```
		fn ref_map<'a, A: 'a, B: 'a>(
			func: impl Fn(&A) -> B + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Pair(func(&fa.0), fa.1.clone())
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefFoldable for PairSecondAppliedBrand<Second> {
		/// Folds the pair by reference (over first).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The element type.",
			"The monoid type."
		)]
		#[document_parameters("The mapping function.", "The pair.")]
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let result = explicit::fold_map::<RcFnBrand, PairSecondAppliedBrand<()>, _, _, _, _>(
		/// 	|x: &i32| x.to_string(),
		/// 	&Pair(5, ()),
		/// );
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			func(&fa.0)
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefTraversable for PairSecondAppliedBrand<Second> {
		/// Traverses the pair by reference (over first).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The brand.",
			"The input type.",
			"The output type.",
			"The applicative."
		)]
		#[document_parameters("The function.", "The pair.")]
		#[document_returns("The traversed result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let result: Option<Pair<String, ()>> =
		/// 	ref_traverse::<PairSecondAppliedBrand<()>, RcFnBrand, _, _, OptionBrand>(
		/// 		|x: &i32| Some(x.to_string()),
		/// 		&Pair(42, ()),
		/// 	);
		/// assert_eq!(result, Some(Pair("42".to_string(), ())));
		/// ```
		fn ref_traverse<'a, FnBrand, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(&A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			FnBrand: LiftFn + 'a,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			let second = ta.1.clone();
			F::map(move |a| Pair(a, second.clone()), func(&ta.0))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefPointed for PairSecondAppliedBrand<Second>
	where
		Second: Monoid,
	{
		/// Creates a pair from a reference by cloning (with empty second).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The value type.")]
		#[document_parameters("The reference to wrap.")]
		#[document_returns("A pair containing a clone of the value and `Monoid::empty()`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x = 42;
		/// let result: Pair<i32, String> = ref_pure::<PairSecondAppliedBrand<String>, _>(&x);
		/// assert_eq!(result, Pair(42, "".to_string()));
		/// ```
		fn ref_pure<'a, A: Clone + 'a>(
			a: &A
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Pair(a.clone(), Monoid::empty())
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefLift for PairSecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Combines two pairs with a by-reference binary function (over first).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "First input.", "Second input.", "Output.")]
		#[document_parameters("The binary function.", "The first pair.", "The second pair.")]
		#[document_returns("A pair with the function result and combined second values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result = explicit::lift2::<PairSecondAppliedBrand<String>, _, _, _, _, _, _>(
		/// 	|a: &i32, b: &i32| *a + *b,
		/// 	&Pair(1, "a".to_string()),
		/// 	&Pair(2, "b".to_string()),
		/// );
		/// assert_eq!(result, Pair(3, "ab".to_string()));
		/// ```
		fn ref_lift2<'a, A: 'a, B: 'a, C: 'a>(
			func: impl Fn(&A, &B) -> C + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) {
			Pair(func(&fa.0, &fb.0), Semigroup::append(fa.1.clone(), fb.1.clone()))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefSemiapplicative for PairSecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Applies a wrapped by-ref function to a pair value (over first).
		#[document_signature]
		#[document_type_parameters(
			"The lifetime.",
			"The function brand.",
			"The input type.",
			"The output type."
		)]
		#[document_parameters(
			"The pair containing the function.",
			"The pair containing the value."
		)]
		#[document_returns("A pair with the function result and combined second values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f: std::rc::Rc<dyn Fn(&i32) -> i32> = std::rc::Rc::new(|x: &i32| *x * 2);
		/// let result = ref_apply::<RcFnBrand, PairSecondAppliedBrand<String>, _, _>(
		/// 	&Pair(f, "a".to_string()),
		/// 	&Pair(5, "b".to_string()),
		/// );
		/// assert_eq!(result, Pair(10, "ab".to_string()));
		/// ```
		fn ref_apply<'a, FnBrand: 'a + CloneFn<Ref>, A: 'a, B: 'a>(
			ff: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneFn<Ref>>::Of<'a, A, B>>),
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Pair((*ff.0)(&fa.0), Semigroup::append(ff.1.clone(), fa.1.clone()))
		}
	}

	#[document_type_parameters("The type of the second value in the pair.")]
	impl<Second: Clone + 'static> RefSemimonad for PairSecondAppliedBrand<Second>
	where
		Second: Semigroup,
	{
		/// Chains pair computations by reference (over first).
		#[document_signature]
		#[document_type_parameters("The lifetime.", "The input type.", "The output type.")]
		#[document_parameters("The input pair.", "The function to apply by reference.")]
		#[document_returns("A pair with combined second values.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let result: Pair<String, String> = explicit::bind::<PairSecondAppliedBrand<String>, _, _, _, _>(
		/// 	&Pair(42, "a".to_string()),
		/// 	|x: &i32| Pair(x.to_string(), "b".to_string()),
		/// );
		/// assert_eq!(result, Pair("42".to_string(), "ab".to_string()));
		/// ```
		fn ref_bind<'a, A: 'a, B: 'a>(
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			f: impl Fn(&A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			let Pair(next_first, next_second) = f(&fa.0);
			Pair(next_first, Semigroup::append(fa.1.clone(), next_second))
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
				semiapplicative::apply as explicit_apply,
				*,
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
		assert_eq!(
			explicit::bimap::<PairBrand, _, _, _, _, _, _>((|a| a + 1, |b| b * 2), x),
			Pair(2, 10)
		);
	}

	// Bifunctor Laws

	/// Tests the identity law for Bifunctor.
	#[quickcheck]
	fn bifunctor_identity(
		first: String,
		second: i32,
	) -> bool {
		let x = Pair(first, second);
		explicit::bimap::<PairBrand, _, _, _, _, _, _>((identity, identity), x.clone()) == x
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

		explicit::bimap::<PairBrand, _, _, _, _, _, _>((compose(f, g), compose(h, i)), x)
			== explicit::bimap::<PairBrand, _, _, _, _, _, _>(
				(f, h),
				explicit::bimap::<PairBrand, _, _, _, _, _, _>((g, i), x),
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
		explicit::map::<PairFirstAppliedBrand<String>, _, _, _, _>(identity, x.clone()) == x
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
		explicit::map::<PairFirstAppliedBrand<String>, _, _, _, _>(compose(f, g), x.clone())
			== explicit::map::<PairFirstAppliedBrand<String>, _, _, _, _>(
				f,
				explicit::map::<PairFirstAppliedBrand<String>, _, _, _, _>(g, x),
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
		explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			pure::<PairFirstAppliedBrand<String>, _>(<RcFnBrand as LiftFn>::new(identity)),
			v.clone(),
		) == v
	}

	/// Tests the homomorphism law for Applicative.
	#[quickcheck]
	fn applicative_homomorphism(x: i32) -> bool {
		let f = |x: i32| x.wrapping_mul(2);
		explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			pure::<PairFirstAppliedBrand<String>, _>(<RcFnBrand as LiftFn>::new(f)),
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

		let u_fn = <RcFnBrand as LiftFn>::new(move |x: i32| x.wrapping_add(u_seed));
		let u = pure::<PairFirstAppliedBrand<String>, _>(u_fn);

		let v_fn = <RcFnBrand as LiftFn>::new(move |x: i32| x.wrapping_mul(v_seed));
		let v = pure::<PairFirstAppliedBrand<String>, _>(v_fn);

		// RHS: u <*> (v <*> w)
		let vw =
			explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(v.clone(), w.clone());
		let rhs = explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(u.clone(), vw);

		// LHS: pure(compose) <*> u <*> v <*> w
		let compose_fn = <RcFnBrand as LiftFn>::new(|f: std::rc::Rc<dyn Fn(i32) -> i32>| {
			let f = f.clone();
			<RcFnBrand as LiftFn>::new(move |g: std::rc::Rc<dyn Fn(i32) -> i32>| {
				let f = f.clone();
				let g = g.clone();
				<RcFnBrand as LiftFn>::new(move |x| f(g(x)))
			})
		});

		let pure_compose = pure::<PairFirstAppliedBrand<String>, _>(compose_fn);
		let u_applied =
			explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(pure_compose, u);
		let uv = explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(u_applied, v);
		let lhs = explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(uv, w);

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
		let u = pure::<PairFirstAppliedBrand<String>, _>(<RcFnBrand as LiftFn>::new(f));

		let lhs = explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
			u.clone(),
			pure::<PairFirstAppliedBrand<String>, _>(y),
		);

		let rhs_fn = <RcFnBrand as LiftFn>::new(move |f: std::rc::Rc<dyn Fn(i32) -> i32>| f(y));
		let rhs = explicit_apply::<RcFnBrand, PairFirstAppliedBrand<String>, _, _>(
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
		explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(
			pure::<PairFirstAppliedBrand<String>, _>(a),
			f,
		) == f(a)
	}

	/// Tests the right identity law for Monad.
	#[quickcheck]
	fn monad_right_identity(
		first: String,
		second: i32,
	) -> bool {
		let m = Pair(first, second);
		explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(
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
		explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(
			explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(m.clone(), f),
			g,
		) == explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(m, |x| {
			explicit::bind::<PairFirstAppliedBrand<String>, _, _, _, _>(f(x), g)
		})
	}

	// MonadRec tests for PairFirstAppliedBrand

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_first_applied_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<PairFirstAppliedBrand<String>, _, _>(
			|a| Pair(String::new(), ControlFlow::Break(a)),
			x,
		) == Pair(String::new(), x)
	}

	/// Tests a recursive computation that accumulates the first value.
	#[test]
	fn monad_rec_first_applied_accumulation() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result = tail_rec_m::<PairFirstAppliedBrand<String>, _, _>(
			|n: i32| {
				if n < 3 {
					Pair(format!("{n},"), ControlFlow::Continue(n + 1))
				} else {
					Pair(format!("{n}"), ControlFlow::Break(n))
				}
			},
			0,
		);
		assert_eq!(result, Pair("0,1,2,3".to_string(), 3));
	}

	/// Tests stack safety of MonadRec for PairFirstAppliedBrand.
	#[test]
	fn monad_rec_first_applied_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<PairFirstAppliedBrand<Vec<()>>, _, _>(
			|acc| {
				if acc < iterations {
					Pair(vec![], ControlFlow::Continue(acc + 1))
				} else {
					Pair(vec![], ControlFlow::Break(acc))
				}
			},
			0i64,
		);
		assert_eq!(result, Pair(vec![], iterations));
	}

	// MonadRec tests for PairSecondAppliedBrand

	/// Tests the MonadRec identity law: `tail_rec_m(|a| pure(Done(a)), x) == pure(x)`.
	#[quickcheck]
	fn monad_rec_second_applied_identity(x: i32) -> bool {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		tail_rec_m::<PairSecondAppliedBrand<String>, _, _>(
			|a| Pair(ControlFlow::Break(a), String::new()),
			x,
		) == Pair(x, String::new())
	}

	/// Tests a recursive computation that accumulates the second value.
	#[test]
	fn monad_rec_second_applied_accumulation() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let result = tail_rec_m::<PairSecondAppliedBrand<String>, _, _>(
			|n: i32| {
				if n < 3 {
					Pair(ControlFlow::Continue(n + 1), format!("{n},"))
				} else {
					Pair(ControlFlow::Break(n), format!("{n}"))
				}
			},
			0,
		);
		assert_eq!(result, Pair(3, "0,1,2,3".to_string()));
	}

	/// Tests stack safety of MonadRec for PairSecondAppliedBrand.
	#[test]
	fn monad_rec_second_applied_stack_safety() {
		use {
			crate::classes::monad_rec::tail_rec_m,
			core::ops::ControlFlow,
		};
		let iterations: i64 = 200_000;
		let result = tail_rec_m::<PairSecondAppliedBrand<Vec<()>>, _, _>(
			|acc| {
				if acc < iterations {
					Pair(ControlFlow::Continue(acc + 1), vec![])
				} else {
					Pair(ControlFlow::Break(acc), vec![])
				}
			},
			0i64,
		);
		assert_eq!(result, Pair(iterations, vec![]));
	}

	// RefBifunctor Laws

	/// RefBifunctor identity
	#[quickcheck]
	fn ref_bifunctor_identity(
		a: i32,
		b: i32,
	) -> bool {
		let p = Pair(a, b);
		explicit::bimap::<PairBrand, _, _, _, _, _, _>((|x: &i32| *x, |x: &i32| *x), &p) == p
	}

	/// RefBifunctor composition
	#[quickcheck]
	fn ref_bifunctor_composition(
		a: i32,
		b: i32,
	) -> bool {
		let p = Pair(a, b);
		let f1 = |x: &i32| x.wrapping_add(1);
		let f2 = |x: &i32| x.wrapping_mul(2);
		let g1 = |x: &i32| x.wrapping_add(10);
		let g2 = |x: &i32| x.wrapping_mul(3);
		explicit::bimap::<PairBrand, _, _, _, _, _, _>(
			(|x: &i32| f2(&f1(x)), |x: &i32| g2(&g1(x))),
			&p,
		) == explicit::bimap::<PairBrand, _, _, _, _, _, _>(
			(f2, g2),
			&explicit::bimap::<PairBrand, _, _, _, _, _, _>((f1, g1), &p),
		)
	}

	// RefBifoldable Laws

	/// RefBifoldable fold_map correctness
	#[quickcheck]
	fn ref_bifoldable_fold_map(
		a: i32,
		b: i32,
	) -> bool {
		let p = Pair(a, b);
		let result = explicit::bi_fold_map::<RcFnBrand, PairBrand, _, _, _, _, _>(
			(|x: &i32| x.to_string(), |x: &i32| x.to_string()),
			&p,
		);
		result == format!("{}{}", a, b)
	}

	// RefBitraversable Laws

	/// RefBitraversable consistency
	#[quickcheck]
	fn ref_bitraversable_consistency(
		a: i32,
		b: i32,
	) -> bool {
		let p = Pair(a, b);
		let f = |x: &i32| Some(x.wrapping_add(1));
		let g = |x: &i32| Some(x.wrapping_mul(2));
		let traversed = explicit::bi_traverse::<RcFnBrand, PairBrand, _, _, _, _, OptionBrand, _, _>(
			(f, g),
			&p,
		);
		let mapped_then_sequenced =
			ref_bi_sequence::<PairBrand, RcFnBrand, _, _, OptionBrand>(&explicit::bimap::<
				PairBrand,
				_,
				_,
				_,
				_,
				_,
				_,
			>((f, g), &p));
		traversed == mapped_then_sequenced
	}
}
