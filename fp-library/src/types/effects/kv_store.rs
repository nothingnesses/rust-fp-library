//! Key-value-store first-order effect type.
//!
//! `KVStore<'a, P, K, V, A>` provides lookup and update operations for
//! a key-value store. The standard W11 runner will use
//! `std::collections::BTreeMap<K, V>` for deterministic examples and
//! tests, while users can reinterpret the two primitive operations to a
//! custom store when needed.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				BoxBrand,
				BoxKVStoreBrand,
				KVStoreBrand,
				SendKVStoreBrand,
			},
			classes::{
				Functor,
				Pointer,
				RefCountedPointer,
				SendFunctor,
				SendRefCountedPointer,
				ToDynCloneFn,
				ToDynFnOnce,
				ToDynSendFn,
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::*,
	};

	define_effect! {
		effect KVStore;
	}
}

pub use inner::*;
