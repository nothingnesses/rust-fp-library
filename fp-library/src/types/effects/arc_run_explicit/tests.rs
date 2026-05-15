use {
	super::*,
	crate::{
		brands::{
			ArcBrand,
			ArcCoyonedaBrand,
			ArcRunExplicitBrand,
			CNilBrand,
			CoproductBrand,
			ExceptBrand,
			IdentityBrand,
			SendReaderBrand,
		},
		classes::{
			RefCountedPointer,
			SendPointed,
		},
		impl_kind,
		kinds::{
			InferableBrand_266801a817966495,
			Kind_266801a817966495,
		},
		types::{
			ArcFreeExplicit,
			effects::{
				except::Except,
				handlers::HandlersNil,
				interpreter::{
					ExplicitBoundaryOf,
					ExplicitBoundaryTypes,
					ScopedBoundaryTypes,
					ScopedContinuation,
				},
				reader::SendReader,
				run_explicit::{
					RunExplicitBracketCarrierLayer,
					RunExplicitCatchCarrierLayer,
					RunExplicitRefBracketCarrierLayer,
					RunExplicitRefLocalCarrierLayer,
					RunExplicitSpanCarrierLayer,
				},
				standard_scoped_handlers::{
					bracket_handler,
					catch_handler,
					ref_bracket_handler,
					ref_local_handler,
					span_handler,
				},
			},
		},
	},
	core::marker::PhantomData,
	std::sync::{
		Arc as StdArc,
		atomic::{
			AtomicUsize,
			Ordering,
		},
	},
};

type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
type Scoped = CNilBrand;
type RunAlias<'a, A> = ArcRunExplicit<'a, FirstRow, Scoped, A>;
type EmptyArcRunExplicit<'a, A> = ArcRunExplicit<'a, CNilBrand, CNilBrand, A>;
type ArcReaderRow = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
type ArcReaderRowMinusReader = CNilBrand;
type ArcReaderRunExplicit<'a, A> = ArcRunExplicit<'a, ArcReaderRow, CNilBrand, A>;
type ArcExceptRow = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type ArcExceptRowMinusExcept = CNilBrand;
type ArcExceptRunExplicit<'a, A> = ArcRunExplicit<'a, ArcExceptRow, CNilBrand, A>;
type ArcSharedBoundaryActionProgram<'a, Action> = EmptyArcRunExplicit<'a, Action>;
type ArcSharedBoundaryFinalProgram<'a, Final> = EmptyArcRunExplicit<'a, Final>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ArcSharedExplicitBoundaryBrand;

impl_kind! {
	impl for ArcSharedExplicitBoundaryBrand {
		type Of<'a, Action: 'a, Final: 'a>: 'a =
			ArcSharedExplicitBoundary<'a, Action, Final>;
	}
}

impl<'a, Action: 'a, Final: 'a> ScopedBoundaryTypes<'a, Action, Final>
	for ArcSharedExplicitBoundaryBrand
{
	type ActionProgram = ArcSharedBoundaryActionProgram<'a, Action>;
	type FinalProgram = ArcSharedBoundaryFinalProgram<'a, Final>;
}

struct ArcSharedExplicitBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a, {
	action: ArcSharedBoundaryActionProgram<'a, Action>,
	outer: StdArc<dyn Fn(Action) -> ArcSharedBoundaryFinalProgram<'a, Final> + Send + Sync + 'a>,
	result: PhantomData<fn(Action) -> Final>,
}

impl<'a, Action, Final> Clone for ArcSharedExplicitBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a,
{
	fn clone(&self) -> Self {
		Self {
			action: self.action.clone(),
			outer: StdArc::clone(&self.outer),
			result: PhantomData,
		}
	}
}

impl<'a, Action, Final> ArcSharedExplicitBoundary<'a, Action, Final>
where
	Action: Clone + Send + Sync + 'a,
	Final: Send + Sync + 'a,
{
	fn new(
		action: ArcSharedBoundaryActionProgram<'a, Action>,
		outer: impl Fn(Action) -> ArcSharedBoundaryFinalProgram<'a, Final> + Send + Sync + 'a,
	) -> Self {
		Self {
			action,
			outer: StdArc::new(outer),
			result: PhantomData,
		}
	}

	fn resume_with_post_action(
		self,
		post_action: impl Fn(Action) -> ArcSharedBoundaryActionProgram<'a, Action> + Send + Sync + 'a,
	) -> ArcSharedBoundaryFinalProgram<'a, Final> {
		let outer = StdArc::clone(&self.outer);

		self.action.bind(move |action_value| {
			let outer = StdArc::clone(&outer);
			post_action(action_value).bind(move |post_value| outer(post_value))
		})
	}
}

fn _send_sync_witness<T: Send + Sync>() {}

fn arc_explicit_scoped_continuation<'a, Action, Final, K>(
	action: EmptyArcRunExplicit<'a, Action>,
	outer: K,
) -> ArcRunExplicitScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
where
	Action: Clone + Send + Sync + 'a,
	Final: Send + Sync + 'a,
	K: Fn(Action) -> EmptyArcRunExplicit<'a, Final> + Send + Sync + 'a, {
	ArcRunExplicitScopedContinuation {
		action,
		outer: <ArcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

fn arc_explicit_action_supplied_scoped_continuation<'a, Action, Final, K>(
	outer: K
) -> ArcRunExplicitActionSuppliedScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
where
	Action: Clone + Send + Sync + 'a,
	Final: Send + Sync + 'a,
	K: Fn(Action) -> EmptyArcRunExplicit<'a, Final> + Send + Sync + 'a, {
	ArcRunExplicitActionSuppliedScopedContinuation {
		outer: <ArcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

#[test]
fn shared_explicit_boundary_protocol_is_send_sync_and_repeats_arc_resume() {
	fn require_private_boundary_protocol<'a, Action, Final>(
		boundary: ExplicitBoundaryOf<'a, ArcSharedExplicitBoundaryBrand, Action, Final>
	) -> ExplicitBoundaryOf<'a, ArcSharedExplicitBoundaryBrand, Action, Final>
	where
		Action: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		ArcSharedExplicitBoundaryBrand: ExplicitBoundaryTypes<
				'a,
				Action,
				Final,
				ActionProgram = ArcSharedBoundaryActionProgram<'a, Action>,
				FinalProgram = ArcSharedBoundaryFinalProgram<'a, Final>,
			>,
		ExplicitBoundaryOf<'a, ArcSharedExplicitBoundaryBrand, Action, Final>: Send + Sync, {
		boundary
	}

	_send_sync_witness::<ExplicitBoundaryOf<'static, ArcSharedExplicitBoundaryBrand, i32, i32>>();

	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let boundary: ExplicitBoundaryOf<'_, ArcSharedExplicitBoundaryBrand, i32, i32> =
		ArcSharedExplicitBoundary::new(EmptyArcRunExplicit::pure(40), move |value| {
			let step = outer_order.fetch_add(1, Ordering::SeqCst);
			assert!(step == 1 || step == 3);
			EmptyArcRunExplicit::pure(value * 10)
		});
	let boundary = require_private_boundary_protocol(boundary);

	let first_order = StdArc::clone(&order);
	let first: EmptyArcRunExplicit<'_, i32> =
		boundary.clone().resume_with_post_action(move |value| {
			assert_eq!(first_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRunExplicit::pure(value + 1)
		});
	let second_order = StdArc::clone(&order);
	let second: EmptyArcRunExplicit<'_, i32> = boundary.resume_with_post_action(move |value| {
		assert_eq!(second_order.fetch_add(1, Ordering::SeqCst), 2);
		EmptyArcRunExplicit::pure(value + 2)
	});

	assert_eq!(first.extract(), 410);
	assert_eq!(second.extract(), 420);
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn from_and_into_round_trip() {
	let arc_free: ArcFreeExplicit<'_, _, i32> = ArcFreeExplicit::pure(42);
	let run: RunAlias<'_, i32> = ArcRunExplicit::from_arc_free_explicit(arc_free);
	let _back = run.into_arc_free_explicit();
}

#[test]
fn clone_branches_are_cheap() {
	let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
	let _branch = run.clone();
}

#[test]
fn brand_send_pure_evaluates() {
	let run: RunAlias<'_, _> = <ArcRunExplicitBrand<FirstRow, Scoped> as SendPointed>::send_pure(7);
	assert_eq!(run.into_arc_free_explicit().evaluate(), 7);
}

#[test]
fn inherent_map_evaluates() {
	let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
	let mapped = run.map(|x: i32| x * 3);
	assert_eq!(mapped.into_arc_free_explicit().evaluate(), 30);
}

#[test]
fn inherent_bind_evaluates() {
	let run: RunAlias<'_, _> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
	let chained =
		run.bind(|x: i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(x + 5)));
	assert_eq!(chained.into_arc_free_explicit().evaluate(), 7);
}

#[test]
fn scoped_continuation_carrier_is_send_sync() {
	_send_sync_witness::<
		ArcRunExplicitScopedContinuation<
			'static,
			CNilBrand,
			CNilBrand,
			i32,
			i32,
			fn(i32) -> EmptyArcRunExplicit<'static, i32>,
		>,
	>();
}

#[test]
fn action_supplied_scoped_continuation_carrier_is_send_sync() {
	_send_sync_witness::<
		ArcRunExplicitActionSuppliedScopedContinuation<
			'static,
			CNilBrand,
			CNilBrand,
			i32,
			i32,
			fn(i32) -> EmptyArcRunExplicit<'static, i32>,
		>,
	>();
}

#[test]
fn scoped_continuation_repeats_action_before_outer_continuation() {
	let outer_calls = StdArc::new(AtomicUsize::new(0));
	let observed_calls = StdArc::clone(&outer_calls);
	let carrier = ScopedContinuation::new(arc_explicit_scoped_continuation(
		EmptyArcRunExplicit::pure(40),
		move |value| {
			observed_calls.fetch_add(1, Ordering::SeqCst);
			EmptyArcRunExplicit::pure(value * 10)
		},
	));

	let first: EmptyArcRunExplicit<'_, i32> = carrier.clone().resume_arc(&HandlersNil);
	let second: EmptyArcRunExplicit<'_, i32> = carrier.resume_arc(&HandlersNil);

	assert_eq!(first.extract(), 400);
	assert_eq!(second.extract(), 400);
	assert_eq!(outer_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn scoped_continuation_transforms_action_before_outer_continuation() {
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let carrier = ScopedContinuation::new(arc_explicit_scoped_continuation(
		EmptyArcRunExplicit::pure(40),
		move |value| {
			assert_eq!(outer_order.fetch_add(1, Ordering::SeqCst), 1);
			EmptyArcRunExplicit::pure(value * 10)
		},
	));
	let post_order = StdArc::clone(&order);

	let result: EmptyArcRunExplicit<'_, i32> =
		carrier.resume_arc_with_post_action(&HandlersNil, move |value| {
			assert_eq!(post_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRunExplicit::pure(value + 1)
		});

	assert_eq!(result.extract(), 410);
	assert_eq!(order.load(Ordering::SeqCst), 2);
}

#[test]
fn scoped_continuation_repeats_action_transform_before_outer_continuation() {
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let carrier = ScopedContinuation::new(arc_explicit_scoped_continuation(
		EmptyArcRunExplicit::pure(40),
		move |value| {
			let step = outer_order.fetch_add(1, Ordering::SeqCst);
			assert!(step == 1 || step == 3);
			EmptyArcRunExplicit::pure(value * 10)
		},
	));

	let first_order = StdArc::clone(&order);
	let first: EmptyArcRunExplicit<'_, i32> =
		carrier.clone().resume_arc_with_action_transform(&HandlersNil, move |action| {
			let transform_order = StdArc::clone(&first_order);
			action.bind(move |value| {
				assert_eq!(transform_order.fetch_add(1, Ordering::SeqCst), 0);
				EmptyArcRunExplicit::pure(value + 1)
			})
		});
	let second_order = StdArc::clone(&order);
	let second: EmptyArcRunExplicit<'_, i32> =
		carrier.resume_arc_with_action_transform(&HandlersNil, move |action| {
			let transform_order = StdArc::clone(&second_order);
			action.bind(move |value| {
				assert_eq!(transform_order.fetch_add(1, Ordering::SeqCst), 2);
				EmptyArcRunExplicit::pure(value + 2)
			})
		});

	assert_eq!(first.extract(), 410);
	assert_eq!(second.extract(), 420);
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn scoped_continuation_repeats_post_action_before_outer_continuation() {
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let carrier = ScopedContinuation::new(arc_explicit_scoped_continuation(
		EmptyArcRunExplicit::pure(10),
		move |value| {
			let step = outer_order.fetch_add(1, Ordering::SeqCst);
			assert!(step == 1 || step == 3);
			EmptyArcRunExplicit::pure(value * 2)
		},
	));

	let first_order = StdArc::clone(&order);
	let first: EmptyArcRunExplicit<'_, i32> =
		carrier.clone().resume_arc_with_post_action(&HandlersNil, move |value| {
			assert_eq!(first_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRunExplicit::pure(value + 1)
		});

	let second_order = StdArc::clone(&order);
	let second: EmptyArcRunExplicit<'_, i32> =
		carrier.resume_arc_with_post_action(&HandlersNil, move |value| {
			assert_eq!(second_order.fetch_add(1, Ordering::SeqCst), 2);
			EmptyArcRunExplicit::pure(value + 2)
		});

	assert_eq!(first.extract(), 22);
	assert_eq!(second.extract(), 24);
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn scoped_continuation_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let carrier = ScopedContinuation::new(arc_explicit_scoped_continuation(
		EmptyArcRunExplicit::pure(label.as_str()),
		|value: &str| EmptyArcRunExplicit::pure(value.len()),
	));

	let result: EmptyArcRunExplicit<'_, usize> =
		carrier.resume_arc_with_post_action(&HandlersNil, EmptyArcRunExplicit::pure);

	assert_eq!(result.extract(), label.len());
}

#[test]
fn scoped_continuation_action_transform_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let carrier = ScopedContinuation::new(arc_explicit_scoped_continuation(
		EmptyArcRunExplicit::pure(label.as_str()),
		|value: &str| EmptyArcRunExplicit::pure(value.len()),
	));

	let result: EmptyArcRunExplicit<'_, usize> = carrier
		.resume_arc_with_action_transform(&HandlersNil, |action| {
			action.bind(EmptyArcRunExplicit::pure)
		});

	assert_eq!(result.extract(), label.len());
}

#[test]
fn action_supplied_scoped_continuation_runs_supplied_action_before_outer_continuation() {
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let carrier =
		ScopedContinuation::new(arc_explicit_action_supplied_scoped_continuation(move |value| {
			assert_eq!(outer_order.fetch_add(1, Ordering::SeqCst), 2);
			EmptyArcRunExplicit::pure(value * 10)
		}));

	let supplied_action_order = StdArc::clone(&order);
	let result: EmptyArcRunExplicit<'_, i32> =
		carrier.resume_arc_with_supplied_action(&HandlersNil, move || {
			let first_order = StdArc::clone(&supplied_action_order);
			EmptyArcRunExplicit::pure(40).bind(move |value| {
				assert_eq!(first_order.fetch_add(1, Ordering::SeqCst), 0);
				let second_order = StdArc::clone(&first_order);
				EmptyArcRunExplicit::pure(value + 1).bind(move |value| {
					assert_eq!(second_order.fetch_add(1, Ordering::SeqCst), 1);
					EmptyArcRunExplicit::pure(value)
				})
			})
		});

	assert_eq!(result.extract(), 410);
	assert_eq!(order.load(Ordering::SeqCst), 3);
}

#[test]
fn bracket_carrier_dispatcher_keeps_send_sync_lifecycle_ordering() {
	let order = StdArc::new(AtomicUsize::new(0));
	let acquire_order = StdArc::clone(&order);
	let body_order = StdArc::clone(&order);
	let release_order = StdArc::clone(&order);
	let outer_order = StdArc::clone(&order);
	let layer = RunExplicitBracketCarrierLayer::<ArcBrand, i32, i32, _, _, _, _>::new(
		move || {
			assert_eq!(acquire_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRunExplicit::pure(7)
		},
		move |resource: StdArc<i32>| {
			assert_eq!(body_order.fetch_add(1, Ordering::SeqCst), 1);
			EmptyArcRunExplicit::pure((*resource, *resource + 35))
		},
		move |resource: StdArc<i32>| {
			assert_eq!(release_order.fetch_add(1, Ordering::SeqCst), 2);
			assert_eq!(*resource, 7);
			EmptyArcRunExplicit::pure(())
		},
		ScopedContinuation::new(arc_explicit_action_supplied_scoped_continuation(move |value| {
			assert_eq!(outer_order.fetch_add(1, Ordering::SeqCst), 3);
			EmptyArcRunExplicit::pure(value)
		})),
	);

	let result: EmptyArcRunExplicit<'_, i32> =
		bracket_handler().dispatch_arc_run_explicit_bracket_carrier(layer, &HandlersNil);

	assert_eq!(result.extract(), 42);
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn ref_bracket_carrier_dispatcher_keeps_send_sync_lifecycle_ordering() {
	let order = StdArc::new(AtomicUsize::new(0));
	let acquire_order = StdArc::clone(&order);
	let body_order = StdArc::clone(&order);
	let release_order = StdArc::clone(&order);
	let outer_order = StdArc::clone(&order);
	let layer = RunExplicitRefBracketCarrierLayer::<ArcBrand, i32, i32, _, _, _, _>::new(
		move || {
			assert_eq!(acquire_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRunExplicit::pure(7)
		},
		move |resource: StdArc<i32>| {
			assert_eq!(body_order.fetch_add(1, Ordering::SeqCst), 1);
			assert_eq!(StdArc::strong_count(&resource), 2);
			EmptyArcRunExplicit::pure(*resource + 35)
		},
		move |resource: StdArc<i32>| {
			assert_eq!(release_order.fetch_add(1, Ordering::SeqCst), 2);
			assert_eq!(StdArc::strong_count(&resource), 2);
			assert_eq!(*resource, 7);
			EmptyArcRunExplicit::pure(())
		},
		ScopedContinuation::new(arc_explicit_action_supplied_scoped_continuation(move |value| {
			assert_eq!(outer_order.fetch_add(1, Ordering::SeqCst), 3);
			EmptyArcRunExplicit::pure(value)
		})),
	);

	let result: EmptyArcRunExplicit<'_, i32> =
		ref_bracket_handler().dispatch_arc_run_explicit_ref_bracket_carrier(layer, &HandlersNil);

	assert_eq!(result.extract(), 42);
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn ref_local_carrier_dispatcher_keeps_send_sync_reader_interpose() {
	fn modify(env: &i32) -> i32 {
		*env + 5
	}

	fn outer(value: i32) -> ArcReaderRunExplicit<'static, i32> {
		ArcReaderRunExplicit::pure(value + 1)
	}

	let action: ArcReaderRunExplicit<'static, i32> =
		ArcRunExplicit::<ArcReaderRow, CNilBrand, i32>::ask::<_>()
			.bind(|env| ArcRunExplicit::pure(env * 2));
	let layer = RunExplicitRefLocalCarrierLayer::<i32, _, _>::new(
		modify,
		ScopedContinuation::new(ArcRunExplicitScopedContinuation {
			action,
			outer: <ArcBrand as RefCountedPointer>::new(outer),
			result: PhantomData,
		}),
	);

	let program: ArcReaderRunExplicit<'static, i32> =
		ref_local_handler::<_, ArcReaderRowMinusReader, _>()
			.dispatch_arc_run_explicit_ref_local_carrier(layer, &HandlersNil);
	let result = program.handle(
			crate::handlers! {
				SendReaderBrand<ArcBrand, i32>: |op: SendReader<'_, ArcBrand, i32, ArcReaderRunExplicit<'static, i32>>| match op {
					SendReader::Ask(k) => k(10),
				},
			},
			crate::types::effects::scoped_nt(),
		);

	assert_eq!(result, 31);
}

#[test]
fn catch_carrier_dispatcher_keeps_send_sync_recovery_ordering() {
	fn recover(err: &'static str) -> ArcExceptRunExplicit<'static, i32> {
		assert_eq!(err, "from-action");
		ArcExceptRunExplicit::pure(41)
	}

	fn outer(value: i32) -> ArcExceptRunExplicit<'static, i32> {
		ArcExceptRunExplicit::pure(value + 1)
	}

	let action: ArcExceptRunExplicit<'static, i32> =
		ArcRunExplicit::throw::<&'static str, _>("from-action");
	let layer = RunExplicitCatchCarrierLayer::<&'static str, _, _>::new(
		recover,
		ScopedContinuation::new(ArcRunExplicitScopedContinuation {
			action,
			outer: <ArcBrand as RefCountedPointer>::new(outer),
			result: PhantomData,
		}),
	);

	let program: ArcExceptRunExplicit<'static, i32> =
		catch_handler::<_, ArcExceptRowMinusExcept, _>()
			.dispatch_arc_run_explicit_catch_carrier(layer, &HandlersNil);
	let result = program.handle(
		crate::handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, ArcExceptRunExplicit<'static, i32>>| {
				ArcExceptRunExplicit::pure(-1)
			},
		},
		crate::types::effects::scoped_nt(),
	);

	assert_eq!(result, 42);
}

#[test]
fn span_handler_repeats_send_sync_carrier_layer_before_outer_continuation() {
	let label = String::from("borrowed-value");
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let layer = RunExplicitSpanCarrierLayer::new(
		"request",
		ScopedContinuation::new(arc_explicit_scoped_continuation(
			EmptyArcRunExplicit::pure(label.as_str()),
			move |value: &str| {
				let step = outer_order.fetch_add(1, Ordering::SeqCst);
				assert!(step == 1 || step == 3);
				EmptyArcRunExplicit::pure(value.len())
			},
		)),
	);

	let first_order = StdArc::clone(&order);
	let first: EmptyArcRunExplicit<'_, usize> = span_handler()
		.dispatch_arc_run_explicit_span_carrier_with_post_action(
			layer.clone(),
			&HandlersNil,
			move |tag, value| {
				assert_eq!(*tag, "request");
				assert_eq!(first_order.fetch_add(1, Ordering::SeqCst), 0);
				EmptyArcRunExplicit::pure(value)
			},
		);
	let second_order = StdArc::clone(&order);
	let second: EmptyArcRunExplicit<'_, usize> = span_handler()
		.dispatch_arc_run_explicit_span_carrier_with_post_action(
			layer,
			&HandlersNil,
			move |tag, value| {
				assert_eq!(*tag, "request");
				assert_eq!(second_order.fetch_add(1, Ordering::SeqCst), 2);
				EmptyArcRunExplicit::pure(value)
			},
		);

	assert_eq!(first.extract(), label.len());
	assert_eq!(second.extract(), label.len());
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn cross_thread_via_spawn() {
	let run: RunAlias<'static, _> =
		ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(10));
	let mapped = run.map(|x: i32| x * 4);
	let handle = std::thread::spawn(move || mapped.into_arc_free_explicit().evaluate());
	assert_eq!(handle.join().expect("thread panicked"), 40);
}

#[test]
fn arc_run_explicit_is_send_sync() {
	fn assert_send_sync<T: Send + Sync>(_: &T) {}
	let run = ArcRunExplicit::<'_, FirstRow, Scoped, i32>::from_arc_free_explicit(
		ArcFreeExplicit::pure(7),
	);
	assert_send_sync(&run);
}

#[test]
fn non_static_payload() {
	let s = String::from("hello");
	let r: &str = &s;
	let run: ArcRunExplicit<'_, FirstRow, Scoped, &str> =
		ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(r));
	assert_eq!(run.into_arc_free_explicit().evaluate(), "hello");
}

#[test]
fn pure_then_peel_returns_value() {
	let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::pure(42);
	assert!(matches!(run.peel(), Ok(42)));
}

#[test]
fn send_produces_suspended_program() {
	use crate::types::{
		Identity,
		effects::{
			coproduct::Coproduct,
			node::Node,
		},
	};
	let layer = Coproduct::inject(Identity(7));
	let run: ArcRunExplicit<'_, FirstRow, Scoped, i32> = ArcRunExplicit::send(Node::First(layer));
	assert!(run.peel().is_err());
}

#[test]
fn from_erased_round_trips_pure() {
	use crate::types::effects::arc_run::ArcRun;
	let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::pure(42);
	let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::from(arc_run);
	assert!(matches!(explicit.peel(), Ok(42)));
}

#[test]
fn from_erased_preserves_suspended_layer() {
	use crate::types::{
		Identity,
		effects::{
			arc_run::ArcRun,
			coproduct::Coproduct,
			node::Node,
		},
	};
	let layer = Coproduct::inject(Identity(7));
	let arc_run: ArcRun<FirstRow, Scoped, i32> = ArcRun::send(Node::First(layer));
	let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::from(arc_run);
	assert!(explicit.peel().is_err());
}

#[test]
fn ref_bind_chains_pure_value_via_clone() {
	let run: RunAlias<'_, i32> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(2));
	let chained = run
		.ref_bind(|x: &i32| ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(*x + 1)));
	assert_eq!(chained.into_arc_free_explicit().evaluate(), 3);
}

#[test]
fn ref_map_transforms_pure_value_via_clone() {
	let run: RunAlias<'_, i32> = ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::pure(7));
	let mapped = run.ref_map(|x: &i32| *x * 3);
	assert_eq!(mapped.into_arc_free_explicit().evaluate(), 21);
}

#[test]
fn ref_pure_wraps_cloned_value() {
	let value = 42;
	let run: RunAlias<'_, i32> = ArcRunExplicit::ref_pure(&value);
	assert_eq!(run.into_arc_free_explicit().evaluate(), 42);
}
