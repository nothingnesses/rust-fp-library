#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// End-to-end integration tests for the standard scoped dispatchers.
//
// These tests exercise behaviour that the substrate shape tests cannot:
// `CatchDispatcher` rewrites `Except::Throw` operations inside a
// protected action via `interpose`; `LocalDispatcher` and
// `RefLocalDispatcher` ask the inherited Reader environment once, then
// answer Reader asks inside the scoped action with the modified
// environment; `SpanDispatcher` consumes the span tag and resumes the
// action unchanged. The nested-span cases prove that interpose preserves
// surrounding scoped operations. The recovery rethrow cases prove that a
// `Throw` produced by the recovery handler is outside the protected
// action and is not caught by the same Catch frame.
//
// The default Box-backed `Run` path uses raw scoped dispatch so the
// single-shot `Free` continuation is attached only after `Catch` chooses
// the protected action or recovery branch. The explicit Box-backed
// `RunExplicit::{catch, local, ref_local}` constructors now return
// indexed boundaries, so this file dispatches those boundaries before
// ordinary interpretation. The explicit Box-backed nested-Span case
// manually constructs an ordinary scoped Span program so this file can
// keep testing interaction between `CatchDispatcher` and ordinary scoped
// interpretation; the public `RunExplicit::span` constructor now returns
// an indexed boundary tested by the Span tests.

use fp_library::{
	brands::{
		ArcBrand,
		ArcCoyonedaBrand,
		BoxBrand,
		BoxCatchBrand,
		BoxLocalBrand,
		BoxReaderBrand,
		BoxRefLocalBrand,
		BoxSpanBrand,
		CNilBrand,
		CatchBrand,
		CoproductBrand,
		CoyonedaBrand,
		ExceptBrand,
		LocalBrand,
		RcBrand,
		RcCoyonedaBrand,
		ReaderBrand,
		RefLocalBrand,
		SendCatchBrand,
		SendLocalBrand,
		SendReaderBrand,
		SendRefLocalBrand,
		SendSpanBrand,
		SpanBrand,
	},
	classes::ToDynFnOnce,
	handlers,
	scoped_handlers,
	types::{
		FreeExplicit,
		effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			coproduct::Coproduct,
			except::Except,
			node::Node,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			reader::{
				BoxReader,
				Reader,
				SendReader,
			},
			run::Run,
			run_explicit::RunExplicit,
			scoped_dispatchers::{
				catch_dispatcher,
				local_dispatcher,
				ref_local_dispatcher,
				span_dispatcher,
			},
			span::BoxSpan,
		},
	},
};

type BoxFirstRow = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type BoxFirstRowMinusExcept = CNilBrand;
type BoxScopedRow = CoproductBrand<
	BoxCatchBrand<BoxBrand, &'static str>,
	CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>,
>;
type BoxProg = Run<BoxFirstRow, BoxScopedRow, i32>;
type BoxExplicitProg = RunExplicit<'static, BoxFirstRow, BoxScopedRow, i32>;

fn box_explicit_span_program(action: BoxExplicitProg) -> BoxExplicitProg {
	let action_free = Box::new(action.into_free_explicit());
	let span = BoxSpan::Span {
		tag: "inner",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
	};
	let layer = Coproduct::Inr(Coproduct::Inl(span));

	RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(layer)))
}

type RcFirstRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type RcFirstRowMinusExcept = CNilBrand;
type RcScopedRow = CoproductBrand<
	CatchBrand<RcBrand, &'static str>,
	CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>,
>;
type RcProg = RcRun<RcFirstRow, RcScopedRow, i32>;
type RcExplicitProg = RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32>;

type ArcFirstRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type ArcFirstRowMinusExcept = CNilBrand;
type ArcScopedRow = CoproductBrand<
	SendCatchBrand<ArcBrand, &'static str>,
	CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>,
>;
type ArcProg = ArcRun<ArcFirstRow, ArcScopedRow, i32>;
type ArcExplicitProg = ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32>;

type BoxLocalFirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type BoxLocalFirstRowMinusReader = CNilBrand;
type BoxLocalScopedRow = CoproductBrand<
	BoxLocalBrand<BoxBrand, i32>,
	CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>,
>;
type BoxLocalProg = Run<BoxLocalFirstRow, BoxLocalScopedRow, i32>;
type BoxLocalExplicitProg = RunExplicit<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>;

type RcLocalFirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcLocalFirstRowMinusReader = CNilBrand;
type RcLocalScopedRow = CoproductBrand<
	LocalBrand<RcBrand, i32>,
	CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>,
>;
type RcLocalProg = RcRun<RcLocalFirstRow, RcLocalScopedRow, i32>;
type RcLocalExplicitProg = RcRunExplicit<'static, RcLocalFirstRow, RcLocalScopedRow, i32>;

type ArcLocalFirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type ArcLocalFirstRowMinusReader = CNilBrand;
type ArcLocalScopedRow = CoproductBrand<
	SendLocalBrand<ArcBrand, i32>,
	CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>,
>;
type ArcLocalProg = ArcRun<ArcLocalFirstRow, ArcLocalScopedRow, i32>;
type ArcLocalExplicitProg = ArcRunExplicit<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>;

type BoxSpanOnlyScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
type BoxSpanOnlyProg = Run<CNilBrand, BoxSpanOnlyScopedRow, i32>;

#[test]
fn run_local_dispatcher_modifies_reader_environment() {
	let action: BoxLocalProg =
		Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(|first: i32| {
			Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
				.bind(move |second: i32| Run::pure(first + second))
		});
	let program: BoxLocalProg = Run::local::<i32, _>(|env| env + 1, action);

	let result = program.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn run_ref_local_dispatcher_modifies_reader_environment() {
	let action: BoxLocalProg =
		Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(|first: i32| {
			Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
				.bind(move |second: i32| Run::pure(first + second))
		});
	let program: BoxLocalProg = Run::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn run_explicit_local_dispatcher_modifies_reader_environment() {
	let action: BoxLocalExplicitProg =
		RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RunExplicit::pure(first + second))
			},
		);
	let boundary = RunExplicit::local::<i32, _>(|env| env + 1, action);
	let program: BoxLocalExplicitProg = local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>()
		.dispatch_run_explicit_local_boundary(boundary, &handlers! {});

	let result = program.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalExplicitProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn run_explicit_ref_local_dispatcher_modifies_reader_environment() {
	let action: BoxLocalExplicitProg =
		RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RunExplicit::pure(first + second))
			},
		);
	let boundary = RunExplicit::ref_local::<i32, _>(|env| *env + 5, action);
	let program: BoxLocalExplicitProg = ref_local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>()
		.dispatch_run_explicit_ref_local_boundary(boundary, &handlers! {});

	let result = program.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalExplicitProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_dispatcher::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn rc_run_local_dispatcher_modifies_reader_environment() {
	let action: RcLocalProg =
		RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(|first: i32| {
			RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| RcRun::pure(first + second))
		});
	let program: RcLocalProg = RcRun::local::<i32, _>(|env| env + 1, action);

	let result = program.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn rc_run_ref_local_dispatcher_modifies_reader_environment() {
	let action: RcLocalProg =
		RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(|first: i32| {
			RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| RcRun::pure(first + second))
		});
	let program: RcLocalProg = RcRun::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn rc_run_explicit_local_dispatcher_modifies_reader_environment() {
	let action: RcLocalExplicitProg =
		RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RcRunExplicit::pure(first + second))
			},
		);
	let program: RcLocalExplicitProg = RcRunExplicit::local::<i32, _>(|env| env + 1, action);

	let result = program.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalExplicitProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn rc_run_explicit_ref_local_dispatcher_modifies_reader_environment() {
	let action: RcLocalExplicitProg =
		RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RcRunExplicit::pure(first + second))
			},
		);
	let program: RcLocalExplicitProg = RcRunExplicit::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalExplicitProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_dispatcher::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn arc_run_local_dispatcher_modifies_reader_environment() {
	let action: ArcLocalProg =
		ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(|first: i32| {
			ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| ArcRun::pure(first + second))
		});
	let program: ArcLocalProg = ArcRun::local::<i32, _>(|env| env + 1, action);

	let result = program.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn arc_run_ref_local_dispatcher_modifies_reader_environment() {
	let action: ArcLocalProg =
		ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(|first: i32| {
			ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| ArcRun::pure(first + second))
		});
	let program: ArcLocalProg = ArcRun::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn arc_run_explicit_local_dispatcher_modifies_reader_environment() {
	let action: ArcLocalExplicitProg =
		ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| ArcRunExplicit::pure(first + second))
			},
		);
	let program: ArcLocalExplicitProg = ArcRunExplicit::local::<i32, _>(|env| env + 1, action);

	let result = program.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalExplicitProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn arc_run_explicit_ref_local_dispatcher_modifies_reader_environment() {
	let action: ArcLocalExplicitProg =
		ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| ArcRunExplicit::pure(first + second))
			},
		);
	let program: ArcLocalExplicitProg = ArcRunExplicit::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalExplicitProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_dispatcher::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn run_span_dispatcher_propagates_nested_action_result() {
	let program: BoxSpanOnlyProg =
		Run::span::<&'static str, _>("outer", Run::span::<&'static str, _>("inner", Run::pure(42)));

	let result = program.interpret(
		handlers! {},
		scoped_handlers! {
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_catch_handles_throw_inside_nested_span() {
	let action: BoxProg =
		Run::span::<&'static str, _>("inner", Run::throw::<&'static str, _>("from-action"));
	let program: BoxProg = Run::catch::<&'static str, _>(action, |_e| Run::pure(42));

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BoxProg>| {
				panic!("CatchDispatcher should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_dispatcher::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_recovery_throw_escapes_same_catch_frame() {
	let action: BoxProg = Run::throw::<&'static str, _>("from-action");
	let program: BoxProg =
		Run::catch::<&'static str, _>(action, |_e| Run::throw::<&'static str, _>("from-recovery"));

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, BoxProg>| match op {
				Except::Throw("from-recovery", _) => Run::pure(42),
				Except::Throw(_, _) => Run::pure(0),
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_dispatcher::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_explicit_catch_handles_throw_inside_nested_span() {
	let action: BoxExplicitProg =
		box_explicit_span_program(RunExplicit::throw::<&'static str, _>("from-action"));
	let boundary = RunExplicit::catch::<&'static str, _>(action, |_e| RunExplicit::pure(42));
	let program: BoxExplicitProg = catch_dispatcher::<_, BoxFirstRowMinusExcept, _>()
		.dispatch_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BoxExplicitProg>| {
				panic!("CatchDispatcher should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_dispatcher::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_explicit_recovery_throw_escapes_same_catch_frame() {
	let action: BoxExplicitProg = RunExplicit::throw::<&'static str, _>("from-action");
	let boundary = RunExplicit::catch::<&'static str, _>(action, |_e| {
		RunExplicit::throw::<&'static str, _>("from-recovery")
	});
	let program: BoxExplicitProg = catch_dispatcher::<_, BoxFirstRowMinusExcept, _>()
		.dispatch_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, BoxExplicitProg>| match op {
				Except::Throw("from-recovery", _) => RunExplicit::pure(42),
				Except::Throw(_, _) => RunExplicit::pure(0),
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_dispatcher::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_dispatcher(),
		},
	);

	assert_eq!(result, 42);
}

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
fn rc_run_explicit_catch_handles_throw_inside_nested_span() {
	let action: RcExplicitProg = RcRunExplicit::span::<&'static str, _>(
		"inner",
		RcRunExplicit::throw::<&'static str, _>("from-action"),
	);
	let program: RcExplicitProg =
		RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(42));

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RcExplicitProg>| {
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
fn rc_run_explicit_recovery_throw_escapes_same_catch_frame() {
	let action: RcExplicitProg = RcRunExplicit::throw::<&'static str, _>("from-action");
	let program: RcExplicitProg = RcRunExplicit::catch::<&'static str, _>(action, |_e| {
		RcRunExplicit::throw::<&'static str, _>("from-recovery")
	});

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcExplicitProg>| match op {
				Except::Throw("from-recovery", _) => RcRunExplicit::pure(42),
				Except::Throw(_, _) => RcRunExplicit::pure(0),
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
fn arc_run_explicit_catch_handles_throw_inside_nested_span() {
	let action: ArcExplicitProg = ArcRunExplicit::span::<&'static str, _>(
		"inner",
		ArcRunExplicit::throw::<&'static str, _>("from-action"),
	);
	let program: ArcExplicitProg =
		ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(42));

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ArcExplicitProg>| {
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
fn arc_run_explicit_recovery_throw_escapes_same_catch_frame() {
	let action: ArcExplicitProg = ArcRunExplicit::throw::<&'static str, _>("from-action");
	let program: ArcExplicitProg = ArcRunExplicit::catch::<&'static str, _>(action, |_e| {
		ArcRunExplicit::throw::<&'static str, _>("from-recovery")
	});

	let result = program.interpret(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcExplicitProg>| match op {
				Except::Throw("from-recovery", _) => ArcRunExplicit::pure(42),
				Except::Throw(_, _) => ArcRunExplicit::pure(0),
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
