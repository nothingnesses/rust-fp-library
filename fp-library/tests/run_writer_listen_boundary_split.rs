#![expect(
	clippy::unwrap_used,
	reason = "Tests unwrap Mutex guards after constructing no poisoning path."
)]

//! Focused coverage for the Explicit Writer `listen` boundary handler split.
//!
//! The selected Writer `listen` operation is consumed through the
//! boundary-aware scoped-handler path. After that boundary resumes, later
//! ordinary scoped layers must still dispatch through the residual scoped
//! handler list without requiring an ordinary `WriterListen` handler for the
//! already-consumed boundary head.

use {
	fp_library::{
		brands::*,
		classes::{
			ToDynCloneFn,
			ToDynFnOnce,
			ToDynSendFn,
		},
		handlers,
		types::effects::{
			arc_run_explicit::ArcRunExplicit,
			coproduct::{
				CNil,
				Coproduct,
			},
			handlers::scoped_nt,
			node::Node,
			rc_run_explicit::RcRunExplicit,
			run_explicit::RunExplicit,
			span::{
				BoxSpan,
				SendSpan,
				Span,
			},
			standard_scoped_handlers::{
				span_handler,
				writer_post_handler,
			},
			writer::{
				BoxWriterListen,
				SendWriterListen,
				Writer,
				WriterListen,
			},
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
		sync::{
			Arc,
			Mutex,
		},
	},
};

type ListenResult = (i32, String);

type RunWriterRow = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RunListenBrand = BoxWriterListenBrand<BoxBrand, String, i32>;
type RunSpanBrand = BoxSpanBrand<BoxBrand, &'static str>;
type RunScopedRow = CoproductBrand<RunListenBrand, CoproductBrand<RunSpanBrand, CNilBrand>>;
type RunProg = RunExplicit<'static, RunWriterRow, RunScopedRow, ListenResult>;
type RunLayer = Coproduct<
	BoxWriterListen<'static, BoxBrand, String, i32, ListenResult>,
	Coproduct<BoxSpan<'static, BoxBrand, &'static str, ListenResult>, CNil>,
>;

type RcWriterRow = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type RcListenBrand = WriterListenBrand<RcBrand, String, i32>;
type RcSpanBrand = SpanBrand<RcBrand, &'static str>;
type RcScopedRow = CoproductBrand<RcListenBrand, CoproductBrand<RcSpanBrand, CNilBrand>>;
type RcProg = RcRunExplicit<'static, RcWriterRow, RcScopedRow, ListenResult>;
type RcLayer = Coproduct<
	WriterListen<'static, RcBrand, String, i32, ListenResult>,
	Coproduct<Span<'static, RcBrand, &'static str, ListenResult>, CNil>,
>;

type ArcWriterRow = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
type ArcListenBrand = SendWriterListenBrand<ArcBrand, String, i32>;
type ArcSpanBrand = SendSpanBrand<ArcBrand, &'static str>;
type ArcScopedRow = CoproductBrand<ArcListenBrand, CoproductBrand<ArcSpanBrand, CNilBrand>>;
type ArcProg = ArcRunExplicit<'static, ArcWriterRow, ArcScopedRow, ListenResult>;
type ArcLayer = Coproduct<
	SendWriterListen<'static, ArcBrand, String, i32, ListenResult>,
	Coproduct<SendSpan<'static, ArcBrand, &'static str, ListenResult>, CNil>,
>;

fn run_residual_span(value: ListenResult) -> RunProg {
	let layer: RunLayer = Coproduct::Inr(Coproduct::Inl(BoxSpan::Span {
		tag: "after-listen",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| value),
	}));
	RunExplicit::<'static, RunWriterRow, RunScopedRow, ListenResult>::send(Node::Scoped(layer))
}

fn rc_residual_span(value: ListenResult) -> RcProg {
	let layer: RcLayer = Coproduct::Inr(Coproduct::Inl(Span::Span {
		tag: "after-listen",
		action: <RcBrand as ToDynCloneFn>::new(move |_: ()| value.clone()),
	}));
	RcRunExplicit::<'static, RcWriterRow, RcScopedRow, ListenResult>::send(Node::Scoped(layer))
}

fn arc_residual_span(value: ListenResult) -> ArcProg {
	let layer: ArcLayer = Coproduct::Inr(Coproduct::Inl(SendSpan::Span {
		tag: "after-listen",
		action: <ArcBrand as ToDynSendFn>::new(move |_: ()| value.clone()),
	}));
	ArcRunExplicit::<'static, ArcWriterRow, ArcScopedRow, ListenResult>::send(Node::Scoped(layer))
}

#[test]
fn run_explicit_listen_boundary_skips_consumed_head_and_dispatches_residual_scoped_layer() {
	let log = Rc::new(RefCell::new(Vec::new()));
	let log_for_handler = Rc::clone(&log);
	let action = RunExplicit::<'static, RunWriterRow, RunScopedRow, ()>::tell::<String, _>(
		"first".to_owned(),
	)
	.bind(|()| {
		RunExplicit::<'static, RunWriterRow, RunScopedRow, ()>::tell::<String, _>(
			"second".to_owned(),
		)
	})
	.bind(|()| RunExplicit::pure(40));
	let boundary = RunExplicit::listen::<String, _>(action)
		.bind(|(value, observed)| run_residual_span((value + 2, observed)));

	let result = boundary.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, RunProg>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_nt()
			.on::<RunSpanBrand, _>(span_handler())
			.on::<RunListenBrand, _>(writer_post_handler::<_, CNilBrand, _>()),
	);

	assert_eq!(result, (42, "firstsecond".to_owned()));
	assert_eq!(*log.borrow(), vec!["first".to_owned(), "second".to_owned()]);
}

#[test]
fn rc_run_explicit_listen_boundary_split_remains_reusable() {
	let log = Rc::new(RefCell::new(Vec::new()));
	let action = RcRunExplicit::<'static, RcWriterRow, RcScopedRow, ()>::tell::<String, _>(
		"first".to_owned(),
	)
	.bind(|()| {
		RcRunExplicit::<'static, RcWriterRow, RcScopedRow, ()>::tell::<String, _>(
			"second".to_owned(),
		)
	})
	.bind(|()| RcRunExplicit::pure(40));
	let make_boundary = || {
		RcRunExplicit::listen::<String, _>(action.clone())
			.bind(|(value, observed)| rc_residual_span((value + 2, observed.clone())))
	};

	let log_for_first_handler = Rc::clone(&log);
	let first = make_boundary().handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, RcProg>| match op {
				Writer::Tell(log, next, _) => {
					log_for_first_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_nt()
			.on::<RcSpanBrand, _>(span_handler())
			.on::<RcListenBrand, _>(writer_post_handler::<_, CNilBrand, _>()),
	);
	let log_for_second_handler = Rc::clone(&log);
	let second = make_boundary().handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, RcProg>| match op {
				Writer::Tell(log, next, _) => {
					log_for_second_handler.borrow_mut().push(log);
					next
				}
			},
		},
		scoped_nt()
			.on::<RcSpanBrand, _>(span_handler())
			.on::<RcListenBrand, _>(writer_post_handler::<_, CNilBrand, _>()),
	);

	assert_eq!(first, (42, "firstsecond".to_owned()));
	assert_eq!(second, (42, "firstsecond".to_owned()));
	assert_eq!(
		*log.borrow(),
		vec!["first".to_owned(), "second".to_owned(), "first".to_owned(), "second".to_owned(),]
	);
}

#[test]
fn arc_run_explicit_listen_boundary_split_preserves_send_sync_path() {
	let log = Arc::new(Mutex::new(Vec::new()));
	let log_for_handler = Arc::clone(&log);
	let action = ArcRunExplicit::<'static, ArcWriterRow, ArcScopedRow, ()>::tell::<String, _>(
		"first".to_owned(),
	)
	.bind(|()| {
		ArcRunExplicit::<'static, ArcWriterRow, ArcScopedRow, ()>::tell::<String, _>(
			"second".to_owned(),
		)
	})
	.bind(|()| ArcRunExplicit::pure(40));
	let boundary = ArcRunExplicit::listen::<String, _>(action)
		.bind(|(value, observed)| arc_residual_span((value + 2, observed.clone())));

	let result = boundary.handle(
		handlers! {
			WriterBrand<String>: move |op: Writer<'_, String, ArcProg>| match op {
				Writer::Tell(log, next, _) => {
					log_for_handler.lock().unwrap().push(log);
					next
				}
			},
		},
		scoped_nt()
			.on::<ArcSpanBrand, _>(span_handler())
			.on::<ArcListenBrand, _>(writer_post_handler::<_, CNilBrand, _>()),
	);

	assert_eq!(result, (42, "firstsecond".to_owned()));
	assert_eq!(*log.lock().unwrap(), vec!["first".to_owned(), "second".to_owned()]);
}
