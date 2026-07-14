//! Proof of concept for tagged (labelled) effects: a `TaggedBrand<Label,
//! EBrand>` wrapper whose identity changes the dispatch key while its kind
//! projection reuses the effect's own operations enum, so a tagged cell is
//! a distinct coproduct member carrying the same operations as its bare
//! effect. Two same-type effects then coexist in one row, selected by
//! label.
//!
//! Pinned here: (1) the generic kind projection delegates through
//! `impl_kind!`; (2) the `Functor` and `OrderOf` delegations compile; (3) a
//! hand-built tagged constructor injects at the tagged brand and both
//! tagged `State` cells dispatch independently through the generic runner;
//! (4) a step written for the bare effect serves the tagged brand by a
//! delegating `AccumStep` implementation.
#![cfg(feature = "effects")]

use {
	fp_library::{
		Apply,
		Kind,
		brands::CNilBrand,
		classes::Functor,
		define_row,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::{
				coproduct::{
					CoprodInjector,
					Coproduct,
				},
				handle::{
					AccumStep,
					extract,
					handle_accum,
				},
				order::OrderOf,
				state::{
					StateBrand,
					StateF,
					StateStep,
				},
			},
		},
	},
	std::marker::PhantomData,
};

/// A label brand: the wrapper's identity is the dispatch key, so the same
/// effect appears in one row once per label.
pub struct TaggedBrand<Label, EBrand>(PhantomData<(Label, EBrand)>);

/// The first label.
pub struct Fst;

/// The second label.
pub struct Snd;

impl_kind! {
	impl<Label, EBrand> for TaggedBrand<Label, EBrand>
	where
		Label: 'static,
		EBrand: LifetimeUnaryKind,
	{
		type Of<'a, A: 'a>: 'a = <EBrand as LifetimeUnaryKind>::Of<'a, A>;
	}
}

impl<Label: 'static, EBrand: Functor + LifetimeUnaryKind> Functor for TaggedBrand<Label, EBrand> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<EBrand as Functor>::map(f, fa)
	}
}

impl<Label, EBrand: OrderOf> OrderOf for TaggedBrand<Label, EBrand> {
	type Order = <EBrand as OrderOf>::Order;
}

/// A step written for the bare effect serves the tagged brand: the tagged
/// projection is the bare projection, so the delegation is an identity.
impl<Label: 'static, Row: fp_library::classes::WrapDrop + 'static, S: Clone + 'static>
	AccumStep<TaggedBrand<Label, StateBrand<S>>, Row, S> for StateStep
{
	fn step<T: 'static>(
		&self,
		s: S,
		op: <TaggedBrand<Label, StateBrand<S>> as LifetimeUnaryKind>::Of<'static, Free<Row, T>>,
	) -> (S, Free<Row, T>) {
		<StateStep as AccumStep<StateBrand<S>, Row, S>>::step(self, s, op)
	}
}

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
		get_at::<Fst, i32, _, _>().bind(|x: i32| {
			get_at::<Snd, i32, _, _>().bind(move |y: i32| Free::pure(x * 100 + y))
		})
	});
	// The delegating implementation makes a bare `StateStep.step` call
	// ambiguous here (the operation type is shared by the bare and tagged
	// implementations), so the eliminated brand is named at the call.
	let fst_handled: Free<SndOnlyRow, (i32, i32)> =
		handle_accum::<TaggedBrand<Fst, StateBrand<i32>>, _, _, _, _, _, _>(
			1,
			program,
			|s, op| {
				<StateStep as AccumStep<TaggedBrand<Fst, StateBrand<i32>>, _, _>>::step(
					&StateStep, s, op,
				)
			},
		);
	let snd_handled: Free<CNilBrand, (i32, (i32, i32))> =
		handle_accum::<TaggedBrand<Snd, StateBrand<i32>>, _, _, _, _, _, _>(
			2,
			fst_handled,
			|s, op| {
				<StateStep as AccumStep<TaggedBrand<Snd, StateBrand<i32>>, _, _>>::step(
					&StateStep, s, op,
				)
			},
		);
	let (snd_final, (fst_final, result)) = extract(snd_handled);
	assert_eq!(result, 1002);
	assert_eq!(fst_final, 10);
	assert_eq!(snd_final, 2);
}
