// Integration tests for the production `FreeExplicit<'a, F, A>` covering
// the questions originally exercised by the standalone POC in this file.
//
// The POC validated that a naive recursive Free monad over a concrete
// functor structure could:
// 1. Compile as a `Kind` with the existing brand macros.
// 2. Support non-`'static` effect payloads (e.g., `&'a str`).
// 3. Interpret to completion iteratively for a concrete functor.
// 4. Survive deep `Wrap` chains under both evaluation and `Drop`.
//
// In production, the type lives at `fp_library::types::FreeExplicit` and
// is bounded `F: Extract + Functor + 'a` so that its custom iterative
// `Drop` can dismantle deep chains without overflowing the stack. The
// POC's `OptionBrand`-based short-circuit tests are gone: `OptionBrand`
// cannot lawfully implement `Extract` (a `None` has no value to surrender),
// and the same Run-shaped semantics are reachable in production via
// handler interpretation rather than direct `FreeExplicit::evaluate`.

use fp_library::{
	Apply,
	brands::{
		FreeExplicitBrand,
		IdentityBrand,
	},
	kinds::*,
	types::{
		FreeExplicit,
		Identity,
	},
};
// `Kind` is referenced only inside `Kind!(...)` macro invocations below.
// rustc's unused-import analysis does not see the macro call as a direct
// use, so the import looks dead even though removing it breaks compilation.
#[expect(
	unused_imports,
	reason = "Kind is referenced via the Kind!(...) macro below, which rustc does not detect as a direct use."
)]
use fp_macros::Kind;

// -- Q1: does the type integrate with the Kind system? --

#[test]
fn q1_kind_integration() {
	// This function is only well-typed if `FreeExplicitBrand<IdentityBrand>`
	// is a valid `Kind_cdc7cd43dac7585f` implementer.
	fn accepts_kind<F>()
	where
		F: Kind_cdc7cd43dac7585f + 'static, {
	}

	accepts_kind::<FreeExplicitBrand<IdentityBrand>>();

	// Also verify the associated-type application produces the concrete
	// FreeExplicit we expect.
	let _typed: Apply!(
		<FreeExplicitBrand<IdentityBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, i32>
	) = FreeExplicit::<IdentityBrand, i32>::pure(42);
}

// -- Q2: does it carry a borrowed payload? --

#[test]
fn q2_borrowed_payload() {
	// `owner` is the lifetime source. `reference` is borrowed from it.
	// Putting `reference` inside FreeExplicit must compile, which is only
	// possible if `FreeExplicit` does not require its `A` to be `'static`.
	let owner = String::from("borrow me");
	let reference: &str = owner.as_str();

	let free: FreeExplicit<'_, IdentityBrand, &str> = FreeExplicit::pure(reference);
	let mapped: FreeExplicit<'_, IdentityBrand, usize> = free.bind(|r: &str| {
		// Closure also borrows from a non-'static scope (the outer test fn).
		FreeExplicit::pure(r.len())
	});

	assert_eq!(mapped.evaluate(), owner.len());
}

#[test]
fn q2_borrowed_in_wrap_layer() {
	// A more demanding variant: the borrowed data lives inside the functor
	// layer, not the payload. Verifies that F::Of<'a, _> genuinely carries
	// the `'a` all the way through.
	let owner = String::from("inner");
	let borrowed: &str = owner.as_str();

	let free: FreeExplicit<'_, IdentityBrand, &str> =
		FreeExplicit::wrap(Identity(Box::new(FreeExplicit::pure(borrowed))));

	assert_eq!(free.evaluate(), "inner");
}

// -- Q4a: iterative evaluate on a very deep chain does not overflow --

#[test]
fn q4_iterative_evaluate_deep() {
	// Build a deep Wrap chain by repeated bind. Each bind inserts one layer.
	// Evaluate iteratively; the loop's constant stack depth should handle
	// any size the heap allows.
	const DEPTH: usize = 100_000;
	let mut free: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(0);
	for _ in 0 .. DEPTH {
		free = FreeExplicit::wrap(Identity(Box::new(free)));
	}
	assert_eq!(free.evaluate(), 0);
}

// -- Q4b: iterative `Drop` on a very deep chain does not overflow --
//
// In the POC this test was `#[ignore]`d because the naive recursive `Drop`
// stack-overflowed. The production type ships a custom iterative `Drop`
// that walks the chain via `Extract::extract`, so the test is now active.

#[test]
fn q4_drop_deep_does_not_overflow() {
	const DEPTH: usize = 100_000;
	let mut free: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(0);
	for _ in 0 .. DEPTH {
		free = FreeExplicit::wrap(Identity(Box::new(free)));
	}
	// Deliberately forget to evaluate. Drop runs at end of scope.
	drop(free);
}

// -- Q5: chained binds compose --

#[test]
fn q5_identity_chained_binds() {
	// A shallow sanity check that bind composes over IdentityBrand. This is
	// the baseline Run needs.
	let program: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(1)
		.bind(|x| FreeExplicit::pure(x + 1))
		.bind(|x| FreeExplicit::pure(x * 10))
		.bind(|x| FreeExplicit::pure(x + 5));
	assert_eq!(program.evaluate(), 25);
}
