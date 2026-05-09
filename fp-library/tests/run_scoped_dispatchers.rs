#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// End-to-end integration tests for the standard scoped dispatchers.
//
// These tests exercise behaviour that the substrate shape tests cannot:
// `CatchDispatcher` rewrites `Except::Throw` operations inside a
// protected action via `interpose`, while `SpanDispatcher` consumes the
// span tag and resumes the action unchanged. The nested-span cases prove
// that interpose preserves surrounding scoped operations. The recovery
// rethrow cases prove that a `Throw` produced by the recovery handler is
// outside the protected action and is not caught by the same Catch frame.
//
// Coverage is limited to Rc/Arc-backed wrappers here. Box-backed Catch
// dispatch needs a different design because the protected action and the
// recovery handler can both need the same single-shot `Free` continuation
// after `Run::peel` maps the suspended `BoxCatch` layer.

use fp_library::{
	brands::{
		ArcBrand,
		ArcCoyonedaBrand,
		CNilBrand,
		CatchBrand,
		CoproductBrand,
		ExceptBrand,
		RcBrand,
		RcCoyonedaBrand,
		SendCatchBrand,
		SendSpanBrand,
		SpanBrand,
	},
	handlers,
	scoped_handlers,
	types::effects::{
		arc_run::ArcRun,
		except::Except,
		rc_run::RcRun,
		scoped_dispatchers::{
			catch_dispatcher,
			span_dispatcher,
		},
	},
};

type RcFirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type RcFirstRowMinusExcept = CNilBrand;
type RcScopedRow = CoproductBrand<
	CatchBrand<RcBrand, &'static str>,
	CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>,
>;
type RcProg = RcRun<RcFirstRow, RcScopedRow, i32>;

type ArcFirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type ArcFirstRowMinusExcept = CNilBrand;
type ArcScopedRow = CoproductBrand<
	SendCatchBrand<ArcBrand, &'static str>,
	CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>,
>;
type ArcProg = ArcRun<ArcFirstRow, ArcScopedRow, i32>;

#[test]
fn rc_run_catch_handles_throw_inside_nested_span() {
	let action: RcProg =
		RcRun::span::<&'static str, _>("inner", RcRun::throw::<&'static str, _>("from-action"));
	let program: RcProg = RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(42));

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RcProg>| {
				panic!("CatchDispatcher should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: catch_dispatcher::<_, RcFirstRowMinusExcept, _>(),
			SpanBrand<RcBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_recovery_throw_escapes_same_catch_frame() {
	let action: RcProg = RcRun::throw::<&'static str, _>("from-action");
	let program: RcProg = RcRun::catch::<&'static str, _>(action, |_e| {
		RcRun::throw::<&'static str, _>("from-recovery")
	});

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcProg>| match op {
				Except::Throw("from-recovery", _) => RcRun::pure(42),
				Except::Throw(_, _) => RcRun::pure(0),
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: catch_dispatcher::<_, RcFirstRowMinusExcept, _>(),
			SpanBrand<RcBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_catch_handles_throw_inside_nested_span() {
	let action: ArcProg =
		ArcRun::span::<&'static str, _>("inner", ArcRun::throw::<&'static str, _>("from-action"));
	let program: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(42));

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ArcProg>| {
				panic!("CatchDispatcher should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			SendCatchBrand<ArcBrand, &'static str>: catch_dispatcher::<_, ArcFirstRowMinusExcept, _>(),
			SendSpanBrand<ArcBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_recovery_throw_escapes_same_catch_frame() {
	let action: ArcProg = ArcRun::throw::<&'static str, _>("from-action");
	let program: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| {
		ArcRun::throw::<&'static str, _>("from-recovery")
	});

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcProg>| match op {
				Except::Throw("from-recovery", _) => ArcRun::pure(42),
				Except::Throw(_, _) => ArcRun::pure(0),
			},
		},
		scoped_handlers! {
			SendCatchBrand<ArcBrand, &'static str>: catch_dispatcher::<_, ArcFirstRowMinusExcept, _>(),
			SendSpanBrand<ArcBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}
