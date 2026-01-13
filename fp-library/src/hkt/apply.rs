//! Convenience type aliases for the [`Kind` traits][crate::hkt::kinds].
//!
//! The naming convention used by these aliases is `ApplyNLMT` where `N`
//! represents the number of lifetimes and `M` represents the number of
//! generic types.
//!
//! If a [`Brand`][crate::brands] `FooBrand` for concrete type `Foo<A>`
//! implements the [`Kind0L1T`] trait, then `Apply0L1T<FooBrand, ()>`
//! represents `Foo<()>`.

use crate::{
	hkt::{Kind0L1T, Kind0L2T, Kind1L0T, Kind1L1T, Kind1L2T},
	make_type_apply,
};

make_type_apply!(Apply0L1T, Kind0L1T, (), (A), "Alias for [types][crate::types] of kind `* -> *`.");

make_type_apply!(
	Apply0L2T,
	Kind0L2T,
	(),
	(A, B),
	"Alias for [types][crate::types] of kind `* -> * -> *`."
);

make_type_apply!(
	Apply1L0T,
	Kind1L0T,
	('a),
	(),
	"Alias for [types][crate::types] of kind `' -> *`."
);

make_type_apply!(
	Apply1L2T,
	Kind1L2T,
	('a),
	(A, B),
	"Alias for [types][crate::types] of kind `' -> * -> * -> *`."
);

make_type_apply!(
	Apply1L1T,
	Kind1L1T,
	('a),
	(A: 'a),
	"Alias for [types][crate::types] of kind `' -> * -> *`."
);
