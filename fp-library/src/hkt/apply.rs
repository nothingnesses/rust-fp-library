//! Convenience type aliases for the [`Kind` traits][crate::hkt::kinds].
//!
//! The naming convention used by these aliases is `ApplyNLMT` where `N`
//! represents the number of lifetimes and `T` represents the number of
//! generic types.
//!
//! If a [`Brand`][crate::brands] `FooBrand` for concrete type `Foo<A>`
//! implements the [`Kind0L1T`] trait, then `Apply0L1T<FooBrand, ()>`
//! represents `Foo<()>`.

use crate::{
	hkt::{Kind0L1T, Kind0L2T, Kind1L2T},
	make_type_apply,
};

make_type_apply!(Apply0L1T, Kind0L1T, (), (A), "* -> *");

make_type_apply!(Apply0L2T, Kind0L2T, (), (A, B), "* -> * -> *");

make_type_apply!(
	Apply1L2T,
	Kind1L2T,
	('a),
	(A, B),
	"' -> * -> * -> *"
);
