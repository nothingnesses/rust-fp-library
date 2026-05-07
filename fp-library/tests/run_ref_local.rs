#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Shape-only integration tests for the substrate-level scoped
// `ref_local<E, Idx>` smart constructor across the Run-wrapper
// family (Ref flavour: modify borrows the environment value).
// Each wrapper's section verifies:
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
// End-to-end environment-modification semantics (the dispatcher
// applying `modify` to a borrow of the inherited environment value
// before invoking `action` via `interpret_with_either`) are not
// exercised here. That path comes online when the scoped-handler
// dispatch protocol and the standard reader handler are in place;
// this file restricts itself to verifying that the substrate
// produces the expected suspended shape and that both thunks fire.

use fp_library::{
	brands::{
		ArcBrand,
		BoxBrand,
		BoxRefLocalBrand,
		CNilBrand,
		CoproductBrand,
		RcBrand,
		RefLocalBrand,
		SendRefLocalBrand,
	},
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		coproduct::Coproduct,
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		ref_local::{
			BoxRefLocal,
			RefLocal,
			SendRefLocal,
		},
		run::Run,
		run_explicit::RunExplicit,
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

// -- RunExplicit --

type RxScopedRow = CoproductBrand<BoxRefLocalBrand<BoxBrand, i32>, CNilBrand>;
type RxFirstRow = CNilBrand;
type RxProg = RunExplicit<'static, RxFirstRow, RxScopedRow, i32>;

#[test]
fn run_explicit_t1_ref_local_produces_scoped_layer() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxRefLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxRefLocal::Local))"),
	}
}

#[test]
fn run_explicit_t2_action_thunk_materialises_action_program() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxRefLocal::Local {
			action, ..
		}))) => {
			let materialised: RxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn run_explicit_t3_modify_transforms_environment_by_reference() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxRefLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(&10), 11);
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

// -- RcRunExplicit --

type RcxScopedRow = CoproductBrand<RefLocalBrand<RcBrand, i32>, CNilBrand>;
type RcxFirstRow = CNilBrand;
type RcxProg = RcRunExplicit<'static, RcxFirstRow, RcxScopedRow, i32>;

#[test]
fn rc_run_explicit_t1_ref_local_produces_scoped_layer() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(RefLocal::Local))"),
	}
}

#[test]
fn rc_run_explicit_t2_action_thunk_materialises_action_program() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			action, ..
		}))) => {
			let materialised: RcxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn rc_run_explicit_t3_modify_transforms_environment_by_reference() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
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
fn rc_run_explicit_t4_clone_yields_two_independent_peels() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: RcxProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(RefLocal::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped ref-local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRunExplicit --

type AcxScopedRow = CoproductBrand<SendRefLocalBrand<ArcBrand, i32>, CNilBrand>;
type AcxFirstRow = CNilBrand;
type AcxProg = ArcRunExplicit<'static, AcxFirstRow, AcxScopedRow, i32>;

#[test]
fn arc_run_explicit_t1_ref_local_produces_scoped_layer() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendRefLocal::Local))"),
	}
}

#[test]
fn arc_run_explicit_t2_action_thunk_materialises_action_program() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			action, ..
		}))) => {
			let materialised: AcxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped ref-local layer"),
	}
}

#[test]
fn arc_run_explicit_t3_modify_transforms_environment_by_reference() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
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
fn arc_run_explicit_t4_clone_yields_two_independent_peels() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::ref_local::<i32, _>(|e: &i32| *e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: AcxProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendRefLocal::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped ref-local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}
