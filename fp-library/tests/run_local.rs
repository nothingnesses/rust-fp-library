#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration tests for the scoped `local<E, Idx>` smart constructor
// across the Run-wrapper family. The default, Rc, Arc, RcExplicit, and
// ArcExplicit sections verify substrate shape:
//   T1: `local(modify, action)` produces a program suspended at a
//       `Node::Scoped` layer carrying a `BoxLocal` / `Local` /
//       `SendLocal` cell projected via `Member::inject` at the head
//       of the scoped row.
//   T2: invoking the cell's stored `action` thunk (with the unit
//       argument) yields a wrapper-typed program whose `peel`
//       returns the original `Pure` payload.
//   T3: invoking the cell's stored `modify` closure with a sample
//       environment value yields the expected transformed value.
//   T4 (clone-able wrappers only): cloning the suspended program
//       produces two independent peelable handles; each clone's
//       action thunk materialises to the original action.
//
// The single-shot `RunExplicit` section exercises the indexed boundary
// returned by `RunExplicit::local`: dispatcher application transforms
// the Reader environment before the selected action runs, and any
// mapped or bound outer continuation runs after the action result.

use fp_library::{
	brands::{
		ArcBrand,
		ArcCoyonedaBrand,
		BoxBrand,
		BoxLocalBrand,
		BoxReaderBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
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
		arc_run_explicit::ArcRunExplicit,
		coproduct::Coproduct,
		local::{
			BoxLocal,
			Local,
			SendLocal,
		},
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
		scoped_dispatchers::local_dispatcher,
	},
};

// -- Run --

// Default-substrate scoped-effect row: a single `BoxLocal` cell over
// the `BoxBrand` pointer carrying `i32` environments. The first-order
// row is left empty because the tests exercise only the scoped layer;
// populating it with `IdentityBrand`-style transparent wrappers would
// force a `Free` layout cycle (no pointer indirection on the variant
// payload), which the existing substrate doctests sidestep the same
// way.
type RunScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
type RunFirstRow = CNilBrand;
type RunProg = Run<RunFirstRow, RunScopedRow, i32>;

#[test]
fn run_t1_local_produces_scoped_layer() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxLocal::Local))"),
	}
}

#[test]
fn run_t2_action_thunk_materialises_action_program() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxLocal::Local {
			action, ..
		}))) => {
			let materialised: RunProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn run_t3_modify_transforms_environment() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(10), 11);
		}
		_ => panic!("expected scoped local layer"),
	}
}

// -- RcRun --

type RcScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
type RcFirstRow = CNilBrand;
type RcProg = RcRun<RcFirstRow, RcScopedRow, i32>;

#[test]
fn rc_run_t1_local_produces_scoped_layer() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Local::Local))"),
	}
}

#[test]
fn rc_run_t2_action_thunk_materialises_action_program() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			action, ..
		}))) => {
			let materialised: RcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn rc_run_t3_modify_transforms_environment() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(10), 11);
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn rc_run_t4_clone_yields_two_independent_peels() {
	// Multi-shot Rc-backed cell: cloning the suspended program
	// produces two peelable handles whose action thunks both
	// materialise to the original `Pure(42)` action.
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::local::<i32, _>(|e: i32| e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: RcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRun --

type ArcScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
type ArcFirstRow = CNilBrand;
type ArcProg = ArcRun<ArcFirstRow, ArcScopedRow, i32>;

#[test]
fn arc_run_t1_local_produces_scoped_layer() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendLocal::Local))"),
	}
}

#[test]
fn arc_run_t2_action_thunk_materialises_action_program() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			action, ..
		}))) => {
			let materialised: ArcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn arc_run_t3_modify_transforms_environment() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(10), 11);
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn arc_run_t4_clone_yields_two_independent_peels() {
	// Multi-shot Arc-backed cell, thread-safe: cloning the suspended
	// program produces two peelable handles whose action thunks both
	// materialise to the original `Pure(42)` action. Mirrors
	// `rc_run_t4` with Send+Sync threading.
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::local::<i32, _>(|e: i32| e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: ArcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

type RcHandledLocalFirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcHandledLocalFirstRowMinusReader = CNilBrand;
type RcHandledLocalScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
type RcHandledLocalProg = RcRun<RcHandledLocalFirstRow, RcHandledLocalScopedRow, i32>;

fn interpret_rc_handled_local(program: RcHandledLocalProg) -> i32 {
	program.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcHandledLocalProg>| match op {
				Reader::Ask(k) => k(20),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcHandledLocalFirstRowMinusReader, _>(),
		},
	)
}

type ArcHandledLocalFirstRow =
	CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type ArcHandledLocalFirstRowMinusReader = CNilBrand;
type ArcHandledLocalScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
type ArcHandledLocalProg = ArcRun<ArcHandledLocalFirstRow, ArcHandledLocalScopedRow, i32>;

fn interpret_arc_handled_local(program: ArcHandledLocalProg) -> i32 {
	program.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcHandledLocalProg>| match op {
				SendReader::Ask(k) => k(30),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, ArcHandledLocalFirstRowMinusReader, _>(),
		},
	)
}

#[test]
fn shared_wrappers_repeat_local_after_outer_map_without_type_erasure_mismatch() {
	// The Local action is selected before the outer map continuation is
	// reattached. Running the same Rc/Arc program twice checks that raw
	// Local dispatch rewrites Reader asks at the selected action result
	// type and does not leave a stale erased-result continuation behind.
	let rc_program: RcHandledLocalProg =
		RcRun::local::<i32, _>(|env| env + 1, RcRun::ask()).map(|value| value * 2);
	assert_eq!(interpret_rc_handled_local(rc_program.clone()), 42);
	assert_eq!(interpret_rc_handled_local(rc_program), 42);

	let arc_program: ArcHandledLocalProg =
		ArcRun::local::<i32, _>(|env| env + 1, ArcRun::ask()).map(|value| value + 11);
	assert_eq!(interpret_arc_handled_local(arc_program.clone()), 42);
	assert_eq!(interpret_arc_handled_local(arc_program), 42);
}

// -- RunExplicit --

type RxScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
type RxFirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type RxFirstRowMinusReader = CNilBrand;
type RxProg = RunExplicit<'static, RxFirstRow, RxScopedRow, i32>;

#[test]
fn run_explicit_t1_local_boundary_uses_modified_environment() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary = RunExplicit::local::<i32, _>(|e: i32| e + 1, action);

	let prog: RxProg = local_dispatcher::<_, RxFirstRowMinusReader, _>()
		.dispatch_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn run_explicit_t2_local_boundary_map_runs_after_action() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary = RunExplicit::local::<i32, _>(|e: i32| e + 1, action).map(|value| value + 1);

	let prog: RxProg = local_dispatcher::<_, RxFirstRowMinusReader, _>()
		.dispatch_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 23);
}

#[test]
fn run_explicit_t3_local_boundary_bind_runs_after_action() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary = RunExplicit::local::<i32, _>(|e: i32| e + 1, action)
		.bind(|value| RunExplicit::pure(value + 20));

	let prog: RxProg = local_dispatcher::<_, RxFirstRowMinusReader, _>()
		.dispatch_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_explicit_t4_local_boundary_interpret_uses_facade() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary = RunExplicit::local::<i32, _>(|e: i32| e + 1, action).map(|value| value + 1);

	let result = boundary.interpret(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_dispatcher::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 23);
}

// -- RcRunExplicit --

type RcxScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
type RcxFirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcxFirstRowMinusReader = CNilBrand;
type RcxProg = RcRunExplicit<'static, RcxFirstRow, RcxScopedRow, i32>;

#[test]
fn rc_run_explicit_t1_local_boundary_uses_modified_environment() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);

	let prog: RcxProg = local_dispatcher::<_, RcxFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn rc_run_explicit_t2_local_boundary_map_runs_after_action() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action).map(|value| value + 1);

	let prog: RcxProg = local_dispatcher::<_, RcxFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 23);
}

#[test]
fn rc_run_explicit_t3_local_boundary_bind_runs_after_action() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action)
		.bind(|value| RcRunExplicit::pure(value + 20));

	let prog: RcxProg = local_dispatcher::<_, RcxFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_explicit_t4_local_boundary_interpret_uses_facade() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action).map(|value| value + 1);

	let result = boundary.interpret(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			LocalBrand<RcBrand, i32>: local_dispatcher::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 23);
}

// -- ArcRunExplicit --

type AcxScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
type AcxFirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type AcxFirstRowMinusReader = CNilBrand;
type AcxProg = ArcRunExplicit<'static, AcxFirstRow, AcxScopedRow, i32>;

#[test]
fn arc_run_explicit_t1_local_boundary_uses_modified_environment() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);

	let prog: AcxProg = local_dispatcher::<_, AcxFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn arc_run_explicit_t2_local_boundary_map_runs_after_action() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action).map(|value| value + 1);

	let prog: AcxProg = local_dispatcher::<_, AcxFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 23);
}

#[test]
fn arc_run_explicit_t3_local_boundary_bind_runs_after_action() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action)
		.bind(|value| ArcRunExplicit::pure(value + 20));

	let prog: AcxProg = local_dispatcher::<_, AcxFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_local_boundary(boundary, &handlers! {});
	let result = prog.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_explicit_t4_local_boundary_interpret_uses_facade() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action).map(|value| value + 1);

	let result = boundary.interpret(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendLocalBrand<ArcBrand, i32>: local_dispatcher::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 23);
}
