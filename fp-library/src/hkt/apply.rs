//! Convenience type aliases for the [kind traits][crate::hkt::kinds].

use crate::{
	hkt::{Kind, Kind1, Kind2},
	make_type_apply,
};

make_type_apply!(Kind1, Apply1, "* -> *", (A));
make_type_apply!(Kind2, Apply2, "* -> * -> *", (A, B));

/// Unifies the specialised [`Apply`][crate::hkt::apply] aliases.
///
/// `Brand` should be the type representing the higher-kinded form of another type.
/// `Parameters` should be a tuple containing the types parameters.
pub type Apply<Brand, Parameters> = <Brand as Kind<Parameters>>::Output;
