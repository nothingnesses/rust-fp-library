use {
	crate::{
		brands::{
			BoxBrand,
			BoxSpanBrand,
			CNilBrand,
			CoproductBrand,
			IdentityBrand,
		},
		classes::ToDynFnOnce,
		impl_kind,
		kinds::{
			InferableBrand_266801a817966495,
			Kind_266801a817966495,
		},
		types::{
			Identity,
			effects::{
				coproduct::{
					CNil,
					Coproduct,
					Here,
					There,
				},
				handlers::HandlersNil,
				interpreter::inner::{
					DefaultScopedResume,
					DispatchHandlers,
					DispatchResidualScopedHandlers,
					DispatchScopedBoundaryHeadHandlers,
					DispatchScopedCarrierHandler,
					DispatchScopedCarrierHandlers,
					DispatchScopedHandler,
					IntoScopedBoundaryParts,
					ScopedBoundaryOf,
					ScopedBoundaryTypes,
					ScopedContinuation,
					ScopedResumeTypes,
				},
				run_explicit::{
					RunExplicit,
					RunExplicitBoundary,
				},
				scoped_nt,
				span::BoxSpan,
			},
		},
	},
	std::marker::PhantomData,
};

type DefaultSpanScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, &'static str>, CNilBrand>;
type DefaultSpanActionProgram<'a, Action> =
	<DefaultSpanScopedRow as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, Action>;
type DefaultSpanFinalProgram<Final> = Final;

fn assert_type_eq<T>(
	_: PhantomData<T>,
	_: PhantomData<T>,
) {
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DefaultSpanBoundaryBrand;

impl_kind! {
	impl for DefaultSpanBoundaryBrand {
		type Of<'a, Action: 'a, Final: 'a>: 'a =
			DefaultSpanBoundary<'a, Action, Final>;
	}
}

impl<'a, Action: 'a, Final: 'a> ScopedBoundaryTypes<'a, Action, Final>
	for DefaultSpanBoundaryBrand
{
	type ActionProgram = DefaultSpanActionProgram<'a, Action>;
	type FinalProgram = DefaultSpanFinalProgram<Final>;
}

struct DefaultSpanBoundary<'a, Action, Final>
where
	Action: 'a,
	Final: 'a, {
	layer: DefaultSpanActionProgram<'a, Action>,
	outer: Box<dyn FnOnce(Action) -> DefaultSpanFinalProgram<Final> + 'a>,
}

impl<'a, Action: 'a, Final: 'a> DefaultSpanBoundary<'a, Action, Final> {
	fn new(
		layer: DefaultSpanActionProgram<'a, Action>,
		outer: impl FnOnce(Action) -> DefaultSpanFinalProgram<Final> + 'a,
	) -> Self {
		Self {
			layer,
			outer: Box::new(outer),
		}
	}

	fn map_final<Next: 'a>(
		self,
		map: impl FnOnce(Final) -> Next + 'a,
	) -> DefaultSpanBoundary<'a, Action, Next> {
		let Self {
			layer,
			outer,
		} = self;
		DefaultSpanBoundary {
			layer,
			outer: Box::new(move |action| map(outer(action))),
		}
	}

	fn bind_final<Next: 'a>(
		self,
		bind: impl FnOnce(Final) -> DefaultSpanFinalProgram<Next> + 'a,
	) -> DefaultSpanBoundary<'a, Action, Next> {
		let Self {
			layer,
			outer,
		} = self;
		DefaultSpanBoundary {
			layer,
			outer: Box::new(move |action| bind(outer(action))),
		}
	}
}

#[derive(Clone, Copy, Debug)]
struct ResumeTo(i32);

impl<'a> ScopedResumeTypes<'a> for ResumeTo {
	type ActionProgram = i32;
	type ActionValue = i32;
	type OperationProgram = i32;
	type OperationValue = i32;
}

impl<'a> DefaultScopedResume<'a, CNil, i32> for ResumeTo {
	fn resume_default(
		self,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
	) -> i32 {
		self.0
	}

	fn resume_default_with_post_action(
		self,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		post_action: impl Fn(i32) -> i32 + 'a,
	) -> i32 {
		post_action(self.0)
	}

	fn resume_default_with_action_transform(
		self,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
		transform: impl Fn(i32) -> i32 + 'a,
	) -> i32 {
		transform(self.0)
	}
}

struct AddAfterAction;

impl<'a> DispatchScopedCarrierHandler<'a, Identity<i32>, CNil, i32, ResumeTo> for AddAfterAction {
	fn dispatch_scoped_carrier_head(
		&self,
		layer: Identity<i32>,
		continuation: ScopedContinuation<ResumeTo>,
		fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
	) -> i32 {
		let amount = layer.0;
		continuation.resume_default_with_post_action(fo_handlers, move |action_result| {
			action_result + amount
		})
	}
}

struct ReturnIdentity;

impl<'a> DispatchScopedHandler<'a, Identity<i32>, CNil, i32> for ReturnIdentity {
	fn dispatch_scoped_head(
		&self,
		layer: Identity<i32>,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, i32>,
	) -> i32 {
		layer.0
	}
}

#[derive(Clone, Copy, Debug)]
struct BorrowedResume<'a> {
	resumed: &'a str,
}

impl<'a> ScopedResumeTypes<'a> for BorrowedResume<'a> {
	type ActionProgram = &'a str;
	type ActionValue = &'a str;
	type OperationProgram = &'a str;
	type OperationValue = &'a str;
}

impl<'a> DefaultScopedResume<'a, CNil, String> for BorrowedResume<'a> {
	fn resume_default(
		self,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, String>,
	) -> String {
		self.resumed.to_owned()
	}

	fn resume_default_with_post_action(
		self,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, String>,
		post_action: impl Fn(&'a str) -> &'a str + 'a,
	) -> String {
		let post_value = post_action(self.resumed);
		format!("resume={};post={post_value}", self.resumed)
	}

	fn resume_default_with_action_transform(
		self,
		_fo_handlers: &impl DispatchHandlers<'a, CNil, String>,
		transform: impl Fn(&'a str) -> &'a str + 'a,
	) -> String {
		let transformed = transform(self.resumed);
		format!("resume={};transform={transformed}", self.resumed)
	}
}

struct RecordBorrowedSpanAction;

impl<'a>
	DispatchScopedCarrierHandler<
		'a,
		BoxSpan<'a, BoxBrand, &'static str, &'a str>,
		CNil,
		String,
		BorrowedResume<'a>,
	> for RecordBorrowedSpanAction
{
	fn dispatch_scoped_carrier_head(
		&self,
		layer: BoxSpan<'a, BoxBrand, &'static str, &'a str>,
		continuation: ScopedContinuation<BorrowedResume<'a>>,
		fo_handlers: &impl DispatchHandlers<'a, CNil, String>,
	) -> String {
		match layer {
			BoxSpan::Span {
				tag,
				action,
			} => {
				let action_value = action(());
				continuation.resume_default_with_post_action(fo_handlers, move |resume_value| {
					assert_eq!(tag, "request");
					assert_eq!(resume_value, "resume");
					action_value
				})
			}
		}
	}
}

#[test]
fn neutral_two_slot_boundary_keeps_action_projection_separate_from_final_slot() {
	fn require_neutral_boundary<'a, Action: 'a, Final: 'a>(
		boundary: ScopedBoundaryOf<'a, DefaultSpanBoundaryBrand, Action, Final>
	) -> ScopedBoundaryOf<'a, DefaultSpanBoundaryBrand, Action, Final>
	where
		DefaultSpanBoundaryBrand: ScopedBoundaryTypes<
				'a,
				Action,
				Final,
				ActionProgram = DefaultSpanActionProgram<'a, Action>,
				FinalProgram = DefaultSpanFinalProgram<Final>,
			>, {
		boundary
	}

	let action_text = String::from("borrowed-action");
	let action_ref = action_text.as_str();
	let layer: DefaultSpanActionProgram<'_, &str> = Coproduct::Inl(BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_ref),
	});
	let boundary = DefaultSpanBoundary::new(layer, |value: &str| value.len())
		.map_final(|length| length + 1)
		.bind_final(|length| format!("length={length}"));
	let boundary = require_neutral_boundary(boundary);

	let DefaultSpanBoundary {
		layer,
		outer,
	} = boundary;
	let action_value = match layer {
		Coproduct::Inl(BoxSpan::Span {
			tag,
			action,
		}) => {
			assert_eq!(tag, "request");
			action(())
		}
		Coproduct::Inr(rest) => match rest {},
	};

	assert_eq!(action_value, "borrowed-action");
	assert_eq!(outer(action_value), "length=16");
}

#[test]
fn resumes_scoped_continuation() {
	let continuation = ScopedContinuation::new(ResumeTo(41));

	assert_eq!(continuation.resume_default(&HandlersNil), 41);
}

#[test]
fn inserts_post_action_before_outer_resume() {
	let continuation = ScopedContinuation::new(ResumeTo(41));

	assert_eq!(
		continuation
			.resume_default_with_post_action(&HandlersNil, |action_result| { action_result + 1 }),
		42
	);
}

#[test]
fn exposes_inner_carrier_for_wrapper_local_rewrites() {
	let continuation = ScopedContinuation::new(ResumeTo(41));

	assert_eq!(continuation.into_inner().0, 41);
}

#[test]
fn dispatches_carrier_aware_scoped_handler_head() {
	let handlers = scoped_nt().on::<IdentityBrand, _>(AddAfterAction);
	let layer = Coproduct::Inl(Identity(1));
	let continuation = ScopedContinuation::new(ResumeTo(41));

	let result = handlers.dispatch_scoped_carrier(layer, continuation, &HandlersNil);

	assert_eq!(result, 42);
}

#[test]
fn dispatches_span_carrier_with_borrowed_action_slot_and_distinct_final_program() {
	let action_text = String::from("action");
	let resume_text = String::from("resume");
	let action_ref = action_text.as_str();
	let resume_ref = resume_text.as_str();
	let handlers =
		scoped_nt().on::<BoxSpanBrand<BoxBrand, &'static str>, _>(RecordBorrowedSpanAction);
	let layer = Coproduct::Inl(BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_ref),
	});
	let continuation = ScopedContinuation::new(BorrowedResume {
		resumed: resume_ref,
	});

	let result = handlers.dispatch_scoped_carrier(layer, continuation, &HandlersNil);

	assert_eq!(result, "resume=resume;post=action");
}

#[test]
fn boundary_parts_expose_consumed_member_evidence() {
	type ConsumedBrand = BoxSpanBrand<BoxBrand, &'static str>;
	type ScopedRow = CoproductBrand<ConsumedBrand, CNilBrand>;
	type Prog = RunExplicit<'static, CNilBrand, ScopedRow, i32>;
	type Boundary = RunExplicitBoundary<
		'static,
		CNilBrand,
		ScopedRow,
		ConsumedBrand,
		Here,
		i32,
		i32,
		fn(i32) -> Prog,
		i32,
	>;

	assert_type_eq::<ConsumedBrand>(
		PhantomData,
		PhantomData::<<Boundary as IntoScopedBoundaryParts<'static>>::ConsumedBrand>,
	);
	assert_type_eq::<Here>(
		PhantomData,
		PhantomData::<<Boundary as IntoScopedBoundaryParts<'static>>::ConsumedIdx>,
	);
}

#[test]
fn boundary_head_dispatches_consumed_head_without_tail_carrier_obligation() {
	type ConsumedBrand = BoxSpanBrand<BoxBrand, &'static str>;
	type Layer<'a> = Coproduct<
		<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, &'a str>,
		Coproduct<Identity<&'a str>, CNil>,
	>;

	let action_text = String::from("action");
	let resume_text = String::from("resume");
	let action_ref = action_text.as_str();
	let resume_ref = resume_text.as_str();
	let handlers = scoped_nt()
		.on::<IdentityBrand, _>(ReturnIdentity)
		.on::<ConsumedBrand, _>(RecordBorrowedSpanAction);
	let layer: Layer<'_> = Coproduct::Inl(BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_ref),
	});
	let continuation = ScopedContinuation::new(BorrowedResume {
		resumed: resume_ref,
	});

	let result = <_ as DispatchScopedBoundaryHeadHandlers<
		'_,
		ConsumedBrand,
		Here,
		Layer<'_>,
		CNil,
		String,
		BorrowedResume<'_>,
	>>::dispatch_scoped_boundary_head(&handlers, layer, continuation, &HandlersNil);

	assert_eq!(result, "resume=resume;post=action");
}

#[test]
fn boundary_head_dispatch_skips_prefix_without_prefix_carrier_obligation() {
	type ConsumedBrand = BoxSpanBrand<BoxBrand, &'static str>;
	type Layer<'a> = Coproduct<
		Identity<&'a str>,
		Coproduct<<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, &'a str>, CNil>,
	>;

	let action_text = String::from("action");
	let resume_text = String::from("resume");
	let action_ref = action_text.as_str();
	let resume_ref = resume_text.as_str();
	let handlers = scoped_nt()
		.on::<ConsumedBrand, _>(RecordBorrowedSpanAction)
		.on::<IdentityBrand, _>(ReturnIdentity);
	let layer: Layer<'_> = Coproduct::Inr(Coproduct::Inl(BoxSpan::Span {
		tag: "request",
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action_ref),
	}));
	let continuation = ScopedContinuation::new(BorrowedResume {
		resumed: resume_ref,
	});

	let result = <_ as DispatchScopedBoundaryHeadHandlers<
		'_,
		ConsumedBrand,
		There<Here>,
		Layer<'_>,
		CNil,
		String,
		BorrowedResume<'_>,
	>>::dispatch_scoped_boundary_head(&handlers, layer, continuation, &HandlersNil);

	assert_eq!(result, "resume=resume;post=action");
}

#[test]
fn residual_scoped_dispatch_skips_consumed_head_without_ordinary_handler() {
	type ConsumedBrand = BoxSpanBrand<BoxBrand, &'static str>;
	type Layer<'a> = Coproduct<
		<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, i32>,
		Coproduct<Identity<i32>, CNil>,
	>;

	let handlers = scoped_nt()
		.on::<IdentityBrand, _>(ReturnIdentity)
		.on::<ConsumedBrand, _>(RecordBorrowedSpanAction);
	let layer: Layer<'_> = Coproduct::Inr(Coproduct::Inl(Identity(42)));

	let result = <_ as DispatchResidualScopedHandlers<
		'_,
		ConsumedBrand,
		Here,
		Layer<'_>,
		CNil,
		i32,
	>>::dispatch_residual_scoped(&handlers, layer, &HandlersNil);

	assert_eq!(result, 42);
}

#[test]
fn residual_scoped_dispatch_preserves_heads_before_consumed_position() {
	type ConsumedBrand = BoxSpanBrand<BoxBrand, &'static str>;
	type Layer<'a> = Coproduct<
		Identity<i32>,
		Coproduct<
			<ConsumedBrand as crate::kinds::Kind_cdc7cd43dac7585f>::Of<'a, i32>,
			Coproduct<Identity<i32>, CNil>,
		>,
	>;

	let handlers = scoped_nt()
		.on::<IdentityBrand, _>(ReturnIdentity)
		.on::<ConsumedBrand, _>(RecordBorrowedSpanAction)
		.on::<IdentityBrand, _>(ReturnIdentity);
	let head_layer: Layer<'_> = Coproduct::Inl(Identity(41));
	let tail_layer: Layer<'_> = Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(Identity(42))));

	let head_result = <_ as DispatchResidualScopedHandlers<
		'_,
		ConsumedBrand,
		There<Here>,
		Layer<'_>,
		CNil,
		i32,
	>>::dispatch_residual_scoped(&handlers, head_layer, &HandlersNil);
	let tail_result = <_ as DispatchResidualScopedHandlers<
		'_,
		ConsumedBrand,
		There<Here>,
		Layer<'_>,
		CNil,
		i32,
	>>::dispatch_residual_scoped(&handlers, tail_layer, &HandlersNil);

	assert_eq!(head_result, 41);
	assert_eq!(tail_result, 42);
}
