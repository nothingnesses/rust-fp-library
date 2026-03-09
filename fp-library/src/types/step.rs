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
				Bifunctor,
				CloneableFn,
				Foldable,
				Functor,
				Lift,
				Monoid,
				ParFoldable,
				Pointed,
				Semiapplicative,
				Semimonad,
				SendCloneableFn,
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
		/// Continue the loop with a new value
		Loop(A),
		/// Finish the computation with a final value
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
			"The type of the mapped done value.",
			"The type of the function to apply to the loop value.",
			"The type of the function to apply to the done value."
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
		/// assert_eq!(bimap::<StepBrand, _, _, _, _, _, _>(|a| a + 1, |b: i32| b * 2, x), Step::Loop(2));
		/// ```
		fn bimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, F, G>(
			f: F,
			g: G,
			p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>)
		where
			F: Fn(A) -> B + 'a,
			G: Fn(C) -> D + 'a, {
			p.bimap(f, g)
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
			"The type of the result of applying the function.",
			"The type of the function to apply."
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
		/// 	map::<StepLoopAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Step::<i32, i32>::Done(5)),
		/// 	Step::Done(10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> B + 'a, {
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
			"The type of the result.",
			"The type of the binary function."
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
		/// 	lift2::<StepLoopAppliedBrand<()>, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Done(1),
		/// 		Step::Done(2)
		/// 	),
		/// 	Step::Done(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<StepLoopAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Done(1),
		/// 		Step::Loop(2)
		/// 	),
		/// 	Step::Loop(2)
		/// );
		/// ```
		fn lift2<'a, A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			Func: Fn(A, B) -> C + 'a,
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
			"The type of the result of the second computation.",
			"The type of the function to apply."
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
		/// 	bind::<StepLoopAppliedBrand<()>, _, _, _>(Step::Done(5), |x| Step::Done(x * 2)),
		/// 	Step::Done(10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a, Func>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a, {
			match ma {
				Step::Done(a) => func(a),
				Step::Loop(e) => Step::Loop(e),
			}
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
			"The type of the accumulator.",
			"The type of the folding function."
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
		/// 	fold_right::<RcFnBrand, StepLoopAppliedBrand<()>, _, _, _>(
		/// 		|x, acc| x + acc,
		/// 		0,
		/// 		Step::Done(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		Step::Loop(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(A, B) -> B + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa {
				Step::Done(a) => func(a, initial),
				Step::Loop(_) => initial,
			}
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
			"The type of the accumulator.",
			"The type of the folding function."
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
		/// 	fold_left::<RcFnBrand, StepLoopAppliedBrand<()>, _, _, _>(
		/// 		|acc, x| acc + x,
		/// 		0,
		/// 		Step::Done(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		Step::Loop(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(B, A) -> B + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa {
				Step::Done(a) => func(initial, a),
				Step::Loop(_) => initial,
			}
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
			"The type of the monoid.",
			"The type of the mapping function."
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
		/// 	fold_map::<RcFnBrand, StepLoopAppliedBrand<()>, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Done(5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Loop(1)
		/// 	),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M, F>(
			func: F,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			F: Fn(A) -> M + 'a,
			FnBrand: CloneableFn + 'a, {
			match fa {
				Step::Done(a) => func(a),
				Step::Loop(_) => M::empty(),
			}
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
			"The applicative context.",
			"The type of the function to apply."
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
		/// 	traverse::<StepLoopAppliedBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Step::Done(5)),
		/// 	Some(Step::Done(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<StepLoopAppliedBrand<i32>, _, _, OptionBrand, _>(
		/// 		|x: i32| Some(x * 2),
		/// 		Step::Loop(1)
		/// 	),
		/// 	Some(Step::Loop(1))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
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

	#[document_type_parameters("The loop type.")]
	impl<LoopType: 'static> ParFoldable for StepLoopAppliedBrand<LoopType> {
		/// Maps the value to a monoid and returns it, or returns empty, in parallel.
		///
		/// This method maps the element of the step to a monoid and then returns it. The mapping operation may be executed in parallel.
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
			"The step to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
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
		/// let x: Step<i32, i32> = Step::Done(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, StepLoopAppliedBrand<i32>, _, _>(f.clone(), x),
		/// 	"5".to_string()
		/// );
		///
		/// let x_loop: Step<i32, i32> = Step::Loop(1);
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, StepLoopAppliedBrand<i32>, _, _>(f, x_loop),
		/// 	"".to_string()
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
			match fa {
				Step::Done(a) => func(a),
				Step::Loop(_) => M::empty(),
			}
		}

		/// Folds the step from the right in parallel.
		///
		/// This method folds the step by applying a function from right to left, potentially in parallel.
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
			"The step to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
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
		/// let x: Step<i32, i32> = Step::Done(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		/// assert_eq!(par_fold_right::<ArcFnBrand, StepLoopAppliedBrand<i32>, _, _>(f.clone(), 10, x), 15);
		///
		/// let x_loop: Step<i32, i32> = Step::Loop(1);
		/// assert_eq!(par_fold_right::<ArcFnBrand, StepLoopAppliedBrand<i32>, _, _>(f, 10, x_loop), 10);
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
			match fa {
				Step::Done(a) => func((a, initial)),
				Step::Loop(_) => initial,
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
			"The type of the result of applying the function.",
			"The type of the function to apply."
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
		/// 	map::<StepDoneAppliedBrand<i32>, _, _, _>(|x: i32| x * 2, Step::<i32, i32>::Loop(5)),
		/// 	Step::Loop(10)
		/// );
		/// ```
		fn map<'a, A: 'a, B: 'a, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> B + 'a, {
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
			"The type of the result loop value.",
			"The type of the binary function."
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
		/// 	lift2::<StepDoneAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Loop(1),
		/// 		Step::Loop(2)
		/// 	),
		/// 	Step::Loop(3)
		/// );
		/// assert_eq!(
		/// 	lift2::<StepDoneAppliedBrand<i32>, _, _, _, _>(
		/// 		|x: i32, y: i32| x + y,
		/// 		Step::Loop(1),
		/// 		Step::Done(2)
		/// 	),
		/// 	Step::Done(2)
		/// );
		/// ```
		fn lift2<'a, A, B, C, Func>(
			func: Func,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			fb: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
		where
			Func: Fn(A, B) -> C + 'a,
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
			"The type of the result of the second computation.",
			"The type of the function to apply."
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
		/// 	bind::<StepDoneAppliedBrand<()>, _, _, _>(Step::Loop(5), |x| Step::Loop(x * 2)),
		/// 	Step::Loop(10)
		/// );
		/// ```
		fn bind<'a, A: 'a, B: 'a, Func>(
			ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			func: Func,
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
		where
			Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a, {
			match ma {
				Step::Done(t) => Step::Done(t),
				Step::Loop(e) => func(e),
			}
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
			"The type of the accumulator.",
			"The type of the folding function."
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
		/// 	fold_right::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		Step::Loop(1)
		/// 	),
		/// 	1
		/// );
		/// assert_eq!(
		/// 	fold_right::<RcFnBrand, StepDoneAppliedBrand<()>, _, _, _>(
		/// 		|x: i32, acc| x + acc,
		/// 		0,
		/// 		Step::Done(())
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(A, B) -> B + 'a,
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
			"The type of the accumulator.",
			"The type of the folding function."
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
		/// 	fold_left::<RcFnBrand, StepDoneAppliedBrand<()>, _, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		Step::Loop(5)
		/// 	),
		/// 	5
		/// );
		/// assert_eq!(
		/// 	fold_left::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _, _>(
		/// 		|acc, x: i32| acc + x,
		/// 		0,
		/// 		Step::Done(1)
		/// 	),
		/// 	0
		/// );
		/// ```
		fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
			func: F,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			F: Fn(B, A) -> B + 'a,
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
			"The type of the monoid.",
			"The type of the mapping function."
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
		/// 	fold_map::<RcFnBrand, StepDoneAppliedBrand<()>, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Loop(5)
		/// 	),
		/// 	"5".to_string()
		/// );
		/// assert_eq!(
		/// 	fold_map::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _, _>(
		/// 		|x: i32| x.to_string(),
		/// 		Step::Done(1)
		/// 	),
		/// 	"".to_string()
		/// );
		/// ```
		fn fold_map<'a, FnBrand, A: 'a, M, F>(
			func: F,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			F: Fn(A) -> M + 'a,
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
			"The applicative context.",
			"The type of the function to apply."
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
		/// 	traverse::<StepDoneAppliedBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Step::Loop(5)),
		/// 	Some(Step::Loop(10))
		/// );
		/// assert_eq!(
		/// 	traverse::<StepDoneAppliedBrand<i32>, _, _, OptionBrand, _>(
		/// 		|x: i32| Some(x * 2),
		/// 		Step::Done(1)
		/// 	),
		/// 	Some(Step::Done(1))
		/// );
		/// ```
		fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
			func: Func,
			ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		where
			Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
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

	#[document_type_parameters("The done type.")]
	impl<DoneType: 'static> ParFoldable for StepDoneAppliedBrand<DoneType> {
		/// Maps the value to a monoid and returns it, or returns empty, in parallel (over loop).
		///
		/// This method maps the element of the step to a monoid and then returns it (over loop). The mapping operation may be executed in parallel.
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
			"The step to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
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
		/// let x: Step<i32, i32> = Step::Loop(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, StepDoneAppliedBrand<i32>, _, _>(f.clone(), x),
		/// 	"5".to_string()
		/// );
		///
		/// let x_done: Step<i32, i32> = Step::Done(1);
		/// assert_eq!(
		/// 	par_fold_map::<ArcFnBrand, StepDoneAppliedBrand<i32>, _, _>(f, x_done),
		/// 	"".to_string()
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
			match fa {
				Step::Loop(e) => func(e),
				Step::Done(_) => M::empty(),
			}
		}

		/// Folds the step from the right in parallel (over loop).
		///
		/// This method folds the step by applying a function from right to left, potentially in parallel (over loop).
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
			"The step to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
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
		/// let x: Step<i32, i32> = Step::Loop(5);
		/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
		/// assert_eq!(par_fold_right::<ArcFnBrand, StepDoneAppliedBrand<i32>, _, _>(f.clone(), 10, x), 15);
		///
		/// let x_done: Step<i32, i32> = Step::Done(1);
		/// assert_eq!(par_fold_right::<ArcFnBrand, StepDoneAppliedBrand<i32>, _, _>(f, 10, x_done), 10);
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
			match fa {
				Step::Loop(e) => func((e, initial)),
				Step::Done(_) => initial,
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
				par_foldable::*,
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
		assert_eq!(map::<StepLoopAppliedBrand<_>, _, _, _>(|x: i32| x * 2, step), Step::Done(10));

		let loop_step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(
			map::<StepLoopAppliedBrand<_>, _, _, _>(|x: i32| x * 2, loop_step),
			Step::Loop(5)
		);
	}

	/// Tests `Functor` implementation for `StepDoneAppliedBrand`.
	#[test]
	fn test_functor_step_with_done() {
		let step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(map::<StepDoneAppliedBrand<_>, _, _, _>(|x: i32| x * 2, step), Step::Loop(10));

		let done_step: Step<i32, i32> = Step::Done(5);
		assert_eq!(
			map::<StepDoneAppliedBrand<_>, _, _, _>(|x: i32| x * 2, done_step),
			Step::Done(5)
		);
	}

	/// Tests `Bifunctor` implementation for `StepBrand`.
	#[test]
	fn test_bifunctor_step() {
		let step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(bimap::<StepBrand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, step), Step::Loop(6));

		let done: Step<i32, i32> = Step::Done(5);
		assert_eq!(
			bimap::<StepBrand, _, _, _, _, _, _>(|a| a + 1, |b| b * 2, done),
			Step::Done(10)
		);
	}

	// Functor Laws for StepLoopAppliedBrand

	#[quickcheck]
	fn functor_identity_step_with_loop(x: Step<i32, i32>) -> bool {
		map::<StepLoopAppliedBrand<i32>, _, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_step_with_loop(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<StepLoopAppliedBrand<i32>, _, _, _>(compose(f, g), x)
			== map::<StepLoopAppliedBrand<i32>, _, _, _>(
				f,
				map::<StepLoopAppliedBrand<i32>, _, _, _>(g, x),
			)
	}

	// Functor Laws for StepDoneAppliedBrand

	#[quickcheck]
	fn functor_identity_step_with_done(x: Step<i32, i32>) -> bool {
		map::<StepDoneAppliedBrand<i32>, _, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_step_with_done(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<StepDoneAppliedBrand<i32>, _, _, _>(compose(f, g), x)
			== map::<StepDoneAppliedBrand<i32>, _, _, _>(
				f,
				map::<StepDoneAppliedBrand<i32>, _, _, _>(g, x),
			)
	}

	// Bifunctor Laws for StepBrand

	#[quickcheck]
	fn bifunctor_identity_step(x: Step<i32, i32>) -> bool {
		bimap::<StepBrand, _, _, _, _, _, _>(identity, identity, x) == x
	}

	#[quickcheck]
	fn bifunctor_composition_step(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		let h = |x: i32| x.wrapping_sub(1);
		let i = |x: i32| if x == 0 { 0 } else { x.wrapping_div(2) };

		bimap::<StepBrand, _, _, _, _, _, _>(compose(f, g), compose(h, i), x)
			== bimap::<StepBrand, _, _, _, _, _, _>(
				f,
				h,
				bimap::<StepBrand, _, _, _, _, _, _>(g, i, x),
			)
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
			lift2::<StepLoopAppliedBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s2),
			Step::Done(3)
		);
		assert_eq!(
			lift2::<StepLoopAppliedBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s3),
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
			lift2::<StepDoneAppliedBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s2),
			Step::Loop(3)
		);
		assert_eq!(
			lift2::<StepDoneAppliedBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s3),
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
			bind::<StepLoopAppliedBrand<()>, _, _, _>(x, |i| pure::<StepLoopAppliedBrand<()>, _>(
				i * 2
			)),
			Step::Done(10)
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			bind::<StepLoopAppliedBrand<i32>, _, _, _>(loop_step, |i| pure::<
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
			bind::<StepDoneAppliedBrand<()>, _, _, _>(x, |i| pure::<StepDoneAppliedBrand<()>, _>(
				i * 2
			)),
			Step::Loop(10)
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			bind::<StepDoneAppliedBrand<i32>, _, _, _>(done_step, |i| pure::<
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
			fold_right::<RcFnBrand, StepLoopAppliedBrand<()>, _, _, _>(|a, b| a + b, 10, x),
			15
		);
		assert_eq!(
			fold_left::<RcFnBrand, StepLoopAppliedBrand<()>, _, _, _>(|b, a| b + a, 10, x),
			15
		);
		assert_eq!(
			fold_map::<RcFnBrand, StepLoopAppliedBrand<()>, _, _, _>(|a: i32| a.to_string(), x),
			"5"
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			fold_right::<RcFnBrand, StepLoopAppliedBrand<i32>, _, _, _>(
				|a, b| a + b,
				10,
				loop_step
			),
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
			fold_right::<RcFnBrand, StepDoneAppliedBrand<()>, _, _, _>(|a, b| a + b, 10, x),
			15
		);
		assert_eq!(
			fold_left::<RcFnBrand, StepDoneAppliedBrand<()>, _, _, _>(|b, a| b + a, 10, x),
			15
		);
		assert_eq!(
			fold_map::<RcFnBrand, StepDoneAppliedBrand<()>, _, _, _>(|a: i32| a.to_string(), x),
			"5"
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			fold_right::<RcFnBrand, StepDoneAppliedBrand<i32>, _, _, _>(
				|a, b| a + b,
				10,
				done_step
			),
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
			traverse::<StepLoopAppliedBrand<()>, _, _, OptionBrand, _>(|a| Some(a * 2), x),
			Some(Step::Done(10))
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			traverse::<StepLoopAppliedBrand<i32>, _, _, OptionBrand, _>(|a| Some(a * 2), loop_step),
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
			traverse::<StepDoneAppliedBrand<()>, _, _, OptionBrand, _>(|a| Some(a * 2), x),
			Some(Step::Loop(10))
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			traverse::<StepDoneAppliedBrand<i32>, _, _, OptionBrand, _>(|a| Some(a * 2), done_step),
			Some(Step::Done(1))
		);
	}

	// ParFoldable Tests

	/// Tests `par_fold_map` for `StepLoopAppliedBrand`.
	///
	/// Verifies parallel folding behavior (conceptually, as it delegates to sequential for simple types).
	#[test]
	fn test_par_foldable_step_with_loop() {
		let x = pure::<StepLoopAppliedBrand<()>, _>(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|a: i32| a.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, StepLoopAppliedBrand<()>, _, _>(f, x), "5");
	}

	/// Tests `par_fold_map` for `StepDoneAppliedBrand`.
	///
	/// Verifies parallel folding behavior.
	#[test]
	fn test_par_foldable_step_with_done() {
		let x = pure::<StepDoneAppliedBrand<()>, _>(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|a: i32| a.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, StepDoneAppliedBrand<()>, _, _>(f, x), "5");
	}

	// Monad Laws for StepLoopAppliedBrand

	/// Verifies the Left Identity law for `StepLoopAppliedBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_step_with_loop(a: i32) -> bool {
		let f = |x: i32| pure::<StepLoopAppliedBrand<i32>, _>(x.wrapping_mul(2));
		bind::<StepLoopAppliedBrand<i32>, _, _, _>(pure::<StepLoopAppliedBrand<i32>, _>(a), f)
			== f(a)
	}

	/// Verifies the Right Identity law for `StepLoopAppliedBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_step_with_loop(x: Step<i32, i32>) -> bool {
		bind::<StepLoopAppliedBrand<i32>, _, _, _>(x, pure::<StepLoopAppliedBrand<i32>, _>) == x
	}

	/// Verifies the Associativity law for `StepLoopAppliedBrand` Monad.
	#[quickcheck]
	fn monad_associativity_step_with_loop(x: Step<i32, i32>) -> bool {
		let f = |x: i32| pure::<StepLoopAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<StepLoopAppliedBrand<i32>, _>(x.wrapping_add(1));
		bind::<StepLoopAppliedBrand<i32>, _, _, _>(
			bind::<StepLoopAppliedBrand<i32>, _, _, _>(x, f),
			g,
		) == bind::<StepLoopAppliedBrand<i32>, _, _, _>(x, |a| {
			bind::<StepLoopAppliedBrand<i32>, _, _, _>(f(a), g)
		})
	}

	// Monad Laws for StepDoneAppliedBrand

	/// Verifies the Left Identity law for `StepDoneAppliedBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_step_with_done(a: i32) -> bool {
		let f = |x: i32| pure::<StepDoneAppliedBrand<i32>, _>(x.wrapping_mul(2));
		bind::<StepDoneAppliedBrand<i32>, _, _, _>(pure::<StepDoneAppliedBrand<i32>, _>(a), f)
			== f(a)
	}

	/// Verifies the Right Identity law for `StepDoneAppliedBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_step_with_done(x: Step<i32, i32>) -> bool {
		bind::<StepDoneAppliedBrand<i32>, _, _, _>(x, pure::<StepDoneAppliedBrand<i32>, _>) == x
	}

	/// Verifies the Associativity law for `StepDoneAppliedBrand` Monad.
	#[quickcheck]
	fn monad_associativity_step_with_done(x: Step<i32, i32>) -> bool {
		let f = |x: i32| pure::<StepDoneAppliedBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<StepDoneAppliedBrand<i32>, _>(x.wrapping_add(1));
		bind::<StepDoneAppliedBrand<i32>, _, _, _>(
			bind::<StepDoneAppliedBrand<i32>, _, _, _>(x, f),
			g,
		) == bind::<StepDoneAppliedBrand<i32>, _, _, _>(x, |a| {
			bind::<StepDoneAppliedBrand<i32>, _, _, _>(f(a), g)
		})
	}
}
