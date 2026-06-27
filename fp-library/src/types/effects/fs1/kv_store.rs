//! FS-1 slice: the `KVStore` effect over a `BTreeMap<&'static str, i32>`.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition, its smart constructors, and its bucket A parity test.

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

/// Key-value store over a `BTreeMap<&'static str, i32>`. `Lookup` reads the
/// current value for a key (`None` if absent); `Update` writes it: `Some(v)`
/// inserts or overwrites, `None` deletes.
pub(crate) struct KVStoreBrand;
pub(crate) enum KVStoreF<'a, A> {
	Lookup(&'static str, Box<dyn FnOnce(Option<i32>) -> A + 'a>),
	Update(&'static str, Option<i32>, Box<dyn FnOnce(()) -> A + 'a>),
}
impl_kind! {
	impl for KVStoreBrand {
		type Of<'a, A: 'a>: 'a = KVStoreF<'a, A>;
	}
}
impl Functor for KVStoreBrand {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			KVStoreF::Lookup(key, k) => KVStoreF::Lookup(key, Box::new(move |v| f(k(v)))),
			KVStoreF::Update(key, value, k) =>
				KVStoreF::Update(key, value, Box::new(move |u| f(k(u)))),
		}
	}
}
impl OrderOf for KVStoreBrand {
	type Order = FirstOrder;
}

pub(crate) fn lookup(key: &'static str) -> Free<Row, Option<i32>> {
	let coyo: Coyoneda<'static, KVStoreBrand, Option<i32>> =
		Coyoneda::lift(KVStoreF::Lookup(key, Box::new(|v| v)));
	let node: Node<Option<i32>> = Coproduct::inject(coyo);
	Free::lift_f(node)
}
pub(crate) fn update(
	key: &'static str,
	value: Option<i32>,
) -> Free<Row, ()> {
	let coyo: Coyoneda<'static, KVStoreBrand, ()> =
		Coyoneda::lift(KVStoreF::Update(key, value, Box::new(|u| u)));
	let node: Node<()> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

#[cfg(test)]
mod tests {
	use {
		crate::types::effects::fs1::{
			Fixture,
			lookup,
			run,
			update,
		},
		std::collections::BTreeMap,
	};

	// Behaviour-parity oracle bucket A (single-effect): over the initial store
	// `{"a": 7, "b": 3}`, `lookup("a")` reads `Some(7)`, `update("a", Some(9))`
	// overwrites, the next `lookup("a")` reads `Some(9)`, and `update("a", None)`
	// deletes the key. The observed pair is `(Some(7), Some(9))` and the final
	// store is `{"b": 3}`, matching the dual-row `BTreeMap` runner.
	#[test]
	fn lookup_update_observes_then_deletes() {
		let program = lookup("a").bind(|before| {
			update("a", Some(9)).bind(move |()| {
				lookup("a").bind(move |after_insert| {
					update("a", None).map(move |()| (before, after_insert))
				})
			})
		});
		let fx = Fixture::with_kv_store(BTreeMap::from([("a", 7), ("b", 3)]));
		assert_eq!(run(program, &fx.handlers()), Ok((Some(7), Some(9))));
		assert_eq!(*fx.kv_store.borrow(), BTreeMap::from([("b", 3)]));
	}
}
