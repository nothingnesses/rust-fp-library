//! POC-5 (foundation sweep, Tier D): elaborate a result-shape-changing
//! higher-order effect (`listen`) as an in-row cell, via an interpret pass,
//! without boundary frames.
//!
//! Charter question: can a result-shape-changing higher-order effect (`listen`,
//! whose operation result `(RAction, W)` differs from its action result
//! `RAction`, the exact case that forced today's boundary/carrier split) live in
//! the unified row as a single cell and be elaborated by a plain interpret pass,
//! with no boundary frames, no result-polymorphic protocol traits, and no scoped
//! row? The target property, restated from the Writer `listen` suite (setup step
//! S5): `listen(tell("first") >> tell("second") >> pure(40))`, then
//! `(value, observed) -> (value + 2, observed)`, yields `(42, "firstsecond")`,
//! and the two tells still propagate to the outer Writer (observed log
//! "firstsecond"). This mirrors the boundary-split suite's
//! `(42, "firstsecond")` result.
//!
//! Mechanism (the adopted Approach (c)): `listen` is an in-row higher-order cell
//! whose action result is a brand type parameter, `ListenBrand<RAction>` with
//! `Of<'a, Next> = ListenCell<'a, RAction, Next>`. The cell stores the action
//! sub-program `Free<Row, RAction>` and a continuation `k: (RAction, W) -> Next`.
//! Carrying `RAction` as a concrete type parameter keeps the hole-`Functor`
//! object-safe (post-composing onto `k` names `RAction`), which a fully
//! existential `RAction` could not, because `Coyoneda::lower` requires `Functor`
//! and a trait object cannot carry the generic method an erased `RAction` would
//! need. The interpret pass elaborates `listen` by running the action
//! sub-program, capturing its accumulated log, feeding `(value, observed)` into
//! `k`, and re-emitting the action's log outward so the outer Writer still sees
//! it. This is heftia's `runListen` semantics expressed over the unified row.
//!
//! Harness (setup steps S2, S3): public-API integration test over the real
//! `Free` substrate with a uniformly `Coyoneda`-wrapped `Writer` + `Listen` row.
//! Throwaway spike code.

#![cfg(feature = "effects")]

use {
	fp_library::{
		Apply,
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		classes::Functor,
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

// First-order Writer effect over a String log.
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

// Higher-order Listen effect. The operation result is `(RAction, String)` (the
// action's result paired with the log the action produced), which differs from
// the action's own result `RAction`. The cell carries the action sub-program and
// the continuation that consumes the result-shape-changed `(RAction, String)`.
// `RAction` is a brand type parameter (the adopted Approach (c)).
struct ListenBrand<RAction>(PhantomData<RAction>);
struct ListenCell<'a, RAction: 'static, Next> {
	action: Free<Row, RAction>,
	k: Box<dyn FnOnce((RAction, String)) -> Next + 'a>,
}
impl_kind! {
	impl<RAction: 'static> for ListenBrand<RAction> {
		type Of<'a, Next: 'a>: 'a = ListenCell<'a, RAction, Next>;
	}
}
impl<RAction: 'static> Functor for ListenBrand<RAction> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		// Post-compose `f` onto the hole (continuation) position. `RAction` is a
		// concrete type here, so this is object-safe; nothing reaches into a
		// trait object with a generic method.
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

// The unified row: a first-order Writer and a higher-order Listen in one row,
// each Coyoneda-wrapped. `Listen` is instantiated at action result `i32` for
// this case.
type Row = CoproductBrand<
	CoyonedaBrand<WriterBrand>,
	CoproductBrand<CoyonedaBrand<ListenBrand<i32>>, CNilBrand>,
>;

type Node<A> = Apply!(<Row as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

fn pure<A: 'static>(value: A) -> Free<Row, A> {
	Free::pure(value)
}

fn tell(w: String) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, WriterBrand, ()> =
		Coyoneda::<WriterBrand, _>::lift(WriterF::Tell(w, Box::new(|u| u)));
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

// `listen(action)` injects the higher-order cell. The cell's continuation starts
// as the identity over `(i32, String)`; any `bind` after `listen` accumulates
// into it (via the Coyoneda map, recovered by `lower` in the interpreter).
fn listen(action: Free<Row, i32>) -> Free<Row, (i32, String)> {
	let cell: ListenCell<'static, i32, (i32, String)> = ListenCell {
		action,
		k: Box::new(|pair| pair),
	};
	let coyo: Coyoneda<'static, ListenBrand<i32>, (i32, String)> =
		Coyoneda::<ListenBrand<i32>, _>::lift(cell);
	let node: Node<(i32, String)> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

// The interpret pass. Returns the program's value and the log it accumulated.
// Writer prepends each tell to the log; Listen is elaborated by running the
// action sub-program, observing its log, feeding `(value, observed)` into the
// continuation, and re-emitting the action's log into the outer total.
fn run<A: 'static>(program: Free<Row, A>) -> (A, String) {
	match program.resume() {
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
				// Elaborate: run the action, observe its log.
				let (action_value, observed) = run(action);
				// Feed the result-shape-changed `(value, observed)` to the
				// continuation, then re-emit the action's log outward.
				let cont = k((action_value, observed.clone()));
				let (value, rest) = run(cont);
				(value, observed + &rest)
			}
			Coproduct::Inr(Coproduct::Inr(cnil)) => match cnil {},
		},
	}
}

#[test]
fn listen_reproduces_the_result_shape_changing_case() {
	// listen(tell("first") >> tell("second") >> pure(40)), then
	// (value, observed) -> (value + 2, observed).
	let action: Free<Row, i32> =
		tell("first".to_string()).bind(|()| tell("second".to_string())).bind(|()| pure(40));
	let program: Free<Row, (i32, String)> =
		listen(action).bind(|(value, observed)| pure((value + 2, observed)));

	let (value, log) = run(program);

	// Result-shape change reproduced: the operation yields `(42, "firstsecond")`
	// (matches the boundary-split suite), and the two tells still propagated to
	// the outer Writer (total log "firstsecond").
	assert_eq!(value, (42, "firstsecond".to_string()));
	assert_eq!(log, "firstsecond".to_string());
}

#[test]
fn listen_observes_only_its_action_while_the_log_propagates() {
	// tell("pre") >> listen(tell("a") >> pure(7)).
	let program: Free<Row, (i32, String)> =
		tell("pre".to_string()).bind(|()| listen(tell("a".to_string()).bind(|()| pure(7))));

	let (value, log) = run(program);

	// `observed` is scoped to the action ("a"), not the surrounding program; the
	// outer tell ("pre") is not observed by listen but is still in the total log.
	assert_eq!(value, (7, "a".to_string()));
	assert_eq!(log, "prea".to_string());
}
