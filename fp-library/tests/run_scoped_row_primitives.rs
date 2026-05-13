use fp_library::{
	brands::{
		ArcBrand,
		ArcCoyonedaBrand,
		BoxBrand,
		BoxSpanBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
		IdentityBrand,
		RcBrand,
		RcCoyonedaBrand,
		SendSpanBrand,
		SpanBrand,
	},
	classes::ToDynFnOnce,
	handlers,
	types::{
		FreeExplicit,
		Identity,
		effects::{
			arc_run::ArcRun,
			arc_run_explicit::ArcRunExplicit,
			coproduct::Coproduct,
			node::Node,
			rc_run::RcRun,
			rc_run_explicit::RcRunExplicit,
			run::Run,
			run_explicit::RunExplicit,
			scoped_nt,
			span::{
				BoxSpan,
				SendSpan,
				Span,
			},
		},
	},
};

type RunFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RunScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
type RcFirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
type ArcFirstRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, &'static str>, CNilBrand>;

fn run_explicit_span_program(
	action: RunExplicit<'static, RunFirstRow, RunScopedRow, i32>
) -> RunExplicit<'static, RunFirstRow, RunScopedRow, i32> {
	let action_free = Box::new(action.into_free_explicit());
	let layer = Coproduct::Inl(BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_free),
	});

	RunExplicit::from_free_explicit(FreeExplicit::wrap(Node::Scoped(layer)))
}

#[test]
fn run_interpret_with_preserves_nested_scoped_span() {
	let action: Run<RunFirstRow, RunScopedRow, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
	let prog: Run<RunFirstRow, RunScopedRow, i32> = Run::span::<&'static str, _>("request", action);

	let narrowed: Run<CNilBrand, RunScopedRow, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<Run<CNilBrand, RunScopedRow, i32>>| op.0,
		);
	let without_span: Run<CNilBrand, CNilBrand, i32> = narrowed
		.interpret_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
			|span: BoxSpan<'static, BoxBrand, &'static str, Run<CNilBrand, CNilBrand, i32>>| {
				match span {
					BoxSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(without_span.extract(), 7);
}

#[test]
fn run_interpose_preserves_nested_scoped_span() {
	let action: Run<RunFirstRow, RunScopedRow, i32> = Run::lift::<IdentityBrand, _>(Identity(7));
	let prog: Run<RunFirstRow, RunScopedRow, i32> = Run::span::<&'static str, _>("request", action);

	let interposed: Run<RunFirstRow, RunScopedRow, i32> = prog
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<Run<RunFirstRow, RunScopedRow, i32>>| Run::pure(99),
		);
	let without_span: Run<RunFirstRow, CNilBrand, i32> = interposed
		.interpret_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
			|span: BoxSpan<'static, BoxBrand, &'static str, Run<RunFirstRow, CNilBrand, i32>>| {
				match span {
					BoxSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.interpret(
		handlers! {
			IdentityBrand: |op: Identity<Run<RunFirstRow, CNilBrand, i32>>| op.0,
		},
		scoped_nt(),
	);

	assert_eq!(result, 99);
}

#[test]
fn rc_run_interpret_with_preserves_nested_scoped_span() {
	let action: RcRun<RcFirstRow, RcScopedRow, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: RcRun<RcFirstRow, RcScopedRow, i32> =
		RcRun::span::<&'static str, _>("request", action);

	let narrowed: RcRun<CNilBrand, RcScopedRow, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RcRun<CNilBrand, RcScopedRow, i32>>| op.0,
		);
	let without_span: RcRun<CNilBrand, CNilBrand, i32> = narrowed
		.interpret_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(
			|span: Span<'static, RcBrand, &'static str, RcRun<CNilBrand, CNilBrand, i32>>| {
				match span {
					Span::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(without_span.extract(), 7);
}

#[test]
fn rc_run_interpose_preserves_nested_scoped_span() {
	let action: RcRun<RcFirstRow, RcScopedRow, i32> = RcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: RcRun<RcFirstRow, RcScopedRow, i32> =
		RcRun::span::<&'static str, _>("request", action);

	let interposed: RcRun<RcFirstRow, RcScopedRow, i32> = prog
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<RcRun<RcFirstRow, RcScopedRow, i32>>| RcRun::pure(99),
		);
	let without_span: RcRun<RcFirstRow, CNilBrand, i32> = interposed
		.interpret_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(
			|span: Span<'static, RcBrand, &'static str, RcRun<RcFirstRow, CNilBrand, i32>>| {
				match span {
					Span::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.interpret(
		handlers! {
			IdentityBrand: |op: Identity<RcRun<RcFirstRow, CNilBrand, i32>>| op.0,
		},
		scoped_nt(),
	);

	assert_eq!(result, 99);
}

#[test]
fn arc_run_interpret_with_preserves_nested_scoped_span() {
	let action: ArcRun<ArcFirstRow, ArcScopedRow, i32> =
		ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: ArcRun<ArcFirstRow, ArcScopedRow, i32> =
		ArcRun::span::<&'static str, _>("request", action);

	let narrowed: ArcRun<CNilBrand, ArcScopedRow, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<ArcRun<CNilBrand, ArcScopedRow, i32>>| op.0,
		);
	let without_span: ArcRun<CNilBrand, CNilBrand, i32> = narrowed
		.interpret_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(
			|span: SendSpan<'static, ArcBrand, &'static str, ArcRun<CNilBrand, CNilBrand, i32>>| {
				match span {
					SendSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(without_span.extract(), 7);
}

#[test]
fn arc_run_interpose_preserves_nested_scoped_span() {
	let action: ArcRun<ArcFirstRow, ArcScopedRow, i32> =
		ArcRun::lift::<IdentityBrand, _>(Identity(7));
	let prog: ArcRun<ArcFirstRow, ArcScopedRow, i32> =
		ArcRun::span::<&'static str, _>("request", action);

	let interposed: ArcRun<ArcFirstRow, ArcScopedRow, i32> = prog
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<ArcRun<ArcFirstRow, ArcScopedRow, i32>>| ArcRun::pure(99),
		);
	let without_span: ArcRun<ArcFirstRow, CNilBrand, i32> = interposed
		.interpret_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(
			|span: SendSpan<
				'static,
				ArcBrand,
				&'static str,
				ArcRun<ArcFirstRow, CNilBrand, i32>,
			>| {
				match span {
					SendSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.interpret(
		handlers! {
			IdentityBrand: |op: Identity<ArcRun<ArcFirstRow, CNilBrand, i32>>| op.0,
		},
		scoped_nt(),
	);

	assert_eq!(result, 99);
}

#[test]
fn run_explicit_interpret_with_preserves_nested_scoped_span() {
	let action: RunExplicit<'static, RunFirstRow, RunScopedRow, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: RunExplicit<'static, RunFirstRow, RunScopedRow, i32> =
		run_explicit_span_program(action);

	let narrowed: RunExplicit<'static, CNilBrand, RunScopedRow, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RunExplicit<'static, CNilBrand, RunScopedRow, i32>>| op.0,
		);
	let without_span: RunExplicit<'static, CNilBrand, CNilBrand, i32> = narrowed
		.interpret_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
			|span: BoxSpan<
				'static,
				BoxBrand,
				&'static str,
				RunExplicit<'static, CNilBrand, CNilBrand, i32>,
			>| {
				match span {
					BoxSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(without_span.extract(), 7);
}

#[test]
fn run_explicit_interpose_preserves_nested_scoped_span() {
	let action: RunExplicit<'static, RunFirstRow, RunScopedRow, i32> =
		RunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: RunExplicit<'static, RunFirstRow, RunScopedRow, i32> =
		run_explicit_span_program(action);

	let interposed: RunExplicit<'static, RunFirstRow, RunScopedRow, i32> = prog
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<RunExplicit<'static, RunFirstRow, RunScopedRow, i32>>| {
				RunExplicit::pure(99)
			},
		);
	let without_span: RunExplicit<'static, RunFirstRow, CNilBrand, i32> = interposed
		.interpret_scoped_with::<BoxSpanBrand<BoxBrand, &'static str>, _, CNilBrand>(
			|span: BoxSpan<
				'static,
				BoxBrand,
				&'static str,
				RunExplicit<'static, RunFirstRow, CNilBrand, i32>,
			>| {
				match span {
					BoxSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.interpret(
		handlers! {
			IdentityBrand: |op: Identity<RunExplicit<'static, RunFirstRow, CNilBrand, i32>>| op.0,
		},
		scoped_nt(),
	);

	assert_eq!(result, 99);
}

#[test]
fn rc_run_explicit_interpret_with_preserves_nested_scoped_span() {
	let action: RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32> =
		RcRunExplicit::span::<&'static str, _>("request", action);

	let narrowed: RcRunExplicit<'static, CNilBrand, RcScopedRow, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<RcRunExplicit<'static, CNilBrand, RcScopedRow, i32>>| op.0,
		);
	let without_span: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> = narrowed
		.interpret_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(
			|span: Span<
				'static,
				RcBrand,
				&'static str,
				RcRunExplicit<'static, CNilBrand, CNilBrand, i32>,
			>| {
				match span {
					Span::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(without_span.extract(), 7);
}

#[test]
fn rc_run_explicit_interpose_preserves_nested_scoped_span() {
	let action: RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32> =
		RcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32> =
		RcRunExplicit::span::<&'static str, _>("request", action);

	let interposed: RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32> = prog
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<RcRunExplicit<'static, RcFirstRow, RcScopedRow, i32>>| {
				RcRunExplicit::pure(99)
			},
		);
	let without_span: RcRunExplicit<'static, RcFirstRow, CNilBrand, i32> = interposed
		.interpret_scoped_with::<SpanBrand<RcBrand, &'static str>, _, CNilBrand>(
			|span: Span<
				'static,
				RcBrand,
				&'static str,
				RcRunExplicit<'static, RcFirstRow, CNilBrand, i32>,
			>| {
				match span {
					Span::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.interpret(
		handlers! {
			IdentityBrand: |op: Identity<RcRunExplicit<'static, RcFirstRow, CNilBrand, i32>>| op.0,
		},
		scoped_nt(),
	);

	assert_eq!(result, 99);
}

#[test]
fn arc_run_explicit_interpret_with_preserves_nested_scoped_span() {
	let action: ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32> =
		ArcRunExplicit::span::<&'static str, _>("request", action);

	let narrowed: ArcRunExplicit<'static, CNilBrand, ArcScopedRow, i32> = prog
		.interpret_with::<IdentityBrand, _, CNilBrand>(
			|op: Identity<ArcRunExplicit<'static, CNilBrand, ArcScopedRow, i32>>| op.0,
		);
	let without_span: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> = narrowed
		.interpret_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(
			|span: SendSpan<
				'static,
				ArcBrand,
				&'static str,
				ArcRunExplicit<'static, CNilBrand, CNilBrand, i32>,
			>| {
				match span {
					SendSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);

	assert_eq!(without_span.extract(), 7);
}

#[test]
fn arc_run_explicit_interpose_preserves_nested_scoped_span() {
	let action: ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32> =
		ArcRunExplicit::lift::<IdentityBrand, _>(Identity(7));
	let prog: ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32> =
		ArcRunExplicit::span::<&'static str, _>("request", action);

	let interposed: ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32> = prog
		.interpose::<IdentityBrand, _, CNilBrand, _>(
			|_op: Identity<ArcRunExplicit<'static, ArcFirstRow, ArcScopedRow, i32>>| {
				ArcRunExplicit::pure(99)
			},
		);
	let without_span: ArcRunExplicit<'static, ArcFirstRow, CNilBrand, i32> =
		interposed.interpret_scoped_with::<SendSpanBrand<ArcBrand, &'static str>, _, CNilBrand>(
			|span: SendSpan<
				'static,
				ArcBrand,
				&'static str,
				ArcRunExplicit<'static, ArcFirstRow, CNilBrand, i32>,
			>| {
				match span {
					SendSpan::Span {
						tag,
						action,
					} => {
						assert_eq!(tag, "request");
						action(())
					}
				}
			},
		);
	let result = without_span.interpret(
		handlers! {
			IdentityBrand: |op: Identity<ArcRunExplicit<'static, ArcFirstRow, CNilBrand, i32>>| op.0,
		},
		scoped_nt(),
	);

	assert_eq!(result, 99);
}
