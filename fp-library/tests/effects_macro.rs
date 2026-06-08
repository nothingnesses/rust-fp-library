#![cfg(feature = "effects")]

// Integration tests for the `effects!` macro, the internal
// `raw_effects!` macro, and the `scoped_effects!` macro.
//
// Covers:
// - Empty input -> CNilBrand.
// - Single-brand and multi-brand expansions.
// - Canonical-ordering property: input order does not affect the
//   resulting type, so two different orderings of the same brand
//   set produce the same canonical row type. This is workaround 1
//   from decisions section 4.1; the test asserts type-equality at
//   compile time via the `assert_type_eq` pattern.
// - Coyoneda wrapping: `effects!` wraps each brand in CoyonedaBrand;
//   `raw_effects!` and `scoped_effects!` do not.
// - Use as the row parameters of a Run wrapper to verify the emitted
//   row brands satisfy wrapper bounds in production.

use {
	core::marker::PhantomData,
	fp_library::{
		__internal::raw_effects,
		brands::{
			ArcCoyonedaBrand,
			BoxBrand,
			BoxSpanBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
			OptionBrand,
			RcCoyonedaBrand,
		},
		define_effect_row_aliases,
		effects,
		kinds::Kind_cdc7cd43dac7585f,
		scoped_effects,
		types::effects::{
			coproduct::{
				CNil,
				Coproduct,
			},
			rc_run::RcRun,
			span::BoxSpan,
		},
	},
};

// Compile-time type-equality assertion. Compiles iff `T == U`.
fn assert_type_eq<T>(
	_: PhantomData<T>,
	_: PhantomData<T>,
) {
}

// -- effects! --

#[test]
fn effects_empty() {
	assert_type_eq::<effects![]>(PhantomData, PhantomData::<CNilBrand>);
}

#[test]
fn effects_single_brand() {
	type Row = effects![IdentityBrand];
	type Expected = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	assert_type_eq::<Row>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn effects_two_brands_canonical() {
	type R1 = effects![IdentityBrand, OptionBrand];
	type R2 = effects![OptionBrand, IdentityBrand];
	// Both orderings produce the same canonical type.
	assert_type_eq::<R1>(PhantomData, PhantomData::<R2>);
}

#[test]
fn effects_two_brands_explicit_shape() {
	type Row = effects![IdentityBrand, OptionBrand];
	// Lexical sort: "IdentityBrand" < "OptionBrand" -> Identity first.
	type Expected = CoproductBrand<
		CoyonedaBrand<IdentityBrand>,
		CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>,
	>;
	assert_type_eq::<Row>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn effects_three_brands_canonical() {
	// Three different orderings; all must produce the same canonical type.
	type R1 = effects![IdentityBrand, OptionBrand, ConstructorIBrand];
	type R2 = effects![OptionBrand, ConstructorIBrand, IdentityBrand];
	type R3 = effects![ConstructorIBrand, IdentityBrand, OptionBrand];
	assert_type_eq::<R1>(PhantomData, PhantomData::<R2>);
	assert_type_eq::<R1>(PhantomData, PhantomData::<R3>);
}

// Helper: a minimal user-defined brand to round out the three-brand test.
// Named with a leading "Constructor" so it sorts after Identity but before Option.
struct ConstructorIBrand;

// Local scoped brands used to assert `scoped_effects!` type shape.
struct AlphaScopedBrand;
struct BetaScopedBrand;

define_effect_row_aliases! {
	type AliasFirstRow = first_order [OptionBrand, IdentityBrand];
	type AliasFirstRowMinusIdentity = first_order [OptionBrand];
	type AliasRcFirstRow = rc_first_order [IdentityBrand];
	type AliasArcFirstRow = arc_first_order [IdentityBrand];
	type AliasScopedRow = scoped [BetaScopedBrand, AlphaScopedBrand];
	type AliasEmptyRow = first_order [];
}

// -- raw_effects! --

#[test]
fn raw_effects_empty() {
	assert_type_eq::<raw_effects![]>(PhantomData, PhantomData::<CNilBrand>);
}

#[test]
fn raw_effects_skips_coyoneda_wrap() {
	type Row = raw_effects![IdentityBrand, OptionBrand];
	// No CoyonedaBrand wrapping: brands appear directly.
	type Expected = CoproductBrand<IdentityBrand, CoproductBrand<OptionBrand, CNilBrand>>;
	assert_type_eq::<Row>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn raw_effects_canonical_order() {
	type R1 = raw_effects![IdentityBrand, OptionBrand];
	type R2 = raw_effects![OptionBrand, IdentityBrand];
	assert_type_eq::<R1>(PhantomData, PhantomData::<R2>);
}

// -- scoped_effects! --

#[test]
fn scoped_effects_empty() {
	assert_type_eq::<scoped_effects![]>(PhantomData, PhantomData::<CNilBrand>);
}

#[test]
fn scoped_effects_skips_coyoneda_wrap() {
	type Row = scoped_effects![AlphaScopedBrand, BetaScopedBrand];
	type Expected = CoproductBrand<AlphaScopedBrand, CoproductBrand<BetaScopedBrand, CNilBrand>>;
	assert_type_eq::<Row>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn scoped_effects_canonical_order() {
	type R1 = scoped_effects![AlphaScopedBrand, BetaScopedBrand];
	type R2 = scoped_effects![BetaScopedBrand, AlphaScopedBrand];
	assert_type_eq::<R1>(PhantomData, PhantomData::<R2>);
}

// -- define_effect_row_aliases! --

#[test]
fn define_effect_row_aliases_first_order_alias() {
	type Expected = CoproductBrand<
		CoyonedaBrand<IdentityBrand>,
		CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>,
	>;
	assert_type_eq::<AliasFirstRow>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn define_effect_row_aliases_row_minus_alias() {
	type Expected = CoproductBrand<CoyonedaBrand<OptionBrand>, CNilBrand>;
	assert_type_eq::<AliasFirstRowMinusIdentity>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn define_effect_row_aliases_shared_wrapper_aliases() {
	type RcExpected = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	type ArcExpected = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
	assert_type_eq::<AliasRcFirstRow>(PhantomData, PhantomData::<RcExpected>);
	assert_type_eq::<AliasArcFirstRow>(PhantomData, PhantomData::<ArcExpected>);
}

#[test]
fn define_effect_row_aliases_scoped_alias() {
	type Expected = CoproductBrand<AlphaScopedBrand, CoproductBrand<BetaScopedBrand, CNilBrand>>;
	assert_type_eq::<AliasScopedRow>(PhantomData, PhantomData::<Expected>);
}

#[test]
fn define_effect_row_aliases_empty_alias() {
	assert_type_eq::<AliasEmptyRow>(PhantomData, PhantomData::<CNilBrand>);
}

#[test]
fn scoped_effects_span_row_keeps_action_slot_in_projection() {
	fn assert_projection<'a>(_lifetime: PhantomData<&'a ()>) {
		type Row = scoped_effects![BoxSpanBrand<BoxBrand, &'static str>];
		type Actual<'a> = <Row as Kind_cdc7cd43dac7585f>::Of<'a, &'a str>;
		type Expected<'a> = Coproduct<BoxSpan<'a, BoxBrand, &'static str, &'a str>, CNil>;

		assert_type_eq::<Actual<'a>>(PhantomData, PhantomData::<Expected<'a>>);
	}

	assert_projection(PhantomData);
}

// -- Production use: row brand drives a Run wrapper --
//
// The Coyoneda-wrapped row (the canonical `effects!` output) and the
// empty scoped row satisfy the Run wrapper's row-level struct bounds,
// so `RcRun::pure` constructs successfully. Inspecting the program via
// `peel` requires an additional Clone bound on the row's projection,
// which Coyoneda-wrapped rows don't satisfy in the Erased family; that
// path is exercised by `raw_effects!`-style tests elsewhere. The
// construction test here is sufficient to prove the emitted brand
// satisfies the wrapper's struct-level bounds.

#[test]
fn effects_row_drives_run_wrapper() {
	type Row = effects![IdentityBrand];
	type Scoped = scoped_effects![];
	let _run: RcRun<Row, Scoped, i32> = RcRun::pure(42);
}

#[test]
fn raw_effects_row_drives_run_wrapper_with_peel() {
	// raw_effects! emits the un-wrapped row that the Erased Rc family's
	// `peel` example uses; it satisfies the Clone bound that
	// Coyoneda-wrapping breaks.
	type Row = raw_effects![IdentityBrand];
	type Scoped = CNilBrand;
	let run: RcRun<Row, Scoped, i32> = RcRun::pure(42);
	assert!(matches!(run.peel(), Ok(42)));
}
