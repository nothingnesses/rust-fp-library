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
