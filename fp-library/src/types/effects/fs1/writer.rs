//! FS-1 slice: the `Writer` effect over a `String` log.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test.

use {
	super::{
		FirstOrder,
		Node,
		OrderOf,
		Row,
	},
	crate::{
		Apply,
		classes::Functor,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::coproduct::Coproduct,
		},
	},
};

/// Writer over a `String` log. `Tell` appends to the log.
pub(crate) struct WriterBrand;
pub(crate) enum WriterF<'a, A> {
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
impl OrderOf for WriterBrand {
	type Order = FirstOrder;
}

pub(crate) fn tell(w: String) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, WriterBrand, ()> =
		Coyoneda::lift(WriterF::Tell(w, Box::new(|u| u)));
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		run,
		tell,
	};

	// Behaviour-parity oracle bucket A (single-effect): successive `tell`s
	// accumulate into the `Writer` log in order.
	#[test]
	fn tell_accumulates_the_log_in_order() {
		let program = tell("Hello".to_string()).bind(|()| tell(" world!".to_string()));
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(()));
		assert_eq!(*fx.log.borrow(), "Hello world!");
	}
}
