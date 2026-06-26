//! FS-1 slice: the `Reader` effect over an `i32` environment.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructor, and its bucket A parity test. The parity
//! case is a higher-order composition (Reader feeding State under a Catch), so
//! it imports the sibling constructors it composes with.

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

/// Reader over an `i32` environment. `Ask` reads the environment.
pub(crate) struct ReaderBrand;
pub(crate) enum ReaderF<'a, A> {
	Ask(Box<dyn FnOnce(i32) -> A + 'a>),
}
impl_kind! {
	impl for ReaderBrand {
		type Of<'a, A: 'a>: 'a = ReaderF<'a, A>;
	}
}
impl Functor for ReaderBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ReaderF::Ask(k) => ReaderF::Ask(Box::new(move |e| f(k(e)))),
		}
	}
}
impl OrderOf for ReaderBrand {
	type Order = FirstOrder;
}

pub(crate) fn ask() -> Free<Row, i32> {
	let coyo: Coyoneda<'static, ReaderBrand, i32> = Coyoneda::lift(ReaderF::Ask(Box::new(|e| e)));
	let node: Node<i32> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use crate::types::{
		Free,
		effects::fs1::{
			Fixture,
			Row,
			ask,
			catch,
			get,
			put,
			run,
			throw,
		},
	};

	// Behaviour-parity oracle bucket A: Reader composes with State and Catch.
	// `ask()` supplies the environment, which is written into State (as its
	// parity), and a caught throw leaves the write intact.
	#[test]
	fn reader_composes_with_state_and_catch() {
		let program: Free<Row, bool> = ask().bind(|env| {
			let parity = env % 2 == 0;
			catch(put(parity).bind(|()| throw::<()>()), || Free::pure(())).bind(|()| get())
		});

		// env = 4 is even, so the State write is `true` and survives the catch.
		let fx = Fixture::with_env(4);
		let result = run(program, &fx.handlers());

		assert_eq!(result, Ok(true));
		assert!(fx.state.get());
	}
}
