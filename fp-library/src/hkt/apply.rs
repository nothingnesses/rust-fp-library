//! Convenience type aliases for the [`Kind` traits][crate::hkt::kinds].

use crate::{
	hkt::{Kind0L0T, Kind0L1T, Kind0L2T, Kind1L0T, Kind1L1T, Kind1L2T},
	make_type_apply,
};

make_type_apply!(Apply0L0T, Kind0L0T, (), (), "*");

make_type_apply!(Apply0L1T, Kind0L1T, (), (A), "* -> *");

make_type_apply!(Apply0L2T, Kind0L2T, (), (A, B), "* -> * -> *");

make_type_apply!(
	Apply1L0T,
	Kind1L0T,
	('a),
	(),
	"' -> *"
);

make_type_apply!(
	Apply1L1T,
	Kind1L1T,
	('a),
	(A),
	"' -> * -> *"
);

make_type_apply!(
	Apply1L2T,
	Kind1L2T,
	('a),
	(A, B),
	"' -> * -> * -> *"
);
