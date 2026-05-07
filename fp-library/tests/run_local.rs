#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Shape-only integration tests for the substrate-level scoped
// `local<E, Idx>` smart constructor across the Run-wrapper family.
// Each wrapper's section verifies:
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
// End-to-end environment-modification semantics (the dispatcher
// applying `modify` to the inherited environment value before
// invoking `action` via `interpret_with_either`) are not exercised
// here. That path comes online when the scoped-handler dispatch
// protocol and the standard reader handler are in place; this file
// restricts itself to verifying that the substrate produces the
// expected suspended shape and that both thunks fire.

use fp_library::{
	brands::{
		ArcBrand,
		BoxBrand,
		BoxLocalBrand,
		CNilBrand,
		CoproductBrand,
		LocalBrand,
		RcBrand,
		SendLocalBrand,
	},
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
		run::Run,
		run_explicit::RunExplicit,
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

// -- RunExplicit --

type RxScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
type RxFirstRow = CNilBrand;
type RxProg = RunExplicit<'static, RxFirstRow, RxScopedRow, i32>;

#[test]
fn run_explicit_t1_local_produces_scoped_layer() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxLocal::Local))"),
	}
}

#[test]
fn run_explicit_t2_action_thunk_materialises_action_program() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxLocal::Local {
			action, ..
		}))) => {
			let materialised: RxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn run_explicit_t3_modify_transforms_environment() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxLocal::Local {
			modify, ..
		}))) => {
			assert_eq!(modify(10), 11);
		}
		_ => panic!("expected scoped local layer"),
	}
}

// -- RcRunExplicit --

type RcxScopedRow = CoproductBrand<LocalBrand<RcBrand, i32>, CNilBrand>;
type RcxFirstRow = CNilBrand;
type RcxProg = RcRunExplicit<'static, RcxFirstRow, RcxScopedRow, i32>;

#[test]
fn rc_run_explicit_t1_local_produces_scoped_layer() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Local::Local))"),
	}
}

#[test]
fn rc_run_explicit_t2_action_thunk_materialises_action_program() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			action, ..
		}))) => {
			let materialised: RcxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn rc_run_explicit_t3_modify_transforms_environment() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
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
fn rc_run_explicit_t4_clone_yields_two_independent_peels() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg = RcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: RcxProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(Local::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRunExplicit --

type AcxScopedRow = CoproductBrand<SendLocalBrand<ArcBrand, i32>, CNilBrand>;
type AcxFirstRow = CNilBrand;
type AcxProg = ArcRunExplicit<'static, AcxFirstRow, AcxScopedRow, i32>;

#[test]
fn arc_run_explicit_t1_local_produces_scoped_layer() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendLocal::Local))"),
	}
}

#[test]
fn arc_run_explicit_t2_action_thunk_materialises_action_program() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			action, ..
		}))) => {
			let materialised: AcxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped local layer"),
	}
}

#[test]
fn arc_run_explicit_t3_modify_transforms_environment() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
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
fn arc_run_explicit_t4_clone_yields_two_independent_peels() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg = ArcRunExplicit::local::<i32, _>(|e: i32| e + 1, action);
	let prog_clone = prog.clone();

	let extract = |p: AcxProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendLocal::Local {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped local layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}
