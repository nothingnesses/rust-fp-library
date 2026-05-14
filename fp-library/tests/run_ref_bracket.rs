#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]
#![recursion_limit = "512"]

// Integration tests for the scoped `ref_bracket<A, Idx>` smart
// constructor across the refcounted Run-wrapper family. `RcRun` and
// `ArcRun` keep direct program-returning constructors; their sections
// verify substrate shape:
//   T1: `ref_bracket(acquire, body, release)` produces a program
//       suspended at a `Node::Scoped` layer carrying a `RefBracket` /
//       `SendRefBracket` cell projected via `Member::inject` at the head
//       of the scoped row.
//   T2: invoking the cell's stored `acquire` thunk (with the unit
//       argument) yields a substrate-typed program that peels to the
//       original resource value after wrapping with the wrapper's
//       `from_*` constructor.
//   T3: invoking the cell's stored `body` closure with a sample
//       refcounted resource pointer yields a substrate-typed program
//       that peels to the expected body result.
//   T4: invoking the cell's stored `release` closure with a sample
//       refcounted resource pointer yields a substrate-typed program
//       that peels to `()`.
//   T5: cloning the suspended program produces two independent
//       peelable handles; each clone's acquire thunk materialises to
//       the original resource value.
//   T6: standard `RefBracketDispatcher` interpretation runs acquire,
//       body, and release in order; body and release receive resource
//       pointer clones, and the dispatcher returns the body result after
//       release.
//
// `RcRunExplicit` and `ArcRunExplicit` return indexed boundaries instead
// of direct suspended programs. Their sections dispatch those boundaries
// through the standard `RefBracketDispatcher` methods and verify body
// result return, map/bind placement after release, pointer-clone
// semantics, repeated Rc use, and Arc `Send + Sync` obligations.

use fp_library::{
	Apply,
	brands::{
		ArcBrand,
		CNilBrand,
		CoproductBrand,
		NodeBrand,
		RcBrand,
		RefBracketBrand,
		RefBracketExplicitBrand,
		SendRefBracketBrand,
		SendRefBracketExplicitBrand,
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
		coproduct::{
			CNil,
			Coproduct,
		},
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::{
			RcRunExplicit,
			RcRunExplicitBoundary,
		},
		ref_bracket::{
			RefBracket,
			SendRefBracket,
		},
		scoped_dispatchers::ref_bracket_dispatcher,
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

// -- RcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunRefBracketRow;

type RcRunUnderlyingRow = CoproductBrand<
	RefBracketBrand<RcBrand, NodeBrand<CNilBrand, RcRunRefBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunRefBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunFirstRow = CNilBrand;
type RcRunAcquireProg = RcRun<RcRunFirstRow, RcRunRefBracketRow, i32>;
type RcRunRefBracketProg = RcRun<RcRunFirstRow, RcRunRefBracketRow, i32>;
type RcRunReleaseProg = RcRun<RcRunFirstRow, RcRunRefBracketRow, ()>;

fn make_rc_run_ref_bracket() -> RcRunRefBracketProg {
	let acquire: RcRunAcquireProg = RcRun::pure(7);
	RcRun::<RcRunFirstRow, RcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRun::pure(*resource + 35),
		|_resource: std::rc::Rc<i32>| RcRun::pure(()),
	)
}

#[test]
fn rc_run_t1_ref_bracket_produces_scoped_layer() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(RefBracket::Bracket))"),
	}
}

#[test]
fn rc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: RcRunAcquireProg = RcRun::from_rc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_t3_body_materialises_body_program() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			body, ..
		}))) => {
			let materialised: RcRunRefBracketProg = RcRun::from_rc_free(body(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_t4_release_materialises_unit_program() {
	match make_rc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
			release, ..
		}))) => {
			let materialised: RcRunReleaseProg = RcRun::from_rc_free(release(std::rc::Rc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn rc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_rc_run_ref_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: RcRunRefBracketProg| -> RcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(RefBracket::Bracket {
				acquire, ..
			}))) => RcRun::from_rc_free(acquire(())),
			_ => panic!("expected scoped ref-bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

#[test]
fn rc_run_t6_ref_bracket_dispatcher_runs_lifecycle_in_order() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);

	let acquire: RcRunAcquireProg = RcRun::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RcRun::pure(resource)
	});
	let program: RcRunRefBracketProg =
		RcRun::<RcRunFirstRow, RcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
			acquire,
			move |resource: std::rc::Rc<i32>| {
				push_rc_event(&body_events, "body");
				RcRun::pure(*resource + 35)
			},
			move |resource: std::rc::Rc<i32>| {
				push_rc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				RcRun::pure(())
			},
		);

	let result = program.interpret(
		handlers! {},
		scoped_handlers! {
			RefBracketBrand<RcBrand, NodeBrand<RcRunFirstRow, RcRunRefBracketRow>, i32, i32>: ref_bracket_dispatcher(),
		},
	);

	assert_eq!(result, 42);
	assert_rc_events(&events, &["acquire", "body", "release"]);
}

// -- ArcRun --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunRefBracketRow;

impl_kind! {
	impl for ArcRunRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Coproduct<SendRefBracket<'a, ArcBrand, NodeBrand<CNilBrand, ArcRunRefBracketRow>, i32, i32>, CNil>;
	}
}

impl WrapDrop for ArcRunRefBracketRow {
	fn drop<'a, X: 'a>(
		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		None
	}
}

impl SendFunctor for ArcRunRefBracketRow {
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
type ArcRunAcquireProg = ArcRun<ArcRunFirstRow, ArcRunRefBracketRow, i32>;
type ArcRunRefBracketProg = ArcRun<ArcRunFirstRow, ArcRunRefBracketRow, i32>;
type ArcRunReleaseProg = ArcRun<ArcRunFirstRow, ArcRunRefBracketRow, ()>;

fn make_arc_run_ref_bracket() -> ArcRunRefBracketProg {
	let acquire: ArcRunAcquireProg = ArcRun::pure(7);
	ArcRun::<ArcRunFirstRow, ArcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRun::pure(*resource + 35),
		|_resource: std::sync::Arc<i32>| ArcRun::pure(()),
	)
}

#[test]
fn arc_run_t1_ref_bracket_produces_scoped_layer() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracket::Bracket {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendRefBracket::Bracket))"),
	}
}

#[test]
fn arc_run_t2_acquire_thunk_materialises_resource_program() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracket::Bracket {
			acquire, ..
		}))) => {
			let materialised: ArcRunAcquireProg = ArcRun::from_arc_free(acquire(()));
			assert!(matches!(materialised.peel(), Ok(7)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_t3_body_materialises_body_program() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracket::Bracket {
			body, ..
		}))) => {
			let materialised: ArcRunRefBracketProg =
				ArcRun::from_arc_free(body(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_t4_release_materialises_unit_program() {
	match make_arc_run_ref_bracket().peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefBracket::Bracket {
			release, ..
		}))) => {
			let materialised: ArcRunReleaseProg =
				ArcRun::from_arc_free(release(std::sync::Arc::new(7)));
			assert!(matches!(materialised.peel(), Ok(())));
		}
		_ => panic!("expected scoped ref-bracket layer"),
	}
}

#[test]
fn arc_run_t5_clone_yields_two_independent_peels() {
	let prog = make_arc_run_ref_bracket();
	let prog_clone = prog.clone();

	let extract_acquire = |p: ArcRunRefBracketProg| -> ArcRunAcquireProg {
		match p.peel() {
			Err(Node::Scoped(Coproduct::Inl(SendRefBracket::Bracket {
				acquire, ..
			}))) => ArcRun::from_arc_free(acquire(())),
			_ => panic!("expected scoped ref-bracket layer"),
		}
	};
	assert!(matches!(extract_acquire(prog).peel(), Ok(7)));
	assert!(matches!(extract_acquire(prog_clone).peel(), Ok(7)));
}

#[test]
fn arc_run_t6_ref_bracket_dispatcher_runs_lifecycle_in_order() {
	let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
	let acquire_events = std::sync::Arc::clone(&events);
	let body_events = std::sync::Arc::clone(&events);
	let release_events = std::sync::Arc::clone(&events);

	let acquire: ArcRunAcquireProg = ArcRun::pure(7).bind(move |resource| {
		push_arc_event(&acquire_events, "acquire");
		ArcRun::pure(resource)
	});
	let program: ArcRunRefBracketProg =
		ArcRun::<ArcRunFirstRow, ArcRunRefBracketRow, i32>::ref_bracket::<i32, _>(
			acquire,
			move |resource: std::sync::Arc<i32>| {
				push_arc_event(&body_events, "body");
				ArcRun::pure(*resource + 35)
			},
			move |resource: std::sync::Arc<i32>| {
				push_arc_event(&release_events, "release");
				assert_eq!(*resource, 7);
				ArcRun::pure(())
			},
		);

	let result = program.interpret(
		handlers! {},
		scoped_handlers! {
			SendRefBracketBrand<ArcBrand, NodeBrand<ArcRunFirstRow, ArcRunRefBracketRow>, i32, i32>: ref_bracket_dispatcher(),
		},
	);

	assert_eq!(result, 42);
	assert_arc_events(&events, &["acquire", "body", "release"]);
}

// -- RcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcRunExplicitRefBracketRow;

type RcRunExplicitUnderlyingRow = CoproductBrand<
	RefBracketExplicitBrand<RcBrand, NodeBrand<CNilBrand, RcRunExplicitRefBracketRow>, i32, i32>,
	CNilBrand,
>;

impl_kind! {
	impl for RcRunExplicitRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<RcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for RcRunExplicitRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<RcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for RcRunExplicitRefBracketRow {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<RcRunExplicitUnderlyingRow as Functor>::map(f, fa)
	}
}

type RcRunExplicitFirstRow = CNilBrand;
type RcRunExplicitAcquireProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, i32>;
type RcRunExplicitRefBracketProg =
	RcRunExplicit<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, i32>;
fn make_rc_run_explicit_ref_bracket() -> RcRunExplicitBoundary<
	'static,
	RcRunExplicitFirstRow,
	RcRunExplicitRefBracketRow,
	i32,
	i32,
	impl Fn(i32) -> RcRunExplicitRefBracketProg + 'static,
> {
	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7);
	RcRunExplicit::<'static, RcRunExplicitFirstRow, RcRunExplicitRefBracketRow, i32>::ref_bracket::<
		i32,
		_,
	>(
		acquire,
		|resource: std::rc::Rc<i32>| RcRunExplicit::pure(*resource + 35),
		|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
	)
}

fn dispatch_rc_run_explicit_ref_bracket_boundary<K>(
	boundary: RcRunExplicitBoundary<
		'static,
		RcRunExplicitFirstRow,
		RcRunExplicitRefBracketRow,
		i32,
		i32,
		K,
	>
) -> RcRunExplicitRefBracketProg
where
	K: Fn(i32) -> RcRunExplicitRefBracketProg + 'static, {
	ref_bracket_dispatcher().dispatch_rc_run_explicit_ref_bracket_boundary(boundary, &handlers! {})
}

#[test]
fn rc_run_explicit_t1_ref_bracket_boundary_dispatches_body_result() {
	let program = dispatch_rc_run_explicit_ref_bracket_boundary(make_rc_run_explicit_ref_bracket());

	assert!(matches!(program.peel(), Ok(42)));
}

#[test]
fn rc_run_explicit_t2_ref_bracket_boundary_map_runs_after_release() {
	let boundary = make_rc_run_explicit_ref_bracket().map(|value| value + 1);
	let program = dispatch_rc_run_explicit_ref_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn rc_run_explicit_t3_ref_bracket_boundary_bind_runs_after_release() {
	let boundary = make_rc_run_explicit_ref_bracket().bind(|value| RcRunExplicit::pure(value + 1));
	let program = dispatch_rc_run_explicit_ref_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn rc_run_explicit_t4_ref_bracket_boundary_runs_lifecycle_in_order() {
	let events = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
	let acquire_events = std::rc::Rc::clone(&events);
	let body_events = std::rc::Rc::clone(&events);
	let release_events = std::rc::Rc::clone(&events);
	let outer_events = std::rc::Rc::clone(&events);

	let acquire: RcRunExplicitAcquireProg = RcRunExplicit::pure(7).bind(move |resource| {
		push_rc_event(&acquire_events, "acquire");
		RcRunExplicit::pure(resource)
	});
	let boundary = RcRunExplicit::<
		'static,
		RcRunExplicitFirstRow,
		RcRunExplicitRefBracketRow,
		i32,
	>::ref_bracket::<i32, _>(
		acquire,
		move |resource: std::rc::Rc<i32>| {
			push_rc_event(&body_events, "body");
			RcRunExplicit::pure(*resource + 35)
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
	let program = dispatch_rc_run_explicit_ref_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(42)));
	assert_rc_events(&events, &["acquire", "body", "release", "outer"]);
}

// -- ArcRunExplicit --

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcRunExplicitRefBracketRow;

type ArcRunExplicitUnderlyingRow = CoproductBrand<
	SendRefBracketExplicitBrand<
		ArcBrand,
		NodeBrand<CNilBrand, ArcRunExplicitRefBracketRow>,
		i32,
		i32,
	>,
	CNilBrand,
>;

impl_kind! {
	impl for ArcRunExplicitRefBracketRow {
		type Of<'a, A: 'a>: 'a =
			Apply!(<ArcRunExplicitUnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

impl WrapDrop for ArcRunExplicitRefBracketRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<ArcRunExplicitUnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl SendFunctor for ArcRunExplicitRefBracketRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		f: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<ArcRunExplicitUnderlyingRow as SendFunctor>::send_map(f, fa)
	}
}

type ArcRunExplicitFirstRow = CNilBrand;
type ArcRunExplicitAcquireProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, i32>;
type ArcRunExplicitRefBracketProg =
	ArcRunExplicit<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, i32>;
fn make_arc_run_explicit_ref_bracket() -> ArcRunExplicitBoundary<
	'static,
	ArcRunExplicitFirstRow,
	ArcRunExplicitRefBracketRow,
	i32,
	i32,
	impl Fn(i32) -> ArcRunExplicitRefBracketProg + Send + Sync + 'static,
> {
	let acquire: ArcRunExplicitAcquireProg = ArcRunExplicit::pure(7);
	ArcRunExplicit::<'static, ArcRunExplicitFirstRow, ArcRunExplicitRefBracketRow, i32>::ref_bracket::<
		i32,
		_,
	>(
		acquire,
		|resource: std::sync::Arc<i32>| ArcRunExplicit::pure(*resource + 35),
		|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
	)
}

fn dispatch_arc_run_explicit_ref_bracket_boundary<K>(
	boundary: ArcRunExplicitBoundary<
		'static,
		ArcRunExplicitFirstRow,
		ArcRunExplicitRefBracketRow,
		i32,
		i32,
		K,
	>
) -> ArcRunExplicitRefBracketProg
where
	K: Fn(i32) -> ArcRunExplicitRefBracketProg + Send + Sync + 'static, {
	ref_bracket_dispatcher().dispatch_arc_run_explicit_ref_bracket_boundary(boundary, &handlers! {})
}

#[test]
fn arc_run_explicit_t1_ref_bracket_boundary_dispatches_body_result() {
	let program =
		dispatch_arc_run_explicit_ref_bracket_boundary(make_arc_run_explicit_ref_bracket());

	assert!(matches!(program.peel(), Ok(42)));
}

#[test]
fn arc_run_explicit_t2_ref_bracket_boundary_map_runs_after_release() {
	let boundary = make_arc_run_explicit_ref_bracket().map(|value| value + 1);
	let program = dispatch_arc_run_explicit_ref_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn arc_run_explicit_t3_ref_bracket_boundary_bind_runs_after_release() {
	let boundary =
		make_arc_run_explicit_ref_bracket().bind(|value| ArcRunExplicit::pure(value + 1));
	let program = dispatch_arc_run_explicit_ref_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(43)));
}

#[test]
fn arc_run_explicit_t4_ref_bracket_boundary_runs_lifecycle_in_order() {
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
		ArcRunExplicitRefBracketRow,
		i32,
	>::ref_bracket::<i32, _>(
		acquire,
		move |resource: std::sync::Arc<i32>| {
			push_arc_event(&body_events, "body");
			ArcRunExplicit::pure(*resource + 35)
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
	let program = dispatch_arc_run_explicit_ref_bracket_boundary(boundary);

	assert!(matches!(program.peel(), Ok(42)));
	assert_arc_events(&events, &["acquire", "body", "release", "outer"]);
}
