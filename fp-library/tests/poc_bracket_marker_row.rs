#![cfg(feature = "effects")]
// POC: marker-struct workaround for user-facing recursive type alias
// rejection on Bracket-containing scoped rows.
//
// Question being answered: with `Bracket<'a, P, Sub, A, B>` and
// `BracketBrand<P, Sub, A, B>` carrying the substrate brand `Sub`
// explicitly, can a user define a scoped row containing
// `BoxBracketBrand` via a marker struct that breaks the type-alias
// cyclicity? The natural
// formulation `type ScopedRow = CoproductBrand<BoxBracketBrand<
// BoxBrand, NodeBrand<R, ScopedRow>, A, B>, CNilBrand>` is rejected
// by Rust with `error[E0391]: cycle detected when expanding type
// alias`. The marker-struct workaround moves the recursion into an
// `impl_kind!` body (which Rust accepts for trait impls on structs).
//
// Five risks the POC must validate:
//
//   R1: Does `impl_kind!` accept a recursive type expression in the
//       impl body when the marker struct references itself
//       transitively through `NodeBrand<R, Marker>`?
//   R2: Do `WrapDrop` / `Functor` / `SendFunctor` delegating impls
//       compile when the recursion goes through the marker?
//   R3: Does typechecking of `Run<R, Marker, X>` succeed when the
//       marker is a fresh struct (not a `CoproductBrand` directly)?
//   R4: At runtime, do dispatch operations (peel / resume) thread
//       correctly through the marker delegation?
//   R5: Does the `Member` trait check satisfy through the marker's
//       `Of` projection (frunk's `CoprodInjector` lookup must succeed
//       against the marker's projection, not the underlying
//       `CoproductBrand`)?
//
// If all five pass, the POC confirms that the marker-struct pattern
// is a viable workaround for recursive Bracket scoped rows and that
// a row-generation macro can emit the boilerplate automatically.
//
// Validation steps in this file:
//   1. Define `MarkerRow` (a unit struct that breaks the cycle).
//   2. Implement `Kind` via `impl_kind!`, body referencing
//      `<UnderlyingRow as Kind>::Of<'a, A>` where `UnderlyingRow`
//      itself contains `BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand,
//      MarkerRow>, i32, i32>` recursively. (R1 + R2 surface during
//      compile.)
//   3. Implement `WrapDrop` / `Functor` / `SendFunctor` on
//      `MarkerRow` by delegating to the underlying `CoproductBrand`.
//      (R2 surfaces during compile.)
//   4. Construct a `BoxBracket` cell directly and inject it via
//      `Member::inject` into the marker row's `Of` projection (R5).
//   5. Wrap the result in `Node::Scoped` and `Free::wrap`, lifting to
//      `Run<CNilBrand, MarkerRow, (i32, i32)>`. (R3 + R4 surface.)
//   6. Call `peel()` and assert the variant tag (R4).
#![allow(dead_code)]
#![expect(clippy::panic, reason = "POC tests use panicking operations for brevity and clarity.")]

use fp_library::{
	Apply,
	brands::{
		BoxBracketBrand,
		BoxBrand,
		CNilBrand,
		CoproductBrand,
		NodeBrand,
	},
	classes::{
		Functor,
		Pointer,
		SendFunctor,
		ToDynFnOnce,
		WrapDrop,
	},
	impl_kind,
	kinds::*,
	types::{
		Free,
		effects::{
			bracket::BoxBracket,
			coproduct::Coproduct,
			member::Member,
			node::Node,
			run::Run,
		},
	},
};

// -- Marker-row struct -----------------------------------------------

// A zero-sized marker struct that "is" the scoped row. The actual
// row body (a `CoproductBrand` of `BoxBracketBrand` plus `CNilBrand`)
// references this marker recursively through `NodeBrand<CNilBrand,
// MarkerRow>` in the brand's `Sub` parameter. Rust forbids recursive
// *type aliases* but accepts recursive references inside trait impl
// bodies (the alias rule applies only to top-level `type X = ...`
// declarations).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MarkerRow;

// -- Kind impl: the recursive part ----------------------------------
//
// `MarkerRow` projects through to the underlying `CoproductBrand`
// row body via `<UnderlyingRow as Kind>::Of<'a, A>`. The recursion
// in the body (`NodeBrand<CNilBrand, MarkerRow>` references
// `MarkerRow`) is what `impl_kind!` must accept (R1).

impl_kind! {
	impl for MarkerRow {
		type Of<'a, A: 'a>: 'a = Apply!(<CoproductBrand<
			BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, MarkerRow>, i32, i32>,
			CNilBrand,
		> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}

// -- WrapDrop / Functor / SendFunctor: delegating impls (R2) --------
//
// `MarkerRow`'s value-level `Of<'a, X>` is the underlying row's
// `Of<'a, X>`, so each method delegates directly to the underlying
// brand's impl.

type UnderlyingRow =
	CoproductBrand<BoxBracketBrand<BoxBrand, NodeBrand<CNilBrand, MarkerRow>, i32, i32>, CNilBrand>;

impl WrapDrop for MarkerRow {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<UnderlyingRow as WrapDrop>::drop(fa)
	}
}

impl Functor for MarkerRow {
	fn map<'a, A: 'a, B: 'a>(
		func: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<UnderlyingRow as Functor>::map(func, fa)
	}
}

impl SendFunctor for MarkerRow {
	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		func: impl Fn(A) -> B + Send + Sync + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<UnderlyingRow as SendFunctor>::send_map(func, fa)
	}
}

// -- POC test: end-to-end construction + peel (R3 / R4 / R5) --------

#[test]
fn marker_row_supports_bracket_construction_and_peel() {
	type ProgramResult = (i32, i32);

	// Construct the BoxBracket cell directly. The closures' return
	// types reference Sub = NodeBrand<CNilBrand, MarkerRow> (which is
	// what `Run<CNilBrand, MarkerRow, _>::into_free()` produces).
	let acquire_program: Free<NodeBrand<CNilBrand, MarkerRow>, i32> = Free::pure(7);
	let bracket: BoxBracket<
		'static,
		BoxBrand,
		NodeBrand<CNilBrand, MarkerRow>,
		i32,
		i32,
	> = BoxBracket::Bracket {
		acquire: <BoxBrand as ToDynFnOnce>::new(move |_: ()| acquire_program),
		body: <BoxBrand as ToDynFnOnce>::new(
			move |a: <BoxBrand as Pointer>::Of<'static, i32>| -> Free<
				NodeBrand<CNilBrand, MarkerRow>,
				ProgramResult,
			> { Free::pure((*a, 42)) },
		),
		release: <BoxBrand as ToDynFnOnce>::new(
			move |_a: <BoxBrand as Pointer>::Of<'static, i32>| -> Free<
				NodeBrand<CNilBrand, MarkerRow>,
				(),
			> { Free::pure(()) },
		),
	};

	// R5: inject the BoxBracket cell into the marker row's Of
	// projection. Frunk's CoprodInjector lookup must succeed against
	// the marker's `Of<'static, Free<NodeBrand<CNilBrand, MarkerRow>,
	// ProgramResult>>` projection (which transparently equals the
	// underlying row's projection).
	let layer = <Apply!(<MarkerRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'static,
		Free<NodeBrand<CNilBrand, MarkerRow>, ProgramResult>,
	>) as Member<
		BoxBracket<'static, BoxBrand, NodeBrand<CNilBrand, MarkerRow>, i32, i32>,
		_,
	>>::inject(bracket);

	// R3 + R4: wrap as Node::Scoped, then Free::wrap, then Run.
	// Free::wrap takes `<F as Kind>::Of<'a, Free<F, A>>` (a layer with
	// nested Free continuations), so the Node's inner A param is the
	// nested Free, not the program's scalar result type directly.
	let node: Node<
		'static,
		CNilBrand,
		MarkerRow,
		Free<NodeBrand<CNilBrand, MarkerRow>, ProgramResult>,
	> = Node::Scoped(layer);
	let free: Free<NodeBrand<CNilBrand, MarkerRow>, ProgramResult> = Free::wrap(node);
	let prog: Run<CNilBrand, MarkerRow, ProgramResult> = Run::from_free(free);

	// R4: peel returns Err carrying a `Node::Scoped(...)` projection,
	// confirming the program is suspended at the scoped Bracket layer
	// and that dispatch operations thread correctly through the marker.
	match prog.peel() {
		Err(Node::Scoped(scoped_layer)) => match scoped_layer {
			Coproduct::Inl(BoxBracket::Bracket {
				..
			}) => {
				// Variant tag matches; R5 dispatch was correct.
			}
			Coproduct::Inr(_) => panic!("expected head Inl (BoxBracket)"),
		},
		Err(Node::First(_)) => panic!("expected Node::Scoped variant"),
		Ok(_) => panic!("expected program suspended at scoped layer"),
	}
}
