//! POC-2 (foundation sweep, Tier A): brand-keyed handler dispatch.
//!
//! Charter question: can handlers be dispatched by effect brand via type-level
//! search over a coproduct, so handler-list order is irrelevant? This is the
//! Axis 3 / remediation-item-8 alternative to today's positional cons-list
//! dispatch (which forces the row and handler macros to sort by a syntactic
//! key, the spelling footgun).
//!
//! Harness (charter setup step S2): standalone from the effects subsystem,
//! over the real `frunk_core` coproduct value type (re-exported behind the
//! `effects` feature). Throwaway spike code; it need only compile and pass its
//! assertions. The missing-handler error case lives in
//! `tests/ui/poc_fs_brand_dispatch_missing.rs`.

#![cfg(feature = "effects")]

use {
	fp_library::types::effects::coproduct::{
		CNil,
		Coproduct,
	},
	std::marker::PhantomData,
};

// Each effect operation value knows its effect brand.
trait HasBrand {
	type Brand;
}

// Effects (brands) and their operation payloads.
struct GetBrand;
struct AskBrand;
struct LogBrand;

struct GetOp(i32);
impl HasBrand for GetOp {
	type Brand = GetBrand;
}
struct AskOp(i32);
impl HasBrand for AskOp {
	type Brand = AskBrand;
}
struct LogOp(i32);
impl HasBrand for LogOp {
	type Brand = LogBrand;
}

// A brand-tagged handler: the closure `F` plus a phantom of the effect brand it
// handles.
struct Handler<EBrand, F> {
	run: F,
	brand: PhantomData<EBrand>,
}
impl<EBrand, F> Handler<EBrand, F> {
	fn new(run: F) -> Self {
		Handler {
			run,
			brand: PhantomData,
		}
	}
}

// Handler list (heterogeneous cons-list; order is the user's writing order).
struct HandlersNil;
struct HandlersCons<H, T> {
	head: H,
	tail: T,
}

// Selector position witnesses (the frunk `Here`/`There` index trick), so the
// two `HandleByBrand` impls do not overlap even when a brand could appear at
// the head.
struct Here;
struct There<I>(PhantomData<I>);

// Brand-keyed dispatch into the handler list: find the handler for `EBrand`
// regardless of its position, and run it on `op`. The position `Idx` is
// inferred. `on_unimplemented` shapes the missing-handler error to name the
// brand.
#[diagnostic::on_unimplemented(
	message = "no handler for the effect `{EBrand}` in the handler list",
	label = "the row contains `{EBrand}`, but the handler list has no entry for it",
	note = "brand-keyed dispatch searches the handler list by effect brand; add a `Handler::<{EBrand}, _>` entry (its position does not matter)"
)]
trait HandleByBrand<EBrand, Op, Out, Idx> {
	fn handle(
		&self,
		op: Op,
	) -> Out;
}

// Head matches the wanted brand.
impl<EBrand, Op, Out, F, T> HandleByBrand<EBrand, Op, Out, Here>
	for HandlersCons<Handler<EBrand, F>, T>
where
	F: Fn(Op) -> Out,
{
	fn handle(
		&self,
		op: Op,
	) -> Out {
		(self.head.run)(op)
	}
}

// Head does not match; search the tail.
impl<EBrand, Op, Out, HHead, T, Idx> HandleByBrand<EBrand, Op, Out, There<Idx>>
	for HandlersCons<HHead, T>
where
	T: HandleByBrand<EBrand, Op, Out, Idx>,
{
	fn handle(
		&self,
		op: Op,
	) -> Out {
		self.tail.handle(op)
	}
}

// Walk the row coproduct; dispatch each active operation to its brand's handler,
// found by brand search rather than by position. `Indices` is a type-level list
// of one selector position per row arm (the frunk `Sculptor` pattern): making
// the per-arm indices trait parameters keeps them constrained, where a single
// free index parameter would be unconstrained (E0207). All of `Indices` is
// inferred at the call site.
trait Dispatch<Handlers, Out, Indices> {
	fn dispatch(
		self,
		handlers: &Handlers,
	) -> Out;
}
impl<Handlers, Out> Dispatch<Handlers, Out, ()> for CNil {
	fn dispatch(
		self,
		_handlers: &Handlers,
	) -> Out {
		match self {}
	}
}
impl<HeadOp, Tail, Handlers, Out, HeadIdx, TailIdx> Dispatch<Handlers, Out, (HeadIdx, TailIdx)>
	for Coproduct<HeadOp, Tail>
where
	HeadOp: HasBrand,
	Handlers: HandleByBrand<HeadOp::Brand, HeadOp, Out, HeadIdx>,
	Tail: Dispatch<Handlers, Out, TailIdx>,
{
	fn dispatch(
		self,
		handlers: &Handlers,
	) -> Out {
		match self {
			Coproduct::Inl(op) =>
				<Handlers as HandleByBrand<HeadOp::Brand, HeadOp, Out, HeadIdx>>::handle(
					handlers, op,
				),
			Coproduct::Inr(rest) => rest.dispatch(handlers),
		}
	}
}

type Row = Coproduct<GetOp, Coproduct<AskOp, Coproduct<LogOp, CNil>>>;

#[test]
fn handler_order_does_not_change_the_result() {
	// Order A: Get, Ask, Log.
	let handlers_a = HandlersCons {
		head: Handler::<GetBrand, _>::new(|op: GetOp| op.0 + 1),
		tail: HandlersCons {
			head: Handler::<AskBrand, _>::new(|op: AskOp| op.0 * 2),
			tail: HandlersCons {
				head: Handler::<LogBrand, _>::new(|op: LogOp| op.0 - 1),
				tail: HandlersNil,
			},
		},
	};
	// Order B: Log, Ask, Get (reversed).
	let handlers_b = HandlersCons {
		head: Handler::<LogBrand, _>::new(|op: LogOp| op.0 - 1),
		tail: HandlersCons {
			head: Handler::<AskBrand, _>::new(|op: AskOp| op.0 * 2),
			tail: HandlersCons {
				head: Handler::<GetBrand, _>::new(|op: GetOp| op.0 + 1),
				tail: HandlersNil,
			},
		},
	};

	// Same row value, both handler orders, same result: dispatch is by brand,
	// not by position.
	let make_get = || -> Row { Coproduct::inject(GetOp(10)) };
	let out_a: i32 = make_get().dispatch(&handlers_a);
	let out_b: i32 = make_get().dispatch(&handlers_b);
	assert_eq!(out_a, 11);
	assert_eq!(out_b, 11);
}

#[test]
fn every_variant_routes_to_its_brand_handler() {
	let handlers = HandlersCons {
		head: Handler::<LogBrand, _>::new(|op: LogOp| op.0 - 1),
		tail: HandlersCons {
			head: Handler::<GetBrand, _>::new(|op: GetOp| op.0 + 1),
			tail: HandlersCons {
				head: Handler::<AskBrand, _>::new(|op: AskOp| op.0 * 2),
				tail: HandlersNil,
			},
		},
	};
	let get: Row = Coproduct::inject(GetOp(10));
	let ask: Row = Coproduct::inject(AskOp(10));
	let log: Row = Coproduct::inject(LogOp(10));
	// No turbofish on the dispatch calls: the brand, op, and index are inferred.
	assert_eq!(get.dispatch(&handlers), 11);
	assert_eq!(ask.dispatch(&handlers), 20);
	assert_eq!(log.dispatch(&handlers), 9);
}

// Compile-fail evidence that a row whose effect has no handler is rejected with
// an error naming the brand lives in the trybuild case below.
#[test]
fn missing_handler_is_rejected_with_readable_error() {
	let t = trybuild::TestCases::new();
	t.compile_fail("tests/ui/poc_fs_brand_dispatch_missing.rs");
}
