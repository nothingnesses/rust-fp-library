//! Multi-shot delimited continuations on the `Rc` store: the `SubShift`
//! effect and its `run_sub_shift` delimiter.
//!
//! [`sub_shift`] captures the continuation from the operation site to the
//! enclosing delimiter as a first-class re-callable value: the capture body
//! receives it as an `Rc<dyn Fn>` and may invoke it once per branch, each
//! invocation re-running the rest of the source program to the delimiter
//! and yielding a fresh answer program over the residual row. This is the
//! multi-shot sibling of the one-shot
//! [`Shift`](crate::types::effects::shift) effect, which stays on the `Box`
//! store with a `FnOnce` continuation; the two tiers keep separate cells
//! and runner families, the store axis marked by the names. The module is
//! pinned to the `Rc` store because re-callable continuations exist only
//! in the multi-shot stores' `Fn` form; the `Arc` tier joins when a `Send`
//! consumer brings the `SendFunctor` composition route.
//!
//! Calling one captured continuation once per branch is the primitive
//! first-order nondeterminism falls out of: a capture body that invokes
//! its continuation with `true` and then `false` and concatenates the
//! answers reproduces the [`Alt`](crate::types::effects::alt) effect's
//! fan-out, which is how the reference implementation derives
//! nondeterminism from `shift`.
//!
//! Handler state under multi-shot capture follows invocation-time
//! threading: the state at the moment a continuation is re-invoked flows
//! forward, with no automatic rollback
//! ([`transact_state`](crate::types::effects::state::transact_state) is
//! the rollback opt-in). A threaded accumulator fold stacked under the
//! delimiter rides the re-emitted capture cell and forks per re-entry,
//! exactly as it forks per branch under the `Alt` runners; a
//! shared-by-reference cell is global across re-entries unless its handler
//! snapshots the cell on entry and restores it on exit, the documented
//! pattern for cell-backed handlers that need per-entry isolation.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::RcBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Coyoneda,
				Free,
				FreeStep,
				closure_storage::ValueFor,
				effects::{
					coproduct::{
						CoprodInjector,
						CoprodUninjector,
						CoproductEmbedder,
					},
					order::{
						FirstOrder,
						OrderOf,
					},
				},
			},
		},
		fp_macros::*,
		std::{
			marker::PhantomData,
			rc::Rc,
		},
	};

	/// The reified re-callable continuation from a capture point to its
	/// delimiter: each invocation re-runs the rest of the source program to
	/// the delimiter and yields a fresh answer program over the residual
	/// row, so a body may invoke it once per branch. Dropping it aborts the
	/// unresumed tail, structurally.
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type (what each invocation passes in)."
	)]
	pub type SubShiftExit<Narrow, Ans, V> = Rc<dyn Fn(V) -> Free<Narrow, Ans, RcBrand>>;

	/// A capture body: from the reified re-callable continuation to the
	/// prompt's answer program over the residual row. The body itself is
	/// re-callable, because a capture reconstructed by a re-callable spine
	/// continuation (a capture under an outer fork) re-runs its body per
	/// encounter.
	#[document_type_parameters(
		"The lifetime bounding the body.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	pub type SubShiftBody<'a, Narrow, Ans, V> =
		Rc<dyn Fn(SubShiftExit<Narrow, Ans, V>) -> Free<Narrow, Ans, RcBrand> + 'a>;

	/// The capture cell as the delimiter selects it from a program layer.
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The program the cell's continuation resumes into."
	)]
	pub type SubShiftCell<Narrow, Ans, V, P> = Coyoneda<'static, SubShiftBrand<Narrow, Ans, V>, P>;

	/// Brand for the multi-shot `SubShift` effect: delimited-continuation
	/// capture with a re-callable continuation, pinned per prompt.
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	pub struct SubShiftBrand<Narrow, Ans, V>(PhantomData<(Narrow, Ans, V)>);

	/// The operations of [`SubShiftBrand`]: one variant, the capture, with
	/// the continuation hole `A`. Both callable positions are in the `Rc`
	/// re-callable form.
	#[document_type_parameters(
		"The lifetime bounding the stored callables.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The continuation hole."
	)]
	pub enum SubShiftF<'a, Narrow, Ans, V, A>
	where
		Narrow: WrapDrop + 'static,
		Ans: 'static, {
		/// Capture: the body receives the reified re-callable continuation
		/// and produces the prompt's answer program; the operation resumes
		/// with `V` on each invocation of the continuation.
		SubShift(SubShiftBody<'a, Narrow, Ans, V>, Rc<dyn Fn(V) -> A + 'a>),
	}

	impl_kind! {
		impl<Narrow, Ans, V> for SubShiftBrand<Narrow, Ans, V>
		where
			Narrow: WrapDrop + 'static,
			Ans: 'static,
			V: 'static,
		{
			type Of<'a, A: 'a>: 'a = SubShiftF<'a, Narrow, Ans, V, A>;
		}
	}

	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	impl<Narrow: WrapDrop + 'static, Ans: 'static, V: 'static> Functor
		for SubShiftBrand<Narrow, Ans, V>
	{
		/// Composes the mapped function into the capture's re-callable
		/// continuation by re-wrapping; the body is untouched.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::CNilBrand,
		/// 		classes::Functor,
		/// 		types::effects::sub_shift::{
		/// 			SubShiftBrand,
		/// 			SubShiftF,
		/// 		},
		/// 	},
		/// 	std::rc::Rc,
		/// };
		///
		/// let op: SubShiftF<'static, CNilBrand, i32, i32, i32> =
		/// 	SubShiftF::SubShift(Rc::new(|exit| exit(5)), Rc::new(|v| v));
		/// let mapped = <SubShiftBrand<CNilBrand, i32, i32> as Functor>::map(|v: i32| v + 1, op);
		/// let SubShiftF::SubShift(_body, k) = mapped;
		/// // The composed continuation stays re-callable.
		/// assert_eq!(k(5), 6);
		/// assert_eq!(k(6), 7);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				SubShiftF::SubShift(body, k) =>
					SubShiftF::SubShift(body, Rc::new(move |v| f(k(v)))),
			}
		}
	}

	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	impl<Narrow, Ans, V> OrderOf for SubShiftBrand<Narrow, Ans, V> {
		type Order = FirstOrder;
	}

	/// Captures the continuation from here to the enclosing
	/// [`run_sub_shift`] as a re-callable value: the body receives it and
	/// produces the prompt's answer program over the residual row. Each
	/// invocation of the continuation resumes this operation with the
	/// passed value and yields a fresh completed answer, so the body may
	/// invoke it once per branch and combine the answers; dropping the
	/// continuation aborts the unresumed tail, structurally.
	///
	/// The body runs over the residual row, so it cannot capture at this
	/// same prompt again, while every other effect of the row remains
	/// available to it.
	#[document_signature]
	///
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The row brand the program runs over.",
		"The coproduct index locating the `SubShift` cell (inferred)."
	)]
	///
	#[document_parameters("The capture body, from the reified continuation to the answer program.")]
	///
	#[document_returns("The one-operation program suspending at the capture.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		CNilBrand,
	/// 		RcBrand,
	/// 	},
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			handle::multi_shot::extract,
	/// 			sub_shift::{
	/// 				SubShiftBrand,
	/// 				run_sub_shift,
	/// 				sub_shift,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell prompt delimiting straight to the empty row.
	/// 	pub row PromptRow {
	/// 		SubShiftBrand<CNilBrand, i32, i32>,
	/// 	}
	/// }
	///
	/// // The body invokes the continuation twice, once per branch, and
	/// // sums the two completed answers: the multi-shot moment.
	/// let program: Free<PromptRow, i32, RcBrand> = sub_shift::<_, _, i32, _, _>(|exit| {
	/// 	let again = exit.clone();
	/// 	exit(1).bind_multi_shot(move |first: i32| {
	/// 		again(2).bind_multi_shot(move |second: i32| Free::pure(first + second))
	/// 	})
	/// })
	/// .bind_multi_shot(|v: i32| Free::pure(v * 10));
	/// let narrowed: Free<CNilBrand, i32, RcBrand> = run_sub_shift(program, Free::pure);
	/// assert_eq!(extract(narrowed), 30);
	/// ```
	pub fn sub_shift<Narrow, Ans, V, R, I>(
		body: impl Fn(SubShiftExit<Narrow, Ans, V>) -> Free<Narrow, Ans, RcBrand> + 'static
	) -> Free<R, V, RcBrand>
	where
		Narrow: WrapDrop + 'static,
		Ans: 'static,
		V: ValueFor<RcBrand>,
		R: Functor + WrapDrop + 'static,
		<R as LifetimeUnaryKind>::Of<'static, V>:
			CoprodInjector<Coyoneda<'static, SubShiftBrand<Narrow, Ans, V>, V>, I>, {
		let cell: SubShiftF<'static, Narrow, Ans, V, V> =
			SubShiftF::SubShift(Rc::new(body), Rc::new(|x| x));
		let coyo: Coyoneda<'static, SubShiftBrand<Narrow, Ans, V>, V> = Coyoneda::lift(cell);
		let node: <R as LifetimeUnaryKind>::Of<'static, V> = CoprodInjector::inject(coyo);
		Free::lift_f(node)
	}

	/// The multi-shot delimiter: folds the program at its own result type,
	/// handing each `SubShift` cell's delimited re-callable continuation to
	/// its body, re-emitting every other effect into the residual row
	/// lazily, and mapping each normal completion through the return
	/// clause. Where the one-shot delimiter consumes its return clause
	/// exactly once, this one reaches normal completion once per re-entry,
	/// so the clause is re-callable and cloned into each path.
	///
	/// Native stack use grows with the number of captures resumed in one
	/// synchronous chain, because each resumed continuation re-enters the
	/// fold inside the caller's frame, matching the one-shot delimiter's
	/// documented trade-off.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The program's result type.",
		"The coproduct index locating the `SubShift` cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The program to delimit.",
		"The return clause, mapping a completion into the answer program; re-callable, invoked once per re-entry."
	)]
	///
	#[document_returns("The prompt's answer program over the residual row.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		CNilBrand,
	/// 		RcBrand,
	/// 	},
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			handle::multi_shot::extract,
	/// 			sub_shift::{
	/// 				SubShiftBrand,
	/// 				run_sub_shift,
	/// 				sub_shift,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell prompt collecting every branch's leaf.
	/// 	pub row PromptRow {
	/// 		SubShiftBrand<CNilBrand, Vec<i32>, bool>,
	/// 	}
	/// }
	///
	/// // The body forks the tail per branch and concatenates the answers;
	/// // the return clause wraps each leaf, once per re-entry.
	/// let program: Free<PromptRow, i32, RcBrand> = sub_shift::<_, _, bool, _, _>(|exit| {
	/// 	let other = exit.clone();
	/// 	exit(true).bind_multi_shot(move |kept: Vec<i32>| {
	/// 		other(false).bind_multi_shot(move |rest: Vec<i32>| {
	/// 			let mut all = kept.clone();
	/// 			all.extend(rest);
	/// 			Free::pure(all)
	/// 		})
	/// 	})
	/// })
	/// .bind_multi_shot(|kept: bool| Free::pure(if kept { 1 } else { 2 }));
	/// let narrowed: Free<CNilBrand, Vec<i32>, RcBrand> =
	/// 	run_sub_shift(program, |leaf| Free::pure(vec![leaf]));
	/// assert_eq!(extract(narrowed), vec![1, 2]);
	/// ```
	pub fn run_sub_shift<Row, Narrow, Ans, V, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A, RcBrand>,
		on_pure: impl Fn(A) -> Free<Narrow, Ans, RcBrand> + Clone + 'static,
	) -> Free<Narrow, Ans, RcBrand>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Ans: 'static,
		V: 'static,
		A: Clone + 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>:
			CoprodUninjector<SubShiftCell<Narrow, Ans, V, Free<Row, A, RcBrand>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>> as CoprodUninjector<
			SubShiftCell<Narrow, Ans, V, Free<Row, A, RcBrand>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>,
				EmbedIndices,
			>, {
		match program.to_view() {
			FreeStep::Done(value) => on_pure(value),
			FreeStep::Suspended(layer) => match layer.uninject() {
				Ok(coyo) => {
					let SubShiftF::SubShift(body, k) = coyo.lower();
					let exit: SubShiftExit<Narrow, Ans, V> =
						Rc::new(move |v| run_sub_shift(k(v), on_pure.clone()));
					body(exit)
				}
				Err(rest) => {
					let narrowed: <Narrow as LifetimeUnaryKind>::Of<
						'static,
						Free<Row, A, RcBrand>,
					> = rest.embed();
					// The fold rides the layer's deferred Coyoneda
					// composition; the closure is re-callable under an outer
					// fork, so the return clause is cloned per call.
					let re_emitted = Narrow::map(
						move |rest_program: Free<Row, A, RcBrand>| {
							run_sub_shift(rest_program, on_pure.clone())
						},
						narrowed,
					);
					Free::wrap(re_emitted)
				}
			},
		}
	}
}

pub use inner::*;
