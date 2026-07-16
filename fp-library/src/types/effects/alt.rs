//! The `Alt` effect: first-order nondeterministic choice with a re-callable
//! continuation, and its forking narrowing runners on the `Rc` store.
//!
//! The effect definition (brand, functor, order marker) and its smart
//! constructors are emitted by `fp_macros::define_effect!` from the operation
//! signatures below. `alt` is a `#[multi_shot]` operation: its continuation
//! is stored re-callably (`Rc<dyn Fn>`), so a forking runner lowers the cell
//! once and calls the captured continuation once per branch (`true` first);
//! `empty` kills the current branch without a value. This is the first-order
//! counterpart of the scoped [`Choose`](crate::types::effects::choose)
//! effect (purescript-run's `Choose` functor, `Alt` plus `Empty`): where the
//! scoped cell owns its branch sub-programs and resumes once with the
//! survivors, `alt` distributes the program's own continuation per branch.
//! No elaboration bridges the two cells: the scoped cell's `Box`-store
//! owned branches and collection-consuming continuation cannot be
//! re-expressed through an emitted `alt` (a forking runner distributes
//! what that cell must resume exactly once), so each tier keeps its own
//! runner family.
//!
//! The runners are pinned to the `Rc` store because the `#[multi_shot]`
//! emission stores continuations in the `Rc` form; the `Arc` tier joins when
//! a `Send` consumer brings the `SendFunctor` composition route. Which
//! effects a choice scopes is runner stacking, not a runner variant: an
//! accumulator runner stacked outside [`handle_alt`] threads one accumulator
//! through the branches in depth-first order (global semantics), while the
//! multi-shot `handle_accum` stacked under it rides the re-emitted `alt`
//! cell and is cloned into each branch at the fork (branch-local semantics).

fp_macros::define_effect! {
	/// First-order nondeterministic choice. `Alt` resumes re-callably, once
	/// per branch under a forking runner (`true` first); `Empty` kills the
	/// current branch without a value.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub effect Alt {
		/// Choose a branch: a forking runner calls the re-callable
		/// continuation once per branch, `true` first.
		#[multi_shot]
		fn alt() -> bool;
		/// Kill the current branch without producing a value.
		fn empty() -> !;
	}
}

#[fp_macros::document_module]
mod runner {
	use {
		super::{
			AltBrand,
			AltF,
		},
		crate::{
			brands::RcBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::LifetimeUnaryKind,
			types::{
				Coyoneda,
				Free,
				FreeStep,
				effects::coproduct::{
					CoprodUninjector,
					CoproductEmbedder,
				},
			},
		},
		fp_macros::*,
		std::rc::Rc,
	};

	/// The row's alt cell over programs yielding `A`, as selected by the
	/// runners' brand-keyed `uninject`.
	type AltCell<Row, A> = Coyoneda<'static, AltBrand, Free<Row, A, RcBrand>>;

	/// A lowered `alt` continuation: re-callable, producing one fresh
	/// program per call. Pending branches are held as continuations paired
	/// with the branch value still to take, never as owned programs
	/// (`Free` is not `Clone`, while `Rc` continuations clone freely into
	/// the re-emission closure).
	type PendingBranch<Row, A> = (Rc<dyn Fn(bool) -> Free<Row, A, RcBrand>>, bool);

	/// Eliminates the `Alt` cell from a row by forking at each `alt`: the
	/// captured continuation is called once per branch (`true` first) and
	/// every surviving leaf is collected in depth-first order; `empty` kills
	/// its branch, contributing nothing. Every other effect is re-emitted
	/// into the residual row in the order the branches run, so an
	/// accumulator runner stacked outside threads one accumulator through
	/// all branches (the global-across-branches ordering); for branch-local
	/// accumulators stack the multi-shot `handle_accum` under this runner
	/// instead, so its fold rides the re-emitted cell and forks per branch.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The program's result type.",
		"The coproduct index locating the alt cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The program to interpret.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding every surviving leaf in depth-first branch order (empty when every branch died)."
	)]
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
	/// 			alt::{
	/// 				AltBrand,
	/// 				alt,
	/// 				empty,
	/// 				handle_alt,
	/// 			},
	/// 			handle::multi_shot::extract,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row of first-order choice.
	/// 	pub row AltRow {
	/// 		AltBrand,
	/// 	}
	/// }
	///
	/// // The false branch dies; the true branch's leaf is the sole survivor.
	/// let program: Free<AltRow, i32, RcBrand> =
	/// 	alt::<AltRow, _, RcBrand>().bind_multi_shot(|kept: bool| {
	/// 		if kept { Free::pure(1) } else { empty::<i32, AltRow, _, RcBrand>() }
	/// 	});
	/// let collected: Free<CNilBrand, Vec<i32>, RcBrand> = handle_alt(program);
	/// assert_eq!(extract(collected), vec![1]);
	/// ```
	pub fn handle_alt<Row, Narrow, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A, RcBrand>
	) -> Free<Narrow, Vec<A>, RcBrand>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: Clone + 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>:
			CoprodUninjector<AltCell<Row, A>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>> as CoprodUninjector<
			AltCell<Row, A>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>,
				EmbedIndices,
			>, {
		collect_leaves(program, Vec::new(), Vec::new())
	}

	/// The depth-first work loop behind [`handle_alt`]: runs the current
	/// program, forking at each `alt` cell by pushing the false branch onto
	/// the pending stack and descending into the true branch, and appending
	/// each completed leaf to the collection. An unmatched layer is
	/// re-emitted into the residual row with the loop mapped into the cell,
	/// cloning the pending continuations and the collected leaves per
	/// re-entry (the closure is re-callable under forking).
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The program's result type.",
		"The coproduct index locating the alt cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The program currently being run.",
		"The branches not yet taken, innermost last.",
		"The leaves collected so far, in depth-first order."
	)]
	///
	#[document_returns(
		"The residual program yielding every leaf collected across the current program and the pending branches."
	)]
	///
	#[document_examples(
		skip_call_check,
		reason = "Direct-call validation is skipped because collect_leaves is the private loop behind handle_alt; the public runner reaches it on every interpretation, as the example demonstrates."
	)]
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
	/// 			alt::{
	/// 				AltBrand,
	/// 				alt,
	/// 				handle_alt,
	/// 			},
	/// 			handle::multi_shot::extract,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row of first-order choice.
	/// 	pub row AltRow {
	/// 		AltBrand,
	/// 	}
	/// }
	///
	/// // Two stacked choices fork into four leaves, depth-first.
	/// let program: Free<AltRow, (bool, bool), RcBrand> =
	/// 	alt::<AltRow, _, RcBrand>().bind_multi_shot(|first: bool| {
	/// 		alt::<AltRow, _, RcBrand>()
	/// 			.bind_multi_shot(move |second: bool| Free::pure((first, second)))
	/// 	});
	/// let collected: Free<CNilBrand, Vec<(bool, bool)>, RcBrand> = handle_alt(program);
	/// assert_eq!(
	/// 	extract(collected),
	/// 	vec![(true, true), (true, false), (false, true), (false, false)]
	/// );
	/// ```
	fn collect_leaves<Row, Narrow, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A, RcBrand>,
		pending: Vec<PendingBranch<Row, A>>,
		collected: Vec<A>,
	) -> Free<Narrow, Vec<A>, RcBrand>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: Clone + 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>:
			CoprodUninjector<AltCell<Row, A>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>> as CoprodUninjector<
			AltCell<Row, A>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>,
				EmbedIndices,
			>, {
		let mut current = program;
		let mut pending = pending;
		let mut collected = collected;
		loop {
			match current.to_view() {
				FreeStep::Done(leaf) => {
					collected.push(leaf);
					match pending.pop() {
						Some((k, branch)) => current = k(branch),
						None => return Free::pure(collected),
					}
				}
				FreeStep::Suspended(layer) => match layer.uninject() {
					Ok(cell) => match cell.lower() {
						AltF::Alt(k) => {
							pending.push((k.clone(), false));
							current = k(true);
						}
						AltF::Empty(_) => match pending.pop() {
							Some((k, branch)) => current = k(branch),
							None => return Free::pure(collected),
						},
					},
					Err(rest) => {
						let narrowed: <Narrow as LifetimeUnaryKind>::Of<
							'static,
							Free<Row, A, RcBrand>,
						> = rest.embed();
						// The loop rides the layer's deferred Coyoneda
						// composition; the closure is re-callable under an
						// outer fork, so the loop state is cloned per call.
						let re_emitted = Narrow::map(
							move |rest_program: Free<Row, A, RcBrand>| {
								collect_leaves(rest_program, pending.clone(), collected.clone())
							},
							narrowed,
						);
						return Free::wrap(re_emitted);
					}
				},
			}
		}
	}

	/// Eliminates the `Alt` cell from a row by forking at each `alt` and
	/// returning the first surviving leaf in depth-first order: the right
	/// branch of a choice runs only when every leaf of the left branch died,
	/// and the pending branches are dropped unrun as soon as a leaf
	/// completes (their effects never reach the residual). Every other
	/// effect is re-emitted into the residual row as in [`handle_alt`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The program's result type.",
		"The coproduct index locating the alt cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The program to interpret.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the first surviving leaf, or `None` when every branch died."
	)]
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
	/// 			alt::{
	/// 				AltBrand,
	/// 				alt,
	/// 				empty,
	/// 				handle_alt_first,
	/// 			},
	/// 			handle::multi_shot::extract,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row of first-order choice.
	/// 	pub row AltRow {
	/// 		AltBrand,
	/// 	}
	/// }
	///
	/// // The true branch dies, so the false branch supplies the value.
	/// let program: Free<AltRow, i32, RcBrand> =
	/// 	alt::<AltRow, _, RcBrand>().bind_multi_shot(|kept: bool| {
	/// 		if kept { empty::<i32, AltRow, _, RcBrand>() } else { Free::pure(2) }
	/// 	});
	/// let first: Free<CNilBrand, Option<i32>, RcBrand> = handle_alt_first(program);
	/// assert_eq!(extract(first), Some(2));
	/// ```
	pub fn handle_alt_first<Row, Narrow, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A, RcBrand>
	) -> Free<Narrow, Option<A>, RcBrand>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: Clone + 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>:
			CoprodUninjector<AltCell<Row, A>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>> as CoprodUninjector<
			AltCell<Row, A>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>,
				EmbedIndices,
			>, {
		first_leaf(program, Vec::new())
	}

	/// The depth-first work loop behind [`handle_alt_first`]: runs the
	/// current program, forking at each `alt` cell as [`handle_alt`]'s loop
	/// does, but returns at the first completed leaf, dropping the pending
	/// branches unrun; `empty` falls back to the most recent pending branch.
	/// An unmatched layer is re-emitted with the loop mapped into the cell,
	/// cloning the pending continuations per re-entry.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The program's result type.",
		"The coproduct index locating the alt cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The program currently being run.",
		"The branches not yet taken, innermost last."
	)]
	///
	#[document_returns(
		"The residual program yielding the first leaf to complete, or `None` when the current program and every pending branch died."
	)]
	///
	#[document_examples(
		skip_call_check,
		reason = "Direct-call validation is skipped because first_leaf is the private loop behind handle_alt_first; the public runner reaches it on every interpretation, as the example demonstrates."
	)]
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
	/// 			alt::{
	/// 				AltBrand,
	/// 				alt,
	/// 				handle_alt_first,
	/// 			},
	/// 			handle::multi_shot::extract,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row of first-order choice.
	/// 	pub row AltRow {
	/// 		AltBrand,
	/// 	}
	/// }
	///
	/// // The true branch completes first; the false branch never runs.
	/// let program: Free<AltRow, i32, RcBrand> = alt::<AltRow, _, RcBrand>()
	/// 	.bind_multi_shot(|kept: bool| Free::pure(if kept { 1 } else { 2 }));
	/// let first: Free<CNilBrand, Option<i32>, RcBrand> = handle_alt_first(program);
	/// assert_eq!(extract(first), Some(1));
	/// ```
	fn first_leaf<Row, Narrow, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A, RcBrand>,
		pending: Vec<PendingBranch<Row, A>>,
	) -> Free<Narrow, Option<A>, RcBrand>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: Clone + 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>:
			CoprodUninjector<AltCell<Row, A>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>> as CoprodUninjector<
			AltCell<Row, A>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A, RcBrand>>,
				EmbedIndices,
			>, {
		let mut current = program;
		let mut pending = pending;
		loop {
			match current.to_view() {
				FreeStep::Done(leaf) => return Free::pure(Some(leaf)),
				FreeStep::Suspended(layer) => match layer.uninject() {
					Ok(cell) => match cell.lower() {
						AltF::Alt(k) => {
							pending.push((k.clone(), false));
							current = k(true);
						}
						AltF::Empty(_) => match pending.pop() {
							Some((k, branch)) => current = k(branch),
							None => return Free::pure(None),
						},
					},
					Err(rest) => {
						let narrowed: <Narrow as LifetimeUnaryKind>::Of<
							'static,
							Free<Row, A, RcBrand>,
						> = rest.embed();
						// The loop rides the layer's deferred Coyoneda
						// composition; the closure is re-callable under an
						// outer fork, so the pending stack is cloned per
						// call.
						let re_emitted = Narrow::map(
							move |rest_program: Free<Row, A, RcBrand>| {
								first_leaf(rest_program, pending.clone())
							},
							narrowed,
						);
						return Free::wrap(re_emitted);
					}
				},
			}
		}
	}
}

pub use runner::*;
