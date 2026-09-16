//! Queue laws for the persistent Rc and Arc continuation queues.
//!
//! Both implementations must preserve FIFO order, append identity and
//! associativity, independent traversal after O(1) cloning, and stack-safe
//! deep draining. These laws exercise shared nodes deliberately: every law
//! clones queue versions before consuming them, so `uncons` must return a
//! persistent remainder rather than copy a shared pending-child container.

use fp_library::types::{
	ArcCatList,
	RcCatList,
	cat_queue::CatQueue,
};

fn queue_from<Q>(values: impl IntoIterator<Item = usize>) -> Q
where
	Q: CatQueue<usize>, {
	values.into_iter().fold(Q::default(), CatQueue::snoc)
}

fn drain<Q>(mut queue: Q) -> Vec<usize>
where
	Q: CatQueue<usize>, {
	let mut values = Vec::new();
	while let Some((value, rest)) = CatQueue::uncons(queue) {
		values.push(value);
		queue = rest;
	}
	values
}

fn check_queue_laws<Q>()
where
	Q: CatQueue<usize> + Clone, {
	let empty = Q::default();
	let values = queue_from::<Q>(0 .. 8);
	assert_eq!(drain(CatQueue::append(empty, values.clone())), (0 .. 8).collect::<Vec<_>>());
	assert_eq!(drain(CatQueue::append(values.clone(), Q::default())), (0 .. 8).collect::<Vec<_>>());

	let first = queue_from::<Q>(0 .. 3);
	let second = queue_from::<Q>(3 .. 6);
	let third = queue_from::<Q>(6 .. 9);
	let left = CatQueue::append(CatQueue::append(first.clone(), second.clone()), third.clone());
	let right = CatQueue::append(first, CatQueue::append(second, third));
	assert_eq!(drain(left), drain(right));

	let shared = queue_from::<Q>(0 .. 1_000);
	let clone = shared.clone();
	assert_eq!(drain(shared), (0 .. 1_000).collect::<Vec<_>>());
	assert_eq!(drain(clone), (0 .. 1_000).collect::<Vec<_>>());
}

#[test]
fn rc_queue_laws_hold_for_shared_versions() {
	check_queue_laws::<RcCatList<usize>>();
}

#[test]
fn arc_queue_laws_hold_for_shared_versions() {
	check_queue_laws::<ArcCatList<usize>>();
}

#[test]
fn deep_shared_queues_drain_independently_without_native_stack_growth() {
	const DEPTH: usize = 100_000;

	let rc = queue_from::<RcCatList<usize>>(0 .. DEPTH);
	let rc_clone = rc.clone();
	assert_eq!(drain(rc).len(), DEPTH);
	assert_eq!(drain(rc_clone).len(), DEPTH);

	let arc = queue_from::<ArcCatList<usize>>(0 .. DEPTH);
	let arc_clone = arc.clone();
	assert_eq!(drain(arc).len(), DEPTH);
	assert_eq!(drain(arc_clone).len(), DEPTH);
}
