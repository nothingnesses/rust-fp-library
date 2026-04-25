//! Feasibility tests for the workaround-1 + workaround-3 hybrid.
//!
//! Each test maps to a specific question raised by the port-plan's
//! section 4.1 ordering-mitigations subsection.
//!
//! Test naming convention: `t<NN>_<question_under_test>`.

use {
	frunk_core::coproduct::{
		CNil,
		Coproduct,
		// Workaround 3 (permutation proofs) lives in this trait. The
		// `.subset()` method on Coproduct is inherent, so Rust does not
		// need the trait imported to resolve it, but the method's
		// where-clause requires `Self: CoproductSubsetter<Target, _>`.
		// The trait is the load-bearing piece; the import is here for
		// documentation and to enable explicit use in t09a below.
		CoproductSubsetter,
	},
	poc_effect_row::effects,
	static_assertions::assert_type_eq_all,
};

// Three example effect types. Lexical order: A < B < C.
#[derive(Debug, PartialEq, Eq)]
struct A(i32);
#[derive(Debug, PartialEq, Eq)]
struct B(&'static str);
#[derive(Debug, PartialEq, Eq)]
struct C(bool);

// A generic effect, exercising sort behaviour over generic params.
#[derive(Debug, PartialEq, Eq)]
struct Reader<E>(std::marker::PhantomData<E>);
#[derive(Debug, PartialEq, Eq)]
struct State<S>(std::marker::PhantomData<S>);

// -- Test 1: simple two-element canonicalisation --

#[test]
fn t01_two_orderings_canonicalise_to_same_type() {
	// Both orderings of {A, B} resolve to the same Coproduct type.
	type Forward = effects![A, B];
	type Reverse = effects![B, A];
	assert_type_eq_all!(Forward, Reverse);
}

// -- Test 2: three-element canonicalisation across all 6 permutations --

#[test]
fn t02_three_orderings_canonicalise_across_all_permutations() {
	// All 3! = 6 permutations of {A, B, C} resolve to the same type.
	type P1 = effects![A, B, C];
	type P2 = effects![A, C, B];
	type P3 = effects![B, A, C];
	type P4 = effects![B, C, A];
	type P5 = effects![C, A, B];
	type P6 = effects![C, B, A];

	assert_type_eq_all!(P1, P2);
	assert_type_eq_all!(P2, P3);
	assert_type_eq_all!(P3, P4);
	assert_type_eq_all!(P4, P5);
	assert_type_eq_all!(P5, P6);
}

// -- Test 3: canonical form is what we expect --

#[test]
fn t03_canonical_form_is_lexical() {
	// Sorting by stringified type yields A < B < C, so the canonical
	// nested coproduct is Coproduct<A, Coproduct<B, Coproduct<C, CNil>>>.
	type Effs = effects![C, A, B];
	type Expected = Coproduct<A, Coproduct<B, Coproduct<C, CNil>>>;
	assert_type_eq_all!(Effs, Expected);
}

// -- Test 4: empty list produces CNil --

#[test]
fn t04_empty_effects_is_cnil() {
	type Empty = effects![];
	assert_type_eq_all!(Empty, CNil);
}

// -- Test 5: single effect is wrapped in Coproduct<_, CNil> --

#[test]
fn t05_single_effect_wraps_in_cnil() {
	type Single = effects![A];
	assert_type_eq_all!(Single, Coproduct<A, CNil>);
}

// -- Test 6: generic types with type parameters sort consistently --

#[test]
fn t06_generic_effects_canonicalise() {
	// Reader<Env> and State<S> sort by their full stringified form.
	struct Env;
	struct S;
	type Forward = effects![Reader<Env>, State<S>];
	type Reverse = effects![State<S>, Reader<Env>];
	assert_type_eq_all!(Forward, Reverse);
}

// -- Test 7: same root, different generic params, sort distinguishes --

#[test]
fn t07_same_root_different_params_sort_consistently() {
	// Reader<i32> and Reader<i64> are different types and sort
	// distinctly. Both orderings still canonicalise to the same form.
	type Forward = effects![Reader<i32>, Reader<i64>];
	type Reverse = effects![Reader<i64>, Reader<i32>];
	assert_type_eq_all!(Forward, Reverse);
}

// -- Test 8: lifetime parameter handling --

#[test]
fn t08_lifetime_parameters_compile() {
	// Effects with explicit lifetimes still compile through the macro.
	#[allow(dead_code)]
	struct LocalRef<'a>(&'a str);
	type _Effs = effects![A, LocalRef<'static>];
}

// -- Test 9: workaround 3 fallback - hand-written non-canonical row --

// A handler designed for the canonical row [A, B, C].
fn handle_canonical(_e: effects![A, B, C]) -> i32 {
	0
}

#[test]
fn t09_subsetter_lets_handler_accept_non_canonical() {
	// User hand-writes a coproduct in non-canonical order.
	let hand_written: Coproduct<C, Coproduct<A, Coproduct<B, CNil>>> =
		Coproduct::Inr(Coproduct::Inl(A(7)));
	// `.subset()` is sugar; CoproductSubsetter is the trait machinery
	// that proves the permutation at the call site. `subset()` returns
	// `Result<TargetCoproduct, Remainder>`; for a full superset we
	// expect `Ok`.
	let canonical: effects![A, B, C] =
		hand_written.subset().expect("non-canonical row is a permutation of the canonical row");
	let _ = handle_canonical(canonical);
}

// -- Test 9a: same as t09 but invokes CoproductSubsetter explicitly --

#[test]
fn t09a_subsetter_via_explicit_trait_call() {
	// Same handler, same hand-written row, but the call to .subset()
	// is rewritten as a fully-qualified trait method call. This is
	// equivalent to t09 at runtime; the value is documentary, showing
	// the trait dependency in the source.
	type Source = Coproduct<C, Coproduct<A, Coproduct<B, CNil>>>;
	let hand_written: Source = Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(B("hello"))));
	let canonical: effects![A, B, C] =
		<Source as CoproductSubsetter<effects![A, B, C], _>>::subset(hand_written)
			.expect("permutation");
	let _ = handle_canonical(canonical);
}

// -- Test 10: handler accepts row produced by either macro or hand-written canonical form --

#[test]
fn t10_handler_accepts_macro_output_directly() {
	// Macro-emitted row goes straight into the handler with no
	// permutation mediation needed (workaround 1's ergonomic win:
	// composition is identity at the type level).
	let row: effects![A, B, C] = Coproduct::Inl(A(42));
	let _ = handle_canonical(row);
}

// -- Test 11: scaling - 5-effect row across two random orderings --

#[test]
fn t11_five_effect_canonicalisation() {
	struct D;
	struct E;
	type Order1 = effects![C, A, E, B, D];
	type Order2 = effects![E, D, A, C, B];
	type Order3 = effects![A, B, C, D, E];
	assert_type_eq_all!(Order1, Order2);
	assert_type_eq_all!(Order2, Order3);
}

// -- Test 12: scaling - 7-effect row to stress trait inference --

#[test]
fn t12_seven_effect_canonicalisation() {
	struct E1;
	struct E2;
	struct E3;
	struct E4;
	struct E5;
	struct E6;
	struct E7;
	type Effs = effects![E5, E1, E7, E3, E6, E2, E4];
	type Canonical = effects![E1, E2, E3, E4, E5, E6, E7];
	assert_type_eq_all!(Effs, Canonical);
}

// -- Test 13: subsetter handles a 5-effect permutation --

#[test]
fn t13_subsetter_handles_five_effect_permutation() {
	struct D;
	struct E;
	type Canonical = effects![A, B, C, D, E];

	// Hand-write a coproduct in reverse order.
	type Reverse = Coproduct<E, Coproduct<D, Coproduct<C, Coproduct<B, Coproduct<A, CNil>>>>>;

	let hand: Reverse = Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(B("hi")))));
	let _: Canonical = hand.subset().expect("5-effect permutation");
}

// -- tstr_crates integration demos --
//
// The tests below answer a sub-question of the overall feasibility
// study: what does tstr_crates buy us on top of the proc-macro
// hybrid? Three concrete demos:
//
//   t14: TStr<...> provides stable type-level identity. Two TS!("name")
//        invocations in different module contexts produce the same
//        type, so an effect's canonical name does not depend on its
//        import path.
//
//   t15: tstr::cmp computes string ordering at compile time, returning
//        a runtime core::cmp::Ordering value. This is the building
//        block for ANY name-driven canonicalisation.
//
//   t16: An effect can carry its canonical TStr name as an associated
//        type via a small custom trait. This is the data shape that
//        a richer macro (or a nightly type-level sort) would consume.
//
// What tstr_crates does NOT give us on stable: the Ordering value from
// tstr::cmp cannot drive type-level dispatch (Ordering is not a valid
// const generic kind without nightly's adt_const_params), so it
// cannot directly canonicalise hand-written coproducts at the type
// level. Lifting Ordering into trait-dispatch terms requires nightly
// `feature(adt_const_params)` plus `feature(generic_const_exprs)`.
// This is the gap noted in port-plan section 4.1's discussion of
// approach 4 (adt_const_params with strings) versus approach 9
// (type-level hashing with type-level result) in the type-level-
// sorting research README.

use tstr::{
	IsTStr,
	TS,
	ts,
};

#[test]
fn t14_tstr_provides_stable_type_level_identity() {
	// Same TStr literal in different module contexts: same type.
	type ReaderName = TS!("reader");

	mod nested {
		pub type ReaderName = ::tstr::TS!("reader");
	}

	assert_type_eq_all!(ReaderName, nested::ReaderName);

	// Different literals: different types (ensured by the type
	// checker; if these were the same, t15's cmp wouldn't return
	// Less / Greater).
	type StateName = TS!("state");
	let _r: ReaderName = ts!("reader");
	let _s: StateName = ts!("state");
}

#[test]
fn t15_tstr_cmp_computes_ordering_at_compile_time() {
	use core::cmp::Ordering;
	// All three branches evaluated in const context, then asserted
	// at runtime.
	const READER_VS_STATE: Ordering = tstr::cmp(ts!("reader"), ts!("state"));
	const STATE_VS_READER: Ordering = tstr::cmp(ts!("state"), ts!("reader"));
	const READER_VS_READER: Ordering = tstr::cmp(ts!("reader"), ts!("reader"));

	assert_eq!(READER_VS_STATE, Ordering::Less);
	assert_eq!(STATE_VS_READER, Ordering::Greater);
	assert_eq!(READER_VS_READER, Ordering::Equal);
}

#[test]
fn t16_effects_can_carry_canonical_tstr_names() {
	use core::cmp::Ordering;

	// A small trait that lets each effect declare its canonical name
	// as both a TYPE (Self::Name) and a const VALUE (Self::NAME). The
	// associated const lets us use the name in const context, which is
	// what makes the comparison runs at compile time.
	trait NamedEffect {
		type Name: IsTStr + Copy;
		const NAME: Self::Name;
	}

	impl NamedEffect for A {
		type Name = TS!("a");

		const NAME: Self::Name = ts!("a");
	}
	impl NamedEffect for B {
		type Name = TS!("b");

		const NAME: Self::Name = ts!("b");
	}
	impl NamedEffect for C {
		type Name = TS!("c");

		const NAME: Self::Name = ts!("c");
	}

	// Canonical names are extractable as types.
	type ANameType = <A as NamedEffect>::Name;
	type BNameType = <B as NamedEffect>::Name;
	assert_type_eq_all!(ANameType, TS!("a"));
	assert_type_eq_all!(BNameType, TS!("b"));

	// And as compile-time-comparable values via tstr::cmp, called in
	// const context using the associated NAME constants.
	const A_VS_B: Ordering = tstr::cmp(<A as NamedEffect>::NAME, <B as NamedEffect>::NAME);
	const B_VS_C: Ordering = tstr::cmp(<B as NamedEffect>::NAME, <C as NamedEffect>::NAME);
	assert_eq!(A_VS_B, Ordering::Less);
	assert_eq!(B_VS_C, Ordering::Less);

	// What this enables on stable Rust:
	//   * A proc-macro variant (effects_by_name!) could read explicit
	//     TStr names from user input and sort by them, giving
	//     canonicalisation independent of import paths.
	//   * Effect-row APIs can take a NamedEffect bound and use the
	//     canonical name in error messages or runtime tracing.
	//
	// What this does NOT enable on stable Rust:
	//   * The Ordering returned by tstr::cmp cannot parameterise types
	//     (Ordering is not a stable const generic kind), so it cannot
	//     drive a recursive type-level sort over a Coproduct. To
	//     auto-canonicalise hand-written coproducts at the type level,
	//     nightly's adt_const_params plus generic_const_exprs are
	//     required. The proc-macro (workaround 1) and CoproductSubsetter
	//     (workaround 3) remain the stable-Rust answer.
}
