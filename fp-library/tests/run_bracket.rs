#![cfg(feature = "effects")]
#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]
#![recursion_limit = "512"]

// Integration tests for the scoped `bracket<Resource, Idx>` smart
// constructor across the Run-wrapper family. The public bracket program
// returns the body result `B`; the stored Val body closure still returns
// `(resource, body_result)` so the dispatcher can pass the resource to
// release before returning `B`. Default `Run`, erased shared `RcRun` /
// `ArcRun`, and single-shot `RunExplicit` keep direct program-returning
// constructors; their sections verify substrate shape:
//   T1: `bracket(acquire, body, release)` produces a program suspended
//       at a `Node::Scoped` layer carrying a `BoxBracket` / `Bracket` /
//       `BoxBracketExplicit` cell projected via `Member::inject` at the
//       head of the scoped row.
//   T2: invoking the cell's stored `acquire` thunk (with the unit
//       argument) yields a substrate-typed program (`Free` / `RcFree` /
//       `Box<FreeExplicit>` / `RcFreeExplicit` / `ArcFreeExplicit`)
//       that, after wrapping with the wrapper's `from_*` constructor,
//       peels to the original resource value.
//   T3: invoking the cell's stored `body` closure with a sample
//       resource value yields a substrate-typed program that peels to
//       the expected `(resource, body_result)` pair.
//   T4: invoking the cell's stored `release` closure with a sample
//       resource value yields a substrate-typed program that peels to
//       `()`.
//   T5 (clone-able direct wrappers only: RcRun and ArcRun): cloning the
//       suspended program produces two independent peelable handles; each
//       clone's acquire thunk materialises to the original resource
//       value.
//   T6: standard `BracketHandler` handling runs acquire, body,
//       and release in order, and returns the body result after release.
//
// The single-shot `RunExplicit` and shared Explicit `RcRunExplicit` /
// `ArcRunExplicit` sections exercise indexed boundaries returned by
// their `bracket` constructors. Dispatcher application generates the
// body action from `acquire`, runs `release`, then resumes any mapped or
// bound outer continuation. Shared Explicit coverage additionally checks
// repeated Rc use and Arc `Send + Sync` obligations through the boundary
// dispatcher methods.
//
// `ArcRun::bracket` uses a custom test scoped row that stores the
// `SendBracket` layer directly. The ordinary recursive `CoproductBrand`
// marker row still overflows rustc's Send + Sync projection evaluator,
// but the dispatcher surface does not require that specific marker-row
// spelling in order to exercise `ArcRun::bracket`, `Member::inject`, and
// the standard lifecycle dispatcher.

use fp_library::{
	Apply,
	brands::{
		ArcBrand,
		BoxBracketBrand,
		BoxBracketExplicitBrand,
		BoxBrand,
		BracketBrand,
		BracketExplicitBrand,
		CNilBrand,
		CoproductBrand,
		NodeBrand,
		RcBrand,
		SendBracketBrand,
		SendBracketExplicitBrand,
	},
	classes::{
		Functor,
		SendFunctor,
		WrapDrop,
	},
	handlers,
	impl_kind,
	kinds::*,
	scoped_handlers,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::{
			ArcRunExplicit,
			ArcRunExplicitBoundary,
		},
		bracket::{
			BoxBracket,
			Bracket,
			SendBracket,
		},
		coproduct::{
			CNil,
			Coproduct,
			Here,
		},
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::{
			RcRunExplicit,
			RcRunExplicitBoundary,
		},
		run::Run,
		run_explicit::{
			RunExplicit,
			RunExplicitBoundary,
		},
		standard_scoped_handlers::bracket_handler,
	},
};

fn push_rc_event(
	events: &std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
	event: &'static str,
) {
	events.borrow_mut().push(event);
}

fn assert_rc_events(
	events: &std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
	expected: &[&'static str],
) {
	assert_eq!(events.borrow().as_slice(), expected);
}

fn push_arc_event(
	events: &std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
	event: &'static str,
) {
	match events.lock() {
		Ok(mut events) => events.push(event),
		Err(_) => panic!("events mutex should not be poisoned"),
	}
}

fn assert_arc_events(
	events: &std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
	expected: &[&'static str],
) {
	match events.lock() {
		Ok(events) => assert_eq!(events.as_slice(), expected),
		Err(_) => panic!("events mutex should not be poisoned"),
	}
}

// -- Run --

// User-facing scoped rows containing `BoxBracketBrand` cannot be
// defined as type aliases (Rust rejects the recursion). The marker
// struct breaks the type-alias cycle by hosting the recursive
// reference inside an `impl_kind!` projection plus trait impls that
// delegate to an `UnderlyingRow` type alias. Same pattern applies to
// every wrapper section below.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RunBracketRow;

type RunUnderlyingRow = CoproductBrand<
	BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, RunBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RunUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RunBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RunUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RunBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RunUnderlyingRow as Functor>::map(f, fa)
	}
}

type RunFirstRow = CNilBrand;
type RunAcquireProg = Run<RunFirstRow, RunBracketRow, i32>;
type RunBracketProg = Run<RunFirstRow, RunBracketRow, i32>;
type RunBracketBodyProg = Run<RunFirstRow, RunBracketRow, (i32, i32)>;
type RunReleaseProg = Run<RunFirstRow, RunBracketRow, ()>;

fn make_run_bracket() -> RunBracketProg {
	let acquire: RunAcquireProg = Run::pure(7);
	Run::<RunFirstRow, RunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: Box<i32>| Run::pure((*resource, 42)),
		|_resource: Box<i32>| Run::pure(()),
	)
}

#[test]
fn run_t1_bracket_produces_scoped_layer() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxBracket::Bracket))"),
	}
}

#[test]
fn run_t2_acquire_thunk_materialises_resource_program() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: RunAcquireProg = Run::from_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_t3_body_materialises_paired_program() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			body, ..
		}))) => {
			let materialised: RunBracketBodyProg = Run::from_free(body(Box::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_t4_release_materialises_unit_program() {
	match make_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxBracket::Bracket {
			release, ..
		}))) => {
			let materialised: RunReleaseProg = Run::from_free(release(Box::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn run_t6_bracket_handler_runs_lifecycle_in_order() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);

	let acquire: RunAcquireProg = Run::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		Run::pure(resource)
	});
	let program: RunBracketProg = Run::<RunFirstRow, RunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		move |resource: Box<i32>| {
			push_rc_event(&body_events, "body");
			Run::pure((*resource, *resource + 35))
		},
		move |resource: Box<i32>| {
			push_rc_event(&release_events, "release");
			assert_eq!(*resource, 7);
			Run::pure(())
		},
	);

	let result = program.handle(
		handlers! {},
		scoped_handlers! {
			BoxBracketBrand<BoxBrand, NodeBrand<RunFirstRow, RunBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_rc_events(&events, &["acquire", "body", "release"]);
}

#[test]
fn run_t7_bracket_handler_runs_lifecycle_before_outer_continuation() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);
	let map_events = std::rc::Rc::clone(&events);
	let bind_events = std::rc::Rc::clone(&events);

	let acquire: RunAcquireProg = Run::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		Run::pure(resource)
	});
	let bracket: RunBracketProg = Run::<RunFirstRow, RunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		move |resource: Box<i32>| {
			push_rc_event(&body_events, "body");
			Run::pure((*resource, *resource + 35))
		},
		move |resource: Box<i32>| {
			push_rc_event(&release_events, "release");
			assert_eq!(*resource, 7);
			Run::pure(())
		},
	);
	let program: Run<RunFirstRow, RunBracketRow, usize> = bracket
		.map(move |value| {
			push_rc_event(&map_events, "outer-map");
			value.to_string()
		})
		.bind(move |value| {
			push_rc_event(&bind_events, "outer-bind");
			Run::pure(value.len())
		});

	let result = program.handle(
		handlers! {},
		scoped_handlers! {
			BoxBracketBrand<BoxBrand, NodeBrand<RunFirstRow, RunBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 2);
	assert_rc_events(&events, &["acquire", "body", "release", "outer-map", "outer-bind"]);
}

// -- RcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunBracketRow;

type RcRunUnderlyingRow = CoproductBrand<
	BracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunFirstRow = CNilBrand;
type RcRunAcquireProg = RcRun<RcRunFirstRow, RcRunBracketRow, i32>;
type RcRunBracketProg = RcRun<RcRunFirstRow, RcRunBracketRow, i32>;
type RcRunBracketBodyProg = RcRun<RcRunFirstRow, RcRunBracketRow, (i32, i32)>;
type RcRunReleaseProg = RcRun<RcRunFirstRow, RcRunBracketRow, ()>;

fn make_rc_run_bracket() -> RcRunBracketProg {
	let acquire: RcRunAcquireProg = RcRun::pure(7);
	RcRun::<RcRunFirstRow, RcRunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRun::pure((*resource, 42)),
		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
	)
}

#[test]
fn rc_run_t1_bracket_produces_scoped_layer() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Bracket::Bracket))"),
	}
}

#[test]
fn rc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: RcRunAcquireProg = RcRun::from_rc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_t3_body_materialises_paired_program() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			body, ..
		}))) => {
			let materialised: RcRunBracketBodyProg = RcRun::from_rc_free(body(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_t4_release_materialises_unit_program() {
	match make_rc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
			release, ..
		}))) => {
			let materialised: RcRunReleaseProg = RcRun::from_rc_free(release(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn rc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_rc_run_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: RcRunBracketProg| -> RcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(Bracket::Bracket {
				acquire, ..
			}))) => RcRun::from_rc_free(acquire(())),
			_ => panic!("expected scoped bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

#[test]
fn rc_run_t6_bracket_handler_runs_lifecycle_in_order() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);

	let acquire: RcRunAcquireProg = RcRun::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RcRun::pure(resource)
	});
	let program: RcRunBracketProg = RcRun::<RcRunFirstRow, RcRunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		move |resource: std::rc::Rc<i32>| {
			push_rc_event(&body_events, "body");
			RcRun::pure((*resource, *resource + 35))
		},
		move |resource: std::rc::Rc<i32>| {
			push_rc_event(&release_events, "release");
			assert_eq!(*resource, 7);
			RcRun::pure(())
		},
	);

	let result = program.handle(
		handlers! {},
		scoped_handlers! {
			BracketBrand<RcBrand, NodeBrand<RcRunFirstRow, RcRunBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_rc_events(&events, &["acquire", "body", "release"]);
}

// -- RunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RunExplicitBracketRow;

type RunExplicitUnderlyingRow = CoproductBrand<
	BoxBracketExplicitBrand<BoxBrand, NodeBrand<CNilBrand, RunExplicitBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RunExplicitBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RunExplicitBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RunExplicitBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RunExplicitUnderlyingRow as Functor>::map(f, fa)
	}
}

type RunExplicitFirstRow = CNilBrand;
type RunExplicitBracketSBrand = BoxBracketExplicitBrand<
	BoxBrand,
	NodeBrand<RunExplicitFirstRow, RunExplicitBracketRow>,
	i32,
	i32,
>;
type RunExplicitAcquireProg = RunExplicit<'static, RunExplicitFirstRow, RunExplicitBracketRow, i32>;
type RunExplicitBracketProg = RunExplicit<'static, RunExplicitFirstRow, RunExplicitBracketRow, i32>;

fn make_run_explicit_bracket() -> RunExplicitBoundary<
	'static,
	RunExplicitFirstRow,
	RunExplicitBracketRow,
	RunExplicitBracketSBrand,
	Here,
	i32,
	i32,
	impl Fn(i32) -> RunExplicitBracketProg + 'static,
> {
	let acquire: RunExplicitAcquireProg = RunExplicit::pure(7);
	RunExplicit::<'static, RunExplicitFirstRow, RunExplicitBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: Box<i32>| RunExplicit::pure((*resource, 42)),
		|_resource: Box<i32>| RunExplicit::pure(()),
	)
}

fn dispatch_run_explicit_bracket_boundary<K>(
	boundary: RunExplicitBoundary<
		'static,
		RunExplicitFirstRow,
		RunExplicitBracketRow,
		RunExplicitBracketSBrand,
		Here,
		i32,
		i32,
		K,
	>
) -> RunExplicitBracketProg
where
	K: Fn(i32) -> RunExplicitBracketProg + 'static, {
	bracket_handler().dispatch_run_explicit_bracket_boundary(boundary, &handlers! {})
}

#[test]
fn run_explicit_t1_bracket_boundary_dispatches_body_result() {
	let program = dispatch_run_explicit_bracket_boundary(make_run_explicit_bracket());

	assert!(matches!(program.peel(), Ok(42)));
}

#[test]
fn run_explicit_t2_bracket_boundary_map_runs_after_release() {
	let boundary = make_run_explicit_bracket().map(|value| value + 1);
	let program = dispatch_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn run_explicit_t3_bracket_boundary_bind_runs_after_release() {
	let boundary = make_run_explicit_bracket().bind(|value| RunExplicit::pure(value + 1));
	let program = dispatch_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn run_explicit_t4_bracket_boundary_runs_lifecycle_before_outer_continuation() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);
	let outer_events = std::rc::Rc::clone(&events);

	let acquire: RunExplicitAcquireProg = RunExplicit::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RunExplicit::pure(resource)
	});
	let program =
		RunExplicit::<'static, RunExplicitFirstRow, RunExplicitBracketRow, i32>::bracket::<i32, _>(
			acquire,
			move |resource: Box<i32>| {
				push_rc_event(&body_events, "body");
				RunExplicit::pure((*resource, *resource + 35))
			},
			move |resource: Box<i32>| {
				push_rc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				RunExplicit::pure(())
			},
		);
	let program = dispatch_run_explicit_bracket_boundary(program.bind(move |value| {
		push_rc_event(&outer_events, "outer");
		RunExplicit::pure(value)
	}));

	assert!(matches!(program.peel(), Ok(42)));
	assert_rc_events(&events, &["acquire", "body", "release", "outer"]);
}

#[test]
fn run_explicit_t5_bracket_boundary_handle_uses_facade() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);
	let outer_events = std::rc::Rc::clone(&events);

	let acquire: RunExplicitAcquireProg = RunExplicit::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RunExplicit::pure(resource)
	});
	let boundary =
		RunExplicit::<'static, RunExplicitFirstRow, RunExplicitBracketRow, i32>::bracket::<i32, _>(
			acquire,
			move |resource: Box<i32>| {
				push_rc_event(&body_events, "body");
				RunExplicit::pure((*resource, *resource + 35))
			},
			move |resource: Box<i32>| {
				push_rc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				RunExplicit::pure(())
			},
		)
		.bind(move |value| {
			push_rc_event(&outer_events, "outer");
			RunExplicit::pure(value)
		});

	let result = boundary.handle(
		handlers! {},
		scoped_handlers! {
			BoxBracketExplicitBrand<BoxBrand, NodeBrand<RunExplicitFirstRow, RunExplicitBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_rc_events(&events, &["acquire", "body", "release", "outer"]);
}

// -- RcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunExplicitBracketRow;

type RcRunExplicitUnderlyingRow = CoproductBrand<
	BracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, RcRunExplicitBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunExplicitBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunExplicitBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunExplicitBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunExplicitUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunExplicitFirstRow = CNilBrand;
type RcRunExplicitBracketSBrand = BracketExplicitBrand<
	RcBrand,
	NodeBrand<RcRunExplicitFirstRow, RcRunExplicitBracketRow>,
	i32,
	i32,
>;
type RcRunExplicitAcquireProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, i32>;
type RcRunExplicitBracketProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, i32>;
fn make_rc_run_explicit_bracket() -> RcRunExplicitBoundary<
	'static,
	RcRunExplicitFirstRow,
	RcRunExplicitBracketRow,
	RcRunExplicitBracketSBrand,
	Here,
	i32,
	i32,
	impl Fn(i32) -> RcRunExplicitBracketProg + 'static,
> {
	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7);
	RcRunExplicit::<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRunExplicit::pure((*resource, 42)),
		|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
	)
}

fn dispatch_rc_run_explicit_bracket_boundary<K>(
	boundary: RcRunExplicitBoundary<
		'static,
		RcRunExplicitFirstRow,
		RcRunExplicitBracketRow,
		RcRunExplicitBracketSBrand,
		Here,
		i32,
		i32,
		K,
	>
) -> RcRunExplicitBracketProg
where
	K: Fn(i32) -> RcRunExplicitBracketProg + 'static, {
	bracket_handler().dispatch_rc_run_explicit_bracket_boundary(boundary, &handlers! {})
}

#[test]
fn rc_run_explicit_t1_bracket_boundary_dispatches_body_result() {
	let program = dispatch_rc_run_explicit_bracket_boundary(make_rc_run_explicit_bracket());

	assert!(matches!(program.peel(), Ok(42)));
}

#[test]
fn rc_run_explicit_t2_bracket_boundary_map_runs_after_release() {
	let boundary = make_rc_run_explicit_bracket().map(|value| value + 1);
	let program = dispatch_rc_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn rc_run_explicit_t3_bracket_boundary_bind_runs_after_release() {
	let boundary = make_rc_run_explicit_bracket().bind(|value| RcRunExplicit::pure(value + 1));
	let program = dispatch_rc_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn rc_run_explicit_t4_bracket_boundary_runs_lifecycle_before_outer_continuation() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);
	let outer_events = std::rc::Rc::clone(&events);

	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RcRunExplicit::pure(resource)
	});
	let boundary =
		RcRunExplicit::<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, i32>::bracket::<
			i32,
			_,
		>(
			acquire,
			move |resource: std::rc::Rc<i32>| {
				push_rc_event(&body_events, "body");
				RcRunExplicit::pure((*resource, *resource + 35))
			},
			move |resource: std::rc::Rc<i32>| {
				push_rc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				RcRunExplicit::pure(())
			},
		)
		.bind(move |value| {
			push_rc_event(&outer_events, "outer");
			RcRunExplicit::pure(value)
		});
	let program = dispatch_rc_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(42)));
	assert_rc_events(&events, &["acquire", "body", "release", "outer"]);
}

#[test]
fn rc_run_explicit_t5_bracket_boundary_handle_uses_facade() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);
	let outer_events = std::rc::Rc::clone(&events);

	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RcRunExplicit::pure(resource)
	});
	let boundary =
		RcRunExplicit::<'static, RcRunExplicitFirstRow, RcRunExplicitBracketRow, i32>::bracket::<
			i32,
			_,
		>(
			acquire,
			move |resource: std::rc::Rc<i32>| {
				push_rc_event(&body_events, "body");
				RcRunExplicit::pure((*resource, *resource + 35))
			},
			move |resource: std::rc::Rc<i32>| {
				push_rc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				RcRunExplicit::pure(())
			},
		)
		.bind(move |value| {
			push_rc_event(&outer_events, "outer");
			RcRunExplicit::pure(value)
		});

	let result = boundary.handle(
		handlers! {},
		scoped_handlers! {
			BracketExplicitBrand<RcBrand, NodeBrand<RcRunExplicitFirstRow, RcRunExplicitBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_rc_events(&events, &["acquire", "body", "release", "outer"]);
}

// -- ArcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunBracketRow;

impl_kind! {
	impl for ArcRunBracketRow {
		type Of<'a, A: 'a>: 'a =
			Coproduct<SendBracket<'a, ArcBrand, NodeBrand<CNilBrand, ArcRunBracketRow>, i32, i32>, CNil>;
	}
}

impl WrapDrop for ArcRunBracketRow {
	fn drop<'a, X: 'a>(
		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		None
	}
}

impl SendFunctor for ArcRunBracketRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		_f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			Coproduct::Inl(layer) => Coproduct::Inl(layer),
			Coproduct::Inr(remainder) => match remainder {},
		}
	}
}

type ArcRunFirstRow = CNilBrand;
type ArcRunAcquireProg = ArcRun<ArcRunFirstRow, ArcRunBracketRow, i32>;
type ArcRunBracketProg = ArcRun<ArcRunFirstRow, ArcRunBracketRow, i32>;
type ArcRunBracketBodyProg = ArcRun<ArcRunFirstRow, ArcRunBracketRow, (i32, i32)>;
type ArcRunReleaseProg = ArcRun<ArcRunFirstRow, ArcRunBracketRow, ()>;

fn make_arc_run_bracket() -> ArcRunBracketProg {
	let acquire: ArcRunAcquireProg = ArcRun::pure(7);
	ArcRun::<ArcRunFirstRow, ArcRunBracketRow, i32>::bracket::<i32, _>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRun::pure((*resource, 42)),
		|_resource: std::sync::Arc<i32>| ArcRun::pure(()),
	)
}

#[test]
fn arc_run_t1_bracket_produces_scoped_layer() {
	match make_arc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendBracket::Bracket))"),
	}
}

#[test]
fn arc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_arc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: ArcRunAcquireProg = ArcRun::from_arc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn arc_run_t3_body_materialises_paired_program() {
	match make_arc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracket::Bracket {
			body, ..
		}))) => {
			let materialised: ArcRunBracketBodyProg =
				ArcRun::from_arc_free(body(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok((7, 42))));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn arc_run_t4_release_materialises_unit_program() {
	match make_arc_run_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendBracket::Bracket {
			release, ..
		}))) => {
			let materialised: ArcRunReleaseProg =
				ArcRun::from_arc_free(release(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped bracket layer"),
	}
}

#[test]
fn arc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_arc_run_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: ArcRunBracketProg| -> ArcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(SendBracket::Bracket {
				acquire, ..
			}))) => ArcRun::from_arc_free(acquire(())),
			_ => panic!("expected scoped bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

#[test]
fn arc_run_t6_bracket_handler_runs_lifecycle_in_order() {
	let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
	let acquire_events = std::sync::Arc::clone(&events);
	let body_events = std::sync::Arc::clone(&events);
	let release_events = std::sync::Arc::clone(&events);

	let acquire: ArcRunAcquireProg = ArcRun::pure(7).bind(move |resource| {
		push_arc_event(&acquire_events, "acquire");
		ArcRun::pure(resource)
	});
	let program: ArcRunBracketProg =
		ArcRun::<ArcRunFirstRow, ArcRunBracketRow, i32>::bracket::<i32, _>(
			acquire,
			move |resource: std::sync::Arc<i32>| {
				push_arc_event(&body_events, "body");
				ArcRun::pure((*resource, *resource + 35))
			},
			move |resource: std::sync::Arc<i32>| {
				push_arc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				ArcRun::pure(())
			},
		);

	let result = program.handle(
		handlers! {},
		scoped_handlers! {
			SendBracketBrand<ArcBrand, NodeBrand<ArcRunFirstRow, ArcRunBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_arc_events(&events, &["acquire", "body", "release"]);
}

// -- ArcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunExplicitBracketRow;

type ArcRunExplicitUnderlyingRow = CoproductBrand<
	SendBracketExplicitBrand<ArcBrand, NodeBrand<CNilBrand, ArcRunExplicitBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for ArcRunExplicitBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<ArcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for ArcRunExplicitBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<ArcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl SendFunctor for ArcRunExplicitBracketRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<ArcRunExplicitUnderlyingRow as SendFunctor>::send_map(f, fa)
	}
}

type ArcRunExplicitFirstRow = CNilBrand;
type ArcRunExplicitBracketSBrand = SendBracketExplicitBrand<
	ArcBrand,
	NodeBrand<ArcRunExplicitFirstRow, ArcRunExplicitBracketRow>,
	i32,
	i32,
>;
type ArcRunExplicitAcquireProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, i32>;
type ArcRunExplicitBracketProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, i32>;
fn make_arc_run_explicit_bracket() -> ArcRunExplicitBoundary<
	'static,
	ArcRunExplicitFirstRow,
	ArcRunExplicitBracketRow,
	ArcRunExplicitBracketSBrand,
	Here,
	i32,
	i32,
	impl Fn(i32) -> ArcRunExplicitBracketProg + Send + Sync + 'static,
> {
	let acquire: ArcRunExplicitAcquireProg = ArcRunExplicit::pure(7);
	ArcRunExplicit::<'static, ArcRunExplicitFirstRow, ArcRunExplicitBracketRow, i32>::bracket::<
		i32,
		_,
	>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRunExplicit::pure((*resource, 42)),
		|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
	)
}

fn dispatch_arc_run_explicit_bracket_boundary<K>(
	boundary: ArcRunExplicitBoundary<
		'static,
		ArcRunExplicitFirstRow,
		ArcRunExplicitBracketRow,
		ArcRunExplicitBracketSBrand,
		Here,
		i32,
		i32,
		K,
	>
) -> ArcRunExplicitBracketProg
where
	K: Fn(i32) -> ArcRunExplicitBracketProg + Send + Sync + 'static, {
	bracket_handler().dispatch_arc_run_explicit_bracket_boundary(boundary, &handlers! {})
}

#[test]
fn arc_run_explicit_t1_bracket_boundary_dispatches_body_result() {
	let program = dispatch_arc_run_explicit_bracket_boundary(make_arc_run_explicit_bracket());

	assert!(matches!(program.peel(), Ok(42)));
}

#[test]
fn arc_run_explicit_t2_bracket_boundary_map_runs_after_release() {
	let boundary = make_arc_run_explicit_bracket().map(|value| value + 1);
	let program = dispatch_arc_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn arc_run_explicit_t3_bracket_boundary_bind_runs_after_release() {
	let boundary = make_arc_run_explicit_bracket().bind(|value| ArcRunExplicit::pure(value + 1));
	let program = dispatch_arc_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn arc_run_explicit_t4_bracket_boundary_runs_lifecycle_before_outer_continuation() {
	let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
	let acquire_events = std::sync::Arc::clone(&events);
	let body_events = std::sync::Arc::clone(&events);
	let release_events = std::sync::Arc::clone(&events);
	let outer_events = std::sync::Arc::clone(&events);

	let acquire: ArcRunExplicitAcquireProg = ArcRunExplicit::pure(7).bind(move |resource| {
		push_arc_event(&acquire_events, "acquire");
		ArcRunExplicit::pure(resource)
	});
	let boundary = ArcRunExplicit::<
		'static,
		ArcRunExplicitFirstRow,
		ArcRunExplicitBracketRow,
		i32,
	>::bracket::<i32, _>(
		acquire,
		move |resource: std::sync::Arc<i32>| {
			push_arc_event(&body_events, "body");
			ArcRunExplicit::pure((*resource, *resource + 35))
		},
		move |resource: std::sync::Arc<i32>| {
			push_arc_event(&release_events, "release");
			assert_eq!(*resource, 7);
			ArcRunExplicit::pure(())
		},
	)
	.bind(move |value| {
		push_arc_event(&outer_events, "outer");
		ArcRunExplicit::pure(value)
	});
	let program = dispatch_arc_run_explicit_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(42)));
	assert_arc_events(&events, &["acquire", "body", "release", "outer"]);
}

#[test]
fn arc_run_explicit_t5_bracket_boundary_handle_uses_facade() {
	let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
	let acquire_events = std::sync::Arc::clone(&events);
	let body_events = std::sync::Arc::clone(&events);
	let release_events = std::sync::Arc::clone(&events);
	let outer_events = std::sync::Arc::clone(&events);

	let acquire: ArcRunExplicitAcquireProg = ArcRunExplicit::pure(7).bind(move |resource| {
		push_arc_event(&acquire_events, "acquire");
		ArcRunExplicit::pure(resource)
	});
	let boundary = ArcRunExplicit::<
		'static,
		ArcRunExplicitFirstRow,
		ArcRunExplicitBracketRow,
		i32,
	>::bracket::<i32, _>(
		acquire,
		move |resource: std::sync::Arc<i32>| {
			push_arc_event(&body_events, "body");
			ArcRunExplicit::pure((*resource, *resource + 35))
		},
		move |resource: std::sync::Arc<i32>| {
			push_arc_event(&release_events, "release");
			assert_eq!(*resource, 7);
			ArcRunExplicit::pure(())
		},
	)
	.bind(move |value| {
		push_arc_event(&outer_events, "outer");
		ArcRunExplicit::pure(value)
	});

	let result = boundary.handle(
		handlers! {},
		scoped_handlers! {
			SendBracketExplicitBrand<ArcBrand, NodeBrand<ArcRunExplicitFirstRow, ArcRunExplicitBracketRow>, i32, i32>: bracket_handler(),
		},
	);

	assert_eq!(result, 42);
	assert_arc_events(&events, &["acquire", "body", "release", "outer"]);
}
