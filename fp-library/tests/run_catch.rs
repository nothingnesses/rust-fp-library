#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Shape-only integration tests for the substrate-level scoped
// `catch<E, Idx>` smart constructor across the Run-wrapper family.
// Each wrapper's section verifies:
//   T1: `catch(action, handler)` produces a program suspended at
//       a `Node::Scoped` layer carrying a `BoxCatch` / `Catch` /
//       `SendCatch` cell projected via `Member::inject` at the
//       head of the scoped row.
//   T2: invoking the cell's stored `action` thunk (with the unit
//       argument) yields a wrapper-typed program whose `peel`
//       returns the original `Pure` payload.
//   T3: invoking the cell's stored `handler` with an error
//       yields the program produced by the user-supplied recovery
//       closure.
//   T4 (clone-able wrappers only): cloning the suspended program
//       produces two independent peelable handles; each clone's
//       action thunk materialises to the original action.
//
// End-to-end recovery semantics (the action's `Throw` propagating
// to the handler via `interpret_with_either`) are not exercised
// here. That path comes online when the scoped-handler dispatch
// protocol and the standard recovery handler are in place; this
// file restricts itself to verifying that the substrate produces
// the expected suspended shape and that both thunks fire.

use fp_library::{
	brands::{
		ArcBrand,
		BoxBrand,
		BoxCatchBrand,
		CNilBrand,
		CatchBrand,
		CoproductBrand,
		RcBrand,
		SendCatchBrand,
	},
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		catch::{
			BoxCatch,
			Catch,
			SendCatch,
		},
		coproduct::Coproduct,
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
	},
};

// -- Run --

// Default-substrate scoped-effect row: a single `BoxCatch` cell
// over the `BoxBrand` pointer carrying `&'static str` errors.
// The first-order row is left empty because the tests exercise
// only the scoped layer; populating it with `IdentityBrand`-style
// transparent wrappers would force a `Free` layout cycle (no
// pointer indirection on the variant payload), which the existing
// substrate doctests sidestep the same way.
type RunScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
type RunFirstRow = CNilBrand;
type RunProg = Run<RunFirstRow, RunScopedRow, i32>;

#[test]
fn run_t1_catch_produces_scoped_layer() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::catch::<&'static str, _>(action, |_e| Run::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxCatch::Catch))"),
	}
}

#[test]
fn run_t2_action_thunk_materialises_action_program() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::catch::<&'static str, _>(action, |_e| Run::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			action, ..
		}))) => {
			let materialised: RunProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn run_t3_handler_produces_recovery_program() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::catch::<&'static str, _>(action, |_e| Run::pure(99));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			handler, ..
		}))) => {
			let recovered: RunProg = handler("oops");
			assert!(matches!(recovered.peel(), Ok(99)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

// -- RcRun --

type RcScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
type RcFirstRow = CNilBrand;
type RcProg = RcRun<RcFirstRow, RcScopedRow, i32>;

#[test]
fn rc_run_t1_catch_produces_scoped_layer() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Catch::Catch))"),
	}
}

#[test]
fn rc_run_t2_action_thunk_materialises_action_program() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			action, ..
		}))) => {
			let materialised: RcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn rc_run_t3_handler_produces_recovery_program() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(99));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			handler, ..
		}))) => {
			let recovered: RcProg = handler("oops");
			assert!(matches!(recovered.peel(), Ok(99)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn rc_run_t4_clone_yields_two_independent_peels() {
	// Multi-shot Rc-backed cell: cloning the suspended program
	// produces two peelable handles whose action thunks both
	// materialise to the original `Pure(42)` action.
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::catch::<&'static str, _>(action, |_e| RcRun::pure(0));
	let prog_clone = prog.clone();

	let extract = |p: RcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped catch layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRun --

type ArcScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
type ArcFirstRow = CNilBrand;
type ArcProg = ArcRun<ArcFirstRow, ArcScopedRow, i32>;

#[test]
fn arc_run_t1_catch_produces_scoped_layer() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendCatch::Catch))"),
	}
}

#[test]
fn arc_run_t2_action_thunk_materialises_action_program() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			action, ..
		}))) => {
			let materialised: ArcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn arc_run_t3_handler_produces_recovery_program() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(99));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			handler, ..
		}))) => {
			let recovered: ArcProg = handler("oops");
			assert!(matches!(recovered.peel(), Ok(99)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn arc_run_t4_clone_yields_two_independent_peels() {
	// Multi-shot Arc-backed cell, thread-safe: cloning the
	// suspended program produces two peelable handles whose
	// action thunks both materialise to the original `Pure(42)`
	// action. Mirrors `rc_run_t4` with Send+Sync threading.
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::catch::<&'static str, _>(action, |_e| ArcRun::pure(0));
	let prog_clone = prog.clone();

	let extract = |p: ArcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped catch layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- RunExplicit --

type RxScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
type RxFirstRow = CNilBrand;
type RxProg = RunExplicit<'static, RxFirstRow, RxScopedRow, i32>;

#[test]
fn run_explicit_t1_catch_produces_scoped_layer() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::catch::<&'static str, _>(action, |_e| RunExplicit::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxCatch::Catch))"),
	}
}

#[test]
fn run_explicit_t2_action_thunk_materialises_action_program() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::catch::<&'static str, _>(action, |_e| RunExplicit::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			action, ..
		}))) => {
			let materialised: RxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn run_explicit_t3_handler_produces_recovery_program() {
	let action: RxProg = RunExplicit::pure(42);
	let prog: RxProg = RunExplicit::catch::<&'static str, _>(action, |_e| RunExplicit::pure(99));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			handler, ..
		}))) => {
			let recovered: RxProg = handler("oops");
			assert!(matches!(recovered.peel(), Ok(99)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

// -- RcRunExplicit --

type RcxScopedRow = CoproductBrand<CatchBrand<RcBrand, &'static str>, CNilBrand>;
type RcxFirstRow = CNilBrand;
type RcxProg = RcRunExplicit<'static, RcxFirstRow, RcxScopedRow, i32>;

#[test]
fn rc_run_explicit_t1_catch_produces_scoped_layer() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg =
		RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Catch::Catch))"),
	}
}

#[test]
fn rc_run_explicit_t2_action_thunk_materialises_action_program() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg =
		RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			action, ..
		}))) => {
			let materialised: RcxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn rc_run_explicit_t3_handler_produces_recovery_program() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg =
		RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(99));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			handler, ..
		}))) => {
			let recovered: RcxProg = handler("oops");
			assert!(matches!(recovered.peel(), Ok(99)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn rc_run_explicit_t4_clone_yields_two_independent_peels() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let prog: RcxProg =
		RcRunExplicit::catch::<&'static str, _>(action, |_e| RcRunExplicit::pure(0));
	let prog_clone = prog.clone();

	let extract = |p: RcxProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(Catch::Catch {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped catch layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRunExplicit --

type AcxScopedRow = CoproductBrand<SendCatchBrand<ArcBrand, &'static str>, CNilBrand>;
type AcxFirstRow = CNilBrand;
type AcxProg = ArcRunExplicit<'static, AcxFirstRow, AcxScopedRow, i32>;

#[test]
fn arc_run_explicit_t1_catch_produces_scoped_layer() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg =
		ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendCatch::Catch))"),
	}
}

#[test]
fn arc_run_explicit_t2_action_thunk_materialises_action_program() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg =
		ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(0));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			action, ..
		}))) => {
			let materialised: AcxProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn arc_run_explicit_t3_handler_produces_recovery_program() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg =
		ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(99));
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			handler, ..
		}))) => {
			let recovered: AcxProg = handler("oops");
			assert!(matches!(recovered.peel(), Ok(99)));
		}
		_ => panic!("expected scoped catch layer"),
	}
}

#[test]
fn arc_run_explicit_t4_clone_yields_two_independent_peels() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let prog: AcxProg =
		ArcRunExplicit::catch::<&'static str, _>(action, |_e| ArcRunExplicit::pure(0));
	let prog_clone = prog.clone();

	let extract = |p: AcxProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendCatch::Catch {
			action, ..
		}))) => action(()),
		_ => panic!("expected scoped catch layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}
