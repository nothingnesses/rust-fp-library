// Integration tests for `fp_library::types::FreeExplicit` covering the
// load-bearing properties of the naive recursive Free substrate:
//
// 1. The brand integrates with the `Kind` system so `FreeExplicit` can
//    be type-applied via the existing Brand macros.
// 2. The type carries non-`'static` payloads (e.g., `&'a str`) and
//    closures that borrow from non-`'static` scopes.
// 3. `evaluate` interprets a deeply nested `Wrap` chain to completion
//    iteratively, never recursing.
// 4. Custom `Drop` dismantles a deeply nested `Wrap` chain iteratively
//    via `<F as WrapDrop>::drop(...)`, never recursing.
// 5. Chained `bind`s compose over a concrete functor (`IdentityBrand`).
//
// The struct is bounded `F: WrapDrop + 'a`; the inherent methods
// additionally require `F: Functor` (the recursive `bind` walks the
// spine via `F::map`); `evaluate` additionally requires `F: Extract`.
// Effect functors that lack a canonical `Extract` (e.g., `OptionBrand`,
// where `None` has no value to surrender) cannot reach their result
// through `FreeExplicit::evaluate` directly; they reach it through
// handler interpretation instead.

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

// -- Brand integrates with the Kind system --

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

// -- Borrowed payload (non-`'static`) --

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

// -- Iterative `evaluate` on a very deep chain does not overflow --

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

// -- Iterative `Drop` on a very deep chain does not overflow --
//
// `FreeExplicit`'s custom `Drop` walks the chain via
// `<F as WrapDrop>::drop(fa)`, dismantling each `Wrap` layer
// iteratively in a `loop`. With `IdentityBrand: WrapDrop` returning
// `Some` (delegating to `Extract::extract`), a 100 000-deep chain
// dismantles without growing the call stack.

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

// -- Chained binds compose --

#[test]
fn q5_identity_chained_binds() {
	// A shallow sanity check that bind composes over IdentityBrand.
	let program: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(1)
		.bind(|x| FreeExplicit::pure(x + 1))
		.bind(|x| FreeExplicit::pure(x * 10))
		.bind(|x| FreeExplicit::pure(x + 5));
	assert_eq!(program.evaluate(), 25);
}
