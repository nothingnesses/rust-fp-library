//! Convenience type aliases for the [kind traits][crate::hkt::kinds].

use crate::{
	hkt::{
		Kind0,
		Kind1,
		Kind2,
	},
	make_type_apply,
};

make_type_apply!(Kind0, Apply0, "*", ());
make_type_apply!(Kind1, Apply1, "* -> *", (A));
make_type_apply!(Kind2, Apply2, "* -> * -> *", (A, B));
