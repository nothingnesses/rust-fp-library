// Integration tests for the `handlers!` macro and the `nt()` builder
// fallback (Phase 3 step 1).
//
// Covers:
// - Empty input -> HandlersNil.
// - Single-entry and multi-entry expansions.
// - Canonical-ordering property: input order does not affect the
//   resulting type, mirroring the `effects!` lexical sort so
//   handler-list cells align cell-for-cell with row brand cells.
// - Brand identity is pinned in the emitted Handler<Brand, _> shape.
// - The emitted value is constructible at runtime and the closures
//   stored at each cell are invocable.
// - Equivalence between macro output and builder output for the same
//   logical handler set (the macro sorts; the builder uses prepend
//   semantics so the user must call `.on()` in reverse-lexical order
//   to match the macro's canonical shape).

use {
	core::marker::PhantomData,
	fp_library::{
		handlers,
		types::effects::handlers::{
			Handler,
			HandlersCons,
			HandlersNil,
			nt,
		},
	},
};

// Compile-time type-equality assertion. Compiles iff `T == U`.
fn assert_type_eq<T>(
	_: PhantomData<T>,
	_: PhantomData<T>,
) {
}

// Local test brands. Names chosen to make the lexical sort visible.
struct AlphaBrand;
struct BetaBrand;
struct GammaBrand;

#[test]
fn handlers_empty_yields_nil() {
	let h = handlers! {};
	let _: HandlersNil = h;
}

#[test]
fn handlers_single_entry_shape() {
	let h = handlers! {
		AlphaBrand: |op: i32| op + 1,
	};
	// Macro emits a single cons cell terminated in HandlersNil.
	type Expected<F> = HandlersCons<Handler<AlphaBrand, F>, HandlersNil>;
	fn check<F>(_: &Expected<F>) {}
	check(&h);
	// The handler closure runs end-to-end.
	assert_eq!((h.head.run)(7), 8);
}

#[test]
fn handlers_two_entries_canonical_ordering() {
	// Lexical sort: "AlphaBrand" < "BetaBrand", so AlphaBrand is the
	// head and BetaBrand is the next cell, regardless of input order.
	let h1 = handlers! {
		AlphaBrand: |x: i32| x,
		BetaBrand: |x: i32| x * 2,
	};
	let h2 = handlers! {
		BetaBrand: |x: i32| x * 2,
		AlphaBrand: |x: i32| x,
	};
	// Both must have the same type-level shape.
	type Shape<FA, FB> =
		HandlersCons<Handler<AlphaBrand, FA>, HandlersCons<Handler<BetaBrand, FB>, HandlersNil>>;
	fn _check<FA, FB>(_: &Shape<FA, FB>) {}
	_check(&h1);
	_check(&h2);
	// Closures dispatch to the lexically-sorted cells in both
	// orderings: head -> AlphaBrand handler, tail.head -> BetaBrand handler.
	assert_eq!((h1.head.run)(3), 3);
	assert_eq!((h1.tail.head.run)(3), 6);
	assert_eq!((h2.head.run)(3), 3);
	assert_eq!((h2.tail.head.run)(3), 6);
}

#[test]
fn handlers_three_entries_canonical_ordering() {
	let h = handlers! {
		GammaBrand: |x: i32| x + 100,
		AlphaBrand: |x: i32| x + 1,
		BetaBrand: |x: i32| x + 10,
	};
	// Lexical sort: Alpha < Beta < Gamma.
	assert_eq!((h.head.run)(0), 1); // Alpha
	assert_eq!((h.tail.head.run)(0), 10); // Beta
	assert_eq!((h.tail.tail.head.run)(0), 100); // Gamma
}

#[test]
fn handlers_trailing_comma_accepted() {
	let h = handlers! {
		AlphaBrand: |x: i32| x,
	};
	let _: HandlersCons<Handler<AlphaBrand, _>, HandlersNil> = h;
}

#[test]
fn handlers_brand_pinned_in_handler_type() {
	// The brand identity flows into the Handler's first type parameter,
	// which is what the Phase 3 step 2 interpreter will use to match
	// each handler against the row's head brand.
	let h = handlers! {
		AlphaBrand: |x: i32| x,
	};
	assert_type_eq::<Handler<AlphaBrand, _>>(PhantomData::<Handler<AlphaBrand, _>>, {
		fn brand_of<E, F>(_: &Handler<E, F>) -> PhantomData<Handler<E, F>> {
			PhantomData
		}
		brand_of(&h.head)
	});
}

// -- nt() builder fallback --

#[test]
fn nt_returns_handlers_nil() {
	let _: HandlersNil = nt();
}

#[test]
fn nt_on_single_entry() {
	let h = nt().on::<AlphaBrand, _>(|x: i32| x + 1);
	let _: HandlersCons<Handler<AlphaBrand, _>, HandlersNil> = h;
	assert_eq!((h.head.run)(0), 1);
}

#[test]
fn nt_on_chain_uses_prepend_semantics() {
	// Builder uses prepend semantics: the last `.on()` is at the head.
	// Calling `.on::<Alpha>(...).on::<Beta>(...)` produces
	// HandlersCons<Beta, HandlersCons<Alpha, HandlersNil>>. To match the
	// macro's lexical-canonical shape (Alpha at head), call in
	// reverse-lexical order: `.on::<Beta>(...).on::<Alpha>(...)`.
	let h = nt().on::<BetaBrand, _>(|x: i32| x * 2).on::<AlphaBrand, _>(|x: i32| x);
	type Shape<FA, FB> =
		HandlersCons<Handler<AlphaBrand, FA>, HandlersCons<Handler<BetaBrand, FB>, HandlersNil>>;
	fn _check<FA, FB>(_: &Shape<FA, FB>) {}
	_check(&h);
	assert_eq!((h.head.run)(5), 5);
	assert_eq!((h.tail.head.run)(5), 10);
}

type AlphaBetaShape<FA, FB> =
	HandlersCons<Handler<AlphaBrand, FA>, HandlersCons<Handler<BetaBrand, FB>, HandlersNil>>;

#[test]
fn nt_builder_matches_macro_shape_for_aligned_input() {
	// Reverse-lexical-order builder calls produce the same type-level
	// shape as the macro's canonical output. Compare both lists by
	// applying their handlers to the same input and checking the
	// values match cell-for-cell.
	let from_macro = handlers! {
		AlphaBrand: |x: i32| x + 1,
		BetaBrand: |x: i32| x + 10,
	};
	let from_builder = nt().on::<BetaBrand, _>(|x: i32| x + 10).on::<AlphaBrand, _>(|x: i32| x + 1);

	// Same shape. (Closure types differ but the cell structure does not.)
	fn _shape_check<FA, FB>(_: &AlphaBetaShape<FA, FB>) {}
	_shape_check(&from_macro);
	_shape_check(&from_builder);

	// Same observable behaviour.
	assert_eq!((from_macro.head.run)(0), (from_builder.head.run)(0));
	assert_eq!((from_macro.tail.head.run)(0), (from_builder.tail.head.run)(0));
}
