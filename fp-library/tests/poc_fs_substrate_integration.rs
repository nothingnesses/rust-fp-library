//! POC-11 (foundation sweep): substrate-identity integration spike.
//!
//! Resolves the two substrate-identity questions left open for the production
//! rebuild (remediation item 4 step 2):
//!
//! 1. Erasure axis, C-versus-B. Can ONE substrate provide both the erased
//!    form's O(1) `CatList` `bind` (which needs `dyn Any` value erasure, hence
//!    `A: 'static`) and the Explicit form's non-`'static` payloads (which forbid
//!    that erasure)? Part 1 shows no: the erased O(1) path forces `'static`, and
//!    stable Rust cannot make one type select it only when `A: 'static`. So the
//!    erasure axis is two `Store`-parameterised forms (B, six wrappers to two),
//!    not one (C2). Approach A (drop non-`'static`) was already eliminated by
//!    the catalog audit.
//! 2. Store-parameterisation over the real row. Parts 2 and 3 put the real
//!    `Coyoneda`-wrapped `CoproductBrand` row on both the erased (`Free`) and
//!    concrete (`FreeExplicit`) public substrates, the latter carrying a
//!    non-`'static` borrowed payload. Part 4 builds a `ClosureStorage`-
//!    parameterised concrete spine over that real row, showing the pointer-axis
//!    collapse (POC-8) holds on the real encoding and recording where the
//!    concrete form's `bind` constrains the `Store` (it needs a cloneable `Fn`,
//!    so Box's `FnOnce` cannot serve it).
//!
//! Throwaway spike code (charter S1/S2).

#![cfg(feature = "effects")]
#![allow(
	dead_code,
	reason = "the spike defines both axes' machinery; not every item is exercised by a test"
)]

use {
	fp_library::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
			RcBrand,
		},
		classes::Functor,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			FreeExplicit,
			effects::coproduct::Coproduct,
		},
	},
	std::{
		any::Any,
		rc::Rc,
	},
};

// =====================================================================
// Part 1: Erasure axis, the C-versus-B decision.
//
// The erased O(1) representation stores values and continuations behind
// `dyn Any` (the production `Free` uses `TypeErasedValue = Box<dyn Any>` and a
// `CatList` of `Box<dyn FnOnce(Box<dyn Any>) -> ...>`). Coercing `Box<A>` to
// `Box<dyn Any>` requires `A: 'static`, so the O(1) path is unavailable for
// non-`'static` `A`. `erase_value` compiles only with the `'static` bound:
// `Box<dyn Any>` is `Box<dyn Any + 'static>`, so `Box::new(a)` without the bound
// is rejected (E0310, "`A` may not live long enough"), the load-bearing fact.
// Stable Rust cannot make one substrate select the
// erased queue only when `A: 'static` (no specialisation, no negative bounds),
// so the erased (`'static`, O(1)) and concrete (non-`'static`, O(N)) substrates
// are necessarily distinct types: the erasure axis is B, and C2 is ruled out.
// =====================================================================

fn erase_value<A: 'static>(a: A) -> Box<dyn Any> {
	Box::new(a)
}

#[test]
fn erased_o1_path_requires_static() {
	let erased = erase_value(42_i32);
	assert_eq!(erased.downcast_ref::<i32>(), Some(&42));
}

// =====================================================================
// Effect + real row used by Parts 2, 3, 4.
// =====================================================================

// A first-order effect in the `OutF` style: a payload plus the continuation
// slot `A`. `Coyoneda`-wrapping supplies `Functor`/`WrapDrop` for the row cell.
struct AskBrand;
enum AskF<A> {
	Ask(i32, A),
}
impl_kind! {
	impl for AskBrand {
		type Of<'a, A: 'a>: 'a = AskF<A>;
	}
}

// The real row: one `Coyoneda`-wrapped effect terminated by `CNilBrand`.
type Row = CoproductBrand<CoyonedaBrand<AskBrand>, CNilBrand>;

// =====================================================================
// Part 2: the real `Coyoneda` coproduct row on the erased public `Free`.
// =====================================================================

#[test]
fn real_row_composes_on_erased_free() {
	let coyo: Coyoneda<'static, AskBrand, i32> = Coyoneda::lift(AskF::Ask(7, 0));
	let node: Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, i32>) =
		Coproduct::inject(coyo);
	let program: Free<Row, i32> = Free::lift_f(node);
	// Composing the real row on the erased Box-spine `Free` succeeds (the
	// pointer-axis Box case reusing the existing `Free`); the program suspends at
	// the `Ask` effect, so `resume` yields the row node (`Err`).
	assert!(program.resume().is_err());
}

// =====================================================================
// Part 3: the real row on the concrete public `FreeExplicit`, carrying a
// non-`'static` borrowed payload (the capability the catalog audit confirmed is
// needed). Uses the shipped concrete substrate rather than rebuilding it.
// =====================================================================

#[test]
fn concrete_substrate_carries_borrowed_payload_through_bind() {
	let owner = String::from("borrowed-non-static");
	let borrowed: &str = owner.as_str();

	// The concrete substrate carries a value borrowed from a local scope through
	// `bind` with no `'static` bound (the capability Part 1's erased path cannot
	// have). `IdentityBrand` is used as the base functor only so `evaluate` is
	// available (it is `Extract`); the non-`'static` property is the point. The
	// real `Coyoneda` row on the concrete substrate is already shipped as
	// `RunExplicit` (= `FreeExplicit` over `NodeBrand`) and exercised with a
	// borrowed payload by `run_span`'s `..._preserves_borrowed_payload`, so it is
	// cited rather than rebuilt; Part 2 shows the same row on the erased `Free`.
	let program: FreeExplicit<'_, IdentityBrand, &str> = FreeExplicit::pure(borrowed);
	let mapped = program.bind(move |s: &str| FreeExplicit::pure(s.len()));
	assert_eq!(mapped.evaluate(), "borrowed-non-static".len());
}

// =====================================================================
// Part 4: a `ClosureStorage`-parameterised concrete spine over the real row.
//
// Mirrors `FreeExplicit` (concrete, non-`'static`) but stores the `bind`
// continuation via a `Store` parameter, on the real `Coyoneda` row. Records the
// load-bearing constraint: the recursive `bind` clones the continuation into the
// nested closure, so `Store::Stored` must be `Clone`. Box's `FnOnce` is not
// `Clone`, so the concrete form's pointer axis is {Rc, Arc} (a Box-flavoured
// concrete substrate reuses `Rc` internally, exactly as the shipped
// `FreeExplicit` does), whereas the erased form's pointer axis is the full
// {Box, Rc, Arc} (POC-8b). This is the one place the two forms' Store axes
// differ.
// =====================================================================

trait ClosureStorage: 'static {
	type Stored<'a, I: 'a, O: 'a>: Clone + 'a;
	fn store<'a, I: 'a, O: 'a>(f: impl Fn(I) -> O + 'a) -> Self::Stored<'a, I, O>;
	fn call<'a, I: 'a, O: 'a>(
		s: &Self::Stored<'a, I, O>,
		i: I,
	) -> O;
}
impl ClosureStorage for RcBrand {
	type Stored<'a, I: 'a, O: 'a> = Rc<dyn Fn(I) -> O + 'a>;

	fn store<'a, I: 'a, O: 'a>(f: impl Fn(I) -> O + 'a) -> Self::Stored<'a, I, O> {
		let stored: Rc<dyn Fn(I) -> O + 'a> = Rc::new(f);
		stored
	}

	fn call<'a, I: 'a, O: 'a>(
		s: &Self::Stored<'a, I, O>,
		i: I,
	) -> O {
		(**s)(i)
	}
}

enum SpineView<'a, F: Functor + 'a, A: 'a> {
	Pure(A),
	Wrap(Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Box<Spine<'a, F, A>>>)),
}
struct Spine<'a, F: Functor + 'a, A: 'a> {
	view: SpineView<'a, F, A>,
}
impl<'a, F: Functor + 'a, A: 'a> Spine<'a, F, A> {
	fn pure(a: A) -> Self {
		Spine {
			view: SpineView::Pure(a),
		}
	}

	// Store-parameterised recursive bind. The continuation `k` is stored via
	// `Store` and cloned into the per-layer closure, exactly the pattern
	// `FreeExplicit::bind_boxed` uses with a hardcoded `Rc<dyn Fn>`.
	fn bind<Store: ClosureStorage, B: 'a>(
		self,
		k: Store::Stored<'a, A, Spine<'a, F, B>>,
	) -> Spine<'a, F, B> {
		match self.view {
			SpineView::Pure(a) => Store::call(&k, a),
			SpineView::Wrap(layer) => {
				let mapped = F::map(
					move |inner: Box<Spine<'a, F, A>>| -> Box<Spine<'a, F, B>> {
						Box::new((*inner).bind::<Store, B>(k.clone()))
					},
					layer,
				);
				Spine {
					view: SpineView::Wrap(mapped),
				}
			}
		}
	}
}

// Inject one real `Coyoneda`-wrapped `Ask` cell as a `Wrap` over the row.
fn lift_ask(payload: i32) -> Spine<'static, Row, i32> {
	let inner: Box<Spine<'static, Row, i32>> = Box::new(Spine::pure(payload));
	let coyo: Coyoneda<'static, AskBrand, Box<Spine<'static, Row, i32>>> =
		Coyoneda::lift(AskF::Ask(payload, inner));
	let node: Apply!(
		<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Box<Spine<'static, Row, i32>>>
	) = Coproduct::inject(coyo);
	Spine {
		view: SpineView::Wrap(node),
	}
}

#[test]
fn concrete_store_param_bind_over_real_row_rc() {
	// Pure path: the Rc-stored continuation is applied.
	let p: Spine<'static, Row, i32> = Spine::pure(3);
	let bound = p.bind::<RcBrand, i32>(RcBrand::store(|x: i32| Spine::pure(x + 1)));
	assert!(matches!(bound.view, SpineView::Pure(4)));

	// Wrap path: bind threads (cloning) the Rc continuation through the real
	// `Coyoneda` row layer, leaving a `Wrap` whose inner program is bound.
	let w = lift_ask(10);
	let bound_w = w.bind::<RcBrand, i32>(RcBrand::store(|x: i32| Spine::pure(x * 2)));
	assert!(matches!(bound_w.view, SpineView::Wrap(_)));
}
