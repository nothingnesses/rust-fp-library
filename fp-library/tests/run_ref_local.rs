#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration tests for the scoped `ref_local<E, Idx>` smart
// constructor across the Run-wrapper family. The default, Rc, Arc,
// RcExplicit, and ArcExplicit sections verify substrate shape:
//   T1: `ref_local(modify, action)` produces a program suspended at
//       a `Node::Scoped` layer carrying a `BoxRefLocal` /
//       `RefLocal` / `SendRefLocal` cell projected via
//       `Member::inject` at the head of the scoped row. The variant
//       tag is uniformly `Local` across all three sibling enums
//       (Val/Ref share variant names; the type tag carries flavour).
//   T2: invoking the cell's stored `action` thunk (with the unit
//       argument) yields a wrapper-typed program whose `peel`
//       returns the original `Pure` payload.
//   T3: invoking the cell's stored `modify` closure with a borrow
//       of a sample environment value yields the expected
//       transformed value (e.g., `modify(&10) == 11` with
//       `|e: &i32| *e + 1`). Differs from the Val cycle's T3 only
//       by passing `&10` instead of `10`.
//   T4 (clone-able wrappers only): cloning the suspended program
//       produces two independent peelable handles; each clone's
//       action thunk materialises to the original action.
//
// The single-shot `RunExplicit` section exercises the indexed boundary
// returned by `RunExplicit::ref_local`: dispatcher application borrows
// the inherited Reader environment to compute the modified environment
// before the selected action runs, and any mapped or bound outer
// continuation runs after the action result.

use fp_library::{
	brands::{
		ArcBrand,
		ArcCoyonedaBrand,
		BoxBrand,
		BoxReaderBrand,
		BoxRefLocalBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
		RcBrand,
		RcCoyonedaBrand,
		ReaderBrand,
		RefLocalBrand,
		SendReaderBrand,
		SendRefLocalBrand,
	},
	handlers,
	scoped_handlers,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		coproduct::Coproduct,
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		reader::{
			BoxReader,
			Reader,
			SendReader,
		},
		ref_local::{
			BoxRefLocal,
			RefLocal,
			SendRefLocal,
		},
		run::Run,
		run_explicit::RunExplicit,
		standard_scoped_handlers::ref_local_handler,
	},
};

// -- Run --

// Default-substrate scoped-effect row: a single `BoxRefLocal` cell
// over the `BoxBrand` pointer carrying `i32` environments. The
// first-order row is left empty because the tests exercise only the
// scoped layer; populating it with `IdentityBrand`-style transparent
// wrappers would force a `Free` layout cycle (no pointer indirection
// on the variant payload), which the existing substrate doctests
// sidestep the same way.
type RunScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
type RunFirstRow = CNilBrand;
type RunProg = Run<RunFirstRow, RunScopedRow, i32>;

#[test]
fn run_t1_ref_local_produces_scoped_layer() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxRefLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxRefLocal::Local))"),
	}
}

#[test]
fn run_t2_action_thunk_materialises_action_program() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxRefLocal::Local {
			action, ..
		}))) => {
			let materialised: RunProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn run_t3_modify_transforms_environment_by_reference() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxRefLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(&10), 11);
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

// -- RcRun --

type RcScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
type RcFirstRow = CNilBrand;
type RcProg = RcRun<RcFirstRow, RcScopedRow, i32>;

#[test]
fn rc_run_t1_ref_local_produces_scoped_layer() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(RefLocal::Local))"),
	}
}

#[test]
fn rc_run_t2_action_thunk_materialises_action_program() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			action, ..
		}))) => {
			let materialised: RcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn rc_run_t3_modify_transforms_environment_by_reference() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(&10), 11);
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn rc_run_t4_clone_yields_two_independent_peels() {
	// Multi-shot Rc-backed cell: cloning the suspended program
	// produces two peelable handles whose action thunks both
	// materialise to the original `Pure(42)` action.
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: RcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped ref-local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRun --

type ArcScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
type ArcFirstRow = CNilBrand;
type ArcProg = ArcRun<ArcFirstRow, ArcScopedRow, i32>;

#[test]
fn arc_run_t1_ref_local_produces_scoped_layer() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendRefLocal::Local))"),
	}
}

#[test]
fn arc_run_t2_action_thunk_materialises_action_program() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			action, ..
		}))) => {
			let materialised: ArcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn arc_run_t3_modify_transforms_environment_by_reference() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(&10), 11);
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn arc_run_t4_clone_yields_two_independent_peels() {
	// Multi-shot Arc-backed cell, thread-safe: cloning the suspended
	// program produces two peelable handles whose action thunks both
	// materialise to the original `Pure(42)` action. Mirrors
	// `rc_run_t4` with Send+Sync threading.
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: ArcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped ref-local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

type RcHandledRefLocalFirstRow =
	CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcHandledRefLocalFirstRowMinusReader = CNilBrand;
type RcHandledRefLocalScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
type RcHandledRefLocalProg = RcRun<RcHandledRefLocalFirstRow, RcHandledRefLocalScopedRow, i32>;

fn handle_rc_handled_ref_local(program: RcHandledRefLocalProg) -> i32 {
	program.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcHandledRefLocalProg>| match op {
				Reader::Ask(k) => k(20),
			},
		},
		scoped_handlers! {
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcHandledRefLocalFirstRowMinusReader, _>(),
		},
	)
}

type ArcHandledRefLocalFirstRow =
	CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type ArcHandledRefLocalFirstRowMinusReader = CNilBrand;
type ArcHandledRefLocalScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
type ArcHandledRefLocalProg = ArcRun<ArcHandledRefLocalFirstRow, ArcHandledRefLocalScopedRow, i32>;

fn handle_arc_handled_ref_local(program: ArcHandledRefLocalProg) -> i32 {
	program.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcHandledRefLocalProg>| match op {
				SendReader::Ask(k) => k(30),
			},
		},
		scoped_handlers! {
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, ArcHandledRefLocalFirstRowMinusReader, _>(),
		},
	)
}

#[test]
fn shared_wrappers_repeat_ref_local_after_outer_map_without_type_erasure_mismatch() {
	// RefLocal computes the modified environment from a borrow of the
	// inherited Reader value, then rewrites Reader asks inside the
	// selected action before the outer map continuation is reattached.
	// Running clones catches stale erased-result continuation queues.
	let rc_program: RcHandledRefLocalProg =
		RcRun::ref_local::<i32, _>(|env| *env + 1, RcRun::ask()).map(|value| value * 2);
	assert_eq!(handle_rc_handled_ref_local(rc_program.clone()), 42);
	assert_eq!(handle_rc_handled_ref_local(rc_program), 42);

	let arc_program: ArcHandledRefLocalProg =
		ArcRun::ref_local::<i32, _>(|env| *env + 1, ArcRun::ask()).map(|value| value + 11);
	assert_eq!(handle_arc_handled_ref_local(arc_program.clone()), 42);
	assert_eq!(handle_arc_handled_ref_local(arc_program), 42);
}

// -- RunExplicit --

type RxScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
type RxFirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type RxFirstRowMinusReader = CNilBrand;
type RxProg = RunExplicit<'static, RxFirstRow, RxScopedRow, i32>;

#[test]
fn run_explicit_t1_ref_local_boundary_uses_modified_environment() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary = RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action);

	let prog: RxProg = ref_local_handler::<_, RxFirstRowMinusReader, _>()
		.dispatch_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn run_explicit_t2_ref_local_boundary_map_runs_after_action() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary =
		RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action).map(|value| value + 1);

	let prog: RxProg = ref_local_handler::<_, RxFirstRowMinusReader, _>()
		.dispatch_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 31);
}

#[test]
fn run_explicit_t3_ref_local_boundary_bind_runs_after_action() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary = RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action)
		.bind(|value| RunExplicit::pure(value + 12));

	let prog: RxProg = ref_local_handler::<_, RxFirstRowMinusReader, _>()
		.dispatch_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn run_explicit_t4_ref_local_boundary_handle_uses_facade() {
	let action: RxProg = RunExplicit::<RxFirstRow, RxScopedRow, i32>::ask::<_>()
		.bind(|env| RunExplicit::pure(env * 2));
	let boundary =
		RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action).map(|value| value + 1);

	let result = boundary.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, RxProg>| match op {
				BoxReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			BoxRefLocalBrand<BoxBrand, i32>: ref_local_handler::<_, RxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 31);
}

// -- RcRunExplicit --

type RcxScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
type RcxFirstRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcxFirstRowMinusReader = CNilBrand;
type RcxProg = RcRunExplicit<'static, RcxFirstRow, RcxScopedRow, i32>;

#[test]
fn rc_run_explicit_t1_ref_local_boundary_uses_modified_environment() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary = RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action);

	let prog: RcxProg = ref_local_handler::<_, RcxFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn rc_run_explicit_t2_ref_local_boundary_map_runs_after_action() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary =
		RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action).map(|value| value + 1);

	let prog: RcxProg = ref_local_handler::<_, RcxFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 31);
}

#[test]
fn rc_run_explicit_t3_ref_local_boundary_bind_runs_after_action() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary = RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action)
		.bind(|value| RcRunExplicit::pure(value + 12));

	let prog: RcxProg = ref_local_handler::<_, RcxFirstRowMinusReader, _>()
		.dispatch_rc_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn rc_run_explicit_t4_ref_local_boundary_handle_uses_facade() {
	let action: RcxProg = RcRunExplicit::<RcxFirstRow, RcxScopedRow, i32>::ask::<_>()
		.bind(|env| RcRunExplicit::pure(env * 2));
	let boundary =
		RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action).map(|value| value + 1);

	let result = boundary.handle(
		handlers! {
			ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcxProg>| match op {
				Reader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			RefLocalBrand<RcBrand, i32>: ref_local_handler::<_, RcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 31);
}

// -- ArcRunExplicit --

type AcxScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
type AcxFirstRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type AcxFirstRowMinusReader = CNilBrand;
type AcxProg = ArcRunExplicit<'static, AcxFirstRow, AcxScopedRow, i32>;

#[test]
fn arc_run_explicit_t1_ref_local_boundary_uses_modified_environment() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary = ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action);

	let prog: AcxProg = ref_local_handler::<_, AcxFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 30);
}

#[test]
fn arc_run_explicit_t2_ref_local_boundary_map_runs_after_action() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary =
		ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action).map(|value| value + 1);

	let prog: AcxProg = ref_local_handler::<_, AcxFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 31);
}

#[test]
fn arc_run_explicit_t3_ref_local_boundary_bind_runs_after_action() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary = ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action)
		.bind(|value| ArcRunExplicit::pure(value + 12));

	let prog: AcxProg = ref_local_handler::<_, AcxFirstRowMinusReader, _>()
		.dispatch_arc_run_explicit_ref_local_boundary(boundary, &handlers! {});
	let result = prog.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 42);
}

#[test]
fn arc_run_explicit_t4_ref_local_boundary_handle_uses_facade() {
	let action: AcxProg = ArcRunExplicit::<AcxFirstRow, AcxScopedRow, i32>::ask::<_>()
		.bind(|env| ArcRunExplicit::pure(env * 2));
	let boundary =
		ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 5, action).map(|value| value + 1);

	let result = boundary.handle(
		handlers! {
			SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, AcxProg>| match op {
				SendReader::Ask(k) => k(10),
			},
		},
		scoped_handlers! {
			SendRefLocalBrand<ArcBrand, i32>: ref_local_handler::<_, AcxFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 31);
}
