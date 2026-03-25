//! Control type representing Loop/Done states for tail-recursive computations.
//!
//! Used by [`MonadRec`](crate::classes::monad_rec::MonadRec) to implement stack-safe tail recursion. [`Step::Loop`] continues iteration, while [`Step::Done`] terminates with a result.
//!
//! ### Examples
//!
//! ```
//! use fp_library::types::*;
//!
//! // Count down from n to 0, accumulating the sum
//! fn sum_to_zero(
//! 	n: i32,
//! 	acc: i32,
//! ) -> Step<(i32, i32), i32> {
//! 	if n <= 0 { Step::Done(acc) } else { Step::Loop((n - 1, acc + n)) }
//! }
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				StepBrand,
				StepDoneAppliedBrand,
				StepLoopAppliedBrand,
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
				Pointed,
				Semiapplicative,
				Semimonad,
				Traversable,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	/// Represents the result of a single step in a tail-recursive computation.
	///
	/// This type is fundamental to stack-safe recursion via `MonadRec`.
	///
	/// ### Higher-Kinded Type Representation
	///
	/// This type has multiple higher-kinded representations:
	/// - [`StepBrand`](crate::brands::StepBrand): fully polymorphic over both loop and done types (bifunctor).
	/// - [`StepLoopAppliedBrand<LoopType>`](crate::brands::StepLoopAppliedBrand): the loop type is fixed, polymorphic over the done type (functor over done).
	/// - [`StepDoneAppliedBrand<DoneType>`](crate::brands::StepDoneAppliedBrand): the done type is fixed, polymorphic over the loop type (functor over loop).
	///
	/// ### Serialization
	///
	/// This type supports serialization and deserialization via [`serde`](https://serde.rs) when the `serde` feature is enabled.
	#[document_type_parameters(
		r#"The "loop" type - when we return `Loop(a)`, we continue with `a`."#,
		r#"The "done" type - when we return `Done(b)`, we're finished."#
	)]
	///
	/// ### Variants
	///
	/// * `Loop(A)`: Continue the loop with a new value.
	/// * `Done(B)`: Finish the computation with a final value.
	#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
	pub enum Step<A, B> {
		/// Continue the loop with a new value.
		Loop(A),
		/// Finish the computation with a final value.
		Done(B),
	}

	#[document_type_parameters(
		r#"The "loop" type - when we return `Loop(a)`, we continue with `a`."#,
		r#"The "done" type - when we return `Done(b)`, we're finished."#
	)]
	#[document_parameters("The step value.")]
	impl<A, B> Step<A, B> {
		/// Returns `true` if this is a `Loop` variant.
		#[document_signature]
		///
		#[document_returns("`true` if the step is a loop, `false` otherwise.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let step: Step<i32, i32> = Step::Loop(1);
		/// assert!(step.is_loop());
		/// ```
		pub fn is_loop(&self) -> bool {
			matches!(self, Step::Loop(_))
		}

		/// Returns `true` if this is a `Done` variant.
		#[document_signature]
		///
		#[document_returns("`true` if the step is done, `false` otherwise.")]
		///
		#[inline]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let step: Step<i32, i32> = Step::Done(1);
		/// assert!(step.is_done());
		/// ```
		pub fn is_done(&self) -> bool {
			matches!(self, Step::Done(_))
		}

		/// Maps a function over the `Loop` variant.
		#[document_signature]
		///
		#[document_type_parameters("The new loop type.")]
		///
		#[document_parameters("The function to apply to the loop value.")]
		///
		#[document_returns("A new `Step` with the loop value transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let step: Step<i32, i32> = Step::Loop(1);
		/// let mapped = step.map_loop(|x| x + 1);
		/// assert_eq!(mapped, Step::Loop(2));
		/// ```
		pub fn map_loop<C>(
			self,
			f: impl FnOnce(A) -> C,
		) -> Step<C, B> {
			match self {
				Step::Loop(a) => Step::Loop(f(a)),
				Step::Done(b) => Step::Done(b),
			}
		}

		/// Maps a function over the `Done` variant.
		#[document_signature]
		///
		#[document_type_parameters("The new done type.")]
		///
		#[document_parameters("The function to apply to the done value.")]
		///
		#[document_returns("A new `Step` with the done value transformed.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let step: Step<i32, i32> = Step::Done(1);
		/// let mapped = step.map_done(|x| x + 1);
		/// assert_eq!(mapped, Step::Done(2));
		/// ```
		pub fn map_done<C>(
			self,
			f: impl FnOnce(B) -> C,
		) -> Step<A, C> {
			match self {
				Step::Loop(a) => Step::Loop(a),
				Step::Done(b) => Step::Done(f(b)),
			}
		}

		/// Applies functions to both variants (bifunctor map).
		#[document_signature]
		///
		#[document_type_parameters("The new loop type.", "The new done type.")]
		///
		#[document_parameters(
			"The function to apply to the loop value.",
			"The function to apply to the done value."
		)]
		///
		#[document_returns("A new `Step` with both values transformed.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let step: Step<i32, i32> = Step::Loop(1);
		/// let mapped = step.bimap(|x| x + 1, |x| x * 2);
		/// assert_eq!(mapped, Step::Loop(2));
		/// ```
		pub fn bimap<C, D>(
			self,
			f: impl FnOnce(A) -> C,
			g: impl FnOnce(B) -> D,
		) -> Step<C, D> {
			match self {
				Step::Loop(a) => Step::Loop(f(a)),
				Step::Done(b) => Step::Done(g(b)),
			}
		}

		/// Folds the step from right to left using two step functions.
		///
		/// See [`Bifoldable::bi_fold_right`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The accumulator type.")]
		///
		#[document_parameters(
			"The step function for the Loop variant.",
			"The step function for the Done variant.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<i32, i32> = Step::Loop(3);
		/// assert_eq!(x.bi_fold_right(|a, acc| acc - a, |b, acc| acc + b, 10), 7);
		/// ```
		pub fn bi_fold_right<C>(
			self,
			f: impl FnOnce(A, C) -> C,
			g: impl FnOnce(B, C) -> C,
			z: C,
		) -> C {
			match self {
				Step::Loop(a) => f(a, z),
				Step::Done(b) => g(b, z),
			}
		}

		/// Folds the step from left to right using two step functions.
		///
		/// See [`Bifoldable::bi_fold_left`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The accumulator type.")]
		///
		#[document_parameters(
			"The step function for the Loop variant.",
			"The step function for the Done variant.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<i32, i32> = Step::Done(5);
		/// assert_eq!(x.bi_fold_left(|acc, a| acc - a, |acc, b| acc + b, 10), 15);
		/// ```
		pub fn bi_fold_left<C>(
			self,
			f: impl FnOnce(C, A) -> C,
			g: impl FnOnce(C, B) -> C,
			z: C,
		) -> C {
			match self {
				Step::Loop(a) => f(z, a),
				Step::Done(b) => g(z, b),
			}
		}

		/// Maps the value to a monoid depending on the variant.
		///
		/// See [`Bifoldable::bi_fold_map`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters("The monoid type.")]
		///
		#[document_parameters(
			"The function mapping the Loop value to the monoid.",
			"The function mapping the Done value to the monoid."
		)]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<i32, i32> = Step::Loop(3);
		/// assert_eq!(x.bi_fold_map(|a: i32| a.to_string(), |b: i32| b.to_string()), "3");
		/// ```
		pub fn bi_fold_map<M>(
			self,
			f: impl FnOnce(A) -> M,
			g: impl FnOnce(B) -> M,
		) -> M {
			match self {
				Step::Loop(a) => f(a),
				Step::Done(b) => g(b),
			}
		}

		/// Folds the Done value, returning `initial` for Loop.
		///
		/// See [`Foldable::fold_right`] for the type class version
		/// (via [`StepLoopAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The accumulator type.")]
		///
		#[document_parameters(
			"The function to apply to the Done value and the accumulator.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<(), i32> = Step::Done(5);
		/// assert_eq!(x.fold_right(|b, acc| b + acc, 10), 15);
		/// ```
		pub fn fold_right<C>(
			self,
			f: impl FnOnce(B, C) -> C,
			initial: C,
		) -> C {
			match self {
				Step::Loop(_) => initial,
				Step::Done(b) => f(b, initial),
			}
		}

		/// Folds the Done value from the left, returning `initial` for Loop.
		///
		/// See [`Foldable::fold_left`] for the type class version
		/// (via [`StepLoopAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The accumulator type.")]
		///
		#[document_parameters(
			"The function to apply to the accumulator and the Done value.",
			"The initial accumulator."
		)]
		///
		#[document_returns("The result of folding.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<(), i32> = Step::Done(5);
		/// assert_eq!(x.fold_left(|acc, b| acc + b, 10), 15);
		/// ```
		pub fn fold_left<C>(
			self,
			f: impl FnOnce(C, B) -> C,
			initial: C,
		) -> C {
			match self {
				Step::Loop(_) => initial,
				Step::Done(b) => f(initial, b),
			}
		}

		/// Maps the Done value to a monoid, returning `M::empty()` for Loop.
		///
		/// See [`Foldable::fold_map`] for the type class version
		/// (via [`StepLoopAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The monoid type.")]
		///
		#[document_parameters("The mapping function.")]
		///
		#[document_returns("The monoid value.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<(), i32> = Step::Done(5);
		/// assert_eq!(x.fold_map(|b: i32| b.to_string()), "5".to_string());
		/// ```
		pub fn fold_map<M: Monoid>(
			self,
			f: impl FnOnce(B) -> M,
		) -> M {
			match self {
				Step::Loop(_) => M::empty(),
				Step::Done(b) => f(b),
			}
		}

		/// Chains the Done value into a new computation, passing through Loop.
		///
		/// See [`Semimonad::bind`] for the type class version
		/// (via [`StepLoopAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The type of the resulting Done value.")]
		///
		#[document_parameters("The function to apply to the Done value.")]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<i32, i32> = Step::Done(5);
		/// let y = x.bind(|b| Step::Done(b * 2));
		/// assert_eq!(y, Step::Done(10));
		/// ```
		pub fn bind<C>(
			self,
			f: impl FnOnce(B) -> Step<A, C>,
		) -> Step<A, C> {
			match self {
				Step::Loop(a) => Step::Loop(a),
				Step::Done(b) => f(b),
			}
		}

		/// Chains the Loop value into a new computation, passing through Done.
		///
		/// See [`Semimonad::bind`] for the type class version
		/// (via [`StepDoneAppliedBrand`]).
		#[document_signature]
		///
		#[document_type_parameters("The type of the resulting Loop value.")]
		///
		#[document_parameters("The function to apply to the Loop value.")]
		///
		#[document_returns("The result of the computation.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::*;
		///
		/// let x: Step<i32, i32> = Step::Loop(5);
		/// let y = x.bind_loop(|a| Step::Loop(a * 2));
		/// assert_eq!(y, Step::Loop(10));
		/// ```
		pub fn bind_loop<C>(
			self,
			f: impl FnOnce(A) -> Step<C, B>,
		) -> Step<C, B> {
			match self {
				Step::Loop(a) => f(a),
				Step::Done(b) => Step::Done(b),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of the values.",
		"The type of the Loop value.",
		"The type of the Done value."
	)]
	#[document_parameters("The step instance.")]
	impl<'a, A: 'a, B: 'a> Step<A, B> {
		/// Traverses the step with two effectful functions.
		///
		/// See [`Bitraversable::bi_traverse`] for the type class version.
		#[document_signature]
		///
		#[document_type_parameters(
			"The output type for the Loop value.",
			"The output type for the Done value.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function for the Loop value.",
			"The function for the Done value."
		)]
		///
		#[document_returns("The transformed step wrapped in the applicative context.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::*,
		/// };
		///
		/// let x: Step<i32, i32> = Step::Loop(3);
		/// let y = x.bi_traverse::<_, _, OptionBrand>(|a| Some(a + 1), |b| Some(b * 2));
		/// assert_eq!(y, Some(Step::Loop(4)));
		/// ```
		pub fn bi_traverse<C: 'a + Clone, D: 'a + Clone, F: Applicative>(
			self,
			f: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>) + 'a,
			g: impl Fn(B) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>) + 'a,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Step<C, D>>)
		where
			Step<C, D>: Clone, {
			match self {
				Step::Loop(a) => F::map(|c| Step::Loop(c), f(a)),
				Step::Done(b) => F::map(|d| Step::Done(d), g(b)),
			}
		}
	}

	impl_kind! {
		for StepBrand {
			type Of<A, B> = Step<A, B>;
		}
	}

	impl_kind! {
		for StepBrand {
			type Of<'a, A: 'a, B: 'a>: 'a = Step<A, B>;
		}
	}

	impl Bifunctor for StepBrand {
		/// Maps functions over the values in the step.
		///
		/// This method applies one function to the loop value and another to the done value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the loop value.",
			"The type of the mapped loop value.",
			"The type of the done value.",
			"The type of the mapped done value."
		)]
		///
		#[document_parameters(
			"The function to apply to the loop value.",
			"The function to apply to the done value.",
			"The step to map over."
		)]
		///
		#[document_returns("A new step containing the mapped values.")]
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
		/// let x = Step::Loop(1);
		/// assert_eq!(bimap::<StepBrand, _, _, _, _>(|a| a + 1, |b: i32| b * 2, x), Step::Loop(2));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a>(
			f: impl Fn(A) -> B + 'a,
			g: impl Fn(C) -> D + 'a,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) {
			p.bimap(f, g)
		}
	}

	impl Bifoldable for StepBrand {
		/// Folds the step from right to left using two step functions.
		///
		/// Applies `f` to the Loop value or `g` to the Done value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Loop value.",
			"The type of the Done value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for the Loop variant.",
			"The step function for the Done variant.",
			"The initial accumulator.",
			"The step to fold."
		)]
		///
		#[document_returns("The folded result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: Step<i32, i32> = Step::Loop(3);
		/// assert_eq!(
		/// 	bi_fold_right::<RcFnBrand, StepBrand, _, _, _>(|a, acc| acc - a, |b, acc| acc + b, 10, x,),
		/// 	7
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

		/// Folds the step from left to right using two step functions.
		///
		/// Applies `f` to the Loop value or `g` to the Done value.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Loop value.",
			"The type of the Done value.",
			"The accumulator type."
		)]
		///
		#[document_parameters(
			"The step function for the Loop variant.",
			"The step function for the Done variant.",
			"The initial accumulator.",
			"The step to fold."
		)]
		///
		#[document_returns("The folded result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: Step<i32, i32> = Step::Done(5);
		/// assert_eq!(
		/// 	bi_fold_left::<RcFnBrand, StepBrand, _, _, _>(|acc, a| acc - a, |acc, b| acc + b, 10, x,),
		/// 	15
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

		/// Maps the value to a monoid depending on the variant.
		///
		/// Applies `f` if Loop, `g` if Done.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the Loop value.",
			"The type of the Done value.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function mapping the Loop value to the monoid.",
			"The function mapping the Done value to the monoid.",
			"The step to fold."
		)]
		///
		#[document_returns("The monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: Step<i32, i32> = Step::Loop(3);
		/// assert_eq!(
		/// 	bi_fold_map::<RcFnBrand, StepBrand, _, _, _>(
		/// 		|a: i32| a.to_string(),
		/// 		|b: i32| b.to_string(),
		/// 		x,
		/// 	),
		/// 	"3".to_string()
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

	impl Bitraversable for StepBrand {
		/// Traverses the step with two effectful functions.
		///
		/// Applies `f` to the Loop value or `g` to the Done value,
		/// wrapping the result in the applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the Loop value.",
			"The type of the Done value.",
			"The output type for Loop.",
			"The output type for Done.",
			"The applicative context."
		)]
		///
		#[document_parameters(
			"The function applied to the Loop value.",
			"The function applied to the Done value.",
			"The step to traverse."
		)]
		///
		#[document_returns("The transformed step wrapped in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let x: Step<i32, i32> = Step::Loop(3);
		/// assert_eq!(
		/// 	bi_traverse::<StepBrand, _, _, _, _, OptionBrand>(
		/// 		|a: i32| Some(a + 1),
		/// 		|b: i32| Some(b * 2),
		/// 		x,
		/// 	),
		/// 	Some(Step::Loop(4))
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

	// StepLoopAppliedBrand<LoopType> (Functor over B - Done)

	impl_kind! {
		impl<LoopType: 'static> for StepLoopAppliedBrand<LoopType> {
			type Of<'a, B: 'a>: 'a = Step<LoopType, B>;
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: 'static> Functor for StepLoopAppliedBrand<LoopType> {
		/// Maps a function over the done value in the step.
		///
		/// This method applies a function to the done value inside the step, producing a new step with the transformed done value. The loop value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the done value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply to the done value.", "The step to map over.")]
		///
		#[document_returns(
			"A new step containing the result of applying the function to the done value."
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
		/// 	map::<StepLoopAppliedBrand<i32>, _, _>(|x: i32| x * 2, Step::<i32, i32>::Done(5)),
		/// 	Step::Done(10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map_done(func)
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: Clone + 'static> Lift for StepLoopAppliedBrand<LoopType> {
		/// Lifts a binary function into the step context.
		///
		/// This method lifts a binary function to operate on values within the step context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first value.",
			"The type of the second value.",
			"The type of the result."
		)]
		///
		#[document_parameters(
			"The binary function to apply.",
			"The first step.",
			"The second step."
		)]
		///
		#[document_returns(
			"`Done(f(a, b))` if both steps are `Done`, otherwise the first loop encountered."
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
		/// 	lift2::<StepLoopAppliedBrand<()>, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Done(1),
		/// 		Step::Done(2)
		/// 	),
		/// 	Step::Done(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<StepLoopAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Done(1),
		/// 		Step::Loop(2)
		/// 	),
		/// 	Step::Loop(2)
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
			match (fa, fb) {
				(Step::Done(a), Step::Done(b)) => Step::Done(func(a, b)),
				(Step::Loop(e), _) => Step::Loop(e),
				(_, Step::Loop(e)) => Step::Loop(e),
			}
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: 'static> Pointed for StepLoopAppliedBrand<LoopType> {
		/// Wraps a value in a step.
		///
		/// This method wraps a value in the `Done` variant of a `Step`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Done(a)`.")]
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
		/// assert_eq!(pure::<StepLoopAppliedBrand<()>, _>(5), Step::Done(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Step::Done(a)
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: Clone + 'static> ApplyFirst for StepLoopAppliedBrand<LoopType> {}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: Clone + 'static> ApplySecond for StepLoopAppliedBrand<LoopType> {}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: Clone + 'static> Semiapplicative for StepLoopAppliedBrand<LoopType> {
		/// Applies a wrapped function to a wrapped value.
		///
		/// This method applies a function wrapped in a step to a value wrapped in a step.
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
			"The step containing the function.",
			"The step containing the value."
		)]
		///
		#[document_returns(
			"`Done(f(a))` if both are `Done`, otherwise the first loop encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f: Step<_, _> = Step::Done(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	apply::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(f, Step::Done(5)),
		/// 	Step::Done(10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(Step::Done(f), Step::Done(a)) => Step::Done(f(a)),
				(Step::Loop(e), _) => Step::Loop(e),
				(_, Step::Loop(e)) => Step::Loop(e),
			}
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: Clone + 'static> Semimonad for StepLoopAppliedBrand<LoopType> {
		/// Chains step computations.
		///
		/// This method chains two computations, where the second computation depends on the result of the first.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters(
			"The first step.",
			"The function to apply to the value inside the step."
		)]
		///
		#[document_returns(
			"The result of applying `f` to the value if `ma` is `Done`, otherwise the original loop."
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
		/// 	bind::<StepLoopAppliedBrand<()>, _, _>(Step::Done(5), |x| Step::Done(x * 2)),
		/// 	Step::Done(10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind(func)
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: 'static> Foldable for StepLoopAppliedBrand<LoopType> {
		/// Folds the step from the right.
		///
		/// This method performs a right-associative fold of the step.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The step to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Done(a)`, otherwise `initial`.")]
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
		/// 	fold_right::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(|x, acc| x + acc, 0, Step::Done(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		Step::Loop(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			fa.fold_right(func, initial)
		}

		/// Folds the step from the left.
		///
		/// This method performs a left-associative fold of the step.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The step to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Done(a)`, otherwise `initial`.")]
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
		/// 	fold_left::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(|acc, x| acc + x, 0, Step::Done(5)),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		Step::Loop(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			fa.fold_left(func, initial)
		}

		/// Maps the value to a monoid and returns it.
		///
		/// This method maps the element of the step to a monoid and then returns it.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The step to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Done(a)`, otherwise `M::empty()`.")]
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
		/// 	fold_map::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Done(5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Loop(1)
		/// 	),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			fa.fold_map(func)
		}
	}

	#[document_type_parameters("The loop type.")]
	impl<LoopType: Clone + 'static> Traversable for StepLoopAppliedBrand<LoopType> {
		/// Traverses the step with an applicative function.
		///
		/// This method maps the element of the step to a computation, evaluates it, and combines the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The step to traverse.")]
		///
		#[document_returns("The step wrapped in the applicative context.")]
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
		/// 	traverse::<StepLoopAppliedBrand<()>, _, _, OptionBrand>(|x| Some(x * 2), Step::Done(5)),
		/// 	Some(Step::Done(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<StepLoopAppliedBrand<i32>, _, _, OptionBrand>(
		/// 		|x: i32| Some(x * 2),
		/// 		Step::Loop(1)
		/// 	),
		/// 	Some(Step::Loop(1))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				Step::Done(a) => F::map(|b| Step::Done(b), func(a)),
				Step::Loop(e) => F::pure(Step::Loop(e)),
			}
		}

		/// Sequences a step of applicative.
		///
		/// This method evaluates the computation inside the step and accumulates the result into an applicative context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The step containing the applicative value.")]
		///
		#[document_returns("The step wrapped in the applicative context.")]
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
		/// 	sequence::<StepLoopAppliedBrand<()>, _, OptionBrand>(Step::Done(Some(5))),
		/// 	Some(Step::Done(5))
		/// );
		/// assert_eq!(
		/// 	sequence::<StepLoopAppliedBrand<i32>, i32, OptionBrand>(Step::Loop::<i32, Option<i32>>(1)),
		/// 	Some(Step::Loop::<i32, i32>(1))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				Step::Done(fa) => F::map(|a| Step::Done(a), fa),
				Step::Loop(e) => F::pure(Step::Loop(e)),
			}
		}
	}

	// StepDoneAppliedBrand<DoneType> (Functor over A - Loop)

	impl_kind! {
		impl<DoneType: 'static> for StepDoneAppliedBrand<DoneType> {
			type Of<'a, A: 'a>: 'a = Step<A, DoneType>;
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: 'static> Functor for StepDoneAppliedBrand<DoneType> {
		/// Maps a function over the loop value in the step.
		///
		/// This method applies a function to the loop value inside the step, producing a new step with the transformed loop value. The done value remains unchanged.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the loop value.",
			"The type of the result of applying the function."
		)]
		///
		#[document_parameters("The function to apply to the loop value.", "The step to map over.")]
		///
		#[document_returns(
			"A new step containing the result of applying the function to the loop value."
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
		/// 	map::<StepDoneAppliedBrand<i32>, _, _>(|x: i32| x * 2, Step::<i32, i32>::Loop(5)),
		/// 	Step::Loop(10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			fa.map_loop(func)
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: Clone + 'static> Lift for StepDoneAppliedBrand<DoneType> {
		/// Lifts a binary function into the step context (over loop).
		///
		/// This method lifts a binary function to operate on loop values within the step context.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the first loop value.",
			"The type of the second loop value.",
			"The type of the result loop value."
		)]
		///
		#[document_parameters(
			"The binary function to apply to the loops.",
			"The first step.",
			"The second step."
		)]
		///
		#[document_returns(
			"`Loop(f(a, b))` if both steps are `Loop`, otherwise the first done encountered."
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
		/// 	lift2::<StepDoneAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Loop(1),
		/// 		Step::Loop(2)
		/// 	),
		/// 	Step::Loop(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<StepDoneAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Loop(1),
		/// 		Step::Done(2)
		/// 	),
		/// 	Step::Done(2)
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
			match (fa, fb) {
				(Step::Loop(a), Step::Loop(b)) => Step::Loop(func(a, b)),
				(Step::Done(t), _) => Step::Done(t),
				(_, Step::Done(t)) => Step::Done(t),
			}
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: 'static> Pointed for StepDoneAppliedBrand<DoneType> {
		/// Wraps a value in a step (as loop).
		///
		/// This method wraps a value in the `Loop` variant of a `Step`.
		#[document_signature]
		///
		#[document_type_parameters("The lifetime of the value.", "The type of the value to wrap.")]
		///
		#[document_parameters("The value to wrap.")]
		///
		#[document_returns("`Loop(a)`.")]
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
		/// assert_eq!(pure::<StepDoneAppliedBrand<()>, _>(5), Step::Loop(5));
		/// ```
		fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Step::Loop(a)
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: Clone + 'static> ApplyFirst for StepDoneAppliedBrand<DoneType> {}

	#[document_type_parameters("The done type.")]
	impl<DoneType: Clone + 'static> ApplySecond for StepDoneAppliedBrand<DoneType> {}

	#[document_type_parameters("The done type.")]
	impl<DoneType: Clone + 'static> Semiapplicative for StepDoneAppliedBrand<DoneType> {
		/// Applies a wrapped function to a wrapped value (over loop).
		///
		/// This method applies a function wrapped in a step (as loop) to a value wrapped in a step (as loop).
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
			"The step containing the function (in Loop).",
			"The step containing the value (in Loop)."
		)]
		///
		#[document_returns(
			"`Loop(f(a))` if both are `Loop`, otherwise the first done encountered."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		///
		/// let f: Step<_, ()> = Step::Loop(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		/// assert_eq!(
		/// 	apply::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(f, Step::Loop(5)),
		/// 	Step::Loop(10)
		/// );
		/// ```
		fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
			ff: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match (ff, fa) {
				(Step::Loop(f), Step::Loop(a)) => Step::Loop(f(a)),
				(Step::Done(t), _) => Step::Done(t),
				(_, Step::Done(t)) => Step::Done(t),
			}
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: Clone + 'static> Semimonad for StepDoneAppliedBrand<DoneType> {
		/// Chains step computations (over loop).
		///
		/// This method chains two computations, where the second computation depends on the result of the first (over loop).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the result of the first computation.",
			"The type of the result of the second computation."
		)]
		///
		#[document_parameters("The first step.", "The function to apply to the loop value.")]
		///
		#[document_returns(
			"The result of applying `f` to the loop if `ma` is `Loop`, otherwise the original done."
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
		/// 	bind::<StepDoneAppliedBrand<()>, _, _>(Step::Loop(5), |x| Step::Loop(x * 2)),
		/// 	Step::Loop(10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: impl Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			ma.bind_loop(func)
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: 'static> Foldable for StepDoneAppliedBrand<DoneType> {
		/// Folds the step from the right (over loop).
		///
		/// This method performs a right-associative fold of the step (over loop).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The step to fold.")]
		///
		#[document_returns("`func(a, initial)` if `fa` is `Loop(a)`, otherwise `initial`.")]
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
		/// 	fold_right::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		Step::Loop(1)
		/// 	),
		/// 	1
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		Step::Done(())
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			match fa {
				Step::Loop(e) => func(e, initial),
				Step::Done(_) => initial,
			}
		}

		/// Folds the step from the left (over loop).
		///
		/// This method performs a left-associative fold of the step (over loop).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters("The folding function.", "The initial value.", "The step to fold.")]
		///
		#[document_returns("`func(initial, a)` if `fa` is `Loop(a)`, otherwise `initial`.")]
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
		/// 	fold_left::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		Step::Loop(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		Step::Done(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: CloneableFn + 'a, {
			match fa {
				Step::Loop(e) => func(initial, e),
				Step::Done(_) => initial,
			}
		}

		/// Maps the value to a monoid and returns it (over loop).
		///
		/// This method maps the element of the step to a monoid and then returns it (over loop).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters("The mapping function.", "The step to fold.")]
		///
		#[document_returns("`func(a)` if `fa` is `Loop(a)`, otherwise `M::empty()`.")]
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
		/// 	fold_map::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Loop(5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Done(1)
		/// 	),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa {
				Step::Loop(e) => func(e),
				Step::Done(_) => M::empty(),
			}
		}
	}

	#[document_type_parameters("The done type.")]
	impl<DoneType: Clone + 'static> Traversable for StepDoneAppliedBrand<DoneType> {
		/// Traverses the step with an applicative function (over loop).
		///
		/// This method maps the element of the step to a computation, evaluates it, and combines the result into an applicative context (over loop).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The type of the elements in the resulting traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The function to apply.", "The step to traverse.")]
		///
		#[document_returns("The step wrapped in the applicative context.")]
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
		/// 	traverse::<StepDoneAppliedBrand<()>, _, _, OptionBrand>(|x| Some(x * 2), Step::Loop(5)),
		/// 	Some(Step::Loop(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<StepDoneAppliedBrand<i32>, _, _, OptionBrand>(
		/// 		|x: i32| Some(x * 2),
		/// 		Step::Done(1)
		/// 	),
		/// 	Some(Step::Done(1))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative>(
			func: impl Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone, {
			match ta {
				Step::Loop(e) => F::map(|b| Step::Loop(b), func(e)),
				Step::Done(t) => F::pure(Step::Done(t)),
			}
		}

		/// Sequences a step of applicative (over loop).
		///
		/// This method evaluates the computation inside the step and accumulates the result into an applicative context (over loop).
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the elements in the traversable structure.",
			"The applicative context."
		)]
		///
		#[document_parameters("The step containing the applicative value.")]
		///
		#[document_returns("The step wrapped in the applicative context.")]
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
		/// 	sequence::<StepDoneAppliedBrand<()>, _, OptionBrand>(Step::Loop(Some(5))),
		/// 	Some(Step::Loop(5))
		/// );
		/// assert_eq!(
		/// 	sequence::<StepDoneAppliedBrand<i32>, i32, OptionBrand>(Step::Done::<Option<i32>, _>(1)),
		/// 	Some(Step::Done::<i32, i32>(1))
		/// );
		/// ```
		fn sequence<'a, A: 'a + Clone, F: Applicative>(
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
		where
			Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
			Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone, {
			match ta {
				Step::Loop(fe) => F::map(|e| Step::Loop(e), fe),
				Step::Done(t) => F::pure(Step::Done(t)),
			}
		}
	}

	// Conversions: Step <-> Result

	/// Converts a [`Result`] into a [`Step`].
	///
	/// [`Ok(b)`] becomes [`Step::Done(b)`] and [`Err(a)`] becomes [`Step::Loop(a)`].
	#[document_type_parameters("The loop type (error type).", "The done type (ok type).")]
	impl<A, B> From<Result<B, A>> for Step<A, B> {
		fn from(result: Result<B, A>) -> Self {
			match result {
				Ok(b) => Step::Done(b),
				Err(a) => Step::Loop(a),
			}
		}
	}

	/// Converts a [`Step`] into a [`Result`].
	///
	/// [`Step::Done(b)`] becomes [`Ok(b)`] and [`Step::Loop(a)`] becomes [`Err(a)`].
	#[document_type_parameters("The loop type (error type).", "The done type (ok type).")]
	impl<A, B> From<Step<A, B>> for Result<B, A> {
		fn from(step: Step<A, B>) -> Self {
			match step {
				Step::Done(b) => Ok(b),
				Step::Loop(a) => Err(a),
			}
		}
	}

	// Conversions: Step <-> ControlFlow

	/// Converts a [`core::ops::ControlFlow`] into a [`Step`].
	///
	/// [`ControlFlow::Break(b)`] becomes [`Step::Done(b)`] and
	/// [`ControlFlow::Continue(a)`] becomes [`Step::Loop(a)`].
	#[document_type_parameters("The loop type (continue type).", "The done type (break type).")]
	impl<A, B> From<core::ops::ControlFlow<B, A>> for Step<A, B> {
		fn from(cf: core::ops::ControlFlow<B, A>) -> Self {
			match cf {
				core::ops::ControlFlow::Break(b) => Step::Done(b),
				core::ops::ControlFlow::Continue(a) => Step::Loop(a),
			}
		}
	}

	/// Converts a [`Step`] into a [`core::ops::ControlFlow`].
	///
	/// [`Step::Done(b)`] becomes [`ControlFlow::Break(b)`] and
	/// [`Step::Loop(a)`] becomes [`ControlFlow::Continue(a)`].
	#[document_type_parameters("The loop type (continue type).", "The done type (break type).")]
	impl<A, B> From<Step<A, B>> for core::ops::ControlFlow<B, A> {
		fn from(step: Step<A, B>) -> Self {
			match step {
				Step::Done(b) => core::ops::ControlFlow::Break(b),
				Step::Loop(a) => core::ops::ControlFlow::Continue(a),
			}
		}
	}
}
pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		super::*,
		crate::{
			brands::*,
			classes::{
				bifunctor::*,
				foldable::*,
				functor::*,
				lift::*,
				pointed::*,
				semiapplicative::*,
				semimonad::*,
				traversable::*,
			},
			functions::*,
		},
		quickcheck::{
			Arbitrary,
			Gen,
		},
		quickcheck_macros::quickcheck,
	};

	impl<A: Arbitrary, B: Arbitrary> Arbitrary for Step<A, B> {
		fn arbitrary(g: &mut Gen) -> Self {
			if bool::arbitrary(g) {
				Step::Loop(A::arbitrary(g))
			} else {
				Step::Done(B::arbitrary(g))
			}
		}
	}

	/// Tests the `is_loop` method.
	///
	/// Verifies that `is_loop` returns true for `Loop` variants and false for `Done` variants.
	#[test]
	fn test_is_loop() {
		let step: Step<i32, i32> = Step::Loop(1);
		assert!(step.is_loop());
		assert!(!step.is_done());
	}

	/// Tests the `is_done` method.
	///
	/// Verifies that `is_done` returns true for `Done` variants and false for `Loop` variants.
	#[test]
	fn test_is_done() {
		let step: Step<i32, i32> = Step::Done(1);
		assert!(step.is_done());
		assert!(!step.is_loop());
	}

	/// Tests the `map_loop` method.
	///
	/// Verifies that `map_loop` transforms the value inside a `Loop` variant and leaves a `Done` variant unchanged.
	#[test]
	fn test_map_loop() {
		let step: Step<i32, i32> = Step::Loop(1);
		let mapped = step.map_loop(|x| x + 1);
		assert_eq!(mapped, Step::Loop(2));

		let done: Step<i32, i32> = Step::Done(1);
		let mapped_done = done.map_loop(|x| x + 1);
		assert_eq!(mapped_done, Step::Done(1));
	}

	/// Tests the `map_done` method.
	///
	/// Verifies that `map_done` transforms the value inside a `Done` variant and leaves a `Loop` variant unchanged.
	#[test]
	fn test_map_done() {
		let step: Step<i32, i32> = Step::Done(1);
		let mapped = step.map_done(|x| x + 1);
		assert_eq!(mapped, Step::Done(2));

		let loop_step: Step<i32, i32> = Step::Loop(1);
		let mapped_loop = loop_step.map_done(|x| x + 1);
		assert_eq!(mapped_loop, Step::Loop(1));
	}

	/// Tests the `bimap` method.
	///
	/// Verifies that `bimap` transforms the value inside both `Loop` and `Done` variants using the appropriate function.
	#[test]
	fn test_bimap() {
		let step: Step<i32, i32> = Step::Loop(1);
		let mapped = step.bimap(|x| x + 1, |x| x * 2);
		assert_eq!(mapped, Step::Loop(2));

		let done: Step<i32, i32> = Step::Done(1);
		let mapped_done = done.bimap(|x| x + 1, |x| x * 2);
		assert_eq!(mapped_done, Step::Done(2));
	}

	/// Tests `Functor` implementation for `StepLoopAppliedBrand`.
	#[test]
	fn test_functor_step_with_loop() {
		let step: Step<i32, i32> = Step::Done(5);
		assert_eq!(map::<StepLoopAppliedBrand<_>, _, _>(|x: i32| x * 2, step), Step::Done(10));

		let loop_step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(map::<StepLoopAppliedBrand<_>, _, _>(|x: i32| x * 2, loop_step), Step::Loop(5));
	}

	/// Tests `Functor` implementation for `StepDoneAppliedBrand`.
	#[test]
	fn test_functor_step_with_done() {
		let step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(map::<StepDoneAppliedBrand<_>, _, _>(|x: i32| x * 2, step), Step::Loop(10));

		let done_step: Step<i32, i32> = Step::Done(5);
		assert_eq!(map::<StepDoneAppliedBrand<_>, _, _>(|x: i32| x * 2, done_step), Step::Done(5));
	}

	/// Tests `Bifunctor` implementation for `StepBrand`.
	#[test]
	fn test_bifunctor_step() {
		let step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(bimap::<StepBrand, _, _, _, _>(|a| a + 1, |b| b * 2, step), Step::Loop(6));

		let done: Step<i32, i32> = Step::Done(5);
		assert_eq!(bimap::<StepBrand, _, _, _, _>(|a| a + 1, |b| b * 2, done), Step::Done(10));
	}

	// Functor Laws for StepLoopAppliedBrand

	#[quickcheck]
	fn functor_identity_step_with_loop(x: Step<i32, i32>) -> bool {
		map::<StepLoopAppliedBrand<i32>, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_step_with_loop(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<StepLoopAppliedBrand<i32>, _, _>(compose(f, g), x)
			== map::<StepLoopAppliedBrand<i32>, _, _>(
				f,
				map::<StepLoopAppliedBrand<i32>, _, _>(g, x),
			)
	}

	// Functor Laws for StepDoneAppliedBrand

	#[quickcheck]
	fn functor_identity_step_with_done(x: Step<i32, i32>) -> bool {
		map::<StepDoneAppliedBrand<i32>, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_step_with_done(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<StepDoneAppliedBrand<i32>, _, _>(compose(f, g), x)
			== map::<StepDoneAppliedBrand<i32>, _, _>(
				f,
				map::<StepDoneAppliedBrand<i32>, _, _>(g, x),
			)
	}

	// Bifunctor Laws for StepBrand

	#[quickcheck]
	fn bifunctor_identity_step(x: Step<i32, i32>) -> bool {
		bimap::<StepBrand, _, _, _, _>(identity, identity, x) == x
	}

	#[quickcheck]
	fn bifunctor_composition_step(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<StepBrand, _, _, _, _>(compose(f, g), compose(h, i), x)
			== bimap::<StepBrand, _, _, _, _>(f, h, bimap::<StepBrand, _, _, _, _>(g, i, x))
	}

	// Lift Tests

	/// Tests the `lift2` function for `StepLoopAppliedBrand`.
	///
	/// Verifies that `lift2` correctly combines two `Step` values using a binary function,
	/// handling `Done` and `Loop` variants according to the `Lift` implementation.
	#[test]
	fn test_lift2_step_with_loop() {
		let s1: Step<i32, i32> = Step::Done(1);
		let s2: Step<i32, i32> = Step::Done(2);
		let s3: Step<i32, i32> = Step::Loop(3);

		assert_eq!(
			lift2::<StepLoopAppliedBrand<i32>, _, _, _>(|x, y| x + y, s1, s2),
			Step::Done(3)
		);
		assert_eq!(
			lift2::<StepLoopAppliedBrand<i32>, _, _, _>(|x, y| x + y, s1, s3),
			Step::Loop(3)
		);
	}

	/// Tests the `lift2` function for `StepDoneAppliedBrand`.
	///
	/// Verifies that `lift2` correctly combines two `Step` values using a binary function,
	/// handling `Done` and `Loop` variants according to the `Lift` implementation.
	#[test]
	fn test_lift2_step_with_done() {
		let s1: Step<i32, i32> = Step::Loop(1);
		let s2: Step<i32, i32> = Step::Loop(2);
		let s3: Step<i32, i32> = Step::Done(3);

		assert_eq!(
			lift2::<StepDoneAppliedBrand<i32>, _, _, _>(|x, y| x + y, s1, s2),
			Step::Loop(3)
		);
		assert_eq!(
			lift2::<StepDoneAppliedBrand<i32>, _, _, _>(|x, y| x + y, s1, s3),
			Step::Done(3)
		);
	}

	// Pointed Tests

	/// Tests the `pure` function for `StepLoopAppliedBrand`.
	///
	/// Verifies that `pure` wraps a value into a `Step::Done` variant.
	#[test]
	fn test_pointed_step_with_loop() {
		assert_eq!(pure::<StepLoopAppliedBrand<()>, _>(5), Step::Done(5));
	}

	/// Tests the `pure` function for `StepDoneAppliedBrand`.
	///
	/// Verifies that `pure` wraps a value into a `Step::Loop` variant.
	#[test]
	fn test_pointed_step_with_done() {
		assert_eq!(pure::<StepDoneAppliedBrand<()>, _>(5), Step::Loop(5));
	}

	// Semiapplicative Tests

	/// Tests the `apply` function for `StepLoopAppliedBrand`.
	///
	/// Verifies that `apply` correctly applies a wrapped function to a wrapped value,
	/// handling `Done` and `Loop` variants.
	#[test]
	fn test_apply_step_with_loop() {
		let f =
			pure::<StepLoopAppliedBrand<()>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| {
				x * 2
			}));
		let x = pure::<StepLoopAppliedBrand<()>, _>(5);
		assert_eq!(apply::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(f, x), Step::Done(10));

		let loop_step: Step<i32, _> = Step::Loop(1);
		let f_loop =
			pure::<StepLoopAppliedBrand<i32>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| {
				x * 2
			}));
		assert_eq!(
			apply::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _>(f_loop, loop_step),
			Step::Loop(1)
		);
	}

	/// Tests the `apply` function for `StepDoneAppliedBrand`.
	///
	/// Verifies that `apply` correctly applies a wrapped function to a wrapped value,
	/// handling `Done` and `Loop` variants.
	#[test]
	fn test_apply_step_with_done() {
		let f =
			pure::<StepDoneAppliedBrand<()>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| {
				x * 2
			}));
		let x = pure::<StepDoneAppliedBrand<()>, _>(5);
		assert_eq!(apply::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(f, x), Step::Loop(10));

		let done_step: Step<_, i32> = Step::Done(1);
		let f_done =
			pure::<StepDoneAppliedBrand<i32>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| {
				x * 2
			}));
		assert_eq!(
			apply::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _>(f_done, done_step),
			Step::Done(1)
		);
	}

	// Semimonad Tests

	/// Tests the `bind` function for `StepLoopAppliedBrand`.
	///
	/// Verifies that `bind` correctly chains computations, handling `Done` and `Loop` variants.
	#[test]
	fn test_bind_step_with_loop() {
		let x = pure::<StepLoopAppliedBrand<()>, _>(5);
		assert_eq!(
			bind::<StepLoopAppliedBrand<()>, _, _>(x, |i| pure::<StepLoopAppliedBrand<()>, _>(
				i * 2
			)),
			Step::Done(10)
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			bind::<StepLoopAppliedBrand<i32>, _, _>(loop_step, |i| pure::<
				StepLoopAppliedBrand<i32>,
				_,
			>(i * 2)),
			Step::Loop(1)
		);
	}

	/// Tests the `bind` function for `StepDoneAppliedBrand`.
	///
	/// Verifies that `bind` correctly chains computations, handling `Done` and `Loop` variants.
	#[test]
	fn test_bind_step_with_done() {
		let x = pure::<StepDoneAppliedBrand<()>, _>(5);
		assert_eq!(
			bind::<StepDoneAppliedBrand<()>, _, _>(x, |i| pure::<StepDoneAppliedBrand<()>, _>(
				i * 2
			)),
			Step::Loop(10)
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			bind::<StepDoneAppliedBrand<i32>, _, _>(done_step, |i| pure::<
				StepDoneAppliedBrand<i32>,
				_,
			>(i * 2)),
			Step::Done(1)
		);
	}

	// Foldable Tests

	/// Tests `Foldable` methods for `StepLoopAppliedBrand`.
	///
	/// Verifies `fold_right`, `fold_left`, and `fold_map` behavior for `Done` and `Loop` variants.
	#[test]
	fn test_foldable_step_with_loop() {
		let x = pure::<StepLoopAppliedBrand<()>, _>(5);
		assert_eq!(
			fold_right::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(|a, b| a + b, 10, x),
			15
		);
		assert_eq!(fold_left::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(|b, a| b + a, 10, x), 15);
		assert_eq!(
			fold_map::<RcFnBrand, StepLoopAppliedBrand<()>, _, _>(|a: i32| a.to_string(), x),
			"5"
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			fold_right::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _>(|a, b| a + b, 10, loop_step),
			10
		);
	}

	/// Tests `Foldable` methods for `StepDoneAppliedBrand`.
	///
	/// Verifies `fold_right`, `fold_left`, and `fold_map` behavior for `Done` and `Loop` variants.
	#[test]
	fn test_foldable_step_with_done() {
		let x = pure::<StepDoneAppliedBrand<()>, _>(5);
		assert_eq!(
			fold_right::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(|a, b| a + b, 10, x),
			15
		);
		assert_eq!(fold_left::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(|b, a| b + a, 10, x), 15);
		assert_eq!(
			fold_map::<RcFnBrand, StepDoneAppliedBrand<()>, _, _>(|a: i32| a.to_string(), x),
			"5"
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			fold_right::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _>(|a, b| a + b, 10, done_step),
			10
		);
	}

	// Traversable Tests

	/// Tests the `traverse` function for `StepLoopAppliedBrand`.
	///
	/// Verifies that `traverse` correctly maps and sequences effects over `Step`.
	#[test]
	fn test_traversable_step_with_loop() {
		let x = pure::<StepLoopAppliedBrand<()>, _>(5);
		assert_eq!(
			traverse::<StepLoopAppliedBrand<()>, _, _, OptionBrand>(|a| Some(a * 2), x),
			Some(Step::Done(10))
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			traverse::<StepLoopAppliedBrand<i32>, _, _, OptionBrand>(|a| Some(a * 2), loop_step),
			Some(Step::Loop(1))
		);
	}

	/// Tests the `traverse` function for `StepDoneAppliedBrand`.
	///
	/// Verifies that `traverse` correctly maps and sequences effects over `Step`.
	#[test]
	fn test_traversable_step_with_done() {
		let x = pure::<StepDoneAppliedBrand<()>, _>(5);
		assert_eq!(
			traverse::<StepDoneAppliedBrand<()>, _, _, OptionBrand>(|a| Some(a * 2), x),
			Some(Step::Loop(10))
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			traverse::<StepDoneAppliedBrand<i32>, _, _, OptionBrand>(|a| Some(a * 2), done_step),
			Some(Step::Done(1))
		);
	}

	// Monad Laws for StepLoopAppliedBrand

	/// Verifies the Left Identity law for `StepLoopAppliedBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_step_with_loop(a: i32) -> bool {
		let f = |x: i32| pure::<StepLoopAppliedBrand<i32>, _>(x.wrapping_mul(2));
		bind::<StepLoopAppliedBrand<i32>, _, _>(pure::<StepLoopAppliedBrand<i32>, _>(a), f) == f(a)
	}

	/// Verifies the Right Identity law for `StepLoopAppliedBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_step_with_loop(x: Step<i32, i32>) -> bool {
		bind::<StepLoopAppliedBrand<i32>, _, _>(x, pure::<StepLoopAppliedBrand<i32>, _>) == x
	}

	/// Verifies the Associativity law for `StepLoopAppliedBrand` Monad.
	#[quickcheck]
	fn monad_associativity_step_with_loop(x: Step<i32, i32>) -> bool {
		let f = |x: i32| pure::<StepLoopAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<StepLoopAppliedBrand<i32>, _>(x.wrapping_add(1));
		bind::<StepLoopAppliedBrand<i32>, _, _>(bind::<StepLoopAppliedBrand<i32>, _, _>(x, f), g)
			== bind::<StepLoopAppliedBrand<i32>, _, _>(x, |a| {
				bind::<StepLoopAppliedBrand<i32>, _, _>(f(a), g)
			})
	}

	// Monad Laws for StepDoneAppliedBrand

	/// Verifies the Left Identity law for `StepDoneAppliedBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_step_with_done(a: i32) -> bool {
		let f = |x: i32| pure::<StepDoneAppliedBrand<i32>, _>(x.wrapping_mul(2));
		bind::<StepDoneAppliedBrand<i32>, _, _>(pure::<StepDoneAppliedBrand<i32>, _>(a), f) == f(a)
	}

	/// Verifies the Right Identity law for `StepDoneAppliedBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_step_with_done(x: Step<i32, i32>) -> bool {
		bind::<StepDoneAppliedBrand<i32>, _, _>(x, pure::<StepDoneAppliedBrand<i32>, _>) == x
	}

	/// Verifies the Associativity law for `StepDoneAppliedBrand` Monad.
	#[quickcheck]
	fn monad_associativity_step_with_done(x: Step<i32, i32>) -> bool {
		let f = |x: i32| pure::<StepDoneAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<StepDoneAppliedBrand<i32>, _>(x.wrapping_add(1));
		bind::<StepDoneAppliedBrand<i32>, _, _>(bind::<StepDoneAppliedBrand<i32>, _, _>(x, f), g)
			== bind::<StepDoneAppliedBrand<i32>, _, _>(x, |a| {
				bind::<StepDoneAppliedBrand<i32>, _, _>(f(a), g)
			})
	}

	// Applicative and Monad marker trait verification

	/// Verifies that `StepLoopAppliedBrand` satisfies the `Applicative` trait.
	#[test]
	fn test_applicative_step_loop_applied() {
		fn assert_applicative<B: crate::classes::Applicative>() {}
		assert_applicative::<StepLoopAppliedBrand<i32>>();
	}

	/// Verifies that `StepDoneAppliedBrand` satisfies the `Applicative` trait.
	#[test]
	fn test_applicative_step_done_applied() {
		fn assert_applicative<B: crate::classes::Applicative>() {}
		assert_applicative::<StepDoneAppliedBrand<i32>>();
	}

	/// Verifies that `StepLoopAppliedBrand` satisfies the `Monad` trait.
	#[test]
	fn test_monad_step_loop_applied() {
		fn assert_monad<B: crate::classes::Monad>() {}
		assert_monad::<StepLoopAppliedBrand<i32>>();
	}

	/// Verifies that `StepDoneAppliedBrand` satisfies the `Monad` trait.
	#[test]
	fn test_monad_step_done_applied() {
		fn assert_monad<B: crate::classes::Monad>() {}
		assert_monad::<StepDoneAppliedBrand<i32>>();
	}

	// Conversion Tests: Step <-> Result

	/// Tests converting `Result` to `Step`.
	#[test]
	fn test_from_result_to_step() {
		let ok: Result<i32, &str> = Ok(42);
		assert_eq!(Step::from(ok), Step::Done(42));

		let err: Result<i32, &str> = Err("error");
		assert_eq!(Step::from(err), Step::Loop("error"));
	}

	/// Tests converting `Step` to `Result`.
	#[test]
	fn test_from_step_to_result() {
		let done: Step<&str, i32> = Step::Done(42);
		assert_eq!(Result::from(done), Ok(42));

		let loop_step: Step<&str, i32> = Step::Loop("error");
		assert_eq!(Result::from(loop_step), Err("error"));
	}

	/// Property: `Step -> Result -> Step` round-trips.
	#[quickcheck]
	fn step_result_roundtrip(x: Step<i32, i32>) -> bool {
		let result: Result<i32, i32> = Result::from(x);
		Step::from(result) == x
	}

	/// Property: `Result -> Step -> Result` round-trips.
	#[quickcheck]
	fn result_step_roundtrip(
		ok: bool,
		val: i32,
	) -> bool {
		let result: Result<i32, i32> = if ok { Ok(val) } else { Err(val) };
		let step: Step<i32, i32> = Step::from(result);
		Result::from(step) == result
	}

	// Conversion Tests: Step <-> ControlFlow

	/// Tests converting `ControlFlow` to `Step`.
	#[test]
	fn test_from_control_flow_to_step() {
		use core::ops::ControlFlow;

		let brk: ControlFlow<i32, &str> = ControlFlow::Break(42);
		assert_eq!(Step::from(brk), Step::Done(42));

		let cont: ControlFlow<i32, &str> = ControlFlow::Continue("again");
		assert_eq!(Step::from(cont), Step::Loop("again"));
	}

	/// Tests converting `Step` to `ControlFlow`.
	#[test]
	fn test_from_step_to_control_flow() {
		use core::ops::ControlFlow;

		let done: Step<&str, i32> = Step::Done(42);
		assert_eq!(ControlFlow::from(done), ControlFlow::Break(42));

		let loop_step: Step<&str, i32> = Step::Loop("again");
		assert_eq!(ControlFlow::from(loop_step), ControlFlow::Continue("again"));
	}

	/// Property: `Step -> ControlFlow -> Step` round-trips.
	#[quickcheck]
	fn step_control_flow_roundtrip(x: Step<i32, i32>) -> bool {
		let cf: core::ops::ControlFlow<i32, i32> = core::ops::ControlFlow::from(x);
		Step::from(cf) == x
	}
}
