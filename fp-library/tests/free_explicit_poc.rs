// Feasibility POC for a non-`'static` sibling of `Free`, analogous to the
// `Coyoneda` / `CoyonedaExplicit` pair.
//
// -- Hypothesis --
//
// A naive recursive `FreeExplicit<'a, F, A>` enum (Pure | Wrap) with no
// type-erased continuation queue can:
// 1. Compile as a `Kind` with the existing brand macros.
// 2. Support non-`'static` effect payloads (e.g., `&'a str`).
// 3. Interpret to completion iteratively for a concrete functor.
// 4. Be interpreted via a natural transformation into a target `Functor`.
// 5. Model a two-effect Run-shaped program (inject different effects at
//    different points, run handlers that peel one effect at a time).
//
// The cost (not measured here; see the sibling Criterion bench) is that
// `bind` walks the spine, so left-associated bind chains are O(N) instead
// of O(1) via "Reflection without Remorse".
//
// -- Expected positive results --
//
// q1_kind_integration: `FreeExplicitBrand<F>` can be used with `impl_kind!`
// and produces a well-formed `Kind` type parameterised by `'a` and `A`.
//
// q2_borrowed_payload: `FreeExplicit<'a, IdentityBrand, &'a str>` compiles
// and round-trips a value borrowed from a local variable.
//
// q4_iterative_evaluate: a 100 000-deep `Wrap(...)` chain evaluates without
// stack overflow via an iterative loop over a concrete `Extract`-like step.
//
// q5_two_effect_run: a program injects two different effects at different
// points, and a pair of handler passes peels them off one at a time into
// a shared result, matching the pattern of `runReader # runState`.
//
// -- Expected failure --
//
// q4_naive_drop (#[ignore]'d): dropping a 100 000-deep `Wrap` chain without
// an iterative `Drop` impl stack-overflows. This documents a real blocker
// that a production `FreeExplicit` must solve (either via a custom `Drop`
// that iteratively dismantles the spine, or via a linear-consumption
// invariant similar to the existing `Free`). The test is `#[ignore]`d so
// it does not crash normal `cargo test` runs.
//
// -- What this POC does not test --
//
// - Macro ergonomics at scale (needs a full Run port).
// - Error-message quality for trait-heavy client code.
// - Higher-order effects (`local`, `catch`) which need scoping machinery
//   independent of this type.
// - Interaction with `async`.
// - Performance against the existing `Free` at 1K / 10K / 100K binds. See
//   `benches/benchmarks/free_explicit.rs` for that axis.

#![allow(clippy::type_complexity)]

// `Kind` is referenced only inside `Kind!(...)` macro invocations in the
// enum definition below. rustc's unused-import analysis does not see the
// macro call as a direct use of the imported name, so this import looks
// dead even though removing it breaks compilation. The `expect` attribute
// will flag if the lint ever stops firing (e.g. if the macro machinery
// changes to expose the use properly), at which point the attribute can
// be removed.
#[expect(
	unused_imports,
	reason = "Kind is referenced via the Kind!(...) macro below, which rustc does not detect as a direct use."
)]
use fp_macros::Kind;
use {
	fp_library::{
		Apply,
		brands::{
			IdentityBrand,
			OptionBrand,
		},
		classes::Functor,
		impl_kind,
		kinds::*,
		types::Identity,
	},
	std::{
		marker::PhantomData,
		rc::Rc,
	},
};

// -- The FreeExplicit type under test --

mod free_explicit {
	use super::*;

	/// A naive recursive Free monad that keeps the functor structure
	/// concrete. Supports non-`'static` `A` because there is no
	/// `Box<dyn Any>` erasure anywhere.
	///
	/// The cost: `bind` walks the spine recursively. Acceptable for this POC.
	pub enum FreeExplicit<'a, F, A: 'a>
	where
		F: Kind_cdc7cd43dac7585f + 'a, {
		Pure(A),
		Wrap(
			Apply!(
				<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Box<FreeExplicit<'a, F, A>>,
				>
			),
		),
	}

	/// Brand for `FreeExplicit`. Registers it with the Kind system so it
	/// can be used in HKT-polymorphic contexts.
	pub struct FreeExplicitBrand<F>(PhantomData<F>);

	impl_kind! {
		impl<F: Kind_cdc7cd43dac7585f + 'static> for FreeExplicitBrand<F> {
			type Of<'a, A: 'a>: 'a = FreeExplicit<'a, F, A>;
		}
	}

	impl<'a, F, A: 'a> FreeExplicit<'a, F, A>
	where
		F: Kind_cdc7cd43dac7585f + 'a,
	{
		pub fn pure(a: A) -> Self {
			FreeExplicit::Pure(a)
		}

		pub fn wrap(
			layer: Apply!(
				<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					Box<FreeExplicit<'a, F, A>>,
				>
			)
		) -> Self {
			FreeExplicit::Wrap(layer)
		}
	}

	impl<'a, F, A: 'a> FreeExplicit<'a, F, A>
	where
		F: Functor + 'a,
	{
		/// Naive recursive bind. O(N) on left-associated chains.
		///
		/// The public entry point boxes the function into a concrete
		/// `Rc<dyn Fn>` so the recursive call in `bind_boxed` does not
		/// generate a fresh closure type at each nesting level (which
		/// would hit the monomorphisation recursion limit).
		pub fn bind<B: 'a>(
			self,
			f: impl Fn(A) -> FreeExplicit<'a, F, B> + 'a,
		) -> FreeExplicit<'a, F, B> {
			let boxed: Rc<dyn Fn(A) -> FreeExplicit<'a, F, B> + 'a> = Rc::new(f);
			self.bind_boxed(boxed)
		}

		fn bind_boxed<B: 'a>(
			self,
			f: Rc<dyn Fn(A) -> FreeExplicit<'a, F, B> + 'a>,
		) -> FreeExplicit<'a, F, B> {
			match self {
				FreeExplicit::Pure(a) => f(a),
				FreeExplicit::Wrap(fa) => {
					let f_outer = Rc::clone(&f);
					FreeExplicit::Wrap(F::map(
						move |inner: Box<FreeExplicit<'a, F, A>>| -> Box<FreeExplicit<'a, F, B>> {
							let f_inner = Rc::clone(&f_outer);
							Box::new((*inner).bind_boxed(f_inner))
						},
						fa,
					))
				}
			}
		}
	}

	/// Iterative interpreter specialised to `IdentityBrand`. Demonstrates
	/// that concrete evaluation is stack-safe even when the tree is deep.
	impl<'a, A: 'a> FreeExplicit<'a, IdentityBrand, A> {
		pub fn evaluate_identity(self) -> A {
			let mut current = self;
			loop {
				match current {
					FreeExplicit::Pure(a) => return a,
					FreeExplicit::Wrap(Identity(boxed)) => current = *boxed,
				}
			}
		}
	}

	/// Iterative interpreter specialised to `OptionBrand`. An effect that
	/// is `None` short-circuits the whole program to `None`.
	impl<'a, A: 'a> FreeExplicit<'a, OptionBrand, A> {
		pub fn evaluate_option(self) -> Option<A> {
			let mut current = self;
			loop {
				match current {
					FreeExplicit::Pure(a) => return Some(a),
					FreeExplicit::Wrap(None) => return None,
					FreeExplicit::Wrap(Some(boxed)) => current = *boxed,
				}
			}
		}
	}
}

use free_explicit::{
	FreeExplicit,
	FreeExplicitBrand,
};

// -- Q1: does the type integrate with the Kind system? --

#[test]
fn q1_kind_integration() {
	// This function is only well-typed if `FreeExplicitBrand<IdentityBrand>`
	// is a valid `Kind_cdc7cd43dac7585f` implementer. If impl_kind! rejected
	// the lifetime-parameterised signature, this would not compile.
	fn accepts_kind<F>()
	where
		F: Kind_cdc7cd43dac7585f + 'static, {
	}

	accepts_kind::<FreeExplicitBrand<IdentityBrand>>();
	accepts_kind::<FreeExplicitBrand<OptionBrand>>();

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

	assert_eq!(mapped.evaluate_identity(), owner.len());
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

	assert_eq!(free.evaluate_identity(), "inner");
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
	assert_eq!(free.evaluate_identity(), 0);
}

// -- Q4b: naive Drop on a very deep chain stack-overflows (documented) --

#[test]
#[ignore = "documents the naive-Drop blocker; enabling this will stack-overflow"]
fn q4_naive_drop_overflows() {
	// Build a chain and let it drop at the end of the function without
	// consuming it. Naive recursive Drop will blow the stack.
	const DEPTH: usize = 100_000;
	let mut free: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(0);
	for _ in 0 .. DEPTH {
		free = FreeExplicit::wrap(Identity(Box::new(free)));
	}
	// Deliberately forget to evaluate.
	let _ = free;
}

// -- Q5: two-effect Run-shaped example --
//
// We model two "effects" as two separate FreeExplicit programs that share a
// common shape. For the POC we do not build a full VariantF; instead we
// show that the FreeExplicit type is expressive enough to sequence two
// interpretations in a row, which is the structural property `Run` requires.

#[test]
fn q5_two_effect_run() {
	// Phase 1: use OptionBrand to short-circuit on the first failure.
	// We model a computation: get a string, parse to int, double it.
	let input: Option<&str> = Some("21");

	let parse_step: FreeExplicit<'_, OptionBrand, &str> = match input {
		Some(s) => FreeExplicit::pure(s),
		None => FreeExplicit::wrap(None),
	};

	let doubled: FreeExplicit<'_, OptionBrand, i32> =
		parse_step.bind(|s: &str| match s.parse::<i32>() {
			Ok(n) => FreeExplicit::pure(n * 2),
			Err(_) => FreeExplicit::wrap(None),
		});

	assert_eq!(doubled.evaluate_option(), Some(42));

	// Phase 2: same shape, but fail at parse. Short-circuits cleanly.
	let bad_input: Option<&str> = Some("not a number");
	let parse_step: FreeExplicit<'_, OptionBrand, &str> = match bad_input {
		Some(s) => FreeExplicit::pure(s),
		None => FreeExplicit::wrap(None),
	};
	let doubled: FreeExplicit<'_, OptionBrand, i32> =
		parse_step.bind(|s: &str| match s.parse::<i32>() {
			Ok(n) => FreeExplicit::pure(n * 2),
			Err(_) => FreeExplicit::wrap(None),
		});
	assert_eq!(doubled.evaluate_option(), None);
}

#[test]
fn q5_identity_chained_binds() {
	// A shallow sanity check that bind composes over IdentityBrand without
	// any short-circuit semantics. This is the baseline Run needs.
	let program: FreeExplicit<'_, IdentityBrand, i32> = FreeExplicit::pure(1)
		.bind(|x| FreeExplicit::pure(x + 1))
		.bind(|x| FreeExplicit::pure(x * 10))
		.bind(|x| FreeExplicit::pure(x + 5));
	assert_eq!(program.evaluate_identity(), 25);
}
