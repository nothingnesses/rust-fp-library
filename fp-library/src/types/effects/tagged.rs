//! Tagged (labelled) effects: a [`TaggedBrand`] wrapper whose identity
//! changes the dispatch key while its kind projection reuses the effect's
//! own operations enum, so the same effect appears in one row once per
//! label and every cell dispatches independently.
//!
//! A label is any user-defined zero-sized type. The wrapper delegates
//! everything to the bare effect: the operations enum, the `Functor`, the
//! order marker, the abort contribution, and the handler pieces, so a
//! tagged cell interprets with the bare effect's arms and steps. A step
//! written for the bare effect serves every label through the
//! [`tag_step`] adapter, a nominal wrapper rather than a blanket
//! delegation, so a bare step passed to a runner keeps inferring the bare
//! brand.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Free,
				effects::{
					handle::{
						AccumStep,
						EffectAbort,
						HandlerPieces,
						RowHandler,
					},
					order::OrderOf,
				},
			},
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// A label brand over an effect brand: the wrapper's identity is the
	/// dispatch key, so the same effect appears in one row once per label,
	/// while the projection, arms, and abort are the bare effect's own.
	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The bare effect brand the label wraps."
	)]
	pub struct TaggedBrand<Label, EBrand>(PhantomData<(Label, EBrand)>);

	impl_kind! {
		impl<Label, EBrand> for TaggedBrand<Label, EBrand>
		where
			Label: 'static,
			EBrand: LifetimeUnaryKind,
		{
			type Of<'a, A: 'a>: 'a = <EBrand as LifetimeUnaryKind>::Of<'a, A>;
		}
	}

	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The bare effect brand the label wraps."
	)]
	impl<Label: 'static, EBrand: Functor + LifetimeUnaryKind> Functor for TaggedBrand<Label, EBrand> {
		/// Delegates to the bare effect's `map`: the tagged projection is
		/// the bare projection, so the delegation is an identity.
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
		/// use fp_library::{
		/// 	classes::Functor,
		/// 	types::effects::{
		/// 		state::{
		/// 			StateBrand,
		/// 			StateF,
		/// 		},
		/// 		tagged::TaggedBrand,
		/// 	},
		/// };
		///
		/// /// The example's label.
		/// pub struct Purse;
		///
		/// let op: StateF<'static, i32, i32> = StateF::Get(Box::new(|x| x));
		/// let mapped = <TaggedBrand<Purse, StateBrand<i32>> as Functor>::map(|x: i32| x + 1, op);
		/// match mapped {
		/// 	StateF::Get(resume) => assert_eq!(resume(4), 5),
		/// 	StateF::Put(..) => {}
		/// }
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			<EBrand as Functor>::map(f, fa)
		}
	}

	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The bare effect brand the label wraps."
	)]
	impl<Label, EBrand: OrderOf> OrderOf for TaggedBrand<Label, EBrand> {
		type Order = <EBrand as OrderOf>::Order;
	}

	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The bare effect brand the label wraps."
	)]
	impl<Label, EBrand: EffectAbort> EffectAbort for TaggedBrand<Label, EBrand> {
		type Abort = <EBrand as EffectAbort>::Abort;
	}

	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The bare effect brand the label wraps.",
		"The row brand the handled programs run over.",
		"The row's abort union."
	)]
	impl<Label, EBrand, Row, RowAbort> HandlerPieces<Row, RowAbort> for TaggedBrand<Label, EBrand>
	where
		Label: 'static,
		EBrand: HandlerPieces<Row, RowAbort>,
		Row: WrapDrop + 'static,
	{
		type Arms<'h> = <EBrand as HandlerPieces<Row, RowAbort>>::Arms<'h>;

		/// Delegates to the bare effect's dispatch: a tagged cell
		/// interprets with the bare effect's arms, aborts, and re-entry.
		#[document_signature]
		///
		#[document_type_parameters("The program result type this dispatch interprets at.")]
		///
		#[document_parameters(
			"The lowered operation.",
			"The bare effect's arm bundle.",
			"The row handler driving interpretation.",
			"The injection from the effect's abort into the row abort."
		)]
		///
		#[document_returns("The continuation program, or the abort that ends interpretation.")]
		#[document_examples(
			skip_call_check,
			reason = "Dispatch is consumed by the emitted one-pass loop rather than called directly; the example demonstrates the delegation through a handler surface whose row holds a tagged cell."
		)]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		define_row,
		/// 		types::{
		/// 			Free,
		/// 			effects::{
		/// 				handle::RowHandler,
		/// 				state::{
		/// 					StateArms,
		/// 					StateBrand,
		/// 					get_at,
		/// 				},
		/// 				tagged::TaggedBrand,
		/// 			},
		/// 		},
		/// 	},
		/// 	std::cell::Cell,
		/// };
		///
		/// /// The example's label.
		/// pub struct Purse;
		///
		/// define_row! {
		/// 	/// One labelled integer state cell.
		/// 	#[handlers]
		/// 	pub row PurseRow {
		/// 		TaggedBrand<Purse, StateBrand<i32>>,
		/// 	}
		/// }
		///
		/// // The tagged cell's arms are the bare effect's arms, by delegation.
		/// let state = Cell::new(7);
		/// let handlers = PurseRowHandlers {
		/// 	purse_state: StateArms {
		/// 		get: Box::new(|| state.get()),
		/// 		put: Box::new(|value| state.set(value)),
		/// 	},
		/// };
		///
		/// // The labelled read targets the cell tagged `Purse`.
		/// let program: Free<PurseRow, i32> = get_at::<Purse, i32, _, _>();
		/// assert_eq!(handlers.handle(program).ok(), Some(7));
		/// ```
		fn dispatch<T: 'static>(
			op: <Self as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
			arms: &Self::Arms<'_>,
			handler: &impl RowHandler<Row, RowAbort>,
			inject_abort: impl Fn(<Self as EffectAbort>::Abort) -> RowAbort,
		) -> Result<Free<Row, T>, RowAbort> {
			<EBrand as HandlerPieces<Row, RowAbort>>::dispatch(op, arms, handler, inject_abort)
		}
	}

	/// A step for the bare effect, adapted to a labelled cell: the adapter's
	/// label picks the tagged brand while the wrapped step does the work. A
	/// nominal adapter rather than a blanket delegation, so passing a bare
	/// step to a runner keeps inferring the bare brand.
	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The adapted step implementation."
	)]
	pub struct TaggedStep<Label, Step>(Step, PhantomData<Label>);

	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The adapted step implementation."
	)]
	#[document_parameters("The adapted step value.")]
	impl<Label, Step: Clone> Clone for TaggedStep<Label, Step> {
		/// Clones the wrapped step; the label is phantom, so no `Clone`
		/// bound falls on it.
		#[document_signature]
		///
		#[document_returns("An independent copy of the adapted step.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::types::effects::{
		/// 	state::StateStep,
		/// 	tagged::tag_step,
		/// };
		///
		/// /// The example's label.
		/// pub struct Purse;
		///
		/// let step = tag_step::<Purse, _>(StateStep);
		/// let copy = step.clone();
		/// assert_eq!(std::mem::size_of_val(&copy), 0);
		/// ```
		fn clone(&self) -> Self {
			TaggedStep(self.0.clone(), PhantomData)
		}
	}

	/// Adapts a bare-effect step to the labelled cell named by `Label`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The adapted step implementation."
	)]
	///
	#[document_parameters("The bare-effect step to adapt.")]
	///
	#[document_returns("The adapted step, usable wherever the labelled cell is eliminated.")]
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
	/// 			handle::{
	/// 				AccumStep,
	/// 				extract,
	/// 				handle_accum,
	/// 			},
	/// 			state::{
	/// 				StateBrand,
	/// 				StateStep,
	/// 				get_at,
	/// 			},
	/// 			tagged::{
	/// 				TaggedBrand,
	/// 				tag_step,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// /// The example's label.
	/// pub struct Purse;
	///
	/// define_row! {
	/// 	/// One labelled integer state cell.
	/// 	pub row PurseRow {
	/// 		TaggedBrand<Purse, StateBrand<i32>>,
	/// 	}
	/// }
	///
	/// // The labelled read targets the cell tagged `Purse`.
	/// let program: Free<PurseRow, i32> = get_at::<Purse, i32, _, _>();
	///
	/// let step = tag_step::<Purse, _>(StateStep);
	/// let narrowed: Free<CNilBrand, (i32, i32)> =
	/// 	handle_accum::<TaggedBrand<Purse, StateBrand<i32>>, _, _, _, _, _, _>(
	/// 		7,
	/// 		program,
	/// 		move |s, op| step.step(s, op),
	/// 	);
	/// assert_eq!(extract(narrowed), (7, 7));
	/// ```
	pub fn tag_step<Label, Step>(step: Step) -> TaggedStep<Label, Step> {
		TaggedStep(step, PhantomData)
	}

	#[document_type_parameters(
		"The label, any user-defined zero-sized type.",
		"The adapted step implementation.",
		"The bare effect brand the label wraps.",
		"The row brand the interpreted programs run over.",
		"The accumulator type."
	)]
	#[document_parameters("The adapted step value.")]
	impl<Label, Step, EBrand, Row, S> AccumStep<TaggedBrand<Label, EBrand>, Row, S>
		for TaggedStep<Label, Step>
	where
		Step: AccumStep<EBrand, Row, S>,
		Label: 'static,
		EBrand: LifetimeUnaryKind,
		Row: WrapDrop + 'static,
	{
		/// Delegates to the wrapped step's bare-effect implementation: the
		/// tagged projection is the bare projection, so the operation
		/// passes through unchanged.
		#[document_signature]
		///
		#[document_type_parameters("The program result type this application interprets at.")]
		///
		#[document_parameters("The current accumulator.", "The lowered operation to interpret.")]
		///
		#[document_returns("The new accumulator paired with the continuation program.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	define_row,
		/// 	types::{
		/// 		Free,
		/// 		effects::{
		/// 			handle::{
		/// 				AccumStep,
		/// 				extract,
		/// 				handle_accum,
		/// 			},
		/// 			state::{
		/// 				StateBrand,
		/// 				StateStep,
		/// 				get_at,
		/// 			},
		/// 			tagged::{
		/// 				TaggedBrand,
		/// 				tag_step,
		/// 			},
		/// 		},
		/// 	},
		/// };
		///
		/// /// The example's label.
		/// pub struct Purse;
		///
		/// define_row! {
		/// 	/// One labelled integer state cell.
		/// 	pub row PurseRow {
		/// 		TaggedBrand<Purse, StateBrand<i32>>,
		/// 	}
		/// }
		///
		/// // The labelled read targets the cell tagged `Purse`.
		/// let program: Free<PurseRow, i32> = get_at::<Purse, i32, _, _>();
		///
		/// let step = tag_step::<Purse, _>(StateStep);
		/// let narrowed: Free<CNilBrand, (i32, i32)> =
		/// 	handle_accum::<TaggedBrand<Purse, StateBrand<i32>>, _, _, _, _, _, _>(
		/// 		7,
		/// 		program,
		/// 		move |s, op| step.step(s, op),
		/// 	);
		/// assert_eq!(extract(narrowed), (7, 7));
		/// ```
		fn step<T: 'static>(
			&self,
			s: S,
			op: <TaggedBrand<Label, EBrand> as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
		) -> (S, Free<Row, T>) {
			<Step as AccumStep<EBrand, Row, S>>::step(&self.0, s, op)
		}
	}
}

pub use inner::*;
