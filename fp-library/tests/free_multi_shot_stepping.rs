//! Multi-shot stepping over a branching functor.
//!
//! `Free::to_view` on the `Rc`/`Arc` stores clones the continuation queue
//! into each branch of a suspended layer, so one `bind` continuation is
//! re-entered once per branch, which the single-shot `Box` store rules out
//! by type (its queue holds `FnOnce` continuations and is moved exactly
//! once). These tests pin that behaviour with a two-branch `Fork` functor
//! and one store-generic driver: a single body over `MultiShotStore`
//! collects every leaf, serving `Rc` and `Arc` alike, and needs only
//! `to_view` (construction happens per store, outside the generic body).

use fp_library::{
	Apply,
	brands::{
		ArcBrand,
		RcBrand,
	},
	classes::{
		Functor,
		WrapDrop,
	},
	impl_kind,
	kinds::*,
	types::{
		Free,
		FreeStep,
		cat_queue::CatQueue,
		closure_storage::{
			ClosureStorage,
			MultiShotStore,
			ValueFor,
		},
		free::Continuation,
	},
};

/// A two-branch functor: the smallest shape whose `map` must run the
/// mapping closure once per branch. The children sit behind `Box` so the
/// recursive `Free<ForkBrand, _>` spine has heap indirection (a by-value
/// child would make the layout infinitely sized).
enum ForkF<A> {
	Fork(Box<A>, Box<A>),
}

struct ForkBrand;

impl_kind! {
	impl for ForkBrand {
		type Of<'a, A: 'a>: 'a = ForkF<A>;
	}
}

impl Functor for ForkBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ForkF::Fork(left, right) => ForkF::Fork(Box::new(f(*left)), Box::new(f(*right))),
		}
	}
}

impl WrapDrop for ForkBrand {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		// Both branches drop in place; safe for the shallow trees these
		// tests build.
		let _ = fa;
		None
	}
}

/// One generic driver over both multi-shot stores: steps the program and
/// collects every leaf, left to right. Only `to_view` is needed inside the
/// generic body; the per-store `bind`/`lift_f` calls happen at the
/// construction sites, so no bind seam is required for interpretation.
fn collect_leaves<S>(program: Free<ForkBrand, i32, S>) -> Vec<i32>
where
	S: MultiShotStore,
	i32: ValueFor<S>,
	<S as ClosureStorage>::Queue<Continuation<ForkBrand, S>>:
		CatQueue<Continuation<ForkBrand, S>> + Clone, {
	match program.to_view() {
		FreeStep::Done(leaf) => vec![leaf],
		FreeStep::Suspended(ForkF::Fork(left, right)) => {
			let mut leaves = collect_leaves(*left);
			leaves.extend(collect_leaves(*right));
			leaves
		}
	}
}

// One suspension, one bind continuation: the continuation runs once per
// branch, so both leaves observe it.
#[test]
fn rc_stepping_resumes_the_continuation_once_per_branch() {
	let program: Free<ForkBrand, i32, RcBrand> =
		Free::<ForkBrand, i32, RcBrand>::lift_f(ForkF::Fork(Box::new(1), Box::new(2)))
			.bind_multi_shot(|x: i32| Free::pure(x * 10));
	assert_eq!(collect_leaves(program), vec![10, 20]);
}

// A two-level tree: the first continuation suspends again per branch and
// the final continuation is re-entered once per leaf (four times), with
// the queue cloned at each fork.
#[test]
fn rc_stepping_re_enters_shared_continuations_across_nested_branches() {
	let program: Free<ForkBrand, i32, RcBrand> =
		Free::<ForkBrand, i32, RcBrand>::lift_f(ForkF::Fork(Box::new(1), Box::new(2)))
			.bind_multi_shot(|x: i32| Free::lift_f(ForkF::Fork(Box::new(x), Box::new(x + 1))))
			.bind_multi_shot(|x: i32| Free::pure(x * 10));
	assert_eq!(collect_leaves(program), vec![10, 20, 20, 30]);
}

// The same generic driver body serves the Arc store unchanged; only the
// construction site names the store (its `bind_multi_shot` requires a
// `Send + Sync` continuation).
#[test]
fn arc_stepping_drives_the_same_generic_body() {
	let program: Free<ForkBrand, i32, ArcBrand> =
		Free::<ForkBrand, i32, ArcBrand>::lift_f(ForkF::Fork(Box::new(3), Box::new(4)))
			.bind_multi_shot(|x: i32| Free::pure(x + 1));
	assert_eq!(collect_leaves(program), vec![4, 5]);
}
