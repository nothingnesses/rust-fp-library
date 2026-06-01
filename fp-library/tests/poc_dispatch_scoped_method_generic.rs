#![cfg(feature = "effects")]
#![expect(clippy::panic, reason = "Tests use panicking operations for brevity and clarity.")]

// Integration test for the scoped-handler dispatch scaffold that sits
// beside the existing first-order `DispatchHandlers` trait. The
// important compiler question is whether a scoped-handler method can
// be generic over the concrete first-order handler-list type while
// still requiring that type to implement `DispatchHandlers` for the
// active first-order row layer.
//
// This test keeps the exercised program concrete:
//   - the scoped row contains an Rc-backed `Span` operation.
//   - the first-order row contains two RcCoyoneda variants, so dispatch
//     must recurse through a real `handlers!` cons list before reaching
//     the Option handler.
//   - the scoped handler invokes the inherited first-order handlers
//     from inside its method-generic `dispatch_scoped_head` method,
//     reached through the production scoped-handler cons list.

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
			interpreter::{
				DispatchHandlers,
				DispatchScopedHandler,
				DispatchScopedHandlers,
			},
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

struct SpanScopedHandlers;

impl<'a> DispatchScopedHandler<'a, Span<'a, RcBrand, &'static str, Prog>, FirstLayer<'a>, Prog>
	for SpanScopedHandlers
{
	fn dispatch_scoped_head(
		&self,
		layer: Span<'a, RcBrand, &'static str, Prog>,
		fo_handlers: &impl DispatchHandlers<'a, FirstLayer<'a>, Prog>,
	) -> Prog {
		match layer {
			Span::Span {
				tag,
				action,
			} => {
				assert_eq!(tag, "request");
				assert!(matches!(action(()).peel(), Ok(7)));

				let first_order_layer: FirstLayer<'a> =
					Coproduct::Inr(Coproduct::Inl(RcCoyoneda::lift(Some(RcRun::pure(41)))));
				fo_handlers.dispatch(first_order_layer)
			}
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
	let scoped_handlers = fp_library::types::effects::handlers::scoped_handlers_ordered()
		.on::<SpanBrand<RcBrand, &'static str>, _>(SpanScopedHandlers)
		.finish();

	let result = scoped_handlers.dispatch_scoped(scoped_layer, &fo_handlers);
	assert!(matches!(result.peel(), Ok(41)));
}
