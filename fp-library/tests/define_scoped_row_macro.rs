//! Integration coverage for the public `define_scoped_row!` macro.
//!
//! These tests keep the macro contract self-contained: a generated
//! marker row must expose the same `Kind` projection as the canonical
//! scoped row, replace bare `Self` placeholders before sorting, and
//! delegate the row traits that `Run` requires.

use {
	core::marker::PhantomData,
	fp_library::{
		brands::{
			BoxBracketBrand,
			BoxBrand,
			BoxCatchBrand,
			BoxSpanBrand,
			CNilBrand,
			CoproductBrand,
			NodeBrand,
		},
		classes::{
			Functor,
			RefFunctor,
			SendFunctor,
			WrapDrop,
		},
		define_scoped_row,
		handlers,
		kinds::Kind_cdc7cd43dac7585f,
		scoped_effects,
		scoped_handlers,
		types::effects::{
			coproduct::{
				CNil,
				Coproduct,
			},
			run::Run,
			scoped_dispatchers::span_dispatcher,
			span::BoxSpan,
		},
	},
};

struct MacroError;

define_scoped_row! {
	pub struct EmptyScopedRow;
	[]
}

define_scoped_row! {
	struct MacroScopedRow;
	[
		BoxCatchBrand<BoxBrand, MacroError>,
		BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, Self>, i32, i32>,
	]
}

define_scoped_row! {
	struct SpanScopedRow;
	[
		BoxSpanBrand<BoxBrand, &'static str>,
	]
}

type ActualScopedProjection =
	<MacroScopedRow as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<'static, i32>;

type ExpectedScopedProjection = <CoproductBrand<
	BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, MacroScopedRow>, i32, i32>,
	CoproductBrand<BoxCatchBrand<BoxBrand, MacroError>, CNilBrand>,
> as fp_library::kinds::Kind_cdc7cd43dac7585f>::Of<'static, i32>;
type DirectSpanScopedRow = scoped_effects![BoxSpanBrand<BoxBrand, &'static str>];
type DirectSpanRun<A> = Run<CNilBrand, DirectSpanScopedRow, A>;
type NamedSpanRun<A> = Run<CNilBrand, SpanScopedRow, A>;

fn assert_same_type<T>(
	_: PhantomData<T>,
	_: PhantomData<T>,
) {
}

#[test]
fn marker_row_projects_to_sorted_underlying_scoped_row() {
	assert_same_type::<ActualScopedProjection>(
		PhantomData,
		PhantomData::<ExpectedScopedProjection>,
	);
}

#[test]
fn marker_row_delegates_required_row_traits() {
	fn assert_by_value_traits<T: WrapDrop + Functor + SendFunctor>() {}
	fn assert_ref_trait<T: RefFunctor>() {}

	assert_by_value_traits::<MacroScopedRow>();
	assert_ref_trait::<EmptyScopedRow>();
}

#[test]
fn empty_marker_row_drives_run_wrapper() {
	let run: Run<CNilBrand, EmptyScopedRow, i32> = Run::pure(42);

	assert!(matches!(run.peel(), Ok(42)));
}

#[test]
fn marker_span_row_keeps_action_slot_in_projection() {
	fn assert_projection<'a>(_lifetime: PhantomData<&'a ()>) {
		type Actual<'a> = <SpanScopedRow as Kind_cdc7cd43dac7585f>::Of<'a, &'a str>;
		type Expected<'a> = Coproduct<BoxSpan<'a, BoxBrand, &'static str, &'a str>, CNil>;

		assert_same_type::<Actual<'a>>(PhantomData, PhantomData::<Expected<'a>>);
	}

	assert_projection(PhantomData);
}

#[test]
fn direct_scoped_effects_row_supports_nested_run_span_constructors() {
	let inner: DirectSpanRun<i32> = Run::span::<&'static str, _>("inner", Run::pure(40));
	let program: DirectSpanRun<i32> =
		Run::span::<&'static str, _>("outer", inner).bind(|value| Run::pure(value + 2));

	let result = program.interpret(
		handlers! {},
		scoped_handlers! {
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn named_marker_row_supports_nested_run_span_constructors() {
	let inner: NamedSpanRun<i32> = Run::span::<&'static str, _>("inner", Run::pure(39));
	let program: NamedSpanRun<i32> =
		Run::span::<&'static str, _>("outer", inner).bind(|value| Run::pure(value + 3));

	let result = program.interpret(
		handlers! {},
		scoped_handlers! {
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}
