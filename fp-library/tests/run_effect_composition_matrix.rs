//! Cross-cutting effects composition regressions.
//!
//! The lower-level scoped-dispatcher tests check each standard handler
//! family in isolation. These tests combine first-order handlers,
//! scoped handlers, shared-wrapper reuse, and Explicit boundary
//! dispatch in small end-to-end programs so composition failures show
//! up before they are hidden inside larger ports.

use {
	fp_library::{
		brands::{
			ArcBrand,
			ArcCoyonedaBrand,
			BoxBrand,
			BoxCatchBrand,
			BoxLocalBrand,
			BoxReaderBrand,
			BoxStateBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			ExceptBrand,
			LocalBrand,
			RcBrand,
			RcCoyonedaBrand,
			ReaderBrand,
			SendLocalBrand,
			SendReaderBrand,
		},
		handlers,
		scoped_handlers,
		types::effects::{
			arc_run::ArcRun,
			except::Except,
			rc_run::RcRun,
			reader::{
				BoxReader,
				Reader,
				SendReader,
			},
			run::Run,
			run_explicit::RunExplicit,
			standard_scoped_handlers::{
				catch_handler,
				local_handler,
			},
			state::BoxState,
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
	},
};

type DefaultFirstRow = CoproductBrand<
	CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>,
	CoproductBrand<
		CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>,
		CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>,
	>,
>;
type DefaultFirstRowMinusReader = CoproductBrand<
	CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>,
	CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>,
>;
type DefaultFirstRowMinusExcept = CoproductBrand<
	CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>,
	CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>,
>;
type DefaultScopedRow = CoproductBrand<
	BoxCatchBrand<BoxBrand, &'static str>,
	CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>,
>;
type DefaultProg<A> = Run<DefaultFirstRow, DefaultScopedRow, A>;

fn interpret_default(program: DefaultProg<i32>) -> (i32, i32) {
	let state = Rc::new(RefCell::new(0));
	let state_for_handler = Rc::clone(&state);
	let result = program.interpret(
		handlers! {
			BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, DefaultProg<i32>>| match op {
				BoxState::Get(k) => k(*state_for_handler.borrow()),
				BoxState::Put(next, k) => {
					*state_for_handler.borrow_mut() = next;
					k(())
				}
			},
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, DefaultProg<i32>>| match op {
				BoxReader::Ask(k) => k(10),
			},
			ExceptBrand<&'static str>: |op: Except<'_, &'static str, DefaultProg<i32>>| match op {
				Except::Throw("uncaught", _) => Run::pure(-1),
				Except::Throw(_, _) => Run::pure(-2),
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, DefaultFirstRowMinusExcept, _>(),
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, DefaultFirstRowMinusReader, _>(),
		},
	);

	(result, *state.borrow())
}

#[test]
fn default_run_composes_first_order_scoped_handlers_and_outer_binds() {
	let action: DefaultProg<i32> = Run::local::<i32, _>(
		|env| env + 2,
		Run::<DefaultFirstRow, DefaultScopedRow, i32>::ask::<_>().bind(|local_env| {
			Run::<DefaultFirstRow, DefaultScopedRow, ()>::put::<i32, _>(local_env)
				.bind(|()| Run::throw::<&'static str, _>("caught"))
		}),
	);
	let program: DefaultProg<i32> = Run::catch::<&'static str, _>(action, |err| {
		assert_eq!(err, "caught");
		Run::<DefaultFirstRow, DefaultScopedRow, i32>::get::<_>()
	})
	.bind(|state_after_catch| {
		Run::<DefaultFirstRow, DefaultScopedRow, i32>::ask::<_>()
			.map(move |outer_env| state_after_catch + outer_env)
	})
	.map(|sum| sum * 2);

	assert_eq!(interpret_default(program), (44, 12));
}

type RcLocalFirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcLocalFirstRowMinusReader = CNilBrand;
type RcLocalScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
type RcLocalProg = RcRun<RcLocalFirstRow, RcLocalScopedRow, i32>;

fn interpret_rc_local(program: RcLocalProg) -> i32 {
	program.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcLocalProg>| match op {
				Reader::Ask(k) => k(20),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_handler::<_, RcLocalFirstRowMinusReader, _>(),
		},
	)
}

type ArcLocalFirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type ArcLocalFirstRowMinusReader = CNilBrand;
type ArcLocalScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
type ArcLocalProg = ArcRun<ArcLocalFirstRow, ArcLocalScopedRow, i32>;

fn interpret_arc_local(program: ArcLocalProg) -> i32 {
	program.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcLocalProg>| match op {
				SendReader::Ask(k) => k(30),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_handler::<_, ArcLocalFirstRowMinusReader, _>(),
		},
	)
}

#[test]
fn shared_wrappers_repeat_scoped_dispatch_without_consuming_programs() {
	let rc_program: RcLocalProg =
		RcRun::local::<i32, _>(|env| env + 1, RcRun::ask()).map(|value| value * 2);
	assert_eq!(interpret_rc_local(rc_program.clone()), 42);
	assert_eq!(interpret_rc_local(rc_program), 42);

	let arc_program: ArcLocalProg =
		ArcRun::local::<i32, _>(|env| env + 1, ArcRun::ask()).map(|value| value + 11);
	assert_eq!(interpret_arc_local(arc_program.clone()), 42);
	assert_eq!(interpret_arc_local(arc_program), 42);
}

type ExplicitFirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type ExplicitFirstRowMinusReader = CNilBrand;
type ExplicitScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
type ExplicitProg = RunExplicit<'static, ExplicitFirstRow, ExplicitScopedRow, i32>;

#[test]
fn explicit_boundary_dispatch_composes_typed_action_with_outer_bind() {
	let action: ExplicitProg =
		RunExplicit::<'static, ExplicitFirstRow, ExplicitScopedRow, i32>::ask()
			.bind(|env| RunExplicit::pure(env + 1));
	let boundary = RunExplicit::local::<i32, _>(|env| env * 2, action);
	let program: ExplicitProg = local_handler::<_, ExplicitFirstRowMinusReader, _>()
		.dispatch_run_explicit_local_boundary(boundary, &handlers! {})
		.bind(|value| RunExplicit::pure(value + 1));

	let result = program.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, ExplicitProg>| match op {
				BoxReader::Ask(k) => k(20),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, ExplicitFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}
