// Integration tests for the `effects!` macro and the internal
// `raw_effects!` macro.
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
//   `raw_effects!` does not.
// - Use as the `R` parameter of a Run wrapper to verify the emitted
//   row brand satisfies the wrapper's bounds in production.

use {
	core::marker::PhantomData,
	fp_library::{
		__internal::raw_effects,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
			OptionBrand,
		},
		effects,
		types::effects::rc_run::RcRun,
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

// -- Production use: row brand drives a Run wrapper --
//
// The Coyoneda-wrapped row (the canonical `effects!` output) satisfies
// the Run wrapper's `R: WrapDrop + Functor + 'static` struct-level
// bound, so `RcRun::pure` constructs successfully. (Inspecting the
// program via `peel` requires an additional Clone bound on the row's
// projection, which Coyoneda-wrapped rows don't satisfy in the Erased
// family; that path is exercised by `raw_effects!`-style tests
// elsewhere. The construction test here is sufficient to prove the
// emitted brand satisfies the wrapper's struct-level bounds.)

#[test]
fn effects_row_drives_run_wrapper() {
	type Row = effects![IdentityBrand];
	type Scoped = CNilBrand;
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
