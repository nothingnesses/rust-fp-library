// Compile-fail (trybuild) companion to poc_fs_brand_dispatch.rs (foundation
// sweep POC-2). A row containing an effect with no handler in the list must be
// rejected, and the error should name the missing brand. The brand-keyed
// dispatch machinery is duplicated here because trybuild compiles this as a
// standalone program.

use {
	fp_library::types::effects::coproduct::{
		CNil,
		Coproduct,
	},
	std::marker::PhantomData,
};

trait HasBrand {
	type Brand;
}

struct GetBrand;
struct LogBrand;

struct GetOp(i32);
impl HasBrand for GetOp {
	type Brand = GetBrand;
}
struct LogOp(i32);
impl HasBrand for LogOp {
	type Brand = LogBrand;
}

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

struct HandlersNil;
struct HandlersCons<H, T> {
	head: H,
	tail: T,
}

struct Here;
struct There<I>(PhantomData<I>);

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

type Row = Coproduct<GetOp, Coproduct<LogOp, CNil>>;

fn main() {
	// The handler list covers Get but not Log, while the row contains Log.
	let handlers = HandlersCons {
		head: Handler::<GetBrand, _>::new(|op: GetOp| op.0 + 1),
		tail: HandlersNil,
	};
	let log: Row = Coproduct::inject(LogOp(10));
	let _out: i32 = log.dispatch(&handlers);
}
