//! FS-1 slice: the `KVStore` effect over a `BTreeMap<&'static str, i32>`.
//!
//! Self-contained per-effect module (the fan-out template): the effect
//! definition and its two smart constructors are emitted by
//! `fp_macros::define_effect!` from the operation signatures below, and the
//! module keeps its bucket A parity test.

fp_macros::define_effect! {
	/// Key-value store over a `BTreeMap<&'static str, i32>`. `lookup` reads the
	/// current value for a key (`None` if absent); `update` writes it: `Some(v)`
	/// inserts or overwrites, `None` deletes.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub(crate) effect KVStore {
		/// Read the current value for `key` (`None` if absent).
		fn lookup(key: &'static str) -> Option<i32>;
		/// Write `value` for `key`: `Some(v)` inserts or overwrites, `None` deletes.
		fn update(key: &'static str, value: Option<i32>) -> ();
	}
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
