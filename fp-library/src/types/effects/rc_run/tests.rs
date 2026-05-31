use {
	super::*,
	crate::{
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
			NodeBrand,
			OptionBrand,
			RcBrand,
			RcCoyonedaBrand,
		},
		classes::RefCountedPointer,
		types::{
			Identity,
			RcFree,
			effects::{
				coproduct::Coproduct,
				handlers::HandlersNil,
				interpreter::ScopedContinuation,
				node::Node,
			},
		},
	},
	core::{
		cell::RefCell,
		marker::PhantomData,
	},
	std::rc::Rc as StdRc,
};

type CoyonedaFirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type CoyonedaScoped = CNilBrand;
type RcCoyonedaFirstRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type WiderRcCoyonedaFirstRow = CoproductBrand<RcCoyonedaBrand<OptionBrand>, RcCoyonedaFirstRow>;
// `peel` carries a per-projection `Clone` bound that the canonical
// `Coyoneda`-wrapped row does not satisfy (`Coyoneda` is `!Clone`);
// tests that exercise `peel` use an `Identity`-headed row instead.
// `RcFree`'s outer `Rc<Inner>` provides the layout indirection that
// makes the `Identity`-headed row well-formed for `RcRun`.
type IdentityFirstRow = CoproductBrand<IdentityBrand, CNilBrand>;
type IdentityScoped = CNilBrand;
type EmptyRcRun<A> = RcRun<CNilBrand, CNilBrand, A>;

fn rc_scoped_continuation<Action, Final, K>(
	action: EmptyRcRun<Action>,
	outer: K,
) -> RcRunScopedContinuation<CNilBrand, CNilBrand, Action, Final, K>
where
	Action: Clone + 'static,
	Final: 'static,
	K: Fn(Action) -> EmptyRcRun<Final> + 'static, {
	RcRunScopedContinuation {
		action,
		outer: <RcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

#[test]
fn from_rc_free_and_into_rc_free_round_trip() {
	let rc_free: RcFree<NodeBrand<CoyonedaFirstRow, CoyonedaScoped>, i32> = RcFree::pure(42);
	let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> = RcRun::from_rc_free(rc_free);
	let _back: RcFree<NodeBrand<CoyonedaFirstRow, CoyonedaScoped>, i32> = rc_run.into_rc_free();
}

#[test]
fn clone_bumps_refcount_in_constant_time() {
	let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
	let _branch = rc_run.clone();
}

#[test]
fn drop_a_pure_rc_run_does_not_panic() {
	let rc_run: RcRun<CoyonedaFirstRow, CoyonedaScoped, i32> = RcRun::from_rc_free(RcFree::pure(7));
	drop(rc_run);
}

#[test]
fn pure_then_peel_returns_value() {
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(42);
	assert!(matches!(rc_run.peel(), Ok(42)));
}

#[test]
fn send_produces_suspended_program() {
	let layer = Coproduct::inject(Identity(7));
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::send(Node::First(layer));
	assert!(rc_run.peel().is_err());
}

#[test]
fn expand_widens_first_order_row_and_preserves_continuation() {
	let run: RcRun<RcCoyonedaFirstRow, CNilBrand, i32> =
		RcRun::lift::<IdentityBrand, _>(Identity(40)).map(|value| value + 2);

	let widened: RcRun<WiderRcCoyonedaFirstRow, CNilBrand, i32> = run.expand();

	let continuation_value = match widened.into_rc_free().resume() {
		Err(Node::First(Coproduct::Inr(Coproduct::Inl(coyo)))) => {
			let Identity(next) = coyo.lower_ref();
			next.resume().ok()
		}
		_ => None,
	};
	assert_eq!(continuation_value, Some(42));
}

#[test]
fn into_explicit_via_into_round_trips_pure() {
	use crate::types::effects::rc_run_explicit::RcRunExplicit;
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(42);
	let explicit: RcRunExplicit<'static, IdentityFirstRow, IdentityScoped, i32> = rc_run.into();
	assert!(matches!(explicit.peel(), Ok(42)));
}

#[test]
fn into_explicit_via_into_preserves_suspended_layer() {
	use crate::types::effects::rc_run_explicit::RcRunExplicit;
	let layer = Coproduct::inject(Identity(7));
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::send(Node::First(layer));
	let explicit: RcRunExplicit<'static, IdentityFirstRow, IdentityScoped, i32> = rc_run.into();
	assert!(explicit.peel().is_err());
}

#[test]
fn bind_chains_pure_values() {
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> =
		RcRun::pure(2).bind(|x| RcRun::pure(x + 1)).bind(|x| RcRun::pure(x * 10));
	assert!(matches!(rc_run.peel(), Ok(30)));
}

#[test]
fn scoped_continuation_repeats_action_before_outer_continuation() {
	let events = StdRc::new(RefCell::new(Vec::new()));
	let outer_events = StdRc::clone(&events);
	let carrier =
		ScopedContinuation::new(rc_scoped_continuation(EmptyRcRun::pure(40), move |value| {
			outer_events.borrow_mut().push("outer");
			EmptyRcRun::pure(value * 10)
		}));

	let first: EmptyRcRun<i32> = carrier.clone().resume_rc(&HandlersNil);
	let second: EmptyRcRun<i32> = carrier.resume_rc(&HandlersNil);

	assert_eq!(first.extract(), 400);
	assert_eq!(second.extract(), 400);
	assert_eq!(&*events.borrow(), &["outer", "outer"]);
}

#[test]
fn scoped_continuation_transforms_action_before_outer_continuation() {
	let events = StdRc::new(RefCell::new(Vec::new()));
	let outer_events = StdRc::clone(&events);
	let carrier = ScopedContinuation::new(rc_scoped_continuation(
		EmptyRcRun::pure(40).bind(|value| EmptyRcRun::pure(value + 1)),
		move |value| {
			outer_events.borrow_mut().push("outer");
			EmptyRcRun::pure(value * 10)
		},
	));
	let post_events = StdRc::clone(&events);

	let result: EmptyRcRun<i32> = carrier.resume_rc_with_post_action(&HandlersNil, move |value| {
		post_events.borrow_mut().push("post");
		EmptyRcRun::pure(value + 1)
	});

	assert_eq!(result.extract(), 420);
	assert_eq!(&*events.borrow(), &["post", "outer"]);
}

#[test]
fn scoped_continuation_repeats_action_transform_before_outer_continuation() {
	let events = StdRc::new(RefCell::new(Vec::new()));
	let outer_events = StdRc::clone(&events);
	let carrier =
		ScopedContinuation::new(rc_scoped_continuation(EmptyRcRun::pure(40), move |value| {
			outer_events.borrow_mut().push("outer");
			EmptyRcRun::pure(value * 10)
		}));

	let first_events = StdRc::clone(&events);
	let first: EmptyRcRun<i32> =
		carrier.clone().resume_rc_with_action_transform(&HandlersNil, move |action| {
			let transform_events = StdRc::clone(&first_events);
			action.bind(move |value| {
				transform_events.borrow_mut().push("transform");
				EmptyRcRun::pure(value + 1)
			})
		});
	let second_events = StdRc::clone(&events);
	let second: EmptyRcRun<i32> =
		carrier.resume_rc_with_action_transform(&HandlersNil, move |action| {
			let transform_events = StdRc::clone(&second_events);
			action.bind(move |value| {
				transform_events.borrow_mut().push("transform");
				EmptyRcRun::pure(value + 2)
			})
		});

	assert_eq!(first.extract(), 410);
	assert_eq!(second.extract(), 420);
	assert_eq!(&*events.borrow(), &["transform", "outer", "transform", "outer"]);
}

#[test]
fn map_transforms_pure_value() {
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(7).map(|x| x * 3);
	assert!(matches!(rc_run.peel(), Ok(21)));
}

#[test]
fn ref_bind_chains_pure_value_via_clone() {
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(2);
	let chained = rc_run.ref_bind(|x: &i32| RcRun::pure(*x + 1));
	assert!(matches!(chained.peel(), Ok(3)));
}

#[test]
fn ref_map_transforms_pure_value_via_clone() {
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::pure(7);
	let mapped = rc_run.ref_map(|x: &i32| *x * 3);
	assert!(matches!(mapped.peel(), Ok(21)));
}

#[test]
fn ref_pure_wraps_cloned_value() {
	let value = 42;
	let rc_run: RcRun<IdentityFirstRow, IdentityScoped, i32> = RcRun::ref_pure(&value);
	assert!(matches!(rc_run.peel(), Ok(42)));
}
