#![expect(
	clippy::panic,
	reason = "Tests use panic! in match arms that should be unreachable on success, for brevity and clarity."
)]

//! Row-canonicalisation regression baseline migrated from
//! `poc-effect-row/tests/`. Covers the workaround-1 (proc-macro
//! sorting) and workaround-3 (CoproductSubsetter for hand-written
//! permutations) hybrid that
//! [`effects!`](fp_library::effects) and the internal
//! [`raw_effects!`](fp_library::__internal::raw_effects) macros
//! together implement (per
//! [decisions.md](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/decisions.md)
//! section 4.1).
//!
//! Coverage focus:
//!
//! - **Canonicalisation**: input order does not affect the resulting
//!   row brand (workaround 1).
//! - **Scaling**: 5- and 7-brand rows compile through the macro
//!   without trait-inference blow-up.
//! - **Workaround 3 (CoproductSubsetter)**: hand-written non-canonical
//!   runtime [`Coproduct`](fp_library::types::effects::coproduct::Coproduct)
//!   values can be `.subset()`-ed into the canonical permutation.
//! - **Coyoneda wrapping**: `effects!` emits Coyoneda-wrapped rows;
//!   `raw_effects!` emits raw rows.
//! - **Run-family integration**: each row brand drives all six Run
//!   wrappers (Erased: `Run`/`RcRun`/`ArcRun`; Explicit:
//!   `RunExplicit`/`RcRunExplicit`/`ArcRunExplicit`). Arc-family
//!   wrappers require `ArcCoyonedaBrand`-headed rows because the
//!   `Arc`-substrate's struct-level
//!   `Of<'static, ArcFree<..., ArcTypeErasedValue>>: Send + Sync`
//!   bound does not hold for bare `CoyonedaBrand` (its
//!   `Box<dyn FnOnce>` continuation is not `Send + Sync`).
//!
//! Tests intentionally skipped from the POC migration:
//!
//! - **`feasibility::t08` (lifetime-parameter-bearing raw effect)**:
//!   the POC tested that the macro tolerates raw effect types with
//!   lifetime parameters. Production effect brands are zero-sized
//!   `'static` markers (no lifetime params at the brand level), so
//!   the test does not translate to production.
//! - **`feasibility::t10` (handler accepts macro output as runtime
//!   value)**: in POC, effect types served as both row brands AND
//!   runtime value types, so the macro's emitted row was directly
//!   constructable. In production, brand types are zero-sized
//!   markers; the analog is the all-six-Run-wrappers integration
//!   tests below (a row brand drives the wrapper's type
//!   parameters).
//! - **`feasibility::t14`-`t16` (tstr_crates demos)**: these
//!   demonstrate `tstr_crates`'s compile-time string-ordering
//!   primitives, not fp-library's behaviour. Adding `tstr` as a
//!   dev-dep would not strengthen the regression baseline.
//! - **`coyoneda::c03`-`c05` (POC-local Coyoneda lift+decoder
//!   mechanics)**: production
//!   [`Coyoneda`](fp_library::types::Coyoneda) has its own `lift`
//!   API (no decoder closure needed) and is covered by its own unit
//!   tests; the POC's mechanics-tests don't translate.
//! - **`coyoneda::c08` (Coproduct-of-Coyoneda fmap dispatch)**: this
//!   property is exercised end-to-end by
//!   [`tests/run_lift.rs`](https://github.com/nothingnesses/rust-fp-library/blob/main/fp-library/tests/run_lift.rs)'s
//!   round-trip tests, which lift through Coyoneda, peel, and lower
//!   to recover the value across all six Run wrappers.
//!
//! See deviations.md step 10a for the full migration mapping.

use {
	core::marker::PhantomData,
	fp_library::{
		__internal::raw_effects,
		brands::{
			ArcCoyonedaBrand,
			BoxBrand,
			CNilBrand,
			CatListBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
			OptionBrand,
			ResultBrand,
			SendThunkBrand,
			ThunkBrand,
			TryThunkBrand,
		},
		effects,
		types::{
			Coyoneda,
			Identity,
			effects::{
				arc_run::ArcRun,
				arc_run_explicit::ArcRunExplicit,
				coproduct::{
					CNil,
					Coproduct,
					CoproductSubsetter,
				},
				rc_run::RcRun,
				rc_run_explicit::RcRunExplicit,
				run::Run,
				run_explicit::RunExplicit,
			},
		},
	},
};

// Compile-time type-equality assertion. Compiles iff `T == U`.
fn assert_type_eq<T>(
	_: PhantomData<T>,
	_: PhantomData<T>,
) {
}

// -- Canonicalisation: workaround 1 (proc-macro sorting) --

#[test]
fn raw_effects_two_brands_canonicalise_across_orderings() {
	// Both orderings of {IdentityBrand, OptionBrand} resolve to the
	// same Coproduct row type.
	type Forward = raw_effects![IdentityBrand, OptionBrand];
	type Reverse = raw_effects![OptionBrand, IdentityBrand];
	assert_type_eq::<Forward>(PhantomData, PhantomData::<Reverse>);
}

#[test]
fn raw_effects_three_brands_six_permutations() {
	// All 3! = 6 permutations of {IdentityBrand, OptionBrand,
	// ResultBrand} resolve to the same canonical type.
	type P1 = raw_effects![IdentityBrand, OptionBrand, ResultBrand];
	type P2 = raw_effects![IdentityBrand, ResultBrand, OptionBrand];
	type P3 = raw_effects![OptionBrand, IdentityBrand, ResultBrand];
	type P4 = raw_effects![OptionBrand, ResultBrand, IdentityBrand];
	type P5 = raw_effects![ResultBrand, IdentityBrand, OptionBrand];
	type P6 = raw_effects![ResultBrand, OptionBrand, IdentityBrand];

	assert_type_eq::<P1>(PhantomData, PhantomData::<P2>);
	assert_type_eq::<P2>(PhantomData, PhantomData::<P3>);
	assert_type_eq::<P3>(PhantomData, PhantomData::<P4>);
	assert_type_eq::<P4>(PhantomData, PhantomData::<P5>);
	assert_type_eq::<P5>(PhantomData, PhantomData::<P6>);
}

#[test]
fn raw_effects_canonical_form_is_lexical() {
	// Sorting by stringified type yields IdentityBrand <
	// OptionBrand < ResultBrand, so the canonical nested
	// CoproductBrand mirrors that order.
	type Effs = raw_effects![ResultBrand, IdentityBrand, OptionBrand];
	type Expected = CoproductBrand<
		IdentityBrand,
		CoproductBrand<OptionBrand, CoproductBrand<ResultBrand, CNilBrand>>,
	>;
	assert_type_eq::<Effs>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn raw_effects_empty_is_cnil_brand() {
	type Empty = raw_effects![];
	assert_type_eq::<Empty>(PhantomData, PhantomData::<CNilBrand>);
}

#[test]
fn raw_effects_single_brand_wraps_in_cnil() {
	type Single = raw_effects![IdentityBrand];
	type Expected = CoproductBrand<IdentityBrand, CNilBrand>;
	assert_type_eq::<Single>(PhantomData, PhantomData::<Expected>);
}

// -- Same-root, different generic params: sort distinguishes --

#[test]
fn raw_effects_same_root_different_params_sort_consistently() {
	// CoyonedaBrand<IdentityBrand> and CoyonedaBrand<OptionBrand> are
	// different types; both orderings still canonicalise to the same
	// form.
	type Forward = raw_effects![CoyonedaBrand<IdentityBrand>, CoyonedaBrand<OptionBrand>];
	type Reverse = raw_effects![CoyonedaBrand<OptionBrand>, CoyonedaBrand<IdentityBrand>];
	assert_type_eq::<Forward>(PhantomData, PhantomData::<Reverse>);
}

// -- Scaling: 5- and 7-brand rows --

#[test]
fn raw_effects_five_brand_canonicalisation() {
	type Order1 = raw_effects![ResultBrand, IdentityBrand, BoxBrand, OptionBrand, CatListBrand];
	type Order2 = raw_effects![BoxBrand, CatListBrand, IdentityBrand, ResultBrand, OptionBrand];
	type Order3 = raw_effects![IdentityBrand, OptionBrand, ResultBrand, BoxBrand, CatListBrand];
	assert_type_eq::<Order1>(PhantomData, PhantomData::<Order2>);
	assert_type_eq::<Order2>(PhantomData, PhantomData::<Order3>);
}

#[test]
fn raw_effects_seven_brand_canonicalisation() {
	type Effs = raw_effects![
		ThunkBrand,
		IdentityBrand,
		TryThunkBrand,
		BoxBrand,
		SendThunkBrand,
		OptionBrand,
		ResultBrand
	];
	type Canonical = raw_effects![
		BoxBrand,
		IdentityBrand,
		OptionBrand,
		ResultBrand,
		SendThunkBrand,
		ThunkBrand,
		TryThunkBrand
	];
	assert_type_eq::<Effs>(PhantomData, PhantomData::<Canonical>);
}

// -- Workaround 3: CoproductSubsetter for hand-written permutations --
//
// `CoproductSubsetter` (re-exported from `frunk_core`) operates on
// runtime [`Coproduct`] values, separately from the macros (which
// emit type-level brand rows). The test below demonstrates that a
// user can hand-write a non-canonical permutation of a row's
// runtime-value shape and `.subset()` it into the canonical
// permutation. Each variant must be a distinct value type so the
// `Member`-style position-by-type inference resolves unambiguously.

#[test]
fn subsetter_lets_handler_accept_non_canonical_runtime_coproduct() {
	// Three distinct value types so subset() resolves uniquely.
	type Canonical = Coproduct<i32, Coproduct<bool, Coproduct<&'static str, CNil>>>;
	type HandWritten = Coproduct<&'static str, Coproduct<i32, Coproduct<bool, CNil>>>;

	let hand: HandWritten = Coproduct::Inr(Coproduct::Inl(7)); // i32(7)
	let canonical: Canonical = hand.subset().expect("permutation");
	let _ = canonical;
}

#[test]
fn subsetter_via_explicit_trait_call() {
	// Same as above but `.subset()` is rewritten as a fully-qualified
	// trait method call. Equivalent at runtime; documents the trait
	// dependency.
	type Source = Coproduct<bool, Coproduct<i32, Coproduct<&'static str, CNil>>>;
	type Target = Coproduct<i32, Coproduct<bool, Coproduct<&'static str, CNil>>>;
	let hand: Source = Coproduct::Inr(Coproduct::Inr(Coproduct::Inl("hello")));
	let canonical: Target =
		<Source as CoproductSubsetter<Target, _>>::subset(hand).expect("permutation");
	let _ = canonical;
}

#[test]
fn subsetter_handles_five_value_permutation() {
	// 5-element runtime permutation. Hand-write a Coproduct in
	// reverse order; subset to the canonical type-order.
	type Canonical = Coproduct<
		i32,
		Coproduct<bool, Coproduct<&'static str, Coproduct<u8, Coproduct<f32, CNil>>>>,
	>;
	type Reverse = Coproduct<
		f32,
		Coproduct<u8, Coproduct<&'static str, Coproduct<bool, Coproduct<i32, CNil>>>>,
	>;
	let hand: Reverse = Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(true))));
	let _canonical: Canonical = hand.subset().expect("5-element permutation");
}

// -- Coyoneda wrapping: effects! vs raw_effects! --

#[test]
fn effects_two_brands_canonicalise_with_coyoneda_wrap() {
	// effects! wraps each brand in CoyonedaBrand and sorts the
	// result. Forward / Reverse orderings yield the same canonical
	// row.
	type Forward = effects![IdentityBrand, OptionBrand];
	type Reverse = effects![OptionBrand, IdentityBrand];
	type Expected = CoproductBrand<
		CoyonedaBrand<IdentityBrand>,
		CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>,
	>;
	assert_type_eq::<Forward>(PhantomData, PhantomData::<Reverse>);
	assert_type_eq::<Forward>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn effects_generic_brands_canonicalise_with_coyoneda_wrap() {
	// Generic brands (parameterised by another brand) also sort and
	// wrap correctly. CoyonedaBrand wraps a CoyonedaBrand<...> from
	// inside the row.
	type Forward = effects![CoyonedaBrand<IdentityBrand>, CoyonedaBrand<OptionBrand>];
	type Reverse = effects![CoyonedaBrand<OptionBrand>, CoyonedaBrand<IdentityBrand>];
	assert_type_eq::<Forward>(PhantomData, PhantomData::<Reverse>);
}

#[test]
fn raw_effects_skips_coyoneda_wrap_vs_effects() {
	// raw_effects! emits raw brand-Coproduct without Coyoneda;
	// effects! wraps each brand in CoyonedaBrand. The two macros
	// produce different canonical rows for the same input.
	type Raw = raw_effects![IdentityBrand, OptionBrand];
	type Wrapped = effects![IdentityBrand, OptionBrand];
	type ExpectedRaw = CoproductBrand<IdentityBrand, CoproductBrand<OptionBrand, CNilBrand>>;
	type ExpectedWrapped = CoproductBrand<
		CoyonedaBrand<IdentityBrand>,
		CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>,
	>;
	assert_type_eq::<Raw>(PhantomData, PhantomData::<ExpectedRaw>);
	assert_type_eq::<Wrapped>(PhantomData, PhantomData::<ExpectedWrapped>);
}

#[test]
fn subsetter_over_runtime_coyoneda_wrapped_values() {
	// Runtime CoproductSubsetter mediates a permutation of a
	// Coproduct whose variants are themselves runtime Coyoneda
	// values over different inner functors. This is the production
	// equivalent of the POC's `coyoneda::c06`: brand types are
	// zero-sized, so the runtime mediation operates on
	// `Coyoneda<EBrand, A>` value types directly, not on the
	// macro-emitted brand row.
	let id_coyo: Coyoneda<'static, IdentityBrand, i32> = Coyoneda::lift(Identity(7));

	// Canonical (lexical) order at the runtime-value level:
	// Coyoneda<IdentityBrand, i32> < Coyoneda<OptionBrand, i32>.
	type Canonical = Coproduct<
		Coyoneda<'static, IdentityBrand, i32>,
		Coproduct<Coyoneda<'static, OptionBrand, i32>, CNil>,
	>;
	// User hand-writes a non-canonical permutation.
	type Hand = Coproduct<
		Coyoneda<'static, OptionBrand, i32>,
		Coproduct<Coyoneda<'static, IdentityBrand, i32>, CNil>,
	>;

	// Build a hand-written value occupying the IdentityBrand-Coyoneda
	// position (Inr(Inl(...))) of the Hand layout.
	let hand_value: Hand = Coproduct::Inr(Coproduct::Inl(id_coyo));

	// .subset() mediates the type-level permutation at runtime,
	// recovering the canonical layout. The IdentityBrand-Coyoneda
	// value is now in position Inl.
	let canonical: Canonical = hand_value.subset().expect("permutation");
	match canonical {
		Coproduct::Inl(coyo) => {
			let Identity(value) = coyo.lower();
			assert_eq!(value, 7);
		}
		Coproduct::Inr(_) => panic!("expected Inl after subset()"),
	}
}

// -- Run-family integration: row brand drives all six Run wrappers --
//
// Erased single-shot: Run uses bare CoyonedaBrand-headed rows.
// Erased multi-shot Rc: RcRun uses bare CoyonedaBrand or RcCoyoneda;
// here we use bare CoyonedaBrand (sufficient for the pure case).
// Erased multi-shot Arc: ArcRun's struct-level HRTB requires
// ArcCoyonedaBrand-headed rows (CoyonedaBrand's Box<dyn FnOnce>
// continuation is not Send + Sync). Same constraint applies to
// ArcRunExplicit.

type ErasedRow = effects![IdentityBrand];
type ArcErasedRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type Scoped = CNilBrand;

#[test]
fn effects_row_drives_run_wrapper() {
	let _run: Run<ErasedRow, Scoped, i32> = Run::pure(7);
}

#[test]
fn effects_row_drives_rc_run_wrapper() {
	let _run: RcRun<ErasedRow, Scoped, i32> = RcRun::pure(7);
}

#[test]
fn effects_row_drives_arc_run_wrapper() {
	// ArcRun requires ArcCoyonedaBrand-headed rows because the
	// substrate's struct-level HRTB needs the row's Of-projection to
	// be Send + Sync.
	let _run: ArcRun<ArcErasedRow, Scoped, i32> = ArcRun::pure(7);
}

#[test]
fn effects_row_drives_run_explicit_wrapper() {
	let _run: RunExplicit<'static, ErasedRow, Scoped, i32> = RunExplicit::pure(7);
}

#[test]
fn effects_row_drives_rc_run_explicit_wrapper() {
	let _run: RcRunExplicit<'static, ErasedRow, Scoped, i32> = RcRunExplicit::pure(7);
}

#[test]
fn effects_row_drives_arc_run_explicit_wrapper() {
	let _run: ArcRunExplicit<'static, ArcErasedRow, Scoped, i32> = ArcRunExplicit::pure(7);
}
