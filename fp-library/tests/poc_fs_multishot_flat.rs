//! POC (item 4 step 5.2 / OQ-6G): does the unified `Free` need an outer `Rc`
//! (a per-`Store` self-storage pointer axis) to support multi-shot effects, or
//! can a FLAT struct (no outer pointer, Increment 1's shape) serve them?
//!
//! `RcFree` wraps its inner state in `Rc<RcFreeInner>`, giving O(1) whole-program
//! `Clone`, which multi-shot effects (`Choose`, `Amb`) use to re-run a suspended
//! program per branch. The unified `Free<F, A, Store>` is flat. This spike models
//! the Rc arm's shape (flat struct, `Rc<dyn Any>` erased values, `Rc<dyn Fn>`
//! re-callable continuations) and shows that multi-shot works WITHOUT an outer
//! pointer: the flat struct is `Clone` structurally (the leaves are already
//! `Rc`-shared), and a continuation queue is re-run over several branch values
//! by re-invoking the `Fn` continuations and sharing the queue (an `Rc`-pointer
//! clone per branch). The per-branch cost is the queue (`CatList`) clone, which
//! is step 5.3's node-sharing concern, not a structural blocker.
//!
//! Expected output: the flat `MiniFree` derives `Clone` with no outer `Rc`; a
//! `Choose` program re-runs its shared continuation queue over two branch values
//! and both branches compute correctly; the program is re-runnable after being
//! cloned (multi-shot), and the continuations are never consumed.

use std::{
	any::Any,
	rc::Rc,
};

// The Rc arm's erased value (multi-shot: shareable, clone to recover owned A).
type Erased = Rc<dyn Any>;

// The Rc arm's continuation: a re-callable `Fn`, shared via `Rc`.
type Cont = Rc<dyn Fn(Erased) -> MiniFree>;

// The flat mini spine: NO outer `Rc`. `Clone` is structural (clone the view plus
// the `Vec<Rc<..>>` queue, each element an `Rc`-pointer bump). This is the whole
// point: the flat Increment-1 struct shape is `Clone`-able for the Rc store
// without a self-storage pointer axis.
#[derive(Clone)]
struct MiniFree {
	view: View,
	conts: Vec<Cont>,
}

#[derive(Clone)]
enum View {
	Ret(Erased),
	// A multi-shot branch point carrying its branch values.
	Choose(Vec<Erased>),
}

// Recover a concrete value from a shared erased cell: clone out of the `Rc`
// (falls back to `Clone` when shared), the established Rc/Arc-arm contract.
fn dc<A: Clone + 'static>(e: &Erased) -> A {
	#[expect(clippy::expect_used, reason = "POC: erased type is maintained by construction")]
	let a = e.downcast_ref::<A>().expect("type maintained by construction");
	a.clone()
}

fn ret_i32(value: i32) -> MiniFree {
	MiniFree {
		view: View::Ret(Rc::new(value) as Erased),
		conts: Vec::new(),
	}
}

// Run one branch value through the SHARED continuation queue. The queue is
// borrowed (the `Fn` continuations are re-callable), so running it for one
// branch does not consume it; other branches reuse the same `Rc<dyn Fn>`s.
fn run_branch(
	start: Erased,
	conts: &[Cont],
) -> i32 {
	let mut current = start;
	for k in conts {
		match k(current).view {
			View::Ret(r) => current = r,
			View::Choose(_) => current = Rc::new(0i32) as Erased,
		}
	}
	dc::<i32>(&current)
}

fn run_multishot(program: &MiniFree) -> Vec<i32> {
	match &program.view {
		View::Choose(branches) =>
			branches.iter().map(|b| run_branch(Rc::clone(b), &program.conts)).collect(),
		View::Ret(r) => vec![dc::<i32>(r)],
	}
}

// (a) The flat struct (no outer Rc) supports multi-shot: one shared continuation
// queue re-run over two branch values, each continuation invoked once per branch.
#[test]
fn flat_rc_free_runs_multishot_over_shared_queue() {
	let conts: Vec<Cont> = vec![
		Rc::new(|e: Erased| ret_i32(dc::<i32>(&e) + 1)),
		Rc::new(|e: Erased| ret_i32(dc::<i32>(&e) * 2)),
	];
	let program = MiniFree {
		view: View::Choose(vec![Rc::new(10i32) as Erased, Rc::new(20i32) as Erased]),
		conts,
	};

	// (10+1)*2 = 22 ; (20+1)*2 = 42
	assert_eq!(run_multishot(&program), vec![22, 42]);
}

// (b) The program is `Clone` (structural, no outer Rc) and re-runnable after the
// clone: multi-shot does not consume the program or its continuations.
#[test]
fn flat_rc_free_is_clone_and_rerunnable() {
	let conts: Vec<Cont> = vec![Rc::new(|e: Erased| ret_i32(dc::<i32>(&e) + 100))];
	let program = MiniFree {
		view: View::Choose(vec![Rc::new(1i32) as Erased, Rc::new(2i32) as Erased]),
		conts,
	};

	let cloned = program.clone();
	assert_eq!(run_multishot(&program), vec![101, 102]);
	// The clone runs independently and identically (the program survived).
	assert_eq!(run_multishot(&cloned), vec![101, 102]);
}
