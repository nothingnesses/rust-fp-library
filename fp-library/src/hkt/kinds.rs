//! Traits representing type-level application.

use crate::make_trait_kind;

make_trait_kind!(Kind0L1T, (), (A), "* -> *");

make_trait_kind!(Kind0L2T, (), (A, B), "* -> * -> *");

make_trait_kind!(
	Kind1L2T,
	('a),
	(A, B),
	"' -> * -> * -> *"
);
