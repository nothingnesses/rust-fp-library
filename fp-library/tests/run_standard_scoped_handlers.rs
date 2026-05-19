#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// End-to-end integration tests for the standard scoped handlers.
//
// These tests exercise behaviour that the substrate shape tests cannot:
// `CatchHandler` rewrites `Except::Throw` operations inside a
// protected action via `interpose`; `LocalHandler` and
// `RefLocalHandler` ask the inherited Reader environment once, then
// answer Reader asks inside the scoped action with the modified
// environment; `SpanHandler` consumes the span tag and resumes the
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
// ordinary handling. The explicit Box-backed nested-Span case
// manually constructs an ordinary scoped Span program so this file can
// keep testing interaction between `CatchHandler` and ordinary scoped
// handling; the public `RunExplicit::span` constructor now returns
// an indexed boundary tested by the Span tests.

use {
	fp_library::{
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
			IdentityBrand,
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
		classes::{
			ToDynCloneFn,
			ToDynFnOnce,
			ToDynSendFn,
		},
		handlers,
		scoped_handlers,
		types::{
			ArcFreeExplicit,
			FreeExplicit,
			Identity,
			RcFreeExplicit,
			effects::{
				DispatchHandlers,
				DispatchScopedHandler,
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
				scoped_handlers_ordered,
				span::{
					BoxSpan,
					SendSpan,
					Span,
				},
				standard_scoped_handlers::{
					catch_handler,
					local_handler,
					ref_local_handler,
					span_handler,
				},
			},
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
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

fn rc_explicit_span_program(
	tag: &'static str,
	action: RcExplicitProg,
) -> RcExplicitProg {
	let span = Span::Span {
		tag,
		action: <RcBrand as ToDynCloneFn>::new(move |_: ()| action.clone().into_rc_free_explicit()),
	};
	let layer = Coproduct::Inr(Coproduct::Inl(span));

	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(Node::Scoped(layer)))
}

fn arc_explicit_span_program(
	tag: &'static str,
	action: ArcExplicitProg,
) -> ArcExplicitProg {
	let span = SendSpan::Span {
		tag,
		action: <ArcBrand as ToDynSendFn>::new(move |_: ()| {
			action.clone().into_arc_free_explicit()
		}),
	};
	let layer = Coproduct::Inr(Coproduct::Inl(span));

	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(Node::Scoped(layer)))
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
type BoxSpanIdentityFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type BoxSpanIdentityProg = Run<BoxSpanIdentityFirstRow, BoxSpanOnlyScopedRow, i32>;
type BoxExplicitSpanOnlyProg = RunExplicit<'static, CNilBrand, BoxSpanOnlyScopedRow, i32>;
type RcExplicitSpanOnlyScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
type RcExplicitSpanOnlyProg = RcRunExplicit<'static, CNilBrand, RcExplicitSpanOnlyScopedRow, i32>;
type ArcExplicitSpanOnlyScopedRow =
	CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;
type ArcExplicitSpanOnlyProg =
	ArcRunExplicit<'static, CNilBrand, ArcExplicitSpanOnlyScopedRow, i32>;

fn box_explicit_span_only_program(action: BoxExplicitSpanOnlyProg) -> BoxExplicitSpanOnlyProg {
	let action_free = Box::new(action.into_free_explicit());
	let span = BoxSpan::Span {
		tag: "ordinary",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
	};

	RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(Coproduct::Inl(span))))
}

fn rc_explicit_span_only_program(action: RcExplicitSpanOnlyProg) -> RcExplicitSpanOnlyProg {
	let span = Span::Span {
		tag: "ordinary",
		action: <RcBrand as ToDynCloneFn>::new(move |_: ()| action.clone().into_rc_free_explicit()),
	};

	RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::wrap(Node::Scoped(Coproduct::Inl(span))))
}

fn arc_explicit_span_only_program(action: ArcExplicitSpanOnlyProg) -> ArcExplicitSpanOnlyProg {
	let span = SendSpan::Span {
		tag: "ordinary",
		action: <ArcBrand as ToDynSendFn>::new(move |_: ()| {
			action.clone().into_arc_free_explicit()
		}),
	};

	ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::wrap(Node::Scoped(Coproduct::Inl(
		span,
	))))
}

// These dispatchers intentionally implement the ordinary scoped-handler
// contract only. Direct Explicit interpreters must accept them for already
// suspended scoped layers because those layers produce the next program
// directly; indexed boundaries use the separate boundary facade.
#[derive(Clone, Copy)]
struct OrdinaryOnlyBoxExplicitSpan;

impl<'a, FirstLayer>
	DispatchScopedHandler<
		'a,
		BoxSpan<'a, BoxBrand, &'static str, BoxExplicitSpanOnlyProg>,
		FirstLayer,
		BoxExplicitSpanOnlyProg,
	> for OrdinaryOnlyBoxExplicitSpan
where
	FirstLayer: 'a,
{
	fn dispatch_scoped_head(
		&self,
		layer: BoxSpan<'a, BoxBrand, &'static str, BoxExplicitSpanOnlyProg>,
		_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, BoxExplicitSpanOnlyProg>,
	) -> BoxExplicitSpanOnlyProg {
		match layer {
			BoxSpan::Span {
				tag,
				action,
			} => {
				assert_eq!(tag, "ordinary");
				action(())
			}
		}
	}
}

#[derive(Clone, Copy)]
struct OrdinaryOnlyRcExplicitSpan;

impl<'a, FirstLayer>
	DispatchScopedHandler<
		'a,
		Span<'a, RcBrand, &'static str, RcExplicitSpanOnlyProg>,
		FirstLayer,
		RcExplicitSpanOnlyProg,
	> for OrdinaryOnlyRcExplicitSpan
where
	FirstLayer: 'a,
{
	fn dispatch_scoped_head(
		&self,
		layer: Span<'a, RcBrand, &'static str, RcExplicitSpanOnlyProg>,
		_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcExplicitSpanOnlyProg>,
	) -> RcExplicitSpanOnlyProg {
		match layer {
			Span::Span {
				tag,
				action,
			} => {
				assert_eq!(tag, "ordinary");
				action(())
			}
		}
	}
}

#[derive(Clone, Copy)]
struct OrdinaryOnlyArcExplicitSpan;

impl<'a, FirstLayer>
	DispatchScopedHandler<
		'a,
		SendSpan<'a, ArcBrand, &'static str, ArcExplicitSpanOnlyProg>,
		FirstLayer,
		ArcExplicitSpanOnlyProg,
	> for OrdinaryOnlyArcExplicitSpan
where
	FirstLayer: 'a,
{
	fn dispatch_scoped_head(
		&self,
		layer: SendSpan<'a, ArcBrand, &'static str, ArcExplicitSpanOnlyProg>,
		_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcExplicitSpanOnlyProg>,
	) -> ArcExplicitSpanOnlyProg {
		match layer {
			SendSpan::Span {
				tag,
				action,
			} => {
				assert_eq!(tag, "ordinary");
				action(())
			}
		}
	}
}

#[test]
fn run_local_handler_modifies_reader_environment() {
	let action: BoxLocalProg =
		Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(|first: i32| {
			Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
				.bind(move |second: i32| Run::pure(first + second))
		});
	let program: BoxLocalProg = Run::local::<i32, _>(|env| env + 1, action);

	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn run_local_handler_modifies_action_before_outer_continuation() {
	let events = Rc::new(RefCell::new(Vec::new()));

	let events_for_action = Rc::clone(&events);
	let action: BoxLocalProg =
		Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(move |env: i32| {
			events_for_action.borrow_mut().push("action-bind");
			Run::pure(env)
		});

	let events_for_modify = Rc::clone(&events);
	let events_for_map = Rc::clone(&events);
	let events_for_bind = Rc::clone(&events);
	let program: BoxLocalProg = Run::local::<i32, _>(
		move |env| {
			events_for_modify.borrow_mut().push("modify");
			env + 1
		},
		action,
	)
	.map(move |value| {
		events_for_map.borrow_mut().push("outer-map");
		value + 1
	})
	.bind(move |value| {
		events_for_bind.borrow_mut().push("outer-bind");
		Run::pure(value * 2)
	});

	let events_for_reader = Rc::clone(&events);
	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: move |op: BoxReader<'_, BoxBrand, i32, BoxLocalProg>| match op {
				BoxReader::Ask(k) => {
					events_for_reader.borrow_mut().push("outer-reader");
					k(10)
				}
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 24);
	assert_eq!(
		events.borrow().as_slice(),
		["outer-reader", "modify", "action-bind", "outer-map", "outer-bind"],
	);
}

#[test]
fn run_ref_local_handler_modifies_reader_environment() {
	let action: BoxLocalProg =
		Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(|first: i32| {
			Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
				.bind(move |second: i32| Run::pure(first + second))
		});
	let program: BoxLocalProg = Run::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn run_ref_local_handler_modifies_action_before_outer_continuation() {
	let events = Rc::new(RefCell::new(Vec::new()));

	let events_for_action = Rc::clone(&events);
	let action: BoxLocalProg =
		Run::<BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(move |env: i32| {
			events_for_action.borrow_mut().push("action-bind");
			Run::pure(env)
		});

	let events_for_modify = Rc::clone(&events);
	let events_for_map = Rc::clone(&events);
	let events_for_bind = Rc::clone(&events);
	let program: BoxLocalProg = Run::ref_local::<i32, _>(
		move |env| {
			events_for_modify.borrow_mut().push("ref-modify");
			*env + 5
		},
		action,
	)
	.map(move |value| {
		events_for_map.borrow_mut().push("outer-map");
		value + 1
	})
	.bind(move |value| {
		events_for_bind.borrow_mut().push("outer-bind");
		Run::pure(value * 2)
	});

	let events_for_reader = Rc::clone(&events);
	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: move |op: BoxReader<'_, BoxBrand, i32, BoxLocalProg>| match op {
				BoxReader::Ask(k) => {
					events_for_reader.borrow_mut().push("outer-reader");
					k(10)
				}
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 32);
	assert_eq!(
		events.borrow().as_slice(),
		["outer-reader", "ref-modify", "action-bind", "outer-map", "outer-bind"],
	);
}

#[test]
fn run_explicit_local_handler_modifies_reader_environment() {
	let action: BoxLocalExplicitProg =
		RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RunExplicit::pure(first + second))
			},
		);
	let boundary = RunExplicit::local::<i32, _>(|env| env + 1, action);
	let program: BoxLocalExplicitProg = local_handler::<_, BoxLocalFirstRowMinusReader, _>()
		.dispatch_run_explicit_local_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalExplicitProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn run_explicit_ref_local_handler_modifies_reader_environment() {
	let action: BoxLocalExplicitProg =
		RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RunExplicit::<'static, BoxLocalFirstRow, BoxLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RunExplicit::pure(first + second))
			},
		);
	let boundary = RunExplicit::ref_local::<i32, _>(|env| *env + 5, action);
	let program: BoxLocalExplicitProg = ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>()
		.dispatch_run_explicit_ref_local_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, BoxLocalExplicitProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, BoxLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn rc_run_local_handler_modifies_reader_environment() {
	let action: RcLocalProg =
		RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(|first: i32| {
			RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| RcRun::pure(first + second))
		});
	let program: RcLocalProg = RcRun::local::<i32, _>(|env| env + 1, action);

	let result = program.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_handler::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn rc_run_ref_local_handler_modifies_reader_environment() {
	let action: RcLocalProg =
		RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(|first: i32| {
			RcRun::<RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| RcRun::pure(first + second))
		});
	let program: RcLocalProg = RcRun::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_handler::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn rc_run_explicit_local_handler_modifies_reader_environment() {
	let action: RcLocalExplicitProg =
		RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RcRunExplicit::pure(first + second))
			},
		);
	let boundary = RcRunExplicit::local::<i32, _>(|env| env + 1, action);
	let program: RcLocalExplicitProg = local_handler::<_, RcLocalFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_local_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalExplicitProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_handler::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn rc_run_explicit_ref_local_handler_modifies_reader_environment() {
	let action: RcLocalExplicitProg =
		RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				RcRunExplicit::<'static, RcLocalFirstRow, RcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| RcRunExplicit::pure(first + second))
			},
		);
	let boundary = RcRunExplicit::ref_local::<i32, _>(|env| *env + 5, action);
	let program: RcLocalExplicitProg = ref_local_handler::<_, RcLocalFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_ref_local_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalExplicitProg>| match op {
				Reader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_handler::<_, RcLocalFirstRowMinusReader, _>(),
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn arc_run_local_handler_modifies_reader_environment() {
	let action: ArcLocalProg =
		ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(|first: i32| {
			ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| ArcRun::pure(first + second))
		});
	let program: ArcLocalProg = ArcRun::local::<i32, _>(|env| env + 1, action);

	let result = program.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn arc_run_ref_local_handler_modifies_reader_environment() {
	let action: ArcLocalProg =
		ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(|first: i32| {
			ArcRun::<ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
				.bind(move |second: i32| ArcRun::pure(first + second))
		});
	let program: ArcLocalProg = ArcRun::ref_local::<i32, _>(|env| *env + 5, action);

	let result = program.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn arc_run_explicit_local_handler_modifies_reader_environment() {
	let action: ArcLocalExplicitProg =
		ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| ArcRunExplicit::pure(first + second))
			},
		);
	let boundary = ArcRunExplicit::local::<i32, _>(|env| env + 1, action);
	let program: ArcLocalExplicitProg = local_handler::<_, ArcLocalFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_local_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalExplicitProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn arc_run_explicit_ref_local_handler_modifies_reader_environment() {
	let action: ArcLocalExplicitProg =
		ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask().bind(
			|first: i32| {
				ArcRunExplicit::<'static, ArcLocalFirstRow, ArcLocalScopedRow, i32>::ask()
					.bind(move |second: i32| ArcRunExplicit::pure(first + second))
			},
		);
	let boundary = ArcRunExplicit::ref_local::<i32, _>(|env| *env + 5, action);
	let program: ArcLocalExplicitProg = ref_local_handler::<_, ArcLocalFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_ref_local_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalExplicitProg>| match op {
				SendReader::Ask(k) => (*k)(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn run_span_handler_propagates_nested_action_result() {
	let program: BoxSpanOnlyProg =
		Run::span::<&'static str, _>("outer", Run::span::<&'static str, _>("inner", Run::pure(42)));

	let result = program.handle(
		handlers! {},
		scoped_handlers! {
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_span_handler_preserves_nested_action_and_outer_continuation_order() {
	let events = Rc::new(RefCell::new(Vec::new()));

	let events_for_action = Rc::clone(&events);
	let action: BoxSpanIdentityProg =
		Run::lift::<IdentityBrand, _>(Identity(10)).bind(move |value| {
			events_for_action.borrow_mut().push("action-bind");
			Run::pure(value + 1)
		});
	let inner: BoxSpanIdentityProg = Run::span::<&'static str, _>("inner", action);

	let events_for_map = Rc::clone(&events);
	let events_for_bind = Rc::clone(&events);
	let program: BoxSpanIdentityProg = Run::span::<&'static str, _>("outer", inner)
		.map(move |value| {
			events_for_map.borrow_mut().push("outer-map");
			value + 1
		})
		.bind(move |value| {
			events_for_bind.borrow_mut().push("outer-bind");
			Run::pure(value * 2)
		});

	let events_for_handler = Rc::clone(&events);
	let result = program.handle(
		handlers! {
			IdentityBrand: move |op: Identity<BoxSpanIdentityProg>| {
				events_for_handler.borrow_mut().push("identity");
				op.0
			},
		},
		scoped_handlers! {
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 24);
	assert_eq!(events.borrow().as_slice(), ["identity", "action-bind", "outer-map", "outer-bind"]);
}

#[test]
fn run_explicit_handle_accepts_ordinary_only_scoped_handlers() {
	let program = box_explicit_span_only_program(RunExplicit::pure(41))
		.bind(|value| RunExplicit::pure(value + 1));

	let result = program.handle(
		handlers! {},
		scoped_handlers_ordered()
			.on::<BoxSpanBrand<BoxBrand, &'static str>, _>(OrdinaryOnlyBoxExplicitSpan)
			.finish(),
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_explicit_handle_accepts_ordinary_only_scoped_handlers() {
	let program = rc_explicit_span_only_program(RcRunExplicit::pure(41))
		.bind(|value| RcRunExplicit::pure(value + 1));

	let result = program.handle(
		handlers! {},
		scoped_handlers_ordered()
			.on::<SpanBrand<RcBrand, &'static str>, _>(OrdinaryOnlyRcExplicitSpan)
			.finish(),
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_explicit_handle_accepts_ordinary_only_scoped_handlers() {
	let program = arc_explicit_span_only_program(ArcRunExplicit::pure(41))
		.bind(|value| ArcRunExplicit::pure(value + 1));

	let result = program.handle(
		handlers! {},
		scoped_handlers_ordered()
			.on::<SendSpanBrand<ArcBrand, &'static str>, _>(OrdinaryOnlyArcExplicitSpan)
			.finish(),
	);

	assert_eq!(result, 42);
}

#[test]
fn run_catch_handles_throw_inside_nested_span() {
	let action: BoxProg =
		Run::span::<&'static str, _>("inner", Run::throw::<&'static str, _>("from-action"));
	let program: BoxProg = Run::catch::<&'static str, _>(action, |_e| Run::pure(42));

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BoxProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_recovery_throw_escapes_same_catch_frame() {
	let action: BoxProg = Run::throw::<&'static str, _>("from-action");
	let program: BoxProg =
		Run::catch::<&'static str, _>(action, |_e| Run::throw::<&'static str, _>("from-recovery"));

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, BoxProg>| match op {
				Except::Throw("from-recovery", _) => Run::pure(42),
				Except::Throw(_, _) => Run::pure(0),
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_catch_recovery_resumes_outer_continuation_after_recovery() {
	let events = Rc::new(RefCell::new(Vec::new()));

	let action: BoxProg = Run::throw::<&'static str, _>("from-action");
	let events_for_recovery = Rc::clone(&events);
	let events_for_map = Rc::clone(&events);
	let events_for_bind = Rc::clone(&events);
	let program: BoxProg = Run::catch::<&'static str, _>(action, move |err| {
		assert_eq!(err, "from-action");
		events_for_recovery.borrow_mut().push("recovery");
		Run::pure(40)
	})
	.map(move |value| {
		events_for_map.borrow_mut().push("outer-map");
		value + 1
	})
	.bind(move |value| {
		events_for_bind.borrow_mut().push("outer-bind");
		Run::pure(value + 1)
	});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BoxProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_eq!(events.borrow().as_slice(), ["recovery", "outer-map", "outer-bind"]);
}

#[test]
fn run_explicit_catch_handles_throw_inside_nested_span() {
	let action: BoxExplicitProg =
		box_explicit_span_program(RunExplicit::throw::<&'static str, _>("from-action"));
	let boundary = RunExplicit::catch::<&'static str, _>(action, |_e| RunExplicit::pure(42));
	let program: BoxExplicitProg = catch_handler::<_, BoxFirstRowMinusExcept, _>()
		.dispatch_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, BoxExplicitProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
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
	let program: BoxExplicitProg = catch_handler::<_, BoxFirstRowMinusExcept, _>()
		.dispatch_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, BoxExplicitProg>| match op {
				Except::Throw("from-recovery", _) => RunExplicit::pure(42),
				Except::Throw(_, _) => RunExplicit::pure(0),
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, BoxFirstRowMinusExcept, _>(),
			BoxSpanBrand<BoxBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_catch_handles_throw_inside_nested_span() {
	let action: RcProg =
		RcRun::span::<&'static str, _>("inner", RcRun::throw::<&'static str, _>("from-action"));
	let program: RcProg = RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(42));

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RcProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: catch_handler::<_, RcFirstRowMinusExcept, _>(),
			SpanBrand<RcBrand, &'static str>: span_handler(),
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

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcProg>| match op {
				Except::Throw("from-recovery", _) => RcRun::pure(42),
				Except::Throw(_, _) => RcRun::pure(0),
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: catch_handler::<_, RcFirstRowMinusExcept, _>(),
			SpanBrand<RcBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_explicit_catch_handles_throw_inside_nested_span() {
	let action: RcExplicitProg =
		rc_explicit_span_program("inner", RcRunExplicit::throw::<&'static str, _>("from-action"));
	let boundary = RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(42));
	let program: RcExplicitProg = catch_handler::<_, RcFirstRowMinusExcept, _>()
		.dispatch_rc_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RcExplicitProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: catch_handler::<_, RcFirstRowMinusExcept, _>(),
			SpanBrand<RcBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_explicit_recovery_throw_escapes_same_catch_frame() {
	let action: RcExplicitProg = RcRunExplicit::throw::<&'static str, _>("from-action");
	let boundary = RcRunExplicit::catch::<&'static str, _>(action, |_e| {
		RcRunExplicit::throw::<&'static str, _>("from-recovery")
	});
	let program: RcExplicitProg = catch_handler::<_, RcFirstRowMinusExcept, _>()
		.dispatch_rc_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, RcExplicitProg>| match op {
				Except::Throw("from-recovery", _) => RcRunExplicit::pure(42),
				Except::Throw(_, _) => RcRunExplicit::pure(0),
			},
		},
		scoped_handlers! {
			CatchBrand<RcBrand, &'static str>: catch_handler::<_, RcFirstRowMinusExcept, _>(),
			SpanBrand<RcBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_catch_handles_throw_inside_nested_span() {
	let action: ArcProg =
		ArcRun::span::<&'static str, _>("inner", ArcRun::throw::<&'static str, _>("from-action"));
	let program: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(42));

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ArcProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			SendCatchBrand<ArcBrand, &'static str>: catch_handler::<_, ArcFirstRowMinusExcept, _>(),
			SendSpanBrand<ArcBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_explicit_catch_handles_throw_inside_nested_span() {
	let action: ArcExplicitProg =
		arc_explicit_span_program("inner", ArcRunExplicit::throw::<&'static str, _>("from-action"));
	let boundary = ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(42));
	let program: ArcExplicitProg = catch_handler::<_, ArcFirstRowMinusExcept, _>()
		.dispatch_arc_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ArcExplicitProg>| {
				panic!("CatchHandler should replace throws inside the protected action")
			},
		},
		scoped_handlers! {
			SendCatchBrand<ArcBrand, &'static str>: catch_handler::<_, ArcFirstRowMinusExcept, _>(),
			SendSpanBrand<ArcBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_explicit_recovery_throw_escapes_same_catch_frame() {
	let action: ArcExplicitProg = ArcRunExplicit::throw::<&'static str, _>("from-action");
	let boundary = ArcRunExplicit::catch::<&'static str, _>(action, |_e| {
		ArcRunExplicit::throw::<&'static str, _>("from-recovery")
	});
	let program: ArcExplicitProg = catch_handler::<_, ArcFirstRowMinusExcept, _>()
		.dispatch_arc_run_explicit_catch_boundary(boundary, &handlers! {});

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcExplicitProg>| match op {
				Except::Throw("from-recovery", _) => ArcRunExplicit::pure(42),
				Except::Throw(_, _) => ArcRunExplicit::pure(0),
			},
		},
		scoped_handlers! {
			SendCatchBrand<ArcBrand, &'static str>: catch_handler::<_, ArcFirstRowMinusExcept, _>(),
			SendSpanBrand<ArcBrand, &'static str>: span_handler(),
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

	let result = program.handle(
		handlers! {
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, ArcProg>| match op {
				Except::Throw("from-recovery", _) => ArcRun::pure(42),
				Except::Throw(_, _) => ArcRun::pure(0),
			},
		},
		scoped_handlers! {
			SendCatchBrand<ArcBrand, &'static str>: catch_handler::<_, ArcFirstRowMinusExcept, _>(),
			SendSpanBrand<ArcBrand, &'static str>: span_handler(),
		},
	);

	assert_eq!(result, 42);
}
