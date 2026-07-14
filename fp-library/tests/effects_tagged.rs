//! Tagged (labelled) effects, driven end to end: `TaggedBrand<Label,
//! EBrand>`'s identity changes the dispatch key while its kind projection
//! reuses the effect's own operations enum, so two same-type effects
//! coexist in one row, selected by label.
//!
//! Pinned here: (1) a hand-built tagged constructor injects at the tagged
//! brand and both tagged `State` cells dispatch independently through the
//! generic runner; (2) a step written for the bare effect serves each
//! labelled cell through the `tag_step` adapter, leaving the bare step's
//! own inference untouched.
#![cfg(feature = "effects")]

use fp_library::{
	brands::CNilBrand,
	classes::Functor,
	define_row,
	kinds::LifetimeUnaryKind,
	types::{
		Coyoneda,
		Free,
		effects::{
			coproduct::CoprodInjector,
			handle::{
				AccumStep,
				extract,
				handle_accum,
			},
			state::{
				StateBrand,
				StateF,
				StateStep,
			},
			tagged::{
				TaggedBrand,
				tag_step,
			},
		},
	},
};

/// The first label.
pub struct Fst;

/// The second label.
pub struct Snd;

// -- Hand-built tagged constructors (the shape the emission will take) --

/// Reads the state cell behind `Label`.
fn get_at<Label, S, R, I>() -> Free<R, S>
where
	Label: 'static,
	S: 'static,
	R: Functor + fp_library::classes::WrapDrop + 'static,
	<R as LifetimeUnaryKind>::Of<'static, S>:
		CoprodInjector<Coyoneda<'static, TaggedBrand<Label, StateBrand<S>>, S>, I>, {
	let cell: StateF<'static, S, S> = StateF::Get(Box::new(|x| x));
	let coyo: Coyoneda<'static, TaggedBrand<Label, StateBrand<S>>, S> = Coyoneda::lift(cell);
	let node: <R as LifetimeUnaryKind>::Of<'static, S> = CoprodInjector::inject(coyo);
	Free::lift_f(node)
}

/// Writes the state cell behind `Label`.
fn put_at<Label, S, R, I>(value: S) -> Free<R, ()>
where
	Label: 'static,
	S: 'static,
	R: Functor + fp_library::classes::WrapDrop + 'static,
	<R as LifetimeUnaryKind>::Of<'static, ()>:
		CoprodInjector<Coyoneda<'static, TaggedBrand<Label, StateBrand<S>>, ()>, I>, {
	let cell: StateF<'static, S, ()> = StateF::Put(value, Box::new(|x| x));
	let coyo: Coyoneda<'static, TaggedBrand<Label, StateBrand<S>>, ()> = Coyoneda::lift(cell);
	let node: <R as LifetimeUnaryKind>::Of<'static, ()> = CoprodInjector::inject(coyo);
	Free::lift_f(node)
}

define_row! {
	/// Two integer states, distinguished by label alone.
	pub row TwoStateRow {
		TaggedBrand<Fst, StateBrand<i32>>,
		TaggedBrand<Snd, StateBrand<i32>>,
	}
}

define_row! {
	/// The residual after the first label is eliminated.
	pub row SndOnlyRow {
		TaggedBrand<Snd, StateBrand<i32>>,
	}
}

#[test]
fn two_tagged_state_cells_dispatch_independently_by_label() {
	// The program writes the first cell and reads both; each label's runner
	// only sees its own operations, so the states stay independent.
	let program: Free<TwoStateRow, i32> = put_at::<Fst, i32, _, _>(10).bind(|()| {
		get_at::<Fst, i32, _, _>()
			.bind(|x: i32| get_at::<Snd, i32, _, _>().bind(move |y: i32| Free::pure(x * 100 + y)))
	});
	// Each label eliminates with the bare effect's step through the
	// `tag_step` adapter; the adapter picks the brand, so the bare step's
	// own call sites stay untouched.
	let fst_step = tag_step::<Fst, _>(StateStep);
	let fst_handled: Free<SndOnlyRow, (i32, i32)> =
		handle_accum::<TaggedBrand<Fst, StateBrand<i32>>, _, _, _, _, _, _>(
			1,
			program,
			move |s, op| fst_step.step(s, op),
		);
	let snd_step = tag_step::<Snd, _>(StateStep);
	let snd_handled: Free<CNilBrand, (i32, (i32, i32))> =
		handle_accum::<TaggedBrand<Snd, StateBrand<i32>>, _, _, _, _, _, _>(
			2,
			fst_handled,
			move |s, op| snd_step.step(s, op),
		);
	let (snd_final, (fst_final, result)) = extract(snd_handled);
	assert_eq!(result, 1002);
	assert_eq!(fst_final, 10);
	assert_eq!(snd_final, 2);
}
