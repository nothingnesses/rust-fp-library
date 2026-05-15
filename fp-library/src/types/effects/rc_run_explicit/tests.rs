use {
	super::*,
	crate::{
		brands::{
			CNilBrand,
			CoproductBrand,
			ExceptBrand,
			IdentityBrand,
			RcBrand,
			RcCoyonedaBrand,
			RcRunExplicitBrand,
			ReaderBrand,
		},
		classes::{
			Pointed,
			RefCountedPointer,
			RefFunctor,
			RefPointed,
			RefSemimonad,
		},
		impl_kind,
		kinds::{
			InferableBrand_266801a817966495,
			Kind_266801a817966495,
		},
		types::{
			RcFreeExplicit,
			effects::{
				except::Except,
				handlers::HandlersNil,
				interpreter::{
					ExplicitBoundaryOf,
					ExplicitBoundaryTypes,
					ScopedBoundaryTypes,
					ScopedContinuation,
				},
				reader::Reader,
				run_explicit::{
					RunExplicitBracketCarrierLayer,
					RunExplicitCatchCarrierLayer,
					RunExplicitLocalCarrierLayer,
					RunExplicitRefBracketCarrierLayer,
					RunExplicitSpanCarrierLayer,
				},
				standard_scoped_handlers::{
					bracket_handler,
					catch_handler,
					local_handler,
					ref_bracket_handler,
					span_handler,
				},
			},
		},
	},
	core::{
		cell::RefCell,
		marker::PhantomData,
	},
	std::rc::Rc as StdRc,
};

type FirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
type Scoped = CNilBrand;
type RunAlias<'a, A> = RcRunExplicit<'a, FirstRow, Scoped, A>;
type EmptyRcRunExplicit<'a, A> = RcRunExplicit<'a, CNilBrand, CNilBrand, A>;
type RcReaderRow = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
type RcReaderRowMinusReader = CNilBrand;
type RcReaderRunExplicit<'a, A> = RcRunExplicit<'a, RcReaderRow, CNilBrand, A>;
type RcExceptRow = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type RcExceptRowMinusExcept = CNilBrand;
type RcExceptRunExplicit<'a, A> = RcRunExplicit<'a, RcExceptRow, CNilBrand, A>;
type RcSharedBoundaryActionProgram<'a, Action> = EmptyRcRunExplicit<'a, Action>;
type RcSharedBoundaryFinalProgram<'a, Final> = EmptyRcRunExplicit<'a, Final>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RcSharedExplicitBoundaryBrand;

impl_kind! {
	impl for RcSharedExplicitBoundaryBrand {
		type Of<'a, Action: 'a, Final: 'a>: 'a =
			RcSharedExplicitBoundary<'a, Action, Final>;
	}
}

impl<'a, Action: 'a, Final: 'a> ScopedBoundaryTypes<'a, Action, Final>
	for RcSharedExplicitBoundaryBrand
{
	type ActionProgram = RcSharedBoundaryActionProgram<'a, Action>;
	type FinalProgram = RcSharedBoundaryFinalProgram<'a, Final>;
}

struct RcSharedExplicitBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a, {
	action: RcSharedBoundaryActionProgram<'a, Action>,
	outer: StdRc<dyn Fn(Action) -> RcSharedBoundaryFinalProgram<'a, Final> + 'a>,
	result: PhantomData<fn(Action) -> Final>,
}

impl<'a, Action, Final> Clone for RcSharedExplicitBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a,
{
	fn clone(&self) -> Self {
		Self {
			action: self.action.clone(),
			outer: StdRc::clone(&self.outer),
			result: PhantomData,
		}
	}
}

impl<'a, Action, Final> RcSharedExplicitBoundary<'a, Action, Final>
where
	Action: Clone + 'a,
	Final: 'a,
{
	fn new(
		action: RcSharedBoundaryActionProgram<'a, Action>,
		outer: impl Fn(Action) -> RcSharedBoundaryFinalProgram<'a, Final> + 'a,
	) -> Self {
		Self {
			action,
			outer: StdRc::new(outer),
			result: PhantomData,
		}
	}

	fn resume_with_post_action(
		self,
		post_action: impl Fn(Action) -> RcSharedBoundaryActionProgram<'a, Action> + 'a,
	) -> RcSharedBoundaryFinalProgram<'a, Final> {
		let outer = StdRc::clone(&self.outer);

		self.action.bind(move |action_value| {
			let outer = StdRc::clone(&outer);
			post_action(action_value).bind(move |post_value| outer(post_value))
		})
	}
}

fn rc_explicit_scoped_continuation<'a, Action, Final, K>(
	action: EmptyRcRunExplicit<'a, Action>,
	outer: K,
) -> RcRunExplicitScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
where
	Action: Clone + 'a,
	Final: 'a,
	K: Fn(Action) -> EmptyRcRunExplicit<'a, Final> + 'a, {
	RcRunExplicitScopedContinuation {
		action,
		outer: <RcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

fn rc_explicit_action_supplied_scoped_continuation<'a, Action, Final, K>(
	outer: K
) -> RcRunExplicitActionSuppliedScopedContinuation<'a, CNilBrand, CNilBrand, Action, Final, K>
where
	Action: Clone + 'a,
	Final: 'a,
	K: Fn(Action) -> EmptyRcRunExplicit<'a, Final> + 'a, {
	RcRunExplicitActionSuppliedScopedContinuation {
		outer: <RcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

#[test]
fn shared_explicit_boundary_protocol_repeats_rc_resume() {
	fn require_private_boundary_protocol<'a, Action: 'a, Final: 'a>(
		boundary: ExplicitBoundaryOf<'a, RcSharedExplicitBoundaryBrand, Action, Final>
	) -> ExplicitBoundaryOf<'a, RcSharedExplicitBoundaryBrand, Action, Final>
	where
		RcSharedExplicitBoundaryBrand: ExplicitBoundaryTypes<
				'a,
				Action,
				Final,
				ActionProgram = RcSharedBoundaryActionProgram<'a, Action>,
				FinalProgram = RcSharedBoundaryFinalProgram<'a, Final>,
			>, {
		boundary
	}

	let events = RefCell::new(Vec::new());
	let boundary: ExplicitBoundaryOf<'_, RcSharedExplicitBoundaryBrand, i32, i32> =
		RcSharedExplicitBoundary::new(EmptyRcRunExplicit::pure(40), |value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value * 10)
		});
	let boundary = require_private_boundary_protocol(boundary);

	let first: EmptyRcRunExplicit<'_, i32> = boundary.clone().resume_with_post_action(|value| {
		events.borrow_mut().push("first-post");
		EmptyRcRunExplicit::pure(value + 1)
	});
	let second: EmptyRcRunExplicit<'_, i32> = boundary.resume_with_post_action(|value| {
		events.borrow_mut().push("second-post");
		EmptyRcRunExplicit::pure(value + 2)
	});

	assert_eq!(first.extract(), 410);
	assert_eq!(second.extract(), 420);
	assert_eq!(events.into_inner(), vec!["first-post", "outer", "second-post", "outer"]);
}

#[test]
fn from_and_into_round_trip() {
	let rc_free: RcFreeExplicit<'_, _, i32> = RcFreeExplicit::pure(42);
	let run: RunAlias<'_, i32> = RcRunExplicit::from_rc_free_explicit(rc_free);
	let _back = run.into_rc_free_explicit();
}

#[test]
fn clone_branches_are_cheap() {
	let run: RunAlias<'_, _> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
	let _branch = run.clone();
}

#[test]
fn brand_pure_evaluates() {
	let run: RunAlias<'_, _> = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(7);
	assert_eq!(run.into_rc_free_explicit().evaluate(), 7);
}

#[test]
fn inherent_map_evaluates() {
	let run: RunAlias<'_, _> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(10));
	let mapped = run.map(|x: i32| x * 3);
	assert_eq!(mapped.into_rc_free_explicit().evaluate(), 30);
}

#[test]
fn inherent_bind_evaluates() {
	let run: RunAlias<'_, _> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(2));
	let chained =
		run.bind(|x: i32| RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(x + 5)));
	assert_eq!(chained.into_rc_free_explicit().evaluate(), 7);
}

#[test]
fn scoped_continuation_repeats_action_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
		EmptyRcRunExplicit::pure(40),
		|value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value * 10)
		},
	));

	let first: EmptyRcRunExplicit<'_, i32> = carrier.clone().resume_rc(&HandlersNil);
	let second: EmptyRcRunExplicit<'_, i32> = carrier.resume_rc(&HandlersNil);

	assert_eq!(first.extract(), 400);
	assert_eq!(second.extract(), 400);
	assert_eq!(events.into_inner(), vec!["outer", "outer"]);
}

#[test]
fn scoped_continuation_transforms_action_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
		EmptyRcRunExplicit::pure(40),
		|value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value * 10)
		},
	));

	let result: EmptyRcRunExplicit<'_, i32> =
		carrier.resume_rc_with_post_action(&HandlersNil, |value| {
			events.borrow_mut().push("post");
			EmptyRcRunExplicit::pure(value + 1)
		});

	assert_eq!(result.extract(), 410);
	assert_eq!(events.into_inner(), vec!["post", "outer"]);
}

#[test]
fn scoped_continuation_repeats_action_transform_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
		EmptyRcRunExplicit::pure(40),
		|value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value * 10)
		},
	));

	let first: EmptyRcRunExplicit<'_, i32> =
		carrier.clone().resume_rc_with_action_transform(&HandlersNil, |action| {
			action.bind(|value| {
				events.borrow_mut().push("transform");
				EmptyRcRunExplicit::pure(value + 1)
			})
		});
	let second: EmptyRcRunExplicit<'_, i32> =
		carrier.resume_rc_with_action_transform(&HandlersNil, |action| {
			action.bind(|value| {
				events.borrow_mut().push("transform");
				EmptyRcRunExplicit::pure(value + 2)
			})
		});

	assert_eq!(first.extract(), 410);
	assert_eq!(second.extract(), 420);
	assert_eq!(events.into_inner(), vec!["transform", "outer", "transform", "outer"]);
}

#[test]
fn scoped_continuation_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
		EmptyRcRunExplicit::pure(label.as_str()),
		|value: &str| EmptyRcRunExplicit::pure(value.len()),
	));

	let result: EmptyRcRunExplicit<'_, usize> =
		carrier.resume_rc_with_post_action(&HandlersNil, EmptyRcRunExplicit::pure);

	assert_eq!(result.extract(), label.len());
}

#[test]
fn scoped_continuation_action_transform_preserves_borrowed_action_value() {
	let label = String::from("borrowed-value");
	let carrier = ScopedContinuation::new(rc_explicit_scoped_continuation(
		EmptyRcRunExplicit::pure(label.as_str()),
		|value: &str| EmptyRcRunExplicit::pure(value.len()),
	));

	let result: EmptyRcRunExplicit<'_, usize> = carrier
		.resume_rc_with_action_transform(&HandlersNil, |action| {
			action.bind(EmptyRcRunExplicit::pure)
		});

	assert_eq!(result.extract(), label.len());
}

#[test]
fn action_supplied_scoped_continuation_repeats_supplied_action_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let carrier =
		ScopedContinuation::new(rc_explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value * 10)
		}));

	let first: EmptyRcRunExplicit<'_, i32> =
		carrier.clone().resume_rc_with_supplied_action(&HandlersNil, || {
			EmptyRcRunExplicit::pure(40).bind(|value| {
				events.borrow_mut().push("first-supplied-action");
				EmptyRcRunExplicit::pure(value + 1)
			})
		});
	let second: EmptyRcRunExplicit<'_, i32> =
		carrier.resume_rc_with_supplied_action(&HandlersNil, || {
			EmptyRcRunExplicit::pure(40).bind(|value| {
				events.borrow_mut().push("second-supplied-action");
				EmptyRcRunExplicit::pure(value + 2)
			})
		});

	assert_eq!(first.extract(), 410);
	assert_eq!(second.extract(), 420);
	assert_eq!(
		events.into_inner(),
		vec!["first-supplied-action", "outer", "second-supplied-action", "outer"]
	);
}

#[test]
fn bracket_carrier_dispatcher_repeats_lifecycle_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitBracketCarrierLayer::<RcBrand, i32, i32, _, _, _, _>::new(
		|| {
			events.borrow_mut().push("acquire");
			EmptyRcRunExplicit::pure(7)
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("body");
			EmptyRcRunExplicit::pure((*resource, *resource + 35))
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("release");
			assert_eq!(*resource, 7);
			EmptyRcRunExplicit::pure(())
		},
		ScopedContinuation::new(rc_explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value)
		})),
	);
	let dispatcher = bracket_handler();

	let first: EmptyRcRunExplicit<'_, i32> =
		dispatcher.dispatch_rc_run_explicit_bracket_carrier(layer.clone(), &HandlersNil);
	let second: EmptyRcRunExplicit<'_, i32> =
		dispatcher.dispatch_rc_run_explicit_bracket_carrier(layer, &HandlersNil);

	assert_eq!(first.extract(), 42);
	assert_eq!(second.extract(), 42);
	assert_eq!(
		events.into_inner(),
		vec!["acquire", "body", "release", "outer", "acquire", "body", "release", "outer"]
	);
}

#[test]
fn ref_bracket_carrier_dispatcher_repeats_lifecycle_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let layer = RunExplicitRefBracketCarrierLayer::<RcBrand, i32, i32, _, _, _, _>::new(
		|| {
			events.borrow_mut().push("acquire");
			EmptyRcRunExplicit::pure(7)
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("body");
			assert_eq!(std::rc::Rc::strong_count(&resource), 2);
			EmptyRcRunExplicit::pure(*resource + 35)
		},
		|resource: std::rc::Rc<i32>| {
			events.borrow_mut().push("release");
			assert_eq!(std::rc::Rc::strong_count(&resource), 2);
			assert_eq!(*resource, 7);
			EmptyRcRunExplicit::pure(())
		},
		ScopedContinuation::new(rc_explicit_action_supplied_scoped_continuation(|value| {
			events.borrow_mut().push("outer");
			EmptyRcRunExplicit::pure(value)
		})),
	);
	let dispatcher = ref_bracket_handler();

	let first: EmptyRcRunExplicit<'_, i32> =
		dispatcher.dispatch_rc_run_explicit_ref_bracket_carrier(layer.clone(), &HandlersNil);
	let second: EmptyRcRunExplicit<'_, i32> =
		dispatcher.dispatch_rc_run_explicit_ref_bracket_carrier(layer, &HandlersNil);

	assert_eq!(first.extract(), 42);
	assert_eq!(second.extract(), 42);
	assert_eq!(
		events.into_inner(),
		vec!["acquire", "body", "release", "outer", "acquire", "body", "release", "outer"]
	);
}

#[test]
fn local_carrier_dispatcher_repeats_reader_interpose() {
	let action: RcReaderRunExplicit<'static, i32> =
		RcRunExplicit::<RcReaderRow, CNilBrand, i32>::ask::<_>()
			.bind(|env| RcRunExplicit::pure(env * 2));
	let layer = RunExplicitLocalCarrierLayer::<i32, _, _>::new(
		|env| env + 1,
		ScopedContinuation::new(RcRunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(|value| {
				RcReaderRunExplicit::pure(value + 1)
			}),
			result: PhantomData,
		}),
	);
	let dispatcher = local_handler::<_, RcReaderRowMinusReader, _>();

	let first: RcReaderRunExplicit<'static, i32> =
		dispatcher.dispatch_rc_run_explicit_local_carrier(layer.clone(), &HandlersNil);
	let second: RcReaderRunExplicit<'static, i32> =
		dispatcher.dispatch_rc_run_explicit_local_carrier(layer, &HandlersNil);
	let first_result = first.interpret(
			crate::handlers! {
				ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcReaderRunExplicit<'static, i32>>| match op {
					Reader::Ask(k) => k(10),
				},
			},
			crate::types::effects::scoped_nt(),
		);
	let second_result = second.interpret(
			crate::handlers! {
				ReaderBrand<RcBrand, i32>: |op: Reader<'_, RcBrand, i32, RcReaderRunExplicit<'static, i32>>| match op {
					Reader::Ask(k) => k(10),
				},
			},
			crate::types::effects::scoped_nt(),
		);

	assert_eq!(first_result, 23);
	assert_eq!(second_result, 23);
}

#[test]
fn catch_carrier_dispatcher_repeats_recovery_before_outer_continuation() {
	let action: RcExceptRunExplicit<'static, i32> =
		RcRunExplicit::throw::<&'static str, _>("from-action");
	let layer = RunExplicitCatchCarrierLayer::<&'static str, _, _>::new(
		|err| {
			assert_eq!(err, "from-action");
			RcExceptRunExplicit::pure(41)
		},
		ScopedContinuation::new(RcRunExplicitScopedContinuation {
			action,
			outer: <RcBrand as RefCountedPointer>::new(|value| {
				RcExceptRunExplicit::pure(value + 1)
			}),
			result: PhantomData,
		}),
	);
	let dispatcher = catch_handler::<_, RcExceptRowMinusExcept, _>();

	let first: RcExceptRunExplicit<'static, i32> =
		dispatcher.dispatch_rc_run_explicit_catch_carrier(layer.clone(), &HandlersNil);
	let second: RcExceptRunExplicit<'static, i32> =
		dispatcher.dispatch_rc_run_explicit_catch_carrier(layer, &HandlersNil);
	let first_result = first.interpret(
		crate::handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RcExceptRunExplicit<'static, i32>>| {
				RcExceptRunExplicit::pure(-1)
			},
		},
		crate::types::effects::scoped_nt(),
	);
	let second_result = second.interpret(
		crate::handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, RcExceptRunExplicit<'static, i32>>| {
				RcExceptRunExplicit::pure(-1)
			},
		},
		crate::types::effects::scoped_nt(),
	);

	assert_eq!(first_result, 42);
	assert_eq!(second_result, 42);
}

#[test]
fn span_handler_repeats_carrier_layer_before_outer_continuation() {
	let events = RefCell::new(Vec::new());
	let label = String::from("borrowed-value");
	let layer = RunExplicitSpanCarrierLayer::new(
		"request",
		ScopedContinuation::new(rc_explicit_scoped_continuation(
			EmptyRcRunExplicit::pure(label.as_str()),
			|value: &str| {
				events.borrow_mut().push("outer");
				EmptyRcRunExplicit::pure(value.len())
			},
		)),
	);

	let first: EmptyRcRunExplicit<'_, usize> = span_handler()
		.dispatch_rc_run_explicit_span_carrier_with_post_action(
			layer.clone(),
			&HandlersNil,
			|tag, value| {
				assert_eq!(*tag, "request");
				events.borrow_mut().push("post");
				EmptyRcRunExplicit::pure(value)
			},
		);
	let second: EmptyRcRunExplicit<'_, usize> = span_handler()
		.dispatch_rc_run_explicit_span_carrier_with_post_action(
			layer,
			&HandlersNil,
			|tag, value| {
				assert_eq!(*tag, "request");
				events.borrow_mut().push("post");
				EmptyRcRunExplicit::pure(value)
			},
		);

	assert_eq!(first.extract(), label.len());
	assert_eq!(second.extract(), label.len());
	assert_eq!(events.into_inner(), vec!["post", "outer", "post", "outer"]);
}

#[test]
fn brand_ref_pure_evaluates() {
	let value = 11;
	let run: RunAlias<'_, _> =
		<RcRunExplicitBrand<FirstRow, Scoped> as RefPointed>::ref_pure(&value);
	assert_eq!(run.into_rc_free_explicit().evaluate(), 11);
}

#[test]
fn brand_ref_map_evaluates() {
	let run = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(4);
	let mapped =
		<RcRunExplicitBrand<FirstRow, Scoped> as RefFunctor>::ref_map(|x: &i32| *x * 5, &run);
	assert_eq!(mapped.into_rc_free_explicit().evaluate(), 20);
}

#[test]
fn brand_ref_bind_evaluates() {
	let run = <RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(8);
	let chained =
		<RcRunExplicitBrand<FirstRow, Scoped> as RefSemimonad>::ref_bind(&run, |x: &i32| {
			<RcRunExplicitBrand<FirstRow, Scoped> as Pointed>::pure(*x + 1)
		});
	assert_eq!(chained.into_rc_free_explicit().evaluate(), 9);
}

#[test]
fn non_static_payload() {
	let s = String::from("hello");
	let r: &str = &s;
	let run: RcRunExplicit<'_, FirstRow, Scoped, &str> =
		RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(r));
	assert_eq!(run.into_rc_free_explicit().evaluate(), "hello");
}

#[test]
fn pure_then_peel_returns_value() {
	let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::pure(42);
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
	let run: RcRunExplicit<'_, FirstRow, Scoped, i32> = RcRunExplicit::send(Node::First(layer));
	assert!(run.peel().is_err());
}

#[test]
fn from_erased_round_trips_pure() {
	use crate::types::effects::rc_run::RcRun;
	let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::pure(42);
	let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::from(rc_run);
	assert!(matches!(explicit.peel(), Ok(42)));
}

#[test]
fn from_erased_preserves_suspended_layer() {
	use crate::types::{
		Identity,
		effects::{
			coproduct::Coproduct,
			node::Node,
			rc_run::RcRun,
		},
	};
	let layer = Coproduct::inject(Identity(7));
	let rc_run: RcRun<FirstRow, Scoped, i32> = RcRun::send(Node::First(layer));
	let explicit: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::from(rc_run);
	assert!(explicit.peel().is_err());
}

#[test]
fn ref_bind_chains_pure_value_via_clone() {
	let run: RunAlias<'_, i32> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(2));
	let chained =
		run.ref_bind(|x: &i32| RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(*x + 1)));
	assert_eq!(chained.into_rc_free_explicit().evaluate(), 3);
}

#[test]
fn ref_map_transforms_pure_value_via_clone() {
	let run: RunAlias<'_, i32> = RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::pure(7));
	let mapped = run.ref_map(|x: &i32| *x * 3);
	assert_eq!(mapped.into_rc_free_explicit().evaluate(), 21);
}

#[test]
fn ref_pure_wraps_cloned_value() {
	let value = 42;
	let run: RunAlias<'_, i32> = RcRunExplicit::ref_pure(&value);
	assert_eq!(run.into_rc_free_explicit().evaluate(), 42);
}
