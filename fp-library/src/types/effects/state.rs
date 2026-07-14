//! The `State` effect: a read/write cell of `S`, with the threaded narrowing
//! runner.
//!
//! The effect definition (brand, functor, order marker) and its smart
//! constructors are emitted by `fp_macros::define_effect!` from the operation
//! signatures below. [`handle_state`] eliminates the effect from a row by
//! threading the state value through interpretation, purescript-run's
//! `runState` shape: the final state is paired with the program's result, and
//! every other effect is re-emitted into the residual row, so where the
//! runner sits in a stack decides which effects scope the state.

fp_macros::define_effect! {
	/// State over a cell of `S`. `Get` reads the current state; `Put` writes it.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub effect State<S: 'static> {
		/// Read the current state.
		fn get() -> S;
		/// Write the state.
		fn put(value: S) -> ();
	}
}

#[fp_macros::document_module]
mod runner {
	use {
		super::{
			StateBrand,
			StateF,
			get,
			put,
		},
		crate::{
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::LifetimeUnaryKind,
			types::{
				Coyoneda,
				Free,
				effects::{
					coproduct::{
						CoprodInjector,
						CoprodUninjector,
						CoproductEmbedder,
					},
					handle::{
						AccumStep,
						handle_accum,
					},
				},
			},
		},
		fp_macros::*,
	};

	/// The `State` step: `get` resumes with the current accumulator, `put`
	/// replaces it. [`handle_state`] threads it sequentially; runners that
	/// fork interpretation (the scoped
	/// [`handle_choose_accum`](crate::types::effects::choose::handle_choose_accum))
	/// take it as their [`AccumStep`] to scope a state cell per branch.
	#[derive(Clone)]
	pub struct StateStep;

	#[document_type_parameters(
		"The row brand the interpreted programs run over.",
		"The state type."
	)]
	#[document_parameters("The step value.")]
	impl<Row: WrapDrop + 'static, S: Clone + 'static> AccumStep<StateBrand<S>, Row, S> for StateStep {
		/// Interprets one lowered `State` operation: `get` resumes with the
		/// current state, `put` replaces it.
		#[document_signature]
		///
		#[document_type_parameters("The program result type this application interprets at.")]
		///
		#[document_parameters("The current state.", "The lowered operation to interpret.")]
		///
		#[document_returns("The new state paired with the continuation program.")]
		#[document_examples(
			skip_call_check,
			reason = "A step is consumed by a forking runner rather than called directly; the example demonstrates this implementation's semantics by driving `handle_choose_accum` with it."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	define_row,
		/// 	types::{
		/// 		Free,
		/// 		effects::{
		/// 			choose::{
		/// 				ChooseBrand,
		/// 				choose,
		/// 				handle_choose_accum,
		/// 			},
		/// 			handle::extract,
		/// 			state::{
		/// 				StateBrand,
		/// 				StateStep,
		/// 				get,
		/// 				put,
		/// 			},
		/// 		},
		/// 	},
		/// };
		///
		/// define_row! {
		/// 	/// Integer choice alongside integer state.
		/// 	pub row ChoiceStateRow {
		/// 		ChooseBrand<ChoiceStateRow, i32>,
		/// 		StateBrand<i32>,
		/// 	}
		/// }
		///
		/// // The forking runner applies the step per branch: the left
		/// // branch's write is branch-local, so the right branch reads its
		/// // own untouched fork of the initial state.
		/// let program: Free<ChoiceStateRow, Vec<i32>> =
		/// 	choose(put(10).bind(|()| get()), get()).bind(|values: Vec<i32>| Free::pure(values));
		/// let narrowed: Free<CNilBrand, (i32, Option<Vec<i32>>)> =
		/// 	handle_choose_accum(1, program, StateStep);
		/// assert_eq!(extract(narrowed), (1, Some(vec![10, 1])));
		/// ```
		fn step<T: 'static>(
			&self,
			s: S,
			op: <StateBrand<S> as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		) -> (S, Free<Row, T>) {
			match op {
				StateF::Get(resume) => {
					let current = s.clone();
					(s, resume(current))
				}
				StateF::Put(next, resume) => (next, resume(())),
			}
		}
	}

	/// Eliminates the `State` effect from a row by threading the state value
	/// through interpretation: each `get` resumes with the current value,
	/// each `put` replaces it, and the final state is paired with the
	/// program's result. Every other effect is re-emitted into the residual
	/// row, so stacking this runner inside or outside another decides which
	/// effects scope the state.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The state type.",
		"The program's result type.",
		"The coproduct index locating the state cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The initial state.", "The program to interpret.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the final state paired with the program's result."
	)]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			handle::extract,
	/// 			state::{
	/// 				StateBrand,
	/// 				get,
	/// 				handle_state,
	/// 				put,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row holding integer state.
	/// 	pub row StateRow {
	/// 		StateBrand<i32>,
	/// 	}
	/// }
	///
	/// let program: Free<StateRow, i32> = put(4).bind(|()| get());
	/// let narrowed: Free<CNilBrand, (i32, i32)> = handle_state(0, program);
	/// assert_eq!(extract(narrowed), (4, 4));
	/// ```
	pub fn handle_state<Row, Narrow, S, A, UninjectIndex, EmbedIndices>(
		initial: S,
		program: Free<Row, A>,
	) -> Free<Narrow, (S, A)>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		S: Clone + 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, StateBrand<S>, Free<Row, A>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, StateBrand<S>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>, {
		handle_accum::<StateBrand<S>, _, _, _, _, _, _>(initial, program, |s, op| {
			StateStep.step(s, op)
		})
	}

	/// Runs `action` transactionally with respect to the row's `State` cell:
	/// the outer state is snapshotted with one `get`, the action's `State`
	/// operations thread a local accumulator seeded from that snapshot while
	/// every other effect re-emits unchanged, and the final local value
	/// commits with one outer `put` only when the action completes. Rollback
	/// on abort is structural rather than guarded: the commit `put` lives in
	/// the continuation an abort discards, so the outer state never sees an
	/// aborted action's writes, under the one-pass handler surface and under
	/// stacked narrowing runners alike.
	///
	/// The state payload cannot be inferred from the action alone, so name it
	/// in the turbofish (all-or-`_`):
	/// `transact_state::<_, S, _, _, _, _, _>(action)`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The row brand the action runs over.",
		"The state payload the transaction scopes.",
		"The action's result type.",
		"The coproduct index locating the state cell at the snapshot's value type (inferred).",
		"The coproduct index locating the state cell at the commit's value type (inferred).",
		"The coproduct index locating the state cell inside the action's layers (inferred).",
		"The coproduct indices embedding the non-state remainder back into the row (inferred)."
	)]
	///
	#[document_parameters("The action to run transactionally.")]
	///
	#[document_returns("The transacted program over the same row, yielding the action's result.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			handle::extract,
	/// 			state::{
	/// 				StateBrand,
	/// 				get,
	/// 				handle_state,
	/// 				put,
	/// 				transact_state,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row holding integer state.
	/// 	pub row StateRow {
	/// 		StateBrand<i32>,
	/// 	}
	/// }
	///
	/// // The action's writes thread locally and commit on completion.
	/// let program: Free<StateRow, i32> =
	/// 	transact_state::<_, i32, _, _, _, _, _>(put(4).bind(|()| get()));
	/// let narrowed: Free<CNilBrand, (i32, i32)> = handle_state(0, program);
	/// assert_eq!(extract(narrowed), (4, 4));
	/// ```
	pub fn transact_state<Row, S, A, GetIndex, PutIndex, UninjectIndex, EmbedIndices>(
		action: Free<Row, A>
	) -> Free<Row, A>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		S: Clone + 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, S>:
			CoprodInjector<Coyoneda<'static, StateBrand<S>, S>, GetIndex>,
		<Row as LifetimeUnaryKind>::Of<'static, ()>:
			CoprodInjector<Coyoneda<'static, StateBrand<S>, ()>, PutIndex>,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, StateBrand<S>, Free<Row, A>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, StateBrand<S>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder:
			CoproductEmbedder<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>, EmbedIndices>, {
		get::<S, Row, GetIndex>().bind(move |pre| {
			handle_accum::<StateBrand<S>, Row, Row, S, A, UninjectIndex, EmbedIndices>(
				pre,
				action,
				|s, op| StateStep.step(s, op),
			)
			.bind(|(post, a)| put::<S, Row, PutIndex>(post).bind(move |()| Free::pure(a)))
		})
	}
}

pub use runner::*;

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		get,
		put,
		run,
	};

	// Behaviour-parity oracle bucket A (single-effect): a `put` then `get` reads
	// back the written value and leaves the state cell holding it.
	#[test]
	fn put_then_get_reads_the_written_state() {
		let program = put(true).bind(|()| get());
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(true));
		assert!(fx.state.get());
	}
}
