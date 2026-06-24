//! Grounding check: can the by-value `CatList`, the `Rc`-backed `RcCatList`, and
//! the `Arc`-backed `ArcCatList` be unified under one `CatQueue` trait and a
//! per-store `Queue<C>` associated type, such that:
//!   - a struct generic over the store can hold the queue as a field and drop it
//!     without naming `CatQueue` (so a one-shot store whose element is a
//!     non-`Clone` `FnOnce` still works), and
//!   - one body generic over the multi-shot stores can `uncons`-step AND clone the
//!     queue per branch (the multi-shot re-run)?
//!
//! The wrinkle this settles: `RcCatList`/`ArcCatList` carry `A: Clone` on their
//! `snoc`/`append`/`uncons` (their internal `link` uses `Rc::make_mut`), whereas
//! `CatList`'s carry no bound. So `CatQueue` is impl'd for `CatList` at every `C`
//! but for `RcCatList`/`ArcCatList` only at `C: Clone`. A store's queue therefore
//! cannot be bounded `Queue<C>: CatQueue<C>` for ALL `C` (the refcounted lists
//! reject non-`Clone` `C`). The resolution validated here:
//!   - the store's `Queue<C>` associated type is bounded only by `Default` (all
//!     three lists impl `Default` unconditionally), which is all the
//!     construction-free accessors (`mem::take`) and the `Drop` need;
//!   - `Drop` relies on the queue's OWN stack-safe iterative `Drop` (each list has
//!     one), so the generic `Drop` names no `CatQueue` bound (it could not add one
//!     beyond the struct's bounds anyway);
//!   - `CatQueue` (with `uncons`/`snoc`/`append`) is required only at the
//!     multi-shot stepping's use site, where the element is always `Clone` (an
//!     `Rc`/`Arc` continuation), so the `C: Clone` impls apply.
//!
//! Asserts: (a) the three lists impl `CatQueue`; (b) a store-generic struct with a
//! `Queue<C>` field drops for a one-shot store whose `C` is non-`Clone`; (c) one
//! body generic over the multi-shot stores unconses and clones the queue, run for
//! both `Rc` and `Arc`; (d) the `Arc` store's queue is statically `Send + Sync`.

use {
	fp_library::types::{
		ArcCatList,
		CatList,
		RcCatList,
	},
	std::{
		marker::PhantomData,
		rc::Rc,
		sync::Arc,
	},
};

fn assert_send_sync<T: Send + Sync>() {}

// One catenable-queue interface over the three lists. `Default` (the empty queue)
// is the supertrait; `snoc`/`append`/`uncons` are the stepping operations. No
// `Clone` supertrait: the one-shot Box queue holds non-`Clone` `FnOnce` elements.
trait CatQueue<C>: Default + Sized {
	fn snoc(
		self,
		c: C,
	) -> Self;
	fn append(
		self,
		other: Self,
	) -> Self;
	fn uncons(self) -> Option<(C, Self)>;
}

// `CatList` impls `CatQueue` at every element type (its ops carry no bound).
impl<C> CatQueue<C> for CatList<C> {
	fn snoc(
		self,
		c: C,
	) -> Self {
		CatList::snoc(self, c)
	}

	fn append(
		self,
		other: Self,
	) -> Self {
		CatList::append(self, other)
	}

	fn uncons(self) -> Option<(C, Self)> {
		CatList::uncons(self)
	}
}

// `RcCatList`/`ArcCatList` impl `CatQueue` only at `C: Clone` (their ops need it).
impl<C: Clone> CatQueue<C> for RcCatList<C> {
	fn snoc(
		self,
		c: C,
	) -> Self {
		RcCatList::snoc(self, c)
	}

	fn append(
		self,
		other: Self,
	) -> Self {
		RcCatList::append(self, other)
	}

	fn uncons(self) -> Option<(C, Self)> {
		RcCatList::uncons(self)
	}
}

impl<C: Clone> CatQueue<C> for ArcCatList<C> {
	fn snoc(
		self,
		c: C,
	) -> Self {
		ArcCatList::snoc(self, c)
	}

	fn append(
		self,
		other: Self,
	) -> Self {
		ArcCatList::append(self, other)
	}

	fn uncons(self) -> Option<(C, Self)> {
		ArcCatList::uncons(self)
	}
}

// The store axis. `Queue<C>` is a GAT bounded ONLY by `Default`, so it holds for a
// one-shot store whose `C` is a non-`Clone` `FnOnce` as well as the multi-shot
// stores. This mirrors the real `ClosureStorage` (`Stored`/`Erased` GATs).
trait Store: 'static {
	type Queue<C>: Default;
}

struct BoxStore;
impl Store for BoxStore {
	type Queue<C> = CatList<C>;
}

struct RcStore;
impl Store for RcStore {
	type Queue<C> = RcCatList<C>;
}

struct ArcStore;
impl Store for ArcStore {
	type Queue<C> = ArcCatList<C>;
}

// The multi-shot stores: their queues are `CatQueue + Clone` at a `Clone` element.
trait MultiShotStore: Store {}
impl MultiShotStore for RcStore {}
impl MultiShotStore for ArcStore {}

// A store-generic spine carrying the queue as a field. Its `Drop` only needs the
// queue's `Default` (to `mem::take`) and the queue's OWN iterative `Drop`; it
// names no `CatQueue` bound, so it compiles even for a one-shot store whose `C` is
// non-`Clone` (the `Drop`-bound rule forbids adding bounds beyond the struct's).
struct Spine<S: Store, C> {
	queue: S::Queue<C>,
	_marker: PhantomData<C>,
}

impl<S: Store, C> Drop for Spine<S, C> {
	fn drop(&mut self) {
		// Replace with the empty queue and let the taken queue drop via its own
		// stack-safe iterative `Drop`. No manual `uncons`, so no `Clone` needed.
		let _drained = std::mem::take(&mut self.queue);
	}
}

// ONE body generic over the multi-shot stores: clone the queue (multi-shot
// re-run), then drain both copies independently by `uncons`. The use-site bound
// `S::Queue<C>: CatQueue<C> + Clone` with `C: Clone` is satisfied by the
// refcounted lists' `C: Clone` impls.
fn clone_then_drain_both<S: MultiShotStore, C: Clone>(queue: S::Queue<C>) -> (usize, usize)
where
	S::Queue<C>: CatQueue<C> + Clone, {
	// Exercise `append` generically (the real stepping appends pending
	// continuations onto a branch); appending the empty queue is the identity.
	let queue = queue.append(<S::Queue<C> as Default>::default());
	let copy = queue.clone();

	let mut original_len = 0;
	let mut cursor = queue;
	while let Some((_c, rest)) = cursor.uncons() {
		original_len += 1;
		cursor = rest;
	}

	let mut copy_len = 0;
	let mut cursor = copy;
	while let Some((_c, rest)) = cursor.uncons() {
		copy_len += 1;
		cursor = rest;
	}

	(original_len, copy_len)
}

// (a) + (b): a one-shot store whose element is a non-`Clone` `FnOnce` builds and
// drops through the store-generic spine. This is the case the GAT's `Default`-only
// bound and the `CatQueue`-free `Drop` exist to support.
#[test]
fn one_shot_store_spine_holds_and_drops_a_non_clone_fnonce_queue() {
	type FnOnceCont = Box<dyn FnOnce(i32) -> i32>;
	let queue: <BoxStore as Store>::Queue<FnOnceCont> =
		CatList::empty().snoc(Box::new(|x| x + 1) as FnOnceCont);
	let spine: Spine<BoxStore, FnOnceCont> = Spine {
		queue,
		_marker: PhantomData,
	};
	// Reaching the end of scope drops `spine` (and its non-`Clone` queue) cleanly.
	drop(spine);
}

// Build a queue generically through `CatQueue::snoc` (the real construction snocs
// continuations onto the queue), starting from the empty `Default` queue.
fn queue_from<S: Store, C>(items: Vec<C>) -> S::Queue<C>
where
	S::Queue<C>: CatQueue<C>, {
	let mut queue = <S::Queue<C> as Default>::default();
	for item in items {
		queue = queue.snoc(item);
	}
	queue
}

// (c): ONE generic body clones and drains the queue for BOTH multi-shot stores,
// with the queue itself built generically through `CatQueue::snoc`.
#[test]
fn one_body_clones_and_drains_for_rc_and_arc() {
	type RcCont = Rc<dyn Fn(i32) -> i32>;
	let rc_queue = queue_from::<RcStore, RcCont>(vec![
		Rc::new(|x| x + 1) as RcCont,
		Rc::new(|x| x * 2) as RcCont,
	]);
	assert_eq!(clone_then_drain_both::<RcStore, RcCont>(rc_queue), (2, 2));

	type ArcCont = Arc<dyn Fn(i32) -> i32 + Send + Sync>;
	let arc_queue = queue_from::<ArcStore, ArcCont>(vec![
		Arc::new(|x| x + 1) as ArcCont,
		Arc::new(|x| x * 2) as ArcCont,
		Arc::new(|x| x - 3) as ArcCont,
	]);
	assert_eq!(clone_then_drain_both::<ArcStore, ArcCont>(arc_queue), (3, 3));
}

// (d): the Arc store's queue is statically `Send + Sync`, so an Arc-store spine
// carrying it is `Send + Sync` by inference (the field that would block it, the
// continuation queue, does not).
#[test]
fn arc_store_queue_is_send_sync() {
	type ArcCont = Arc<dyn Fn(i32) -> i32 + Send + Sync>;
	assert_send_sync::<<ArcStore as Store>::Queue<ArcCont>>();
}
