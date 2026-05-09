use fp_library::{
	brands::{
		BoxBrand,
		BoxSpanBrand,
		CNilBrand,
		CoproductBrand,
		CoyonedaBrand,
		IdentityBrand,
		RcBrand,
		RcCoyonedaBrand,
		SpanBrand,
	},
	handlers,
	types::{
		Identity,
		effects::{
			rc_run::RcRun,
			run::Run,
			scoped_nt,
			span::{
				BoxSpan,
				Span,
			},
		},
	},
};

type RunFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type RunScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
type RcFirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type RcScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;

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
