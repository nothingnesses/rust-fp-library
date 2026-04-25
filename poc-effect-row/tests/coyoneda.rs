//! POC: static-via-Coyoneda for port-plan section 4.2.
//!
//! Question: when `VariantF<Coproduct<Eff1, Eff2, ...>>` needs to
//! implement `Functor`, can the static option (each variant must
//! be a `Functor`) be satisfied by wrapping each effect in
//! `Coyoneda<E>` at construction time? Section 5.2 commits to this
//! path; this POC tests:
//!
//!   c01 - `effects_coyo!` emits a Coproduct over Coyoneda-wrapped
//!         variants and canonicalises across input orderings.
//!   c02 - The wrapping does not break the lexical sort: two
//!         orderings of the same inner effect set produce the same
//!         outer Coyoneda-wrapped row.
//!   c03 - Coyoneda<F, A> implements the POC's Functor trait for any
//!         F, regardless of whether F is itself a Functor. This is
//!         the key claim of the static-via-Coyoneda path.
//!   c04 - `map` over a Coyoneda value composes the function without
//!         touching the inner F. (Confirms the lazy-fmap semantics.)
//!   c05 - `lower` runs the composed function over the stored F to
//!         produce the final A. End-to-end round-trip.
//!   c06 - The macro plus Coyoneda story integrates with the
//!         workaround-3 fallback: hand-written non-canonical rows
//!         of Coyoneda-wrapped effects can be subset()ed into the
//!         macro-emitted canonical row.
//!   c07 - Generic effect types wrap and sort correctly under
//!         `effects_coyo!`.

use {
	frunk_core::coproduct::{
		CNil,
		Coproduct,
	},
	poc_effect_row::{
		Functor,
		coyoneda::Coyoneda,
		effects_coyo,
	},
	static_assertions::assert_type_eq_all,
};

// Plain-enum effects with NO Functor impl. The static-via-Coyoneda
// claim is that we can lift these into a row that participates in
// Functor dispatch via the Coyoneda wrap.
#[derive(Debug)]
struct Logger(String);
#[derive(Debug)]
struct Reader<E>(std::marker::PhantomData<E>);
#[derive(Debug)]
struct State<S>(std::marker::PhantomData<S>);

// -- c01: macro emits Coyoneda-wrapped Coproduct in canonical order --

#[test]
fn c01_macro_emits_coyoneda_wrapped_coproduct() {
	type Row = effects_coyo![i32; Logger, Reader<()>];
	type Expected = Coproduct<Coyoneda<Logger, i32>, Coproduct<Coyoneda<Reader<()>, i32>, CNil>>;
	assert_type_eq_all!(Row, Expected);
}

// -- c02: two orderings canonicalise --

#[test]
fn c02_two_orderings_canonicalise_with_coyoneda_wrap() {
	type Forward = effects_coyo![i32; Logger, Reader<()>];
	type Reverse = effects_coyo![i32; Reader<()>, Logger];
	assert_type_eq_all!(Forward, Reverse);
}

// -- c03: Coyoneda<F, A> implements Functor for any F --

#[test]
fn c03_coyoneda_is_functor_for_any_inner() {
	// Logger is not a Functor on its own. Coyoneda<Logger, A> IS.
	let wrapped: Coyoneda<Logger, &'static str> =
		Coyoneda::lift(Logger("hello".to_string()), |any| {
			// In a real Coyoneda, the lifted decoder converts the
			// erased intermediate type B back to A. Here we just
			// downcast and return.
			*any.downcast::<&'static str>().expect("type-correct lift")
		});

	// We can fmap from &str to usize without ever touching Logger.
	let mapped: Coyoneda<Logger, usize> = wrapped.fmap(|s: &'static str| s.len());

	// The inner F is unchanged.
	assert_eq!(mapped.fb.0, "hello");
}

// -- c04: map composes the function without touching F --

#[test]
fn c04_map_composes_lazily() {
	let coyo: Coyoneda<Logger, i32> = Coyoneda::lift(Logger("seed".to_string()), |any| {
		// Pretend the encoder produces 42 from the inner state.
		*any.downcast::<i32>().expect("type-correct lift")
	});

	// Chain three maps.
	let result: Coyoneda<Logger, String> =
		coyo.fmap(|n: i32| n + 1).fmap(|n: i32| n * 2).fmap(|n: i32| format!("result={n}"));

	// The inner F has been carried through untouched.
	assert_eq!(result.fb.0, "seed");
}

// -- c05: lower runs the composed function for end-to-end round-trip --

#[test]
fn c05_lower_runs_composed_function() {
	let coyo: Coyoneda<i32, i32> = Coyoneda::lift(7, |any| {
		// Decoder: extract the i32 from the erased Box<dyn Any>.
		*any.downcast::<i32>().expect("type-correct lift")
	});

	let mapped: Coyoneda<i32, i32> = coyo.fmap(|n: i32| n + 1).fmap(|n: i32| n * 2);

	// Lower: feed the original 7 through (7 -> 8 -> 16).
	let final_value: i32 = mapped.lower(|fb: i32| Box::new(fb) as Box<dyn std::any::Any>);
	assert_eq!(final_value, 16);
}

// -- c06: workaround-3 fallback over Coyoneda-wrapped rows --

#[test]
fn c06_subsetter_works_over_coyoneda_wrapped_rows() {
	// Macro-emitted canonical row over A = ().
	type Canonical = effects_coyo![(); Logger, Reader<()>];

	// User hand-writes a non-canonical row over the same wrapped
	// variants.
	type Hand = Coproduct<Coyoneda<Reader<()>, ()>, Coproduct<Coyoneda<Logger, ()>, CNil>>;

	// Construct a Reader-flavoured value and route it through.
	let r: Coyoneda<Reader<()>, ()> = Coyoneda::lift(Reader(std::marker::PhantomData), |any| {
		*any.downcast::<()>().expect("type-correct lift")
	});
	let hand_value: Hand = Coproduct::Inl(r);

	// CoproductSubsetter mediates the permutation.
	let _canonical: Canonical = hand_value.subset().expect("permutation");
}

// -- c07: generic effects sort correctly --

#[test]
fn c07_generic_effects_canonicalise_under_coyoneda_wrap() {
	struct Env;
	struct S;
	type Forward = effects_coyo![bool; Reader<Env>, State<S>];
	type Reverse = effects_coyo![bool; State<S>, Reader<Env>];
	assert_type_eq_all!(Forward, Reverse);
}

// -- c08: Coproduct-of-Coyoneda implements Functor end-to-end --

#[test]
fn c08_coproduct_dispatches_fmap_to_active_variant() {
	// This is the actual section-4.2 question: when `VariantF<R>`
	// needs `Functor`, does the static option (each variant in R is
	// itself a Functor via Coyoneda) work end-to-end? The recursive
	// Functor impls on Coproduct + CNil + Coyoneda compose so that
	// `fmap` on the row dispatches to the active variant's `fmap`
	// without any runtime dictionary.

	type Row = effects_coyo![&'static str; Logger, Reader<()>];

	let logger: Coyoneda<Logger, &'static str> =
		Coyoneda::lift(Logger("seed".to_string()), |any| {
			*any.downcast::<&'static str>().expect("type-correct lift")
		});
	let row: Row = Coproduct::Inl(logger);

	// fmap on the row: the trait dispatches to Logger's variant,
	// which dispatches to Coyoneda::fmap, which composes the
	// closure. No runtime Functor lookup; pure trait resolution.
	let mapped = row.fmap(|s: &'static str| s.len());

	// Verify the type is what we expect: same shape as Row but with
	// the answer type swapped from &str to usize.
	type Expected =
		Coproduct<Coyoneda<Logger, usize>, Coproduct<Coyoneda<Reader<()>, usize>, CNil>>;
	let _check: Expected = mapped;

	// (Skipping the full lower() round-trip here; tests c04-c05
	// already cover the fmap composition semantics for one Coyoneda
	// variant. The point of c08 is demonstrating that the row's
	// type-level dispatch picks the right variant's fmap.)
}
