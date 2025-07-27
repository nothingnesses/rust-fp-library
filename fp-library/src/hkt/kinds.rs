//! Traits representing type-level application.

use crate::{
	hkt::{Apply1, Apply2},
	make_trait_kind,
};

make_trait_kind!(Kind1, Apply1, "* -> *", (A));
make_trait_kind!(Kind2, Apply2, "* -> * -> *", (A, B));

/// Unifies the specialised [`KindN` traits][crate::hkt::kinds]. Represents all kinds.
///
/// `Parameters` should be a tuple containing the types parameters.
/// `Output` represents the reified, concrete type.
pub trait Kind<Parameters> {
	type Output;
}
