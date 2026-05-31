use {
	super::*,
	crate::{
		brands::{
			BoxBrand,
			BoxCatchBrand,
			BoxLocalBrand,
			BoxReaderBrand,
			BoxStateBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			ExceptBrand,
			IdentityBrand,
			NodeBrand,
			OptionBrand,
		},
		classes::{
			Functor,
			ToDynFnOnce,
			WrapDrop,
		},
		handlers,
		kinds::Kind_cdc7cd43dac7585f,
		scoped_handlers,
		types::{
			CatList,
			Coyoneda,
			Free,
			Identity,
			effects::{
				catch::BoxCatch,
				coproduct::Coproduct,
				except::Except,
				handlers::HandlersNil,
				interpreter::ScopedContinuation,
				node::Node,
				reader::BoxReader,
				standard_scoped_handlers::{
					catch_handler,
					local_handler,
				},
				state::BoxState,
			},
			free::{
				Continuation,
				FreeRawStep,
				TypeErasedValue,
			},
		},
	},
	std::{
		cell::RefCell,
		rc::Rc,
	},
};

type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type WiderFirstRow = CoproductBrand<CoyonedaBrand<OptionBrand>, FirstRow>;
type Scoped = CNilBrand;
type RunAlias<A> = Run<FirstRow, Scoped, A>;
type EmptyNode = NodeBrand<CNilBrand, CNilBrand>;
type EmptyRawRun = RawRunFree<CNilBrand, CNilBrand>;
type EmptyRun<A> = Run<CNilBrand, CNilBrand, A>;
type CatchScopedRow = CoproductBrand<BoxCatchBrand<BoxBrand, &'static str>, CNilBrand>;
type WiderCatchScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CatchScopedRow>;
type CatchNode = NodeBrand<CNilBrand, CatchScopedRow>;
type CatchRawRun = RawRunFree<CNilBrand, CatchScopedRow>;
type CatchRun<A> = Run<CNilBrand, CatchScopedRow, A>;
type IdentityCatchRawRun = RawRunFree<FirstRow, CatchScopedRow>;
type IdentityCatchRun<A> = Run<FirstRow, CatchScopedRow, A>;
type NarrowedCatchNode = NodeBrand<CNilBrand, CatchScopedRow>;
type NarrowedCatchRawRun = RawRunFree<CNilBrand, CatchScopedRow>;
type StateCatchFirstRow = CoproductBrand<
	CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>,
	CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>,
>;
type StateCatchFirstRowMinusState =
	CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
type StateCatchFirstRowMinusExcept =
	CoproductBrand<CoyonedaBrand<BoxStateBrand<BoxBrand, i32>>, CNilBrand>;
type StateCatchRun<A> = Run<StateCatchFirstRow, CatchScopedRow, A>;
type StateCatchMinusStateRun<A> = Run<StateCatchFirstRowMinusState, CatchScopedRow, A>;
type ReaderLocalFirstRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type ReaderLocalFirstRowMinusReader = CNilBrand;
type ReaderLocalScopedRow = CoproductBrand<BoxLocalBrand<BoxBrand, i32>, CNilBrand>;
type ReaderLocalRun<A> = Run<ReaderLocalFirstRow, ReaderLocalScopedRow, A>;

struct IdentityPolymorphicHandler;

impl<RMinusE, S> RunFirstOrderHandler<IdentityBrand, RMinusE, S> for IdentityPolymorphicHandler
where
	RMinusE: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
{
	fn handle<T: 'static>(
		&self,
		effect: Identity<Run<RMinusE, S, T>>,
	) -> Run<RMinusE, S, T> {
		effect.0
	}
}

struct IdentityPassthroughReplacer;

impl<R, S> RunFirstOrderReplacer<IdentityBrand, R, S> for IdentityPassthroughReplacer
where
	R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
	S: WrapDrop + Functor + 'static,
{
	fn replace<T: 'static>(
		&self,
		effect: Identity<Run<R, S, T>>,
	) -> Run<R, S, T> {
		effect.0
	}
}

struct StatePolymorphicHandler {
	state: Rc<RefCell<i32>>,
}

impl
	RunFirstOrderHandler<BoxStateBrand<BoxBrand, i32>, StateCatchFirstRowMinusState, CatchScopedRow>
	for StatePolymorphicHandler
{
	fn handle<T: 'static>(
		&self,
		effect: BoxState<'static, BoxBrand, i32, StateCatchMinusStateRun<T>>,
	) -> StateCatchMinusStateRun<T> {
		match effect {
			BoxState::Get(k) => {
				let state = *self.state.borrow();
				k(state)
			}
			BoxState::Put(state, k) => {
				*self.state.borrow_mut() = state;
				k(())
			}
		}
	}
}

fn raw_i32(value: i32) -> EmptyRawRun {
	Free::<EmptyNode, _>::pure(value).cast_erased()
}

fn boxed_raw_i32(value: i32) -> EmptyRawRun {
	Free::<EmptyNode, _>::pure(value).erase_type()
}

fn multiply_by_ten_continuation() -> Continuation<EmptyNode> {
	Box::new(|value| {
		let value = match value.downcast::<i32>() {
			Ok(value) => *value,
			Err(_) => return raw_i32(0),
		};

		raw_i32(value * 10)
	})
}

fn increment_raw_i32_value(value: TypeErasedValue) -> EmptyRawRun {
	let value = match value.downcast::<i32>() {
		Ok(value) => *value,
		Err(_) => return raw_i32(0),
	};

	raw_i32(value + 1)
}

fn append_increment_to_raw_action(action: EmptyRawRun) -> EmptyRawRun {
	action.bind(increment_raw_i32_value)
}

fn run_scoped_continuation(
	action: EmptyRawRun
) -> RunScopedContinuation<CNilBrand, CNilBrand, i32> {
	RunScopedContinuation {
		action,
		continuations: CatList::singleton(multiply_by_ten_continuation()),
		result: core::marker::PhantomData,
	}
}

fn catch_raw_i32(value: i32) -> CatchRawRun {
	Free::<CatchNode, _>::pure(value).cast_erased()
}

fn identity_catch_raw_i32(value: i32) -> IdentityCatchRawRun {
	Run::<FirstRow, CatchScopedRow, i32>::lift::<IdentityBrand, _>(Identity(value))
		.into_free()
		.erase_type()
}

fn catch_boundary(action_value: i32) -> CatchRun<i32> {
	let action = catch_raw_i32(action_value);
	let catch: BoxCatch<'static, BoxBrand, &'static str, CatchRawRun> = BoxCatch::Catch {
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
		handler: <BoxBrand as ToDynFnOnce>::new(|_: &'static str| catch_raw_i32(0)),
	};
	let layer = Coproduct::inject(catch);
	Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
		layer,
		continuations: CatList::empty(),
		result: core::marker::PhantomData,
	}))
}

fn identity_catch_boundary(
	action_value: i32,
	recovery_value: i32,
) -> IdentityCatchRun<i32> {
	let action = identity_catch_raw_i32(action_value);
	let catch: BoxCatch<'static, BoxBrand, &'static str, IdentityCatchRawRun> = BoxCatch::Catch {
		action: <BoxBrand as ToDynFnOnce>::new(move |_: ()| action),
		handler: <BoxBrand as ToDynFnOnce>::new(move |_: &'static str| {
			identity_catch_raw_i32(recovery_value)
		}),
	};
	let layer = Coproduct::inject(catch);
	Run(RunRepresentation::ScopedBoundary(RunScopedBoundaryFrame {
		layer,
		continuations: CatList::empty(),
		result: core::marker::PhantomData,
	}))
}

fn public_catch(action_value: i32) -> CatchRun<i32> {
	Run::catch::<&'static str, _>(Run::pure(action_value), |_err| Run::pure(0))
}

fn assert_boundary_action_and_result<A>(
	program: CatchRun<A>,
	expected_action_value: i32,
	expected_result: A,
	expected_continuations: usize,
) where
	A: core::fmt::Debug + PartialEq + 'static, {
	let boundary = match program.0 {
		RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
		RunRepresentation::Free(_) => None,
	};
	assert!(boundary.is_some(), "expected scoped boundary representation");
	let Some(boundary) = boundary else {
		return;
	};

	assert_eq!(boundary.continuations.len(), expected_continuations);
	match boundary.layer {
		Coproduct::Inl(BoxCatch::Catch {
			action,
			handler: _,
		}) => {
			let action: Free<CatchNode, i32> =
				Free::continue_from_erased(action(()), CatList::empty());
			assert!(matches!(
				action.into_raw_step(),
				FreeRawStep::Done(value) if value == expected_action_value
			));

			let final_free: Free<CatchNode, A> = Free::continue_from_erased(
				catch_raw_i32(expected_action_value),
				boundary.continuations,
			);
			assert!(matches!(
				final_free.into_raw_step(),
				FreeRawStep::Done(value) if value == expected_result
			));
		}
		Coproduct::Inr(cnil) => match cnil {},
	}
}

fn assert_boundary_handler_and_result<A>(
	program: CatchRun<A>,
	expected_recovery_value: i32,
	expected_result: A,
	expected_continuations: usize,
) where
	A: core::fmt::Debug + PartialEq + 'static, {
	let boundary = match program.0 {
		RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
		RunRepresentation::Free(_) => None,
	};
	assert!(boundary.is_some(), "expected scoped boundary representation");
	let Some(boundary) = boundary else {
		return;
	};

	assert_eq!(boundary.continuations.len(), expected_continuations);
	match boundary.layer {
		Coproduct::Inl(BoxCatch::Catch {
			action: _,
			handler,
		}) => {
			let recovery: Free<CatchNode, i32> =
				Free::continue_from_erased(handler("oops"), CatList::empty());
			assert!(matches!(
				recovery.into_raw_step(),
				FreeRawStep::Done(value) if value == expected_recovery_value
			));

			let final_free: Free<CatchNode, A> = Free::continue_from_erased(
				catch_raw_i32(expected_recovery_value),
				boundary.continuations,
			);
			assert!(matches!(
				final_free.into_raw_step(),
				FreeRawStep::Done(value) if value == expected_result
			));
		}
		Coproduct::Inr(cnil) => match cnil {},
	}
}

#[test]
fn from_free_and_into_free_round_trip() {
	let free: Free<NodeBrand<FirstRow, Scoped>, i32> = Free::pure(42);
	let run: RunAlias<i32> = Run::from_free(free);
	let _back: Free<NodeBrand<FirstRow, Scoped>, i32> = run.into_free();
}

#[test]
fn drop_a_pure_run_does_not_panic() {
	let run: RunAlias<i32> = Run::from_free(Free::pure(7));
	drop(run);
}

#[test]
fn pure_then_peel_returns_value() {
	let run: RunAlias<i32> = Run::pure(42);
	assert!(matches!(run.peel(), Ok(42)));
}

#[test]
fn send_produces_suspended_program() {
	let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
	let layer = Coproduct::inject(coyo);
	let run: RunAlias<i32> = Run::send(Node::First(layer));
	assert!(run.peel().is_err());
}

#[test]
fn pure_uses_free_backed_representation() {
	let run: RunAlias<i32> = Run::pure(42);
	assert!(matches!(run.0, RunRepresentation::Free(_)));
}

#[test]
fn first_order_send_uses_free_backed_representation() {
	let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
	let layer = Coproduct::inject(coyo);
	let run: RunAlias<i32> = Run::send(Node::First(layer));
	assert!(matches!(run.0, RunRepresentation::Free(_)));
}

#[test]
fn result_polymorphic_handler_narrows_free_backed_first_order_step() {
	let run: RunAlias<i32> = Run::lift::<IdentityBrand, _>(Identity(42));

	let narrowed: EmptyRun<i32> =
		run.handle_with_handler::<IdentityBrand, _, CNilBrand>(IdentityPolymorphicHandler);

	assert_eq!(narrowed.extract(), 42);
}

#[test]
fn expand_widens_free_backed_first_order_row() {
	let run: RunAlias<i32> = Run::lift::<IdentityBrand, _>(Identity(40)).map(|value| value + 2);

	let widened: Run<WiderFirstRow, Scoped, i32> = run.expand();

	let continuation_value = match widened.peel() {
		Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
			let Identity(next) = coyo.lower();
			next.peel().ok()
		}
		_ => None,
	};
	assert_eq!(continuation_value, Some(42));
}

#[test]
fn expand_widens_scoped_boundary_without_lowering_it() {
	let program = catch_boundary(7).map(|value| value + 1);

	let widened: Run<CNilBrand, WiderCatchScopedRow, i32> = program.expand();

	let boundary = match widened.0 {
		RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
		RunRepresentation::Free(_) => None,
	};
	assert!(boundary.is_some(), "expected scoped boundary representation");
	let Some(boundary) = boundary else {
		return;
	};

	assert_eq!(boundary.continuations.len(), 1);
	assert!(
		matches!(
			boundary.layer,
			Coproduct::Inr(Coproduct::Inl(BoxCatch::Catch {
				action: _,
				handler: _,
			}))
		),
		"expected catch layer embedded behind the new scoped row head",
	);
}

#[test]
fn run_catch_uses_scoped_boundary_representation() {
	let program = public_catch(7);

	assert_boundary_action_and_result(program, 7, 7, 0);
}

#[test]
fn result_polymorphic_handler_narrows_boundary_catch_branches_before_outer_continuation() {
	let program = identity_catch_boundary(41, 5).bind(|value| Run::pure(format!("value={value}")));
	let boundary = match program.0 {
		RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
		RunRepresentation::Free(_) => None,
	};
	assert!(boundary.is_some(), "expected scoped boundary representation");
	let Some(boundary) = boundary else {
		return;
	};

	assert_eq!(boundary.continuations.len(), 1);
	match boundary.layer {
		Coproduct::Inl(BoxCatch::Catch {
			action,
			handler,
		}) => {
			let narrowed_action: NarrowedCatchRawRun =
				Run::<FirstRow, CatchScopedRow, TypeErasedValue>::from_free(action(()))
					.handle_with_handler::<IdentityBrand, _, CNilBrand>(IdentityPolymorphicHandler)
					.into_free();
			let action: Free<NarrowedCatchNode, i32> =
				Free::continue_from_reboxed_erased(narrowed_action, CatList::empty());
			assert!(matches!(
				action.into_raw_step(),
				FreeRawStep::Done(value) if value == 41
			));

			let narrowed_recovery: NarrowedCatchRawRun =
				Run::<FirstRow, CatchScopedRow, TypeErasedValue>::from_free(handler("err"))
					.handle_with_handler::<IdentityBrand, _, CNilBrand>(IdentityPolymorphicHandler)
					.into_free();
			let recovery: Free<NarrowedCatchNode, i32> =
				Free::continue_from_reboxed_erased(narrowed_recovery, CatList::empty());
			assert!(matches!(
				recovery.into_raw_step(),
				FreeRawStep::Done(value) if value == 5
			));
		}
		Coproduct::Inr(cnil) => match cnil {},
	}
}

#[test]
fn result_polymorphic_replacer_preserves_boundary_representation() {
	let program = identity_catch_boundary(41, 5).bind(|value| Run::pure(format!("value={value}")));

	let interposed: IdentityCatchRun<String> = program
		.interpose_with_replacer::<IdentityBrand, _, CNilBrand, _>(IdentityPassthroughReplacer);

	let boundary = match interposed.0 {
		RunRepresentation::ScopedBoundary(boundary) => Some(boundary),
		RunRepresentation::Free(_) => None,
	};
	assert!(boundary.is_some(), "expected scoped boundary representation");
	let Some(boundary) = boundary else {
		return;
	};
	assert_eq!(boundary.continuations.len(), 1);
}

#[test]
fn result_polymorphic_handler_rewrites_state_inside_catch_boundary_before_outer_map() {
	let state = Rc::new(RefCell::new(0));
	let action: StateCatchRun<i32> =
		Run::<StateCatchFirstRow, CatchScopedRow, ()>::put::<i32, _>(7).bind(|()| {
			Run::<StateCatchFirstRow, CatchScopedRow, i32>::throw::<&'static str, _>("boom")
		});
	let program: StateCatchRun<String> = Run::catch::<&'static str, _>(action, |err| {
		assert_eq!(err, "boom");
		Run::<StateCatchFirstRow, CatchScopedRow, i32>::get::<_>()
	})
	.map(|value| format!("state={value}"));

	let narrowed: StateCatchMinusStateRun<String> = program
		.handle_with_handler::<BoxStateBrand<BoxBrand, i32>, _, StateCatchFirstRowMinusState>(
			StatePolymorphicHandler {
				state: Rc::clone(&state),
			},
		);
	let result = narrowed.handle(
		handlers! {
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, StateCatchMinusStateRun<String>>| {
				Run::pure("uncaught".to_string())
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, CNilBrand, _>(),
		},
	);

	assert_eq!(result, "state=7");
	assert_eq!(*state.borrow(), 7);
}

#[test]
fn catch_handler_interposes_except_without_losing_state_or_outer_map() {
	let state = Rc::new(RefCell::new(0));
	let state_for_handler = Rc::clone(&state);
	let action: StateCatchRun<i32> =
		Run::<StateCatchFirstRow, CatchScopedRow, ()>::put::<i32, _>(7).bind(|()| {
			Run::<StateCatchFirstRow, CatchScopedRow, i32>::throw::<&'static str, _>("boom")
		});
	let program: StateCatchRun<String> = Run::catch::<&'static str, _>(action, |err| {
		assert_eq!(err, "boom");
		Run::<StateCatchFirstRow, CatchScopedRow, i32>::get::<_>()
	})
	.map(|value| format!("state={value}"));

	let result = program.handle(
		handlers! {
			BoxStateBrand<BoxBrand, i32>: move |op: BoxState<'_, BoxBrand, i32, StateCatchRun<String>>| {
				match op {
					BoxState::Get(k) => k(*state_for_handler.borrow()),
					BoxState::Put(next, k) => {
						*state_for_handler.borrow_mut() = next;
						k(())
					}
				}
			},
			ExceptBrand<&'static str>: |_op: Except<'_, &'static str, StateCatchRun<String>>| {
				Run::pure("uncaught".to_string())
			},
		},
		scoped_handlers! {
			BoxCatchBrand<BoxBrand, &'static str>: catch_handler::<_, StateCatchFirstRowMinusExcept, _>(),
		},
	);

	assert_eq!(result, "state=7");
	assert_eq!(*state.borrow(), 7);
}

#[test]
fn local_handler_interposes_reader_without_losing_outer_map() {
	let action: ReaderLocalRun<i32> =
		Run::<ReaderLocalFirstRow, ReaderLocalScopedRow, i32>::ask::<_>();
	let program: ReaderLocalRun<i32> =
		Run::local::<i32, _>(|env| env + 1, action).map(|value| value * 2);

	let result = program.handle(
		handlers! {
			BoxReaderBrand<BoxBrand, i32>: |op: BoxReader<'_, BoxBrand, i32, ReaderLocalRun<i32>>| {
				match op {
					BoxReader::Ask(k) => k(10),
				}
			},
		},
		scoped_handlers! {
			BoxLocalBrand<BoxBrand, i32>: local_handler::<_, ReaderLocalFirstRowMinusReader, _>(),
		},
	);

	assert_eq!(result, 22);
}

#[test]
fn into_explicit_via_into_round_trips_pure() {
	use crate::types::effects::run_explicit::RunExplicit;
	let run: RunAlias<i32> = Run::pure(42);
	let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = run.into();
	assert!(matches!(explicit.peel(), Ok(42)));
}

#[test]
fn into_explicit_via_into_preserves_suspended_layer() {
	use crate::types::effects::run_explicit::RunExplicit;
	let coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));
	let layer = Coproduct::inject(coyo);
	let run: RunAlias<i32> = Run::send(Node::First(layer));
	let explicit: RunExplicit<'static, FirstRow, Scoped, i32> = run.into();
	assert!(explicit.peel().is_err());
}

#[test]
fn bind_chains_pure_values() {
	let run: RunAlias<i32> = Run::pure(2).bind(|x| Run::pure(x + 1)).bind(|x| Run::pure(x * 10));
	assert!(matches!(run.peel(), Ok(30)));
}

#[test]
fn scoped_continuation_resumes_raw_action_before_outer_continuation() {
	let carrier = ScopedContinuation::new(run_scoped_continuation(raw_i32(41)));

	let result: EmptyRun<i32> = carrier.resume_default(&HandlersNil);

	assert_eq!(result.extract(), 410);
}

#[test]
fn scoped_continuation_transforms_raw_action_before_outer_continuation() {
	let carrier = ScopedContinuation::new(run_scoped_continuation(raw_i32(40)));

	let result: EmptyRun<i32> =
		carrier.resume_default_with_post_action(&HandlersNil, increment_raw_i32_value);

	assert_eq!(result.extract(), 410);
}

#[test]
fn scoped_continuation_transforms_raw_action_program_before_outer_continuation() {
	let carrier = ScopedContinuation::new(run_scoped_continuation(boxed_raw_i32(40)));

	let result: EmptyRun<i32> =
		carrier.resume_default_with_action_transform(&HandlersNil, append_increment_to_raw_action);

	assert_eq!(result.extract(), 410);
}

#[test]
fn scoped_boundary_map_preserves_action_and_stores_outer_continuation() {
	let program = catch_boundary(7).map(|value| value + 1);

	assert_boundary_action_and_result(program, 7, 8, 1);
}

#[test]
fn scoped_boundary_bind_preserves_action_and_stores_outer_continuation() {
	let program = catch_boundary(7).bind(|value| Run::pure(format!("value={value}")));

	assert_boundary_action_and_result(program, 7, "value=7".to_owned(), 1);
}

#[test]
fn scoped_boundary_map_then_bind_keeps_continuations_outside_action() {
	let program =
		catch_boundary(7).map(|value| value + 1).bind(|value| Run::pure(format!("value={value}")));

	assert_boundary_action_and_result(program, 7, "value=8".to_owned(), 2);
}

#[test]
fn run_catch_map_keeps_continuation_outside_action() {
	let program = public_catch(7).map(|value| value + 1);

	assert_boundary_action_and_result(program, 7, 8, 1);
}

#[test]
fn run_catch_bind_keeps_continuation_outside_action() {
	let program = public_catch(7).bind(|value| Run::pure(format!("value={value}")));

	assert_boundary_action_and_result(program, 7, "value=7".to_owned(), 1);
}

#[test]
fn safe_boundary_downcasts_follow_outer_continuations() {
	let program: CatchRun<String> =
		Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(0))
			.map(|value| value + 1)
			.bind(|value| Run::pure(format!("value={value}")));

	let interpreted: EmptyRun<String> = program
		.handle_scoped_with::<BoxCatchBrand<BoxBrand, &'static str>, _, CNilBrand>(|catch| {
			match catch {
				BoxCatch::Catch {
					action,
					handler: _,
				} => action(()),
			}
		});

	assert_eq!(interpreted.extract(), "value=8");
}

#[test]
fn run_catch_recovery_handler_is_stored_in_boundary_representation() {
	let program: CatchRun<i32> =
		Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(40)).map(|value| value + 2);

	assert_boundary_handler_and_result(program, 40, 42, 1);
}

#[test]
fn run_catch_peel_action_view_runs_pending_continuation() {
	let program = public_catch(7).map(|value| value + 1);
	let action_result = match program.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			action,
			handler: _,
		}))) => Some(action(())),
		_ => None,
	};

	assert!(matches!(action_result.map(Run::peel), Some(Ok(8))));
}

#[test]
fn run_catch_peel_handler_view_runs_pending_continuation() {
	let program: CatchRun<i32> =
		Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(40)).map(|value| value + 2);
	let handler_result = match program.peel() {
		Err(Node::Scoped(Coproduct::Inl(BoxCatch::Catch {
			action: _,
			handler,
		}))) => Some(handler("oops")),
		_ => None,
	};

	assert!(matches!(handler_result.map(Run::peel), Some(Ok(42))));
}

#[test]
fn run_catch_handle_scoped_with_action_runs_pending_continuation() {
	let program = public_catch(7).map(|value| value + 1);
	let interpreted: EmptyRun<i32> = program
		.handle_scoped_with::<BoxCatchBrand<BoxBrand, &'static str>, _, CNilBrand>(|catch| {
			match catch {
				BoxCatch::Catch {
					action,
					handler: _,
				} => action(()),
			}
		});

	assert_eq!(interpreted.extract(), 8);
}

#[test]
fn run_catch_handle_scoped_with_handler_runs_pending_continuation() {
	let program: CatchRun<i32> =
		Run::catch::<&'static str, _>(Run::pure(7), |_err| Run::pure(40)).map(|value| value + 2);
	let interpreted: EmptyRun<i32> = program
		.handle_scoped_with::<BoxCatchBrand<BoxBrand, &'static str>, _, CNilBrand>(|catch| {
			match catch {
				BoxCatch::Catch {
					action: _,
					handler,
				} => handler("oops"),
			}
		});

	assert_eq!(interpreted.extract(), 42);
}

#[test]
fn map_transforms_pure_value() {
	let run: RunAlias<i32> = Run::pure(7).map(|x| x * 3);
	assert!(matches!(run.peel(), Ok(21)));
}
