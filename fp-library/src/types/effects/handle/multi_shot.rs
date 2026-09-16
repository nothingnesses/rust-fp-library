//! The multi-shot arm of the interpretation surface: the narrowing
//! accumulator runner and the terminal extractor for the `Rc`/`Arc`
//! stores, under the same names as their `Box`-store siblings in the
//! parent module (the module path is the store-axis marker, mirroring how
//! `Free::to_view` and `Free::evaluate` split into a `Box` arm and one
//! multi-shot arm).
//!
//! One body serves both multi-shot stores. Stepping goes through the
//! multi-shot `Free::to_view` arm, and the residual re-emission needs no
//! store-specific `bind`: the recursive fold is mapped into the unmatched
//! layer with the row's ordinary `Functor::map` (which composes lazily
//! into the layer's `Coyoneda` cell, so the recursion is deferred exactly
//! as the `Box` arm's `bind` defers it) and the layer is re-wrapped with
//! the store-generic `Free::wrap`. The accumulator and step are cloned
//! into each re-emission, which is why this arm carries `Clone` bounds
//! the `Box` arm does not.
//!
//! A row cell lowers exactly once per encounter, so this non-forking
//! runner is correct on the one-shot cells the effect macros emit today;
//! re-entering a cell (forking) is a separate capability with its own
//! cell requirements.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::CNilBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Coyoneda,
				Free,
				FreeStep,
				cat_queue::CatQueue,
				closure_storage::{
					ClosureStorage,
					MultiShotStore,
					ValueFor,
				},
				effects::coproduct::{
					CoprodUninjector,
					CoproductEmbedder,
				},
				free::Continuation,
			},
		},
		fp_macros::*,
	};

	/// Eliminates one effect brand from a row over a multi-shot store,
	/// threading an accumulator through the matched operations iteratively
	/// and re-emitting every unmatched layer into the residual row with the
	/// recursive fold deferred into the layer's cell.
	///
	/// The multi-shot sibling of the parent module's `handle_accum`: the
	/// walk steps through the multi-shot `Free::to_view` arm, and the
	/// accumulator and step are cloned into each re-emission (the `Clone`
	/// bounds the `Box` arm does not need), so stacked runners compose over
	/// the `Rc` and `Arc` stores exactly as they do over `Box`.
	#[document_signature]
	#[document_type_parameters(
		"The effect brand being eliminated.",
		"The source row brand.",
		"The residual row brand.",
		"The multi-shot closure store.",
		"The accumulator type.",
		"The program's result type.",
		"The coproduct index locating the eliminated cell (inferred).",
		"The embedding indices for the residual row (inferred)."
	)]
	#[document_parameters(
		"The initial accumulator value.",
		"The program over the source row.",
		"The step: from the accumulator and a matched operation to the next accumulator and the continuation program."
	)]
	#[document_returns(
		"The residual program over the narrowed row, completing with the final accumulator and the program's result."
	)]
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
	/// 			handle::multi_shot::{
	/// 				extract,
	/// 				handle_accum,
	/// 			},
	/// 			state::{
	/// 				StateBrand,
	/// 				StateF,
	/// 				get,
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
	/// let program: Free<StateRow, i32, RcBrand> =
	/// 	get::<i32, StateRow, _, RcBrand>().bind_multi_shot(|s: i32| Free::pure(s + 1));
	///
	/// let folded: Free<CNilBrand, (i32, i32), RcBrand> =
	/// 	handle_accum(5, program, |s: i32, op| match op {
	/// 		StateF::Get(k) => (s, k(s)),
	/// 		StateF::Put(next, k) => (next, k(())),
	/// 	});
	/// assert_eq!(extract(folded), (5, 6));
	/// ```
	pub fn handle_accum<EBrand, Row, Narrow, Store, S, A, UninjectIndex, EmbedIndices>(
		s: S,
		program: Free<Row, A, Store>,
		step: impl Fn(
			S,
			<EBrand as LifetimeUnaryKind>::Of<'static, Free<Row, A, Store>>,
		) -> (S, Free<Row, A, Store>)
		+ Clone
		+ 'static,
	) -> Free<Narrow, (S, A), Store>
	where
		EBrand: LifetimeUnaryKind + Functor + 'static,
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Store: MultiShotStore,
		S: Clone + 'static,
		A: ValueFor<Store>,
		(S, A): ValueFor<Store>,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, Store>>:
			CoprodUninjector<Coyoneda<'static, EBrand, Free<Row, A, Store>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, Store>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, A, Store>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A, Store>>,
				EmbedIndices,
			>,
		<Store as ClosureStorage>::Queue<Continuation<Row, Store>>:
			CatQueue<Continuation<Row, Store>> + Clone, {
		let mut s = s;
		let mut program = program;
		loop {
			match program.to_view() {
				FreeStep::Done(value) => return Free::pure((s, value)),
				FreeStep::Suspended(layer) => match layer.uninject() {
					Ok(op) => {
						let (next_s, next_program) = step(s, op.lower());
						s = next_s;
						program = next_program;
					}
					Err(rest) => {
						let narrowed: <Narrow as LifetimeUnaryKind>::Of<
							'static,
							Free<Row, A, Store>,
						> = rest.embed();
						// The recursive fold rides the layer's deferred
						// Coyoneda composition, so it runs only when a
						// downstream handler lowers this cell, mirroring
						// the Box arm's deferral into `bind`.
						let re_emitted = Narrow::map(
							move |rest_program: Free<Row, A, Store>| {
								handle_accum(s.clone(), rest_program, step.clone())
							},
							narrowed,
						);
						return Free::wrap(re_emitted);
					}
				},
			}
		}
	}

	/// Extracts the final value from a fully narrowed multi-shot program:
	/// the residual row is empty, so the program is necessarily a pure
	/// value.
	///
	/// The multi-shot sibling of the parent module's `extract`, closing a
	/// runner pipeline over the `Rc` or `Arc` store.
	#[document_signature]
	#[document_type_parameters("The program's result type.", "The multi-shot closure store.")]
	#[document_parameters("The fully narrowed program.")]
	#[document_returns("The program's final value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::{
	/// 		CNilBrand,
	/// 		RcBrand,
	/// 	},
	/// 	types::{
	/// 		Free,
	/// 		effects::handle::multi_shot::extract,
	/// 	},
	/// };
	///
	/// let program: Free<CNilBrand, i32, RcBrand> = Free::pure(7);
	/// assert_eq!(extract(program), 7);
	/// ```
	pub fn extract<A, Store>(program: Free<CNilBrand, A, Store>) -> A
	where
		Store: MultiShotStore,
		A: ValueFor<Store>,
		<Store as ClosureStorage>::Queue<Continuation<CNilBrand, Store>>:
			CatQueue<Continuation<CNilBrand, Store>> + Clone, {
		match program.to_view() {
			FreeStep::Done(value) => value,
			FreeStep::Suspended(layer) => match layer {},
		}
	}
}

pub use inner::*;
