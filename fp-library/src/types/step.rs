use fp_macros::doc_type_params;
use crate::{
	Apply,
	brands::{StepBrand, StepWithDoneBrand, StepWithLoopBrand},
	classes::{
		applicative::Applicative, apply_first::ApplyFirst, apply_second::ApplySecond,
		bifunctor::Bifunctor, cloneable_fn::CloneableFn, foldable::Foldable, functor::Functor,
		lift::Lift, monoid::Monoid, par_foldable::ParFoldable, pointed::Pointed,
		semiapplicative::Semiapplicative, semimonad::Semimonad, send_cloneable_fn::SendCloneableFn,
		traversable::Traversable,
	},
	impl_kind,
	kinds::*,
};
use fp_macros::hm_signature;

/// Represents the result of a single step in a tail-recursive computation.
///
/// This type is fundamental to stack-safe recursion via `MonadRec`.
///
/// ### Type Parameters
///
/// * `A`: The "loop" type - when we return `Loop(a)`, we continue with `a`.
/// * `B`: The "done" type - when we return `Done(b)`, we're finished.
///
/// ### Variants
///
/// * `Loop(A)`: Continue the loop with a new value.
/// * `Done(B)`: Finish the computation with a final value.
///
/// ### Examples
///
/// ```
/// use fp_library::types::*;
///
/// let loop_step: Step<i32, i32> = Step::Loop(10);
/// let done_step: Step<i32, i32> = Step::Done(20);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step<A, B> {
	/// Continue the loop with a new value
	Loop(A),
	/// Finish the computation with a final value
	Done(B),
}

impl<A, B> Step<A, B> {
	/// Returns `true` if this is a `Loop` variant.
	///
	/// ### Type Signature
	///
	/// `forall b a. Step a b -> bool`
	///
	/// ### Returns
	///
	/// `true` if the step is a loop, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let step: Step<i32, i32> = Step::Loop(1);
	/// assert!(step.is_loop());
	/// ```
	#[inline]
	pub fn is_loop(&self) -> bool {
		matches!(self, Step::Loop(_))
	}

	/// Returns `true` if this is a `Done` variant.
	///
	/// ### Type Signature
	///
	/// `forall b a. Step a b -> bool`
	///
	/// ### Returns
	///
	/// `true` if the step is done, `false` otherwise.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::*;
	///
	/// let step: Step<i32, i32> = Step::Done(1);
	/// assert!(step.is_done());
	/// ```
	#[inline]
	pub fn is_done(&self) -> bool {
		matches!(self, Step::Done(_))
	}

	/// Maps a function over the `Loop` variant.
	///
	/// ### Type Signature
	///
	/// `forall c b a. (a -> c, Step a b) -> Step c b`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The new loop type."
	)]	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the loop value.
	///
	/// ### Returns
	///
	/// A new `Step` with the loop value transformed.
	///
	/// ### Examples
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
	///
	/// ### Type Signature
	///
	/// `forall c b a. (b -> c, Step a b) -> Step a c`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The new done type."
	)]	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the done value.
	///
	/// ### Returns
	///
	/// A new `Step` with the done value transformed.
	///
	/// ### Examples
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
	///
	/// ### Type Signature
	///
	/// `forall d c b a. (a -> c, b -> d, Step a b) -> Step c d`
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"The new loop type.",
		"The new done type."
	)]	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the loop value.
	/// * `g`: The function to apply to the done value.
	///
	/// ### Returns
	///
	/// A new `Step` with both values transformed.
	///
	/// ### Examples
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
	///
	/// ### Type Signature
	///
	#[hm_signature(Bifunctor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the loop value.",
		"The type of the mapped loop value.",
		"The type of the done value.",
		"The type of the mapped done value.",
		"The type of the function to apply to the loop value.",
		"The type of the function to apply to the done value."
	)]	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the loop value.
	/// * `g`: The function to apply to the done value.
	/// * `p`: The step to map over.
	///
	/// ### Returns
	///
	/// A new step containing the mapped values.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::bifunctor::*, functions::*, types::*};
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
		G: Fn(C) -> D + 'a,
	{
		p.bimap(f, g)
	}
}

// StepWithLoopBrand<LoopType> (Functor over B - Done)

impl_kind! {
	impl<LoopType: 'static> for StepWithLoopBrand<LoopType> {
		type Of<'a, B: 'a>: 'a = Step<LoopType, B>;
	}
}

impl<LoopType: 'static> Functor for StepWithLoopBrand<LoopType> {
	/// Maps a function over the done value in the step.
	///
	/// This method applies a function to the done value inside the step, producing a new step with the transformed done value. The loop value remains unchanged.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the done value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the done value.
	/// * `fa`: The step to map over.
	///
	/// ### Returns
	///
	/// A new step containing the result of applying the function to the done value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(map::<StepWithLoopBrand<i32>, _, _, _>(|x: i32| x * 2, Step::<i32, i32>::Done(5)), Step::Done(10));
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		fa.map_done(func)
	}
}

impl<LoopType: Clone + 'static> Lift for StepWithLoopBrand<LoopType> {
	/// Lifts a binary function into the step context.
	///
	/// This method lifts a binary function to operate on values within the step context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"The type of the binary function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The binary function to apply.
	/// * `fa`: The first step.
	/// * `fb`: The second step.
	///
	/// ### Returns
	///
	/// `Done(f(a, b))` if both steps are `Done`, otherwise the first loop encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     lift2::<StepWithLoopBrand<()>, _, _, _, _>(|x: i32, y: i32| x + y, Step::Done(1), Step::Done(2)),
	///     Step::Done(3)
	/// );
	/// assert_eq!(
	///     lift2::<StepWithLoopBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Step::Done(1), Step::Loop(2)),
	///     Step::Loop(2)
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
		C: 'a,
	{
		match (fa, fb) {
			(Step::Done(a), Step::Done(b)) => Step::Done(func(a, b)),
			(Step::Loop(e), _) => Step::Loop(e),
			(_, Step::Loop(e)) => Step::Loop(e),
		}
	}
}

impl<LoopType: 'static> Pointed for StepWithLoopBrand<LoopType> {
	/// Wraps a value in a step.
	///
	/// This method wraps a value in the `Done` variant of a `Step`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the value to wrap."
	)]	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// `Done(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(pure::<StepWithLoopBrand<()>, _>(5), Step::Done(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Step::Done(a)
	}
}

impl<LoopType: Clone + 'static> ApplyFirst for StepWithLoopBrand<LoopType> {}
impl<LoopType: Clone + 'static> ApplySecond for StepWithLoopBrand<LoopType> {}

impl<LoopType: Clone + 'static> Semiapplicative for StepWithLoopBrand<LoopType> {
	/// Applies a wrapped function to a wrapped value.
	///
	/// This method applies a function wrapped in a step to a value wrapped in a step.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]	///
	/// ### Parameters
	///
	/// * `ff`: The step containing the function.
	/// * `fa`: The step containing the value.
	///
	/// ### Returns
	///
	/// `Done(f(a))` if both are `Done`, otherwise the first loop encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let f: Step<_, _> = Step::Done(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, StepWithLoopBrand<()>, _, _>(f, Step::Done(5)), Step::Done(10));
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

impl<LoopType: Clone + 'static> Semimonad for StepWithLoopBrand<LoopType> {
	/// Chains step computations.
	///
	/// This method chains two computations, where the second computation depends on the result of the first.
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		("A", "The type of the result of the first computation.")
	)]	///
	/// ### Parameters
	///
	/// * `ma`: The first step.
	/// * `f`: The function to apply to the value inside the step.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the value if `ma` is `Done`, otherwise the original loop.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     bind::<StepWithLoopBrand<()>, _, _, _>(Step::Done(5), |x| Step::Done(x * 2)),
	///     Step::Done(10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		match ma {
			Step::Done(a) => func(a),
			Step::Loop(e) => Step::Loop(e),
		}
	}
}

impl<LoopType: 'static> Foldable for StepWithLoopBrand<LoopType> {
	/// Folds the step from the right.
	///
	/// This method performs a right-associative fold of the step.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)` if `fa` is `Done(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_right::<RcFnBrand, StepWithLoopBrand<()>, _, _, _>(|x, acc| x + acc, 0, Step::Done(5)), 5);
	/// assert_eq!(fold_right::<RcFnBrand, StepWithLoopBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Step::Loop(1)), 0);
	/// ```
	fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
		func: F,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		match fa {
			Step::Done(a) => func(a, initial),
			Step::Loop(_) => initial,
		}
	}

	/// Folds the step from the left.
	///
	/// This method performs a left-associative fold of the step.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// `func(initial, a)` if `fa` is `Done(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_left::<RcFnBrand, StepWithLoopBrand<()>, _, _, _>(|acc, x| acc + x, 0, Step::Done(5)), 5);
	/// assert_eq!(fold_left::<RcFnBrand, StepWithLoopBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Step::Loop(1)), 0);
	/// ```
	fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
		func: F,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		match fa {
			Step::Done(a) => func(initial, a),
			Step::Loop(_) => initial,
		}
	}

	/// Maps the value to a monoid and returns it.
	///
	/// This method maps the element of the step to a monoid and then returns it.
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// `func(a)` if `fa` is `Done(a)`, otherwise `M::empty()`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, StepWithLoopBrand<()>, _, _, _>(|x: i32| x.to_string(), Step::Done(5)),
	///     "5".to_string()
	/// );
	/// assert_eq!(
	///     fold_map::<RcFnBrand, StepWithLoopBrand<i32>, _, _, _>(|x: i32| x.to_string(), Step::Loop(1)),
	///     "".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, A: 'a, M, F>(
		func: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		match fa {
			Step::Done(a) => func(a),
			Step::Loop(_) => M::empty(),
		}
	}
}

impl<LoopType: Clone + 'static> Traversable for StepWithLoopBrand<LoopType> {
	/// Traverses the step with an applicative function.
	///
	/// This method maps the element of the step to a computation, evaluates it, and combines the result into an applicative context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `ta`: The step to traverse.
	///
	/// ### Returns
	///
	/// The step wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     traverse::<StepWithLoopBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Step::Done(5)),
	///     Some(Step::Done(10))
	/// );
	/// assert_eq!(
	///     traverse::<StepWithLoopBrand<i32>, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Step::Loop(1)),
	///     Some(Step::Loop(1))
	/// );
	/// ```
	fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		match ta {
			Step::Done(a) => F::map(|b| Step::Done(b), func(a)),
			Step::Loop(e) => F::pure(Step::Loop(e)),
		}
	}

	/// Sequences a step of applicative.
	///
	/// This method evaluates the computation inside the step and accumulates the result into an applicative context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]	///
	/// ### Parameters
	///
	/// * `ta`: The step containing the applicative value.
	///
	/// ### Returns
	///
	/// The step wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     sequence::<StepWithLoopBrand<()>, _, OptionBrand>(Step::Done(Some(5))),
	///     Some(Step::Done(5))
	/// );
	/// assert_eq!(
	///     sequence::<StepWithLoopBrand<i32>, i32, OptionBrand>(Step::Loop::<i32, Option<i32>>(1)),
	///     Some(Step::Loop::<i32, i32>(1))
	/// );
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		match ta {
			Step::Done(fa) => F::map(|a| Step::Done(a), fa),
			Step::Loop(e) => F::pure(Step::Loop(e)),
		}
	}
}

impl<LoopType: 'static> ParFoldable for StepWithLoopBrand<LoopType> {
	/// Maps the value to a monoid and returns it, or returns empty, in parallel.
	///
	/// This method maps the element of the step to a monoid and then returns it. The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of thread-safe function to use.",
		"The element type (must be `Send + Sync`).",
		"The monoid type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let x: Step<i32, i32> = Step::Done(5);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, StepWithLoopBrand<i32>, _, _>(f.clone(), x), "5".to_string());
	///
	/// let x_loop: Step<i32, i32> = Step::Loop(1);
	/// assert_eq!(par_fold_map::<ArcFnBrand, StepWithLoopBrand<i32>, _, _>(f, x_loop), "".to_string());
	/// ```
	fn par_fold_map<'a, FnBrand, A, M>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		match fa {
			Step::Done(a) => func(a),
			Step::Loop(_) => M::empty(),
		}
	}

	/// Folds the step from the right in parallel.
	///
	/// This method folds the step by applying a function from right to left, potentially in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of thread-safe function to use.",
		"The element type (must be `Send + Sync`).",
		"The accumulator type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to apply to each element and the accumulator.
	/// * `initial`: The initial value.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let x: Step<i32, i32> = Step::Done(5);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, StepWithLoopBrand<i32>, _, _>(f.clone(), 10, x), 15);
	///
	/// let x_loop: Step<i32, i32> = Step::Loop(1);
	/// assert_eq!(par_fold_right::<ArcFnBrand, StepWithLoopBrand<i32>, _, _>(f, 10, x_loop), 10);
	/// ```
	fn par_fold_right<'a, FnBrand, A, B>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		match fa {
			Step::Done(a) => func((a, initial)),
			Step::Loop(_) => initial,
		}
	}
}

// StepWithDoneBrand<DoneType> (Functor over A - Loop)

impl_kind! {
	impl<DoneType: 'static> for StepWithDoneBrand<DoneType> {
		type Of<'a, A: 'a>: 'a = Step<A, DoneType>;
	}
}

impl<DoneType: 'static> Functor for StepWithDoneBrand<DoneType> {
	/// Maps a function over the loop value in the step.
	///
	/// This method applies a function to the loop value inside the step, producing a new step with the transformed loop value. The done value remains unchanged.
	///
	/// ### Type Signature
	///
	#[hm_signature(Functor)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the loop value.",
		"The type of the result of applying the function.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the loop value.
	/// * `fa`: The step to map over.
	///
	/// ### Returns
	///
	/// A new step containing the result of applying the function to the loop value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(map::<StepWithDoneBrand<i32>, _, _, _>(|x: i32| x * 2, Step::<i32, i32>::Loop(5)), Step::Loop(10));
	/// ```
	fn map<'a, A: 'a, B: 'a, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> B + 'a,
	{
		fa.map_loop(func)
	}
}

impl<DoneType: Clone + 'static> Lift for StepWithDoneBrand<DoneType> {
	/// Lifts a binary function into the step context (over loop).
	///
	/// This method lifts a binary function to operate on loop values within the step context.
	///
	/// ### Type Signature
	///
	#[hm_signature(Lift)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the first loop value.",
		"The type of the second loop value.",
		"The type of the result loop value.",
		"The type of the binary function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The binary function to apply to the loops.
	/// * `fa`: The first step.
	/// * `fb`: The second step.
	///
	/// ### Returns
	///
	/// `Loop(f(a, b))` if both steps are `Loop`, otherwise the first done encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     lift2::<StepWithDoneBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Step::Loop(1), Step::Loop(2)),
	///     Step::Loop(3)
	/// );
	/// assert_eq!(
	///     lift2::<StepWithDoneBrand<i32>, _, _, _, _>(|x: i32, y: i32| x + y, Step::Loop(1), Step::Done(2)),
	///     Step::Done(2)
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
		C: 'a,
	{
		match (fa, fb) {
			(Step::Loop(a), Step::Loop(b)) => Step::Loop(func(a, b)),
			(Step::Done(t), _) => Step::Done(t),
			(_, Step::Done(t)) => Step::Done(t),
		}
	}
}

impl<DoneType: 'static> Pointed for StepWithDoneBrand<DoneType> {
	/// Wraps a value in a step (as loop).
	///
	/// This method wraps a value in the `Loop` variant of a `Step`.
	///
	/// ### Type Signature
	///
	#[hm_signature(Pointed)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the value to wrap."
	)]	///
	/// ### Parameters
	///
	/// * `a`: The value to wrap.
	///
	/// ### Returns
	///
	/// `Loop(a)`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(pure::<StepWithDoneBrand<()>, _>(5), Step::Loop(5));
	/// ```
	fn pure<'a, A: 'a>(a: A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		Step::Loop(a)
	}
}

impl<DoneType: Clone + 'static> ApplyFirst for StepWithDoneBrand<DoneType> {}
impl<DoneType: Clone + 'static> ApplySecond for StepWithDoneBrand<DoneType> {}

impl<DoneType: Clone + 'static> Semiapplicative for StepWithDoneBrand<DoneType> {
	/// Applies a wrapped function to a wrapped value (over loop).
	///
	/// This method applies a function wrapped in a step (as loop) to a value wrapped in a step (as loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Semiapplicative)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function wrapper.",
		"The type of the input value.",
		"The type of the output value."
	)]	///
	/// ### Parameters
	///
	/// * `ff`: The step containing the function (in Loop).
	/// * `fa`: The step containing the value (in Loop).
	///
	/// ### Returns
	///
	/// `Loop(f(a))` if both are `Loop`, otherwise the first done encountered.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let f: Step<_, ()> = Step::Loop(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
	/// assert_eq!(apply::<RcFnBrand, StepWithDoneBrand<()>, _, _>(f, Step::Loop(5)), Step::Loop(10));
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

impl<DoneType: Clone + 'static> Semimonad for StepWithDoneBrand<DoneType> {
	/// Chains step computations (over loop).
	///
	/// This method chains two computations, where the second computation depends on the result of the first (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Semimonad)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the result of the first computation.",
		"The type of the result of the second computation.",
		("A", "The type of the result of the first computation.")
	)]	///
	/// ### Parameters
	///
	/// * `ma`: The first step.
	/// * `f`: The function to apply to the loop value.
	///
	/// ### Returns
	///
	/// The result of applying `f` to the loop if `ma` is `Loop`, otherwise the original done.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     bind::<StepWithDoneBrand<()>, _, _, _>(Step::Loop(5), |x| Step::Loop(x * 2)),
	///     Step::Loop(10)
	/// );
	/// ```
	fn bind<'a, A: 'a, B: 'a, Func>(
		ma: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		func: Func,
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Func: Fn(A) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
	{
		match ma {
			Step::Done(t) => Step::Done(t),
			Step::Loop(e) => func(e),
		}
	}
}

impl<DoneType: 'static> Foldable for StepWithDoneBrand<DoneType> {
	/// Folds the step from the right (over loop).
	///
	/// This method performs a right-associative fold of the step (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// `func(a, initial)` if `fa` is `Loop(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_right::<RcFnBrand, StepWithDoneBrand<i32>, _, _, _>(|x: i32, acc| x + acc, 0, Step::Loop(1)), 1);
	/// assert_eq!(fold_right::<RcFnBrand, StepWithDoneBrand<()>, _, _, _>(|x: i32, acc| x + acc, 0, Step::Done(())), 0);
	/// ```
	fn fold_right<'a, FnBrand, A: 'a, B: 'a, F>(
		func: F,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		match fa {
			Step::Loop(e) => func(e, initial),
			Step::Done(_) => initial,
		}
	}

	/// Folds the step from the left (over loop).
	///
	/// This method performs a left-associative fold of the step (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The folding function.
	/// * `initial`: The initial value.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// `func(initial, a)` if `fa` is `Loop(a)`, otherwise `initial`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(fold_left::<RcFnBrand, StepWithDoneBrand<()>, _, _, _>(|acc, x: i32| acc + x, 0, Step::Loop(5)), 5);
	/// assert_eq!(fold_left::<RcFnBrand, StepWithDoneBrand<i32>, _, _, _>(|acc, x: i32| acc + x, 0, Step::Done(1)), 0);
	/// ```
	fn fold_left<'a, FnBrand, A: 'a, B: 'a, F>(
		func: F,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		FnBrand: CloneableFn + 'a,
	{
		match fa {
			Step::Loop(e) => func(initial, e),
			Step::Done(_) => initial,
		}
	}

	/// Maps the value to a monoid and returns it (over loop).
	///
	/// This method maps the element of the step to a monoid and then returns it (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Foldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The mapping function.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// `func(a)` if `fa` is `Loop(a)`, otherwise `M::empty()`.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     fold_map::<RcFnBrand, StepWithDoneBrand<()>, _, _, _>(|x: i32| x.to_string(), Step::Loop(5)),
	///     "5".to_string()
	/// );
	/// assert_eq!(
	///     fold_map::<RcFnBrand, StepWithDoneBrand<i32>, _, _, _>(|x: i32| x.to_string(), Step::Done(1)),
	///     "".to_string()
	/// );
	/// ```
	fn fold_map<'a, FnBrand, A: 'a, M, F>(
		func: F,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		FnBrand: CloneableFn + 'a,
	{
		match fa {
			Step::Loop(e) => func(e),
			Step::Done(_) => M::empty(),
		}
	}
}

impl<DoneType: Clone + 'static> Traversable for StepWithDoneBrand<DoneType> {
	/// Traverses the step with an applicative function (over loop).
	///
	/// This method maps the element of the step to a computation, evaluates it, and combines the result into an applicative context (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The type of the elements in the resulting traversable structure.",
		"The applicative context.",
		"The type of the function to apply."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The function to apply.
	/// * `ta`: The step to traverse.
	///
	/// ### Returns
	///
	/// The step wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     traverse::<StepWithDoneBrand<()>, _, _, OptionBrand, _>(|x| Some(x * 2), Step::Loop(5)),
	///     Some(Step::Loop(10))
	/// );
	/// assert_eq!(
	///     traverse::<StepWithDoneBrand<i32>, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Step::Done(1)),
	///     Some(Step::Done(1))
	/// );
	/// ```
	fn traverse<'a, A: 'a + Clone, B: 'a + Clone, F: Applicative, Func>(
		func: Func,
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		Func: Fn(A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
	{
		match ta {
			Step::Loop(e) => F::map(|b| Step::Loop(b), func(e)),
			Step::Done(t) => F::pure(Step::Done(t)),
		}
	}

	/// Sequences a step of applicative (over loop).
	///
	/// This method evaluates the computation inside the step and accumulates the result into an applicative context (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(Traversable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The type of the elements in the traversable structure.",
		"The applicative context."
	)]	///
	/// ### Parameters
	///
	/// * `ta`: The step containing the applicative value.
	///
	/// ### Returns
	///
	/// The step wrapped in the applicative context.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*, types::*};
	///
	/// assert_eq!(
	///     sequence::<StepWithDoneBrand<()>, _, OptionBrand>(Step::Loop(Some(5))),
	///     Some(Step::Loop(5))
	/// );
	/// assert_eq!(
	///     sequence::<StepWithDoneBrand<i32>, i32, OptionBrand>(Step::Done::<Option<i32>, _>(1)),
	///     Some(Step::Done::<i32, i32>(1))
	/// );
	/// ```
	fn sequence<'a, A: 'a + Clone, F: Applicative>(
		ta: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)>)
	where
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
		Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>): Clone,
	{
		match ta {
			Step::Loop(fe) => F::map(|e| Step::Loop(e), fe),
			Step::Done(t) => F::pure(Step::Done(t)),
		}
	}
}

impl<DoneType: 'static> ParFoldable for StepWithDoneBrand<DoneType> {
	/// Maps the value to a monoid and returns it, or returns empty, in parallel (over loop).
	///
	/// This method maps the element of the step to a monoid and then returns it (over loop). The mapping operation may be executed in parallel.
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of thread-safe function to use.",
		"The element type (must be `Send + Sync`).",
		"The monoid type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to map each element to a monoid.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let x: Step<i32, i32> = Step::Loop(5);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|x: i32| x.to_string());
	/// assert_eq!(par_fold_map::<ArcFnBrand, StepWithDoneBrand<i32>, _, _>(f.clone(), x), "5".to_string());
	///
	/// let x_done: Step<i32, i32> = Step::Done(1);
	/// assert_eq!(par_fold_map::<ArcFnBrand, StepWithDoneBrand<i32>, _, _>(f, x_done), "".to_string());
	/// ```
	fn par_fold_map<'a, FnBrand, A, M>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, A, M>,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		M: Monoid + Send + Sync + 'a,
	{
		match fa {
			Step::Loop(e) => func(e),
			Step::Done(_) => M::empty(),
		}
	}

	/// Folds the step from the right in parallel (over loop).
	///
	/// This method folds the step by applying a function from right to left, potentially in parallel (over loop).
	///
	/// ### Type Signature
	///
	#[hm_signature(ParFoldable)]
	///
	/// ### Type Parameters
	///
	#[doc_type_params(
		"Undocumented",
		"The brand of thread-safe function to use.",
		"The element type (must be `Send + Sync`).",
		"The accumulator type (must be `Send + Sync`)."
	)]	///
	/// ### Parameters
	///
	/// * `func`: The thread-safe function to apply to each element and the accumulator.
	/// * `initial`: The initial value.
	/// * `fa`: The step to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, classes::*, functions::*, types::*};
	///
	/// let x: Step<i32, i32> = Step::Loop(5);
	/// let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|(a, b): (i32, i32)| a + b);
	/// assert_eq!(par_fold_right::<ArcFnBrand, StepWithDoneBrand<i32>, _, _>(f.clone(), 10, x), 15);
	///
	/// let x_done: Step<i32, i32> = Step::Done(1);
	/// assert_eq!(par_fold_right::<ArcFnBrand, StepWithDoneBrand<i32>, _, _>(f, 10, x_done), 10);
	/// ```
	fn par_fold_right<'a, FnBrand, A, B>(
		func: <FnBrand as SendCloneableFn>::SendOf<'a, (A, B), B>,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		FnBrand: 'a + SendCloneableFn,
		A: 'a + Clone + Send + Sync,
		B: Send + Sync + 'a,
	{
		match fa {
			Step::Loop(e) => func((e, initial)),
			Step::Done(_) => initial,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		brands::*,
		classes::{
			bifunctor::*, foldable::*, functor::*, lift::*, par_foldable::*, pointed::*,
			semiapplicative::*, semimonad::*, traversable::*,
		},
		functions::*,
	};
	use quickcheck::{Arbitrary, Gen};
	use quickcheck_macros::quickcheck;

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

	/// Tests `Functor` implementation for `StepWithLoopBrand`.
	#[test]
	fn test_functor_step_with_loop() {
		let step: Step<i32, i32> = Step::Done(5);
		assert_eq!(map::<StepWithLoopBrand<_>, _, _, _>(|x: i32| x * 2, step), Step::Done(10));

		let loop_step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(map::<StepWithLoopBrand<_>, _, _, _>(|x: i32| x * 2, loop_step), Step::Loop(5));
	}

	/// Tests `Functor` implementation for `StepWithDoneBrand`.
	#[test]
	fn test_functor_step_with_done() {
		let step: Step<i32, i32> = Step::Loop(5);
		assert_eq!(map::<StepWithDoneBrand<_>, _, _, _>(|x: i32| x * 2, step), Step::Loop(10));

		let done_step: Step<i32, i32> = Step::Done(5);
		assert_eq!(map::<StepWithDoneBrand<_>, _, _, _>(|x: i32| x * 2, done_step), Step::Done(5));
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

	// Functor Laws for StepWithLoopBrand

	#[quickcheck]
	fn functor_identity_step_with_loop(x: Step<i32, i32>) -> bool {
		map::<StepWithLoopBrand<i32>, _, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_step_with_loop(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<StepWithLoopBrand<i32>, _, _, _>(compose(f, g), x)
			== map::<StepWithLoopBrand<i32>, _, _, _>(
				f,
				map::<StepWithLoopBrand<i32>, _, _, _>(g, x),
			)
	}

	// Functor Laws for StepWithDoneBrand

	#[quickcheck]
	fn functor_identity_step_with_done(x: Step<i32, i32>) -> bool {
		map::<StepWithDoneBrand<i32>, _, _, _>(identity, x) == x
	}

	#[quickcheck]
	fn functor_composition_step_with_done(x: Step<i32, i32>) -> bool {
		let f = |x: i32| x.wrapping_add(1);
		let g = |x: i32| x.wrapping_mul(2);
		map::<StepWithDoneBrand<i32>, _, _, _>(compose(f, g), x)
			== map::<StepWithDoneBrand<i32>, _, _, _>(
				f,
				map::<StepWithDoneBrand<i32>, _, _, _>(g, x),
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

	/// Tests the `lift2` function for `StepWithLoopBrand`.
	///
	/// Verifies that `lift2` correctly combines two `Step` values using a binary function,
	/// handling `Done` and `Loop` variants according to the `Lift` implementation.
	#[test]
	fn test_lift2_step_with_loop() {
		let s1: Step<i32, i32> = Step::Done(1);
		let s2: Step<i32, i32> = Step::Done(2);
		let s3: Step<i32, i32> = Step::Loop(3);

		assert_eq!(
			lift2::<StepWithLoopBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s2),
			Step::Done(3)
		);
		assert_eq!(
			lift2::<StepWithLoopBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s3),
			Step::Loop(3)
		);
	}

	/// Tests the `lift2` function for `StepWithDoneBrand`.
	///
	/// Verifies that `lift2` correctly combines two `Step` values using a binary function,
	/// handling `Done` and `Loop` variants according to the `Lift` implementation.
	#[test]
	fn test_lift2_step_with_done() {
		let s1: Step<i32, i32> = Step::Loop(1);
		let s2: Step<i32, i32> = Step::Loop(2);
		let s3: Step<i32, i32> = Step::Done(3);

		assert_eq!(
			lift2::<StepWithDoneBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s2),
			Step::Loop(3)
		);
		assert_eq!(
			lift2::<StepWithDoneBrand<i32>, _, _, _, _>(|x, y| x + y, s1, s3),
			Step::Done(3)
		);
	}

	// Pointed Tests

	/// Tests the `pure` function for `StepWithLoopBrand`.
	///
	/// Verifies that `pure` wraps a value into a `Step::Done` variant.
	#[test]
	fn test_pointed_step_with_loop() {
		assert_eq!(pure::<StepWithLoopBrand<()>, _>(5), Step::Done(5));
	}

	/// Tests the `pure` function for `StepWithDoneBrand`.
	///
	/// Verifies that `pure` wraps a value into a `Step::Loop` variant.
	#[test]
	fn test_pointed_step_with_done() {
		assert_eq!(pure::<StepWithDoneBrand<()>, _>(5), Step::Loop(5));
	}

	// Semiapplicative Tests

	/// Tests the `apply` function for `StepWithLoopBrand`.
	///
	/// Verifies that `apply` correctly applies a wrapped function to a wrapped value,
	/// handling `Done` and `Loop` variants.
	#[test]
	fn test_apply_step_with_loop() {
		let f =
			pure::<StepWithLoopBrand<()>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let x = pure::<StepWithLoopBrand<()>, _>(5);
		assert_eq!(apply::<RcFnBrand, StepWithLoopBrand<()>, _, _>(f, x), Step::Done(10));

		let loop_step: Step<i32, _> = Step::Loop(1);
		let f_loop =
			pure::<StepWithLoopBrand<i32>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		assert_eq!(
			apply::<RcFnBrand, StepWithLoopBrand<i32>, _, _>(f_loop, loop_step),
			Step::Loop(1)
		);
	}

	/// Tests the `apply` function for `StepWithDoneBrand`.
	///
	/// Verifies that `apply` correctly applies a wrapped function to a wrapped value,
	/// handling `Done` and `Loop` variants.
	#[test]
	fn test_apply_step_with_done() {
		let f =
			pure::<StepWithDoneBrand<()>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		let x = pure::<StepWithDoneBrand<()>, _>(5);
		assert_eq!(apply::<RcFnBrand, StepWithDoneBrand<()>, _, _>(f, x), Step::Loop(10));

		let done_step: Step<_, i32> = Step::Done(1);
		let f_done =
			pure::<StepWithDoneBrand<i32>, _>(cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2));
		assert_eq!(
			apply::<RcFnBrand, StepWithDoneBrand<i32>, _, _>(f_done, done_step),
			Step::Done(1)
		);
	}

	// Semimonad Tests

	/// Tests the `bind` function for `StepWithLoopBrand`.
	///
	/// Verifies that `bind` correctly chains computations, handling `Done` and `Loop` variants.
	#[test]
	fn test_bind_step_with_loop() {
		let x = pure::<StepWithLoopBrand<()>, _>(5);
		assert_eq!(
			bind::<StepWithLoopBrand<()>, _, _, _>(x, |i| pure::<StepWithLoopBrand<()>, _>(i * 2)),
			Step::Done(10)
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			bind::<StepWithLoopBrand<i32>, _, _, _>(
				loop_step,
				|i| pure::<StepWithLoopBrand<i32>, _>(i * 2)
			),
			Step::Loop(1)
		);
	}

	/// Tests the `bind` function for `StepWithDoneBrand`.
	///
	/// Verifies that `bind` correctly chains computations, handling `Done` and `Loop` variants.
	#[test]
	fn test_bind_step_with_done() {
		let x = pure::<StepWithDoneBrand<()>, _>(5);
		assert_eq!(
			bind::<StepWithDoneBrand<()>, _, _, _>(x, |i| pure::<StepWithDoneBrand<()>, _>(i * 2)),
			Step::Loop(10)
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			bind::<StepWithDoneBrand<i32>, _, _, _>(
				done_step,
				|i| pure::<StepWithDoneBrand<i32>, _>(i * 2)
			),
			Step::Done(1)
		);
	}

	// Foldable Tests

	/// Tests `Foldable` methods for `StepWithLoopBrand`.
	///
	/// Verifies `fold_right`, `fold_left`, and `fold_map` behavior for `Done` and `Loop` variants.
	#[test]
	fn test_foldable_step_with_loop() {
		let x = pure::<StepWithLoopBrand<()>, _>(5);
		assert_eq!(
			fold_right::<RcFnBrand, StepWithLoopBrand<()>, _, _, _>(|a, b| a + b, 10, x),
			15
		);
		assert_eq!(fold_left::<RcFnBrand, StepWithLoopBrand<()>, _, _, _>(|b, a| b + a, 10, x), 15);
		assert_eq!(
			fold_map::<RcFnBrand, StepWithLoopBrand<()>, _, _, _>(|a: i32| a.to_string(), x),
			"5"
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			fold_right::<RcFnBrand, StepWithLoopBrand<i32>, _, _, _>(|a, b| a + b, 10, loop_step),
			10
		);
	}

	/// Tests `Foldable` methods for `StepWithDoneBrand`.
	///
	/// Verifies `fold_right`, `fold_left`, and `fold_map` behavior for `Done` and `Loop` variants.
	#[test]
	fn test_foldable_step_with_done() {
		let x = pure::<StepWithDoneBrand<()>, _>(5);
		assert_eq!(
			fold_right::<RcFnBrand, StepWithDoneBrand<()>, _, _, _>(|a, b| a + b, 10, x),
			15
		);
		assert_eq!(fold_left::<RcFnBrand, StepWithDoneBrand<()>, _, _, _>(|b, a| b + a, 10, x), 15);
		assert_eq!(
			fold_map::<RcFnBrand, StepWithDoneBrand<()>, _, _, _>(|a: i32| a.to_string(), x),
			"5"
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			fold_right::<RcFnBrand, StepWithDoneBrand<i32>, _, _, _>(|a, b| a + b, 10, done_step),
			10
		);
	}

	// Traversable Tests

	/// Tests the `traverse` function for `StepWithLoopBrand`.
	///
	/// Verifies that `traverse` correctly maps and sequences effects over `Step`.
	#[test]
	fn test_traversable_step_with_loop() {
		let x = pure::<StepWithLoopBrand<()>, _>(5);
		assert_eq!(
			traverse::<StepWithLoopBrand<()>, _, _, OptionBrand, _>(|a| Some(a * 2), x),
			Some(Step::Done(10))
		);

		let loop_step: Step<i32, i32> = Step::Loop(1);
		assert_eq!(
			traverse::<StepWithLoopBrand<i32>, _, _, OptionBrand, _>(|a| Some(a * 2), loop_step),
			Some(Step::Loop(1))
		);
	}

	/// Tests the `traverse` function for `StepWithDoneBrand`.
	///
	/// Verifies that `traverse` correctly maps and sequences effects over `Step`.
	#[test]
	fn test_traversable_step_with_done() {
		let x = pure::<StepWithDoneBrand<()>, _>(5);
		assert_eq!(
			traverse::<StepWithDoneBrand<()>, _, _, OptionBrand, _>(|a| Some(a * 2), x),
			Some(Step::Loop(10))
		);

		let done_step: Step<i32, i32> = Step::Done(1);
		assert_eq!(
			traverse::<StepWithDoneBrand<i32>, _, _, OptionBrand, _>(|a| Some(a * 2), done_step),
			Some(Step::Done(1))
		);
	}

	// ParFoldable Tests

	/// Tests `par_fold_map` for `StepWithLoopBrand`.
	///
	/// Verifies parallel folding behavior (conceptually, as it delegates to sequential for simple types).
	#[test]
	fn test_par_foldable_step_with_loop() {
		let x = pure::<StepWithLoopBrand<()>, _>(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|a: i32| a.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, StepWithLoopBrand<()>, _, _>(f, x), "5");
	}

	/// Tests `par_fold_map` for `StepWithDoneBrand`.
	///
	/// Verifies parallel folding behavior.
	#[test]
	fn test_par_foldable_step_with_done() {
		let x = pure::<StepWithDoneBrand<()>, _>(5);
		let f = send_cloneable_fn_new::<ArcFnBrand, _, _>(|a: i32| a.to_string());
		assert_eq!(par_fold_map::<ArcFnBrand, StepWithDoneBrand<()>, _, _>(f, x), "5");
	}

	// Monad Laws for StepWithLoopBrand

	/// Verifies the Left Identity law for `StepWithLoopBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_step_with_loop(a: i32) -> bool {
		let f = |x: i32| pure::<StepWithLoopBrand<i32>, _>(x.wrapping_mul(2));
		bind::<StepWithLoopBrand<i32>, _, _, _>(pure::<StepWithLoopBrand<i32>, _>(a), f) == f(a)
	}

	/// Verifies the Right Identity law for `StepWithLoopBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_step_with_loop(x: Step<i32, i32>) -> bool {
		bind::<StepWithLoopBrand<i32>, _, _, _>(x, pure::<StepWithLoopBrand<i32>, _>) == x
	}

	/// Verifies the Associativity law for `StepWithLoopBrand` Monad.
	#[quickcheck]
	fn monad_associativity_step_with_loop(x: Step<i32, i32>) -> bool {
		let f = |x: i32| pure::<StepWithLoopBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<StepWithLoopBrand<i32>, _>(x.wrapping_add(1));
		bind::<StepWithLoopBrand<i32>, _, _, _>(bind::<StepWithLoopBrand<i32>, _, _, _>(x, f), g)
			== bind::<StepWithLoopBrand<i32>, _, _, _>(x, |a| {
				bind::<StepWithLoopBrand<i32>, _, _, _>(f(a), g)
			})
	}

	// Monad Laws for StepWithDoneBrand

	/// Verifies the Left Identity law for `StepWithDoneBrand` Monad.
	#[quickcheck]
	fn monad_left_identity_step_with_done(a: i32) -> bool {
		let f = |x: i32| pure::<StepWithDoneBrand<i32>, _>(x.wrapping_mul(2));
		bind::<StepWithDoneBrand<i32>, _, _, _>(pure::<StepWithDoneBrand<i32>, _>(a), f) == f(a)
	}

	/// Verifies the Right Identity law for `StepWithDoneBrand` Monad.
	#[quickcheck]
	fn monad_right_identity_step_with_done(x: Step<i32, i32>) -> bool {
		bind::<StepWithDoneBrand<i32>, _, _, _>(x, pure::<StepWithDoneBrand<i32>, _>) == x
	}

	/// Verifies the Associativity law for `StepWithDoneBrand` Monad.
	#[quickcheck]
	fn monad_associativity_step_with_done(x: Step<i32, i32>) -> bool {
		let f = |x: i32| pure::<StepWithDoneBrand<i32>, _>(x.wrapping_mul(2));
		let g = |x: i32| pure::<StepWithDoneBrand<i32>, _>(x.wrapping_add(1));
		bind::<StepWithDoneBrand<i32>, _, _, _>(bind::<StepWithDoneBrand<i32>, _, _, _>(x, f), g)
			== bind::<StepWithDoneBrand<i32>, _, _, _>(x, |a| {
				bind::<StepWithDoneBrand<i32>, _, _, _>(f(a), g)
			})
	}
}
