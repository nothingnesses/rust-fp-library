//! Grounding check: does the general `CatList` share its spine through an
//! internal reference count, or does it hold its elements and sublists by value?
//!
//! The answer determines two properties that matter for a continuation queue
//! built on `CatList`:
//!   - whether `CatList<A>: Send + Sync` follows from `A: Send + Sync` (true iff
//!     there is no internal `Rc` in the representation), and
//!   - whether cloning a `CatList` is O(1) (a refcount bump, possible only with an
//!     internal `Rc`/`Arc`) or O(N) (a deep structural copy).
//!
//! `CatList<A>` wraps `CatListInner::Cons(A, VecDeque<CatList<A>>, usize)`: it
//! holds its head and the deque of sublists BY VALUE, with a derived `Clone`.
//! There is no internal `Rc`. Therefore a `CatList` of `Send + Sync` elements is
//! itself `Send + Sync`, and its whole-list `Clone` is O(N) (deep), not O(1). This
//! is the opposite trade-off from the refcounted `RcCatList`
//! (`Cons(A, Rc<VecDeque<..>>, usize)`, O(1) `Clone` but non-`Send`) and
//! `ArcCatList` (`Arc`-backed, O(1) `Clone` and `Send + Sync`).
//!
//! The load-bearing evidence is assertion (a): a `CatList` whose elements are an
//! `Arc<dyn Fn + Send + Sync>` (the thread-safe continuation kind) is statically
//! `Send + Sync`. This spike does not assert the O(N)-vs-O(1) `Clone` cost (a cost
//! is not a type-level property); it asserts only the `Send`-ness, which is what a
//! thread-safe queue built on the general `CatList` depends on.

use {
	fp_library::types::cat_list::CatList,
	std::{
		rc::Rc,
		sync::Arc,
	},
};

fn assert_send_sync<T: Send + Sync>() {}
fn assert_send<T: Send>() {}

// A thread-safe continuation, structurally: an owned `Arc` to a `Send + Sync`
// `Fn`. The `Arc<dyn .. + Send + Sync>` is itself `Send + Sync`.
type ArcCont = Arc<dyn Fn(i32) -> i32 + Send + Sync>;

// A non-thread-safe continuation, structurally: an owned `Rc` to a `Fn`. The
// `Rc<..>` is neither `Send` nor `Sync`.
type RcCont = Rc<dyn Fn(i32) -> i32>;

// (a) A `CatList` whose elements are thread-safe `Arc` continuations is itself
// `Send + Sync`. If `CatList` shared its spine through an internal `Rc`, this
// assertion would NOT compile.
#[test]
fn catlist_of_arc_continuations_is_send_sync() {
	assert_send_sync::<CatList<ArcCont>>();
	assert_send_sync::<CatList<Arc<dyn std::any::Any + Send + Sync>>>();
}

// (b) `CatList` is therefore `Send`-transparent in its element: `CatList<A>: Send`
// follows from `A: Send`, with no internal pointer blocking it. A plain
// `CatList<i32>` is `Send + Sync`, confirming the container adds no `!Send` field.
#[test]
fn catlist_send_is_transparent_in_its_element() {
	assert_send_sync::<CatList<i32>>();
	assert_send::<CatList<String>>();
}

// (c) Control: a `CatList` of `Rc` continuations carries the `Rc`'s
// non-`Send`-ness through, so the difference between a thread-safe and a
// non-thread-safe continuation queue is the ELEMENT pointer (`Rc` vs `Arc`), not
// any pointer internal to `CatList`. This is shown only by construction here (a
// negative `Send` bound cannot be written as a compiling assertion); the positive
// `Arc` case (a) is the load-bearing evidence.
#[test]
fn rc_continuation_queue_constructs() {
	let queue: CatList<RcCont> = CatList::empty().snoc(Rc::new(|x: i32| x + 1));
	let (head, _rest) = queue.uncons().unwrap();
	assert_eq!(head(41), 42);
}
