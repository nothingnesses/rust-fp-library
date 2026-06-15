//! POC-6 (foundation sweep, Tier D): row widening (`expand`) over the unified
//! row, including a higher-order cell that carries a sub-program, via a
//! row-parameterised cell over nominal recursive rows (Approach (B-nominal)).
//!
//! Charter question: can `expand` widen a program containing a higher-order
//! `listen` cell from one row into a larger row, preserving semantics, over the
//! public `Free` stepping surface (no boundary frames, no raw-step path)?
//!
//! Approach: the `listen` cell is row-parameterised, `ListenBrand<R, RAction>`
//! with `Of<'a, Next> = ListenCell<'a, R, RAction, Next>` storing the action
//! `Free<R, RAction>`, so the row appears in the cell type. The rows `R1` and
//! `R2` are nominal `Kind`s (marker structs whose `Of`/`WrapDrop`/`Functor`
//! delegate to a coproduct that names the row itself in the `ListenBrand<Self,
//! _>` position). Encoding the row nominally breaks the type-alias cycle a
//! self-referential `type Row = ...ListenBrand<Row, _>...` would hit, the same
//! way a recursive struct is allowed because `Free<Self, _>` is nominal and
//! boxed. `expand::<R1, R2>` is a hand-written traversal over the public
//! `resume`/`lift_f`/`bind`/`pure` surface; at a `listen` cell it rebuilds
//! `ListenCell<R2>` from `ListenCell<R1>` by recursively widening both the
//! action and the continuation; the first-order coproduct widening is manual
//! `Inl`/`Inr` re-injection.
//!
//! Target property (setup step S5): widening
//! `listen(tell("first") >> tell("second") >> pure(40))`, then
//! `(value, observed) -> (value + 2, observed)`, from `R1` into the larger `R2`
//! preserves the result `(42, "firstsecond")` and the propagated log
//! "firstsecond" (the boundary-split suite's listen value). Throwaway spike
//! code.

#![cfg(feature = "effects")]
#![allow(dead_code, reason = "OtherBrand widens the target row but is unused by the program")]

use {
	fp_library::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		classes::{
			Functor,
			WrapDrop,
		},
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::Coproduct,
		},
	},
	std::marker::PhantomData,
};

// First-order Writer effect over a String log (row-agnostic).
struct WriterBrand;
enum WriterF<'a, A> {
	Tell(String, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! {
	impl for WriterBrand {
		type Of<'a, A: 'a>: 'a = WriterF<'a, A>;
	}
}
impl Functor for WriterBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			WriterF::Tell(w, k) => WriterF::Tell(w, Box::new(move |u| f(k(u)))),
		}
	}
}

// An extra first-order effect that exists only to make the target row strictly
// larger. It is never used by the program, so it needs only a `Kind` projection.
struct OtherBrand;
struct OtherF<A>(PhantomData<A>);
impl_kind! {
	impl for OtherBrand {
		type Of<'a, A: 'a>: 'a = OtherF<A>;
	}
}

// Higher-order Listen effect, row-parameterised: the cell stores the action
// sub-program `Free<R, RAction>` (so the row `R` appears in the cell type) and a
// continuation consuming the result-shape-changed `(RAction, String)`.
struct ListenBrand<R, RAction>(PhantomData<(R, RAction)>);
struct ListenCell<'a, R: WrapDrop + 'static, RAction: 'static, Next> {
	action: Free<R, RAction>,
	k: Box<dyn FnOnce((RAction, String)) -> Next + 'a>,
}
impl_kind! {
	impl<R: WrapDrop + 'static, RAction: 'static> for ListenBrand<R, RAction> {
		type Of<'a, Next: 'a>: 'a = ListenCell<'a, R, RAction, Next>;
	}
}
impl<R: WrapDrop + 'static, RAction: 'static> Functor for ListenBrand<R, RAction> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		// Post-compose `f` onto the hole; `RAction` and `R` are concrete, so this
		// is object-safe.
		let ListenCell {
			action,
			k,
		} = fa;
		ListenCell {
			action,
			k: Box::new(move |pair| f(k(pair))),
		}
	}
}

// The source row R1 = { Writer, Listen<R1> }, encoded nominally so it can name
// itself in the `ListenBrand<R1, _>` position without a type-alias cycle.
struct R1;
type R1Coproduct = CoproductBrand<
	CoyonedaBrand<WriterBrand>,
	CoproductBrand<CoyonedaBrand<ListenBrand<R1, i32>>, CNilBrand>,
>;
impl_kind! {
	impl for R1 {
		type Of<'a, A: 'a>: 'a = Apply!(<R1Coproduct as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}
impl WrapDrop for R1 {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<R1Coproduct as WrapDrop>::drop(fa)
	}
}
impl Functor for R1 {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<R1Coproduct as Functor>::map(f, fa)
	}
}

// The target row R2 = { Writer, Listen<R2>, Other }, strictly larger than R1.
struct R2;
type R2Coproduct = CoproductBrand<
	CoyonedaBrand<WriterBrand>,
	CoproductBrand<
		CoyonedaBrand<ListenBrand<R2, i32>>,
		CoproductBrand<CoyonedaBrand<OtherBrand>, CNilBrand>,
	>,
>;
impl_kind! {
	impl for R2 {
		type Of<'a, A: 'a>: 'a = Apply!(<R2Coproduct as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}
}
impl WrapDrop for R2 {
	fn drop<'a, X: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
	) -> Option<X> {
		<R2Coproduct as WrapDrop>::drop(fa)
	}
}
impl Functor for R2 {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		<R2Coproduct as Functor>::map(f, fa)
	}
}

type Node1<X> = Apply!(<R1 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, X>);
type Node2<X> = Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, X>);

// Constructors over R1.
fn tell1(w: String) -> Free<R1, ()> {
	let coyo: Coyoneda<'static, WriterBrand, ()> =
		Coyoneda::<WriterBrand, _>::lift(WriterF::Tell(w, Box::new(|u| u)));
	let node: Node1<()> = Coproduct::Inl(coyo);
	Free::lift_f(node)
}
fn listen1(action: Free<R1, i32>) -> Free<R1, (i32, String)> {
	let cell: ListenCell<'static, R1, i32, (i32, String)> = ListenCell {
		action,
		k: Box::new(|pair| pair),
	};
	let coyo: Coyoneda<'static, ListenBrand<R1, i32>, (i32, String)> =
		Coyoneda::<ListenBrand<R1, i32>, _>::lift(cell);
	let node: Node1<(i32, String)> = Coproduct::Inr(Coproduct::Inl(coyo));
	Free::lift_f(node)
}

// Constructors over R2 (used by `expand` to rebuild widened layers).
fn tell2(w: String) -> Free<R2, ()> {
	let coyo: Coyoneda<'static, WriterBrand, ()> =
		Coyoneda::<WriterBrand, _>::lift(WriterF::Tell(w, Box::new(|u| u)));
	let node: Node2<()> = Coproduct::Inl(coyo);
	Free::lift_f(node)
}
fn listen2(action: Free<R2, i32>) -> Free<R2, (i32, String)> {
	let cell: ListenCell<'static, R2, i32, (i32, String)> = ListenCell {
		action,
		k: Box::new(|pair| pair),
	};
	let coyo: Coyoneda<'static, ListenBrand<R2, i32>, (i32, String)> =
		Coyoneda::<ListenBrand<R2, i32>, _>::lift(cell);
	let node: Node2<(i32, String)> = Coproduct::Inr(Coproduct::Inl(coyo));
	Free::lift_f(node)
}

// `expand`: widen a program from R1 into R2 over the public stepping surface.
// First-order layers are re-injected manually; the higher-order `listen` layer
// is rebuilt by recursively widening both its action and its continuation.
fn expand<A: 'static>(prog: Free<R1, A>) -> Free<R2, A> {
	match prog.resume() {
		Ok(value) => Free::pure(value),
		Err(layer) => match layer {
			Coproduct::Inl(writer_coyo) => match writer_coyo.lower() {
				WriterF::Tell(w, k) => tell2(w).bind(move |()| expand(k(()))),
			},
			Coproduct::Inr(Coproduct::Inl(listen_coyo)) => {
				let ListenCell {
					action,
					k,
				} = listen_coyo.lower();
				let widened_action = expand(action);
				listen2(widened_action).bind(move |pair| expand(k(pair)))
			}
			Coproduct::Inr(Coproduct::Inr(cnil)) => match cnil {},
		},
	}
}

// Interpreter for R2: Writer accumulates and propagates; Listen is elaborated by
// running the action, observing its log, and resuming with `(value, observed)`.
fn run<A: 'static>(prog: Free<R2, A>) -> (A, String) {
	match prog.resume() {
		Ok(value) => (value, String::new()),
		Err(layer) => match layer {
			Coproduct::Inl(writer_coyo) => match writer_coyo.lower() {
				WriterF::Tell(w, k) => {
					let (value, rest) = run(k(()));
					(value, w + &rest)
				}
			},
			Coproduct::Inr(Coproduct::Inl(listen_coyo)) => {
				let ListenCell {
					action,
					k,
				} = listen_coyo.lower();
				let (action_value, observed) = run(action);
				let cont = k((action_value, observed.clone()));
				let (value, rest) = run(cont);
				(value, observed + &rest)
			}
			Coproduct::Inr(Coproduct::Inr(Coproduct::Inl(_other))) => {
				unreachable!("OtherBrand is not used by this program")
			}
			Coproduct::Inr(Coproduct::Inr(Coproduct::Inr(cnil))) => match cnil {},
		},
	}
}

#[test]
fn expand_preserves_listen_semantics_across_row_widening() {
	// Program in the source row R1.
	let action: Free<R1, i32> =
		tell1("first".to_string()).bind(|()| tell1("second".to_string())).bind(|()| Free::pure(40));
	let program: Free<R1, (i32, String)> =
		listen1(action).bind(|(value, observed)| Free::pure((value + 2, observed)));

	// Widen into the larger row R2, then run.
	let widened: Free<R2, (i32, String)> = expand(program);
	let (value, log) = run(widened);

	// The higher-order cell and its sub-program were widened; semantics preserved.
	assert_eq!(value, (42, "firstsecond".to_string()));
	assert_eq!(log, "firstsecond".to_string());
}
