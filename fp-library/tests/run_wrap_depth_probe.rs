// Regression guard: structural `Wrap`-arm depth in Free programs that
// use the same shape as Run-style effect computations.
//
// `Free`'s `Drop` calls `<F as WrapDrop>::drop(fa)` on each `Suspend`
// layer. Brands that materially store the inner `Free` return
// `Some(inner)` to keep the iterative dismantling path engaged; brands
// that don't (effect-row brands, Coyoneda) return `None` and let the
// layer drop in place. The `None` policy is sound only if the
// structural `Wrap` depth stays bounded: a deep recursive drop over a
// long `Wrap` chain would still overflow the stack.
//
// This file measures, for the four Run-typical patterns, how deep the
// structural `Wrap` chain actually gets in the original view (before
// `to_view` applies any continuations):
//
//   1. `pure(0)` plus a long flat bind chain has structural depth 0.
//   2. A single `lift_f(eff)` plus a long flat bind chain has
//      structural depth 1 regardless of chain length (continuations
//      live in the CatList, not in the view).
//   3. `pure(0).bind(|x| lift_f(eff))` chained N times has structural
//      depth 0; the inner `lift_f`s materialise as Wrap layers only
//      at evaluation time (when `to_view` applies the closures).
//   4. Hand-built `Free::wrap(Free::wrap(...))` chains do grow the
//      structural depth linearly. Run-typical programs do not produce
//      this pattern; if a future change starts emitting it for
//      Run-shaped programs, the `WrapDrop::drop = None` policy on
//      effect-row brands becomes unsound and this file's tests will
//      need to evolve.
//
// `ThunkBrand` is the row stand-in because `Free<IdentityBrand, _>`
// is layout-cyclic (Identity has no indirection, so the recursive
// Wrap arm has no termination at the type level). The structural
// behaviour the probe measures is brand-independent.

use fp_library::{
	brands::ThunkBrand,
	classes::Extract,
	types::{
		Free,
		FreeStep,
		Thunk,
	},
};

/// Walks a `Free<ThunkBrand, _>` value via `to_view` + `Extract`,
/// counting how many `Suspended` (Wrap) layers materialize before
/// reaching `Done`.
///
/// IMPORTANT: this measures **evaluation depth**, not the original
/// program's structural Wrap depth. `to_view` applies pending
/// continuations, which can produce new Wrap layers from `bind`
/// closures that return `lift_f(...)`. The structural depth is what
/// `Drop` traverses; evaluation depth is what an interpreter sees.
fn evaluation_depth(free: Free<ThunkBrand, i32>) -> usize {
	let mut current = free;
	let mut depth = 0;
	loop {
		match current.to_view() {
			FreeStep::Done(_) => return depth,
			FreeStep::Suspended(layer) => {
				depth += 1;
				let extracted: Free<ThunkBrand, i32> = <ThunkBrand as Extract>::extract(layer);
				current = extracted;
			}
		}
	}
}

#[test]
fn pure_only_has_wrap_depth_zero() {
	let program: Free<ThunkBrand, i32> = Free::pure(42);
	assert_eq!(evaluation_depth(program), 0);
}

#[test]
fn pure_then_bind_chain_stays_at_depth_zero() {
	// CatList-Free's bind appends to the continuation queue; it does
	// NOT add to the view's Wrap chain. A deep bind chain over `pure`
	// has Wrap depth 0.
	let mut program: Free<ThunkBrand, i32> = Free::pure(0);
	for _ in 0 .. 1000 {
		program = program.bind(|x| Free::pure(x + 1));
	}
	assert_eq!(
		evaluation_depth(program),
		0,
		"flat bind chain over pure should not grow Wrap depth"
	);
}

#[test]
fn lift_f_alone_has_wrap_depth_one() {
	// One lift_f produces one Wrap layer.
	let program: Free<ThunkBrand, i32> = Free::lift_f(Thunk::new(|| 42));
	assert_eq!(evaluation_depth(program), 1);
}

#[test]
fn lift_f_then_flat_bind_chain_stays_at_depth_one() {
	// The key property for the Erased Run family: bind appends to
	// continuations, not to Wrap. Even N=1000 binds after a single
	// lift_f keep Wrap depth at 1.
	let program: Free<ThunkBrand, i32> = Free::lift_f(Thunk::new(|| 0));
	let mut program = program;
	for _ in 0 .. 1000 {
		program = program.bind(|x| Free::pure(x + 1));
	}
	assert_eq!(
		evaluation_depth(program),
		1,
		"lift_f then 1000 binds should keep Wrap depth at 1 \
		 (continuations grow in the CatList, not in the view's Wrap chain)"
	);
}

#[test]
fn nested_lift_f_via_bind_materializes_wraps_at_evaluation_time() {
	// Building a chain via binds whose closures call lift_f: each
	// closure RETURNS a Wrap when applied to its argument, so the
	// `to_view` driver sees a Wrap layer per closure invocation.
	// Evaluation depth equals the number of effect-returning binds.
	//
	// However, the program's STRUCTURAL Wrap depth (which is what
	// Drop traverses) is 0: the original view is `Pure(0)` and the
	// bind closures live inside the CatList of continuations. The
	// 100 Wraps below are hypothetical — they only materialize when
	// `to_view` applies the continuations, which Drop never does.
	let mut program: Free<ThunkBrand, i32> = Free::pure(0);
	for _ in 0 .. 100 {
		program = program.bind(|x| Free::lift_f(Thunk::new(move || x + 1)));
	}
	assert_eq!(
		evaluation_depth(program),
		100,
		"each bind whose closure returns lift_f materializes one Wrap \
		 per evaluation step; the original view is still Pure (depth 0 \
		 for Drop)."
	);
}

#[test]
fn drop_a_typical_run_shaped_program_does_not_overflow() {
	// Bottom-line soundness check: a program with N effect calls
	// assembled via a flat bind chain has structural Wrap depth at
	// most 1 regardless of N, so dropping it without iterating over
	// the Wrap arm cannot overflow.
	//
	// We can't prove "no overflow" via assertion alone, but we can
	// confirm the test runs to completion. If a recursive drop on
	// the Wrap arm were unsafe at this size, this test would crash.
	let mut program: Free<ThunkBrand, i32> = Free::lift_f(Thunk::new(|| 0));
	for _ in 0 .. 100_000 {
		program = program.bind(|x| Free::pure(x + 1));
	}
	// Construct, then immediately drop. No evaluation. Drop has to
	// walk 100,000 continuations (iteratively) and one Wrap layer.
	drop(program);
}

#[test]
fn explicit_wrap_chain_grows_linearly() {
	// Repeated explicit calls to Free::wrap DO grow the view's Wrap
	// chain. This is the artificial pattern that requires
	// `WrapDrop::drop` to return `Some(inner)` (not `None`) for
	// soundness. Run-typical usage does not produce this shape; this
	// test exists to make the contrast with the other patterns
	// explicit.
	const DEPTH: usize = 100;
	let mut program: Free<ThunkBrand, i32> = Free::pure(0);
	for _ in 0 .. DEPTH {
		program = Free::wrap(Thunk::new(move || program));
	}
	assert_eq!(
		evaluation_depth(program),
		DEPTH,
		"each explicit Free::wrap adds one to the view's Wrap depth"
	);
}
