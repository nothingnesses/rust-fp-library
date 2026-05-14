#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Shape-only integration tests for the substrate-level scoped
// `span<Tag, Idx>` smart constructor across the Run-wrapper family.
// Each wrapper's section verifies:
//   T1: `span(tag, action)` produces a program suspended at a
//       `Node::Scoped` layer carrying a `BoxSpan` / `Span` /
//       `SendSpan` cell projected via `Member::inject` at the head
//       of the scoped row.
//   T2: the by-value tag is stored in the scoped cell with the
//       expected value. The default Box-backed wrappers use a
//       non-Clone tag to exercise the single-shot bound surface.
//   T3: invoking the cell's stored `action` thunk with the unit
//       argument yields a wrapper-typed program whose `peel`
//       returns the original `Pure` payload.
//   T4 (clone-able wrappers only): cloning the suspended program
//       produces two independent peelable handles; each clone's
//       action thunk materialises to the original action.
//
// The Explicit-family `span` constructors return indexed boundaries
// rather than plain programs, so those tests dispatch the boundary and
// check that the tag and selected action value are recoverable before
// the final continuation resumes. End-to-end instrumentation semantics
// are otherwise covered by scoped-dispatcher integration tests.

use fp_library::{
	brands::{
		ArcBrand,
		BoxBrand,
		BoxSpanBrand,
		CNilBrand,
		CoproductBrand,
		RcBrand,
		SendSpanBrand,
		SpanBrand,
	},
	handlers,
	types::effects::{
		arc_run::ArcRun,
		arc_run_explicit::ArcRunExplicit,
		coproduct::Coproduct,
		node::Node,
		rc_run::RcRun,
		rc_run_explicit::RcRunExplicit,
		run::Run,
		run_explicit::RunExplicit,
		scoped_dispatchers::span_dispatcher,
		span::{
			BoxSpan,
			SendSpan,
			Span,
		},
	},
};

#[derive(Debug, PartialEq, Eq)]
struct NonCloneTag(&'static str);

// -- Run --

type RunScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, NonCloneTag>, CNilBrand>;
type RunFirstRow = CNilBrand;
type RunProg = Run<RunFirstRow, RunScopedRow, i32>;

#[test]
fn run_t1_span_produces_scoped_layer() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::span::<NonCloneTag, _>(NonCloneTag("request"), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxSpan::Span {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(BoxSpan::Span))"),
	}
}

#[test]
fn run_t2_tag_is_stored_by_value_without_clone_bound() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::span::<NonCloneTag, _>(NonCloneTag("request"), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxSpan::Span {
			tag, ..
		}))) => {
			assert_eq!(tag, NonCloneTag("request"));
		}
		_ => panic!("expected scoped span layer"),
	}
}

#[test]
fn run_t3_action_thunk_materialises_action_program() {
	let action: RunProg = Run::pure(42);
	let prog: RunProg = Run::span::<NonCloneTag, _>(NonCloneTag("request"), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxSpan::Span {
			action, ..
		}))) => {
			let materialised: RunProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped span layer"),
	}
}

// -- RcRun --

type RcScopedRow = CoproductBrand<SpanBrand<RcBrand, String>, CNilBrand>;
type RcFirstRow = CNilBrand;
type RcProg = RcRun<RcFirstRow, RcScopedRow, i32>;

#[test]
fn rc_run_t1_span_produces_scoped_layer() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::span::<String, _>("request".to_owned(), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Span::Span {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(Span::Span))"),
	}
}

#[test]
fn rc_run_t2_tag_is_stored_by_value() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::span::<String, _>("request".to_owned(), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Span::Span {
			tag, ..
		}))) => {
			assert_eq!(tag, "request");
		}
		_ => panic!("expected scoped span layer"),
	}
}

#[test]
fn rc_run_t3_action_thunk_materialises_action_program() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::span::<String, _>("request".to_owned(), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(Span::Span {
			action, ..
		}))) => {
			let materialised: RcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped span layer"),
	}
}

#[test]
fn rc_run_t4_clone_yields_two_independent_peels() {
	let action: RcProg = RcRun::pure(42);
	let prog: RcProg = RcRun::span::<String, _>("request".to_owned(), action);
	let prog_clone = prog.clone();

	let extract = |p: RcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(Span::Span {
			tag,
			action,
		}))) => {
			assert_eq!(tag, "request");
			action(())
		}
		_ => panic!("expected scoped span layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- ArcRun --

type ArcScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, String>, CNilBrand>;
type ArcFirstRow = CNilBrand;
type ArcProg = ArcRun<ArcFirstRow, ArcScopedRow, i32>;

#[test]
fn arc_run_t1_span_produces_scoped_layer() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::span::<String, _>("request".to_owned(), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendSpan::Span {
			..
		}))) => {}
		_ => panic!("expected Node::Scoped(Coproduct::Inl(SendSpan::Span))"),
	}
}

#[test]
fn arc_run_t2_tag_is_stored_by_value() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::span::<String, _>("request".to_owned(), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendSpan::Span {
			tag, ..
		}))) => {
			assert_eq!(tag, "request");
		}
		_ => panic!("expected scoped span layer"),
	}
}

#[test]
fn arc_run_t3_action_thunk_materialises_action_program() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::span::<String, _>("request".to_owned(), action);
	match prog.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendSpan::Span {
			action, ..
		}))) => {
			let materialised: ArcProg = action(());
			assert!(matches!(materialised.peel(), Ok(42)));
		}
		_ => panic!("expected scoped span layer"),
	}
}

#[test]
fn arc_run_t4_clone_yields_two_independent_peels() {
	let action: ArcProg = ArcRun::pure(42);
	let prog: ArcProg = ArcRun::span::<String, _>("request".to_owned(), action);
	let prog_clone = prog.clone();

	let extract = |p: ArcProg| match p.peel() {
		Err(Node::Scoped(Coproduct::Inl(SendSpan::Span {
			tag,
			action,
		}))) => {
			assert_eq!(tag, "request");
			action(())
		}
		_ => panic!("expected scoped span layer"),
	};
	assert!(matches!(extract(prog).peel(), Ok(42)));
	assert!(matches!(extract(prog_clone).peel(), Ok(42)));
}

// -- RunExplicit --

type RxScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, NonCloneTag>, CNilBrand>;
type RxFirstRow = CNilBrand;
type RxProg = RunExplicit<'static, RxFirstRow, RxScopedRow, i32>;

#[test]
fn run_explicit_t1_span_produces_scoped_layer() {
	let action: RxProg = RunExplicit::pure(42);
	let boundary = RunExplicit::span::<NonCloneTag, _>(NonCloneTag("request"), action);
	let prog: RxProg = span_dispatcher().dispatch_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|_, value| RunExplicit::pure(value),
	);

	assert!(matches!(prog.peel(), Ok(42)));
}

#[test]
fn run_explicit_t2_tag_is_stored_by_value_without_clone_bound() {
	let action: RxProg = RunExplicit::pure(42);
	let boundary = RunExplicit::span::<NonCloneTag, _>(NonCloneTag("request"), action);
	let prog: RxProg = span_dispatcher().dispatch_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(*tag, NonCloneTag("request"));
			RunExplicit::pure(value)
		},
	);

	assert!(matches!(prog.peel(), Ok(42)));
}

#[test]
fn run_explicit_t3_action_thunk_materialises_action_program() {
	let action: RxProg = RunExplicit::pure(42);
	let boundary = RunExplicit::span::<NonCloneTag, _>(NonCloneTag("request"), action);
	let prog: RxProg = span_dispatcher().dispatch_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|_, value| {
			assert_eq!(value, 42);
			RunExplicit::pure(value)
		},
	);

	assert!(matches!(prog.peel(), Ok(42)));
}

// -- RcRunExplicit --

type RcxScopedRow = CoproductBrand<SpanBrand<RcBrand, String>, CNilBrand>;
type RcxFirstRow = CNilBrand;
type RcxProg = RcRunExplicit<'static, RcxFirstRow, RcxScopedRow, i32>;

#[test]
fn rc_run_explicit_t1_span_boundary_dispatch_observes_tag() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let boundary = RcRunExplicit::span::<String, _>("request".to_owned(), action);
	let prog: RcxProg = span_dispatcher().dispatch_rc_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(tag, "request");
			RcRunExplicit::pure(value)
		},
	);

	assert!(matches!(prog.peel(), Ok(42)));
}

#[test]
fn rc_run_explicit_t2_span_boundary_map_runs_after_post_action() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let boundary =
		RcRunExplicit::span::<String, _>("request".to_owned(), action).map(|value| value + 1);
	let prog: RcxProg = span_dispatcher().dispatch_rc_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(tag, "request");
			RcRunExplicit::pure(value + 1)
		},
	);

	assert!(matches!(prog.peel(), Ok(44)));
}

#[test]
fn rc_run_explicit_t3_span_boundary_bind_runs_after_post_action() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let boundary = RcRunExplicit::span::<String, _>("request".to_owned(), action)
		.bind(|value| RcRunExplicit::pure(value + 1));
	let prog: RcxProg = span_dispatcher().dispatch_rc_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(tag, "request");
			RcRunExplicit::pure(value + 1)
		},
	);

	assert!(matches!(prog.peel(), Ok(44)));
}

#[test]
fn rc_run_explicit_t4_independent_boundaries_share_action() {
	let action: RcxProg = RcRunExplicit::pure(42);
	let first = RcRunExplicit::span::<String, _>("request".to_owned(), action.clone());
	let second = RcRunExplicit::span::<String, _>("request".to_owned(), action);

	let dispatch = |boundary| {
		span_dispatcher().dispatch_rc_run_explicit_span_boundary_with_post_action(
			boundary,
			&handlers! {},
			|tag, value| {
				assert_eq!(tag, "request");
				RcRunExplicit::pure(value)
			},
		)
	};

	assert!(matches!(dispatch(first).peel(), Ok(42)));
	assert!(matches!(dispatch(second).peel(), Ok(42)));
}

// -- ArcRunExplicit --

type AcxScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, String>, CNilBrand>;
type AcxFirstRow = CNilBrand;
type AcxProg = ArcRunExplicit<'static, AcxFirstRow, AcxScopedRow, i32>;

#[test]
fn arc_run_explicit_t1_span_boundary_dispatch_observes_tag() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let boundary = ArcRunExplicit::span::<String, _>("request".to_owned(), action);
	let prog: AcxProg = span_dispatcher().dispatch_arc_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(tag, "request");
			ArcRunExplicit::pure(value)
		},
	);

	assert!(matches!(prog.peel(), Ok(42)));
}

#[test]
fn arc_run_explicit_t2_span_boundary_map_runs_after_post_action() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let boundary =
		ArcRunExplicit::span::<String, _>("request".to_owned(), action).map(|value| value + 1);
	let prog: AcxProg = span_dispatcher().dispatch_arc_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(tag, "request");
			ArcRunExplicit::pure(value + 1)
		},
	);

	assert!(matches!(prog.peel(), Ok(44)));
}

#[test]
fn arc_run_explicit_t3_span_boundary_bind_runs_after_post_action() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let boundary = ArcRunExplicit::span::<String, _>("request".to_owned(), action)
		.bind(|value| ArcRunExplicit::pure(value + 1));
	let prog: AcxProg = span_dispatcher().dispatch_arc_run_explicit_span_boundary_with_post_action(
		boundary,
		&handlers! {},
		|tag, value| {
			assert_eq!(tag, "request");
			ArcRunExplicit::pure(value + 1)
		},
	);

	assert!(matches!(prog.peel(), Ok(44)));
}

#[test]
fn arc_run_explicit_t4_independent_boundaries_share_action() {
	let action: AcxProg = ArcRunExplicit::pure(42);
	let first = ArcRunExplicit::span::<String, _>("request".to_owned(), action.clone());
	let second = ArcRunExplicit::span::<String, _>("request".to_owned(), action);

	let dispatch = |boundary| {
		span_dispatcher().dispatch_arc_run_explicit_span_boundary_with_post_action(
			boundary,
			&handlers! {},
			|tag, value| {
				assert_eq!(tag, "request");
				ArcRunExplicit::pure(value)
			},
		)
	};

	assert!(matches!(dispatch(first).peel(), Ok(42)));
	assert!(matches!(dispatch(second).peel(), Ok(42)));
}
