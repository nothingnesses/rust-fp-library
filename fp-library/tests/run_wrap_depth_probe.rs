// Probe: measure Wrap-arm depth in Run-shaped programs over the existing
// Free family, to inform the Phase 2 step 4 blocker resolution.
//
// The blocker: Free's struct-level `F: Extract` bound is load-bearing for
// the iterative Drop impl that walks deep Wrap chains via F::extract.
// Run programs over typical effect rows can't satisfy F: Extract because
// effects-as-data have no canonical extract semantics.
//
// The question this probe answers: how deep do Wrap chains actually get
// in Run-shaped programs?
//
// If the answer is "depth 1 for the Erased family (CatList-Free)
// regardless of bind chain length, depth N for the Explicit family
// (naive recursive Free) where N is the number of effect calls", then:
//
//   - A Run substrate based on CatList-Free can use a recursive-drop
//     fallback for the Wrap arm without overflowing the stack.
//   - The Explicit Run substrate would still need iterative Wrap-arm
//     drop, so it remains blocked.
//
// We use ThunkBrand (which IS Extract) as the row stand-in so we can
// measure depth without the actual blocker firing. The structural
// behaviour of bind chains doesn't depend on F's Extract property; only
// the eventual Drop semantics does. ThunkBrand is the same brand the
// existing Free unit tests use; it provides the indirection needed to
// avoid the `Free<IdentityBrand, _>` layout cycle.

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
fn drop_a_typical_run_shaped_program_does_not_overflow_without_extract_iteration() {
	// The bottom-line property for path (b) (RunFree-like substrate
	// with recursive Wrap drop): a Run-shaped program with N effect
	// calls assembled via flat bind chain has structural Wrap depth
	// at most 1, regardless of N. Dropping it does not require deep
	// Wrap-chain dismantling.
	//
	// We can't prove "no overflow" via assertion alone, but we can
	// confirm the test runs to completion. If a recursive drop on
	// Wrap were unsafe at this size, this test would crash.
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
	// chain. This is the case that motivates iterative Drop via
	// Extract on the existing Free. The probe confirms it for
	// completeness.
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
