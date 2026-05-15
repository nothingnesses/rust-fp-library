use {
	super::*,
	crate::{
		brands::{
			ArcBrand,
			CNilBrand,
			CoproductBrand,
			IdentityBrand,
			NodeBrand,
		},
		classes::RefCountedPointer,
		types::{
			ArcFree,
			effects::{
				handlers::HandlersNil,
				interpreter::ScopedContinuation,
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
type ArcRunAlias<A> = ArcRun<FirstRow, Scoped, A>;
type EmptyArcRun<A> = ArcRun<CNilBrand, CNilBrand, A>;

fn arc_scoped_continuation<Action, Final, K>(
	action: EmptyArcRun<Action>,
	outer: K,
) -> ArcRunScopedContinuation<CNilBrand, CNilBrand, Action, Final, K>
where
	Action: Clone + Send + Sync + 'static,
	Final: Send + Sync + 'static,
	K: Fn(Action) -> EmptyArcRun<Final> + Send + Sync + 'static, {
	ArcRunScopedContinuation {
		action,
		outer: <ArcBrand as RefCountedPointer>::new(outer),
		result: PhantomData,
	}
}

#[test]
fn from_arc_free_and_into_arc_free_round_trip() {
	let arc_free: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = ArcFree::pure(42);
	let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(arc_free);
	let _back: ArcFree<NodeBrand<FirstRow, Scoped>, i32> = arc_run.into_arc_free();
}

#[test]
fn clone_bumps_atomic_refcount_in_constant_time() {
	let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(ArcFree::pure(7));
	let _branch = arc_run.clone();
}

#[test]
fn drop_a_pure_arc_run_does_not_panic() {
	let arc_run: ArcRunAlias<i32> = ArcRun::from_arc_free(ArcFree::pure(7));
	drop(arc_run);
}

fn _send_sync_witness<T: Send + Sync>() {}

#[test]
fn arc_run_is_send_sync() {
	_send_sync_witness::<ArcRunAlias<i32>>();
}

#[test]
fn scoped_continuation_carrier_is_send_sync() {
	_send_sync_witness::<
		ArcRunScopedContinuation<CNilBrand, CNilBrand, i32, i32, fn(i32) -> EmptyArcRun<i32>>,
	>();
}

#[test]
fn scoped_continuation_repeats_action_before_outer_continuation() {
	let outer_calls = StdArc::new(AtomicUsize::new(0));
	let observed_calls = StdArc::clone(&outer_calls);
	let carrier =
		ScopedContinuation::new(arc_scoped_continuation(EmptyArcRun::pure(40), move |value| {
			observed_calls.fetch_add(1, Ordering::SeqCst);
			EmptyArcRun::pure(value * 10)
		}));

	let first: EmptyArcRun<i32> = carrier.clone().resume_arc(&HandlersNil);
	let second: EmptyArcRun<i32> = carrier.resume_arc(&HandlersNil);

	assert_eq!(first.extract(), 400);
	assert_eq!(second.extract(), 400);
	assert_eq!(outer_calls.load(Ordering::SeqCst), 2);
}

#[test]
fn scoped_continuation_transforms_action_before_outer_continuation() {
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let carrier = ScopedContinuation::new(arc_scoped_continuation(
		EmptyArcRun::pure(40).bind(|value| EmptyArcRun::pure(value + 1)),
		move |value| {
			assert_eq!(outer_order.fetch_add(1, Ordering::SeqCst), 1);
			EmptyArcRun::pure(value * 10)
		},
	));
	let post_order = StdArc::clone(&order);

	let result: EmptyArcRun<i32> =
		carrier.resume_arc_with_post_action(&HandlersNil, move |value| {
			assert_eq!(post_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRun::pure(value + 1)
		});

	assert_eq!(result.extract(), 420);
	assert_eq!(order.load(Ordering::SeqCst), 2);
}

#[test]
fn scoped_continuation_repeats_action_transform_before_outer_continuation() {
	let order = StdArc::new(AtomicUsize::new(0));
	let outer_order = StdArc::clone(&order);
	let carrier =
		ScopedContinuation::new(arc_scoped_continuation(EmptyArcRun::pure(40), move |value| {
			let step = outer_order.fetch_add(1, Ordering::SeqCst);
			assert!(step == 1 || step == 3);
			EmptyArcRun::pure(value * 10)
		}));

	let first_order = StdArc::clone(&order);
	let first: EmptyArcRun<i32> =
		carrier.clone().resume_arc_with_action_transform(&HandlersNil, move |action| {
			let transform_order = StdArc::clone(&first_order);
			action.bind(move |value| {
				assert_eq!(transform_order.fetch_add(1, Ordering::SeqCst), 0);
				EmptyArcRun::pure(value + 1)
			})
		});
	let second_order = StdArc::clone(&order);
	let second: EmptyArcRun<i32> =
		carrier.resume_arc_with_action_transform(&HandlersNil, move |action| {
			let transform_order = StdArc::clone(&second_order);
			action.bind(move |value| {
				assert_eq!(transform_order.fetch_add(1, Ordering::SeqCst), 2);
				EmptyArcRun::pure(value + 2)
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
	let carrier =
		ScopedContinuation::new(arc_scoped_continuation(EmptyArcRun::pure(10), move |value| {
			let step = outer_order.fetch_add(1, Ordering::SeqCst);
			assert!(step == 1 || step == 3);
			EmptyArcRun::pure(value * 2)
		}));

	let first_order = StdArc::clone(&order);
	let first: EmptyArcRun<i32> =
		carrier.clone().resume_arc_with_post_action(&HandlersNil, move |value| {
			assert_eq!(first_order.fetch_add(1, Ordering::SeqCst), 0);
			EmptyArcRun::pure(value + 1)
		});

	let second_order = StdArc::clone(&order);
	let second: EmptyArcRun<i32> =
		carrier.resume_arc_with_post_action(&HandlersNil, move |value| {
			assert_eq!(second_order.fetch_add(1, Ordering::SeqCst), 2);
			EmptyArcRun::pure(value + 2)
		});

	assert_eq!(first.extract(), 22);
	assert_eq!(second.extract(), 24);
	assert_eq!(order.load(Ordering::SeqCst), 4);
}

#[test]
fn pure_then_peel_returns_value() {
	let arc_run: ArcRunAlias<i32> = ArcRun::pure(42);
	assert!(matches!(arc_run.peel(), Ok(42)));
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
	let arc_run: ArcRunAlias<i32> = ArcRun::send(Node::First(layer));
	assert!(arc_run.peel().is_err());
}

#[test]
fn into_explicit_via_into_round_trips_pure() {
	use crate::types::effects::arc_run_explicit::ArcRunExplicit;
	let arc_run: ArcRunAlias<i32> = ArcRun::pure(42);
	let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = arc_run.into();
	assert!(matches!(explicit.peel(), Ok(42)));
}

#[test]
fn into_explicit_via_into_preserves_suspended_layer() {
	use crate::types::{
		Identity,
		effects::{
			arc_run_explicit::ArcRunExplicit,
			coproduct::Coproduct,
			node::Node,
		},
	};
	let layer = Coproduct::inject(Identity(7));
	let arc_run: ArcRunAlias<i32> = ArcRun::send(Node::First(layer));
	let explicit: ArcRunExplicit<'static, FirstRow, Scoped, i32> = arc_run.into();
	assert!(explicit.peel().is_err());
}

#[test]
fn bind_chains_pure_values() {
	let arc_run: ArcRunAlias<i32> =
		ArcRun::pure(2).bind(|x| ArcRun::pure(x + 1)).bind(|x| ArcRun::pure(x * 10));
	assert!(matches!(arc_run.peel(), Ok(30)));
}

#[test]
fn map_transforms_pure_value() {
	let arc_run: ArcRunAlias<i32> = ArcRun::pure(7).map(|x| x * 3);
	assert!(matches!(arc_run.peel(), Ok(21)));
}

#[test]
fn ref_bind_chains_pure_value_via_clone() {
	let arc_run: ArcRunAlias<i32> = ArcRun::pure(2);
	let chained = arc_run.ref_bind(|x: &i32| ArcRun::pure(*x + 1));
	assert!(matches!(chained.peel(), Ok(3)));
}

#[test]
fn ref_map_transforms_pure_value_via_clone() {
	let arc_run: ArcRunAlias<i32> = ArcRun::pure(7);
	let mapped = arc_run.ref_map(|x: &i32| *x * 3);
	assert!(matches!(mapped.peel(), Ok(21)));
}

#[test]
fn ref_pure_wraps_cloned_value() {
	let value = 42;
	let arc_run: ArcRunAlias<i32> = ArcRun::ref_pure(&value);
	assert!(matches!(arc_run.peel(), Ok(42)));
}
