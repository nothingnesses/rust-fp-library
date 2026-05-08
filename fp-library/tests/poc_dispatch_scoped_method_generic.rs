#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Prototype for the scoped-handler dispatch shape that will sit beside
// the existing first-order `DispatchHandlers` trait. The important
// compiler question is whether a scoped-handler method can be generic
// over the concrete first-order handler-list type while still requiring
// that type to implement `DispatchHandlers` for the active first-order
// row layer.
//
// This test keeps the prototype local and concrete:
//   - the scoped row contains an Rc-backed `Span` operation.
//   - the first-order row contains two RcCoyoneda variants, so dispatch
//     must recurse through a real `handlers!` cons list before reaching
//     the Option handler.
//   - the scoped handler invokes the inherited first-order handlers from
//     inside its method-generic `dispatch_scoped<FOH>` method.

use fp_library::{
	brands::{
		CNilBrand,
		CoproductBrand,
		IdentityBrand,
		OptionBrand,
		RcBrand,
		RcCoyonedaBrand,
		SpanBrand,
	},
	handlers,
	types::{
		Identity,
		RcCoyoneda,
		effects::{
			coproduct::{
				CNil,
				Coproduct,
			},
			interpreter::DispatchHandlers,
			node::Node,
			rc_run::RcRun,
			span::Span,
		},
	},
};

type FirstRow = CoproductBrand<
	RcCoyonedaBrand<IdentityBrand>,
	CoproductBrand<RcCoyonedaBrand<OptionBrand>, CNilBrand>,
>;
type ScopedRow = CoproductBrand<SpanBrand<RcBrand, &'static str>, CNilBrand>;
type Prog = RcRun<FirstRow, ScopedRow, i32>;
type FirstLayer<'a> = Coproduct<
	RcCoyoneda<'a, IdentityBrand, Prog>,
	Coproduct<RcCoyoneda<'a, OptionBrand, Prog>, CNil>,
>;
type ScopedLayer<'a> = Coproduct<Span<'a, RcBrand, &'static str, Prog>, CNil>;

trait PrototypeDispatchScopedHandlers<'a, ScopedLayer, FirstLayer, NextProgram>
where
	ScopedLayer: 'a,
	FirstLayer: 'a,
	NextProgram: 'a, {
	fn dispatch_scoped<FOH>(
		&self,
		layer: ScopedLayer,
		fo_handlers: &FOH,
	) -> NextProgram
	where
		FOH: DispatchHandlers<'a, FirstLayer, NextProgram>;
}

struct SpanScopedHandlers;

impl<'a> PrototypeDispatchScopedHandlers<'a, ScopedLayer<'a>, FirstLayer<'a>, Prog>
	for SpanScopedHandlers
{
	fn dispatch_scoped<FOH>(
		&self,
		layer: ScopedLayer<'a>,
		fo_handlers: &FOH,
	) -> Prog
	where
		FOH: DispatchHandlers<'a, FirstLayer<'a>, Prog>, {
		match layer {
			Coproduct::Inl(Span::Span {
				tag,
				action,
			}) => {
				assert_eq!(tag, "request");
				assert!(matches!(action(()).peel(), Ok(7)));

				let first_order_layer: FirstLayer<'a> =
					Coproduct::Inr(Coproduct::Inl(RcCoyoneda::lift(Some(RcRun::pure(41)))));
				fo_handlers.dispatch(first_order_layer)
			}
			Coproduct::Inr(cnil) => match cnil {},
		}
	}
}

#[test]
fn method_generic_scoped_dispatch_can_consume_first_order_handlers() {
	let action: Prog = RcRun::pure(7);
	let prog: Prog = RcRun::span::<&'static str, _>("request", action);
	let scoped_layer = match prog.peel() {
		Err(Node::Scoped(layer)) => layer,
		Err(Node::First(_)) => panic!("expected scoped layer"),
		Ok(_) => panic!("expected suspended program"),
	};

	let fo_handlers = handlers! {
		IdentityBrand: |op: Identity<Prog>| op.0,
		OptionBrand: |op: Option<Prog>| op.unwrap_or_else(|| RcRun::pure(-1)),
	};

	let result = SpanScopedHandlers.dispatch_scoped(scoped_layer, &fo_handlers);
	assert!(matches!(result.peel(), Ok(41)));
}
