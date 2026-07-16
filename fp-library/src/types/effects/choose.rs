//! The `Choose` effect: scoped nondeterministic choice with owned branches,
//! with the collecting and the accumulator-forking narrowing runners.
//!
//! The effect definition (brand, functor, order marker) and its smart
//! constructors are emitted by `fp_macros::define_effect!` from the operation
//! signatures below. `choose` owns its two branch sub-programs and resumes
//! exactly once, with the values of the branches that survived, so the cell
//! is single-shot by construction (the Box store's continuations are
//! `FnOnce`); per-branch continuation distribution is expressed program-side,
//! by placing the continuation inside the owned branches, and running one
//! continuation once per branch is the multi-shot substrate's re-expression
//! of this cell. `empty` kills the current branch without a value.
//!
//! Which effects a choice scopes is a runner choice: [`handle_choose_accum`]
//! forks its accumulator effect per branch (branch-local semantics), while
//! [`handle_choose`] with the accumulator's runner stacked outside threads
//! one accumulator through all branches in sequence (global semantics). Even
//! which branches run is a runner choice: [`handle_choose_first`] reads the
//! same cell as an or-else fallback, running the right branch only when the
//! left dies. Folding the surviving values is program-side under this cell:
//! the continuation owns the `Vec` of survivors, so a monoid fold over it is
//! ordinary code rather than a separate runner.

fp_macros::define_effect! {
	/// Scoped nondeterministic choice over branch values of `RAction`.
	/// `Choose` owns its two branch sub-programs and resumes exactly once
	/// with the values of the branches that survived; `Empty` kills the
	/// current branch without a value.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub effect Choose<RAction: 'static> {
		/// Run both owned branches once each, resuming once with the
		/// surviving branch values in branch order.
		fn choose(left: Program<RAction>, right: Program<RAction>) -> Vec<RAction>;
		/// Kill the current branch without producing a value.
		fn empty() -> !;
	}
}

#[fp_macros::document_module]
mod runner {
	use {
		super::{
			ChooseBrand,
			ChooseF,
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
					handle::AccumStep,
				},
			},
		},
		fp_macros::*,
	};

	/// The row's choice cell over programs yielding `T`, as selected by the
	/// runners' brand-keyed `uninject`.
	type ChooseCell<Row, RAction, T> = Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, T>>;

	/// Eliminates the `Choose` cell from a row by running each owned branch
	/// once through recursive self-application and resuming the continuation
	/// exactly once with the surviving branch values: a branch yields at most
	/// one value, `empty` is the branch death, and the top level is itself a
	/// branch, so the result is an `Option`. Every other effect is re-emitted
	/// into the residual row, so an accumulator runner stacked outside
	/// threads one accumulator through all branches in sequence (the
	/// global-across-branches ordering); for branch-local accumulators use
	/// [`handle_choose_accum`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The branch value type the row's choice cell is pinned at.",
		"The program's result type.",
		"The coproduct index locating the choice cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The program to interpret.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the program's result, or `None` when the program itself died as a branch."
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
	/// 			choose::{
	/// 				ChooseBrand,
	/// 				choose,
	/// 				empty,
	/// 				handle_choose,
	/// 			},
	/// 			handle::extract,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row choosing integers.
	/// 	pub row ChoiceRow {
	/// 		ChooseBrand<ChoiceRow, i32>,
	/// 	}
	/// }
	///
	/// // The dead right branch contributes nothing; the survivors are [1].
	/// let program: Free<ChoiceRow, i32> = choose(Free::pure(1), empty::<_, i32, _, _, _>())
	/// 	.bind(|values: Vec<i32>| Free::pure(values.iter().sum()));
	/// let narrowed: Free<CNilBrand, Option<i32>> = handle_choose(program);
	/// assert_eq!(extract(narrowed), Some(1));
	/// ```
	pub fn handle_choose<Row, Narrow, RAction, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A>
	) -> Free<Narrow, Option<A>>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		RAction: 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>: CoprodUninjector<
				Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, A>>,
				UninjectIndex,
			>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>>: CoprodUninjector<
				Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, RAction>>,
				UninjectIndex,
			>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>> as CoprodUninjector<
			Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, RAction>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>>,
				EmbedIndices,
			>, {
		let layer = match program.resume() {
			Ok(value) => return Free::pure(Some(value)),
			Err(layer) => layer,
		};
		let selected: Result<ChooseCell<Row, RAction, A>, _> = layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				ChooseF::Choose {
					left,
					right,
					k,
				} => {
					let left_run: Free<Narrow, Option<RAction>> = handle_choose(left);
					left_run.bind(move |left_value| {
						let right_run: Free<Narrow, Option<RAction>> = handle_choose(right);
						right_run.bind(move |right_value| {
							let survivors: Vec<RAction> =
								left_value.into_iter().chain(right_value).collect();
							handle_choose(k(survivors))
						})
					})
				}
				ChooseF::Empty(_) => Free::pure(None),
			},
			Err(rest) => {
				let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
					rest.embed();
				// The Box-store `bind` arm must be selected explicitly:
				// `bind` is defined per store, and in generic position the
				// method call is ambiguous until the receiver's store is
				// pinned.
				let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
				lifted.bind(move |rest_program| handle_choose(rest_program))
			}
		}
	}

	/// Eliminates the `Choose` cell with first-success semantics: the left
	/// branch runs first, and only when it dies does the right branch run at
	/// all, so a surviving left branch drops the right branch unrun (none of
	/// its effects reach the residual row). The continuation receives the
	/// first survivor as a singleton, or the empty `Vec` when both branches
	/// die; `empty` outside any choice is still the branch death of the
	/// program itself. This is the `or-else` fallback reading of the same
	/// cell [`handle_choose`] reads as collection: which branches run is
	/// decided by the runner, not the program.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The branch value type the row's choice cell is pinned at.",
		"The program's result type.",
		"The coproduct index locating the choice cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The program to interpret.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the program's result, or `None` when the program itself died as a branch."
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
	/// 			choose::{
	/// 				ChooseBrand,
	/// 				choose,
	/// 				empty,
	/// 				handle_choose_first,
	/// 			},
	/// 			handle::extract,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row choosing integers.
	/// 	pub row ChoiceRow {
	/// 		ChooseBrand<ChoiceRow, i32>,
	/// 	}
	/// }
	///
	/// // The left branch dies, so the right branch supplies the fallback.
	/// let program: Free<ChoiceRow, i32> = choose(empty::<_, i32, _, _, _>(), Free::pure(2))
	/// 	.bind(|survivors: Vec<i32>| Free::pure(survivors.iter().sum()));
	/// let narrowed: Free<CNilBrand, Option<i32>> = handle_choose_first(program);
	/// assert_eq!(extract(narrowed), Some(2));
	/// ```
	pub fn handle_choose_first<Row, Narrow, RAction, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A>
	) -> Free<Narrow, Option<A>>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		RAction: 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>: CoprodUninjector<
				Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, A>>,
				UninjectIndex,
			>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>>: CoprodUninjector<
				Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, RAction>>,
				UninjectIndex,
			>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>> as CoprodUninjector<
			Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, RAction>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>>,
				EmbedIndices,
			>, {
		let layer = match program.resume() {
			Ok(value) => return Free::pure(Some(value)),
			Err(layer) => layer,
		};
		let selected: Result<ChooseCell<Row, RAction, A>, _> = layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				ChooseF::Choose {
					left,
					right,
					k,
				} => {
					let left_run: Free<Narrow, Option<RAction>> = handle_choose_first(left);
					left_run.bind(move |left_value| match left_value {
						Some(value) => handle_choose_first(k(vec![value])),
						None => {
							let right_run: Free<Narrow, Option<RAction>> =
								handle_choose_first(right);
							right_run.bind(move |right_value| {
								handle_choose_first(k(right_value.into_iter().collect()))
							})
						}
					})
				}
				ChooseF::Empty(_) => Free::pure(None),
			},
			Err(rest) => {
				let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
					rest.embed();
				// The Box-store `bind` arm must be selected explicitly:
				// `bind` is defined per store, and in generic position the
				// method call is ambiguous until the receiver's store is
				// pinned.
				let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
				lifted.bind(move |rest_program| handle_choose_first(rest_program))
			}
		}
	}

	/// Eliminates an accumulator effect and the `Choose` cell together: the
	/// matched accumulator effect threads the accumulator through the step,
	/// a choice runs each owned branch once with its own clone of the
	/// accumulator (branch-final accumulators die with their branches and
	/// the trunk continues with the original, so the accumulator is
	/// branch-local), and `empty` is the branch death. This is the one place
	/// the accumulator needs `Clone`; sequential runners move it instead.
	#[document_signature]
	///
	#[document_type_parameters(
		"The eliminated accumulator effect brand (inferred through the step).",
		"The source row brand.",
		"The residual row brand.",
		"The accumulator type.",
		"The branch value type the row's choice cell is pinned at.",
		"The program's result type.",
		"The coproduct index locating the accumulator cell (inferred).",
		"The coproduct index locating the choice cell in the remainder (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The initial accumulator.",
		"The program to interpret.",
		"The step applied to each matched accumulator operation."
	)]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the trunk's final accumulator paired with the program's result, or `None` when the program itself died as a branch."
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
	/// // The left branch writes 10 and reads it back; the right branch reads
	/// // its own untouched fork of the initial state; the trunk continues
	/// // with the original accumulator.
	/// let program: Free<ChoiceStateRow, Vec<i32>> =
	/// 	choose(put(10).bind(|()| get()), get()).bind(|values: Vec<i32>| Free::pure(values));
	/// let narrowed: Free<CNilBrand, (i32, Option<Vec<i32>>)> =
	/// 	handle_choose_accum(1, program, StateStep);
	/// assert_eq!(extract(narrowed), (1, Some(vec![10, 1])));
	/// ```
	pub fn handle_choose_accum<
		EBrand,
		Row,
		Narrow,
		S,
		RAction,
		A,
		UninjectE,
		UninjectC,
		EmbedIndices,
	>(
		s: S,
		program: Free<Row, A>,
		step: impl AccumStep<EBrand, Row, S>,
	) -> Free<Narrow, (S, Option<A>)>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		EBrand: LifetimeUnaryKind + Functor + 'static,
		S: Clone + 'static,
		RAction: 'static,
		A: 'static,
		UninjectE: 'static,
		UninjectC: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, EBrand, Free<Row, A>>, UninjectE>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, A>>,
			UninjectE,
		>>::Remainder:
			CoprodUninjector<Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, A>>, UninjectC>,
		<<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, A>>,
			UninjectE,
		>>::Remainder as CoprodUninjector<
			Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, A>>,
			UninjectC,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>>:
			CoprodUninjector<Coyoneda<'static, EBrand, Free<Row, RAction>>, UninjectE>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, RAction>>,
			UninjectE,
		>>::Remainder: CoprodUninjector<
				Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, RAction>>,
				UninjectC,
			>,
		<<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, RAction>>,
			UninjectE,
		>>::Remainder as CoprodUninjector<
			Coyoneda<'static, ChooseBrand<Row, RAction>, Free<Row, RAction>>,
			UninjectC,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, RAction>>,
				EmbedIndices,
			>, {
		let mut s = s;
		let mut program = program;
		loop {
			let layer = match program.resume() {
				Ok(value) => return Free::pure((s, Some(value))),
				Err(layer) => layer,
			};
			let matched: Result<Coyoneda<'static, EBrand, Free<Row, A>>, _> = layer.uninject();
			let rest = match matched {
				Ok(op) => {
					let (next_s, next_program) = step.step(s, op.lower());
					s = next_s;
					program = next_program;
					continue;
				}
				Err(rest) => rest,
			};
			let chosen: Result<ChooseCell<Row, RAction, A>, _> = rest.uninject();
			match chosen {
				Ok(op) => match op.lower() {
					ChooseF::Choose {
						left,
						right,
						k,
					} => {
						let left_run: Free<Narrow, (S, Option<RAction>)> =
							handle_choose_accum(s.clone(), left, step.clone());
						return left_run.bind(move |(_, left_value)| {
							let right_run: Free<Narrow, (S, Option<RAction>)> =
								handle_choose_accum(s.clone(), right, step.clone());
							right_run.bind(move |(_, right_value)| {
								let survivors: Vec<RAction> =
									left_value.into_iter().chain(right_value).collect();
								handle_choose_accum(s, k(survivors), step)
							})
						});
					}
					ChooseF::Empty(_) => return Free::pure((s, None)),
				},
				Err(rest) => {
					let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
						rest.embed();
					// The Box-store `bind` arm must be selected explicitly:
					// `bind` is defined per store, and in generic position
					// the method call is ambiguous until the receiver's
					// store is pinned.
					let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
					return lifted
						.bind(move |rest_program| handle_choose_accum(s, rest_program, step));
				}
			}
		}
	}
}

pub use runner::*;
